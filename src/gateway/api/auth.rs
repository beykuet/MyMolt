use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
};
use crate::gateway::AppState;
use serde::Deserialize;

pub struct AuthenticatedUser;

#[derive(Debug, Deserialize)]
pub struct AuthQuery {
    pub token: Option<String>,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 1. Check Authorization header
        if let Some(auth_header) = parts.headers.get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    if state.pairing.is_authenticated(token) {
                        return Ok(AuthenticatedUser);
                    }
                }
            }
        }

        // 2. Check Query param (for WebSocket or easy CLI testing)
        // We have to manually parse query string because FromRequestParts doesn't compose easily with Query extractor here without cloning
        if let Some(query) = parts.uri.query() {
            if let Ok(params) = serde_urlencoded::from_str::<AuthQuery>(query) {
                if let Some(token) = params.token {
                    if state.pairing.is_authenticated(&token) {
                        return Ok(AuthenticatedUser);
                    }
                }
            }
        }

        // If pairing is disabled, we might allow access, but usually we want to enforce it if configured.
        // The PairingGuard::is_authenticated handles "pairing disabled" logic (returns true).
        // If we fell through here, it means either no token provided OR token invalid.
        
        // Final check for "no auth required" case if guard says so (empty token)
        if state.pairing.is_authenticated("") {
             return Ok(AuthenticatedUser);
        }

        Err((StatusCode::UNAUTHORIZED, "Unauthorized"))
    }
}
