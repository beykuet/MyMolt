// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use super::traits::{Memory, MemoryCategory, MemoryEntry};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct SimpleMemory {
    store: RwLock<HashMap<String, MemoryEntry>>,
}

impl SimpleMemory {
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Memory for SimpleMemory {
    fn name(&self) -> &str {
        "simple"
    }

    async fn store(&self, key: &str, content: &str, category: MemoryCategory) -> anyhow::Result<()> {
        let entry = MemoryEntry {
            key: key.to_string(),
            content: content.to_string(),
            category,
            timestamp: chrono::Utc::now().to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            session_id: None,
            score: None,
        };
        self.store.write().unwrap().insert(key.to_string(), entry);
        Ok(())
    }

    async fn recall(&self, _query: &str, _limit: usize) -> anyhow::Result<Vec<MemoryEntry>> {
        Ok(vec![])
    }

    async fn get(&self, key: &str) -> anyhow::Result<Option<MemoryEntry>> {
        Ok(self.store.read().unwrap().get(key).cloned())
    }

    async fn list(&self, _category: Option<&MemoryCategory>) -> anyhow::Result<Vec<MemoryEntry>> {
        Ok(self.store.read().unwrap().values().cloned().collect())
    }

    async fn forget(&self, key: &str) -> anyhow::Result<bool> {
        Ok(self.store.write().unwrap().remove(key).is_some())
    }

    async fn count(&self) -> anyhow::Result<usize> {
        Ok(self.store.read().unwrap().len())
    }

    async fn health_check(&self) -> bool {
        true
    }
}
