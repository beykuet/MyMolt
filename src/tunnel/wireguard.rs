// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use super::Tunnel;
use anyhow::{Context, Result};
use tokio::process::Command;

pub struct WireGuardTunnel {
    config_path: String,
    interface: String,
}

impl WireGuardTunnel {
    pub fn new(config_path: String, interface: Option<String>) -> Self {
        let interface = interface.unwrap_or_else(|| {
            std::path::Path::new(&config_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("wg0")
                .to_string()
        });
        Self {
            config_path,
            interface,
        }
    }
}

#[async_trait::async_trait]
impl Tunnel for WireGuardTunnel {
    fn name(&self) -> &str {
        "wireguard"
    }

    async fn start(&self, _local_host: &str, _local_port: u16) -> Result<String> {
        // WireGuard usually requires root/sudo. 
        // We assume the user has set up sudoers or is running as root/capability.
        // Or we use `wg-quick up`.
        
        // 1. Check if already up
        let status = Command::new("ip")
            .args(&["link", "show", &self.interface])
            .output()
            .await;

        if status.is_ok() && status.unwrap().status.success() {
            // Already up
        } else {
            // Start it
            let output = Command::new("wg-quick")
                .arg("up")
                .arg(&self.config_path)
                .output()
                .await
                .context("Failed to execute wg-quick")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("wg-quick up failed: {}", stderr);
            }
        }

        // Return the IP address of the interface
        // We parse `ip -j addr show <interface>` or just return a generic info.
        // For MyMolt, we might just return "wireguard:<interface>" as the URL.
        Ok(format!("wireguard://{}", self.interface))
    }

    async fn stop(&self) -> Result<()> {
         // Stop: wg-quick down
         let output = Command::new("wg-quick")
            .arg("down")
            .arg(&self.config_path)
            .output()
            .await
            .context("Failed to execute wg-quick")?;
            
         if !output.status.success() {
             // If interfaces is already gone, it might error, but we can ignore or log.
             // Usually we want to be clean.
         }
         Ok(())
    }

    async fn health_check(&self) -> bool {
        // Check interface existence
        let status = Command::new("ip")
            .args(&["link", "show", &self.interface])
            .output()
            .await;
        
        match status {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    fn public_url(&self) -> Option<String> {
        // WireGuard doesn't have a single "Public URL" like ngrok.
        // But we can return the interface IP if we knew it.
        // For now return None or the interface schema.
        Some(format!("wireguard://{}", self.interface))
    }
}
