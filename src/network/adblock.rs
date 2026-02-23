// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use anyhow::Result;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DnsBlocker {
    enabled: Arc<RwLock<bool>>,
    blocklist: Arc<RwLock<HashSet<String>>>,
}

impl DnsBlocker {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(RwLock::new(true)),
            blocklist: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn load_defaults(&self) -> Result<()> {
        let mut list = self.blocklist.write().await;
        // Basic defaults. In a real app, we'd load from an external URL or file.
        list.insert("google-analytics.com".to_string());
        list.insert("doubleclick.net".to_string());
        list.insert("adservice.google.com".to_string());
        list.insert("facebook.com".to_string()); // For radical context blocking
        Ok(())
    }

    pub async fn is_blocked(&self, domain: &str) -> bool {
        if !*self.enabled.read().await {
            return false;
        }
        let list = self.blocklist.read().await;
        list.contains(domain)
    }

    pub async fn toggle(&self, enabled: bool) {
        let mut lock = self.enabled.write().await;
        *lock = enabled;
    }

    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    pub async fn count(&self) -> usize {
        self.blocklist.read().await.len()
    }
}
