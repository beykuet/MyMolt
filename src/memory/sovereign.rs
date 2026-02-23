// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use crate::memory::{Memory, MemoryCategory, MemoryEntry};
use crate::security::VaultManager;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use std::path::Path;
use std::sync::Arc;
use crate::security::AuditLogger;

use aho_corasick::AhoCorasick;

/// Scans text for sensitive patterns.
///
/// Uses a two-phase approach for performance:
/// 1. **Aho-Corasick pre-filter** — single-pass scan for fixed prefixes
///    (`sk-`, `AIza`, `AKIA`, `-----BEGIN`). If none found, those 4
///    regex checks are skipped entirely.
/// 2. **Regex validation** — financial patterns (IBAN, CC, PIN) always
///    run; prefix-guarded patterns only run when their prefix is present.
pub struct SensitivityScanner {
    /// Fixed-prefix pre-filter (single-pass Aho-Corasick automaton).
    prefix_filter: AhoCorasick,
    /// Full regex patterns for precise matching.
    /// Indices 0..3 are prefix-guarded, 4..6 always run.
    patterns: Vec<(String, Regex)>,
    /// Number of prefix-guarded patterns at the start of `patterns`.
    prefix_guarded_count: usize,
}

impl SensitivityScanner {
    pub fn new() -> Self {
        // Prefix-guarded patterns (indices 0..3): only checked when prefix is found
        let prefix_patterns = vec![
            (
                "OpenAI Key".into(),
                Regex::new(r"sk-[a-zA-Z0-9-]{20,}").unwrap(),
            ),
            (
                "Google API Key".into(),
                Regex::new(r"AIza[0-9A-Za-z-_]{35}").unwrap(),
            ),
            (
                "AWS Access Key".into(),
                Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
            ),
            (
                "Private Key Block".into(),
                Regex::new(r"-----BEGIN [A-Z]+ PRIVATE KEY-----").unwrap(),
            ),
        ];
        let prefix_guarded_count = prefix_patterns.len();

        // Always-run patterns (financial PII): no fixed prefix available
        let always_patterns = vec![
            (
                "IBAN".into(),
                Regex::new(r"[A-Z]{2}[0-9]{2}(?:[ ]?[0-9]{4}){4,}(?:[ ]?[0-9]{1,2})?").unwrap(),
            ),
            (
                "Credit Card".into(),
                Regex::new(r"\b(?:\d[ -]*?){13,16}\b").unwrap(),
            ),
            (
                "Bank PIN".into(),
                // Match "PIN is 1234", "PIN: 1234", "code 1234", "cvv 123"
                Regex::new(r"(?i)(pin|conf|cvv|code)\s*(?:is|:|=|-)?\s*(\d{3,8})").unwrap(),
            ),
        ];

        let mut patterns = prefix_patterns;
        patterns.extend(always_patterns);

        // Build Aho-Corasick automaton from fixed prefixes
        let prefix_filter =
            AhoCorasick::new(["sk-", "AIza", "AKIA", "-----BEGIN"]).unwrap();

        Self {
            prefix_filter,
            patterns,
            prefix_guarded_count,
        }
    }

    pub fn scan(&self, text: &str) -> Option<String> {
        // Phase 1: Aho-Corasick single-pass pre-filter
        let has_prefix = self.prefix_filter.is_match(text);

        // Phase 2: Run applicable regex patterns
        for (i, (name, re)) in self.patterns.iter().enumerate() {
            // Skip prefix-guarded regexes if no prefix was found
            if i < self.prefix_guarded_count && !has_prefix {
                continue;
            }
            if re.is_match(text) {
                return Some(name.clone());
            }
        }
        None
    }

    /// Replace all sensitive matches in `text` with `[REDACTED:{pattern_name}]`.
    /// Returns `(redacted_text, Vec<pattern_names_found>)`.
    pub fn redact(&self, text: &str) -> (String, Vec<String>) {
        let has_prefix = self.prefix_filter.is_match(text);
        let mut result = text.to_string();
        let mut found = Vec::new();
        for (i, (name, re)) in self.patterns.iter().enumerate() {
            if i < self.prefix_guarded_count && !has_prefix {
                continue;
            }
            if re.is_match(&result) {
                found.push(name.clone());
                result = re
                    .replace_all(&result, format!("[REDACTED:{}]", name))
                    .to_string();
            }
        }
        (result, found)
    }
}

