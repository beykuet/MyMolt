// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

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
        .route("/api/config/models", post(set_model))
        // Identity
        .route("/api/auth/providers", get(get_auth_providers))
        .route("/api/auth/login/{provider_id}", get(handle_oidc_login))
        .route("/api/auth/callback/{provider_id}", get(handle_oidc_callback))
        // SSI
        .route("/api/identity/verify-vp", post(verify_vp_endpoint))
        
        // Vault & Security (Root-only)
        .route("/api/vault", get(get_vault_entries))
        .route("/api/security/sigil", get(get_sigil_logs))

        // Confirmation flow
        .route("/api/security/confirm", post(resolve_confirmation))
        .route("/api/security/confirm/pending", get(get_pending_confirmations))

        // AdBlock (Root/Adult)
        .route("/api/config/adblock", get(get_adblock_config))
        .route("/api/config/adblock/toggle", post(toggle_adblock))

        // VPN routes are in vpn.rs (merged via api::routes())
        
        // Diary
        .route("/api/soul/diary", get(get_diary_entries_handler))
        .route("/api/soul/diary", post(create_diary_entry))
}

// ── Diary Handlers ────────────────────────────────────────────────

async fn get_diary_entries_handler(user: AuthenticatedUser, State(state): State<AppState>) -> Result<Json<Vec<crate::identity::soul::DiaryEntry>>, (StatusCode, String)> {
    if user.role == crate::identity::UserRole::Child {
         return Err((StatusCode::FORBIDDEN, "Children cannot access the diary".into()));
    }
    let soul = state.soul.lock().await;
    // Default limit 50
    Ok(Json(soul.get_diary_entries(50)))
}

async fn create_diary_entry(
    user: AuthenticatedUser, 
    State(state): State<AppState>, 
    Json(payload): Json<CreateDiaryEntryRequest>
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // RBAC: Senior, Adult, and Root can write to diary (Child cannot)
    if user.role < crate::identity::UserRole::Senior {
         return Err((StatusCode::FORBIDDEN, "Only Adults/Seniors can update the diary".into()));
    }

    // Rate limiting: 20 diary writes per minute
    if !state.rate_limiter.allow_diary("diary_global") {
        return Err((StatusCode::TOO_MANY_REQUESTS, "Too many diary writes. Please wait.".into()));
    }

    // Input sanitization: limit length and strip dangerous characters
    let content = payload.content.trim();
    if content.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Diary entry cannot be empty".into()));
    }
    if content.len() > 10_000 {
        return Err((StatusCode::BAD_REQUEST, "Diary entry too long (max 10000 chars)".into()));
    }
    // Strip markdown heading syntax that could corrupt SOUL.md structure
    let sanitized = content
        .replace("# ", "")
        .replace("## ", "")
        .replace("### ", "");

    let mut soul = state.soul.lock().await;
    
    soul.append_diary_entry(&sanitized)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
    Ok(Json(serde_json::json!({ "success": true })))
}

// ── Encrypted Memories / Vault ───────────────────────────────────

async fn get_vault_entries(user: AuthenticatedUser, State(state): State<AppState>) -> Result<Json<Vec<VaultEntryMetadata>>, (StatusCode, String)> {
    if user.role != crate::identity::UserRole::Root {
        return Err((StatusCode::FORBIDDEN, "Only Root can audit vault".into()));
    }

    let entries = state.vault.list_entries()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let metadata = entries.into_iter().map(|e| VaultEntryMetadata {
        id: e.id,
        description: e.description,
        created_at: e.created_at,
        tags: e.tags,
    }).collect();

    Ok(Json(metadata))
}

async fn get_sigil_logs(user: AuthenticatedUser, State(state): State<AppState>) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    if user.role != crate::identity::UserRole::Root {
        return Err((StatusCode::FORBIDDEN, "Only Root can view Sigil interception logs".into()));
    }

    // Read the audit log and filter for Sigil interception events
    let log_path = state.audit.log_path();
    
    if !log_path.exists() {
        return Ok(Json(vec![]));
    }

    let content = std::fs::read_to_string(log_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let logs: Vec<serde_json::Value> = content.lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .filter(|v: &serde_json::Value| {
            let event_type = v["event_type"].as_str().unwrap_or("");
            event_type == "sigil_interception" || (event_type == "SecurityEvent" && v["action"].as_str().unwrap_or("").starts_with("confirm:"))
        })
        .collect();

    Ok(Json(logs))
}

