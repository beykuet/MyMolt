// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! Scoped memory wrapper — adds per-user isolation on top of any `Memory` backend.
//!
//! In family mode, each user gets a private scope for their conversations
//! while the `shared` scope is visible to everyone. The scoping is implemented
//! by prefixing memory keys with the user scope.

use super::traits::{Memory, MemoryCategory, MemoryEntry};
use async_trait::async_trait;
use std::sync::Arc;

/// Scoped memory: wraps an inner `Memory` to provide per-user isolation.
///
/// - `store()` always prefixes the key with the user scope.
/// - `recall()` searches both the user scope AND the `shared` scope.
/// - `forget()` only forgets keys in the user's own scope.
/// - `list()` returns both shared and private entries.
/// - `count()` counts both scopes.
pub struct ScopedMemory {
    inner: Arc<dyn Memory>,
    /// The user's scope (e.g. `"user:benjamin"` or `"shared"`).
    user_scope: String,
}

impl ScopedMemory {
    /// Create a new scoped memory wrapper.
    ///
    /// `user_scope` should be the output of `FamilyMember::scope()`.
    pub fn new(inner: Arc<dyn Memory>, user_scope: String) -> Self {
        Self { inner, user_scope }
    }

    /// Prefix a key with the user scope.
    fn scoped_key(&self, key: &str) -> String {
        format!("{}:{}", self.user_scope, key)
    }

    /// Prefix a key with the shared scope.
    fn shared_key(key: &str) -> String {
        format!("{}:{}", crate::identity::family::SCOPE_SHARED, key)
    }
}

#[async_trait]
impl Memory for ScopedMemory {
    fn name(&self) -> &str {
        "scoped"
    }

    async fn store(
        &self,
        key: &str,
        content: &str,
        category: MemoryCategory,
    ) -> anyhow::Result<()> {
        // Store under the user's own scope
        let scoped = self.scoped_key(key);
        self.inner.store(&scoped, content, category).await
    }

    async fn recall(&self, query: &str, limit: usize) -> anyhow::Result<Vec<MemoryEntry>> {
        // Recall from inner (which has both scoped and shared entries),
        // then filter to only this user's scope + shared.
        let all = self.inner.recall(query, limit * 3).await?;

        let user_prefix = format!("{}:", self.user_scope);
        let shared_prefix = format!("{}:", crate::identity::family::SCOPE_SHARED);

        let filtered: Vec<MemoryEntry> = all
            .into_iter()
            .filter(|e| e.key.starts_with(&user_prefix) || e.key.starts_with(&shared_prefix))
            .take(limit)
            .map(|mut e| {
                // Strip scope prefix from key for clean display
                if let Some(clean) = e.key.strip_prefix(&user_prefix) {
                    e.key = clean.to_string();
                } else if let Some(clean) = e.key.strip_prefix(&shared_prefix) {
                    e.key = format!("[shared] {clean}");
                }
                e
            })
            .collect();

        Ok(filtered)
    }

    async fn get(&self, key: &str) -> anyhow::Result<Option<MemoryEntry>> {
        // Try user scope first, then shared
        let scoped = self.scoped_key(key);
        if let Some(entry) = self.inner.get(&scoped).await? {
            return Ok(Some(entry));
        }

        let shared = Self::shared_key(key);
        self.inner.get(&shared).await
    }

    async fn list(&self, category: Option<&MemoryCategory>) -> anyhow::Result<Vec<MemoryEntry>> {
        let all = self.inner.list(category).await?;

        let user_prefix = format!("{}:", self.user_scope);
        let shared_prefix = format!("{}:", crate::identity::family::SCOPE_SHARED);

        Ok(all
            .into_iter()
            .filter(|e| e.key.starts_with(&user_prefix) || e.key.starts_with(&shared_prefix))
            .collect())
    }

    async fn forget(&self, key: &str) -> anyhow::Result<bool> {
        // Only forget from own scope (can't delete shared memories)
        let scoped = self.scoped_key(key);
        self.inner.forget(&scoped).await
    }

