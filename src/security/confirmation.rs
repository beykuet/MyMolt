// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! Confirmation gate for high-risk tool execution.
//!
//! When a tool requires user confirmation (configured via
//! `security.confirmation_required` in YAML), the `SecurityWrapper`
//! calls `ConfirmationGate::request()`, which:
//!
//! 1. Generates a unique request ID
//! 2. Stores a `oneshot::Sender` in a pending map
//! 3. Notifies all registered listeners (WebSocket, Telegram, CLI)
//! 4. Awaits the `oneshot::Receiver` with a 30-second timeout
//!
//! The frontend resolves the request by calling `gate.resolve(id, approved)`.
//! If no response arrives within the timeout, the request is auto-denied.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex, broadcast};

/// A pending confirmation request sent to the user.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConfirmationRequest {
    /// Unique request ID (UUID v4).
    pub id: String,
    /// Tool name requiring confirmation.
    pub tool_name: String,
    /// Human-readable summary of what the tool will do.
    pub description: String,
    /// Risk level (from SecurityPolicy).
    pub risk_level: String,
    /// Timestamp of the request.
    pub requested_at: String,
    /// Seconds until auto-deny.
    pub timeout_secs: u64,
}

/// Response to a confirmation request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfirmationResponse {
    /// The request ID being responded to.
    pub id: String,
    /// Whether the user approved the action.
    pub approved: bool,
}

/// The confirmation gate: manages pending requests and their resolution.
pub struct ConfirmationGate {
    /// Pending requests waiting for user response.
    pending: Mutex<HashMap<String, (ConfirmationRequest, oneshot::Sender<bool>)>>,
    /// Broadcast channel for notifying listeners of new requests.
    notify_tx: broadcast::Sender<ConfirmationRequest>,
    /// Default timeout in seconds.
    timeout_secs: u64,
}

impl ConfirmationGate {
    /// Create a new confirmation gate.
    ///
    /// `timeout_secs` is the default auto-deny timeout (30 recommended).
    pub fn new(timeout_secs: u64) -> Arc<Self> {
        let (notify_tx, _) = broadcast::channel(64);
        Arc::new(Self {
            pending: Mutex::new(HashMap::new()),
            notify_tx,
            timeout_secs,
        })
    }

    /// Subscribe to confirmation request notifications.
    ///
    /// WebSocket handlers, Telegram channels, and CLI loops call this
    /// to receive `ConfirmationRequest` events.
    pub fn subscribe(&self) -> broadcast::Receiver<ConfirmationRequest> {
        self.notify_tx.subscribe()
    }

    /// Request user confirmation for a tool execution.
    ///
    /// Returns `true` if approved, `false` if denied or timed out.
    /// The caller (SecurityWrapper) should block on this.
    pub async fn request(&self, tool_name: &str, args_summary: &str) -> bool {
        let id = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();

        let req = ConfirmationRequest {
            id: id.clone(),
            tool_name: tool_name.to_string(),
            description: args_summary.to_string(),
            risk_level: "high".to_string(),
            requested_at: chrono::Utc::now().to_rfc3339(),
            timeout_secs: self.timeout_secs,
        };

        // Store the pending sender
        {
            let mut pending = self.pending.lock().await;
            pending.insert(id.clone(), (req.clone(), tx));
        }

        // Notify all listeners (WebSocket, Telegram, CLI)
        // If no listeners are subscribed, this is a no-op (broadcast drops).
        let _ = self.notify_tx.send(req);

        // Await response with timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            rx,
        )
        .await;

        // Clean up
        {
            let mut pending = self.pending.lock().await;
            pending.remove(&id);
        }

