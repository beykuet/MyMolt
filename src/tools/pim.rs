// SPDX-License-Identifier: EUPL-1.2
// Copyright (c) 2026 Benjamin Küttner <benjamin.kuettner@icloud.com>
// Patent Pending — DE Gebrauchsmuster, filed 2026-02-23

//! Sovereign PIM — Personal Information Manager tools.
//!
//! Local-first Calendar, Contacts, and Notes stored as encrypted JSON
//! inside the workspace. Data stays sovereign — no cloud sync.
//! When a `SecretStore` is provided, PIM data is encrypted at rest
//! using ChaCha20-Poly1305.

use crate::security::secrets::SecretStore;
use crate::tools::{Tool, ToolResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

// ── Data types ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub date: String,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
}

// ── Storage ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PimStore {
    #[serde(default)]
    events: Vec<CalendarEvent>,
    #[serde(default)]
    contacts: Vec<Contact>,
    #[serde(default)]
    notes: Vec<Note>,
}

fn pim_path(workspace: &std::path::Path) -> PathBuf {
    workspace.join(".mymolt").join("pim.json")
}

fn load_store(workspace: &std::path::Path, secrets: &Option<SecretStore>) -> PimStore {
    let path = pim_path(workspace);
    if path.exists() {
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => return PimStore::default(),
        };
        // If encrypted, decrypt first
        let json = if let Some(store) = secrets {
            store.decrypt(&raw).unwrap_or(raw)
        } else {
            raw
        };
        serde_json::from_str(&json).unwrap_or_default()
    } else {
        PimStore::default()
    }
}

fn save_store(
    workspace: &std::path::Path,
    store: &PimStore,
    secrets: &Option<SecretStore>,
) -> anyhow::Result<()> {
    let path = pim_path(workspace);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(store)?;
    let data = if let Some(secret_store) = secrets {
        secret_store.encrypt(&json)?
    } else {
        json
    };
    std::fs::write(&path, data)?;
    Ok(())
}

// ── Shared PIM State (RwLock) ──────────────────────────────────

/// Shared PIM state: in-memory store protected by `RwLock`.
///
/// Read-only tools (list/search/read) acquire a read lock, so
/// multiple concurrent readers proceed without blocking.
/// Write tools (add/create) acquire a write lock and flush to disk.
pub struct PimState {
    store: RwLock<PimStore>,
    workspace: PathBuf,
    secrets: Option<SecretStore>,
}

impl PimState {
    pub fn new(workspace: PathBuf, secrets: Option<SecretStore>) -> Self {
        let store = load_store(&workspace, &secrets);
        Self {
            store: RwLock::new(store),
            workspace,
            secrets,
        }
    }

    /// Flush the in-memory store to disk (encrypted if SecretStore is set).
    async fn flush(&self) -> anyhow::Result<()> {
        let store = self.store.read().await;
        save_store(&self.workspace, &store, &self.secrets)
    }
}

// ── Calendar Add Tool ────────────────────────────────────────

pub struct CalendarAddTool {
    state: Arc<PimState>,
}