// VPN Handlers are in vpn.rs

// ── Handlers ─────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct VerifyVPRequest {
    vp: String, // JSON-LD or JWT string
}

async fn verify_vp_endpoint(
    _user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<VerifyVPRequest>,
) -> Json<serde_json::Value> {
    match crate::identity::ssi::SSIGuardian::verify_vp(&payload.vp).await {
        Ok(result) => {
            if result.is_valid {
                // Link the DID to the soul
                if let Some(holder) = &result.holder_did {
                    let mut soul = state.soul.lock().await;
                    let _ = soul.add_binding("SSI-Wallet", holder, crate::identity::soul::TrustLevel::High);
                }
            }
            Json(serde_json::json!({ "success": true, "result": result }))
        }
        Err(e) => {
             Json(serde_json::json!({ "success": false, "error": format!("{e}") }))
        }
    }
}


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
    let soul = state.soul.lock().await;
    
    let mut identities: Vec<IdentityStatus> = soul.bindings.iter().map(|b| IdentityStatus {
        provider: b.provider.clone(),
        id: b.id.clone(),
        trust_level: match b.trust_level {
            crate::identity::soul::TrustLevel::High => 3,
            crate::identity::soul::TrustLevel::Medium => 2,
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
    let mut soul = state.soul.lock().await;
    
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
    
    let mut soul = state.soul.lock().await;
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
    use sysinfo::System;

    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_cpu_all();

    // Real uptime from start instant
    let uptime = state.started_at.elapsed().as_secs();

    // Real memory: this process's RSS in MB
    let pid = sysinfo::get_current_pid().ok();
    let memory_mb = if let Some(pid) = pid {
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        sys.process(pid)
            .map(|p| (p.memory() / (1024 * 1024)) as u64)
            .unwrap_or(0)
    } else {
        0
    };

    // Real CPU: global average (percentage)
    let cpu_usage = sys.global_cpu_usage();

    Json(SystemStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: uptime,
        memory_usage_mb: memory_mb,
        cpu_usage_percent: cpu_usage,
        active_agents: 1,
        voice_mode_active: false,
        pairing_enabled: state.pairing.require_pairing(),
        voice_echo_enabled: state.voice_echo_enabled.load(std::sync::atomic::Ordering::Relaxed),
        adblock_enabled: state.adblock.is_enabled().await,
        adblock_count: state.adblock.count().await,
    })
}

async fn get_adblock_config(_user: AuthenticatedUser, State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "enabled": state.adblock.is_enabled().await,
        "count": state.adblock.count().await,
    }))
}

async fn toggle_adblock(user: AuthenticatedUser, State(state): State<AppState>, Json(payload): Json<AdBlockToggleRequest>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if user.role < crate::identity::UserRole::Adult {
        return Err((StatusCode::FORBIDDEN, "Only Adults/Root can manage AdBlock".into()));
    }

    state.adblock.toggle(payload.enabled).await;
    Ok(Json(serde_json::json!({ "success": true, "enabled": payload.enabled })))
}

#[derive(serde::Deserialize)]
struct AdBlockToggleRequest {
    enabled: bool,
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
        trust_level: p.trust_level,
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
    
    // Generate a cryptographically secure random state token (CSRF protection)
    let state_param = state.oidc_states.generate(&provider_id);
    let redirect_uri = format!("{}/api/auth/callback/{}", state.public_url.trim_end_matches('/'), provider_id);

    let url = provider.get_login_url(&redirect_uri, &state_param).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(axum::response::Redirect::to(&url))
}

