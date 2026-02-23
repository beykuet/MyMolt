// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use crate::memory::{Memory, MemoryCategory};
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use zeroize::Zeroize;

// ── Hoodik-native E2EE ──────────────────────────────────────────────────
// We use a hybrid encryption scheme:
//   1. Generate a random 256-bit symmetric key (ChaCha20-Poly1305).
//   2. Encrypt the plaintext with ChaCha20-Poly1305 (AEAD).
//   3. Encrypt the symmetric key with the Hoodik admin RSA public key.
//   4. Store:  base64(RSA-encrypted-sym-key) || "." || base64(nonce||ciphertext)
//
// This removes the external `age` CLI dependency and uses the same crypto
// stack as the rest of the Hoodik system.
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{AeadCore, ChaCha20Poly1305, Key, Nonce};

/// ChaCha20-Poly1305 nonce length (96 bits).
const NONCE_LEN: usize = 12;

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultMetadata {
    pub id: String,
    pub description: String,
    pub vault_path: PathBuf,
    pub created_at: String,
    pub tags: Vec<String>,
}

pub struct VaultManager {
    base_dir: PathBuf,
    vault_dir: PathBuf,
    meta_dir: PathBuf,
    journal_dir: PathBuf,
}

impl VaultManager {
    pub fn new(root: &Path) -> Self {
        let base = root.to_path_buf();
        Self {
            vault_dir: base.join("data/vault"),
            meta_dir: base.join("data/meta"),
            journal_dir: base.join("data/journal"),
            base_dir: base,
        }
    }

    // ── Key helpers ──────────────────────────────────────────────────

    /// Load the Hoodik admin RSA public key PEM.
    fn load_admin_pubkey(&self) -> Result<String> {
        let path = self.base_dir.join("hoodik/keys/admin.pub");
        fs::read_to_string(&path)
            .with_context(|| format!("Cannot read Hoodik admin public key at {}", path.display()))
    }

    /// Load the Hoodik admin RSA private key PEM.
    fn load_admin_privkey(&self) -> Result<String> {
        let path = self.base_dir.join("hoodik/keys/admin.key");
        fs::read_to_string(&path)
            .with_context(|| format!("Cannot read Hoodik admin private key at {}", path.display()))
    }

    // ── Hybrid encrypt / decrypt ────────────────────────────────────

    /// Hybrid-encrypt: ChaCha20 for data, RSA for key wrapping.
    ///
    /// Output format (UTF-8): `<base64(rsa-encrypted-sym-key)>.<base64(nonce||aead-ciphertext)>`
    fn hybrid_encrypt(&self, plaintext: &[u8]) -> Result<String> {
        let pubkey_pem = self.load_admin_pubkey()?;

        // 1. Random 256-bit symmetric key
        let sym_key = ChaCha20Poly1305::generate_key(&mut OsRng);
        let cipher = ChaCha20Poly1305::new(&sym_key);
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        // 2. AEAD encrypt
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("ChaCha20 encryption failed: {e}"))?;

        // nonce || ciphertext
        let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        blob.extend_from_slice(&nonce);
        blob.extend_from_slice(&ciphertext);
        let data_b64 = cryptfns::base64::encode(&blob);

        // 3. RSA-wrap the symmetric key
        let wrapped = cryptfns::rsa::public::encrypt(
            &cryptfns::base64::encode(sym_key.as_slice()),
            &pubkey_pem,
        )
        .map_err(|e| anyhow::anyhow!("RSA key-wrap failed: {e}"))?;

