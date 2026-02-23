# Remaining Hardening Plan

> **Status as of 2026-02-20 16:55 CET**
> **All 7 of 7 issues RESOLVED ✅**

---

## Overview

| # | Area | Priority | Status |
|---|------|----------|--------|
| 1 | SecurityWrapper scope | 🔴 High | ✅ **DONE** — All tool categories now trust-gated |
| 2 | TrustLevel granularity | 🟡 Medium | ✅ **DONE** — Medium tier added to both crates |
| 3 | PIM concurrency | 🟢 Low | ✅ **DONE** — RwLock replaces per-tool Mutex |
| 4 | Scanner performance | 🟢 Low | ✅ **DONE** — Aho-Corasick pre-filter added |

---

## Item 1: SecurityWrapper Scope Expansion

### Problem

`SecurityWrapper.execute()` only trust-gates `"delegate"` and `"shell"` by name (line 45-48 in `tools/security.rs`). Other sensitive tools pass through unchecked.

### Plan

**Step 1.1** — Expand the `match` in `SecurityWrapper.execute()` to cover all sensitive tools:

```rust
// tools/security.rs — line 44-49, replace the match block:
let trust_check = match name {
    "delegate"                              => self.security.check_trust(self.security.required_trust_for_delegation),
    "shell"                                 => self.security.check_trust(self.security.required_trust_for_shell),
    "http_request" | "browser" | "browser_open" => self.security.check_trust(self.security.required_trust_for_mcp),
    n if n.starts_with("mcp:")              => self.security.check_trust(self.security.required_trust_for_mcp),
    n if n.starts_with("calendar_") 
       || n.starts_with("contacts_") 
       || n.starts_with("notes_")           => self.security.check_trust(self.security.required_trust_for_vault),
    _ => Ok(()),
};
```

**Rationale:**

- `http_request`, `browser`, `browser_open` can exfiltrate data → gate at MCP trust level
- MCP tools (prefixed `mcp:`) → already gated by SigilGatekeeper, but defense-in-depth adds a second check
- PIM tools handle contacts/calendar → gate at vault trust level (contains PII)
- `file_read`, `file_write`, `git_operations` are already path-gated by SecurityPolicy, so `Ok(())` is fine

**Step 1.2** — Add tests to verify each category is gated:

```rust
// tools/security.rs — add to existing tests module
#[tokio::test]
async fn trust_gate_blocks_http_for_low_trust() { ... }

#[tokio::test]
async fn trust_gate_blocks_pim_for_low_trust() { ... }

#[tokio::test]
async fn trust_gate_allows_file_read_for_any_trust() { ... }
```

**Files to modify:**

- `src/tools/security.rs` (expand match + add 3 tests)

**Verification:** `cargo test --lib` — expect 1461+ tests passing

---

## Item 2: TrustLevel Granularity

### Problem

`TrustLevel` has only 2 values: `Low = 1` and `High = 3`. The gap between 1 and 3 is intentional (leaves room), but there's no `Medium` tier. The role system (Root/Adult/Senior/Child) compensates at the UX level, but the protocol layer can't express "verified email but not government ID."

### Plan

**Step 2.1** — Add `Medium = 2` to the `TrustLevel` enum in both `sigil-rs` and `mymolt`:

```rust
// sigil-rs/src/identity.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    Low = 1,      // Anonymous / unverified
    Medium = 2,   // Verified email / OIDC (Google, Apple, etc.)
    High = 3,     // Government ID (eIDAS, passport)
}

// mymolt/src/identity/soul.rs — mirror the same
```

**Step 2.2** — Update `sigil_bridge.rs` to map the new variant:

```rust
// mymolt/src/security/sigil_bridge.rs
crate::identity::soul::TrustLevel::Medium => sigil::TrustLevel::Medium,
```

**Step 2.3** — Update `TrustConfig::parse_level()` to handle `"medium"`:

```rust
// config/schema.rs
pub fn parse_level(s: &str) -> TrustLevel {
    match s.to_lowercase().as_str() {
        "high" | "3" => TrustLevel::High,
        "medium" | "2" => TrustLevel::Medium,
        _ => TrustLevel::Low,
    }
}
```

**Step 2.4** — Update role resolution to use Medium:

```rust
// identity/roles.rs — resolve_role()
TrustLevel::Medium => {
    if config.is_local { Role::Adult } else { Role::Adult }
}
```

**Step 2.5** — Update existing tests and add new Medium-tier tests:

- `soul_implements_sigil_identity_provider` — add Medium case
- `resolve_role` — add Medium trust test
- `check_trust` — verify Medium >= Low, Medium < High

**Files to modify:**

- `sigil-rs/src/identity.rs` (add variant)
- `mymolt/src/identity/soul.rs` (add variant)
- `mymolt/src/security/sigil_bridge.rs` (add mapping)
- `mymolt/src/config/schema.rs` (update parser)
- `mymolt/src/identity/roles.rs` (update resolution)
- `mymolt/src/security/policy.rs` (check_trust already works with numeric ordering)

**Verification:** `cargo test` in both crates — all existing + 4 new tests

---

## Item 3: PIM Concurrency

### Problem

Each PIM tool holds a `Mutex<PathBuf>` for the workspace path. All tools share the same file (`pim.json`), so concurrent operations from different channels (e.g., Telegram adding a contact while Gateway lists calendar) will serialize at the file level. Under single-user load this is fine, but multi-channel concurrent access could queue.

### Plan

**Step 3.1** — Replace per-tool `Arc<Mutex<PathBuf>>` with a shared `Arc<RwLock<PimStore>>`:

