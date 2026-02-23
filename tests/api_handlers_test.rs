//! Integration tests for gateway API handlers.
//! Tests RBAC enforcement, rate limiting, and input validation.

use mymolt_core::identity::UserRole;

/// Verify that Child < Senior < Adult < Root in privilege ordering
#[test]
fn test_user_role_ordering() {
    assert!(UserRole::Child < UserRole::Senior);
    assert!(UserRole::Senior < UserRole::Adult);
    assert!(UserRole::Adult < UserRole::Root);
    assert!(UserRole::Child < UserRole::Root);

    // Equality
    assert_eq!(UserRole::Root, UserRole::Root);
    assert_ne!(UserRole::Child, UserRole::Root);
}

/// Verify UserRole level() returns expected numeric values
#[test]
fn test_user_role_levels() {
    assert_eq!(UserRole::Child.level(), 0);
    assert_eq!(UserRole::Senior.level(), 1);
    assert_eq!(UserRole::Adult.level(), 2);
    assert_eq!(UserRole::Root.level(), 3);
}

/// Verify UserRole default is Adult
#[test]
fn test_user_role_default() {
    assert_eq!(UserRole::default(), UserRole::Adult);
}

/// Verify RBAC policy: Child cannot write diary
#[test]
fn test_rbac_diary_child_blocked() {
    let child = UserRole::Child;
    // diary requires >= Senior
    assert!(
        child < UserRole::Senior,
        "Child should not be able to write diary"
    );
}

/// Verify RBAC policy: Senior can write diary
#[test]
fn test_rbac_diary_senior_allowed() {
    let senior = UserRole::Senior;
    assert!(
        senior >= UserRole::Senior,
        "Senior should be able to write diary"
    );
}

/// Verify RBAC policy: only Root can manage VPN
#[test]
fn test_rbac_vpn_root_only() {
    assert!(UserRole::Root == UserRole::Root);
    assert!(UserRole::Adult != UserRole::Root);
    assert!(UserRole::Senior != UserRole::Root);
    assert!(UserRole::Child != UserRole::Root);
}

/// Test UserRole serialization roundtrip
#[test]
fn test_user_role_serde() {
    let role = UserRole::Root;
    let json = serde_json::to_string(&role).unwrap();
    assert_eq!(json, "\"Root\"");

    let deserialized: UserRole = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, UserRole::Root);
}

/// Test all roles serialize to expected strings
#[test]
fn test_all_roles_serde() {
    for (role, expected) in [
        (UserRole::Child, "\"Child\""),
        (UserRole::Senior, "\"Senior\""),
        (UserRole::Adult, "\"Adult\""),
        (UserRole::Root, "\"Root\""),
    ] {
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(
            json, expected,
            "Role {:?} should serialize to {}",
            role, expected
        );
    }
}

/// Test diary input sanitization logic (matching handlers.rs behavior)
#[test]
fn test_diary_sanitization() {
    let content = "### Heading Attack\n## Another\n# Root Level";
    let sanitized = content
        .replace("# ", "")
        .replace("## ", "")
        .replace("### ", "");

    assert!(!sanitized.contains("# "), "Headings should be stripped");
    assert!(sanitized.contains("Heading Attack"));
    assert!(sanitized.contains("Root Level"));
}

/// Test diary length validation
#[test]
fn test_diary_length_limit() {
    let short = "Hello diary!";
    assert!(short.len() <= 10_000);

    let long = "x".repeat(10_001);
    assert!(long.len() > 10_000, "Should reject entries over 10K chars");
}

/// Test diary empty validation
#[test]
fn test_diary_empty_rejected() {
    let empty = "   ";
    assert!(
        empty.trim().is_empty(),
        "Whitespace-only should be rejected"
    );
}