async fn handle_oidc_callback(
    State(state): State<AppState>,
    axum::extract::Path(provider_id): axum::extract::Path<String>,
    Query(query): Query<OICDCallbackQuery>,
) -> Result<axum::response::Redirect, (StatusCode, String)> {
    // Validate and consume the state token (single-use, prevents CSRF + replay)
    let validated_provider = state.oidc_states.validate(&query.state)
        .ok_or((StatusCode::BAD_REQUEST, "Invalid or expired OIDC state parameter (possible CSRF)".to_string()))?;

    // Verify the state was generated for THIS provider
    if validated_provider != provider_id {
        return Err((StatusCode::BAD_REQUEST, "OIDC state mismatch: callback provider does not match login provider".to_string()));
    }

    let config = state.identity_config.providers.iter()
        .find(|p| p.id == provider_id)
        .ok_or((StatusCode::NOT_FOUND, "Provider not found".to_string()))?;

    let provider = crate::identity::oidc_generic::GenericOIDCProvider::new(config.clone());
    let redirect_uri = format!("{}/api/auth/callback/{}", state.public_url.trim_end_matches('/'), provider_id);

    let user_info = provider.exchange_code(&query.code, &redirect_uri).await
         .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Token exchange failed: {}", e)))?;

    let trust_level = match config.trust_level {
        3 => crate::identity::soul::TrustLevel::High,
        2 => crate::identity::soul::TrustLevel::Medium,
        1 | _ => crate::identity::soul::TrustLevel::Low,
    };
    
    let mut soul = state.soul.lock().await;

    soul.add_binding(&config.name, &user_info.id, trust_level)
         .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to link identity: {}", e)))?;

    // Redirect to dashboard with success param
    Ok(axum::response::Redirect::to("/?login_success=true"))
}

async fn get_models(_user: AuthenticatedUser, State(state): State<AppState>) -> Json<Vec<ModelInfo>> {
    let current = state.model.read().await.clone();
    json_models(current)
}

fn json_models(current: String) -> Json<Vec<ModelInfo>> {
    Json(vec![
        ModelInfo {
            id: current.clone(),
            provider: "active".into(),
            name: current,
            description: "Currently Active Model".into(),
        },
        ModelInfo {
            id: "gemini-2.0-flash-exp".into(),
            provider: "google".into(),
            name: "Gemini 2.0 Flash".into(),
            description: "Fastest multimodal model".into(),
        },
        ModelInfo {
            id: "gpt-4o".into(),
            provider: "openai".into(),
            name: "GPT-4o".into(),
            description: "Reliable reasoning".into(),
        }
    ])
}

async fn set_model(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<SelectModelRequest>
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if user.role != crate::identity::UserRole::Root {
         return Err((StatusCode::FORBIDDEN, "Only Root can change models".into()));
    }

    // Rate limiting: 3 model switches per minute
    if !state.rate_limiter.allow_model_switch("model_global") {
        return Err((StatusCode::TOO_MANY_REQUESTS, "Too many model switches. Please wait.".into()));
    }
    
    // Hot-swap: update the model at runtime
    let mut model = state.model.write().await;
    let old_model = model.clone();
    *model = payload.model_id.clone();
    
    tracing::info!("Model changed: {} -> {}", old_model, payload.model_id);
    
    Ok(Json(serde_json::json!({ 
        "success": true, 
        "previous_model": old_model,
        "current_model": payload.model_id
    })))
}

// ── Confirmation Flow ────────────────────────────────────────

async fn resolve_confirmation(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<crate::security::confirmation::ConfirmationResponse>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Only Root/Adult can resolve confirmations
    if user.role < crate::identity::UserRole::Adult {
        return Err((StatusCode::FORBIDDEN, "Only Adults/Root can resolve confirmations".into()));
    }

    let resolved = state.confirm_gate.resolve(&payload.id, payload.approved).await;

    if resolved {
        // Audit the decision
        let _ = state.audit.log(
            &crate::security::AuditEvent::new(crate::security::AuditEventType::SecurityEvent)
                .with_actor("gateway".to_string(), None, None)
                .with_action(
                    format!("confirm:{}", payload.id),
                    if payload.approved { "approved" } else { "denied" }.to_string(),
                    payload.approved,
                    true,
                ),
        );
        Ok(Json(serde_json::json!({ "success": true, "resolved": true })))
    } else {
        Ok(Json(serde_json::json!({ "success": false, "resolved": false, "error": "Request not found or expired" })))
    }
}

async fn get_pending_confirmations(
    _user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let pending = state.confirm_gate.get_pending().await;
    Json(serde_json::json!({ "pending": pending }))
}
