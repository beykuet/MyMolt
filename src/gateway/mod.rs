// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! Axum-based HTTP gateway with proper HTTP/1.1 compliance, body limits, and timeouts.
//!
//! This module replaces the raw TCP implementation with axum for:
//! - Proper HTTP/1.1 parsing and compliance
//! - Content-Length validation (handled by hyper)
//! - Request body size limits (64KB max)
//! - Request timeouts (30s) to prevent slow-loris attacks
//! - Header sanitization (handled by axum/hyper)

use crate::channels::{Channel, WhatsAppChannel};
use crate::config::Config;

pub mod api;
use crate::memory::{self, Memory, MemoryCategory};
use crate::observability::{self, Observer};
use crate::providers::{self, ChatMessage, Provider};
use crate::runtime;
use crate::security::{
    pairing::{constant_time_eq, is_public_bind, PairingGuard},
    SecurityPolicy,
};
use crate::tools::{self, Tool};
use crate::util::truncate_with_ellipsis;
use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
use uuid::Uuid;

/// Maximum request body size (64KB) — prevents memory exhaustion
pub const MAX_BODY_SIZE: usize = 65_536;
/// Request timeout (30s) — prevents slow-loris attacks
pub const REQUEST_TIMEOUT_SECS: u64 = 30;
/// Sliding window used by gateway rate limiting.
pub const RATE_LIMIT_WINDOW_SECS: u64 = 60;

fn webhook_memory_key() -> String {
    format!("webhook_msg_{}", Uuid::new_v4())
}

fn whatsapp_memory_key(msg: &crate::channels::traits::ChannelMessage) -> String {
    format!("whatsapp_{}_{}", msg.sender, msg.id)
}

fn normalize_gateway_reply(reply: String) -> String {
    if reply.trim().is_empty() {
        return "Model returned an empty response.".to_string();
    }

    reply
}

async fn gateway_agent_reply(state: &AppState, message: &str) -> Result<String> {
    let system_prompt = state.system_prompt.read().await;
    let temperature = *state.temperature.read().await;

    let mut history = vec![
        ChatMessage::system(system_prompt.as_str()),
        ChatMessage::user(message),
    ];

    let reply = crate::agent::loop_::run_tool_call_loop(
        state.provider.as_ref(),
        &mut history,
        state.tools_registry.as_ref(),
        state.observer.as_ref(),
        "gateway",
        &state.model.read().await,
        temperature,
    )
    .await?;

    Ok(normalize_gateway_reply(reply))
}

#[derive(Debug)]
struct SlidingWindowRateLimiter {
    limit_per_window: u32,
    window: Duration,
    requests: Mutex<HashMap<String, Vec<Instant>>>,
}

impl SlidingWindowRateLimiter {
    fn new(limit_per_window: u32, window: Duration) -> Self {
        Self {
            limit_per_window,
            window,
            requests: Mutex::new(HashMap::new()),
        }
    }

    fn allow(&self, key: &str) -> bool {
        if self.limit_per_window == 0 {
            return true;
        }

        let now = Instant::now();
        let cutoff = now.checked_sub(self.window).unwrap_or_else(Instant::now);

        let mut requests = self
            .requests
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let entry = requests.entry(key.to_owned()).or_default();
        entry.retain(|instant| *instant > cutoff);

        if entry.len() >= self.limit_per_window as usize {
            return false;
        }

        entry.push(now);
        true
    }
}

#[derive(Debug)]
pub struct GatewayRateLimiter {
    pair: SlidingWindowRateLimiter,
    webhook: SlidingWindowRateLimiter,
    vpn: SlidingWindowRateLimiter,
    diary: SlidingWindowRateLimiter,
    model_switch: SlidingWindowRateLimiter,
}

impl GatewayRateLimiter {
    fn new(pair_per_minute: u32, webhook_per_minute: u32) -> Self {
        let window = Duration::from_secs(RATE_LIMIT_WINDOW_SECS);
        Self {
            pair: SlidingWindowRateLimiter::new(pair_per_minute, window),
            webhook: SlidingWindowRateLimiter::new(webhook_per_minute, window),
            vpn: SlidingWindowRateLimiter::new(5, window),            // 5 VPN ops/min
            diary: SlidingWindowRateLimiter::new(20, window),          // 20 diary writes/min
            model_switch: SlidingWindowRateLimiter::new(3, window),    // 3 model switches/min
        }
    }

    fn allow_pair(&self, key: &str) -> bool {
        self.pair.allow(key)
    }

    fn allow_webhook(&self, key: &str) -> bool {
        self.webhook.allow(key)
    }

    pub fn allow_vpn(&self, key: &str) -> bool {
        self.vpn.allow(key)
    }

    pub fn allow_diary(&self, key: &str) -> bool {
        self.diary.allow(key)
    }

    pub fn allow_model_switch(&self, key: &str) -> bool {
        self.model_switch.allow(key)
    }
}

#[derive(Debug)]
pub struct IdempotencyStore {
    ttl: Duration,
    keys: Mutex<HashMap<String, Instant>>,
}

impl IdempotencyStore {
    fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            keys: Mutex::new(HashMap::new()),
        }
    }

    /// Returns true if this key is new and is now recorded.
    fn record_if_new(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut keys = self
            .keys
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        keys.retain(|_, seen_at| now.duration_since(*seen_at) < self.ttl);

        if keys.contains_key(key) {
            return false;
        }

        keys.insert(key.to_owned(), now);
        true
    }
}

/// Cryptographically secure OIDC state parameter store.
///
/// Generates random state tokens, stores them with a TTL, and validates
/// (consumes) them on callback. This prevents CSRF attacks in the OAuth flow.
#[derive(Debug)]
pub struct OidcStateStore {
    ttl: Duration,
    /// Maps state_token → (provider_id, created_at)
    states: Mutex<HashMap<String, (String, Instant)>>,
}

impl OidcStateStore {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            states: Mutex::new(HashMap::new()),
        }
    }

    /// Generate a cryptographically random state token and store it.
    /// Returns the generated token.
    pub fn generate(&self, provider_id: &str) -> String {
        use rand::Rng;
        use std::fmt::Write;
        let buf: [u8; 32] = rand::thread_rng().gen();
        let mut token = String::with_capacity(64);
        for byte in &buf {
            write!(token, "{byte:02x}").unwrap();
        }

        let mut states = self
            .states
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // Purge expired entries while we have the lock
        let now = Instant::now();
        states.retain(|_, (_, ts)| now.duration_since(*ts) < self.ttl);

        states.insert(token.clone(), (provider_id.to_owned(), now));
        token
    }

    /// Validate and consume a state token. Returns the provider_id if valid.
    /// The token is removed on successful validation (single-use).
    pub fn validate(&self, state_token: &str) -> Option<String> {
        let mut states = self
            .states
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let now = Instant::now();
        states.retain(|_, (_, ts)| now.duration_since(*ts) < self.ttl);

        states.remove(state_token).map(|(provider_id, _)| provider_id)
    }
}