    async fn count(&self) -> anyhow::Result<usize> {
        let all = self.inner.list(None).await?;
        let user_prefix = format!("{}:", self.user_scope);
        let shared_prefix = format!("{}:", crate::identity::family::SCOPE_SHARED);
        Ok(all
            .iter()
            .filter(|e| e.key.starts_with(&user_prefix) || e.key.starts_with(&shared_prefix))
            .count())
    }

    async fn health_check(&self) -> bool {
        self.inner.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::simple::SimpleMemory;

    fn make_scoped(scope: &str) -> (Arc<SimpleMemory>, ScopedMemory) {
        let inner = Arc::new(SimpleMemory::new());
        let scoped = ScopedMemory::new(inner.clone(), scope.to_string());
        (inner, scoped)
    }

    #[tokio::test]
    async fn store_prefixes_key_with_scope() {
        let (inner, scoped) = make_scoped("user:benjamin");
        scoped
            .store("favorite_lang", "Rust", MemoryCategory::Core)
            .await
            .unwrap();

        // The inner store should have the scoped key
        let entry = inner.get("user:benjamin:favorite_lang").await.unwrap();
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().content, "Rust");
    }

    #[tokio::test]
    async fn list_sees_own_scope_and_shared_not_others() {
        let (inner, scoped) = make_scoped("user:benjamin");

        // Store private entry via scoped wrapper
        scoped
            .store("private_note", "my secret", MemoryCategory::Core)
            .await
            .unwrap();

        // Store shared entry directly in inner
        inner
            .store("shared:family_plan", "vacation in July", MemoryCategory::Core)
            .await
            .unwrap();

        // Store another user's entry directly in inner
        inner
            .store("user:maria:her_note", "her secret", MemoryCategory::Core)
            .await
            .unwrap();

        let results = scoped.list(None).await.unwrap();
        let keys: Vec<&str> = results.iter().map(|e| e.key.as_str()).collect();

        // Should see own scoped entry
        assert!(
            keys.iter().any(|k| k.contains("benjamin") && k.contains("private_note")),
            "Should see own entry, got: {keys:?}"
        );
        // Should see shared entry
        assert!(
            keys.iter().any(|k| k.contains("shared") && k.contains("family_plan")),
            "Should see shared entry, got: {keys:?}"
        );
        // Should NOT see Maria's entry
        assert!(
            !keys.iter().any(|k| k.contains("maria")),
            "Should NOT see Maria's entry, got: {keys:?}"
        );
    }

    #[tokio::test]
    async fn forget_only_affects_own_scope() {
        let (inner, scoped) = make_scoped("user:luca");

        // Store entry
        scoped
            .store("game_score", "9001", MemoryCategory::Core)
            .await
            .unwrap();

        // Store shared entry
        inner
            .store("shared:wifi_password", "hunter2", MemoryCategory::Core)
            .await
            .unwrap();

        // Forget own entry → should succeed
        assert!(scoped.forget("game_score").await.unwrap());

        // Forget shared entry → should fail (scoped key doesn't match)
        assert!(!scoped.forget("wifi_password").await.unwrap());

        // Shared entry should still exist
        let shared = inner.get("shared:wifi_password").await.unwrap();
        assert!(shared.is_some());
    }

    #[tokio::test]
    async fn get_falls_back_to_shared() {
        let (inner, scoped) = make_scoped("user:helga");

        // Store only in shared
        inner
            .store("shared:doctor_name", "Dr. Müller", MemoryCategory::Core)
            .await
            .unwrap();

        // Get through scoped should find it
        let entry = scoped.get("doctor_name").await.unwrap();
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().content, "Dr. Müller");
    }

    #[tokio::test]
    async fn scoped_memory_name() {
        let (_inner, scoped) = make_scoped("user:test");
        assert_eq!(scoped.name(), "scoped");
    }

    #[tokio::test]
    async fn health_check_delegates() {
        let (_inner, scoped) = make_scoped("user:test");
        assert!(scoped.health_check().await);
    }
}
