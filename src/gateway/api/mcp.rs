// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! MCP server management API endpoints — list, add, remove MCP servers.
//!
//! Root-only. Modifies the config.mcp section and persists.

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
pub struct McpServerView {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub status: String, // "configured" — runtime status requires actual connection check
}

#[derive(Debug, Serialize)]
pub struct McpToolView {
    pub server: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct AddMcpServerRequest {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
}

// ── Handlers ───────────────────────────────────────────────────────

/// GET /api/mcp/servers — list configured MCP servers
pub async fn list_mcp_servers(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<McpServerView>>, StatusCode> {
    if user.role != UserRole::Root {
        return Err(StatusCode::FORBIDDEN);
    }

    let config = state.config.read().await;
    let servers = config.mcp.servers.iter().map(|s| McpServerView {
        name: s.name.clone(),
        command: s.command.clone(),
        args: s.args.clone(),
        env: s.env.clone(),
        status: "configured".into(),
    }).collect();

    Ok(Json(servers))
}

/// POST /api/mcp/servers — add a new MCP server config
pub async fn add_mcp_server(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<AddMcpServerRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if user.role != UserRole::Root {
        return Err((StatusCode::FORBIDDEN, "Only Root can manage MCP servers".into()));
    }

    let mut config = state.config.write().await;

    // Check for duplicate name
    if config.mcp.servers.iter().any(|s| s.name.eq_ignore_ascii_case(&payload.name)) {
        return Err((
            StatusCode::CONFLICT,
            format!("MCP server '{}' already exists", payload.name),
        ));
    }

    let new_server = crate::config::schema::McpServerConfig {
        name: payload.name.clone(),
        command: payload.command,
        args: payload.args.unwrap_or_default(),
        env: payload.env.unwrap_or_default(),
    };

    config.mcp.servers.push(new_server);
    config.mcp.enabled = true;
    config.save().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save: {e}")))?;

    Ok(Json(serde_json::json!({
        "status": "added",
        "name": payload.name,
    })))
}

/// DELETE /api/mcp/servers/:name — remove an MCP server
pub async fn remove_mcp_server(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if user.role != UserRole::Root {
        return Err((StatusCode::FORBIDDEN, "Only Root can manage MCP servers".into()));
    }

    let mut config = state.config.write().await;
    let before = config.mcp.servers.len();
    config.mcp.servers.retain(|s| !s.name.eq_ignore_ascii_case(&name));

    if config.mcp.servers.len() == before {
        return Err((StatusCode::NOT_FOUND, format!("MCP server '{}' not found", name)));
    }

    config.save().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save: {e}")))?;

    Ok(Json(serde_json::json!({
        "status": "removed",
        "name": name,
    })))
}

/// GET /api/mcp/tools — list all tools from registered MCP servers
pub async fn list_mcp_tools(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<McpToolView>>, StatusCode> {
    if user.role != UserRole::Root {
        return Err(StatusCode::FORBIDDEN);
    }

    // Tools are dynamically registered in the tools_registry
    // We return what's available from the registry
    let tools: Vec<McpToolView> = state.tools_registry.iter().map(|t| McpToolView {
        server: "mymolt".into(), // All tools are surfaced through MyMolt's registry
        name: t.name().to_string(),
        description: t.description().to_string(),
    }).collect();

    Ok(Json(tools))
}

#[derive(Deserialize)]
pub struct CallToolRequest {
    pub payload: serde_json::Value,
}

/// POST /api/mcp/tools/:name/call — manually test a tool
pub async fn call_mcp_tool(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<CallToolRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if user.role != UserRole::Root {
        return Err((StatusCode::FORBIDDEN, "Access denied".into()));
    }

    let tool = state.tools_registry.iter().find(|t| t.name() == name)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Tool {name} not found")))?;

    let result = tool.execute(payload.payload).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Tool error: {e}")))?;

    // ToolResult is Serialize, so we can return it directly as JSON
    Ok(Json(serde_json::to_value(result).unwrap_or_else(|_| serde_json::json!({ "error": "failed to serialize result" }))))
}

// ── Router ─────────────────────────────────────────────────────────

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/mcp/servers", get(list_mcp_servers).post(add_mcp_server))
        .route("/api/mcp/servers/{name}", delete(remove_mcp_server))
        .route("/api/mcp/tools", get(list_mcp_tools))
        .route("/api/mcp/tools/{name}/call", axum::routing::post(call_mcp_tool))
}
