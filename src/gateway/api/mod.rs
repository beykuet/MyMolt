// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

pub mod admin;
pub mod auth;
pub mod browse;
pub mod family;
pub mod handlers;
pub mod mcp;
pub mod proxy;
pub mod security;
pub mod types;
pub mod vpn;
pub mod ws;

use crate::gateway::AppState;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(handlers::router())
        .merge(vpn::router())
        .merge(admin::router())
        .merge(proxy::router())
        .merge(family::router())
        .merge(mcp::router())
        .merge(security::router())
        .merge(browse::router())
        .route("/ws/chat", get(ws::ws_handler))
}
