// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

pub mod schema;

#[allow(unused_imports)]
pub use schema::{
    AuditConfig, AutonomyConfig, BrowserConfig, ChannelsConfig, ComposioConfig, Config,
    DelegateAgentConfig, DiscordConfig, DockerRuntimeConfig, FamilyConfig, FamilyMemberConfig,
    GatewayConfig, HeartbeatConfig, HttpRequestConfig, IMessageConfig, IdentityConfig, LarkConfig,
    MatrixConfig, McpConfig, McpServerConfig, MemoryConfig, ModelRouteConfig, ObservabilityConfig,
    ReliabilityConfig, ResourceLimitsConfig, RuntimeConfig, SandboxBackend, SandboxConfig,
    SecretsConfig, SecurityConfig, SlackConfig, SttConfig, TelegramConfig, TrustConfig,
    TunnelConfig, WebhookConfig,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reexported_config_default_is_constructible() {
        let config = Config::default();

        assert!(config.default_provider.is_some());
        assert!(config.default_model.is_some());
        assert!(config.default_temperature > 0.0);
    }

    #[test]
    fn reexported_channel_configs_are_constructible() {
        let telegram = TelegramConfig {
            bot_token: "token".into(),
            allowed_users: vec!["alice".into()],
        };

        let discord = DiscordConfig {
            bot_token: "token".into(),
            guild_id: Some("123".into()),
            allowed_users: vec![],
            listen_to_bots: false,
        };

        let lark = LarkConfig {
            app_id: "app-id".into(),
            app_secret: "app-secret".into(),
            encrypt_key: None,
            verification_token: None,
            allowed_users: vec![],
            use_feishu: false,
        };

        assert_eq!(telegram.allowed_users.len(), 1);
        assert_eq!(discord.guild_id.as_deref(), Some("123"));
        assert_eq!(lark.app_id, "app-id");
    }
}
