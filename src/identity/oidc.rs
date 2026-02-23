// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use anyhow::{anyhow, Context, Result};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const GOOGLE_ISSUER: &str = "https://accounts.google.com";
const APPLE_ISSUER: &str = "https://appleid.apple.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
}

/// Simple JWKS cache to avoid fetching keys on every request.
struct JwksCache {
    keys: HashMap<String, DecodingKey>,
    last_update: SystemTime,
}

pub struct OidcVerifier {
    google_client_id: String,
    jwks_cache: Arc<Mutex<JwksCache>>,
}

impl OidcVerifier {
    pub fn new(google_client_id: &str) -> Self {
        Self {
            google_client_id: google_client_id.to_string(),
            jwks_cache: Arc::new(Mutex::new(JwksCache {
                keys: HashMap::new(),
                last_update: UNIX_EPOCH,
            })),
        }
    }

    /// Update JWKS from Google if cache is stale (> 1 hour)
    async fn refresh_google_certs(&self) -> Result<()> {
        let mut cache = self.jwks_cache.lock().map_err(|_| anyhow!("Lock poisoned"))?;
        
        let now = SystemTime::now();
        if now.duration_since(cache.last_update).unwrap_or_default() < Duration::from_secs(3600) && !cache.keys.is_empty() {
            return Ok(());
        }

        // Fetch Google Certs
        let url = "https://www.googleapis.com/oauth2/v3/certs";
        let jwks: serde_json::Value = reqwest::get(url).await?.json().await?;
        
        if let Some(keys) = jwks["keys"].as_array() {
            cache.keys.clear();
            for key in keys {
                if let (Some(kid), Some(n), Some(e)) = (
                    key["kid"].as_str(),
                    key["n"].as_str(),
                    key["e"].as_str(),
                ) {
                    if let Ok(decoding_key) = DecodingKey::from_rsa_components(n, e) {
                        cache.keys.insert(kid.to_string(), decoding_key);
                    }
                }
            }
            cache.last_update = now;
        }

        Ok(())
    }

    pub async fn verify_google_token(&self, token: &str) -> Result<Claims> {
        // 1. Decode Header to find 'kid'
        let header = decode_header(token).context("Failed to decode token header")?;
        let kid = header.kid.ok_or_else(|| anyhow!("Token header missing 'kid'"))?;

        // 2. Ensure keys are fresh
        self.refresh_google_certs().await.context("Failed to refresh Google Certs")?;

        // 3. Get Key
        let cache = self.jwks_cache.lock().map_err(|_| anyhow!("Lock poisoned"))?;
        let key = cache.keys.get(&kid).ok_or_else(|| anyhow!("Unknown key ID: {}", kid))?;

        // 4. Validate
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.google_client_id]);
        validation.set_issuer(&[GOOGLE_ISSUER, "accounts.google.com"]); 

        let token_data = decode::<Claims>(token, key, &validation)
            .context("Token validation failed")?;

        Ok(token_data.claims)
    }
}
