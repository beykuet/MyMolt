use anyhow::{Context, Result};
use k256::ecdsa::{SigningKey, VerifyingKey, Signature, signature::Signer};
use rand::rngs::OsRng;
use std::path::Path;
use std::fs;

/// Represents the agent's cryptographic identity (private key).
pub struct AgentKey {
    signing_key: SigningKey,
}

impl AgentKey {
    /// Generate a new random secp256k1 key pair.
    pub fn generate() -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        Self { signing_key }
    }

    /// Load key from a raw binary file (32 bytes).
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let bytes = fs::read(path).context("Failed to read key file")?;
        let signing_key = SigningKey::from_bytes(bytes.as_slice().into())
            .context("Invalid key format")?;
        Ok(Self { signing_key })
    }

    /// Save private key to a raw binary file (32 bytes).
    /// WARNING: This should be stored in a secure location or encrypted.
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let bytes = self.signing_key.to_bytes();
        fs::write(path, bytes).context("Failed to write key file")?;
        // Set permissions to 600 (owner read/write only) on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)?;
        }
        Ok(())
    }

    /// Get the public key (VerifyingKey).
    pub fn public_key(&self) -> VerifyingKey {
        *self.signing_key.verifying_key()
    }

    /// Get the public key as a hexadecimal string (compressed format).
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_key().to_encoded_point(true).as_bytes())
    }

    /// Sign a message (byte slice).
    /// Returns the signature as bytes (ASN.1 DER or fixed size, k256 default is fixed).
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }
}
