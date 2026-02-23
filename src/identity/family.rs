// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! Family member registry — maps channel users to family members with roles.
//!
//! Each family member declares which channel accounts they use
//! (Telegram ID, WhatsApp number, Discord ID, etc.). At runtime,
//! the registry resolves an incoming `(channel, user_id)` pair to a
//! `FamilyMember` with a role and a unique memory scope.

use crate::identity::UserRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A registered family member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyMember {
    /// Display name (e.g. "Benjamin", "Oma Helga").
    pub name: String,
    /// Role for security policy.
    pub role: UserRole,
    /// Channel bindings: `{ "telegram": "12345678", "whatsapp": "+49170..." }`.
    pub channels: HashMap<String, String>,
}

impl FamilyMember {
    /// Unique memory scope key for this member.
    /// Format: `user:<name_lowercase_ascii>` (stable across channel changes).
    pub fn scope(&self) -> String {
        let slug: String = self
            .name
            .to_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        format!("user:{slug}")
    }
}

/// Registry of all family members, loaded from config.
#[derive(Debug, Clone)]
pub struct FamilyRegistry {
    /// Members list.
    members: Vec<FamilyMember>,
    /// Reverse index: `"telegram:12345678"` → index into `members`.
    index: HashMap<String, usize>,
    /// Maximum allowed members.
    max_members: usize,
}

impl FamilyRegistry {
    /// Create a new registry from config.
    ///
    /// Returns an error if `members.len() > max_members`.
    pub fn new(members: Vec<FamilyMember>, max_members: usize) -> anyhow::Result<Self> {
        if members.len() > max_members {
            anyhow::bail!(
                "Family has {} members but max is {}",
                members.len(),
                max_members
            );
        }

        let mut index = HashMap::new();
        for (i, member) in members.iter().enumerate() {
            for (channel, user_id) in &member.channels {
                let key = format!("{}:{}", channel.to_lowercase(), user_id);
                index.insert(key, i);
            }
        }

        Ok(Self {
            members,
            index,
            max_members,
        })
    }

    /// Create an empty (single-user, no family) registry.
    pub fn empty() -> Self {
        Self {
            members: Vec::new(),
            index: HashMap::new(),
            max_members: 8,
        }
    }

    /// Resolve a channel user to a family member.
    ///
    /// Returns `None` if the user is not registered.
    pub fn resolve(&self, channel: &str, user_id: &str) -> Option<&FamilyMember> {
        let key = format!("{}:{}", channel.to_lowercase(), user_id);
        self.index.get(&key).map(|&i| &self.members[i])
    }

    /// Check if a channel user is a registered family member.
    pub fn is_known(&self, channel: &str, user_id: &str) -> bool {
        self.resolve(channel, user_id).is_some()
    }

    /// Whether family mode is active (at least one member registered).
    pub fn is_active(&self) -> bool {
        !self.members.is_empty()
    }

    /// Number of registered members.
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Maximum allowed members.
    pub fn max_members(&self) -> usize {
        self.max_members
    }

    /// Get all members (for admin UI).
    pub fn members(&self) -> &[FamilyMember] {
        &self.members
    }
}

/// Shared memory scope constant.
pub const SCOPE_SHARED: &str = "shared";

#[cfg(test)]
mod tests {
    use super::*;

    fn test_family() -> FamilyRegistry {
        let members = vec![
            FamilyMember {
                name: "Benjamin".into(),
                role: UserRole::Root,
                channels: [
                    ("telegram".into(), "12345678".into()),
                    ("websocket".into(), "ben@local".into()),
                ]
                .into(),
            },
            FamilyMember {
                name: "Maria".into(),
                role: UserRole::Adult,
                channels: [("whatsapp".into(), "+491701234567".into())].into(),
            },
            FamilyMember {
                name: "Oma Helga".into(),
                role: UserRole::Senior,
                channels: [("imessage".into(), "+491607654321".into())].into(),
            },
            FamilyMember {
                name: "Luca".into(),
                role: UserRole::Child,
                channels: [("discord".into(), "987654321".into())].into(),
            },
        ];
        FamilyRegistry::new(members, 8).unwrap()
    }

    #[test]
    fn resolve_known_member() {
        let reg = test_family();
        let member = reg.resolve("telegram", "12345678").unwrap();
        assert_eq!(member.name, "Benjamin");
        assert_eq!(member.role, UserRole::Root);
    }

    #[test]
    fn resolve_case_insensitive_channel() {
        let reg = test_family();
        assert!(reg.resolve("Telegram", "12345678").is_some());
        assert!(reg.resolve("TELEGRAM", "12345678").is_some());
    }

    #[test]
    fn resolve_unknown_user_returns_none() {
        let reg = test_family();
        assert!(reg.resolve("telegram", "99999999").is_none());
    }

    #[test]
    fn resolve_unknown_channel_returns_none() {
        let reg = test_family();
        assert!(reg.resolve("signal", "12345678").is_none());
    }

    #[test]
    fn is_known_works() {
        let reg = test_family();
        assert!(reg.is_known("whatsapp", "+491701234567"));
        assert!(!reg.is_known("whatsapp", "+49000000000"));
    }

    #[test]
    fn member_scope_is_stable() {
        let member = FamilyMember {
            name: "Oma Helga".into(),
            role: UserRole::Senior,
            channels: HashMap::new(),
        };
        assert_eq!(member.scope(), "user:omahelga");
    }

    #[test]
    fn member_count() {
        let reg = test_family();
        assert_eq!(reg.member_count(), 4);
        assert!(reg.is_active());
    }

    #[test]
    fn empty_registry_is_inactive() {
        let reg = FamilyRegistry::empty();
        assert!(!reg.is_active());
        assert_eq!(reg.member_count(), 0);
    }

    #[test]
    fn max_members_enforced() {
        let members: Vec<FamilyMember> = (0..10)
            .map(|i| FamilyMember {
                name: format!("User{i}"),
                role: UserRole::Adult,
                channels: [(format!("ch{i}"), format!("id{i}"))].into(),
            })
            .collect();

        let result = FamilyRegistry::new(members, 8);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max is 8"));
    }

    #[test]
    fn multi_channel_same_member() {
        let reg = test_family();
        // Benjamin is on both telegram and websocket
        let via_telegram = reg.resolve("telegram", "12345678").unwrap();
        let via_ws = reg.resolve("websocket", "ben@local").unwrap();
        assert_eq!(via_telegram.name, via_ws.name);
        assert_eq!(via_telegram.scope(), via_ws.scope());
    }
}