fn client_key_from_headers(headers: &HeaderMap) -> String {
    for header_name in ["X-Forwarded-For", "X-Real-IP"] {
        if let Some(value) = headers.get(header_name).and_then(|v| v.to_str().ok()) {
            let first = value.split(',').next().unwrap_or("").trim();
            if !first.is_empty() {
                return first.to_owned();
            }
        }
    }
    "unknown".into()
}

/// Shared state for all axum handlers
#[derive(Clone)]
pub struct AppState {
    pub provider: Arc<dyn Provider>,
    pub observer: Arc<dyn Observer>,
    pub tools_registry: Arc<Vec<Box<dyn Tool>>>,
    pub system_prompt: Arc<tokio::sync::RwLock<String>>,
    pub model: Arc<tokio::sync::RwLock<String>>,
    pub temperature: Arc<tokio::sync::RwLock<f64>>,
    pub mem: Arc<dyn Memory>,
    pub auto_save: bool,
    pub webhook_secret: Option<Arc<str>>,
    pub pairing: Arc<PairingGuard>,
    pub rate_limiter: Arc<GatewayRateLimiter>,
    pub idempotency_store: Arc<IdempotencyStore>,
    pub whatsapp: Option<Arc<WhatsAppChannel>>,
    /// `WhatsApp` app secret for webhook signature verification (`X-Hub-Signature-256`)
    pub whatsapp_app_secret: Option<Arc<str>>,
    pub soul: Arc<tokio::sync::Mutex<crate::identity::Soul>>,
    pub voice_echo_enabled: Arc<std::sync::atomic::AtomicBool>,
    pub identity_config: Arc<crate::config::IdentityConfig>,
    pub vpn_manager: Arc<crate::network::VpnManager>,
    pub vault: Arc<crate::security::VaultManager>,
    pub audit: Arc<crate::security::AuditLogger>,
    pub adblock: Arc<crate::network::adblock::DnsBlocker>,
    pub stt: Arc<dyn crate::providers::stt::SttProvider>,
    pub public_url: String,
    pub oidc_states: Arc<OidcStateStore>,
    pub workspace_dir: std::path::PathBuf,
    pub config: Arc<tokio::sync::RwLock<Config>>,
    /// Monotonic start instant for uptime calculation.
    pub started_at: std::time::Instant,
    /// Confirmation gate for interactive approval flow.
    pub confirm_gate: Arc<crate::security::confirmation::ConfirmationGate>,
}