        Ok(format!("{wrapped}.{data_b64}"))
    }

    /// Hybrid-decrypt: RSA-unwrap the symmetric key, then ChaCha20 decrypt.
    fn hybrid_decrypt(&self, envelope: &str) -> Result<String> {
        let privkey_pem = self.load_admin_privkey()?;

        let (wrapped_key_b64, data_b64) = envelope
            .split_once('.')
            .context("Invalid vault envelope (missing '.' separator)")?;

        // 1. RSA-unwrap
        let sym_key_b64 = cryptfns::rsa::private::decrypt(wrapped_key_b64, &privkey_pem)
            .map_err(|e| anyhow::anyhow!("RSA key-unwrap failed: {e}"))?;

        let sym_key_bytes = cryptfns::base64::decode(&sym_key_b64)
            .map_err(|e| anyhow::anyhow!("Sym-key decode failed: {e}"))?;

        anyhow::ensure!(
            sym_key_bytes.len() == 32,
            "Unwrapped symmetric key has wrong length ({})",
            sym_key_bytes.len()
        );

        let key = Key::from_slice(&sym_key_bytes);
        let cipher = ChaCha20Poly1305::new(key);

        // 2. Decode data blob
        let blob = cryptfns::base64::decode(&data_b64)
            .map_err(|e| anyhow::anyhow!("Data blob decode failed: {e}"))?;

        anyhow::ensure!(
            blob.len() > NONCE_LEN,
            "Encrypted data too short (missing nonce)"
        );

        let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);

        // 3. AEAD decrypt
        let plaintext_bytes = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| anyhow::anyhow!("Decryption failed — wrong key or tampered data"))?;

        String::from_utf8(plaintext_bytes).context("Decrypted vault data is not valid UTF-8")
    }

    // ── Public API ──────────────────────────────────────────────────

    /// Encrypt sensitive data using Hoodik-native hybrid E2EE and create metadata.
    pub async fn encrypt_to_vault<M: Memory + ?Sized>(
        &self,
        memory: &M,
        mut plaintext: String,
        description: &str,
        _recipient: &str, // kept for API compat; we always use the admin key
    ) -> Result<PathBuf> {
        let id = uuid::Uuid::new_v4().to_string();
        let vault_filename = format!("{}.vault", id);
        let vault_path = self.vault_dir.join(&vault_filename);
        let meta_path = self.meta_dir.join(format!("{}.json", id));

        // Ensure directories exist
        fs::create_dir_all(&self.vault_dir)?;
        fs::create_dir_all(&self.meta_dir)?;

        // 1. Hybrid encrypt (ChaCha20 + RSA key-wrap)
        let encrypted = self.hybrid_encrypt(plaintext.as_bytes())?;
        fs::write(&vault_path, &encrypted)?;

        // 2. Wipe plaintext from RAM
        plaintext.zeroize();

        // 3. Create metadata
        let metadata = VaultMetadata {
            id: id.clone(),
            description: description.to_string(),
            vault_path: vault_path.clone(),
            created_at: Utc::now().to_rfc3339(),
            tags: vec!["vault".to_string()],
        };

        let meta_content = serde_json::to_string_pretty(&metadata)?;
        fs::write(&meta_path, meta_content)?;

        // Index ONLY metadata in the memory brain
        let index_key = format!("vault:{}", id);
        memory
            .store(
                &index_key,
                description,
                MemoryCategory::Custom("vault".to_string()),
            )
            .await?;

        // 4. Git Commit
        self.commit_to_git(description)?;

        Ok(meta_path)
    }

    /// Decrypt data from vault using Hoodik-native hybrid E2EE.
    pub fn decrypt_from_vault(&self, id: &str) -> Result<String> {
        let vault_path = self.vault_dir.join(format!("{}.vault", id));
        if !vault_path.exists() {
            anyhow::bail!("Vault entry {} not found", id);
        }

        let envelope = fs::read_to_string(&vault_path)?;
        self.hybrid_decrypt(&envelope)
    }

    fn commit_to_git(&self, message: &str) -> Result<()> {
        // Basic git integration
        Command::new("git")
            .arg("-C")
            .arg(&self.base_dir)
            .arg("add")
            .arg("data/")
            .output()?;

        Command::new("git")
            .arg("-C")
            .arg(&self.base_dir)
            .arg("commit")
            .arg("-m")
            .arg(format!("Secure entry: {}", message))
            .output()?;

        Ok(())
    }

    pub fn list_entries(&self) -> Result<Vec<VaultMetadata>> {
        let mut entries = Vec::new();
        if !self.meta_dir.exists() {
            return Ok(entries);
        }

        for entry in fs::read_dir(&self.meta_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(meta) = serde_json::from_str::<VaultMetadata>(&content) {
                        entries.push(meta);
                    }
                }
            }
        }

        // Sort by date desc
        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper: generate Hoodik RSA keys in the expected directory structure.
    fn setup_hoodik_keys(root: &Path) {
        let key_dir = root.join("hoodik/keys");
        std::fs::create_dir_all(&key_dir).unwrap();

        let priv_key = cryptfns::rsa::private::generate().unwrap();
        let pub_key = cryptfns::rsa::public::from_private(&priv_key).unwrap();

        let priv_pem = cryptfns::rsa::private::to_string(&priv_key).unwrap();
        let pub_pem = cryptfns::rsa::public::to_string(&pub_key).unwrap();

        std::fs::write(key_dir.join("admin.key"), priv_pem).unwrap();
        std::fs::write(key_dir.join("admin.pub"), pub_pem).unwrap();
    }

    #[test]
    fn hybrid_encrypt_decrypt_roundtrip() {
        let tmp = TempDir::new().unwrap();
        setup_hoodik_keys(tmp.path());

        let vault = VaultManager::new(tmp.path());
        let secret = "super-secret-api-key-12345";

        let encrypted = vault.hybrid_encrypt(secret.as_bytes()).unwrap();
        assert!(encrypted.contains('.'), "envelope must contain separator");
        assert_ne!(encrypted, secret);

        let decrypted = vault.hybrid_decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, secret);
    }

    #[test]
    fn hybrid_tampered_data_rejected() {
        let tmp = TempDir::new().unwrap();
        setup_hoodik_keys(tmp.path());

        let vault = VaultManager::new(tmp.path());
        let encrypted = vault.hybrid_encrypt(b"sensitive-data").unwrap();

        // Flip a character in the data portion
        let (key_part, data_part) = encrypted.split_once('.').unwrap();
        let mut data_bytes = data_part.as_bytes().to_vec();
        if data_bytes.len() > 5 {
            data_bytes[5] ^= 0xff;
        }
        let tampered = format!("{}.{}", key_part, String::from_utf8_lossy(&data_bytes));

        let result = vault.hybrid_decrypt(&tampered);
        assert!(result.is_err(), "Tampered data must be rejected");
    }

    #[test]
    fn missing_keys_gives_clear_error() {
        let tmp = TempDir::new().unwrap();
        // Don't create keys
        let vault = VaultManager::new(tmp.path());

        let result = vault.hybrid_encrypt(b"test");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("admin public key"),
            "Error should mention missing key: {err}"
        );
    }

    #[test]
    fn decrypt_from_vault_file_roundtrip() {
        let tmp = TempDir::new().unwrap();
        setup_hoodik_keys(tmp.path());

        let vault = VaultManager::new(tmp.path());
        let id = "test-entry-001";
        let secret = "my-vault-secret";

        // Write directly to vault location
        fs::create_dir_all(tmp.path().join("data/vault")).unwrap();
        let encrypted = vault.hybrid_encrypt(secret.as_bytes()).unwrap();
        fs::write(
            tmp.path().join(format!("data/vault/{}.vault", id)),
            &encrypted,
        )
        .unwrap();

        let decrypted = vault.decrypt_from_vault(id).unwrap();
        assert_eq!(decrypted, secret);
    }

    #[test]
    fn list_entries_empty_when_no_meta() {
        let tmp = TempDir::new().unwrap();
        let vault = VaultManager::new(tmp.path());
        let entries = vault.list_entries().unwrap();
        assert!(entries.is_empty());
    }
}
