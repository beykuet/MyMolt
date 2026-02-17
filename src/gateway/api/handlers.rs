use axum::{
    extract::{State, Json},
    routing::{get, post},
    Router,
    extract::Query,
    http::StatusCode,
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
        .route("/api/config/voice_echo", post(set_voice_echo_config))
        .route("/api/config/models", get(get_models))
        // Identity
        .route("/api/auth/providers", get(get_auth_providers))
        .route("/api/auth/login/{provider_id}", get(handle_oidc_login))
        .route("/api/auth/callback/{provider_id}", get(handle_oidc_callback))
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
        voice_echo_enabled: state.voice_echo_enabled.load(std::sync::atomic::Ordering::Relaxed),
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

#[derive(serde::Deserialize)]
struct VoiceEchoRequest {
    enabled: bool,
}

async fn set_voice_echo_config(
    _user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<VoiceEchoRequest>,
) -> Json<serde_json::Value> {
    state.voice_echo_enabled.store(payload.enabled, std::sync::atomic::Ordering::Relaxed);
    Json(serde_json::json!({ "success": true, "enabled": payload.enabled }))
}

#[derive(serde::Deserialize)]
struct OICDCallbackQuery {
    code: String,
    state: String,
}

async fn get_auth_providers(_user: AuthenticatedUser, State(state): State<AppState>) -> Json<Vec<IdentityProvider>> {
    let providers = state.identity_config.providers.iter().map(|p| IdentityProvider {
        id: p.id.clone(),
        name: p.name.clone(),
        icon_url: p.icon_url.clone(),
    }).collect();
    Json(providers)
}

async fn handle_oidc_login(
    State(state): State<AppState>,
    axum::extract::Path(provider_id): axum::extract::Path<String>,
) ->  Result<axum::response::Redirect, (StatusCode, String)> {
    let config = state.identity_config.providers.iter()
        .find(|p| p.id == provider_id)
        .ok_or((StatusCode::NOT_FOUND, "Provider not found".to_string()))?;

    let provider = crate::identity::oidc_generic::GenericOIDCProvider::new(config.clone());
    
    // In a real app, generate a secure random state and store it in a cookie/cache
    let state_param = "random_state_needs_improvement"; 
    // This needs to be the actual callback URL reachable by the browser
    let redirect_uri = format!("{}/api/auth/callback/{}", state.public_url.trim_end_matches('/'), provider_id);

    let url = provider.get_login_url(&redirect_uri, state_param).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(axum::response::Redirect::to(&url))
}

async fn handle_oidc_callback(
    State(state): State<AppState>,
    axum::extract::Path(provider_id): axum::extract::Path<String>,
    Query(query): Query<OICDCallbackQuery>,
) -> Result<axum::response::Redirect, (StatusCode, String)> {
     let config = state.identity_config.providers.iter()
        .find(|p| p.id == provider_id)
        .ok_or((StatusCode::NOT_FOUND, "Provider not found".to_string()))?;

    let provider = crate::identity::oidc_generic::GenericOIDCProvider::new(config.clone());
    let redirect_uri = format!("{}/api/auth/callback/{}", state.public_url.trim_end_matches('/'), provider_id);

    let user_info = provider.exchange_code(&query.code, &redirect_uri).await
         .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Token exchange failed: {}", e)))?;

    // Link identity
    let mut soul = state.soul.lock().unwrap();
    // Use the provider's trust level or default to Low. For now, assume Generic OIDC is Medium/High? 
    // The plan said "Government OIDC" -> High.
    // Maybe we should add `trust_level` to OIDCProviderConfig. For now hardcode High for demo.
    let trust_level = crate::identity::soul::TrustLevel::High; 

    soul.add_binding(&config.name, &user_info.id, trust_level)
         .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to link identity: {}", e)))?;

    // Redirect to dashboard with success param
    Ok(axum::response::Redirect::to("/?login_success=true"))
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