/// Run the HTTP gateway using axum with proper HTTP/1.1 compliance.
#[allow(clippy::too_many_lines)]
pub async fn run_gateway(host: &str, port: u16, config: Config) -> Result<()> {
    // ── Security: refuse public bind without tunnel or explicit opt-in ──
    if is_public_bind(host) && config.tunnel.provider == "none" && !config.gateway.allow_public_bind
    {
        anyhow::bail!(
            "🛑 Refusing to bind to {host} — gateway would be exposed to the internet.\n\
             Fix: use --host 127.0.0.1 (default), configure a tunnel, or set\n\
             [gateway] allow_public_bind = true in config.toml (NOT recommended)."
        );
    }
    
    let shared_config = Arc::new(tokio::sync::RwLock::new(config.clone()));

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_port = listener.local_addr()?.port();
    let display_addr = format!("{host}:{actual_port}");

    let provider: Arc<dyn Provider> = Arc::from(providers::create_resilient_provider(
        config.default_provider.as_deref().unwrap_or("openrouter"),
        config.api_key.as_deref(),
        &config.reliability,
    )?);

    let stt_key = providers::resolve_api_key(&config.stt.provider, config.api_key.as_deref())
        .ok_or_else(|| anyhow::anyhow!("Stt provider {} requires an API key. Please check your config or env vars.", config.stt.provider))?;
    let stt: Arc<dyn crate::providers::stt::SttProvider> = Arc::from(crate::providers::stt::create_stt_provider(
        &config.stt.provider,
        &stt_key,
        config.stt.model.clone(),
    )?);
    let model = config
        .default_model
        .clone()
        .unwrap_or_else(|| "anthropic/claude-sonnet-4".into());
    let temperature = config.default_temperature;
    let audit: Arc<crate::security::AuditLogger> = Arc::new(
        crate::security::AuditLogger::new(config.security.audit.clone(), config.workspace_dir.clone())?,
    );
    let mem: Arc<dyn Memory> = Arc::from(memory::create_memory(
        &config.memory,
        &config.workspace_dir,
        config.api_key.as_deref(),
        Arc::clone(&audit),
    )?);
    let observer: Arc<dyn Observer> =
        Arc::from(observability::create_observer(&config.observability));
    let runtime: Arc<dyn runtime::RuntimeAdapter> =
        Arc::from(runtime::create_runtime(&config.runtime)?);
    let adblock = Arc::new(crate::network::adblock::DnsBlocker::new());
    if let Err(e) = adblock.load_defaults().await {
        tracing::warn!("Failed to load adblock defaults: {e}");
    }
    
    let mut security = SecurityPolicy::from_config(
        &config.autonomy,
        &config.security,
        &config.workspace_dir,
    );
    let actor_name;
    {
        let mut soul = crate::identity::soul::Soul::new(&config.workspace_dir);
        if let Ok(()) = soul.load() {
            let trust = soul.max_trust_level();
            actor_name = soul.bindings.first().map(|b| format!("{}:{}", b.provider, b.id));
            tracing::info!(?trust, ?actor_name, bindings = soul.bindings.len(), "SIGIL: Identity resolved from SOUL.md");
            security.set_trust_level(trust);
        } else {
            actor_name = None;
        }
    }
    let security = Arc::new(security);

    let composio_key = if config.composio.enabled {
        config.composio.api_key.as_deref()
    } else {
        None
    };

    // Discover MCP tools from configured servers
    let mcp_tools = crate::mcp::discover_mcp_tools(
        &config.mcp,
        &security,
        &audit,
    ).await;

    let tools_registry = Arc::new(tools::all_tools_with_runtime(
        &security,
        runtime,
        Arc::clone(&mem),
        composio_key,
        &config.browser,
        &config.http_request,
        &config.workspace_dir,
        &config.agents,
        config.api_key.as_deref(),
        mcp_tools,
        Some(Arc::clone(&audit)),
        actor_name,
    ));
    let skills = crate::skills::load_skills(&config.workspace_dir);
    let tool_descs: Vec<(&str, &str)> = tools_registry
        .iter()
        .map(|tool| (tool.name(), tool.description()))
        .collect();

    let mut system_prompt = crate::channels::build_system_prompt(
        &config.workspace_dir,
        &model,
        &tool_descs,
        &skills,
        Some(&config.identity),
    );
    system_prompt.push_str(&crate::agent::loop_::build_tool_instructions(
        tools_registry.as_ref(),
    ));
    let system_prompt = Arc::new(tokio::sync::RwLock::new(system_prompt));

    // Extract webhook secret for authentication
    let webhook_secret: Option<Arc<str>> = config
        .channels_config
        .webhook
        .as_ref()
        .and_then(|w| w.secret.as_deref())
        .map(Arc::from);

    // WhatsApp channel (if configured)
    let whatsapp_channel: Option<Arc<WhatsAppChannel>> =
        config.channels_config.whatsapp.as_ref().map(|wa| {
            Arc::new(WhatsAppChannel::new(
                wa.access_token.clone(),
                wa.phone_number_id.clone(),
                wa.verify_token.clone(),
                wa.allowed_numbers.clone(),
            ))
        });

    // WhatsApp app secret for webhook signature verification
    // Priority: environment variable > config file
    let whatsapp_app_secret: Option<Arc<str>> = std::env::var("MYMOLT_WHATSAPP_APP_SECRET")
        .ok()
        .and_then(|secret| {
            let secret = secret.trim();
            (!secret.is_empty()).then(|| secret.to_owned())
        })
        .or_else(|| {
            config.channels_config.whatsapp.as_ref().and_then(|wa| {
                wa.app_secret
                    .as_deref()
                    .map(str::trim)
                    .filter(|secret| !secret.is_empty())
                    .map(ToOwned::to_owned)
            })
        })
        .map(Arc::from);

    // ── Pairing guard ──────────────────────────────────────
    let pairing = Arc::new(PairingGuard::new(
        config.gateway.require_pairing,
        &config.gateway.paired_tokens,
    ));
    let rate_limiter = Arc::new(GatewayRateLimiter::new(
        config.gateway.pair_rate_limit_per_minute,
        config.gateway.webhook_rate_limit_per_minute,
    ));
    let idempotency_store = Arc::new(IdempotencyStore::new(Duration::from_secs(
        config.gateway.idempotency_ttl_secs.max(1),
    )));

    // ── Tunnel ────────────────────────────────────────────────
    let tunnel = crate::tunnel::create_tunnel(&config.tunnel)?;
    let mut tunnel_url: Option<String> = None;

    if let Some(ref tun) = tunnel {
        println!("🔗 Starting {} tunnel...", tun.name());
        match tun.start(host, actual_port).await {
            Ok(url) => {
                println!("🌐 Tunnel active: {url}");
                tunnel_url = Some(url);
            }
            Err(e) => {
                println!("⚠️  Tunnel failed to start: {e}");
                println!("   Falling back to local-only mode.");
            }
        }
    }

    println!("🦎 MyMolt Gateway listening on http://{display_addr}");
    if let Some(ref url) = tunnel_url {
        println!("  🌐 Public URL: {url}");
    }
    println!("  POST /pair      — pair a new client (X-Pairing-Code header)");
    println!("  POST /webhook   — {{\"message\": \"your prompt\"}}");
    if whatsapp_channel.is_some() {
        println!("  GET  /whatsapp  — Meta webhook verification");
        println!("  POST /whatsapp  — WhatsApp message webhook");
    }
    println!("  GET  /health    — health check");
    if let Some(code) = pairing.pairing_code() {
        println!();
        println!("  🔐 PAIRING REQUIRED — use this one-time code:");
        println!("     ┌──────────────┐");
        println!("     │  {code}  │");
        println!("     └──────────────┘");
        println!("     Send: POST /pair with header X-Pairing-Code: {code}");
    } else if pairing.require_pairing() {
        println!("  🔒 Pairing: ACTIVE (bearer token required)");
    } else {
        println!("  ⚠️  Pairing: DISABLED (all requests accepted)");
    }
    if webhook_secret.is_some() {
        println!("  🔒 Webhook secret: ENABLED");
    }
    println!("  Press Ctrl+C to stop.\n");

    crate::health::mark_component_ok("gateway");

    // Build shared state
    let state = AppState {
        provider,
        observer,
        tools_registry,
        system_prompt,
        model: Arc::new(tokio::sync::RwLock::new(model)),
        temperature: Arc::new(tokio::sync::RwLock::new(temperature)),
        mem,
        auto_save: config.memory.auto_save,
        webhook_secret,
        pairing,
        rate_limiter,
        idempotency_store,
        whatsapp: whatsapp_channel,
        whatsapp_app_secret,
        soul: Arc::new(tokio::sync::Mutex::new({
            let mut s = crate::identity::Soul::new(&config.workspace_dir);
            if let Err(e) = s.load() {
                tracing::warn!("Failed to load Soul: {e}");
            }
            s
        })),
        voice_echo_enabled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        identity_config: Arc::new(config.identity),
        vpn_manager: Arc::new(crate::network::VpnManager::new(
            &config.workspace_dir.join("network").join("wg0.conf")
        )),
        vault: Arc::new(crate::security::VaultManager::new(&config.workspace_dir)),
        audit,
        adblock,
        stt,
        // Use tunnel URL if available, otherwise host:port
        public_url: tunnel_url.unwrap_or_else(|| format!("http://{display_addr}")),
        oidc_states: Arc::new(OidcStateStore::new(Duration::from_secs(600))), // 10 min TTL
        workspace_dir: config.workspace_dir.clone(),
        config: Arc::clone(&shared_config),
        started_at: std::time::Instant::now(),
        confirm_gate: crate::security::confirmation::ConfirmationGate::new(30),
    };


use tower_http::compression::CompressionLayer;
use tower_http::cors::{CorsLayer, AllowOrigin};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use axum::http::HeaderValue;

    // ── Security Headers Middleware ───────────────────────────
    let security_headers = tower::ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        // Basic CSP to prevent XSS/Injection
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self' ws: wss:;"),
        ));

    // ── CORS (Restricted) ─────────────────────────────────────
    // Compute allowed origins based on config
    let mut allowed_origins = vec![
        "http://localhost:5173".parse::<HeaderValue>().unwrap(),
        "http://localhost:3000".parse::<HeaderValue>().unwrap(),
    ];
    
    // Add public URL if valid
    if let Ok(origin) = state.public_url.parse::<HeaderValue>() {
        allowed_origins.push(origin);
    }

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    // Build router with middleware
    let app = Router::new()
        .route("/health", get(handle_health))
        .route("/pair", post(handle_pair))
        .route("/webhook", post(handle_webhook))
        .route("/whatsapp", get(handle_whatsapp_verify))
        .route("/whatsapp", post(handle_whatsapp_message))
        .merge(api::routes()) // Merge API routes
        .with_state(state)
        .layer(RequestBodyLimitLayer::new(MAX_BODY_SIZE))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(REQUEST_TIMEOUT_SECS),
        ))
        .layer(CompressionLayer::new())
        .layer(security_headers)
        .layer(cors);

    // Run the server
    // SPA Fallback: serve index.html for unknown routes (client-side routing)
    let app = app.fallback_service(
         ServeDir::new("frontend/dist")
            .not_found_service(ServeFile::new("frontend/dist/index.html"))
    );

    axum::serve(listener, app).await?;

    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
// AXUM HANDLERS
// ══════════════════════════════════════════════════════════════════════════════

/// GET /health — always public (no secrets leaked)
async fn handle_health(State(state): State<AppState>) -> impl IntoResponse {
    let body = serde_json::json!({
        "status": "ok",
        "paired": state.pairing.is_paired(),
        "pairing_enabled": state.pairing.require_pairing(),
        "runtime": crate::health::snapshot_json(),
    });
    Json(body)
}

