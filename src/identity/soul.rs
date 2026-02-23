// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustLevel {
    /// Anonymous / unverified
    Low = 1,
    /// Verified email / OIDC (Google, Apple)
    Medium = 2,
    /// Verified eIDAS / Government ID
    High = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityBinding {
    pub provider: String,
    pub id: String,
    pub trust_level: TrustLevel,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiaryEntry {
    pub timestamp: String,
    pub content: String,
}

pub struct Soul {
    pub path: PathBuf,
    pub bindings: Vec<IdentityBinding>,
    pub raw_content: String,
}

impl Soul {
    pub fn new(workspace_dir: &Path) -> Self {
        let path = workspace_dir.join("SOUL.md");
        Self {
            path,
            bindings: Vec::new(),
            raw_content: String::new(),
        }
    }

    /// Load bindings from SOUL.md
    /// Expected format in markdown:
    /// ## Identity Bindings
    /// - **Google**: 12345 (Level 1)
    /// - **eIDAS**: DE/123 (Level 3)
    pub fn load(&mut self) -> Result<()> {
        if !self.path.exists() {
            // Create default if missing
            self.raw_content = "# Soul\n\n## Identity Bindings\n\n".to_string();
            self.save()?;
            return Ok(());
        }

        let content = fs::read_to_string(&self.path).context("Failed to read SOUL.md")?;
        self.raw_content = content.clone();
        self.bindings.clear();

        let mut in_bindings_section = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("## Identity Bindings") {
                in_bindings_section = true;
                continue;
            } else if trimmed.starts_with("##") && in_bindings_section {
                in_bindings_section = false;
                continue;
            }

            if in_bindings_section && trimmed.starts_with("- **") {
                // Parse line: - **Provider**: ID (Level N)
                if let Some(binding) = Self::parse_binding_line(trimmed) {
                    self.bindings.push(binding);
                }
            }
        }

        Ok(())
    }

    fn parse_binding_line(line: &str) -> Option<IdentityBinding> {
        // Regex would be cleaner but avoiding extra deps for now.
        // Format: "- **Provider**: ID (Level N)"

        let parts: Vec<&str> = line.splitn(2, "**").collect();
        if parts.len() < 2 {
            return None;
        }

        let rest = parts[1]; // "Provider**: ID (Level N)"
        let parts2: Vec<&str> = rest.splitn(2, "**:").collect();
        if parts2.len() < 2 {
            return None;
        }

        let provider = parts2[0].trim().to_string();
        let val_part = parts2[1].trim(); // "ID (Level N)"

        // Find last "(" to split ID and Level
        if let Some(idx) = val_part.rfind(" (Level ") {
            let id = val_part[..idx].trim().to_string();
            let level_str = &val_part[idx + 8..val_part.len() - 1]; // "N"

            let trust_level = match level_str {
                "3" | "High" => TrustLevel::High,
                _ => TrustLevel::Low,
            };

            return Some(IdentityBinding {
                provider,
                id,
                trust_level,
                created_at: String::new(), // Not stored in simple markdown
            });
        }

        None
    }

    pub fn save(&self) -> Result<()> {
        // Simpler implementation: Just recreate the file content if we modify it.
        // For now, if we just want to Initialize, standard write is enough.
        // If we want to Preserve other content, we need a smarter replacer.

        // Current strategy: If file exists, read it, replace/append Indentity Bindings section.
        // But for MVP, let's just Append if not present or Rewrite if we manage the whole file.

        // For this task, strict "rewrite bindings" is acceptable if we claim ownership of that section.
        // But let's just write to file if it was empty.

        if !self.path.exists() {
            fs::write(&self.path, &self.raw_content)?;
        }

        // TODO: Implement smart section replacement to update bindings persistently without wiping user notes.
        // For now, load() reads it, but simple save() might not update bindings in-place perfectly without regex.
        // We will assumes 'save' is mostly for initialization or simple appends.

        Ok(())
    }

    /// Add a binding (Appends to file immediately for persistence)
    pub fn add_binding(&mut self, provider: &str, id: &str, level: TrustLevel) -> Result<()> {
        // check for duplicates
        if self
            .bindings
            .iter()
            .any(|b| b.provider == provider && b.id == id)
        {
            return Ok(());
        }

        // Access file content
        let mut content = self.raw_content.clone();

        // Check if section exists
        if !content.contains("## Identity Bindings") {
            content.push_str("\n\n## Identity Bindings\n");
        }

        let level_int = match level {
            TrustLevel::High => 3,
            TrustLevel::Medium => 2,
            TrustLevel::Low => 1,
        };

        let line = format!("- **{}**: {} (Level {})\n", provider, id, level_int);

        // Insert after header
        // Simple string manipulation:
        let header = "## Identity Bindings";
        if let Some(idx) = content.find(header) {
            let insert_pos = idx + header.len();
            // Find next newline
            if let Some(mut next_nl) = content[insert_pos..].find('\n') {
                // Insert after the newline of header
                next_nl += insert_pos + 1;
                content.insert_str(next_nl, &line);
            } else {
                // EOF after header
                content.push('\n');
                content.push_str(&line);
            }
        }

        self.raw_content = content;
        fs::write(&self.path, &self.raw_content)?;

        // Reload to update memory struct
        self.load()?;

        Ok(())
    }

    /// Compute the maximum trust level across all bindings.
    pub fn max_trust_level(&self) -> TrustLevel {
        self.bindings
            .iter()
            .map(|b| b.trust_level)
            .max_by_key(|t| *t as u8)
            .unwrap_or(TrustLevel::Low)
    }

    /// Check if any binding exists for the given provider.
    pub fn has_binding(&self, provider: &str) -> bool {
        self.bindings.iter().any(|b| b.provider == provider)
    }

    pub fn get_diary_entries(&self, limit: usize) -> Vec<DiaryEntry> {
        let mut entries = Vec::new();
        let content = &self.raw_content;

        let mut in_diary = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("## Diary") {
                in_diary = true;
                continue;
            } else if trimmed.starts_with("##") && in_diary {
                in_diary = false;
                continue;
            }

            if in_diary && trimmed.starts_with("- **") {
                // - **YYYY-MM-DD HH:MM**: content
                if let Some((ts, text)) = trimmed
                    .strip_prefix("- **")
                    .and_then(|s| s.split_once("**: "))
                {
                    entries.push(DiaryEntry {
                        timestamp: ts.to_string(),
                        content: text.to_string(),
                    });
                }
            }
        }

        // Return last N entries
        entries.into_iter().rev().take(limit).collect()
    }

    pub fn append_diary_entry(&mut self, content: &str) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M").to_string();
        let line = format!("- **{}**: {}\n", timestamp, content.trim());

        let mut file_content = self.raw_content.clone();

        // Check if section exists
        if !file_content.contains("## Diary") {
            file_content.push_str("\n\n## Diary\n");
        }

        // Append to Diary section
        let header = "## Diary";
        if let Some(idx) = file_content.find(header) {
            let insert_pos = idx + header.len();
            if let Some(mut next_nl) = file_content[insert_pos..].find('\n') {
                next_nl += insert_pos + 1;
                file_content.insert_str(next_nl, &line);
            } else {
                file_content.push('\n');
                file_content.push_str(&line);
            }
        }

        self.raw_content = file_content;
        fs::write(&self.path, &self.raw_content)?;
        self.load()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_soul() -> (Soul, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let mut soul = Soul::new(dir.path());
        soul.load().unwrap();
        (soul, dir)
    }

    // ── eIDAS Binding Tests ──────────────────────────────────────────

    #[test]
    fn eidas_binding_has_high_trust_level() {
        let (mut soul, _dir) = make_soul();
        soul.add_binding("eIDAS", "DE-abc12345", TrustLevel::High)
            .unwrap();

        assert_eq!(soul.bindings.len(), 1);
        assert_eq!(soul.bindings[0].provider, "eIDAS");
        assert_eq!(soul.bindings[0].id, "DE-abc12345");
        assert_eq!(soul.bindings[0].trust_level, TrustLevel::High);
    }

    #[test]
    fn eidas_binding_persists_to_file() {
        let dir = tempdir().unwrap();

        // Create and bind
        {
            let mut soul = Soul::new(dir.path());
            soul.load().unwrap();
            soul.add_binding("eIDAS", "AT-eid-001", TrustLevel::High)
                .unwrap();
        }

        // Reload from file
        {
            let mut soul = Soul::new(dir.path());
            soul.load().unwrap();
            assert_eq!(soul.bindings.len(), 1);
            assert_eq!(soul.bindings[0].provider, "eIDAS");
            assert_eq!(soul.bindings[0].trust_level, TrustLevel::High);
        }
    }

    // ── Google OIDC Binding Tests ────────────────────────────────────

    #[test]
    fn google_oidc_binding_has_low_trust_level() {
        let (mut soul, _dir) = make_soul();
        soul.add_binding("Google OIDC", "user@gmail.com", TrustLevel::Low)
            .unwrap();

        assert_eq!(soul.bindings.len(), 1);
        assert_eq!(soul.bindings[0].provider, "Google OIDC");
        assert_eq!(soul.bindings[0].trust_level, TrustLevel::Low);
    }

    #[test]
    fn oidc_binding_persists_and_reloads() {
        let dir = tempdir().unwrap();

        {
            let mut soul = Soul::new(dir.path());
            soul.load().unwrap();
            soul.add_binding("Google OIDC", "user@gmail.com", TrustLevel::Low)
                .unwrap();
        }

        {
            let mut soul = Soul::new(dir.path());
            soul.load().unwrap();
            assert_eq!(soul.bindings.len(), 1);
            assert_eq!(soul.bindings[0].provider, "Google OIDC");
        }
    }

    // ── SSI Wallet Binding Tests ─────────────────────────────────────

    #[test]
    fn ssi_wallet_binding_has_high_trust() {
        let (mut soul, _dir) = make_soul();
        soul.add_binding("SSI-Wallet", "did:key:z6MkpXyz", TrustLevel::High)
            .unwrap();

        assert_eq!(soul.bindings.len(), 1);
        assert_eq!(soul.bindings[0].provider, "SSI-Wallet");
        assert_eq!(soul.bindings[0].id, "did:key:z6MkpXyz");
        assert_eq!(soul.bindings[0].trust_level, TrustLevel::High);
    }

    #[test]
    fn ssi_wallet_binding_persists() {
        let dir = tempdir().unwrap();

        {
            let mut soul = Soul::new(dir.path());
            soul.load().unwrap();
            soul.add_binding("SSI-Wallet", "did:web:example.com", TrustLevel::High)
                .unwrap();
        }

        {
            let mut soul = Soul::new(dir.path());
            soul.load().unwrap();
            assert_eq!(soul.bindings.len(), 1);
            assert!(soul.bindings[0].id.starts_with("did:"));
        }
    }

    // ── Multi-Provider / Combination Tests ───────────────────────────

    #[test]
    fn multi_provider_binding_all_three() {
        let (mut soul, _dir) = make_soul();

        soul.add_binding("Google OIDC", "user@gmail.com", TrustLevel::Low)
            .unwrap();
        soul.add_binding("eIDAS", "DE-abc12345", TrustLevel::High)
            .unwrap();
        soul.add_binding("SSI-Wallet", "did:key:z6MkpXyz", TrustLevel::High)
            .unwrap();

        assert_eq!(soul.bindings.len(), 3);

        // Verify each provider is present
        let providers: Vec<&str> = soul.bindings.iter().map(|b| b.provider.as_str()).collect();
        assert!(providers.contains(&"Google OIDC"));
        assert!(providers.contains(&"eIDAS"));
        assert!(providers.contains(&"SSI-Wallet"));
    }

    #[test]
    fn multi_provider_binding_persists_across_sessions() {
        let dir = tempdir().unwrap();

        {
            let mut soul = Soul::new(dir.path());
            soul.load().unwrap();
            soul.add_binding("Google OIDC", "user@gmail.com", TrustLevel::Low)
                .unwrap();
            soul.add_binding("eIDAS", "DE-abc12345", TrustLevel::High)
                .unwrap();
            soul.add_binding("SSI-Wallet", "did:key:z6MkpXyz", TrustLevel::High)
                .unwrap();
        }

        {
            let mut soul = Soul::new(dir.path());
            soul.load().unwrap();
            assert_eq!(soul.bindings.len(), 3);
        }
    }

    #[test]
    fn max_trust_level_from_bindings() {
        let (mut soul, _dir) = make_soul();

        // Only low-trust binding
        soul.add_binding("Google OIDC", "user@gmail.com", TrustLevel::Low)
            .unwrap();
        let max = soul
            .bindings
            .iter()
            .map(|b| b.trust_level.clone())
            .max_by_key(|t| match t {
                TrustLevel::Low => 1,
                TrustLevel::Medium => 2,
                TrustLevel::High => 3,
            })
            .unwrap();
        assert_eq!(max, TrustLevel::Low);

        // Add high-trust binding
        soul.add_binding("eIDAS", "DE-abc12345", TrustLevel::High)
            .unwrap();
        let max = soul
            .bindings
            .iter()
            .map(|b| b.trust_level.clone())
            .max_by_key(|t| match t {
                TrustLevel::Low => 1,
                TrustLevel::Medium => 2,
                TrustLevel::High => 3,
            })
            .unwrap();
        assert_eq!(max, TrustLevel::High);
    }

    // ── Duplicate Prevention ─────────────────────────────────────────

    #[test]
    fn duplicate_binding_is_ignored() {
        let (mut soul, _dir) = make_soul();

        soul.add_binding("eIDAS", "DE-abc12345", TrustLevel::High)
            .unwrap();
        soul.add_binding("eIDAS", "DE-abc12345", TrustLevel::High)
            .unwrap();

        assert_eq!(soul.bindings.len(), 1);
    }

    #[test]
    fn same_provider_different_id_adds_both() {
        let (mut soul, _dir) = make_soul();

        soul.add_binding("Google OIDC", "user1@gmail.com", TrustLevel::Low)
            .unwrap();
        soul.add_binding("Google OIDC", "user2@gmail.com", TrustLevel::Low)
            .unwrap();

        assert_eq!(soul.bindings.len(), 2);
    }

    // ── Binding + Diary Coexistence ──────────────────────────────────

    #[test]
    fn bindings_and_diary_coexist() {
        let (mut soul, _dir) = make_soul();

        soul.add_binding("eIDAS", "DE-abc12345", TrustLevel::High)
            .unwrap();
        soul.append_diary_entry("Linked eIDAS identity").unwrap();
        soul.add_binding("Google OIDC", "user@gmail.com", TrustLevel::Low)
            .unwrap();
        soul.append_diary_entry("Linked Google OIDC identity")
            .unwrap();

        assert_eq!(soul.bindings.len(), 2);
        let diary = soul.get_diary_entries(10);
        assert_eq!(diary.len(), 2);
    }

    // ── Parse Line Edge Cases ────────────────────────────────────────

    #[test]
    fn parse_binding_line_valid() {
        let binding = Soul::parse_binding_line("- **eIDAS**: DE-abc12345 (Level 3)");
        assert!(binding.is_some());
        let b = binding.unwrap();
        assert_eq!(b.provider, "eIDAS");
        assert_eq!(b.id, "DE-abc12345");
        assert_eq!(b.trust_level, TrustLevel::High);
    }

    #[test]
    fn parse_binding_line_low_trust() {
        let binding = Soul::parse_binding_line("- **Google OIDC**: user@gmail.com (Level 1)");
        assert!(binding.is_some());
        let b = binding.unwrap();
        assert_eq!(b.provider, "Google OIDC");
        assert_eq!(b.trust_level, TrustLevel::Low);
    }

    #[test]
    fn parse_binding_line_invalid() {
        assert!(Soul::parse_binding_line("random text").is_none());
        assert!(Soul::parse_binding_line("- no bold markers").is_none());
        assert!(Soul::parse_binding_line("- **NoLevel**: id123").is_none());
    }
}
