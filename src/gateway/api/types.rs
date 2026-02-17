use serde::{Deserialize, Serialize};

// ── System Status ────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct SystemStatus {
    pub version: String,
    pub uptime_secs: u64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f32,
    pub active_agents: usize,
    pub voice_mode_active: bool,
    pub pairing_enabled: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct WidgetConfig {
    pub id: String,
    pub type_: String, // "skill", "shortcut", "panic"
    pub title: String,
    pub icon: Option<String>,
    pub action_url: Option<String>,
}

// ── Identity ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct IdentityStatus {
    pub provider: String, // "google", "apple", "eidas"
    pub id: String,
    pub trust_level: u8, // 1=Low, 3=High
    pub linked_at: String,
}

#[derive(Debug, Deserialize)]
pub struct LinkIdentityRequest {
    pub provider: String,
    pub token: String,
}

// ── Config ───────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub provider: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct SelectModelRequest {
    pub model_id: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct AdBlockStatus {
    pub enabled: bool,
    pub blocklist_count: usize,
    pub last_update: String,
}

#[derive(Debug, Deserialize)]
pub struct ToggleAdBlockRequest {
    pub enabled: bool,
}

// ── Chat / WebSocket ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum WsMessage {
    /// Text message from user or agent
    #[serde(rename = "text")]
    Text { 
        content: String, 
        sender: String, // "user", "agent", "system"
        is_final: bool 
    },
    
    /// Audio chunk (base64 encoded for JSON, but binary preferred over raw WS)
    #[serde(rename = "audio")]
    Audio { 
        data: String, // Base64
        format: String // "pcm", "opus", "mp3"
    },
    
    /// Control events
    #[serde(rename = "control")]
    Control {
        event: String, // "voice_start", "voice_end", "interrupt"
    },

    /// Error message
    #[serde(rename = "error")]
    Error {
        code: String,
        message: String,
    }
}
