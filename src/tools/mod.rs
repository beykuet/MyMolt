// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

pub mod browser;
pub mod browser_open;
pub mod composio;
pub mod delegate;
pub mod file_read;
pub mod file_write;
pub mod git_operations;
pub mod http_request;
pub mod image_info;
pub mod memory_forget;
pub mod memory_recall;
pub mod memory_store;
pub mod pim;
pub mod screenshot;
pub mod security;
pub mod shell;
pub mod traits;

pub use browser::BrowserTool;
pub use browser_open::BrowserOpenTool;
pub use composio::ComposioTool;
pub use delegate::DelegateTool;
pub use file_read::FileReadTool;
pub use file_write::FileWriteTool;
pub use git_operations::GitOperationsTool;
pub use http_request::HttpRequestTool;
pub use image_info::ImageInfoTool;
pub use memory_forget::MemoryForgetTool;
pub use memory_recall::MemoryRecallTool;
pub use memory_store::MemoryStoreTool;
pub use screenshot::ScreenshotTool;
pub use security::SecurityWrapper;
pub use shell::ShellTool;
pub use traits::Tool;
#[allow(unused_imports)]
pub use traits::{ToolResult, ToolSpec};

use crate::config::DelegateAgentConfig;
use crate::memory::sovereign::SensitivityScanner;
use crate::memory::Memory;
use crate::runtime::{NativeRuntime, RuntimeAdapter};
use crate::security::{AuditLogger, SecurityPolicy};
use std::collections::HashMap;
use std::sync::Arc;

/// Create the default tool registry
pub fn default_tools(security: Arc<SecurityPolicy>) -> Vec<Box<dyn Tool>> {
    default_tools_with_runtime(security, Arc::new(NativeRuntime::new()))
}

/// Create the default tool registry with explicit runtime adapter.
pub fn default_tools_with_runtime(
    security: Arc<SecurityPolicy>,
    runtime: Arc<dyn RuntimeAdapter>,
) -> Vec<Box<dyn Tool>> {
    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(ShellTool::new(security.clone(), runtime)),
        Box::new(FileReadTool::new(security.clone())),
        Box::new(FileWriteTool::new(security.clone())),
    ];

    tools
        .into_iter()
        .map(|t| Box::new(SecurityWrapper::new(t, security.clone())) as Box<dyn Tool>)
        .collect()
}

/// Create full tool registry including memory tools and optional Composio
#[allow(clippy::implicit_hasher)]
pub fn all_tools(
    security: &Arc<SecurityPolicy>,
    memory: Arc<dyn Memory>,
    composio_key: Option<&str>,
    browser_config: &crate::config::BrowserConfig,
    http_config: &crate::config::HttpRequestConfig,
    workspace_dir: &std::path::Path,
    agents: &HashMap<String, DelegateAgentConfig>,
    fallback_api_key: Option<&str>,
    extra_tools: Vec<Box<dyn Tool>>,
    audit: Option<Arc<AuditLogger>>,
    actor_name: Option<String>,
) -> Vec<Box<dyn Tool>> {
    all_tools_with_runtime(
        security,
        Arc::new(NativeRuntime::new()),
        memory,
        composio_key,
        browser_config,
        http_config,
        workspace_dir,
        agents,
        fallback_api_key,
        extra_tools,
        audit,
        actor_name,
    )
}

