// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! Role-based experience layer.
//!
//! Derives user **roles** from SIGIL trust levels and optional config,
//! then determines which capabilities are available. The UI adapts its
//! presentation based on the resolved role.

use crate::identity::soul::{Soul, TrustLevel};
use serde::{Deserialize, Serialize};

/// User roles derived from SIGIL identity trust levels.
///
/// Roles determine what the user sees and can do in the UI.
/// Enforcement happens in `SecurityPolicy`; roles are for UX adaptation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// Full sovereign control: SIGIL transparency, MCP config, all tools.
    Root,
    /// Everyday agent OS: chat, files, VPN, DNS, PIM.
    Adult,
    /// Guided assistance: voice-first, diary, reminders, simplified UI.
    Senior,
    /// Safe sandbox: no shell, no delegation, filtered browsing.
    Child,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Root => write!(f, "Root"),
            Role::Adult => write!(f, "Adult"),
            Role::Senior => write!(f, "Senior"),
            Role::Child => write!(f, "Child"),
        }
    }
}

/// Configuration for role derivation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConfig {
    /// Override: force a specific role regardless of trust level.
    #[serde(default)]
    pub override_role: Option<Role>,
    /// Whether this is a local/CLI session (implies Root if High trust).
    #[serde(default)]
    pub is_local: bool,
    /// Age-based hint (optional, e.g. from onboarding).
    #[serde(default)]
    pub user_age: Option<u8>,
}

impl Default for RoleConfig {
    fn default() -> Self {
        Self {
            override_role: None,
            is_local: true,
            user_age: None,
        }
    }
}

/// Resolve the user's role from their trust level and configuration.
///
/// Resolution order:
/// 1. **Explicit override** — if config specifies a role, use it
/// 2. **Child gate** — if user_age < 16, always Child
/// 3. **Trust-based** — High + local = Root, High + remote = Adult,
///    Low + age ≥ 65 = Senior, Low = Child
pub fn resolve_role(trust: TrustLevel, config: &RoleConfig) -> Role {
    // 1. Explicit override
    if let Some(role) = config.override_role {
        return role;
    }

    // 2. Child gate: age < 16 always gets Child
    if let Some(age) = config.user_age {
        if age < 16 {
            return Role::Child;
        }
    }

    // 3. Trust-based resolution
    match trust {
        TrustLevel::High => {
            if config.is_local {
                Role::Root
            } else {
                Role::Adult
            }
        }
        TrustLevel::Medium => {
            // Verified email/OIDC — Adult regardless of local/remote
            Role::Adult
        }
        TrustLevel::Low => {
            // Senior mode for elderly users with low trust
            if config.user_age.map_or(false, |age| age >= 65) {
                Role::Senior
            } else {
                Role::Child
            }
        }
    }
}

/// Resolve role directly from a Soul + config.
pub fn resolve_role_from_soul(soul: &Soul, config: &RoleConfig) -> Role {
    resolve_role(soul.max_trust_level(), config)
}

/// Capabilities available for each role.
#[derive(Debug, Clone)]
pub struct RoleCapabilities {
    pub can_use_shell: bool,
    pub can_delegate: bool,
    pub can_access_vault: bool,
    pub can_configure_mcp: bool,
    pub can_browse_unrestricted: bool,
    pub can_manage_pim: bool,
    pub can_view_audit_log: bool,
    pub voice_first: bool,
}

