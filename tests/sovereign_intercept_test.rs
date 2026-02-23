use anyhow::Result;
use std::sync::Arc;
use tempfile::TempDir;
    use mymolt_core::config::{AuditConfig, MemoryConfig};
    use mymolt_core::memory::{create_memory, MemoryCategory};
    use mymolt_core::security::{AuditLogger, VaultManager};

    #[tokio::test]
    async fn test_sovereign_interception() -> Result<()> {
        // 1. Setup
        let tmp = TempDir::new()?;

        // Provision Hoodik RSA keys for VaultManager
        let key_dir = tmp.path().join("hoodik/keys");
        std::fs::create_dir_all(&key_dir)?;
        let priv_key = cryptfns::rsa::private::generate().expect("generate RSA key");
        let pub_key = cryptfns::rsa::public::from_private(&priv_key).expect("derive public key");
        let priv_pem = cryptfns::rsa::private::to_string(&priv_key).expect("PEM encode private key");
        let pub_pem = cryptfns::rsa::public::to_string(&pub_key).expect("PEM encode public key");
        std::fs::write(key_dir.join("admin.key"), &priv_pem)?;
        std::fs::write(key_dir.join("admin.pub"), &pub_pem)?;

        let config = MemoryConfig {
            backend: "markdown".into(), // Use simple backend
            ..Default::default()
        };
        
        // Create AuditLogger for testing
        let audit = Arc::new(AuditLogger::new(
            AuditConfig::default(),
            tmp.path().to_path_buf(),
        )?);
        
        // Create memory (this will be SovereignMemory wrapping MarkdownMemory)
        let memory = create_memory(&config, tmp.path(), None, audit)?;
    
    // Ensure we are using the Sovereign wrapper
    assert_eq!(memory.name(), "sovereign");
    
    // 2. Store sensitive content
    let key = "test_secret";
    let sensitive_content = "Here is my secret API key: sk-12345678901234567890123456789012";
    
    memory.store(key, sensitive_content, MemoryCategory::Core).await?;
    
    // 3. Recall and Verify Interception
    // The stored content should be an Opaque Pointer, NOT the original secret
    let entries = memory.recall(key, 1).await?;
    assert!(!entries.is_empty());
    
    let stored_content = &entries[0].content;
    println!("Stored content: {}", stored_content);
    
    assert!(stored_content.contains("[VAULT: OpenAI Key - Access Required]"));
    assert!(!stored_content.contains("sk-12345"));
    
    // 4. Verify Vault Entry Exists
    let _vault = VaultManager::new(tmp.path());
    // We can't easily guess the UUID used, but strict checking would look in data/vault
    let vault_dir = tmp.path().join("data/vault");
    assert!(vault_dir.exists());
    let entries = std::fs::read_dir(vault_dir)?;
    let count = entries.count();
    assert_eq!(count, 1, "Should have exactly one vaulted file");

    Ok(())
}
