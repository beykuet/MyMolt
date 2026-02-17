use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, post},
    Router,
};
use crate::gateway::AppState;
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/vpn/setup", post(setup_vpn))
        .route("/api/vpn/peers", post(add_peer).get(list_peers))
        .route("/api/vpn/peers/{id}", delete(delete_peer))
}

#[derive(Deserialize)]
struct SetupRequest {
    interface: String,
    port: u16,
    cidr: String,
}

async fn setup_vpn(
    State(state): State<AppState>,
    Json(payload): Json<SetupRequest>,
) -> impl IntoResponse {
    match state.vpn_manager.init_server(&payload.interface, payload.port, &payload.cidr) {
        Ok(config) => {
             let body: serde_json::Value = serde_json::json!(config);
             (StatusCode::OK, Json(body)).into_response()
        },
        Err(e) => {
             let msg = format!("{}", e);
             let body: serde_json::Value = serde_json::json!({"error": msg});
             (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
        }
    }
}

#[derive(Deserialize)]
struct AddPeerRequest {
    name: String,
}

async fn add_peer(
    State(state): State<AppState>,
    Json(payload): Json<AddPeerRequest>,
) -> impl IntoResponse {
    match state.vpn_manager.add_peer(&payload.name) {
        Ok((peer, client_conf)) => {
            let response: serde_json::Value = serde_json::json!({
                "peer": peer,
                "client_config": client_conf
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

async fn list_peers(State(state): State<AppState>) -> impl IntoResponse {
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
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
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

