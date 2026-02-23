// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! Browse API — proxy, page comprehension, history, bookmarks.
//!
//! Powers both the Sovereign Browser widget and the Chrome extension.

use axum::{
    extract::{State, Json, Query},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use crate::gateway::AppState;
use crate::gateway::api::auth::AuthenticatedUser;
use crate::identity::UserRole;
use serde::{Deserialize, Serialize};

// ── Types ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ProxyQuery {
    pub url: String,
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProxyResponse {
    pub html: String,
    pub text: String,
    pub title: String,
    pub blocked: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AskRequest {
    pub url: String,
    pub page_text: String,
    pub question: String,
    pub role: Option<String>,
    pub conversation: Option<Vec<ConversationMessage>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct AskResponse {
    pub answer: String,
    pub sources: Vec<SourceRef>,
    pub media: Vec<MediaItem>,
}

#[derive(Debug, Serialize)]
pub struct SourceRef {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct MediaItem {
    #[serde(rename = "type")]
    pub media_type: String,
    pub url: String,
    pub caption: String,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkRequest {
    pub url: String,
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BookmarkEntry {
    pub url: String,
    pub title: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct HistoryEntry {
    pub url: String,
    pub title: String,
    pub visited_at: String,
}

// ── Blocked domain lists ───────────────────────────────────────────

const CHILD_BLOCKED_PATTERNS: &[&str] = &[
    "adult", "porn", "xxx", "gambling", "casino", "bet",
    "onlyfans", "tinder", "grindr",
];

fn is_blocked_for_role(url: &str, role: &UserRole) -> Option<String> {
    let url_lower = url.to_lowercase();
    match role {
        UserRole::Child => {
            for pattern in CHILD_BLOCKED_PATTERNS {
                if url_lower.contains(pattern) {
                    return Some(format!("Content filtered: '{}' is not available for your role", pattern));
                }
            }
            None
        }
        _ => None,
    }
}

// ── Handlers ───────────────────────────────────────────────────────

/// GET /api/browse/proxy?url=...&role=... — fetch and sanitize a page
pub async fn browse_proxy(
    _user: AuthenticatedUser,
    State(_state): State<AppState>,
    Query(params): Query<ProxyQuery>,
) -> Result<Json<ProxyResponse>, (StatusCode, String)> {
    let role = match params.role.as_deref() {
        Some("Root") => UserRole::Root,
        Some("Senior") => UserRole::Senior,
        Some("Child") => UserRole::Child,
        _ => UserRole::Adult,
    };

    // Check if blocked for role
    if let Some(reason) = is_blocked_for_role(&params.url, &role) {
        return Ok(Json(ProxyResponse {
            html: String::new(),
            text: String::new(),
            title: String::new(),
            blocked: true,
            reason: Some(reason),
        }));
    }

    // Fetch the page
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("MyMolt/1.0 (Sovereign Browser)")
        .build()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("HTTP client error: {e}")))?;

    let response = client.get(&params.url)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to fetch: {e}")))?;

    let status = response.status();
    if !status.is_success() {
        return Err((StatusCode::BAD_GATEWAY, format!("Remote returned {status}")));
    }

    let body = response.text().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to read body: {e}")))?;

    // Extract title
    let title = extract_title(&body);

    // Extract readable text (simple approach)
    let text = extract_readable_text(&body);

    // Sanitize HTML based on role
    let html = sanitize_html(&body, &role);

    Ok(Json(ProxyResponse {
        html,
        text,
        title,
        blocked: false,
        reason: None,
    }))
}

/// POST /api/browse/ask — ask the agent about the current page
pub async fn browse_ask(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<AskRequest>,
) -> Result<Json<AskResponse>, (StatusCode, String)> {
    let role = match payload.role.as_deref() {
        Some("Child") => UserRole::Child,
        Some("Senior") => UserRole::Senior,
        _ => user.role.clone(),
    };

    // Build the prompt based on role
    let role_instruction = match role {
        UserRole::Child => "Answer simply, use short sentences, be friendly and encouraging. Use emojis. Explain like talking to a 10-year-old.",
        UserRole::Senior => "Answer clearly with larger concepts. Be patient and thorough. Avoid jargon. Summarize key points at the start.",
        _ => "Answer thoroughly with full detail. Include relevant sources and technical depth where appropriate.",
    };

    let context = format!(
        "The user is viewing this webpage: {}\n\nPage content (excerpt):\n{}\n\n---\nInstruction: {}\n\nUser question: {}",
        payload.url,
        &payload.page_text[..payload.page_text.len().min(8000)],
        role_instruction,
        payload.question,
    );

    // Use the agent to generate an answer
    let answer = crate::gateway::gateway_agent_reply(&state, &context)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Agent error: {e}")))?;

    Ok(Json(AskResponse {
        answer,
        sources: vec![], // Would be populated by web search tool if available
        media: vec![],   // Would be populated by media search
    }))
}

/// GET /api/browse/history — browsing history (stub — needs per-user storage)
pub async fn browse_history(
    _user: AuthenticatedUser,
) -> Json<Vec<HistoryEntry>> {
    // Stub — real implementation would query per-user encrypted storage
    Json(vec![])
}

/// POST /api/browse/bookmark — save a bookmark
pub async fn browse_bookmark(
    _user: AuthenticatedUser,
    Json(payload): Json<BookmarkRequest>,
) -> Json<serde_json::Value> {
    // Stub — real implementation would persist to user's memory/vault
    Json(serde_json::json!({
        "status": "saved",
        "url": payload.url,
        "title": payload.title,
    }))
}

// ── Vault match for extension autofill ─────────────────────────────

#[derive(Debug, Deserialize)]
pub struct VaultMatchQuery {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct VaultMatchResult {
    pub id: String,
    pub url_pattern: String,
    pub username: String,
}

/// GET /api/vault/match?url=... — find vault credentials matching a URL
pub async fn vault_match(
    _user: AuthenticatedUser,
    Query(_params): Query<VaultMatchQuery>,
) -> Json<Vec<VaultMatchResult>> {
    // Stub — real implementation would search vault entries by URL pattern
    Json(vec![])
}

/// POST /api/vault/autofill-log — log an autofill event for audit
pub async fn vault_autofill_log(
    _user: AuthenticatedUser,
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    // Log the autofill event — in production this would create an AuditEvent
    let _url = payload.get("url").and_then(|v| v.as_str()).unwrap_or("unknown");
    Json(serde_json::json!({"status": "logged"}))
}

// ── DNS rules for extension ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DnsRulesQuery {
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DnsBlockRule {
    pub id: u32,
    pub priority: u32,
    pub action: serde_json::Value,
    pub condition: serde_json::Value,
}

/// GET /api/dns/rules?role=... — DNS block rules for Chrome extension
pub async fn dns_rules(
    _user: AuthenticatedUser,
    Query(params): Query<DnsRulesQuery>,
) -> Json<Vec<DnsBlockRule>> {
    let role = params.role.as_deref().unwrap_or("Adult");

    let mut rules = vec![];

    if role == "Child" {
        let patterns = vec![
            ("*://*.xxx/*", 1),
            ("*://adult*/*", 2),
            ("*://*.porn*/*", 3),
            ("*://gambling*/*", 4),
            ("*://*.casino*/*", 5),
            ("*://*.bet*/*", 6),
            ("*://*.onlyfans.com/*", 7),
        ];

        for (pattern, id) in patterns {
            rules.push(DnsBlockRule {
                id: 80000 + id,
                priority: 1,
                action: serde_json::json!({"type": "block"}),
                condition: serde_json::json!({
                    "urlFilter": pattern,
                    "resourceTypes": ["main_frame", "sub_frame"]
                }),
            });
        }
    }

    Json(rules)
}

// ── Helpers ────────────────────────────────────────────────────────

fn extract_title(html: &str) -> String {
    // Simple regex-free title extraction
    if let Some(start) = html.find("<title") {
        if let Some(tag_end) = html[start..].find('>') {
            let after_tag = start + tag_end + 1;
            if let Some(close) = html[after_tag..].find("</title>") {
                return html[after_tag..after_tag + close].trim().to_string();
            }
        }
    }
    String::new()
}

fn extract_readable_text(html: &str) -> String {
    // Strip HTML tags for readable text extraction
    let mut text = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
            // Check for script/style
            let remaining = &html[html.len().saturating_sub(100)..]; // approximate
            if remaining.to_lowercase().starts_with("<script") {
                in_script = true;
            } else if remaining.to_lowercase().starts_with("<style") {
                in_style = true;
            }
            continue;
        }
        if ch == '>' {
            in_tag = false;
            continue;
        }
        if !in_tag && !in_script && !in_style {
            text.push(ch);
        }
        if in_tag {
            let lower = ch.to_lowercase().to_string();
            if lower == "/" {
                in_script = false;
                in_style = false;
            }
        }
    }

    // Clean up whitespace
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn sanitize_html(html: &str, role: &UserRole) -> String {
    match role {
        UserRole::Child => {
            // Strip all scripts for children
            let mut result = html.to_string();
            // Remove script tags (simple approach)
            while let Some(start) = result.find("<script") {
                if let Some(end) = result[start..].find("</script>") {
                    result = format!("{}{}", &result[..start], &result[start + end + 9..]);
                } else {
                    break;
                }
            }
            result
        }
        _ => html.to_string(),
    }
}

// ── Router ─────────────────────────────────────────────────────────

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/browse/proxy", get(browse_proxy))
        .route("/api/browse/ask", post(browse_ask))
        .route("/api/browse/history", get(browse_history))
        .route("/api/browse/bookmark", post(browse_bookmark))
        .route("/api/vault/match", get(vault_match))
        .route("/api/vault/autofill-log", post(vault_autofill_log))
        .route("/api/dns/rules", get(dns_rules))
}
