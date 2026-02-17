use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State, Query},
    response::IntoResponse,
    http::StatusCode,
};
use crate::gateway::AppState;
use super::types::WsMessage;
use super::auth::AuthQuery;
use serde_json;

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

async fn process_message(msg: WsMessage, socket: &mut WebSocket, state: &AppState) {
    match msg {
        WsMessage::Text { content, .. } => {
            // Echo user message back as confirmation (optional)
            // Call LLM Agent
            
            // Build chat history (simplified for now)
            let mut history = vec![
                crate::providers::ChatMessage::system(state.system_prompt.as_str()),
                crate::providers::ChatMessage::user(&content),
            ];

            // Trigger Agent Loop
            // Note: In a real implementation we would stream tokens back.
            // For MVP, we wait for full response.
            
            let response = crate::agent::loop_::run_tool_call_loop(
                state.provider.as_ref(),
                &mut history,
                state.tools_registry.as_ref(),
                state.observer.as_ref(),
                "dashboard",
                &state.model,
                state.temperature,
            ).await;

            match response {
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
                // TODO: Implement STT here
                // 1. Decode base64 `data`
                // 2. Feed to STT engine (e.g. Whisper)
                
                // Ack
                let ack = WsMessage::Control { event: "audio_received".into() };
                 if let Ok(json) = serde_json::to_string(&ack) {
                    let _ = socket.send(Message::Text(json.into())).await;
                }
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