/// POST /pair — exchange one-time code for bearer token
async fn handle_pair(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let client_key = client_key_from_headers(&headers);
    if !state.rate_limiter.allow_pair(&client_key) {
        tracing::warn!("/pair rate limit exceeded for key: {client_key}");
        let err = serde_json::json!({
            "error": "Too many pairing requests. Please retry later.",
            "retry_after": RATE_LIMIT_WINDOW_SECS,
        });
        return (StatusCode::TOO_MANY_REQUESTS, Json(err));
    }

    let code = headers
        .get("X-Pairing-Code")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    match state.pairing.try_pair(code) {
        Ok(Some(token)) => {
            tracing::info!("🔐 New client paired successfully");
            let body = serde_json::json!({
                "paired": true,
                "token": token,
                "message": "Save this token — use it as Authorization: Bearer <token>"
            });
            (StatusCode::OK, Json(body))
        }
        Ok(None) => {
            tracing::warn!("🔐 Pairing attempt with invalid code");
            let err = serde_json::json!({"error": "Invalid pairing code"});
            (StatusCode::FORBIDDEN, Json(err))
        }
        Err(lockout_secs) => {
            tracing::warn!(
                "🔐 Pairing locked out — too many failed attempts ({lockout_secs}s remaining)"
            );
            let err = serde_json::json!({
                "error": format!("Too many failed attempts. Try again in {lockout_secs}s."),
                "retry_after": lockout_secs
            });
            (StatusCode::TOO_MANY_REQUESTS, Json(err))
        }
    }
}

/// Webhook request body
#[derive(serde::Deserialize)]
pub struct WebhookBody {
    pub message: String,
}

/// POST /webhook — main webhook endpoint
async fn handle_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<WebhookBody>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    let client_key = client_key_from_headers(&headers);
    if !state.rate_limiter.allow_webhook(&client_key) {
        tracing::warn!("/webhook rate limit exceeded for key: {client_key}");
        let err = serde_json::json!({
            "error": "Too many webhook requests. Please retry later.",
            "retry_after": RATE_LIMIT_WINDOW_SECS,
        });
        return (StatusCode::TOO_MANY_REQUESTS, Json(err));
    }

    // ── Bearer token auth (pairing) ──
    if state.pairing.require_pairing() {
        let auth = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let token = auth.strip_prefix("Bearer ").unwrap_or("");
        if !state.pairing.is_authenticated(token) {
            tracing::warn!("Webhook: rejected — not paired / invalid bearer token");
            let err = serde_json::json!({
                "error": "Unauthorized — pair first via POST /pair, then send Authorization: Bearer <token>"
            });
            return (StatusCode::UNAUTHORIZED, Json(err));
        }
    }

    // ── Webhook secret auth (optional, additional layer) ──
    if let Some(ref secret) = state.webhook_secret {
        let header_val = headers
            .get("X-Webhook-Secret")
            .and_then(|v| v.to_str().ok());
        match header_val {
            Some(val) if constant_time_eq(val, secret.as_ref()) => {}
            _ => {
                tracing::warn!("Webhook: rejected request — invalid or missing X-Webhook-Secret");
                let err = serde_json::json!({"error": "Unauthorized — invalid or missing X-Webhook-Secret header"});
                return (StatusCode::UNAUTHORIZED, Json(err));
            }
        }
    }

    // ── Parse body ──
    let Json(webhook_body) = match body {
        Ok(b) => b,
        Err(e) => {
            let err = serde_json::json!({
                "error": format!("Invalid JSON: {e}. Expected: {{\"message\": \"...\"}}")
            });
            return (StatusCode::BAD_REQUEST, Json(err));
        }
    };

    // ── Idempotency (optional) ──
    if let Some(idempotency_key) = headers
        .get("X-Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if !state.idempotency_store.record_if_new(idempotency_key) {
            tracing::info!("Webhook duplicate ignored (idempotency key: {idempotency_key})");
            let body = serde_json::json!({
                "status": "duplicate",
                "idempotent": true,
                "message": "Request already processed for this idempotency key"
            });
            return (StatusCode::OK, Json(body));
        }
    }

    let message = &webhook_body.message;

    if state.auto_save {
        let key = webhook_memory_key();
        let _ = state
            .mem
            .store(&key, message, MemoryCategory::Conversation)
            .await;
    }

    match gateway_agent_reply(&state, message).await {
        Ok(reply) => {
            let model = state.model.read().await.clone();
            let body = serde_json::json!({"response": reply, "model": model});
            (StatusCode::OK, Json(body))
        }
        Err(e) => {
            tracing::error!(
                "Webhook provider error: {}",
                providers::sanitize_api_error(&e.to_string())
            );
            let err = serde_json::json!({"error": "LLM request failed"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
        }
    }
}

/// `WhatsApp` verification query params
#[derive(serde::Deserialize)]
pub struct WhatsAppVerifyQuery {
    #[serde(rename = "hub.mode")]
    pub mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    pub verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    pub challenge: Option<String>,
}

/// GET /whatsapp — Meta webhook verification
async fn handle_whatsapp_verify(
    State(state): State<AppState>,
    Query(params): Query<WhatsAppVerifyQuery>,
) -> impl IntoResponse {
    let Some(ref wa) = state.whatsapp else {
        return (StatusCode::NOT_FOUND, "WhatsApp not configured".to_string());
    };

    // Verify the token matches (constant-time comparison to prevent timing attacks)
    let token_matches = params
        .verify_token
        .as_deref()
        .is_some_and(|t| constant_time_eq(t, wa.verify_token()));
    if params.mode.as_deref() == Some("subscribe") && token_matches {
        if let Some(ch) = params.challenge {
            tracing::info!("WhatsApp webhook verified successfully");
            return (StatusCode::OK, ch);
        }
        return (StatusCode::BAD_REQUEST, "Missing hub.challenge".to_string());
    }

    tracing::warn!("WhatsApp webhook verification failed — token mismatch");
    (StatusCode::FORBIDDEN, "Forbidden".to_string())
}

/// Verify `WhatsApp` webhook signature (`X-Hub-Signature-256`).
/// Returns true if the signature is valid, false otherwise.
/// See: <https://developers.facebook.com/docs/graph-api/webhooks/getting-started#verification-requests>
pub fn verify_whatsapp_signature(app_secret: &str, body: &[u8], signature_header: &str) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    // Signature format: "sha256=<hex_signature>"
    let Some(hex_sig) = signature_header.strip_prefix("sha256=") else {
        return false;
    };

    // Decode hex signature
    let Ok(expected) = hex::decode(hex_sig) else {
        return false;
    };

    // Compute HMAC-SHA256
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(app_secret.as_bytes()) else {
        return false;
    };
    mac.update(body);

    // Constant-time comparison
    mac.verify_slice(&expected).is_ok()
}

