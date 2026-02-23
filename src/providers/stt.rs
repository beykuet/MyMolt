// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::{multipart, Client};
use serde::Deserialize;

/// Interface for Speech-to-Text providers.
#[async_trait]
pub trait SttProvider: Send + Sync {
    /// Transcribe audio data to text.
    ///
    /// # Arguments
    /// * `audio_data` - Raw audio bytes (e.g., WAV, MP3 content).
    /// * `format` - File extension or format hint (e.g., "wav", "mp3").
    async fn transcribe(&self, audio_data: Vec<u8>, format: &str) -> Result<String>;

    /// Return the provider name (e.g., "OpenAI Whisper").
    fn name(&self) -> &str;
}

// ══════════════════════════════════════════════════════════════════════════════
// OpenAI Whisper Implementation
// ══════════════════════════════════════════════════════════════════════════════

pub struct OpenAiSttProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAiSttProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: "whisper-1".to_string(),
        }
    }
}

#[derive(Deserialize)]
struct WhisperResponse {
    text: String,
}

#[async_trait]
impl SttProvider for OpenAiSttProvider {
    async fn transcribe(&self, audio_data: Vec<u8>, format: &str) -> Result<String> {
        let part = multipart::Part::bytes(audio_data)
            .file_name(format!("audio.{}", format))
            .mime_str(&format!("audio/{}", format))
            .context("Failed to create multipart form")?;

        let form = multipart::Form::new()
            .part("file", part)
            .text("model", self.model.clone());

        let response = self.client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await
            .context("Failed to send request to OpenAI Whisper API")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI Whisper API error: {}", error_text);
        }

        let resp: WhisperResponse = response.json().await.context("Failed to parse Whisper response")?;
        Ok(resp.text)
    }

    fn name(&self) -> &str {
        "OpenAI Whisper"
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// OpenRouter STT Implementation (Placeholder)
// ══════════════════════════════════════════════════════════════════════════════

// TODO: Implement OpenRouter multimodal STT.
// Requires standardizing audio input format (e.g. `input_audio` vs data URI).


// ══════════════════════════════════════════════════════════════════════════════
// Factory
// ══════════════════════════════════════════════════════════════════════════════

pub fn create_stt_provider(
    provider_name: &str,
    api_key: &str,
    _model_hint: Option<String>,
) -> Result<Box<dyn SttProvider>> {
    match provider_name {
        "openai" => Ok(Box::new(OpenAiSttProvider::new(api_key.to_string()))),
        // Placeholder for future expansion
        _ => anyhow::bail!("Unsupported STT provider: {}", provider_name),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Mock STT Provider (for testing)
// ══════════════════════════════════════════════════════════════════════════════

/// A mock STT provider that returns a pre-configured transcription.
/// Used for unit and integration tests so we never call a real API.
pub struct MockSttProvider {
    pub transcription: String,
}

impl MockSttProvider {
    pub fn new(transcription: impl Into<String>) -> Self {
        Self {
            transcription: transcription.into(),
        }
    }
}

#[async_trait]
impl SttProvider for MockSttProvider {
    async fn transcribe(&self, _audio_data: Vec<u8>, _format: &str) -> Result<String> {
        Ok(self.transcription.clone())
    }

    fn name(&self) -> &str {
        "Mock STT"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Factory Tests ──────────────────────────────────────────────

    #[test]
    fn factory_creates_openai_provider() {
        let provider = create_stt_provider("openai", "sk-test-key", None);
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "OpenAI Whisper");
    }

    #[test]
    fn factory_rejects_unknown_provider() {
        let result = create_stt_provider("deepseek", "sk-test", None);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(err.contains("Unsupported STT provider: deepseek"));
    }

    #[test]
    fn factory_with_model_hint_still_works() {
        let provider = create_stt_provider("openai", "sk-test", Some("whisper-1".into()));
        assert!(provider.is_ok());
    }

    // ── Mock Provider Tests ────────────────────────────────────────

    #[tokio::test]
    async fn mock_provider_returns_configured_transcription() {
        let provider = MockSttProvider::new("My PIN is 1234");
        let result = provider.transcribe(vec![0u8; 100], "webm").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "My PIN is 1234");
    }

    #[tokio::test]
    async fn mock_provider_ignores_audio_data() {
        let provider = MockSttProvider::new("Hello world");
        // Empty audio should still work — the mock doesn't process real audio
        let result = provider.transcribe(vec![], "wav").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello world");
    }

    #[test]
    fn mock_provider_name() {
        let provider = MockSttProvider::new("test");
        assert_eq!(provider.name(), "Mock STT");
    }

    // ── Trait Object Tests ─────────────────────────────────────────

    #[tokio::test]
    async fn stt_provider_trait_object_works() {
        // Ensure the provider can be used as a trait object (Arc<dyn SttProvider>)
        let provider: std::sync::Arc<dyn SttProvider> = std::sync::Arc::new(
            MockSttProvider::new("I am a trait object"),
        );
        let result = provider.transcribe(vec![1, 2, 3], "mp3").await;
        assert_eq!(result.unwrap(), "I am a trait object");
    }

    // ── OpenAI Provider Unit Tests (no network) ────────────────────

    #[test]
    fn openai_provider_defaults_to_whisper_model() {
        let provider = OpenAiSttProvider::new("sk-test".to_string());
        assert_eq!(provider.model, "whisper-1");
        assert_eq!(provider.name(), "OpenAI Whisper");
    }
}