        match result {
            Ok(Ok(approved)) => approved,
            Ok(Err(_)) => {
                // Sender dropped — treat as denied
                tracing::warn!(request_id = %id, "Confirmation sender dropped");
                false
            }
            Err(_) => {
                // Timeout — auto-deny
                tracing::info!(
                    request_id = %id,
                    tool = tool_name,
                    "Confirmation timed out after {}s — auto-denied",
                    self.timeout_secs
                );
                false
            }
        }
    }

    /// Resolve a pending confirmation request.
    ///
    /// Called by the frontend (via API/WebSocket) or CLI when the user
    /// approves or denies the action.
    ///
    /// Returns `true` if the request was found and resolved, `false` if
    /// the request ID was not found (expired or already resolved).
    pub async fn resolve(&self, id: &str, approved: bool) -> bool {
        let sender = {
            let mut pending = self.pending.lock().await;
            pending.remove(id)
        };

        match sender {
            Some((_, tx)) => {
                let _ = tx.send(approved);
                tracing::info!(
                    request_id = %id,
                    approved,
                    "Confirmation resolved"
                );
                true
            }
            None => {
                tracing::warn!(
                    request_id = %id,
                    "Confirmation resolve failed: request not found (expired?)"
                );
                false
            }
        }
    }

    /// Get all pending confirmation requests.
    pub async fn get_pending(&self) -> Vec<ConfirmationRequest> {
        self.pending.lock().await.values().map(|(req, _)| req.clone()).collect()
    }

    /// Get the number of pending confirmation requests.
    pub async fn pending_count(&self) -> usize {
        self.pending.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn approved_request_returns_true() {
        let gate = ConfirmationGate::new(5);
        let gate_clone = Arc::clone(&gate);

        // Spawn a task that approves after a short delay
        let handle = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            // We need to resolve by ID, so subscribe to get the ID
            gate_clone.resolve("test-id", true).await
        });

        // Manually insert a pending request for "test-id"
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = gate.pending.lock().await;
            pending.insert("test-id".to_string(), (ConfirmationRequest {
                id: "test-id".into(), tool_name: "test".into(), description: "test".into(),
                risk_level: "high".into(), requested_at: "now".into(), timeout_secs: 5
            }, tx));
        }

        // Spawn resolver
        handle.await.unwrap();

        // The receiver should get true
        assert!(rx.await.unwrap());
    }

    #[tokio::test]
    async fn denied_request_returns_false() {
        let gate = ConfirmationGate::new(5);
        let gate_clone = Arc::clone(&gate);

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = gate.pending.lock().await;
            pending.insert("deny-id".to_string(), (ConfirmationRequest {
                id: "deny-id".into(), tool_name: "test".into(), description: "test".into(),
                risk_level: "high".into(), requested_at: "now".into(), timeout_secs: 5
            }, tx));
        }

        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            gate_clone.resolve("deny-id", false).await;
        });

        assert!(!rx.await.unwrap());
    }

    #[tokio::test]
    async fn timeout_auto_denies() {
        let gate = ConfirmationGate::new(1); // 1 second timeout

        let result = gate.request("shell", "rm -rf /tmp/test").await;
        assert!(!result, "Timed-out request should be denied");
    }

    #[tokio::test]
    async fn resolve_nonexistent_returns_false() {
        let gate = ConfirmationGate::new(5);
        assert!(!gate.resolve("does-not-exist", true).await);
    }

    #[tokio::test]
    async fn pending_count_tracks_correctly() {
        let gate = ConfirmationGate::new(30);

        assert_eq!(gate.pending_count().await, 0);

        let (tx, _rx) = oneshot::channel();
        {
            let mut pending = gate.pending.lock().await;
            pending.insert("a".into(), (ConfirmationRequest {
                id: "a".into(), tool_name: "test".into(), description: "test".into(),
                risk_level: "high".into(), requested_at: "now".into(), timeout_secs: 5
            }, tx));
        }
        assert_eq!(gate.pending_count().await, 1);

        gate.resolve("a", true).await;
        assert_eq!(gate.pending_count().await, 0);
    }

    #[tokio::test]
    async fn broadcast_notifies_subscribers() {
        let gate = ConfirmationGate::new(1);
        let mut rx = gate.subscribe();

        // Spawn request in background (will timeout in 1s)
        let gate_clone = Arc::clone(&gate);
        tokio::spawn(async move {
            gate_clone.request("shell", "echo hello").await;
        });

        // Should receive the notification
        let req = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            rx.recv(),
        )
        .await
        .expect("Should receive notification within 2s")
        .expect("Broadcast should not close");

        assert_eq!(req.tool_name, "shell");
        assert_eq!(req.description, "echo hello");
    }
}
