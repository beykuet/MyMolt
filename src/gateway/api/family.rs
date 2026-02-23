// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! Family management API endpoints — CRUD operations for family members.
//!
//! Root-only. Reads/writes to the config file via `AppState.config`.

use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    routing::{get, delete},
    Router,
};
use crate::gateway::AppState;
use crate::gateway::api::auth::AuthenticatedUser;
use crate::identity::UserRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Types ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct FamilyMemberView {
    pub name: String,
    pub role: String,
    pub channels: HashMap<String, String>,
    pub scope: String,
}

#[derive(Debug, Serialize)]
pub struct FamilyListResponse {
    pub members: Vec<FamilyMemberView>,
    pub max_members: usize,
    pub count: usize,
}

#[derive(Debug, Deserialize)]
pub struct AddFamilyMemberRequest {
    pub name: String,
    pub role: Option<String>,
    pub channels: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFamilyMemberRequest {
    pub role: Option<String>,
    pub channels: Option<HashMap<String, String>>,
}

// ── Handlers ───────────────────────────────────────────────────────

/// GET /api/family/members — list all family members
pub async fn list_family_members(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<FamilyListResponse>, StatusCode> {
    if user.role != UserRole::Root {
        return Err(StatusCode::FORBIDDEN);
    }

    let config = state.config.read().await;
    let members: Vec<FamilyMemberView> = config.family.members.iter().map(|m| {
        let role_enum = match m.role.to_lowercase().as_str() {
            "root" => UserRole::Root,
            "senior" => UserRole::Senior,
            "child" => UserRole::Child,
            _ => UserRole::Adult,
        };
        // Compute scope the same way FamilyMember does
        let slug: String = m.name.to_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();

        FamilyMemberView {
            name: m.name.clone(),
            role: format!("{:?}", role_enum),
            channels: m.channels.clone(),
            scope: format!("user:{slug}"),
        }
    }).collect();

    let count = members.len();
    Ok(Json(FamilyListResponse {
        members,
        max_members: config.family.max_members,
        count,
    }))
}

/// POST /api/family/members — add a new family member
pub async fn add_family_member(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<AddFamilyMemberRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if user.role != UserRole::Root {
        return Err((StatusCode::FORBIDDEN, "Only Root can manage family".into()));
    }

    let mut config = state.config.write().await;

    // Check max members
    if config.family.members.len() >= config.family.max_members {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Maximum {} family members reached", config.family.max_members),
        ));
    }

    // Check for duplicate name
    if config.family.members.iter().any(|m| m.name.eq_ignore_ascii_case(&payload.name)) {
        return Err((
            StatusCode::CONFLICT,
            format!("Family member '{}' already exists", payload.name),
        ));
    }

    let new_member = crate::config::schema::FamilyMemberConfig {
        name: payload.name.clone(),
        role: payload.role.unwrap_or_else(|| "adult".into()),
        channels: payload.channels.unwrap_or_default(),
    };

    config.family.members.push(new_member);
    config.save().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save: {e}")))?;

    Ok(Json(serde_json::json!({
        "status": "added",
        "name": payload.name,
        "count": config.family.members.len(),
    })))
}

/// DELETE /api/family/members/:name — remove a family member by name
pub async fn remove_family_member(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if user.role != UserRole::Root {
        return Err((StatusCode::FORBIDDEN, "Only Root can manage family".into()));
    }

    let mut config = state.config.write().await;
    let before = config.family.members.len();
    config.family.members.retain(|m| !m.name.eq_ignore_ascii_case(&name));

    if config.family.members.len() == before {
        return Err((StatusCode::NOT_FOUND, format!("Family member '{}' not found", name)));
    }

    config.save().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save: {e}")))?;

    Ok(Json(serde_json::json!({
        "status": "removed",
        "name": name,
        "count": config.family.members.len(),
    })))
}

/// PUT /api/family/members/:name — update a family member
pub async fn update_family_member(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<UpdateFamilyMemberRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if user.role != UserRole::Root {
        return Err((StatusCode::FORBIDDEN, "Only Root can manage family".into()));
    }

    let mut config = state.config.write().await;
    let member = config.family.members.iter_mut()
        .find(|m| m.name.eq_ignore_ascii_case(&name))
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Family member '{}' not found", name)))?;

    if let Some(role) = payload.role {
        member.role = role;
    }
    if let Some(channels) = payload.channels {
        member.channels = channels;
    }

    config.save().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save: {e}")))?;

    Ok(Json(serde_json::json!({
        "status": "updated",
        "name": name,
    })))
}

// ── Router ─────────────────────────────────────────────────────────

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/family/members", get(list_family_members).post(add_family_member))
        .route("/api/family/members/{name}", delete(remove_family_member).put(update_family_member))
}
