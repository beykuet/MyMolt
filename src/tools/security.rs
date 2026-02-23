// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use crate::security::confirmation::ConfirmationGate;
use crate::security::SecurityPolicy;
use crate::tools::{Tool, ToolResult, ToolSpec};
use async_trait::async_trait;
use std::sync::Arc;

/// Wraps a tool to enforce security policies, including:
/// - Skill allowlist
/// - SIGIL trust gating
/// - User confirmation for high-risk actions
pub struct SecurityWrapper {
    inner: Box<dyn Tool>,
    security: Arc<SecurityPolicy>,
    /// Optional confirmation gate. If `None`, tools requiring
    /// confirmation are blocked (defensive default).
    confirm_gate: Option<Arc<ConfirmationGate>>,
}

impl SecurityWrapper {
    pub fn new(inner: Box<dyn Tool>, security: Arc<SecurityPolicy>) -> Self {
        Self {
            inner,
            security,
            confirm_gate: None,
        }
    }

    /// Attach a confirmation gate for interactive confirmation flow.
    pub fn with_confirmation(mut self, gate: Arc<ConfirmationGate>) -> Self {
        self.confirm_gate = Some(gate);
        self
    }
}

#[async_trait]
impl Tool for SecurityWrapper {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn parameters_schema(&self) -> serde_json::Value {
        self.inner.parameters_schema()
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let name = self.name();
        
        // 1. Check if skill is allowed
        if !self.security.is_skill_allowed(name) {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Skill '{}' is disabled by security policy.", name)),
            });
        }

        // 2. SIGIL: Check trust level for sensitive operations
        let trust_check = match name {
            // Agent delegation — high sensitivity
            "delegate" => self.security.check_trust(self.security.required_trust_for_delegation),
            // Shell execution — configurable (default: Low)
            "shell" => self.security.check_trust(self.security.required_trust_for_shell),
            // Network tools — can exfiltrate data
            "http_request" | "browser" | "browser_open" => {
                self.security.check_trust(self.security.required_trust_for_mcp)
            }
            // PIM tools — contain PII (contacts, calendar, notes)
            n if n.starts_with("calendar_")
                || n.starts_with("contacts_")
                || n.starts_with("notes_") =>
            {
                self.security.check_trust(self.security.required_trust_for_vault)
            }
            // MCP tools — defense-in-depth (also gated by SigilGatekeeper)
            n if n.starts_with("mcp:") => {
                self.security.check_trust(self.security.required_trust_for_mcp)
            }
            // File, Git, Memory, Screenshot, etc. — gated by path policy
            _ => Ok(()),
        };
        if let Err(reason) = trust_check {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "SIGIL trust gate: '{}' blocked — {}",
                    name, reason
                )),
            });
        }

        // 3. Check if action requires user confirmation
        if self.security.requires_confirmation(name, "execute") {
            match &self.confirm_gate {
                Some(gate) => {
                    // Build human-readable summary of what the tool will do
                    let summary = format!(
                        "Tool '{}' wants to execute with args: {}",
                        name,
                        serde_json::to_string(&args)
                            .unwrap_or_else(|_| "<unparseable>".into())
                            .chars()
                            .take(200)
                            .collect::<String>()
                    );
                    let approved = gate.request(name, &summary).await;
                    if !approved {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(format!(
                                "User denied confirmation for '{}' (or request timed out).",
                                name
                            )),
                        });
                    }
                    // Approved — fall through to execute
                }
                None => {
                    // No gate attached — block defensively (same as old behavior)
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!(
                            "Skill '{}' requires user confirmation but no confirmation channel is available.",
                            name
                        )),
                    });
                }
            }
        }

        self.inner.execute(args).await
    }
    
    fn spec(&self) -> ToolSpec {
        self.inner.spec()
    }
}
