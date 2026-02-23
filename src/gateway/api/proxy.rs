// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use crate::gateway::AppState;
use reqwest::Client;
use std::sync::OnceLock;

static CLIENT: OnceLock<Client> = OnceLock::new();

fn get_client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Failed to build reqwest client for proxy")
    })
}

/// Maximum body size for Hoodik proxy routes (100 MB for file uploads).
const PROXY_MAX_BODY_SIZE: usize = 100 * 1024 * 1024;

pub fn router() -> Router<AppState> {
    Router::new()
        // Capture everything under /system
        .route("/system", any(proxy_handler))
        .route("/system/{*path}", any(proxy_handler))
        // Aliases for user convenience (if they access /hoodik)
        .route("/hoodik", any(proxy_handler))
        .route("/hoodik/{*path}", any(proxy_handler))
        // Override the global 64KB limit for these routes (Hoodik file uploads)
        .layer(DefaultBodyLimit::max(PROXY_MAX_BODY_SIZE))
}

async fn proxy_handler(State(_state): State<AppState>, req: Request) -> impl IntoResponse {
    let path = req.uri().path();
    let path_query = req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or(path);

    // Determine target path by stripping known prefixes
    let target_path = if let Some(stripped) = path_query.strip_prefix("/system") {
        stripped
    } else if let Some(stripped) = path_query.strip_prefix("/hoodik") {
        stripped
    } else {
        path_query
    };
    
    // Normalize: ensure it starts with / if strictly needed, or empty if root.
    // Hoodik root is http://127.0.0.1:3001/
    // If target_path is empty, request "http://...:3001" which reqwest handles as "/"?
    // Actually reqwest needs the slash if path is empty?
    let target_path = if target_path.is_empty() { "/" } else { target_path };

    let target_url = format!("http://127.0.0.1:3001{}", target_path);

    // tracing::debug!("Proxying: {} -> {}", path_query, target_url);

    let method = req.method().clone();
    let headers = req.headers().clone();
    
    // Buffer request body to keep things simple and robust
    // MAX limit is already applied by gateway middleware (64KB), but Hoodik uploads might need more?
    // The middleware in gateway/mod.rs sets 64KB limit: .layer(RequestBodyLimitLayer::new(MAX_BODY_SIZE))
    // This will break Hoodik file uploads (which can be large).
    // We might need to increase the limit for specific routes or globally?
    // For now, respect the limit.
    let body_bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(b) => b,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    
    let mut request_builder = get_client()
        .request(method, &target_url)
        .body(body_bytes);

    for (key, value) in &headers {
        if key.as_str() != "host" {
            request_builder = request_builder.header(key, value);
        }
    }

    match request_builder.send().await {
        Ok(res) => {
            let status = res.status();
            let headers = res.headers().clone();
            let body = res.bytes_stream(); 
            
            let mut response_builder = Response::builder().status(status);
            for (key, value) in &headers {
                response_builder = response_builder.header(key, value);
            }
            
            response_builder
                .body(Body::from_stream(body))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => {
            tracing::error!("Hoodik proxy error: {}", e);
            StatusCode::BAD_GATEWAY.into_response()
        }, 
    }
}
