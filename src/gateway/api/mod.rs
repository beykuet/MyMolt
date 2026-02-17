pub mod types;
pub mod handlers;
pub mod ws;
pub mod auth;
pub mod vpn;

use axum::{Router, routing::get};
use crate::gateway::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(handlers::router())
        .merge(vpn::router())
        .route("/ws/chat", get(ws::ws_handler))
}