/// POST /whatsapp — incoming message webhook
async fn handle_whatsapp_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let Some(ref wa) = state.whatsapp else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "WhatsApp not configured"})),
        );
    };

    // ── Security: Verify X-Hub-Signature-256 if app_secret is configured ──
    if let Some(ref app_secret) = state.whatsapp_app_secret {
        let signature = headers
            .get("X-Hub-Signature-256")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !verify_whatsapp_signature(app_secret, &body, signature) {
            tracing::warn!(
                "WhatsApp webhook signature verification failed (signature: {})",
                if signature.is_empty() {
                    "missing"
                } else {
                    "invalid"
                }
            );
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid signature"})),
            );
        }
    }

    // Parse JSON body
    let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&body) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid JSON payload"})),
        );
    };

    // Parse messages from the webhook payload
    let messages = wa.parse_webhook_payload(&payload);

    if messages.is_empty() {
        // Acknowledge the webhook even if no messages (could be status updates)
        return (StatusCode::OK, Json(serde_json::json!({"status": "ok"})));
    }

    // Process each message
    for msg in &messages {
        tracing::info!(
            "WhatsApp message from {}: {}",
            msg.sender,
            truncate_with_ellipsis(&msg.content, 50)
        );

        // Auto-save to memory
        if state.auto_save {
            let key = whatsapp_memory_key(msg);
            let _ = state
                .mem
                .store(&key, &msg.content, MemoryCategory::Conversation)
                .await;
        }

        // Call the LLM
        match gateway_agent_reply(&state, &msg.content).await {
            Ok(reply) => {
                // Send reply via WhatsApp
                if let Err(e) = wa.send(&reply, &msg.sender).await {
                    tracing::error!("Failed to send WhatsApp reply: {e}");
                }
            }
            Err(e) => {
                tracing::error!("LLM error for WhatsApp message: {e:#}");
                let _ = wa
                    .send(
                        "Sorry, I couldn't process your message right now.",
                        &msg.sender,
                    )
                    .await;
            }
        }
    }

    // Acknowledge the webhook
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::traits::ChannelMessage;
    use crate::memory::{Memory, MemoryCategory, MemoryEntry};
    use crate::providers::Provider;
    use async_trait::async_trait;
    use axum::http::HeaderValue;
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    #[test]
    fn security_body_limit_is_64kb() {
        assert_eq!(MAX_BODY_SIZE, 65_536);
    }

    #[test]
    fn security_timeout_is_30_seconds() {
        assert_eq!(REQUEST_TIMEOUT_SECS, 30);
    }

    #[test]
    fn webhook_body_requires_message_field() {
        let valid = r#"{"message": "hello"}"#;
        let parsed: Result<WebhookBody, _> = serde_json::from_str(valid);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().message, "hello");

        let missing = r#"{"other": "field"}"#;
        let parsed: Result<WebhookBody, _> = serde_json::from_str(missing);
        assert!(parsed.is_err());
    }

    #[test]
    fn whatsapp_query_fields_are_optional() {
        let q = WhatsAppVerifyQuery {
            mode: None,
            verify_token: None,
            challenge: None,
        };
        assert!(q.mode.is_none());
    }

    #[test]
    fn app_state_is_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<AppState>();
    }

    #[test]
    fn gateway_rate_limiter_blocks_after_limit() {
        let limiter = GatewayRateLimiter::new(2, 2);
        assert!(limiter.allow_pair("127.0.0.1"));
        assert!(limiter.allow_pair("127.0.0.1"));
        assert!(!limiter.allow_pair("127.0.0.1"));
    }

    #[test]
    fn idempotency_store_rejects_duplicate_key() {
        let store = IdempotencyStore::new(Duration::from_secs(30));
        assert!(store.record_if_new("req-1"));
        assert!(!store.record_if_new("req-1"));
        assert!(store.record_if_new("req-2"));
    }

    #[test]
    fn webhook_memory_key_is_unique() {
        let key1 = webhook_memory_key();
        let key2 = webhook_memory_key();

        assert!(key1.starts_with("webhook_msg_"));
        assert!(key2.starts_with("webhook_msg_"));
        assert_ne!(key1, key2);
    }

    #[test]
    fn whatsapp_memory_key_includes_sender_and_message_id() {
        let msg = ChannelMessage {
            id: "wamid-123".into(),
            sender: "+1234567890".into(),
            content: "hello".into(),
            channel: "whatsapp".into(),
            timestamp: 1,
        };

        let key = whatsapp_memory_key(&msg);
        assert_eq!(key, "whatsapp_+1234567890_wamid-123");
    }

    #[derive(Default)]
    struct MockMemory;

    #[async_trait]
    impl Memory for MockMemory {
        fn name(&self) -> &str {
            "mock"
        }

        async fn store(
            &self,
            _key: &str,
            _content: &str,
            _category: MemoryCategory,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn recall(&self, _query: &str, _limit: usize) -> anyhow::Result<Vec<MemoryEntry>> {
            Ok(Vec::new())
        }

        async fn get(&self, _key: &str) -> anyhow::Result<Option<MemoryEntry>> {
            Ok(None)
        }

        async fn list(
            &self,
            _category: Option<&MemoryCategory>,
        ) -> anyhow::Result<Vec<MemoryEntry>> {
            Ok(Vec::new())
        }

        async fn forget(&self, _key: &str) -> anyhow::Result<bool> {
            Ok(false)
        }

        async fn count(&self) -> anyhow::Result<usize> {
            Ok(0)
        }

        async fn health_check(&self) -> bool {
            true
        }
    }

    #[derive(Default)]
    struct MockProvider {
        calls: AtomicUsize,
    }

    #[async_trait]
    impl Provider for MockProvider {
        async fn chat_with_system(
            &self,
            _system_prompt: Option<&str>,
            _message: &str,
            _model: &str,
            _temperature: f64,
        ) -> anyhow::Result<crate::providers::ChatResponse> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(crate::providers::ChatResponse::with_text("ok"))
        }
    }

    #[derive(Default)]
    struct TrackingMemory {
        keys: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl Memory for TrackingMemory {
        fn name(&self) -> &str {
            "tracking"
        }

        async fn store(
            &self,
            key: &str,
            _content: &str,
            _category: MemoryCategory,
        ) -> anyhow::Result<()> {
            self.keys
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(key.to_string());
            Ok(())
        }

        async fn recall(&self, _query: &str, _limit: usize) -> anyhow::Result<Vec<MemoryEntry>> {
            Ok(Vec::new())
        }

        async fn get(&self, _key: &str) -> anyhow::Result<Option<MemoryEntry>> {
            Ok(None)
        }

        async fn list(
            &self,
            _category: Option<&MemoryCategory>,
        ) -> anyhow::Result<Vec<MemoryEntry>> {
            Ok(Vec::new())
        }

        async fn forget(&self, _key: &str) -> anyhow::Result<bool> {
            Ok(false)
        }

        async fn count(&self) -> anyhow::Result<usize> {
            let size = self
                .keys
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .len();
            Ok(size)
        }

        async fn health_check(&self) -> bool {
            true
        }
    }

    fn test_app_state(
        provider: Arc<dyn Provider>,
        memory: Arc<dyn Memory>,
        auto_save: bool,
    ) -> AppState {
        let tmp = tempfile::tempdir().unwrap();
        let audit = Arc::new(
            crate::security::AuditLogger::new(
                crate::config::AuditConfig::default(),
                tmp.path().to_path_buf(),
            )
            .unwrap(),
        );
        AppState {
            provider,
            observer: Arc::new(crate::observability::NoopObserver),
            tools_registry: Arc::new(Vec::new()),
            system_prompt: Arc::new(tokio::sync::RwLock::new("test-system-prompt".into())),
            model: Arc::new(tokio::sync::RwLock::new("test-model".into())),
            temperature: Arc::new(tokio::sync::RwLock::new(0.0)),
            mem: memory,
            auto_save,
            webhook_secret: None,
            pairing: Arc::new(PairingGuard::new(false, &[])),
            rate_limiter: Arc::new(GatewayRateLimiter::new(100, 100)),
            idempotency_store: Arc::new(IdempotencyStore::new(Duration::from_secs(300))),
            whatsapp: None,
            whatsapp_app_secret: None,
            soul: Arc::new(tokio::sync::Mutex::new(crate::identity::Soul::new(tmp.path()))),
            voice_echo_enabled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            identity_config: Arc::new(crate::config::IdentityConfig::default()),
            vpn_manager: Arc::new(crate::network::VpnManager::new(tmp.path())),
            vault: Arc::new(crate::security::VaultManager::new(tmp.path())),
            audit,
            adblock: Arc::new(crate::network::adblock::DnsBlocker::new()),
            stt: Arc::new(crate::providers::stt::MockSttProvider::new("test transcription")),
            public_url: "http://localhost:3000".into(),
            oidc_states: Arc::new(OidcStateStore::new(Duration::from_secs(600))),
            workspace_dir: tmp.path().to_path_buf(),
            config: Arc::new(tokio::sync::RwLock::new(crate::config::Config::default())),
            started_at: std::time::Instant::now(),
            confirm_gate: crate::security::confirmation::ConfirmationGate::new(5),
        }
    }

    #[tokio::test]
    async fn webhook_idempotency_skips_duplicate_provider_calls() {
        let provider_impl = Arc::new(MockProvider::default());
        let provider: Arc<dyn Provider> = provider_impl.clone();
        let memory: Arc<dyn Memory> = Arc::new(MockMemory);

        let state = test_app_state(provider, memory, false);

        let mut headers = HeaderMap::new();
        headers.insert("X-Idempotency-Key", HeaderValue::from_static("abc-123"));

        let body = Ok(Json(WebhookBody {
            message: "hello".into(),
        }));
        let first = handle_webhook(State(state.clone()), headers.clone(), body)
            .await
            .into_response();
        assert_eq!(first.status(), StatusCode::OK);

        let body = Ok(Json(WebhookBody {
            message: "hello".into(),
        }));
        let second = handle_webhook(State(state), headers, body)
            .await
            .into_response();
        assert_eq!(second.status(), StatusCode::OK);

        let payload = second.into_body().collect().await.unwrap().to_bytes();
        let parsed: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        assert_eq!(parsed["status"], "duplicate");
        assert_eq!(parsed["idempotent"], true);
        assert_eq!(provider_impl.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn webhook_autosave_stores_distinct_keys_per_request() {
        let provider_impl = Arc::new(MockProvider::default());
        let provider: Arc<dyn Provider> = provider_impl.clone();

        let tracking_impl = Arc::new(TrackingMemory::default());
        let memory: Arc<dyn Memory> = tracking_impl.clone();

        let state = test_app_state(provider, memory, true);

        let headers = HeaderMap::new();

        let body1 = Ok(Json(WebhookBody {
            message: "hello one".into(),
        }));
        let first = handle_webhook(State(state.clone()), headers.clone(), body1)
            .await
            .into_response();
        assert_eq!(first.status(), StatusCode::OK);

        let body2 = Ok(Json(WebhookBody {
            message: "hello two".into(),
        }));
        let second = handle_webhook(State(state), headers, body2)
            .await
            .into_response();
        assert_eq!(second.status(), StatusCode::OK);

        let keys = tracking_impl
            .keys
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        assert_eq!(keys.len(), 2);
        assert_ne!(keys[0], keys[1]);
        assert!(keys[0].starts_with("webhook_msg_"));
        assert!(keys[1].starts_with("webhook_msg_"));
        assert_eq!(provider_impl.calls.load(Ordering::SeqCst), 2);
    }

    #[derive(Default)]
    struct StructuredToolCallProvider {
        calls: AtomicUsize,
    }

    #[async_trait]
    impl Provider for StructuredToolCallProvider {
        async fn chat_with_system(
            &self,
            _system_prompt: Option<&str>,
            _message: &str,
            _model: &str,
            _temperature: f64,
        ) -> anyhow::Result<crate::providers::ChatResponse> {
            let turn = self.calls.fetch_add(1, Ordering::SeqCst);

            if turn == 0 {
                return Ok(crate::providers::ChatResponse {
                    text: Some("Running tool...".into()),
                    tool_calls: vec![crate::providers::ToolCall {
                        id: "call_1".into(),
                        name: "mock_tool".into(),
                        arguments: r#"{"query":"gateway"}"#.into(),
                    }],
                });
            }

            Ok(crate::providers::ChatResponse::with_text(
                "Gateway tool result ready.",
            ))
        }
    }

    struct MockTool {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            "mock_tool"
        }

        fn description(&self) -> &str {
            "Mock tool for gateway tests"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                },
                "required": ["query"]
            })
        }

        async fn execute(
            &self,
            args: serde_json::Value,
        ) -> anyhow::Result<crate::tools::ToolResult> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            assert_eq!(args["query"], "gateway");

            Ok(crate::tools::ToolResult {
                success: true,
                output: "ok".into(),
                error: None,
            })
        }
    }

    #[tokio::test]
    async fn webhook_executes_structured_tool_calls() {
        let provider_impl = Arc::new(StructuredToolCallProvider::default());
        let provider: Arc<dyn Provider> = provider_impl.clone();
        let memory: Arc<dyn Memory> = Arc::new(MockMemory);

        let tool_calls = Arc::new(AtomicUsize::new(0));
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(MockTool {
            calls: Arc::clone(&tool_calls),
        })];

        let mut state = test_app_state(provider, memory, false);
        state.tools_registry = Arc::new(tools);

        let response = handle_webhook(
            State(state),
            HeaderMap::new(),
            Ok(Json(WebhookBody {
                message: "please use tool".into(),
            })),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let payload = response.into_body().collect().await.unwrap().to_bytes();
        let parsed: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        assert_eq!(parsed["response"], "Gateway tool result ready.");
        assert_eq!(tool_calls.load(Ordering::SeqCst), 1);
        assert_eq!(provider_impl.calls.load(Ordering::SeqCst), 2);
    }

    // ══════════════════════════════════════════════════════════
    // WhatsApp Signature Verification Tests (CWE-345 Prevention)
    // ══════════════════════════════════════════════════════════

    fn compute_whatsapp_signature_hex(secret: &str, body: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        hex::encode(mac.finalize().into_bytes())
    }

    fn compute_whatsapp_signature_header(secret: &str, body: &[u8]) -> String {
        format!("sha256={}", compute_whatsapp_signature_hex(secret, body))
    }

    #[test]
    fn whatsapp_signature_valid() {
        // Test with known values
        let app_secret = "test_secret_key";
        let body = b"test body content";

        let signature_header = compute_whatsapp_signature_header(app_secret, body);

        assert!(verify_whatsapp_signature(
            app_secret,
            body,
            &signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_invalid_wrong_secret() {
        let app_secret = "correct_secret";
        let wrong_secret = "wrong_secret";
        let body = b"test body content";

        let signature_header = compute_whatsapp_signature_header(wrong_secret, body);

        assert!(!verify_whatsapp_signature(
            app_secret,
            body,
            &signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_invalid_wrong_body() {
        let app_secret = "test_secret";
        let original_body = b"original body";
        let tampered_body = b"tampered body";

        let signature_header = compute_whatsapp_signature_header(app_secret, original_body);

        // Verify with tampered body should fail
        assert!(!verify_whatsapp_signature(
            app_secret,
            tampered_body,
            &signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_missing_prefix() {
        let app_secret = "test_secret";
        let body = b"test body";

        // Signature without "sha256=" prefix
        let signature_header = "abc123def456";

        assert!(!verify_whatsapp_signature(
            app_secret,
            body,
            signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_empty_header() {
        let app_secret = "test_secret";
        let body = b"test body";

        assert!(!verify_whatsapp_signature(app_secret, body, ""));
    }

    #[test]
    fn whatsapp_signature_invalid_hex() {
        let app_secret = "test_secret";
        let body = b"test body";

        // Invalid hex characters
        let signature_header = "sha256=not_valid_hex_zzz";

        assert!(!verify_whatsapp_signature(
            app_secret,
            body,
            signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_empty_body() {
        let app_secret = "test_secret";
        let body = b"";

        let signature_header = compute_whatsapp_signature_header(app_secret, body);

        assert!(verify_whatsapp_signature(
            app_secret,
            body,
            &signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_unicode_body() {
        let app_secret = "test_secret";
        let body = "Hello 🦀 世界".as_bytes();

        let signature_header = compute_whatsapp_signature_header(app_secret, body);

        assert!(verify_whatsapp_signature(
            app_secret,
            body,
            &signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_json_payload() {
        let app_secret = "my_app_secret_from_meta";
        let body = br#"{"entry":[{"changes":[{"value":{"messages":[{"from":"1234567890","text":{"body":"Hello"}}]}}]}]}"#;

        let signature_header = compute_whatsapp_signature_header(app_secret, body);

        assert!(verify_whatsapp_signature(
            app_secret,
            body,
            &signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_case_sensitive_prefix() {
        let app_secret = "test_secret";
        let body = b"test body";

        let hex_sig = compute_whatsapp_signature_hex(app_secret, body);

        // Wrong case prefix should fail
        let wrong_prefix = format!("SHA256={hex_sig}");
        assert!(!verify_whatsapp_signature(app_secret, body, &wrong_prefix));

        // Correct prefix should pass
        let correct_prefix = format!("sha256={hex_sig}");
        assert!(verify_whatsapp_signature(app_secret, body, &correct_prefix));
    }

    #[test]
    fn whatsapp_signature_truncated_hex() {
        let app_secret = "test_secret";
        let body = b"test body";

        let hex_sig = compute_whatsapp_signature_hex(app_secret, body);
        let truncated = &hex_sig[..32]; // Only half the signature
        let signature_header = format!("sha256={truncated}");

        assert!(!verify_whatsapp_signature(
            app_secret,
            body,
            &signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_extra_bytes() {
        let app_secret = "test_secret";
        let body = b"test body";

        let hex_sig = compute_whatsapp_signature_hex(app_secret, body);
        let extended = format!("{hex_sig}deadbeef");
        let signature_header = format!("sha256={extended}");

        assert!(!verify_whatsapp_signature(
            app_secret,
            body,
            &signature_header
        ));
    }

    // ══════════════════════════════════════════════════════════
    // SlidingWindowRateLimiter Tests
    // ══════════════════════════════════════════════════════════

    #[test]
    fn rate_limiter_allows_within_limit() {
        let rl = SlidingWindowRateLimiter::new(3, Duration::from_secs(60));
        assert!(rl.allow("client_a"));
        assert!(rl.allow("client_a"));
        assert!(rl.allow("client_a"));
    }

    #[test]
    fn rate_limiter_blocks_over_limit() {
        let rl = SlidingWindowRateLimiter::new(2, Duration::from_secs(60));
        assert!(rl.allow("client_a"));
        assert!(rl.allow("client_a"));
        assert!(!rl.allow("client_a")); // 3rd request → denied
    }

    #[test]
    fn rate_limiter_tracks_keys_independently() {
        let rl = SlidingWindowRateLimiter::new(1, Duration::from_secs(60));
        assert!(rl.allow("client_a")); // a: 1/1
        assert!(rl.allow("client_b")); // b: 1/1 (separate key)
        assert!(!rl.allow("client_a")); // a: over limit
        assert!(!rl.allow("client_b")); // b: over limit
    }

    #[test]
    fn rate_limiter_zero_limit_allows_all() {
        let rl = SlidingWindowRateLimiter::new(0, Duration::from_secs(60));
        for _ in 0..100 {
            assert!(rl.allow("any_client"));
        }
    }

    #[test]
    fn rate_limiter_limit_of_one() {
        let rl = SlidingWindowRateLimiter::new(1, Duration::from_secs(60));
        assert!(rl.allow("x"));
        assert!(!rl.allow("x"));
    }

    #[test]
    fn rate_limiter_empty_key_works() {
        let rl = SlidingWindowRateLimiter::new(2, Duration::from_secs(60));
        assert!(rl.allow(""));
        assert!(rl.allow(""));
        assert!(!rl.allow(""));
    }

    #[test]
    fn rate_limiter_unicode_key_works() {
        let rl = SlidingWindowRateLimiter::new(1, Duration::from_secs(60));
        assert!(rl.allow("用户🦀"));
        assert!(!rl.allow("用户🦀"));
    }

    // ══════════════════════════════════════════════════════════
    // GatewayRateLimiter Tests
    // ══════════════════════════════════════════════════════════

    #[test]
    fn gateway_rate_limiter_creates_with_limits() {
        let grl = GatewayRateLimiter::new(10, 20);
        // Should allow initial requests
        assert!(grl.allow_pair("user1"));
        assert!(grl.allow_webhook("wh1"));
        assert!(grl.allow_vpn("vpn1"));
        assert!(grl.allow_diary("diary1"));
        assert!(grl.allow_model_switch("ms1"));
    }

    #[test]
    fn gateway_rate_limiter_vpn_limit_is_5() {
        let grl = GatewayRateLimiter::new(100, 100);
        for _ in 0..5 {
            assert!(grl.allow_vpn("user"));
        }
        assert!(!grl.allow_vpn("user")); // 6th → denied
    }

    #[test]
    fn gateway_rate_limiter_model_switch_limit_is_3() {
        let grl = GatewayRateLimiter::new(100, 100);
        for _ in 0..3 {
            assert!(grl.allow_model_switch("user"));
        }
        assert!(!grl.allow_model_switch("user")); // 4th → denied
    }

    #[test]
    fn gateway_rate_limiter_diary_limit_is_20() {
        let grl = GatewayRateLimiter::new(100, 100);
        for _ in 0..20 {
            assert!(grl.allow_diary("user"));
        }
        assert!(!grl.allow_diary("user")); // 21st → denied
    }

    // ══════════════════════════════════════════════════════════
    // IdempotencyStore Tests
    // ══════════════════════════════════════════════════════════

    #[test]
    fn idempotency_first_key_is_new() {
        let store = IdempotencyStore::new(Duration::from_secs(300));
        assert!(store.record_if_new("request-1"));
    }

    #[test]
    fn idempotency_duplicate_key_is_not_new() {
        let store = IdempotencyStore::new(Duration::from_secs(300));
        assert!(store.record_if_new("request-1"));
        assert!(!store.record_if_new("request-1"));
    }

    #[test]
    fn idempotency_different_keys_are_new() {
        let store = IdempotencyStore::new(Duration::from_secs(300));
        assert!(store.record_if_new("request-1"));
        assert!(store.record_if_new("request-2"));
        assert!(store.record_if_new("request-3"));
    }

    #[test]
    fn idempotency_empty_key_works() {
        let store = IdempotencyStore::new(Duration::from_secs(300));
        assert!(store.record_if_new(""));
        assert!(!store.record_if_new(""));
    }

    #[test]
    fn idempotency_many_keys_no_false_positives() {
        let store = IdempotencyStore::new(Duration::from_secs(300));
        for i in 0..1000 {
            let key = format!("req-{i}");
            assert!(store.record_if_new(&key), "Key {key} should be new");
        }
        // Verify duplicates are detected
        for i in 0..1000 {
            let key = format!("req-{i}");
            assert!(!store.record_if_new(&key), "Key {key} should be duplicate");
        }
    }

    // ══════════════════════════════════════════════════════════
    // OidcStateStore Tests (CSRF Prevention)
    // ══════════════════════════════════════════════════════════

    #[test]
    fn oidc_state_generates_hex_token() {
        let store = OidcStateStore::new(Duration::from_secs(300));
        let token = store.generate("google");
        assert_eq!(token.len(), 64); // 32 bytes = 64 hex chars
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn oidc_state_tokens_are_unique() {
        let store = OidcStateStore::new(Duration::from_secs(300));
        let t1 = store.generate("google");
        let t2 = store.generate("google");
        let t3 = store.generate("github");
        assert_ne!(t1, t2);
        assert_ne!(t2, t3);
    }

    #[test]
    fn oidc_state_validate_returns_provider() {
        let store = OidcStateStore::new(Duration::from_secs(300));
        let token = store.generate("google");
        let result = store.validate(&token);
        assert_eq!(result, Some("google".to_string()));
    }

    #[test]
    fn oidc_state_single_use_consumption() {
        let store = OidcStateStore::new(Duration::from_secs(300));
        let token = store.generate("google");
        assert!(store.validate(&token).is_some()); // First use → OK
        assert!(store.validate(&token).is_none());  // Second use → consumed
    }

    #[test]
    fn oidc_state_rejects_unknown_token() {
        let store = OidcStateStore::new(Duration::from_secs(300));
        store.generate("google"); // Generate one token
        assert!(store.validate("totally_fake_token").is_none());
    }

    #[test]
    fn oidc_state_rejects_empty_token() {
        let store = OidcStateStore::new(Duration::from_secs(300));
        assert!(store.validate("").is_none());
    }

    #[test]
    fn oidc_state_multiple_providers() {
        let store = OidcStateStore::new(Duration::from_secs(300));
        let google_token = store.generate("google");
        let github_token = store.generate("github");
        let azure_token = store.generate("azure-ad");

        assert_eq!(store.validate(&github_token), Some("github".to_string()));
        assert_eq!(store.validate(&google_token), Some("google".to_string()));
        assert_eq!(store.validate(&azure_token), Some("azure-ad".to_string()));
    }

    // ══════════════════════════════════════════════════════════
    // Helper Function Tests
    // ══════════════════════════════════════════════════════════

    #[test]
    fn normalize_reply_returns_message_for_empty() {
        assert_eq!(
            normalize_gateway_reply("".to_string()),
            "Model returned an empty response."
        );
    }

    #[test]
    fn normalize_reply_returns_message_for_whitespace() {
        assert_eq!(
            normalize_gateway_reply("   \n\t  ".to_string()),
            "Model returned an empty response."
        );
    }

    #[test]
    fn normalize_reply_passes_through_normal_text() {
        let reply = "Hello, I'm your assistant.".to_string();
        assert_eq!(normalize_gateway_reply(reply.clone()), reply);
    }

    #[test]
    fn normalize_reply_preserves_unicode() {
        let reply = "Hallo! 🦀 Guten Tag.".to_string();
        assert_eq!(normalize_gateway_reply(reply.clone()), reply);
    }

    #[test]
    fn webhook_memory_key_has_uuid_format() {
        let key = webhook_memory_key();
        assert!(key.starts_with("webhook_msg_"));
        // UUID v4 format: 8-4-4-4-12 hex chars
        let uuid_part = &key["webhook_msg_".len()..];
        assert_eq!(uuid_part.len(), 36);
        assert_eq!(uuid_part.chars().filter(|c| *c == '-').count(), 4);
    }

    #[test]
    fn webhook_memory_keys_are_unique() {
        let k1 = webhook_memory_key();
        let k2 = webhook_memory_key();
        assert_ne!(k1, k2);
    }

    #[test]
    fn whatsapp_memory_key_format() {
        let msg = crate::channels::traits::ChannelMessage {
            id: "msg123".to_string(),
            sender: "4915123456789".to_string(),
            content: "Hello".to_string(),
            channel: "whatsapp".to_string(),
            timestamp: 0,
        };
        let key = whatsapp_memory_key(&msg);
        assert_eq!(key, "whatsapp_4915123456789_msg123");
    }

    #[test]
    fn client_key_from_empty_headers() {
        let headers = HeaderMap::new();
        let key = client_key_from_headers(&headers);
        assert!(!key.is_empty());
    }

    #[test]
    fn client_key_from_forwarded_header() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("192.168.1.1"));
        let key = client_key_from_headers(&headers);
        assert!(key.contains("192.168.1.1"));
    }

    #[test]
    fn constants_are_reasonable() {
        assert!(MAX_BODY_SIZE >= 1024, "Body limit too small");
        assert!(MAX_BODY_SIZE <= 128 * 1024 * 1024, "Body limit unreasonably large");
        assert!(REQUEST_TIMEOUT_SECS >= 5, "Timeout too short");
        assert!(REQUEST_TIMEOUT_SECS <= 300, "Timeout too long");
        assert!(RATE_LIMIT_WINDOW_SECS >= 1, "Rate window too short");
    }
}
