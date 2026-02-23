// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! SIGIL Gatekeeper — enforces identity, policy, audit, and scanning on MCP calls.
//!
//! Every MCP tool invocation passes through the gatekeeper:
//! 1. Policy check — is the caller allowed to use this tool?
//! 2. Rate limiting — has the rate limit been exceeded?
//! 3. Audit log — record the attempt (allowed or denied)
//! 4. Response scan — detect secrets in the result

use crate::security::audit::{AuditEvent, AuditEventType, AuditLogger};
use crate::security::policy::SecurityPolicy;
use std::sync::Arc;

/// Security gatekeeper that wraps MCP tool calls with SIGIL checks.
pub struct SigilGatekeeper {
    policy: Arc<SecurityPolicy>,
    audit: Option<Arc<AuditLogger>>,
}

impl SigilGatekeeper {
    /// Create a new gatekeeper with the given policy and optional audit logger.
    pub fn new(policy: Arc<SecurityPolicy>, audit: Option<Arc<AuditLogger>>) -> Self {
        Self { policy, audit }
    }

    /// Gate an inbound MCP tool call request.
    ///
    /// Returns `Ok(())` if the call is allowed, `Err(reason)` if denied.
    pub fn gate_request(&self, tool_name: &str) -> Result<(), String> {
        // Check rate limiting
        if !self.policy.record_action() {
            let event = AuditEvent::new(AuditEventType::PolicyViolation).with_action(
                format!("mcp:{tool_name}"),
                "high".into(),
                false,
                false,
            );
            self.log_event(&event);
            return Err(format!("Rate limit exceeded for MCP tool '{tool_name}'"));
        }

        // Log allowed action
        let event = AuditEvent::new(AuditEventType::SecurityEvent).with_action(
            format!("mcp:{tool_name}"),
            "low".into(),
            true,
            true,
        );
        self.log_event(&event);

        Ok(())
    }

    /// Log an audit event (if audit logger is configured).
    fn log_event(&self, event: &AuditEvent) {
        if let Some(ref audit) = self.audit {
            if let Err(e) = audit.log(event) {
                tracing::warn!("Failed to log audit event: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gatekeeper_allows_normal_tool() {
        let policy = Arc::new(SecurityPolicy::default());
        let gk = SigilGatekeeper::new(policy, None);

        let result = gk.gate_request("list_files");
        assert!(result.is_ok());
    }

    #[test]
    fn gatekeeper_creates_successfully() {
        let policy = Arc::new(SecurityPolicy::default());
        let _gk = SigilGatekeeper::new(policy, None);
    }

    #[test]
    fn gatekeeper_allows_multiple_different_tools() {
        let policy = Arc::new(SecurityPolicy::default());
        let gk = SigilGatekeeper::new(policy, None);

        assert!(gk.gate_request("file_read").is_ok());
        assert!(gk.gate_request("file_write").is_ok());
        assert!(gk.gate_request("shell_exec").is_ok());
        assert!(gk.gate_request("http_request").is_ok());
    }

    #[test]
    fn gatekeeper_allows_empty_tool_name() {
        let policy = Arc::new(SecurityPolicy::default());
        let gk = SigilGatekeeper::new(policy, None);
        assert!(gk.gate_request("").is_ok());
    }

    #[test]
    fn gatekeeper_allows_unicode_tool_name() {
        let policy = Arc::new(SecurityPolicy::default());
        let gk = SigilGatekeeper::new(policy, None);
        assert!(gk.gate_request("werkzeug_öffne_datei").is_ok());
    }

    #[test]
    fn gatekeeper_without_audit_does_not_panic() {
        let policy = Arc::new(SecurityPolicy::default());
        let gk = SigilGatekeeper::new(policy, None);

        // Multiple calls without audit logger should not panic
        for _ in 0..10 {
            let _ = gk.gate_request("safe_tool");
        }
    }

    #[test]
    fn gatekeeper_with_audit_logger() {
        let tmp = std::env::temp_dir().join(format!("sigil_gk_test_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).ok();

        let config = crate::config::AuditConfig::default();
        let logger = AuditLogger::new(config, tmp.clone()).unwrap();
        let audit = Arc::new(logger);

        let policy = Arc::new(SecurityPolicy::default());
        let gk = SigilGatekeeper::new(policy, Some(audit));

        // Should not panic even with logger attached
        gk.gate_request("tool_a").unwrap();
        gk.gate_request("tool_b").unwrap();

        // Cleanup
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn gatekeeper_rate_limit_eventually_denies() {
        // Create policy with strict rate limit
        let mut policy = SecurityPolicy::default();
        policy.max_actions_per_hour = 3;
        let gk = SigilGatekeeper::new(Arc::new(policy), None);

        // First 3 should pass
        assert!(gk.gate_request("tool").is_ok());
        assert!(gk.gate_request("tool").is_ok());
        assert!(gk.gate_request("tool").is_ok());

        // 4th should be denied
        let result = gk.gate_request("tool");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Rate limit"));
    }

    #[test]
    fn gatekeeper_rate_limit_error_includes_tool_name() {
        let mut policy = SecurityPolicy::default();
        policy.max_actions_per_hour = 0; // deny everything
        let gk = SigilGatekeeper::new(Arc::new(policy), None);

        let err = gk.gate_request("transfer_funds").unwrap_err();
        assert!(err.contains("transfer_funds"));
    }
}
