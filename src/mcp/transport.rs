// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! MCP transport layer — lightweight JSON-RPC client over stdio.
//!
//! Implements the MCP client protocol using `serde_json` + `tokio::process`.
//! No external MCP SDK needed — the protocol is simple JSON-RPC 2.0 over stdin/stdout.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

// ── JSON-RPC types ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: Option<u64>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    #[allow(dead_code)]
    code: i64,
    message: String,
}

// ── MCP model types ─────────────────────────────────────────────

/// An MCP tool definition returned by `tools/list`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpToolInfo {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub input_schema: Value,
}

/// The result of calling an MCP tool.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpCallToolResult {
    #[serde(default)]
    pub content: Vec<McpContent>,
    #[serde(default)]
    pub is_error: Option<bool>,
}

/// A content block in an MCP tool result.
#[derive(Debug, Deserialize)]
pub struct McpContent {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(default)]
    pub text: Option<String>,
}

// ── MCP Client ──────────────────────────────────────────────────

/// A connected MCP client that communicates with an MCP server over stdio.
pub struct McpClient {
    child: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    stdout: Arc<Mutex<BufReader<tokio::process::ChildStdout>>>,
    next_id: AtomicU64,
    server_name: String,
}

impl McpClient {
    /// Spawn an MCP server as a child process and connect via stdio.
    pub async fn connect(
        name: &str,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<Self> {
        let mut cmd = Command::new(command);
        for arg in args {
            cmd.arg(arg);
        }
        for (k, v) in env {
            cmd.env(k, v);
        }
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null());

        let mut child = cmd
            .spawn()
            .context(format!("Failed to spawn MCP server '{name}'"))?;

        let stdin = child
            .stdin
            .take()
            .context("Failed to capture MCP server stdin")?;
        let stdout = child
            .stdout
            .take()
            .context("Failed to capture MCP server stdout")?;

        let client = Self {
            child: Arc::new(Mutex::new(child)),
            stdin: Arc::new(Mutex::new(stdin)),
            stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
            next_id: AtomicU64::new(1),
            server_name: name.to_string(),
        };

        // Send initialize request
        let init_result = client
            .request(
                "initialize",
                Some(serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "mymolt",
                        "version": "0.1.0"
                    }
                })),
            )
            .await
            .context("MCP initialize handshake failed")?;

        tracing::info!(
            server = name,
            info = %init_result,
            "Connected to MCP server"
        );

        // Send initialized notification
        client.notify("notifications/initialized", None).await?;

        Ok(client)
    }

    /// Send a JSON-RPC request and wait for the response.
    async fn request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method: method.to_string(),
            params,
        };

        let mut payload = serde_json::to_string(&request)?;
        payload.push('\n');

        // Write request
        {
            let mut stdin = self.stdin.lock().await;
            stdin
                .write_all(payload.as_bytes())
                .await
                .context("Failed to write to MCP server stdin")?;
            stdin.flush().await?;
        }

        // Read response
        let mut line = String::new();
        {
            let mut stdout = self.stdout.lock().await;
            loop {
                line.clear();
                let n = stdout
                    .read_line(&mut line)
                    .await
                    .context("Failed to read from MCP server stdout")?;
                if n == 0 {
                    anyhow::bail!("MCP server closed stdout unexpectedly");
                }
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                // Try to parse as a JSON-RPC response
                if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(trimmed) {
                    if let Some(err) = resp.error {
                        anyhow::bail!("MCP error: {}", err.message);
                    }
                    return Ok(resp.result.unwrap_or(Value::Null));
                }
                // If it's a notification or other message, skip it
            }
        }
    }

    /// Send a JSON-RPC notification (no response expected).
    async fn notify(&self, method: &str, params: Option<Value>) -> Result<()> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params.unwrap_or(Value::Null)
        });

        let mut payload = serde_json::to_string(&notification)?;
        payload.push('\n');

        let mut stdin = self.stdin.lock().await;
        stdin.write_all(payload.as_bytes()).await?;
        stdin.flush().await?;
        Ok(())
    }

    /// Discover available tools from the MCP server.
    pub async fn list_tools(&self) -> Result<Vec<McpToolInfo>> {
        let result = self.request("tools/list", None).await?;
        let tools: Vec<McpToolInfo> = serde_json::from_value(
            result
                .get("tools")
                .cloned()
                .unwrap_or(Value::Array(vec![])),
        )
        .context("Failed to parse tools/list response")?;
        Ok(tools)
    }

    /// Call a tool on the MCP server.
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<McpCallToolResult> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments.unwrap_or_default()
        });
        let result = self.request("tools/call", Some(params)).await?;
        let call_result: McpCallToolResult =
            serde_json::from_value(result).context("Failed to parse tools/call response")?;
        Ok(call_result)
    }

    /// Get the server name.
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    /// Gracefully shut down the MCP server connection.
    pub async fn shutdown(self) -> Result<()> {
        let mut child = self.child.lock().await;
        let _ = child.kill().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_client_types_exist() {
        // Smoke test — ensure the types compile
        let _name: fn(&McpClient) -> &str = McpClient::server_name;
    }

    #[test]
    fn mcp_tool_info_deserializes() {
        let json = r#"{"name": "test_tool", "description": "A test", "inputSchema": {"type": "object"}}"#;
        let tool: McpToolInfo = serde_json::from_str(json).unwrap();
        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description.as_deref(), Some("A test"));
    }

    #[test]
    fn mcp_call_result_deserializes() {
        let json =
            r#"{"content": [{"type": "text", "text": "hello"}], "isError": false}"#;
        let result: McpCallToolResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.content.len(), 1);
        assert_eq!(result.content[0].text.as_deref(), Some("hello"));
        assert_eq!(result.is_error, Some(false));
    }
}