/// The "Guard" that intercepts memory operations.
///
/// Wraps an underlying memory backend and intercepts strict `store` calls.
/// If sensitive content is detected, it is encrypted into the Vault,
/// and only an "Opaque Pointer" is stored in the underlying memory.
pub struct SovereignMemory {
    inner: Arc<dyn Memory>,
    vault: VaultManager,
    scanner: SensitivityScanner,
    recipient: String,
    audit: Arc<AuditLogger>,
}

impl SovereignMemory {
    pub fn new(inner: Arc<dyn Memory>, workspace_dir: &Path, audit: Arc<AuditLogger>) -> Self {
        // Defaults to user's SSH public key for encryption.
        // In a future update, this should be configurable via `config.toml`.
        // We assume ~/.ssh/id_ed25519.pub is the intended recipient for now.
        let recipient = shellexpand::tilde("~/.ssh/id_ed25519.pub").to_string();

        Self {
            inner,
            vault: VaultManager::new(workspace_dir),
            scanner: SensitivityScanner::new(),
            recipient,
            audit,
        }
    }
}

#[async_trait]
impl Memory for SovereignMemory {
    fn name(&self) -> &str {
        "sovereign"
    }

    async fn store(&self, key: &str, content: &str, category: MemoryCategory) -> Result<()> {
        // 1. Safety Check: Avoid infinite recursion.
        // The VaultManager itself calls `store` to save metadata index.
        // We must identify these calls and let them pass through.
        if let MemoryCategory::Custom(ref s) = category {
            if s == "vault" {
                return self.inner.store(key, content, category).await;
            }
        }

        // 2. Scan for Sensitivity
        if let Some(reason) = self.scanner.scan(content) {
            tracing::info!(
                "🛡️ Sovereign Interceptor: Detected sensitive data ({}) for key '{}'. Vaulting...",
                reason,
                key
            );

            // 3. Encrypt to Vault
            // We pass `self.inner` (Arc<dyn Memory>) to `encrypt_to_vault`.
            // The VaultManager will use it to store the metadata.
            let description = format!("Vaulted content for {}: {}", key, reason);
            
            // Note: `encrypt_to_vault` expects generic M: Memory. 
            // Since we carry `Arc<dyn Memory>`, we dereferencing it creates a trait object match.
            self.vault
                .encrypt_to_vault(
                    self.inner.as_ref(),
                    content.to_string(),
                    &description,
                    &self.recipient,
                )
                .await?;

            // 4. Store Opaque Pointer in Cleartext Memory
            // This replaces the actual sensitive content with a safe placeholder.
            let pointer = format!("[VAULT: {} - Access Required]", reason);

            // 5. Log Sigil Interception for UI Transparency
            let _ = self.audit.log(
                &crate::security::AuditEvent::new(crate::security::AuditEventType::SigilInterception)
                    .with_action(
                        format!("Redacted {} from memory", reason),
                        "low".to_string(),
                        true,
                        true
                    )
            );

            return self.inner.store(key, &pointer, category).await;
        }

        // 5. Pass through safe content
        self.inner.store(key, content, category).await
    }

