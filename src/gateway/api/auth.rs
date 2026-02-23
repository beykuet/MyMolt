// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
};
use crate::gateway::AppState;
use serde::Deserialize;

pub struct AuthenticatedUser {
    pub role: crate::identity::UserRole,
}

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
        // Helper to validate token and return role
        let check_token = |token: &str| -> Option<crate::identity::UserRole> {
            if state.pairing.is_authenticated(token) {
                 // For MVP: The main pairing token grants ROOT access.
                 // Future: Look up token in DB to find specific user/role.
                 Some(crate::identity::UserRole::Root) 
            } else if token.is_empty() && state.pairing.is_authenticated("") {
                 // No-auth mode (development only) -> Root
                 Some(crate::identity::UserRole::Root)
            } else {
                None
            }
        };

        // 1. Check Authorization header
        if let Some(auth_header) = parts.headers.get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    if let Some(role) = check_token(token) {
                        return Ok(AuthenticatedUser { role });
                    }
                }
            }
        }

        // 2. Check Query param (DEPRECATED for security, but kept for WS compatibility if needed)
        if let Some(query) = parts.uri.query() {
            if let Ok(params) = serde_urlencoded::from_str::<AuthQuery>(query) {
                if let Some(token) = params.token {
                    if let Some(role) = check_token(&token) {
                        return Ok(AuthenticatedUser { role });
                    }
                }
            }
        }

        // 3. Check for "no auth required" case (empty strings)
        if let Some(role) = check_token("") {
             return Ok(AuthenticatedUser { role });
        }

        Err((StatusCode::UNAUTHORIZED, "Unauthorized"))
    }
}