/// Create full tool registry including memory tools and optional Composio.
#[allow(clippy::implicit_hasher)]
pub fn all_tools_with_runtime(
    security: &Arc<SecurityPolicy>,
    runtime: Arc<dyn RuntimeAdapter>,
    memory: Arc<dyn Memory>,
    composio_key: Option<&str>,
    browser_config: &crate::config::BrowserConfig,
    http_config: &crate::config::HttpRequestConfig,
    workspace_dir: &std::path::Path,
    agents: &HashMap<String, DelegateAgentConfig>,
    fallback_api_key: Option<&str>,
    extra_tools: Vec<Box<dyn Tool>>,
    audit: Option<Arc<AuditLogger>>,
    actor_name: Option<String>,
) -> Vec<Box<dyn Tool>> {
    let mut tools: Vec<Box<dyn Tool>> = vec![
        Box::new(ShellTool::new(security.clone(), runtime)),
        Box::new(FileReadTool::new(security.clone())),
        Box::new(FileWriteTool::new(security.clone())),
        Box::new(MemoryStoreTool::new(memory.clone())),
        Box::new(MemoryRecallTool::new(memory.clone())),
        Box::new(MemoryForgetTool::new(memory)),
        Box::new(GitOperationsTool::new(
            security.clone(),
            workspace_dir.to_path_buf(),
        )),
    ];

    if browser_config.enabled {
        // Add legacy browser_open tool for simple URL opening
        tools.push(Box::new(BrowserOpenTool::new(
            security.clone(),
            browser_config.allowed_domains.clone(),
        )));
        // Add full browser automation tool (pluggable backend)
        tools.push(Box::new(BrowserTool::new_with_backend(
            security.clone(),
            browser_config.allowed_domains.clone(),
            browser_config.session_name.clone(),
            browser_config.backend.clone(),
            browser_config.native_headless,
            browser_config.native_webdriver_url.clone(),
            browser_config.native_chrome_path.clone(),
        )));
    }

    if http_config.enabled {
        tools.push(Box::new(HttpRequestTool::new(
            security.clone(),
            http_config.allowed_domains.clone(),
            http_config.max_response_size,
            http_config.timeout_secs,
        )));
    }

    // Vision tools are always available
    tools.push(Box::new(ScreenshotTool::new(security.clone())));
    tools.push(Box::new(ImageInfoTool::new(security.clone())));

    if let Some(key) = composio_key {
        if !key.is_empty() {
            tools.push(Box::new(ComposioTool::new(key)));
        }
    }

    // Add delegation tool when agents are configured
    if !agents.is_empty() {
        tools.push(Box::new(
            DelegateTool::new(
                agents.clone(),
                fallback_api_key.map(String::from),
                Arc::new(SensitivityScanner::new()),
                audit,
            )
            .with_actor(actor_name),
        ));
    }

    // Add PIM tools (calendar, contacts, notes) — encrypted at rest
    let pim_secrets = {
        let mymolt_dir = workspace_dir.join(".mymolt");
        Some(crate::security::secrets::SecretStore::new(
            &mymolt_dir,
            true,
        ))
    };
    tools.extend(pim::pim_tools(workspace_dir, pim_secrets));

    // Add MCP tools (already gated by SigilGatekeeper, no extra SecurityWrapper needed)
    let mcp_count = extra_tools.len();
    let mut wrapped: Vec<Box<dyn Tool>> = tools
        .into_iter()
        .map(|t| Box::new(SecurityWrapper::new(t, security.clone())) as Box<dyn Tool>)
        .collect();
    wrapped.extend(extra_tools);

    if mcp_count > 0 {
        tracing::info!(count = mcp_count, "MCP tools added to registry");
    }

    wrapped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BrowserConfig, MemoryConfig};
    use tempfile::TempDir;

    #[test]
    fn default_tools_has_three() {
        let security = Arc::new(SecurityPolicy::default());
        let tools = default_tools(security);
        assert_eq!(tools.len(), 3);
    }

    #[test]
    fn all_tools_excludes_browser_when_disabled() {
        let tmp = TempDir::new().unwrap();
        let security = Arc::new(SecurityPolicy::default());
        let mem_cfg = MemoryConfig {
            backend: "markdown".into(),
            ..MemoryConfig::default()
        };
        let audit = Arc::new(
            crate::security::AuditLogger::new(
                crate::config::AuditConfig::default(),
                tmp.path().to_path_buf(),
            )
            .unwrap(),
        );
        let mem: Arc<dyn Memory> =
            Arc::from(crate::memory::create_memory(&mem_cfg, tmp.path(), None, audit).unwrap());

        let browser = BrowserConfig {
            enabled: false,
            allowed_domains: vec!["example.com".into()],
            session_name: None,
            ..BrowserConfig::default()
        };
        let http = crate::config::HttpRequestConfig::default();

        let tools = all_tools(
            &security,
            mem,
            None,
            &browser,
            &http,
            tmp.path(),
            &HashMap::new(),
            None,
            Vec::new(),
            None,
            None,
        );
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        assert!(!names.contains(&"browser_open"));
    }

    #[test]
    fn all_tools_includes_browser_when_enabled() {
        let tmp = TempDir::new().unwrap();
        let security = Arc::new(SecurityPolicy::default());
        let mem_cfg = MemoryConfig {
            backend: "markdown".into(),
            ..MemoryConfig::default()
        };
        let audit = Arc::new(
            crate::security::AuditLogger::new(
                crate::config::AuditConfig::default(),
                tmp.path().to_path_buf(),
            )
            .unwrap(),
        );
        let mem: Arc<dyn Memory> =
            Arc::from(crate::memory::create_memory(&mem_cfg, tmp.path(), None, audit).unwrap());

        let browser = BrowserConfig {
            enabled: true,
            allowed_domains: vec!["example.com".into()],
            session_name: None,
            ..BrowserConfig::default()
        };
        let http = crate::config::HttpRequestConfig::default();

        let tools = all_tools(
            &security,
            mem,
            None,
            &browser,
            &http,
            tmp.path(),
            &HashMap::new(),
            None,
            Vec::new(),
            None,
            None,
        );
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        assert!(names.contains(&"browser_open"));
    }

    #[test]
    fn default_tools_names() {
        let security = Arc::new(SecurityPolicy::default());
        let tools = default_tools(security);
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        assert!(names.contains(&"shell"));
        assert!(names.contains(&"file_read"));
        assert!(names.contains(&"file_write"));
    }

    #[test]
    fn default_tools_all_have_descriptions() {
        let security = Arc::new(SecurityPolicy::default());
        let tools = default_tools(security);
        for tool in &tools {
            assert!(
                !tool.description().is_empty(),
                "Tool {} has empty description",
                tool.name()
            );
        }
    }

    #[test]
    fn default_tools_all_have_schemas() {
        let security = Arc::new(SecurityPolicy::default());
        let tools = default_tools(security);
        for tool in &tools {
            let schema = tool.parameters_schema();
            assert!(
                schema.is_object(),
                "Tool {} schema is not an object",
                tool.name()
            );
            assert!(
                schema["properties"].is_object(),
                "Tool {} schema has no properties",
                tool.name()
            );
        }
    }

    #[test]
    fn tool_spec_generation() {
        let security = Arc::new(SecurityPolicy::default());
        let tools = default_tools(security);
        for tool in &tools {
            let spec = tool.spec();
            assert_eq!(spec.name, tool.name());
            assert_eq!(spec.description, tool.description());
            assert!(spec.parameters.is_object());
        }
    }

    #[test]
    fn tool_result_serde() {
        let result = ToolResult {
            success: true,
            output: "hello".into(),
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: ToolResult = serde_json::from_str(&json).unwrap();
        assert!(parsed.success);
        assert_eq!(parsed.output, "hello");
        assert!(parsed.error.is_none());
    }

    #[test]
    fn tool_result_with_error_serde() {
        let result = ToolResult {
            success: false,
            output: String::new(),
            error: Some("boom".into()),
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: ToolResult = serde_json::from_str(&json).unwrap();
        assert!(!parsed.success);
        assert_eq!(parsed.error.as_deref(), Some("boom"));
    }

    #[test]
    fn tool_spec_serde() {
        let spec = ToolSpec {
            name: "test".into(),
            description: "A test tool".into(),
            parameters: serde_json::json!({"type": "object"}),
        };
        let json = serde_json::to_string(&spec).unwrap();
        let parsed: ToolSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.description, "A test tool");
    }

    #[test]
    fn all_tools_includes_delegate_when_agents_configured() {
        let tmp = TempDir::new().unwrap();
        let security = Arc::new(SecurityPolicy::default());
        let mem_cfg = MemoryConfig {
            backend: "markdown".into(),
            ..MemoryConfig::default()
        };
        let audit = Arc::new(
            crate::security::AuditLogger::new(
                crate::config::AuditConfig::default(),
                tmp.path().to_path_buf(),
            )
            .unwrap(),
        );
        let mem: Arc<dyn Memory> =
            Arc::from(crate::memory::create_memory(&mem_cfg, tmp.path(), None, audit).unwrap());

        let browser = BrowserConfig::default();
        let http = crate::config::HttpRequestConfig::default();

        let mut agents = HashMap::new();
        agents.insert(
            "researcher".to_string(),
            DelegateAgentConfig {
                provider: "ollama".to_string(),
                model: "llama3".to_string(),
                system_prompt: None,
                api_key: None,
                temperature: None,
                max_depth: 3,
            },
        );

        let tools = all_tools(
            &security,
            mem,
            None,
            &browser,
            &http,
            tmp.path(),
            &agents,
            Some("sk-test"),
            Vec::new(),
            None,
            None,
        );
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        assert!(names.contains(&"delegate"));
    }

    #[test]
    fn all_tools_excludes_delegate_when_no_agents() {
        let tmp = TempDir::new().unwrap();
        let security = Arc::new(SecurityPolicy::default());
        let mem_cfg = MemoryConfig {
            backend: "markdown".into(),
            ..MemoryConfig::default()
        };
        let audit = Arc::new(
            crate::security::AuditLogger::new(
                crate::config::AuditConfig::default(),
                tmp.path().to_path_buf(),
            )
            .unwrap(),
        );
        let mem: Arc<dyn Memory> =
            Arc::from(crate::memory::create_memory(&mem_cfg, tmp.path(), None, audit).unwrap());

        let browser = BrowserConfig::default();
        let http = crate::config::HttpRequestConfig::default();

        let tools = all_tools(
            &security,
            mem,
            None,
            &browser,
            &http,
            tmp.path(),
            &HashMap::new(),
            None,
            Vec::new(),
            None,
            None,
        );
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        assert!(!names.contains(&"delegate"));
    }
}