    async fn recall(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>> {
        self.inner.recall(query, limit).await
    }

    async fn get(&self, key: &str) -> Result<Option<MemoryEntry>> {
        self.inner.get(key).await
    }

    async fn list(&self, category: Option<&MemoryCategory>) -> Result<Vec<MemoryEntry>> {
        self.inner.list(category).await
    }

    async fn forget(&self, key: &str) -> Result<bool> {
        self.inner.forget(key).await
    }

    async fn count(&self) -> Result<usize> {
        self.inner.count().await
    }

    async fn health_check(&self) -> bool {
        self.inner.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::simple::SimpleMemory;

    /// Helper: build a SovereignMemory with a SimpleMemory backend.
    ///
    /// Provisions Hoodik RSA keys in the temp directory so the
    /// `VaultManager` can encrypt during tests.
    fn make_sovereign() -> SovereignMemory {
        let inner: Arc<dyn Memory> = Arc::new(SimpleMemory::new());
        let tmp = tempfile::tempdir().unwrap();

        // Provision Hoodik RSA keys for VaultManager
        let key_dir = tmp.path().join("hoodik/keys");
        std::fs::create_dir_all(&key_dir).unwrap();
        let priv_key = cryptfns::rsa::private::generate().unwrap();
        let pub_key = cryptfns::rsa::public::from_private(&priv_key).unwrap();
        let priv_pem = cryptfns::rsa::private::to_string(&priv_key).unwrap();
        let pub_pem = cryptfns::rsa::public::to_string(&pub_key).unwrap();
        std::fs::write(key_dir.join("admin.key"), &priv_pem).unwrap();
        std::fs::write(key_dir.join("admin.pub"), &pub_pem).unwrap();

        let audit = Arc::new(
            crate::security::AuditLogger::new(
                crate::config::AuditConfig::default(),
                tmp.path().to_path_buf(),
            )
            .unwrap(),
        );
        // Leak the tempdir so it lives for the duration of the test
        let path = tmp.into_path();
        SovereignMemory::new(inner, &path, audit)
    }

    // ── SensitivityScanner Tests ─────────────────────────────────

    #[test]
    fn scanner_detects_openai_key() {
        let scanner = SensitivityScanner::new();
        let result = scanner.scan("My key is sk-abc123def456ghi789jkl");
        assert_eq!(result, Some("OpenAI Key".to_string()));
    }

    #[test]
    fn scanner_detects_iban() {
        let scanner = SensitivityScanner::new();
        let result = scanner.scan("Transfer to DE89 3704 0044 0532 0130 00");
        assert_eq!(result, Some("IBAN".to_string()));
    }

    #[test]
    fn scanner_detects_pin() {
        let scanner = SensitivityScanner::new();
        let result = scanner.scan("My PIN is 1234");
        assert_eq!(result, Some("Bank PIN".to_string()));
    }

    #[test]
    fn scanner_detects_pin_with_colon() {
        let scanner = SensitivityScanner::new();
        let result = scanner.scan("PIN: 5678");
        assert_eq!(result, Some("Bank PIN".to_string()));
    }

    #[test]
    fn scanner_detects_cvv() {
        let scanner = SensitivityScanner::new();
        let result = scanner.scan("cvv 123");
        assert_eq!(result, Some("Bank PIN".to_string()));
    }

    #[test]
    fn scanner_detects_confirmation_code() {
        let scanner = SensitivityScanner::new();
        let result = scanner.scan("confirmation code is 456789");
        assert_eq!(result, Some("Bank PIN".to_string()));
    }

    #[test]
    fn scanner_detects_private_key_block() {
        let scanner = SensitivityScanner::new();
        let result = scanner.scan("-----BEGIN RSA PRIVATE KEY-----");
        assert_eq!(result, Some("Private Key Block".to_string()));
    }

    #[test]
    fn scanner_detects_aws_key() {
        let scanner = SensitivityScanner::new();
        let result = scanner.scan("AKIAIOSFODNN7EXAMPLE");
        assert_eq!(result, Some("AWS Access Key".to_string()));
    }

    #[test]
    fn scanner_detects_google_api_key() {
        let scanner = SensitivityScanner::new();
        let result = scanner.scan("AIzaSyA1234567890abcdefghijklmnopqrstuvwx");
        assert_eq!(result, Some("Google API Key".to_string()));
    }

    #[test]
    fn scanner_passes_safe_text() {
        let scanner = SensitivityScanner::new();
        assert!(scanner.scan("The weather is nice today").is_none());
        assert!(scanner.scan("Tell me about quantum physics").is_none());
        assert!(scanner.scan("Set a reminder for 3pm").is_none());
    }

    // ── SovereignMemory Sigil Tests ──────────────────────────────

    #[tokio::test]
    async fn sigil_redacts_pin_from_transcription() {
        // Simulates: User says "My PIN is 1234" → STT transcribes → stored in memory
        let sovereign = make_sovereign();

        // This is exactly what handle_text_interaction does with transcribed audio
        let transcribed_text = "My PIN is 1234";
        sovereign
            .store("user_msg_audio_001", transcribed_text, MemoryCategory::Conversation)
            .await
            .unwrap();

        // Verify the stored content is redacted, not the original
        let stored = sovereign.inner.get("user_msg_audio_001").await.unwrap();
        assert!(stored.is_some());
        let entry = stored.unwrap();
        assert!(
            entry.content.contains("[VAULT:"),
            "Expected redacted content, got: {}",
            entry.content
        );
        assert!(
            !entry.content.contains("1234"),
            "PIN should be redacted from memory"
        );
    }

    #[tokio::test]
    async fn sigil_redacts_api_key_from_transcription() {
        let sovereign = make_sovereign();

        let transcribed_text = "Use this API key sk-abcdefghijklmnopqrstuvwxyz";
        sovereign
            .store("user_msg_audio_002", transcribed_text, MemoryCategory::Conversation)
            .await
            .unwrap();

        let stored = sovereign.inner.get("user_msg_audio_002").await.unwrap().unwrap();
        assert!(stored.content.contains("[VAULT:"));
        assert!(!stored.content.contains("sk-abcdef"));
    }

    #[tokio::test]
    async fn sigil_redacts_iban_from_transcription() {
        let sovereign = make_sovereign();

        let transcribed_text = "Send money to DE89 3704 0044 0532 0130 00";
        sovereign
            .store("user_msg_audio_003", transcribed_text, MemoryCategory::Conversation)
            .await
            .unwrap();

        let stored = sovereign.inner.get("user_msg_audio_003").await.unwrap().unwrap();
        assert!(stored.content.contains("[VAULT:"));
    }

    #[tokio::test]
    async fn sigil_passes_harmless_text_through() {
        let sovereign = make_sovereign();

        let transcribed_text = "What is the weather like today";
        sovereign
            .store("user_msg_text_001", transcribed_text, MemoryCategory::Conversation)
            .await
            .unwrap();

        let stored = sovereign.inner.get("user_msg_text_001").await.unwrap().unwrap();
        assert_eq!(stored.content, "What is the weather like today");
    }

    #[tokio::test]
    async fn sigil_name_is_sovereign() {
        let sovereign = make_sovereign();
        assert_eq!(sovereign.name(), "sovereign");
    }

    // ── Full STT → Sigil Pipeline Simulation ──────────────────────

    #[tokio::test]
    async fn full_stt_sigil_pipeline_pin_redaction() {
        use crate::providers::stt::{SttProvider, MockSttProvider};

        // 1. Mock STT transcribes audio
        let stt: std::sync::Arc<dyn SttProvider> =
            std::sync::Arc::new(MockSttProvider::new("My PIN is 1234"));
        let transcription = stt.transcribe(vec![0u8; 100], "webm").await.unwrap();
        assert_eq!(transcription, "My PIN is 1234");

        // 2. SovereignMemory stores transcription (Sigil scanner kicks in)
        let sovereign = make_sovereign();
        sovereign
            .store("user_msg_voice_001", &transcription, MemoryCategory::Conversation)
            .await
            .unwrap();

        // 3. Verify redaction
        let stored = sovereign.inner.get("user_msg_voice_001").await.unwrap().unwrap();
        assert!(
            stored.content.contains("[VAULT:"),
            "Sigil should redact the PIN"
        );
        assert!(
            !stored.content.contains("1234"),
            "Original PIN should NOT be in cleartext memory"
        );
    }

    #[tokio::test]
    async fn full_stt_sigil_pipeline_safe_message() {
        use crate::providers::stt::{SttProvider, MockSttProvider};

        // 1. Mock STT transcribes harmless audio
        let stt: std::sync::Arc<dyn SttProvider> =
            std::sync::Arc::new(MockSttProvider::new("Tell me about the weather"));
        let transcription = stt.transcribe(vec![0u8; 50], "webm").await.unwrap();

        // 2. SovereignMemory stores it
        let sovereign = make_sovereign();
        sovereign
            .store("user_msg_voice_002", &transcription, MemoryCategory::Conversation)
            .await
            .unwrap();

        // 3. Verify NO redaction — safe text passes through
        let stored = sovereign.inner.get("user_msg_voice_002").await.unwrap().unwrap();
        assert_eq!(stored.content, "Tell me about the weather");
    }

    #[tokio::test]
    async fn full_stt_sigil_pipeline_multiple_secrets() {
        use crate::providers::stt::{SttProvider, MockSttProvider};

        // 1. User speaks: contains IBAN
        let stt: std::sync::Arc<dyn SttProvider> = std::sync::Arc::new(
            MockSttProvider::new("Pay to DE89 3704 0044 0532 0130 00 and my PIN is 5678"),
        );
        let transcription = stt.transcribe(vec![0u8; 200], "webm").await.unwrap();

        // 2. Store — Sigil should detect the first secret pattern
        let sovereign = make_sovereign();
        sovereign
            .store("user_msg_voice_003", &transcription, MemoryCategory::Conversation)
            .await
            .unwrap();

        // 3. Verify redaction
        let stored = sovereign.inner.get("user_msg_voice_003").await.unwrap().unwrap();
        assert!(stored.content.contains("[VAULT:"));
    }
}
