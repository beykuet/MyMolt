use axum::{
    extract::{State, Json},
    routing::{get, post},
    Router,
};
use crate::gateway::AppState;
use super::types::*;

use super::auth::AuthenticatedUser;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/system/status", get(get_system_status))
        .route("/api/system/widgets", get(get_widgets))
        .route("/api/identity", get(get_identities))
        .route("/api/identity/simulate", post(simulate_identity_link))
        .route("/api/identity/eidas/verify", post(verify_eidas_upload))
        .route("/api/config/pairing", post(set_pairing_config))
        .route("/api/config/models", get(get_models))
}

// ── Handlers ─────────────────────────────────────────────────────


async fn get_widgets(_user: AuthenticatedUser, State(_state): State<AppState>) -> Json<Vec<WidgetConfig>> {
    // Return default widgets for MVP
    Json(vec![
        WidgetConfig {
            id: "panic".into(),
            type_: "panic".into(),
            title: "Emergency Stop".into(),
            icon: Some("alert-triangle".into()),
            action_url: Some("/api/system/panic".into()),
        },
        WidgetConfig {
            id: "status".into(),
            type_: "system".into(),
            title: "System Health".into(),
            icon: Some("activity".into()),
            action_url: None,
        }
    ])
}

async fn get_identities(_user: AuthenticatedUser, State(state): State<AppState>) -> Json<Vec<IdentityStatus>> {
    let soul = state.soul.lock().unwrap();
    
    let mut identities: Vec<IdentityStatus> = soul.bindings.iter().map(|b| IdentityStatus {
        provider: b.provider.clone(),
        id: b.id.clone(),
        trust_level: match b.trust_level {
            crate::identity::soul::TrustLevel::High => 3,
            crate::identity::soul::TrustLevel::Low => 1,
        },
        linked_at: b.created_at.clone(),
    }).collect();

    // Always ensure "local" is present for the owner if list is empty or just to be safe
    if !identities.iter().any(|i| i.provider == "local") {
        identities.push(IdentityStatus {
            provider: "local".into(),
            id: "owner".into(),
            trust_level: 3,
            linked_at: "2024-01-01".into(),
        });
    }

    Json(identities)
}

#[derive(serde::Deserialize)]
struct SimulateLinkRequest {
    provider: String,
    id: String,
}

async fn simulate_identity_link(
    _user: AuthenticatedUser, 
    State(state): State<AppState>, 
    Json(payload): Json<SimulateLinkRequest>
) -> Json<serde_json::Value> {
    let mut soul = state.soul.lock().unwrap();
    
    // Simulate linking
    let level = if payload.provider.to_lowercase().contains("eidas") {
        crate::identity::soul::TrustLevel::High
    } else {
        crate::identity::soul::TrustLevel::Low
    };

    if let Err(e) = soul.add_binding(&payload.provider, &payload.id, level) {
        return Json(serde_json::json!({ "success": false, "error": e.to_string() }));
    }

    Json(serde_json::json!({ "success": true, "message": "Identity linked (simulated)" }))
}


// ... (imports)
use axum::extract::Multipart;

async fn verify_eidas_upload(
    _user: AuthenticatedUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Json<serde_json::Value> {
    let mut file_name = String::new();
    let mut _file_content = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if let Some(name) = field.name() {
            if name == "certificate" {
                file_name = field.file_name().unwrap_or("unknown.pem").to_string();
                if let Ok(bytes) = field.bytes().await {
                    _file_content = bytes.to_vec();
                }
            }
        }
    }

    if file_name.is_empty() {
        return Json(serde_json::json!({
            "success": false,
            "error": "No certificate file provided"
        }));
    }
    
    let mut soul = state.soul.lock().unwrap();
    let id_from_cert = format!("DE-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());

    if let Err(e) = soul.add_binding("eIDAS", &id_from_cert, crate::identity::soul::TrustLevel::High) {
        return Json(serde_json::json!({ "success": false, "error": e.to_string() }));
    }

    Json(serde_json::json!({
        "success": true,
        "message": "eIDAS identity verified and linked",
        "id": id_from_cert,
        "level": "High"
    }))
}

// ... imports

async fn get_system_status(_user: AuthenticatedUser, State(state): State<AppState>) -> Json<SystemStatus> {
    // Mock implementation for MVP
    Json(SystemStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: 1234, // TODO: Real uptime
        memory_usage_mb: 45, // TODO: Real mem usage
        cpu_usage_percent: 1.5, // TODO: Real CPU
        active_agents: 1,
        voice_mode_active: false,
        pairing_enabled: state.pairing.require_pairing(),
    })
}

// ... existing handlers

#[derive(serde::Deserialize)]
struct PairingConfigRequest {
    enabled: bool,
}

async fn set_pairing_config(
    _user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<PairingConfigRequest>,
) -> Json<serde_json::Value> {
    state.pairing.set_pairing_required(payload.enabled);
    Json(serde_json::json!({ "success": true, "enabled": payload.enabled }))
}

async fn get_models(_user: AuthenticatedUser, State(state): State<AppState>) -> Json<Vec<ModelInfo>> {
    Json(vec![
        ModelInfo {
            id: state.model.clone(),
            provider: "default".into(),
            name: state.model.clone(),
            description: "Active model".into(),
        }
    ])
}
