// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use x25519_dalek::{PublicKey, StaticSecret};
use rand::rngs::OsRng;
use base64::Engine;

#[derive(Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub name: String,
    pub public_key: String,
    pub allowed_ips: String,
    pub created_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VpnConfig {
    pub server_private_key: String,
    pub server_public_key: String,
    pub listen_port: u16,
    pub subnet: String, // e.g. "10.100.0.0/24"
    pub peers: Vec<Peer>,
}

pub struct VpnManager {
    config_path: PathBuf,
    wg_interface: String,
}

impl VpnManager {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            config_path: data_dir.join("vpn_config.json"),
            wg_interface: "wg0".to_string(),
        }
    }

    pub fn init(&self) -> anyhow::Result<()> {
        if !self.config_path.exists() {
            let (private_key, public_key) = Self::generate_keys();
            
            let config = VpnConfig {
                server_private_key: Self::encode_key(&private_key),
                server_public_key: Self::encode_key(&public_key),
                listen_port: 51820,
                subnet: "10.100.0.0/24".to_string(),
                peers: vec![],
            };
            self.save_config(&config)?;
        }
        
        // Ensure WG interface is up (Linux only)
        #[cfg(target_os = "linux")]
        self.apply_config()?;
        
        Ok(())
    }

    pub fn list_peers(&self) -> anyhow::Result<Vec<Peer>> {
        let config = self.load_config()?;
        Ok(config.peers)
    }

    /// Add a new peer and return the client configuration (for QR code)
    pub async fn add_peer(&self, name: &str) -> anyhow::Result<(Peer, String)> {
        let mut config = self.load_config()?;
        
        // Generate client keys
        let (client_priv, client_pub) = Self::generate_keys();
        let client_pub_key_str = Self::encode_key(&client_pub);
        let client_priv_key_str = Self::encode_key(&client_priv);
        let server_pub_key_str = config.server_public_key.clone();

        // Assign IP
        let next_ip = self.next_available_ip(&config)?;
        let client_ip = format!("{}/32", next_ip);

        let peer = Peer {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            public_key: client_pub_key_str.clone(),
            allowed_ips: client_ip.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        config.peers.push(peer.clone());
        self.save_config(&config)?;

        #[cfg(target_os = "linux")]
        self.apply_config()?;

        // Generate Client Config
        // Note: Endpoint should be the server's public IP/Domain.
        let endpoint = get_public_ip().await.unwrap_or_else(|_| "YOUR_SERVER_IP".to_string());
        
        let client_conf = format!(
            "[Interface]\nPrivateKey = {}\nAddress = {}\nDNS = 10.100.0.1\n\n[Peer]\nPublicKey = {}\nAllowedIPs = 0.0.0.0/0\nEndpoint = {}:{}\nPersistentKeepalive = 25\n",
            client_priv_key_str,
            client_ip,
            server_pub_key_str,
            endpoint,
            51820 // Default WG port
        );

        Ok((peer, client_conf))
    }

    pub fn delete_peer(&self, id: &str) -> anyhow::Result<()> {
        let mut config = self.load_config()?;
        config.peers.retain(|p| p.id != id);
        self.save_config(&config)?;

        #[cfg(target_os = "linux")]
        self.apply_config()?;
        
        Ok(())
    }

    // --- Helpers ---

    fn load_config(&self) -> anyhow::Result<VpnConfig> {
        let content = fs::read_to_string(&self.config_path)?;
        Ok(serde_json::from_str(&content)?)
    }

    fn save_config(&self, config: &VpnConfig) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    fn generate_keys() -> (StaticSecret, PublicKey) {
        let private = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&private);
        (private, public)
    }

    fn encode_key<K: AsRef<[u8]>>(key: K) -> String {
        base64::engine::general_purpose::STANDARD.encode(key)
    }

    fn next_available_ip(&self, config: &VpnConfig) -> anyhow::Result<String> {
        // Simple subnet logic: 10.100.0.x
        // Server uses .1
        let mut used = vec![1];
        for p in &config.peers {
            if let Some(last_octet) = p.allowed_ips.split('.').last() {
                if let Some(num_str) = last_octet.split('/').next() {
                     if let Ok(n) = num_str.parse::<u8>() {
                         used.push(n);
                     }
                }
            }
        }
        
        for i in 2..254 {
            if !used.contains(&i) {
                return Ok(format!("10.100.0.{}", i));
            }
        }
        
        bail!("Maximum peer limit reached (252 peers). Remove unused peers before adding new ones.")
    }

    #[cfg(target_os = "linux")]
    fn apply_config(&self) -> anyhow::Result<()> {
        use std::process::Command;
        let config = self.load_config()?;
        
        // 1. Generate server wg0.conf
        let mut wg_conf = format!(
            "[Interface]\nPrivateKey = {}\nAddress = 10.100.0.1/24\nListenPort = {}\nPostUp = iptables -A FORWARD -i {} -j ACCEPT; iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE\nPostDown = iptables -D FORWARD -i {} -j ACCEPT; iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE\n\n",
            config.server_private_key,
            config.listen_port,
            self.wg_interface,
            self.wg_interface
        );

        for peer in &config.peers {
            wg_conf.push_str(&format!(
                "[Peer]\nPublicKey = {}\nAllowedIPs = {}\n\n",
                peer.public_key, peer.allowed_ips
            ));
        }

        let conf_path = format!("/etc/wireguard/{}.conf", self.wg_interface);
        fs::write(&conf_path, wg_conf)?;

        // 2. Sync interface
        // This requires 'wg-quick' or 'wg' tools installed
        let _ = Command::new("wg-quick")
            .arg("strip")
            .arg(&self.wg_interface)
            .status(); // Check output?
            
        let output = Command::new("wg")
            .arg("syncconf")
            .arg(&self.wg_interface)
            .arg(format!("<(wg-quick strip {})", self.wg_interface))
            .output();

        // If sync failed, try restart
        if output.is_err() || !output.as_ref().unwrap().status.success() {
             let _ = Command::new("systemctl")
                .arg("restart")
                .arg(format!("wg-quick@{}", self.wg_interface))
                .status();
        }

        Ok(())
    }
}

async fn get_public_ip() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(3)).build()?;
    let ip = client.get("https://api.ipify.org").send().await?.text().await?;
    Ok(ip)
}
