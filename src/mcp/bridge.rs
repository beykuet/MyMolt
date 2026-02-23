// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! MCP → MyMolt Tool bridge.
//!
//! Discovers tools from connected MCP servers and wraps each as a MyMolt `Tool`,
//! so they appear alongside native tools in the agent loop.

use crate::mcp::gatekeeper::SigilGatekeeper;
use crate::mcp::transport::McpClient;
use crate::tools::traits::{Tool, ToolResult};
use async_trait::async_trait;
use std::sync::Arc;

/// An MCP tool exposed as a MyMolt `Tool`.
///
/// Wraps a single tool from an MCP server, forwarding calls through
/// the SIGIL gatekeeper.
pub struct McpTool {
    tool_name: String,
    description: String,
    schema: serde_json::Value,
    client: Arc<McpClient>,
    gatekeeper: Arc<SigilGatekeeper>,
    server_name: String,
}

impl McpTool {
    /// Create a new MCP tool wrapper.
    pub fn new(
        tool_name: String,
        description: String,
        schema: serde_json::Value,
        client: Arc<McpClient>,
        gatekeeper: Arc<SigilGatekeeper>,
        server_name: String,
    ) -> Self {
        Self {
            tool_name,
            description,
            schema,
            client,
            gatekeeper,
            server_name,
        }
    }
}

#[async_trait]
impl Tool for McpTool {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters_schema(&self) -> serde_json::Value {
        self.schema.clone()
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // 1. SIGIL gate check
        if let Err(reason) = self.gatekeeper.gate_request(&self.tool_name) {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("SIGIL: {reason}")),
            });
        }

        // 2. Forward to MCP server
        let arguments = args.as_object().cloned();
        match self.client.call_tool(&self.tool_name, arguments).await {
            Ok(result) => {
                // Extract text content from MCP response
                let output: String = result
                    .content
                    .iter()
                    .filter_map(|c| c.text.as_deref())
                    .collect::<Vec<_>>()
                    .join("\n");

                let is_error = result.is_error.unwrap_or(false);

                Ok(ToolResult {
                    success: !is_error,
                    output: output.clone(),
                    error: if is_error { Some(output) } else { None },
                })
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "MCP server '{}' error: {e}",
                    self.server_name
                )),
            }),
        }
    }
}

/// Bridge that connects to MCP servers and produces MyMolt `Tool` instances.
pub struct McpToolBridge;

impl McpToolBridge {
    /// Discover tools from an MCP client and wrap them as MyMolt `Tool` instances.
    pub async fn discover_tools(
        client: Arc<McpClient>,
        gatekeeper: Arc<SigilGatekeeper>,
    ) -> anyhow::Result<Vec<Box<dyn Tool>>> {
        let tools_info = client.list_tools().await?;
        let server_name = client.server_name().to_string();
        let mut tools: Vec<Box<dyn Tool>> = Vec::new();

        for tool_info in tools_info {
            tracing::info!(
                server = server_name,
                tool = tool_info.name,
                "Discovered MCP tool"
            );

            tools.push(Box::new(McpTool::new(
                tool_info.name,
                tool_info.description.unwrap_or_default(),
                tool_info.input_schema,
                client.clone(),
                gatekeeper.clone(),
                server_name.clone(),
            )));
        }

        tracing::info!(
            server = server_name,
            count = tools.len(),
            "MCP tool discovery complete"
        );

        Ok(tools)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_tool_implements_tool_trait() {
        // Compile-time check that McpTool implements Tool
        fn _assert_tool<T: Tool>() {}
        _assert_tool::<McpTool>();
    }
}
