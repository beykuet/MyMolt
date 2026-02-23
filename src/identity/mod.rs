// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

pub mod aieos;
pub mod crypto;
pub mod family;
pub mod oidc;
pub mod oidc_generic;
pub mod roles;
pub mod soul;
pub mod ssi;

pub use aieos::{aieos_to_system_prompt, is_aieos_configured, load_aieos_identity};
pub use roles::{resolve_role, resolve_role_from_soul, Role, RoleCapabilities, RoleConfig};
pub use soul::Soul;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UserRole {
    Child,
    Senior,
    Adult,
    Root,
}

impl UserRole {
    /// Privilege level (higher = more access)
    pub fn level(&self) -> u8 {
        match self {
            Self::Child => 0,
            Self::Senior => 1,
            Self::Adult => 2,
            Self::Root => 3,
        }
    }
}

impl PartialOrd for UserRole {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UserRole {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.level().cmp(&other.level())
    }
}

impl Default for UserRole {
    fn default() -> Self {
        Self::Adult
    }
}
