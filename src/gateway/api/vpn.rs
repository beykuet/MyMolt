// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, post},
    Router,
};
use crate::gateway::AppState;
use super::auth::AuthenticatedUser;
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/vpn/peers", post(add_peer).get(list_peers))
        .route("/api/vpn/peers/{id}", delete(delete_peer))
}

#[derive(Deserialize)]
struct AddPeerRequest {
    name: String,
}

async fn add_peer(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<AddPeerRequest>,
) -> impl IntoResponse {
    // RBAC: Only Root can manage VPN
    if user.role != crate::identity::UserRole::Root {
        let body: serde_json::Value = serde_json::json!({"error": "Only Root can manage VPN"});
        return (StatusCode::FORBIDDEN, Json(body)).into_response();
    }

    // Rate limiting: 5 VPN operations per minute
    if !state.rate_limiter.allow_vpn("vpn_global") {
        let body: serde_json::Value = serde_json::json!({"error": "Too many VPN operations. Please wait."});
        return (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response();
    }

    match state.vpn_manager.add_peer(&payload.name).await {
        Ok((peer, client_conf)) => {
            let response: serde_json::Value = serde_json::json!({
                "peer": peer,
                "config_file": client_conf
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
             let msg = format!("{}", e);
             let body: serde_json::Value = serde_json::json!({"error": msg});
             (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
        }
    }
}

async fn list_peers(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // RBAC: Only Root can manage VPN
    if user.role != crate::identity::UserRole::Root {
        let body: serde_json::Value = serde_json::json!({"error": "Only Root can manage VPN"});
        return (StatusCode::FORBIDDEN, Json(body)).into_response();
    }

    match state.vpn_manager.list_peers() {
        Ok(peers) => {
             let body: serde_json::Value = serde_json::json!(peers);
             (StatusCode::OK, Json(body)).into_response()
        },
        Err(_) => {
             let body: serde_json::Value = serde_json::json!([]);
             (StatusCode::OK, Json(body)).into_response()
        }
    }
}

async fn delete_peer(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // RBAC: Only Root can manage VPN
    if user.role != crate::identity::UserRole::Root {
        let body: serde_json::Value = serde_json::json!({"error": "Only Root can manage VPN"});
        return (StatusCode::FORBIDDEN, Json(body)).into_response();
    }

    match state.vpn_manager.delete_peer(&id) {
        Ok(_) => {
             let body: serde_json::Value = serde_json::json!({"status": "deleted"});
             (StatusCode::OK, Json(body)).into_response()
        },
        Err(e) => {
             let msg = format!("{}", e);
             let body: serde_json::Value = serde_json::json!({"error": msg});
             (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
        }
    }
}
