use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrustLevel {
    /// Basic email/OIDC verification (Google/Apple)
    Low = 1,
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
        if parts.len() < 2 { return None; }
        
        let rest = parts[1]; // "Provider**: ID (Level N)"
        let parts2: Vec<&str> = rest.splitn(2, "**:").collect();
        if parts2.len() < 2 { return None; }
        
        let provider = parts2[0].trim().to_string();
        let val_part = parts2[1].trim(); // "ID (Level N)"
        
        // Find last "(" to split ID and Level
        if let Some(idx) = val_part.rfind(" (Level ") {
            let id = val_part[..idx].trim().to_string();
            let level_str = &val_part[idx + 8 .. val_part.len() - 1]; // "N"
            
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
        if self.bindings.iter().any(|b| b.provider == provider && b.id == id) {
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
}
