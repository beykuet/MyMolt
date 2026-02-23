// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

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
    pub voice_echo_enabled: bool,
    pub adblock_enabled: bool,
    pub adblock_count: usize,
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

#[derive(Debug, Serialize, Clone)]
pub struct IdentityProvider {
    pub id: String,
    pub name: String,
    pub icon_url: Option<String>,
    pub trust_level: u8,
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
        is_final: bool,
    },

    /// Audio chunk (base64 encoded for JSON, but binary preferred over raw WS)
    #[serde(rename = "audio")]
    Audio {
        data: String,   // Base64
        format: String, // "pcm", "opus", "mp3"
    },

    /// Control events
    #[serde(rename = "control")]
    Control {
        event: String, // "voice_start", "voice_end", "interrupt"
    },

    /// Error message
    #[serde(rename = "error")]
    Error { code: String, message: String },

    /// Agent internal thought (for UI streaming)
    #[serde(rename = "thought")]
    Thought { content: String },

    /// Confirmation request from security gate
    #[serde(rename = "confirm")]
    Confirm {
        id: String,
        tool_name: String,
        description: String,
        timeout_secs: u64,
    },
}

// ── VPN ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct VpnPeer {
    pub id: String,
    pub name: String,
    pub allowed_ips: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct CreatePeerResponse {
    pub peer: VpnPeer,
    pub config_file: String, // Complete WireGuard config
    pub qr_code_svg: String, // SVG data URL for frontend
}

#[derive(Debug, Deserialize)]
pub struct CreatePeerRequest {
    pub name: String,
}

// ── Vault ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct VaultEntryMetadata {
    pub id: String,
    pub description: String,
    pub created_at: String,
    pub tags: Vec<String>,
}

// ── Diary ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateDiaryEntryRequest {
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── WsMessage Serialization Tests ─────────────────────────────

    #[test]
    fn ws_text_message_serialization_roundtrip() {
        let msg = WsMessage::Text {
            content: "Hello world".into(),
            sender: "user".into(),
            is_final: true,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("\"content\":\"Hello world\""));
        assert!(json.contains("\"sender\":\"user\""));

        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            WsMessage::Text {
                content,
                sender,
                is_final,
            } => {
                assert_eq!(content, "Hello world");
                assert_eq!(sender, "user");
                assert!(is_final);
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn ws_audio_message_serialization_roundtrip() {
        let msg = WsMessage::Audio {
            data: "SGVsbG8=".into(), // "Hello" in base64
            format: "webm".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"audio\""));
        assert!(json.contains("\"data\":\"SGVsbG8=\""));
        assert!(json.contains("\"format\":\"webm\""));

        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            WsMessage::Audio { data, format } => {
                assert_eq!(data, "SGVsbG8=");
                assert_eq!(format, "webm");
            }
            _ => panic!("Expected Audio variant"),
        }
    }

    #[test]
    fn ws_thought_message_serialization() {
        let msg = WsMessage::Thought {
            content: "👂 Listening...".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"thought\""));
        assert!(json.contains("Listening..."));

        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            WsMessage::Thought { content } => {
                assert!(content.contains("Listening"));
            }
            _ => panic!("Expected Thought variant"),
        }
    }

    #[test]
    fn ws_error_message_serialization() {
        let msg = WsMessage::Error {
            code: "STT_ERROR".into(),
            message: "Speech-to-text failed".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("STT_ERROR"));

        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            WsMessage::Error { code, message } => {
                assert_eq!(code, "STT_ERROR");
                assert_eq!(message, "Speech-to-text failed");
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn ws_control_message_serialization() {
        let msg = WsMessage::Control {
            event: "voice_start".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"control\""));

        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            WsMessage::Control { event } => assert_eq!(event, "voice_start"),
            _ => panic!("Expected Control variant"),
        }
    }

    #[test]
    fn ws_audio_decode_error_message_format() {
        let msg = WsMessage::Error {
            code: "AUDIO_DECODE_ERROR".into(),
            message: "Invalid base64 audio data".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("AUDIO_DECODE_ERROR"));
    }

    // ── Frontend Payload Format Tests ─────────────────────────────

    #[test]
    fn frontend_audio_payload_deserializes() {
        // This is the exact JSON the frontend sends for audio
        let frontend_json =
            r#"{"type":"audio","payload":{"data":"SGVsbG8gV29ybGQ=","format":"webm"}}"#;
        let parsed: WsMessage = serde_json::from_str(frontend_json).unwrap();
        match parsed {
            WsMessage::Audio { data, format } => {
                assert_eq!(data, "SGVsbG8gV29ybGQ=");
                assert_eq!(format, "webm");
            }
            _ => panic!("Expected Audio variant from frontend payload"),
        }
    }

    #[test]
    fn frontend_text_payload_deserializes() {
        let frontend_json =
            r#"{"type":"text","payload":{"content":"Hello bot","sender":"user","is_final":true}}"#;
        let parsed: WsMessage = serde_json::from_str(frontend_json).unwrap();
        match parsed {
            WsMessage::Text {
                content, sender, ..
            } => {
                assert_eq!(content, "Hello bot");
                assert_eq!(sender, "user");
            }
            _ => panic!("Expected Text variant from frontend payload"),
        }
    }
}
