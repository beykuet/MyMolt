// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State, Query},
    response::IntoResponse,
    http::StatusCode,
};
use crate::gateway::AppState;
use super::types::WsMessage;
use super::auth::AuthQuery;
use serde_json;
use base64::Engine;
use uuid::Uuid;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<AuthQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if !state.pairing.is_authenticated(params.token.as_deref().unwrap_or("")) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    tracing::info!("New WebSocket connection established");

    // Send welcome message
    let welcome = WsMessage::Text {
        content: "Connected to MyMolt Core Gateway".into(),
        sender: "system".into(),
        is_final: true,
    };
    
    if let Ok(msg) = serde_json::to_string(&welcome) {
        let _ = socket.send(Message::Text(msg.into())).await;
    }

    // Main loop
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                tracing::warn!("WebSocket error: {e}");
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    process_message(ws_msg, &mut socket, &state).await;
                }
            }
            Message::Binary(data) => {
                // Handle raw audio chunks here if we decide to use binary frames
                // For now we expect JSON-wrapped base64 or control messages
                tracing::debug!("Received binary frame of size: {}", data.len());
            }
            Message::Close(_) => {
                tracing::info!("WebSocket connection closed");
                break;
            }
            _ => {}
        }
    }
}

async fn handle_text_interaction(content: String, socket: &mut WebSocket, state: &AppState) {
    // 1. Store user message in memory for Sigil scanning (Crucial step!)
    if state.auto_save {
        let key = format!("user_msg_{}", Uuid::new_v4());
        let _ = state
            .mem
            .store(&key, &content, crate::memory::MemoryCategory::Conversation)
            .await;
    }

    // 2. Prepare observer and channels
    let (thought_tx, mut thought_rx) = tokio::sync::mpsc::unbounded_channel();
    let observer = WsObserver::new(thought_tx);
    
    let model = state.model.read().await.clone();
    let state_clone = state.clone();
    let content_clone = content.clone();
    
    // Channel for the final result
    let (result_tx, mut result_rx) = tokio::sync::mpsc::channel(1);

    tokio::spawn(async move {
        // Build context preamble (same as loop_.rs)
        // We do a simplified version here: retrieve relevant memories first
        let context = if let Ok(entries) = state_clone.mem.recall(&content_clone, 5).await {
            if entries.is_empty() {
                String::new()
            } else {
                let mut ctx = String::from("[Memory context]\n");
                for entry in entries {
                    use std::fmt::Write;
                    let _ = writeln!(ctx, "- {}: {}", entry.key, entry.content);
                }
                ctx.push('\n');
                ctx
            }
        } else {
            String::new()
        };

        let enriched = if context.is_empty() {
            content_clone
        } else {
            format!("{context}{content_clone}")
        };

        // Read dynamic config
        let system_prompt = state_clone.system_prompt.read().await.clone();
        let temperature = *state_clone.temperature.read().await;

        let mut history = vec![
            crate::providers::ChatMessage::system(&system_prompt),
            crate::providers::ChatMessage::user(&enriched),
        ];

        let res = crate::agent::loop_::run_tool_call_loop(
            state_clone.provider.as_ref(),
            &mut history,
            state_clone.tools_registry.as_ref(),
            &observer,
            "dashboard",
            &model,
            temperature,
        ).await;
        let _ = result_tx.send(res).await;
    });

    // Handle streaming thoughts and the final result
    loop {
        tokio::select! {
             Some(thought) = thought_rx.recv() => {
                if let Ok(json) = serde_json::to_string(&thought) {
                    let _ = socket.send(Message::Text(json.into())).await;
                }
            }
            Some(res) = result_rx.recv() => {
                match res {
                    Ok(reply) => {
                        let resp_msg = WsMessage::Text {
                            content: reply,
                            sender: "agent".into(),
                            is_final: true,
                        };
                        if let Ok(json) = serde_json::to_string(&resp_msg) {
                            let _ = socket.send(Message::Text(json.into())).await;
                        }
                    }
                    Err(e) => {
                        let err_msg = WsMessage::Error {
                            code: "AGENT_ERROR".into(),
                            message: e.to_string(),
                        };
                        if let Ok(json) = serde_json::to_string(&err_msg) {
                            let _ = socket.send(Message::Text(json.into())).await;
                        }
                    }
                }
                break;
            }
        }
    }
}

