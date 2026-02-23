// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! SIGIL Protocol trait implementations for MyMolt.
//!
//! This module bridges MyMolt's concrete security types with the
//! SIGIL protocol traits, making MyMolt the reference implementation.

/// Implement `sigil::SensitivityScanner` for MyMolt's `SensitivityScanner`.
impl sigil::SensitivityScanner for crate::memory::sovereign::SensitivityScanner {
    fn scan(&self, text: &str) -> Option<String> {
        // Delegate to MyMolt's existing regex-based scanner
        self.scan(text)
    }
}

/// Implement `sigil::AuditLogger` for MyMolt's `AuditLogger`.
impl sigil::AuditLogger for crate::security::AuditLogger {
    fn log(&self, event: &sigil::AuditEvent) -> anyhow::Result<()> {
        // Convert SIGIL event to MyMolt's internal event and delegate
        let mymolt_type = match event.event_type {
            sigil::AuditEventType::SigilInterception => {
                crate::security::AuditEventType::SigilInterception
            }
            sigil::AuditEventType::CommandExecution => {
                crate::security::AuditEventType::CommandExecution
            }
            sigil::AuditEventType::FileAccess => crate::security::AuditEventType::FileAccess,
            sigil::AuditEventType::ConfigChange => crate::security::AuditEventType::ConfigChange,
            sigil::AuditEventType::AuthSuccess => crate::security::AuditEventType::AuthSuccess,
            sigil::AuditEventType::AuthFailure => crate::security::AuditEventType::AuthFailure,
            sigil::AuditEventType::PolicyViolation => {
                crate::security::AuditEventType::PolicyViolation
            }
            sigil::AuditEventType::SecurityEvent => crate::security::AuditEventType::SecurityEvent,
            sigil::AuditEventType::McpToolGated => crate::security::AuditEventType::SecurityEvent,
            sigil::AuditEventType::DelegationCrossing => {
                crate::security::AuditEventType::DelegationCrossing
            }
        };

        let mymolt_event = crate::security::AuditEvent::new(mymolt_type).with_action(
            event.action.description.clone(),
            event.action.risk_level.clone(),
            event.action.approved,
            event.action.allowed,
        );

        self.log(&mymolt_event)
    }
}

/// Implement `sigil::IdentityProvider` for MyMolt's `Soul`.
///
/// This bridges SOUL.md-based identity bindings to the SIGIL protocol,
/// enabling trust-gated operations across agent interactions and MCP.
impl sigil::IdentityProvider for crate::identity::soul::Soul {
    fn bindings(&self) -> Vec<sigil::IdentityBinding> {
        self.bindings
            .iter()
            .map(|b| sigil::IdentityBinding {
                provider: b.provider.clone(),
                id: b.id.clone(),
                trust_level: match b.trust_level {
                    crate::identity::soul::TrustLevel::High => sigil::TrustLevel::High,
                    crate::identity::soul::TrustLevel::Medium => sigil::TrustLevel::Medium,
                    crate::identity::soul::TrustLevel::Low => sigil::TrustLevel::Low,
                },
                bound_at: b.created_at.clone(),
            })
            .collect()
    }

    fn add_binding(
        &mut self,
        provider: &str,
        id: &str,
        level: sigil::TrustLevel,
    ) -> anyhow::Result<()> {
        let mymolt_level = match level {
            sigil::TrustLevel::High => crate::identity::soul::TrustLevel::High,
            sigil::TrustLevel::Medium => crate::identity::soul::TrustLevel::Medium,
            sigil::TrustLevel::Low => crate::identity::soul::TrustLevel::Low,
        };
        self.add_binding(provider, id, mymolt_level)
    }

    fn max_trust_level(&self) -> sigil::TrustLevel {
        match self.max_trust_level() {
            crate::identity::soul::TrustLevel::High => sigil::TrustLevel::High,
            crate::identity::soul::TrustLevel::Medium => sigil::TrustLevel::Medium,
            crate::identity::soul::TrustLevel::Low => sigil::TrustLevel::Low,
        }
    }

    fn has_binding(&self, provider: &str) -> bool {
        self.has_binding(provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mymolt_scanner_implements_sigil_trait() {
        let scanner = crate::memory::sovereign::SensitivityScanner::new();
        let sigil_scanner: &dyn sigil::SensitivityScanner = &scanner;
        assert!(sigil_scanner.scan("sk-abc123def456ghi789jkl").is_some());
        assert!(sigil_scanner.scan("safe text").is_none());
    }

    #[test]
    fn mymolt_audit_logger_implements_sigil_trait() {
        let tmp = tempfile::tempdir().unwrap();
        let logger = crate::security::AuditLogger::new(
            crate::config::AuditConfig::default(),
            tmp.path().to_path_buf(),
        )
        .unwrap();

        let sigil_logger: &dyn sigil::AuditLogger = &logger;
        let event = sigil::AuditEvent::new(sigil::AuditEventType::SigilInterception).with_action(
            "Test redaction".into(),
            "low".into(),
            true,
            true,
        );

        assert!(sigil_logger.log(&event).is_ok());
    }

    #[test]
    fn soul_implements_sigil_identity_provider() {
        let tmp = tempfile::tempdir().unwrap();
        let mut soul = crate::identity::soul::Soul::new(tmp.path());
        soul.load().unwrap();
        soul.add_binding(
            "eIDAS",
            "DE-abc12345",
            crate::identity::soul::TrustLevel::High,
        )
        .unwrap();

        let provider: &dyn sigil::IdentityProvider = &soul;
        assert_eq!(provider.max_trust_level(), sigil::TrustLevel::High);
        assert!(provider.has_binding("eIDAS"));
        assert!(!provider.has_binding("Google"));
        assert_eq!(provider.bindings().len(), 1);
    }
}
