//! MyMolt daemon lifecycle management.
//!
//! On desktop: spawns the `mymolt` binary as a child process.
//! On mobile: the daemon runs on a remote server (LAN or cloud).
//!
//! The Tauri app connects to the daemon via HTTP/WebSocket.

use tauri::{AppHandle, Manager, Runtime};
use std::time::Duration;

/// Start the MyMolt daemon as a background child process.
///
/// Looks for the `mymolt` binary in:
/// 1. Same directory as the Tauri app
/// 2. PATH
/// 3. ~/mymolt/mymolt (default install location)
pub async fn start_daemon<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<crate::DaemonState>();
    let port = state.port;

    // Find the mymolt binary
    let binary = find_mymolt_binary(app);

    match &binary {
        Some(path) => {
            log::info!("Found MyMolt binary at: {}", path.display());

            // Spawn the daemon
            match tokio::process::Command::new(path)
                .arg("daemon")
                .arg("--port")
                .arg(port.to_string())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
            {
                Ok(child) => {
                    log::info!("MyMolt daemon started on port {}", port);
                    *state.process.lock().await = Some(child);

                    // Wait for readiness
                    wait_for_ready(port).await;
                    state.ready.store(true, std::sync::atomic::Ordering::Relaxed);
                    log::info!("MyMolt daemon is ready");
                }
                Err(e) => {
                    log::error!("Failed to start MyMolt daemon: {}", e);
                    // Fall through — user can still connect to a running daemon manually
                }
            }
        }
        None => {
            log::warn!("MyMolt binary not found — checking if daemon is already running on port {}...", port);

            // Maybe the daemon is already running (dev mode)
            if check_health(port).await {
                state.ready.store(true, std::sync::atomic::Ordering::Relaxed);
                log::info!("Connected to existing MyMolt daemon on port {}", port);
            } else {
                log::error!(
                    "No MyMolt daemon found. Install mymolt or start it manually: mymolt daemon --port {}",
                    port
                );
            }
        }
    }
}

/// Find the `mymolt` binary on the system.
fn find_mymolt_binary<R: Runtime>(app: &AppHandle<R>) -> Option<std::path::PathBuf> {
    // 1. Bundled alongside the Tauri app
    if let Ok(exe_dir) = app.path().resource_dir() {
        let bundled = exe_dir.join("mymolt");
        if bundled.exists() {
            return Some(bundled);
        }
        // Windows
        let bundled_exe = exe_dir.join("mymolt.exe");
        if bundled_exe.exists() {
            return Some(bundled_exe);
        }
    }

    // 2. In the same directory as the app executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let sibling = exe_dir.join("mymolt");
            if sibling.exists() {
                return Some(sibling);
            }
        }
    }

    // 3. Default install location
    if let Some(home) = dirs::home_dir() {
        let default_path = home.join(".mymolt").join("bin").join("mymolt");
        if default_path.exists() {
            return Some(default_path);
        }
    }

    // 4. In PATH
    if let Ok(path) = which::which("mymolt") {
        return Some(path);
    }

    None
}

/// Wait for the daemon to become healthy, with timeout.
async fn wait_for_ready(port: u16) {
    let max_attempts = 30;
    for i in 0..max_attempts {
        if check_health(port).await {
            return;
        }
        log::debug!("Waiting for daemon to start... attempt {}/{}", i + 1, max_attempts);
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    log::warn!("Daemon did not become ready within {}s", max_attempts / 2);
}

/// Check if the daemon is healthy.
async fn check_health(port: u16) -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap_or_default();

    client
        .get(format!("http://localhost:{}/api/system/status", port))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}