async fn process_message(msg: WsMessage, socket: &mut WebSocket, state: &AppState) {
    match msg {
        WsMessage::Text { content, .. } => {
            handle_text_interaction(content, socket, state).await;
        }
        WsMessage::Audio { data, format } => {
            tracing::info!("Received audio chunk: {} bytes, format: {}", data.len(), format);
            
            // WebSocket Echo (Loopback) Mode
            if state.voice_echo_enabled.load(std::sync::atomic::Ordering::Relaxed) {
                let echo_msg = WsMessage::Audio {
                    data: data.clone(),
                    format: format.clone(),
                };
                if let Ok(json) = serde_json::to_string(&echo_msg) {
                    let _ = socket.send(Message::Text(json.into())).await;
                }
            } else {
                // 1. Decode base64
                let audio_bytes = match base64::engine::general_purpose::STANDARD.decode(&data) {
                    Ok(b) => b,
                    Err(e) => {
                        tracing::error!("Failed to decode audio base64: {}", e);
                         let _ = socket.send(Message::Text(serde_json::to_string(&WsMessage::Error {
                            code: "AUDIO_DECODE_ERROR".into(),
                            message: "Invalid base64 audio data".into(),
                        }).unwrap().into())).await;
                        return;
                    }
                };

                // 2. Transcribe
                // Send a thought first so user knows we are processing
                let _ = socket.send(Message::Text(serde_json::to_string(&WsMessage::Thought {
                    content: "👂 Listening...".into()
                }).unwrap().into())).await;

                let transcription = match state.stt.transcribe(audio_bytes, &format).await {
                    Ok(t) => t,
                    Err(e) => {
                        tracing::error!("STT error: {}", e);
                        let _ = socket.send(Message::Text(serde_json::to_string(&WsMessage::Error {
                            code: "STT_ERROR".into(),
                            message: "Speech-to-text failed".into(),
                        }).unwrap().into())).await;
                        return;
                    }
                };
                
                tracing::info!("Transcribed audio: '{}'", transcription);

                // Send transcription back to UI as a 'thought'
                let _ = socket.send(Message::Text(serde_json::to_string(&WsMessage::Thought {
                    content: format!("🎤 Heard: \"{}\"", transcription)
                }).unwrap().into())).await;

                // 3. Process as text message
                handle_text_interaction(transcription, socket, state).await;
            }
        }
        WsMessage::Control { event } => {
            tracing::info!("Received control event: {}", event);
            
            if event == "voice_test" {
                // Send mock bot response
                let mock_audio = crate::providers::mock_voice::MockVoiceProvider::get_response_audio();
                let resp = WsMessage::Audio {
                    data: mock_audio,
                    format: "wav".into(),
                };
                if let Ok(json) = serde_json::to_string(&resp) {
                    let _ = socket.send(Message::Text(json.into())).await;
                }
            }
        }
        _ => {}
    }
}

/// A specialized observer that streams agent progress over a WebSocket.
struct WsObserver {
    tx: tokio::sync::mpsc::UnboundedSender<WsMessage>,
}

impl WsObserver {
    fn new(tx: tokio::sync::mpsc::UnboundedSender<WsMessage>) -> Self {
        Self { tx }
    }
}

impl crate::observability::Observer for WsObserver {
    fn name(&self) -> &str {
        "websocket"
    }

    fn record_event(&self, event: &crate::observability::ObserverEvent) {
        use crate::observability::ObserverEvent;
        
        // Convert internal events to UI thoughts
        let thought = match event {
            ObserverEvent::ToolCallStart { tool } => {
                Some(format!("🔧 Executing tool: {}...", tool))
            }
            ObserverEvent::ToolCall { tool, duration, success } => {
                let status = if *success { "Completed" } else { "Failed" };
                Some(format!("✅ {} {} ({}ms)", tool, status, duration.as_millis()))
            }
            ObserverEvent::LlmRequest { model, .. } => {
                Some(format!("🧠 Consulting {}...", model))
            }
            _ => None,
        };

        if let Some(content) = thought {
            let _ = self.tx.send(WsMessage::Thought { content });
        }
    }

    fn record_metric(&self, _metric: &crate::observability::traits::ObserverMetric) {
        // We don't stream raw metrics over chat WS yet
    }
}
