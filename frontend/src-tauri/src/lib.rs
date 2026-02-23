//! MyMolt Tauri Desktop App — Sovereign Runtime
//!
//! This wraps the MyMolt web frontend in a native desktop window and
//! manages the MyMolt daemon as a background process.

use serde::{Deserialize, Serialize};
use tauri::{
    Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
};
use std::sync::Arc;
use tokio::sync::Mutex;

mod daemon;

// ── State ──────────────────────────────────────────────────────────

pub struct DaemonState {
    pub process: Arc<Mutex<Option<tokio::process::Child>>>,
    pub port: u16,
    pub ready: Arc<std::sync::atomic::AtomicBool>,
}

// ── Tauri Commands ─────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub port: u16,
    pub version: String,
}

/// Get the MyMolt daemon status
#[tauri::command]
async fn get_daemon_status(state: tauri::State<'_, DaemonState>) -> Result<DaemonStatus, String> {
    let running = state.ready.load(std::sync::atomic::Ordering::Relaxed);
    Ok(DaemonStatus {
        running,
        port: state.port,
        version: "1.0.0".into(),
    })
}

/// Get the API base URL for the MyMolt daemon
#[tauri::command]
fn get_api_url(state: tauri::State<'_, DaemonState>) -> String {
    format!("http://localhost:{}", state.port)
}

/// Ask MyMolt about page content (direct bridge, no HTTP overhead)
#[tauri::command]
async fn ask_agent(
    state: tauri::State<'_, DaemonState>,
    question: String,
    page_url: String,
    page_text: String,
) -> Result<String, String> {
    let port = state.port;
    let client = reqwest::Client::new();
    let res = client
        .post(format!("http://localhost:{}/api/browse/ask", port))
        .json(&serde_json::json!({
            "url": page_url,
            "page_text": page_text,
            "question": question,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(body.get("answer").and_then(|a| a.as_str()).unwrap_or("No response").to_string())
}

// ── App Entry ──────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let port = 3000u16; // MyMolt daemon port

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .manage(DaemonState {
            process: Arc::new(Mutex::new(None)),
            port,
            ready: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
        .setup(|app| {
            // ── System Tray ────────────────────────────────────
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                let show = MenuItemBuilder::with_id("show", "Show MyMolt").build(app)?;
                let quit = MenuItemBuilder::with_id("quit", "Quit MyMolt").build(app)?;
                let menu = MenuBuilder::new(app)
                    .item(&show)
                    .separator()
                    .item(&quit)
                    .build()?;

                let _tray = TrayIconBuilder::new()
                    .menu(&menu)
                    .tooltip("MyMolt — Sovereign Runtime")
                    .on_menu_event(|app, event| {
                        match event.id().as_ref() {
                            "show" => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                            "quit" => {
                                app.exit(0);
                            }
                            _ => {}
                        }
                    })
                    .build(app)?;

                // Single instance (desktop only)
                app.handle().plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }))?;
            }

            // ── Start MyMolt Daemon ────────────────────────────
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                daemon::start_daemon(&handle).await;
            });

            log::info!("MyMolt desktop app started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_daemon_status,
            get_api_url,
            ask_agent,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MyMolt");
}
