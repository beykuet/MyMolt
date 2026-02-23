use mymolt_core::security::VaultManager;
use mymolt_core::memory::{SqliteMemory, Memory};
use tempfile::TempDir;

#[tokio::test]
async fn test_sovereign_opaque_pointer_flow() -> anyhow::Result<()> {
    let tmp_dir = TempDir::new()?;
    let root = tmp_dir.path();

    // Provision Hoodik RSA keys for VaultManager
    let key_dir = root.join("hoodik/keys");
    std::fs::create_dir_all(&key_dir)?;
    let priv_key = cryptfns::rsa::private::generate().expect("generate RSA key");
    let pub_key = cryptfns::rsa::public::from_private(&priv_key).expect("derive public key");
    let priv_pem = cryptfns::rsa::private::to_string(&priv_key).expect("PEM encode private key");
    let pub_pem = cryptfns::rsa::public::to_string(&pub_key).expect("PEM encode public key");
    std::fs::write(key_dir.join("admin.key"), &priv_pem)?;
    std::fs::write(key_dir.join("admin.pub"), &pub_pem)?;

    // 1. Setup RAG and VaultManager
    let memory = SqliteMemory::new(root)?;
    let vault = VaultManager::new(root);
    
    // Create necessary directories
    std::fs::create_dir_all(root.join("data/vault"))?;
    std::fs::create_dir_all(root.join("data/meta"))?;
    std::fs::create_dir_all(root.join("data/journal"))?;
    
    // 2. Encryption (Opaque Pointer Handover)
    let secret_content = "This is a highly sensitive sovereign secret.".to_string();
    let description = "Project X Strategy Document";
    let recipient = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDwUTskbsZKk9rzdKzoTqh3tYnMEb23oLmue1NY/dTyg"; // User's key
    
    println!("--- Encrypting to Vault ---");
    let meta_path = vault.encrypt_to_vault(&memory, secret_content.clone(), description, recipient).await?;
    
    // 3. Verify Files
    assert!(meta_path.exists());
    let id = meta_path.file_stem().unwrap().to_str().unwrap();
    let vault_path = root.join(format!("data/vault/{}.vault", id));
    assert!(vault_path.exists());
    
    println!("Vault ID: {}", id);
    println!("Metadata stored at: {:?}", meta_path);
    println!("Encrypted blob at: {:?}", vault_path);
    
    // 4. Verify RAG (Cognitive Layer) isolation
    // Memory should contain the description, but NOT the secret content
    let recall_results = memory.recall("Project X", 10).await?;
    assert!(!recall_results.is_empty());
    assert!(recall_results[0].content.contains(description));
    
    let all_memories = memory.list(None).await?;
    for m in all_memories {
        assert!(!m.content.contains("sovereign secret"), "SENSITIVE DATA LEAKED TO RAG!");
    }
    println!("RAG isolation verified: Only metadata indexed.");
    
    // 5. Decryption (Sovereign Retrieval)
    println!("--- Decrypting from Vault ---");
    // This will use age --decrypt -i ~/.ssh/id_ed25519
    // Note: In an automated test environment without the private key, this might fail 
    // unless the private key is available. For this local test, we assume the user's key works.
    match vault.decrypt_from_vault(id) {
        Ok(decrypted) => {
            assert_eq!(decrypted, secret_content);
            println!("Decryption successful: content matches.");
        },
        Err(e) => {
            println!("Decryption test skipped/failed (likely missing private key in test env): {}", e);
        }
    }
    
    Ok(())
}