impl RoleCapabilities {
    /// Derive capabilities from a role.
    pub fn for_role(role: Role) -> Self {
        match role {
            Role::Root => Self {
                can_use_shell: true,
                can_delegate: true,
                can_access_vault: true,
                can_configure_mcp: true,
                can_browse_unrestricted: true,
                can_manage_pim: true,
                can_view_audit_log: true,
                voice_first: false,
            },
            Role::Adult => Self {
                can_use_shell: true,
                can_delegate: true,
                can_access_vault: true,
                can_configure_mcp: false,
                can_browse_unrestricted: true,
                can_manage_pim: true,
                can_view_audit_log: false,
                voice_first: false,
            },
            Role::Senior => Self {
                can_use_shell: false,
                can_delegate: false,
                can_access_vault: false,
                can_configure_mcp: false,
                can_browse_unrestricted: false,
                can_manage_pim: true,
                can_view_audit_log: false,
                voice_first: true,
            },
            Role::Child => Self {
                can_use_shell: false,
                can_delegate: false,
                can_access_vault: false,
                can_configure_mcp: false,
                can_browse_unrestricted: false,
                can_manage_pim: false,
                can_view_audit_log: false,
                voice_first: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_trust_local_is_root() {
        let config = RoleConfig {
            is_local: true,
            ..Default::default()
        };
        assert_eq!(resolve_role(TrustLevel::High, &config), Role::Root);
    }

    #[test]
    fn high_trust_remote_is_adult() {
        let config = RoleConfig {
            is_local: false,
            ..Default::default()
        };
        assert_eq!(resolve_role(TrustLevel::High, &config), Role::Adult);
    }

    #[test]
    fn low_trust_no_age_is_child() {
        let config = RoleConfig {
            is_local: false,
            ..Default::default()
        };
        assert_eq!(resolve_role(TrustLevel::Low, &config), Role::Child);
    }

    #[test]
    fn low_trust_elderly_is_senior() {
        let config = RoleConfig {
            is_local: false,
            user_age: Some(70),
            ..Default::default()
        };
        assert_eq!(resolve_role(TrustLevel::Low, &config), Role::Senior);
    }

    #[test]
    fn child_age_gate_overrides_high_trust() {
        let config = RoleConfig {
            is_local: true,
            user_age: Some(10),
            ..Default::default()
        };
        assert_eq!(resolve_role(TrustLevel::High, &config), Role::Child);
    }

    #[test]
    fn explicit_override_wins() {
        let config = RoleConfig {
            override_role: Some(Role::Senior),
            is_local: true,
            user_age: Some(25),
        };
        assert_eq!(resolve_role(TrustLevel::High, &config), Role::Senior);
    }

    #[test]
    fn root_has_all_capabilities() {
        let caps = RoleCapabilities::for_role(Role::Root);
        assert!(caps.can_use_shell);
        assert!(caps.can_delegate);
        assert!(caps.can_access_vault);
        assert!(caps.can_configure_mcp);
        assert!(caps.can_browse_unrestricted);
        assert!(caps.can_manage_pim);
        assert!(caps.can_view_audit_log);
        assert!(!caps.voice_first);
    }

    #[test]
    fn child_has_no_capabilities() {
        let caps = RoleCapabilities::for_role(Role::Child);
        assert!(!caps.can_use_shell);
        assert!(!caps.can_delegate);
        assert!(!caps.can_access_vault);
        assert!(!caps.can_configure_mcp);
        assert!(!caps.can_browse_unrestricted);
        assert!(!caps.can_manage_pim);
        assert!(!caps.can_view_audit_log);
    }

    #[test]
    fn senior_is_voice_first() {
        let caps = RoleCapabilities::for_role(Role::Senior);
        assert!(caps.voice_first);
        assert!(caps.can_manage_pim);
        assert!(!caps.can_use_shell);
    }

    #[test]
    fn adult_can_use_tools_but_not_configure() {
        let caps = RoleCapabilities::for_role(Role::Adult);
        assert!(caps.can_use_shell);
        assert!(caps.can_delegate);
        assert!(!caps.can_configure_mcp);
        assert!(!caps.can_view_audit_log);
    }

    #[test]
    fn role_display_formatting() {
        assert_eq!(format!("{}", Role::Root), "Root");
        assert_eq!(format!("{}", Role::Child), "Child");
        assert_eq!(format!("{}", Role::Senior), "Senior");
        assert_eq!(format!("{}", Role::Adult), "Adult");
    }

    #[test]
    fn resolve_from_soul_with_eidas() {
        let tmp = tempfile::tempdir().unwrap();
        let mut soul = Soul::new(tmp.path());
        soul.load().unwrap();
        soul.add_binding("eIDAS", "DE-123", TrustLevel::High)
            .unwrap();

        let config = RoleConfig::default();
        let role = resolve_role_from_soul(&soul, &config);
        assert_eq!(role, Role::Root);
    }

    #[test]
    fn resolve_from_soul_with_no_bindings() {
        let tmp = tempfile::tempdir().unwrap();
        let mut soul = Soul::new(tmp.path());
        soul.load().unwrap();

        let config = RoleConfig {
            is_local: false,
            ..Default::default()
        };
        let role = resolve_role_from_soul(&soul, &config);
        assert_eq!(role, Role::Child);
    }
}
