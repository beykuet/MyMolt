// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! MCP (Model Context Protocol) client integration.
//!
//! Connects MyMolt to MCP servers, discovers their tools, and exposes them
//! to the agent loop — all gated through SIGIL security.

pub mod bridge;
pub mod gatekeeper;
pub mod transport;

pub use bridge::McpToolBridge;
pub use gatekeeper::SigilGatekeeper;
pub use transport::McpClient;

use crate::config::McpConfig;
use crate::security::{AuditLogger, SecurityPolicy};
use crate::tools::Tool;
use std::sync::Arc;

/// Connect to all configured MCP servers and discover their tools.
///
/// Each tool is wrapped through the SIGIL gatekeeper for policy enforcement
/// and audit logging. Returns an empty vec if no MCP servers are configured.
pub async fn discover_mcp_tools(
    mcp_config: &McpConfig,
    security: &Arc<SecurityPolicy>,
    audit: &Arc<AuditLogger>,
) -> Vec<Box<dyn Tool>> {
    if !mcp_config.enabled || mcp_config.servers.is_empty() {
        return Vec::new();
    }

    let mut mcp_tools: Vec<Box<dyn Tool>> = Vec::new();

    for server_cfg in &mcp_config.servers {
        tracing::info!(
            server = %server_cfg.name,
            command = %server_cfg.command,
            "Connecting to MCP server"
        );

        match McpClient::connect(
            &server_cfg.name,
            &server_cfg.command,
            &server_cfg.args,
            &server_cfg.env,
        )
        .await
        {
            Ok(client) => {
                let client = Arc::new(client);
                let gatekeeper = Arc::new(SigilGatekeeper::new(
                    security.clone(),
                    Some(audit.clone()),
                ));

                match McpToolBridge::discover_tools(client, gatekeeper).await {
                    Ok(discovered) => {
                        let count = discovered.len();
                        tracing::info!(
                            server = %server_cfg.name,
                            count,
                            "MCP tools discovered"
                        );
                        mcp_tools.extend(discovered);
                    }
                    Err(e) => {
                        tracing::warn!(
                            server = %server_cfg.name,
                            error = %e,
                            "Failed to discover MCP tools"
                        );
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    server = %server_cfg.name,
                    error = %e,
                    "Failed to connect to MCP server"
                );
            }
        }
    }

    mcp_tools
}