```rust
// tools/pim.rs — new shared state
pub struct PimState {
    store: tokio::sync::RwLock<PimStore>,
    workspace: PathBuf,
    secrets: Option<SecretStore>,
}

impl PimState {
    pub fn new(workspace: PathBuf, secrets: Option<SecretStore>) -> Self {
        let store = load_store(&workspace, &secrets);
        Self {
            store: tokio::sync::RwLock::new(store),
            workspace,
            secrets,
        }
    }

    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, PimStore> {
        self.store.read().await
    }

    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, PimStore> {
        self.store.write().await
    }

    pub async fn flush(&self) -> anyhow::Result<()> {
        let store = self.store.read().await;
        save_store(&self.workspace, &store, &self.secrets)
    }
}
```

**Step 3.2** — Refactor all 7 tools to take `Arc<PimState>` instead of individual `Arc<Mutex<PathBuf>>` + `Option<SecretStore>`:

- Read-only tools (`calendar_list`, `contacts_search`, `notes_search`, `notes_read`) use `state.read().await`
- Write tools (`calendar_add`, `contacts_add`, `notes_create`) use `state.write().await` + `state.flush().await`

**Benefits:**

- Multiple readers can proceed concurrently (e.g., search contacts + list calendar)
- Writers still serialize (correct behavior for a single JSON file)
- In-memory cache avoids repeated disk reads
- Single flush point reduces I/O

**Step 3.3** — Update `pim_tools()` factory:

```rust
pub fn pim_tools(workspace: &Path, secrets: Option<SecretStore>) -> Vec<Box<dyn Tool>> {
    let state = Arc::new(PimState::new(workspace.to_path_buf(), secrets));
    vec![
        Box::new(CalendarAddTool { state: state.clone() }),
        // ...
    ]
}
```

**Files to modify:**

- `src/tools/pim.rs` (refactor all 7 tools + factory)

**Verification:** All existing PIM tests still pass + add 1 concurrent access test

---

## Item 4: Scanner Performance (Aho-Corasick)

### Problem

`SensitivityScanner::scan()` runs 8 regexes sequentially. Each regex does a full text scan. For most inputs this is fast (<1ms), but for large delegated responses (100KB+) or batch scanning it's suboptimal.

### Plan

**Step 4.1** — Add Aho-Corasick pre-filter for fixed-prefix patterns:

4 of the 8 patterns have fixed prefixes that can be checked with Aho-Corasick in a single pass:

- `sk-` (OpenAI)
- `AIza` (Google)
- `AKIA` (AWS)
- `-----BEGIN` (Private Key)

```rust
// memory/sovereign.rs
use aho_corasick::AhoCorasick;

pub struct SensitivityScanner {
    // Fast pre-filter: if none of these prefixes are found, skip regexes entirely
    prefix_filter: AhoCorasick,
    // Full regex patterns for precise matching
    patterns: Vec<(String, Regex)>,
    // Indices in `patterns` that correspond to non-prefix patterns (IBAN, CC, PIN)
    always_check: Vec<usize>,
}
```

**Step 4.2** — Implement two-phase scanning:

```rust
pub fn scan(&self, text: &str) -> Option<String> {
    // Phase 1: Aho-Corasick pre-filter (single pass over text)
    let has_prefix = self.prefix_filter.is_match(text);
    
    // Phase 2: Only run prefix-based regexes if prefix was found
    for (i, (name, re)) in self.patterns.iter().enumerate() {
        let should_check = self.always_check.contains(&i) || has_prefix;
        if should_check && re.is_match(text) {
            return Some(name.clone());
        }
    }
    None
}
```

**Step 4.3** — Add `aho-corasick` dependency:

```toml
# mymolt/Cargo.toml
aho-corasick = "1"
```

**Step 4.4** — Add benchmark test:

```rust
#[test]
fn scanner_performance_clean_text() {
    let scanner = SensitivityScanner::new();
    let clean = "a".repeat(100_000);
    let start = std::time::Instant::now();
    for _ in 0..100 {
        scanner.scan(&clean);
    }
    let elapsed = start.elapsed();
    // 100 scans of 100KB should complete in < 100ms
    assert!(elapsed.as_millis() < 100, "Scanner too slow: {elapsed:?}");
}
```

**Files to modify:**

- `Cargo.toml` (add `aho-corasick`)
- `src/memory/sovereign.rs` (refactor SensitivityScanner)

**Verification:** All existing scanner tests pass + benchmark test

---

## Execution Order

```
Phase 1 (Security — do first):
  └─ Item 1: SecurityWrapper scope    [~30 min, high impact]

Phase 2 (Protocol — builds on Phase 1):
  └─ Item 2: TrustLevel granularity   [~45 min, medium impact]

Phase 3 (Performance — independent):
  ├─ Item 3: PIM concurrency          [~20 min, low impact]
  └─ Item 4: Scanner performance      [~30 min, low impact]
```

**Total estimated effort:** ~2 hours
**Expected test count after:** ~1475+ (17+ new tests)

---

## Risk Assessment

| Item | Risk of Regression | Mitigation |
|------|--------------------|-----------|
| 1. SecurityWrapper | Low — additive match arms | Existing tests cover the bypass path |
| 2. TrustLevel | Medium — enum change | Full exhaustive matching will surface all callsites |
| 3. PIM concurrency | Low — internal refactor | Same API surface, same tests |
| 4. Scanner perf | Low — optimization only | Pre-existing regex tests validate correctness |