impl CalendarAddTool {
    pub fn new(workspace: PathBuf, secrets: Option<SecretStore>) -> Self {
        Self {
            state: Arc::new(PimState::new(workspace, secrets)),
        }
    }
    fn from_state(state: Arc<PimState>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Tool for CalendarAddTool {
    fn name(&self) -> &str {
        "calendar_add"
    }

    fn description(&self) -> &str {
        "Add an event to the local calendar. Provide title, date (YYYY-MM-DD), optionally time (HH:MM) and description."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "Event title"},
                "date": {"type": "string", "description": "Date in YYYY-MM-DD format"},
                "time": {"type": "string", "description": "Optional time in HH:MM format"},
                "description": {"type": "string", "description": "Optional event description"}
            },
            "required": ["title", "date"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'title' parameter"))?;
        let date = args
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'date' parameter"))?;
        let time = args.get("time").and_then(|v| v.as_str()).map(String::from);
        let description = args
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let event = CalendarEvent {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            date: date.to_string(),
            time,
            description,
        };

        let summary = format!(
            "📅 Added: {} on {}{}",
            event.title,
            event.date,
            event.time.as_deref().map_or("".into(), |t| format!(" at {t}"))
        );

        {
            let mut store = self.state.store.write().await;
            store.events.push(event);
        }
        self.state.flush().await?;

        Ok(ToolResult {
            success: true,
            output: summary,
            error: None,
        })
    }
}

// ── Calendar List Tool ──────────────────────────────────────

pub struct CalendarListTool {
    state: Arc<PimState>,
}

impl CalendarListTool {
    pub fn new(workspace: PathBuf, secrets: Option<SecretStore>) -> Self {
        Self {
            state: Arc::new(PimState::new(workspace, secrets)),
        }
    }
    fn from_state(state: Arc<PimState>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Tool for CalendarListTool {
    fn name(&self) -> &str {
        "calendar_list"
    }

    fn description(&self) -> &str {
        "List upcoming events from the local calendar. Optionally filter by date."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "from": {"type": "string", "description": "Show events from this date (YYYY-MM-DD). Default: today"},
                "limit": {"type": "integer", "description": "Maximum number of events to return. Default: 10"}
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let from = args
            .get("from")
            .and_then(|v| v.as_str())
            .unwrap_or("0000-01-01");
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let store = self.state.store.read().await;

        let mut events: Vec<&CalendarEvent> = store
            .events
            .iter()
            .filter(|e| e.date.as_str() >= from)
            .collect();
        events.sort_by(|a, b| a.date.cmp(&b.date));
        events.truncate(limit);

        if events.is_empty() {
            return Ok(ToolResult {
                success: true,
                output: "📅 No upcoming events.".into(),
                error: None,
            });
        }

        let lines: Vec<String> = events
            .iter()
            .map(|e| {
                let time_str = e.time.as_deref().map_or("".into(), |t| format!(" {t}"));
                let desc = if e.description.is_empty() {
                    ""
                } else {
                    " — "
                };
                format!("📅 {} {}{}: {}{}{}", e.date, time_str, "", e.title, desc, e.description)
            })
            .collect();

        Ok(ToolResult {
            success: true,
            output: lines.join("\n"),
            error: None,
        })
    }
}

// ── Contacts Add Tool ───────────────────────────────────────

pub struct ContactsAddTool {
    state: Arc<PimState>,
}

impl ContactsAddTool {
    pub fn new(workspace: PathBuf, secrets: Option<SecretStore>) -> Self {
        Self {
            state: Arc::new(PimState::new(workspace, secrets)),
        }
    }
    fn from_state(state: Arc<PimState>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Tool for ContactsAddTool {
    fn name(&self) -> &str {
        "contacts_add"
    }

    fn description(&self) -> &str {
        "Add or update a contact. Provide a name and optionally email, phone, notes."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "description": "Contact name"},
                "email": {"type": "string", "description": "Email address"},
                "phone": {"type": "string", "description": "Phone number"},
                "notes": {"type": "string", "description": "Additional notes"}
            },
            "required": ["name"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;

        let mut store = self.state.store.write().await;

        // Update existing or create new
        let existing = store.contacts.iter_mut().find(|c| c.name == name);
        if let Some(contact) = existing {
            if let Some(email) = args.get("email").and_then(|v| v.as_str()) {
                contact.email = Some(email.to_string());
            }
            if let Some(phone) = args.get("phone").and_then(|v| v.as_str()) {
                contact.phone = Some(phone.to_string());
            }
            if let Some(notes) = args.get("notes").and_then(|v| v.as_str()) {
                contact.notes = notes.to_string();
            }
            drop(store);
            self.state.flush().await?;
            return Ok(ToolResult {
                success: true,
                output: format!("👤 Updated contact: {name}"),
                error: None,
            });
        }

        let contact = Contact {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            email: args.get("email").and_then(|v| v.as_str()).map(String::from),
            phone: args.get("phone").and_then(|v| v.as_str()).map(String::from),
            notes: args
                .get("notes")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        };

        store.contacts.push(contact);
        drop(store);
        self.state.flush().await?;

        Ok(ToolResult {
            success: true,
            output: format!("👤 Added contact: {name}"),
            error: None,
        })
    }
}

// ── Contacts Search Tool ────────────────────────────────────

pub struct ContactsSearchTool {
    state: Arc<PimState>,
}

impl ContactsSearchTool {
    pub fn new(workspace: PathBuf, secrets: Option<SecretStore>) -> Self {
        Self {
            state: Arc::new(PimState::new(workspace, secrets)),
        }
    }
    fn from_state(state: Arc<PimState>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Tool for ContactsSearchTool {
    fn name(&self) -> &str {
        "contacts_search"
    }

    fn description(&self) -> &str {
        "Search contacts by name. Returns matching contacts with their details."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query (case-insensitive name match)"}
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?
            .to_lowercase();

        let store = self.state.store.read().await;

        let matches: Vec<String> = store
            .contacts
            .iter()
            .filter(|c| c.name.to_lowercase().contains(&query))
            .map(|c| {
                let mut parts = vec![format!("👤 {}", c.name)];
                if let Some(ref email) = c.email {
                    parts.push(format!("  📧 {email}"));
                }
                if let Some(ref phone) = c.phone {
                    parts.push(format!("  📞 {phone}"));
                }
                if !c.notes.is_empty() {
                    parts.push(format!("  📝 {}", c.notes));
                }
                parts.join("\n")
            })
            .collect();

        if matches.is_empty() {
            return Ok(ToolResult {
                success: true,
                output: format!("No contacts matching '{query}'."),
                error: None,
            });
        }

        Ok(ToolResult {
            success: true,
            output: matches.join("\n\n"),
            error: None,
        })
    }
}

// ── Notes Create Tool ───────────────────────────────────────

pub struct NotesCreateTool {
    state: Arc<PimState>,
}

impl NotesCreateTool {
    pub fn new(workspace: PathBuf, secrets: Option<SecretStore>) -> Self {
        Self {
            state: Arc::new(PimState::new(workspace, secrets)),
        }
    }
    fn from_state(state: Arc<PimState>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Tool for NotesCreateTool {
    fn name(&self) -> &str {
        "notes_create"
    }

    fn description(&self) -> &str {
        "Create a new note with a title and content. Stored locally."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "Note title"},
                "content": {"type": "string", "description": "Note body content"}
            },
            "required": ["title", "content"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'title' parameter"))?;
        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;

        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let note = Note {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            content: content.to_string(),
            created_at: now,
        };

        let summary = format!("📝 Created note: {}", note.title);
        {
            let mut store = self.state.store.write().await;
            store.notes.push(note);
        }
        self.state.flush().await?;

        Ok(ToolResult {
            success: true,
            output: summary,
            error: None,
        })
    }
}

// ── Notes Search Tool ───────────────────────────────────────

pub struct NotesSearchTool {
    state: Arc<PimState>,
}

impl NotesSearchTool {
    pub fn new(workspace: PathBuf, secrets: Option<SecretStore>) -> Self {
        Self {
            state: Arc::new(PimState::new(workspace, secrets)),
        }
    }
    fn from_state(state: Arc<PimState>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Tool for NotesSearchTool {
    fn name(&self) -> &str {
        "notes_search"
    }

    fn description(&self) -> &str {
        "Search notes by title or content. Returns matching notes."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query (case-insensitive)"},
                "limit": {"type": "integer", "description": "Max results. Default: 10"}
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?
            .to_lowercase();
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let store = self.state.store.read().await;

        let matches: Vec<String> = store
            .notes
            .iter()
            .filter(|n| {
                n.title.to_lowercase().contains(&query)
                    || n.content.to_lowercase().contains(&query)
            })
            .take(limit)
            .map(|n| {
                let preview = if n.content.len() > 100 {
                    format!("{}…", &n.content[..100])
                } else {
                    n.content.clone()
                };
                format!("📝 {} ({})\\n   {}", n.title, n.created_at, preview)
            })
            .collect();

        if matches.is_empty() {
            return Ok(ToolResult {
                success: true,
                output: format!("No notes matching '{query}'."),
                error: None,
            });
        }

        Ok(ToolResult {
            success: true,
            output: matches.join("\n\n"),
            error: None,
        })
    }
}

// ── Notes Read Tool ─────────────────────────────────────────

pub struct NotesReadTool {
    state: Arc<PimState>,
}

impl NotesReadTool {
    pub fn new(workspace: PathBuf, secrets: Option<SecretStore>) -> Self {
        Self {
            state: Arc::new(PimState::new(workspace, secrets)),
        }
    }
    fn from_state(state: Arc<PimState>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Tool for NotesReadTool {
    fn name(&self) -> &str {
        "notes_read"
    }

    fn description(&self) -> &str {
        "Read a specific note by title. Returns the full content."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "Exact note title to read"}
            },
            "required": ["title"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let title = args
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'title' parameter"))?;

        let store = self.state.store.read().await;

        let note = store.notes.iter().find(|n| n.title == title);

        match note {
            Some(n) => Ok(ToolResult {
                success: true,
                output: format!(
                    "📝 {}\nCreated: {}\n\n{}",
                    n.title, n.created_at, n.content
                ),
                error: None,
            }),
            None => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Note '{}' not found.", title)),
            }),
        }
    }
}

/// Create all PIM tools for a given workspace.
///
/// All tools share a single `PimState` with `RwLock` — readers
/// proceed concurrently, writers serialize and flush to disk.
/// If `secrets` is provided, PIM data will be encrypted at rest.
pub fn pim_tools(workspace: &std::path::Path, secrets: Option<SecretStore>) -> Vec<Box<dyn Tool>> {
    let state = Arc::new(PimState::new(workspace.to_path_buf(), secrets));
    vec![
        Box::new(CalendarAddTool::from_state(state.clone())),
        Box::new(CalendarListTool::from_state(state.clone())),
        Box::new(ContactsAddTool::from_state(state.clone())),
        Box::new(ContactsSearchTool::from_state(state.clone())),
        Box::new(NotesCreateTool::from_state(state.clone())),
        Box::new(NotesSearchTool::from_state(state.clone())),
        Box::new(NotesReadTool::from_state(state)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_workspace() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        (dir, path)
    }

    // ── Calendar Tests ──────────────────────────────────────────

    #[tokio::test]
    async fn calendar_add_and_list() {
        let (_dir, ws) = test_workspace();
        let state = Arc::new(PimState::new(ws, None));
        let add = CalendarAddTool::from_state(state.clone());
        let list = CalendarListTool::from_state(state);

        // Add event
        let result = add
            .execute(serde_json::json!({
                "title": "Team standup",
                "date": "2026-03-01",
                "time": "09:00"
            }))
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.output.contains("Team standup"));

        // List events (same shared state)
        let result = list
            .execute(serde_json::json!({}))
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.output.contains("Team standup"));
    }

    #[tokio::test]
    async fn calendar_list_empty() {
        let (_dir, ws) = test_workspace();
        let list = CalendarListTool::new(ws, None);
        let result = list.execute(serde_json::json!({})).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("No upcoming"));
    }

    #[tokio::test]
    async fn calendar_list_with_date_filter() {
        let (_dir, ws) = test_workspace();
        let state = Arc::new(PimState::new(ws, None));
        let add = CalendarAddTool::from_state(state.clone());
        let list = CalendarListTool::from_state(state);

        add.execute(serde_json::json!({"title": "Past", "date": "2020-01-01"}))
            .await
            .unwrap();
        add.execute(serde_json::json!({"title": "Future", "date": "2030-01-01"}))
            .await
            .unwrap();

        let result = list
            .execute(serde_json::json!({"from": "2025-01-01"}))
            .await
            .unwrap();
        assert!(result.output.contains("Future"));
        assert!(!result.output.contains("Past"));
    }

    // ── Contacts Tests ──────────────────────────────────────────

    #[tokio::test]
    async fn contacts_add_and_search() {
        let (_dir, ws) = test_workspace();
        let state = Arc::new(PimState::new(ws, None));
        let add = ContactsAddTool::from_state(state.clone());
        let search = ContactsSearchTool::from_state(state);

        let result = add
            .execute(serde_json::json!({
                "name": "Max Mustermann",
                "email": "max@example.com",
                "phone": "+49 170 1234567"
            }))
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.output.contains("Added"));

        let result = search
            .execute(serde_json::json!({"query": "max"}))
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.output.contains("Max Mustermann"));
        assert!(result.output.contains("max@example.com"));
    }

    #[tokio::test]
    async fn contacts_update_existing() {
        let (_dir, ws) = test_workspace();
        let state = Arc::new(PimState::new(ws, None));
        let add = ContactsAddTool::from_state(state.clone());
        let search = ContactsSearchTool::from_state(state);

        add.execute(serde_json::json!({"name": "Anna", "email": "old@example.com"}))
            .await
            .unwrap();
        let result = add
            .execute(serde_json::json!({"name": "Anna", "email": "new@example.com"}))
            .await
            .unwrap();
        assert!(result.output.contains("Updated"));

        let result = search
            .execute(serde_json::json!({"query": "anna"}))
            .await
            .unwrap();
        assert!(result.output.contains("new@example.com"));
        assert!(!result.output.contains("old@example.com"));
    }

    #[tokio::test]
    async fn contacts_search_no_results() {
        let (_dir, ws) = test_workspace();
        let search = ContactsSearchTool::new(ws, None);
        let result = search
            .execute(serde_json::json!({"query": "nobody"}))
            .await
            .unwrap();
        assert!(result.output.contains("No contacts"));
    }

    // ── Notes Tests ─────────────────────────────────────────────

    #[tokio::test]
    async fn notes_create_and_search() {
        let (_dir, ws) = test_workspace();
        let state = Arc::new(PimState::new(ws, None));
        let create = NotesCreateTool::from_state(state.clone());
        let search = NotesSearchTool::from_state(state);

        let result = create
            .execute(serde_json::json!({
                "title": "Shopping list",
                "content": "Milk, bread, butter"
            }))
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.output.contains("Shopping list"));

        let result = search
            .execute(serde_json::json!({"query": "bread"}))
            .await
            .unwrap();
        assert!(result.output.contains("Shopping list"));
    }

    #[tokio::test]
    async fn notes_read_existing() {
        let (_dir, ws) = test_workspace();
        let state = Arc::new(PimState::new(ws, None));
        let create = NotesCreateTool::from_state(state.clone());
        let read = NotesReadTool::from_state(state);

        create
            .execute(serde_json::json!({
                "title": "Secret recipe",
                "content": "Two cups of flour, one egg"
            }))
            .await
            .unwrap();

        let result = read
            .execute(serde_json::json!({"title": "Secret recipe"}))
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.output.contains("Two cups of flour"));
    }

    #[tokio::test]
    async fn notes_read_not_found() {
        let (_dir, ws) = test_workspace();
        let read = NotesReadTool::new(ws, None);
        let result = read
            .execute(serde_json::json!({"title": "Ghost note"}))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("not found"));
    }

    #[tokio::test]
    async fn notes_search_no_results() {
        let (_dir, ws) = test_workspace();
        let search = NotesSearchTool::new(ws, None);
        let result = search
            .execute(serde_json::json!({"query": "nonexistent"}))
            .await
            .unwrap();
        assert!(result.output.contains("No notes"));
    }

    // ── PIM Tools Factory ───────────────────────────────────────

    #[test]
    fn pim_tools_creates_all_seven() {
        let (_dir, ws) = test_workspace();
        let tools = pim_tools(&ws, None);
        assert_eq!(tools.len(), 7);
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        assert!(names.contains(&"calendar_add"));
        assert!(names.contains(&"calendar_list"));
        assert!(names.contains(&"contacts_add"));
        assert!(names.contains(&"contacts_search"));
        assert!(names.contains(&"notes_create"));
        assert!(names.contains(&"notes_search"));
        assert!(names.contains(&"notes_read"));
    }
}
