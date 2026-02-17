use crate::config::schema::OIDCProviderConfig;
use anyhow::{Context, Result};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GenericOIDCProvider {
    config: OIDCProviderConfig,
    client: Client,
}

#[derive(Debug, Deserialize)]
struct OIDCDiscovery {
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    id_token: Option<String>,
}

impl GenericOIDCProvider {
    pub fn new(config: OIDCProviderConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Discover OIDC endpoints from issuer URL (.well-known/openid-configuration)
    async fn discover(&self) -> Result<OIDCDiscovery> {
        let discovery_url = format!("{}/.well-known/openid-configuration", self.config.issuer_url.trim_end_matches('/'));
        let resp = self.client.get(&discovery_url).send().await?
            .json::<OIDCDiscovery>().await
            .context("Failed to fetch OIDC discovery document")?;
        Ok(resp)
    }

    /// Generate the login URL for the frontend to redirect to
    pub async fn get_login_url(&self, redirect_uri: &str, state: &str) -> Result<String> {
        let discovery = self.discover().await?;
        
        let mut url = Url::parse(&discovery.authorization_endpoint)?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.config.client_id)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", "openid email profile")
            .append_pair("state", state);
            
        Ok(url.to_string())
    }

    /// Exchange code for token and fetch user info
    pub async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<UserInfo> {
        let discovery = self.discover().await?;
        
        // 1. Exchange Code
        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", code);
        params.insert("redirect_uri", redirect_uri);
        params.insert("client_id", &self.config.client_id);
        
        if let Some(secret) = &self.config.client_secret {
            params.insert("client_secret", secret);
        }

        let token_resp = self.client.post(&discovery.token_endpoint)
            .form(&params)
            .send().await?
            .json::<TokenResponse>().await
            .context("Failed to exchange OIDC code for token")?;

        // 2. Fetch User Info
        let user_info_resp = self.client.get(&discovery.userinfo_endpoint)
            .bearer_auth(token_resp.access_token)
            .send().await?
            .json::<serde_json::Value>().await
            .context("Failed to fetch user info")?;

        // 3. Map to UserInfo struct based on config mapping rules
        Ok(self.map_user_info(&user_info_resp))
    }

    fn map_user_info(&self, data: &serde_json::Value) -> UserInfo {
        let default_sub = data.get("sub").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
        
        // If mapping is provided, try to find the ID field
        let id_field = self.config.mapping.get("id").map(|s| s.as_str()).unwrap_or("sub");
        let id = data.get(id_field).and_then(|v| v.as_str()).unwrap_or(&default_sub).to_string();
        
        let email_field = self.config.mapping.get("email").map(|s| s.as_str()).unwrap_or("email");
        let email = data.get(email_field).and_then(|v| v.as_str()).map(|s| s.to_string());
        
        let name_field = self.config.mapping.get("name").map(|s| s.as_str()).unwrap_or("name");
        let name = data.get(name_field).and_then(|v| v.as_str()).map(|s| s.to_string());

        UserInfo {
            id,
            email,
            name,
            raw: data.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub raw: serde_json::Value,
}
