// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

pub mod chunker;
pub mod embeddings;
pub mod hygiene;
pub mod markdown;
pub mod scoped;
pub mod sqlite;
pub mod traits;
pub mod vector;

pub mod sovereign;

pub use markdown::MarkdownMemory;
pub use sqlite::SqliteMemory;
pub mod simple;
pub use traits::Memory;
#[allow(unused_imports)]
pub use traits::{MemoryCategory, MemoryEntry};

use crate::config::MemoryConfig;
use std::path::Path;
use std::sync::Arc;

/// Factory: create the right memory backend from config
pub fn create_memory(
    config: &MemoryConfig,
    workspace_dir: &Path,
    api_key: Option<&str>,
    audit: Arc<crate::security::AuditLogger>,
) -> anyhow::Result<Box<dyn Memory>> {
    // Best-effort memory hygiene/retention pass (throttled by state file).
    if let Err(e) = hygiene::run_if_due(config, workspace_dir) {
        tracing::warn!("memory hygiene skipped: {e}");
    }

    let backend: Box<dyn Memory> = match config.backend.as_str() {
        "sqlite" => {
            let embedder: Arc<dyn embeddings::EmbeddingProvider> =
                Arc::from(embeddings::create_embedding_provider(
                    &config.embedding_provider,
                    api_key,
                    &config.embedding_model,
                    config.embedding_dimensions,
                ));

            #[allow(clippy::cast_possible_truncation)]
            let mem = SqliteMemory::with_embedder(
                workspace_dir,
                embedder,
                config.vector_weight as f32,
                config.keyword_weight as f32,
                config.embedding_cache_size,
            )?;
            Box::new(mem)
        }
        "markdown" | "none" => Box::new(MarkdownMemory::new(workspace_dir)),
        other => {
            tracing::warn!("Unknown memory backend '{other}', falling back to markdown");
            Box::new(MarkdownMemory::new(workspace_dir))
        }
    };

    // Wrap with SovereignMemory (The Guard)
    Ok(Box::new(sovereign::SovereignMemory::new(
        Arc::from(backend),
        workspace_dir,
        audit,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn factory_sqlite() {
        let tmp = TempDir::new().unwrap();
        let cfg = MemoryConfig {
            backend: "sqlite".into(),
            ..MemoryConfig::default()
        };
        let audit = Arc::new(
            crate::security::AuditLogger::new(
                crate::config::AuditConfig::default(),
                tmp.path().to_path_buf(),
            )
            .unwrap(),
        );
        let mem = create_memory(&cfg, tmp.path(), None, audit).unwrap();
        // Wrapped in sovereign
        assert_eq!(mem.name(), "sovereign");
    }

    #[test]
    fn factory_markdown() {
        let tmp = TempDir::new().unwrap();
        let cfg = MemoryConfig {
            backend: "markdown".into(),
            ..MemoryConfig::default()
        };
        let audit = Arc::new(
            crate::security::AuditLogger::new(
                crate::config::AuditConfig::default(),
                tmp.path().to_path_buf(),
            )
            .unwrap(),
        );
        let mem = create_memory(&cfg, tmp.path(), None, audit).unwrap();
        assert_eq!(mem.name(), "sovereign");
    }

    #[test]
    fn factory_none_falls_back_to_markdown() {
        let tmp = TempDir::new().unwrap();
        let cfg = MemoryConfig {
            backend: "none".into(),
            ..MemoryConfig::default()
        };
        let audit = Arc::new(
            crate::security::AuditLogger::new(
                crate::config::AuditConfig::default(),
                tmp.path().to_path_buf(),
            )
            .unwrap(),
        );
        let mem = create_memory(&cfg, tmp.path(), None, audit).unwrap();
        assert_eq!(mem.name(), "sovereign");
    }

    #[test]
    fn factory_unknown_falls_back_to_markdown() {
        let tmp = TempDir::new().unwrap();
        let cfg = MemoryConfig {
            backend: "redis".into(),
            ..MemoryConfig::default()
        };
        let audit = Arc::new(
            crate::security::AuditLogger::new(
                crate::config::AuditConfig::default(),
                tmp.path().to_path_buf(),
            )
            .unwrap(),
        );
        let mem = create_memory(&cfg, tmp.path(), None, audit).unwrap();
        assert_eq!(mem.name(), "sovereign");
    }
}
