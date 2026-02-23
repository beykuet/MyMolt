// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

use crate::config::Config as MyMoltConfig;
use config::Config as HoodikConfig;
use context::Context as HoodikContext;
use tokio::task::JoinHandle;
use anyhow::Result;

pub fn spawn_server(mymolt_config: &MyMoltConfig) -> JoinHandle<()> {
    let workspace_dir = mymolt_config.workspace_dir.clone();
    
    // Create hoodik runtime directory inside mymolt workspace
    let hoodik_base = workspace_dir.join("hoodik");
    let db_path = hoodik_base.join("hoodik.db");
    
    // Run Hoodik in a dedicated thread with its own Actix Runtime
    // Actix-web 4.x HttpServer is !Send, so it cannot be spawned on a tokio::task
    std::thread::spawn(move || {
        println!("🚀 Spawning Hoodik thread...");
        let rt = actix_rt::System::new();
        
        rt.block_on(async move {
            println!("🚀 Inside Hoodik Actix Runtime...");
            // Ensure directory exists
            if let Err(e) = tokio::fs::create_dir_all(&hoodik_base).await {
                tracing::error!("Failed to create Hoodik directory: {}", e);
                return;
            }

            // Programmatic Hoodik Configuration via Environment Variables
            std::env::set_var("HTTP_ADDRESS", "127.0.0.1");
            std::env::set_var("HTTP_PORT", "3001");
            std::env::set_var("DATABASE_URL", format!("sqlite://{}?mode=rwc", db_path.display()));
            std::env::set_var("DATA_DIR", hoodik_base.join("storage").to_string_lossy().to_string());
            
            // Disable SSL for internal loopback
            std::env::set_var("SSL_DISABLED", "true");

            tracing::info!("🛡️ Starting Embedded Hoodik Server at http://127.0.0.1:3001");
            tracing::info!("   Storage Root: {}", hoodik_base.display());

            // Initialize Config (reads env vars we just set, ignores CLI args)
            let mut config = HoodikConfig::env_only("MyMolt-Hoodik", "0.1.0", "Embedded Hoodik for MyMolt");
            
            let storage_path = hoodik_base.join("storage");
            if let Err(e) = tokio::fs::create_dir_all(&storage_path).await {
                 tracing::error!("Failed to create Hoodik storage directory: {}", e);
                 return;
            }
            // Ensure config knows directory is ready
            config.app.ensure_data_dir(Some(storage_path.to_string_lossy().to_string()));

            // Initialize Context
            match HoodikContext::new(config).await {
                Ok(ctx) => {
                    // 1. Run Migrations
                    tracing::info!("Running Hoodik migrations...");
                    use migration::MigratorTrait;
                    if let Err(e) = migration::Migrator::up(&ctx.db, None).await {
                        tracing::error!("Failed to run Hoodik migrations: {}", e);
                        return;
                    }

                    // 2. Ensure Admin User
                    if let Err(e) = ensure_admin_user(&ctx, &hoodik_base).await {
                        tracing::error!("Failed to ensure admin user: {}", e);
                    }

                    // 3. Bridge: Bind Hoodik admin identity to MyMolt Soul
                    {
                        use crate::identity::{Soul, soul::TrustLevel};
                        // workspace_dir is hoodik_base's parent
                        let workspace_dir = hoodik_base.parent().unwrap_or(&hoodik_base);
                        let mut soul = Soul::new(workspace_dir);
                        if let Err(e) = soul.load() {
                            tracing::debug!("Soul not loaded (first boot?): {}", e);
                        }
                        if let Err(e) = soul.add_binding("hoodik", "admin@mymolt.local", TrustLevel::High) {
                            tracing::warn!("Failed to bind Hoodik admin to Soul: {}", e);
                        } else {
                            tracing::info!("🔗 Soul ↔ Hoodik admin identity bridged");
                        }
                    }

                    // 4. Start Server
                    tracing::info!("Engaging Hoodik server...");
                    if let Err(e) = hoodik::server::engage(ctx).await {
                        tracing::error!("Hoodik server crashed: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to initialize Hoodik context: {}", e);
                }
            }
        });
    });

    // Return a dummy handle since we are using std::thread for now
    tokio::spawn(async {}) 
}

async fn ensure_admin_user(ctx: &HoodikContext, hoodik_base: &std::path::Path) -> Result<()> {
    use entity::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, ActiveValue::Set};
    use entity::users;
    use cryptfns::rsa::{private, public, fingerprint};

    // 1. Determine admin email
    let email = "admin@mymolt.local";

    // 2. Check if user exists
    let exists = users::Entity::find()
        .filter(users::Column::Email.eq(email))
        .one(&ctx.db)
        .await
        .map_err(|e| anyhow::anyhow!("DB error: {}", e))?;

    if exists.is_some() {
        return Ok(());
    }

    tracing::info!("👤 Creating Hoodik admin user: {}", email);

    // 3. Generate Keys
    let priv_key = private::generate().map_err(|e| anyhow::anyhow!("Keygen error: {}", e))?;
    let pub_key = public::from_private(&priv_key).map_err(|e| anyhow::anyhow!("Pubkey error: {}", e))?;
    let finger = fingerprint(pub_key.clone()).map_err(|e| anyhow::anyhow!("Fingerprint error: {}", e))?;
    
    let priv_pem = private::to_string(&priv_key).map_err(|e| anyhow::anyhow!("PEM error: {}", e))?;
    let pub_pem = public::to_string(&pub_key).map_err(|e| anyhow::anyhow!("PEM error: {}", e))?;

    // 4. Save keys to workspace for the user/agent to use
    let key_dir = hoodik_base.join("keys");
    tokio::fs::create_dir_all(&key_dir).await?;
    tokio::fs::write(key_dir.join("admin.key"), &priv_pem).await?;
    tokio::fs::write(key_dir.join("admin.pub"), &pub_pem).await?;
    
    tracing::info!("🔑 Saved admin keys to {}", key_dir.display());

    // 5. Create User — generate a cryptographically random password and persist
    //    it encrypted via SecretStore so subsequent boots can read it.
    let password = generate_admin_password(hoodik_base)?;
    let password_hash = util::password::hash(&password);

    let user = users::ActiveModel {
        id: Set(entity::Uuid::new_v4()),
        email: Set(email.to_string()),
        password: Set(Some(password_hash)),
        pubkey: Set(pub_pem),
        fingerprint: Set(finger),
        role: Set(Some("admin".to_string())),
        created_at: Set(chrono::Utc::now().timestamp()),
        updated_at: Set(chrono::Utc::now().timestamp()),
        ..Default::default()
    };

    user.insert(&ctx.db)
        .await
        .map_err(|e| anyhow::anyhow!("Insert error: {}", e))?;
        
    tracing::info!("✅ Admin user created successfully.");

    Ok(())
}

/// Generate (or load) a cryptographically random admin password.
///
/// On first call the password is generated and persisted encrypted via
/// `SecretStore` at `{hoodik_base}/admin_password.enc`. Subsequent calls
/// read and decrypt the stored password.
fn generate_admin_password(hoodik_base: &std::path::Path) -> Result<String> {
    use crate::security::SecretStore;
    use rand::Rng;

    let pw_path = hoodik_base.join("admin_password.enc");

    let store = SecretStore::new(hoodik_base, true);

    if pw_path.exists() {
        let encrypted = std::fs::read_to_string(&pw_path)
            .map_err(|e| anyhow::anyhow!("Cannot read admin password file: {e}"))?;
        return store
            .decrypt(encrypted.trim())
            .map_err(|e| anyhow::anyhow!("Cannot decrypt admin password: {e}"));
    }

    // Generate 32 random alphanumeric characters
    let password: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let encrypted = store
        .encrypt(&password)
        .map_err(|e| anyhow::anyhow!("Cannot encrypt admin password: {e}"))?;

    std::fs::create_dir_all(hoodik_base)?;
    std::fs::write(&pw_path, &encrypted)?;

    tracing::info!("🔒 Admin password generated and encrypted at {}", pw_path.display());

    Ok(password)
}
