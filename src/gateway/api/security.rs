// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! Security overview API — aggregated security stats for the dashboard.

use axum::{
    extract::State,
    extract::Json,
    http::StatusCode,
    routing::get,
    Router,
};
use crate::gateway::AppState;
use crate::gateway::api::auth::AuthenticatedUser;
use crate::identity::UserRole;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SecurityOverview {
    // Trust & sandbox
    pub trust_level: String,
    pub sandbox_type: String,
    pub autonomy_mode: String,

    // Confirmation gate
    pub confirmation_gate_enabled: bool,
    pub pending_confirmations: usize,

    // VPN
    pub vpn_connected: bool,
    pub vpn_provider: String,

    // DNS Shield
    pub dns_shield_enabled: bool,
    pub dns_blocked_today: u64,

    // Sensitivity scanner
    pub sensitivity_patterns: usize,

    // Audit
    pub audit_enabled: bool,

    // TLS / tunnel
    pub tls_active: bool,
}

/// GET /api/security/overview — aggregated security stats
pub async fn get_security_overview(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<SecurityOverview>, StatusCode> {
    if user.role != UserRole::Root && user.role != UserRole::Adult {
        return Err(StatusCode::FORBIDDEN);
    }

    let config = state.config.read().await;

    // Gather stats from various subsystems
    let vpn_connected = state.vpn_manager.list_peers().map(|p| !p.is_empty()).unwrap_or(false);
    let dns_enabled = state.adblock.is_enabled().await;
    let dns_count = state.adblock.count().await;
    let pending = state.confirm_gate.pending_count().await;

    // Determine confirmation gate status from confirmation_required map
    let gate_enabled = !config.security.confirmation_required.is_empty();

    let overview = SecurityOverview {
        trust_level: "high".into(),
        sandbox_type: format!("{:?}", config.security.sandbox.backend),
        autonomy_mode: if gate_enabled {
            "supervised".into()
        } else {
            "autonomous".into()
        },
        confirmation_gate_enabled: gate_enabled,
        pending_confirmations: pending,
        vpn_connected,
        vpn_provider: "WireGuard".into(),
        dns_shield_enabled: dns_enabled,
        dns_blocked_today: dns_count as u64,
        sensitivity_patterns: 0, // Would need to count patterns from security policy
        audit_enabled: config.security.audit.enabled,
        tls_active: state.public_url.starts_with("https"),
    };

    Ok(Json(overview))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/security/overview", get(get_security_overview))
}
