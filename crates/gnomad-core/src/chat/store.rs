use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::chat::attachments::ChatAttachment;
use crate::config::paths::DataPaths;
use crate::error::{into_invoke_err, GnomadError};

fn chat_fs_err(message: impl Into<String>, detail: Option<String>) -> String {
    into_invoke_err(GnomadError::Fs {
        message: message.into(),
        detail,
    })
}

fn chat_internal_err(message: impl Into<String>, detail: Option<String>) -> String {
    into_invoke_err(GnomadError::Internal {
        message: message.into(),
        detail,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    #[serde(default)]
    pub status_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredChatMessage {
    pub role: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_executed: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_result: Option<StoredCommandResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<ChatAttachment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub messages: Vec<StoredChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionSummary {
    pub id: String,
    pub title: String,
    pub updated_at: u64,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStoreState {
    pub active_id: Option<String>,
    pub sessions: Vec<ChatSessionSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatStore {
    version: u32,
    active_id: Option<String>,
    sessions: Vec<ChatSession>,
}

fn chats_dir(paths: &DataPaths) -> PathBuf {
    paths.chats_dir()
}

fn store_path(dir: &Path) -> PathBuf {
    dir.join("store.json")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn new_session_id() -> String {
    format!("chat-{}", now_secs())
}

fn default_welcome_messages() -> Vec<StoredChatMessage> {
    vec![StoredChatMessage {
        role: "assistant".into(),
        text: "Hey — I'm Gnomad 🦙 I watch your active window and clipboard, run safe shell commands, and help you automate. What's on your mind?".into(),
        command_executed: None,
        command_result: None,
        attachments: None,
    }]
}

fn title_from_messages(messages: &[StoredChatMessage]) -> String {
    for m in messages {
        if m.role == "user" {
            let t = m.text.trim();
            if !t.is_empty() {
                let one_line = t.lines().next().unwrap_or(t);
                if one_line.chars().count() > 48 {
                    return format!("{}…", one_line.chars().take(48).collect::<String>());
                }
                return one_line.to_string();
            }
            if let Some(atts) = &m.attachments {
                if let Some(first) = atts.first() {
                    return format!("📎 {}", first.name);
                }
            }
        }
    }
    "New chat".into()
}

fn preview_from_messages(messages: &[StoredChatMessage]) -> String {
    for m in messages.iter().rev() {
        if m.role == "user" || m.role == "assistant" {
            let t = m.text.trim();
            if !t.is_empty() {
                let one_line = t.lines().next().unwrap_or(t);
                if one_line.chars().count() > 60 {
                    return format!("{}…", one_line.chars().take(60).collect::<String>());
                }
                return one_line.to_string();
            }
        }
    }
    String::new()
}

fn load_store(dir: &Path) -> Result<ChatStore, String> {
    fs::create_dir_all(dir)
        .map_err(|e| chat_fs_err("Could not create chat storage directory.", Some(e.to_string())))?;
    let path = store_path(dir);
    if !path.exists() {
        return Ok(ChatStore {
            version: 1,
            active_id: None,
            sessions: vec![],
        });
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| chat_fs_err("Could not read chat store.", Some(e.to_string())))?;
    serde_json::from_str(&raw)
        .map_err(|e| chat_fs_err("Chat store file is corrupted.", Some(e.to_string())))
}

fn save_store(dir: &Path, store: &ChatStore) -> Result<(), String> {
    fs::create_dir_all(dir)
        .map_err(|e| chat_fs_err("Could not create chat storage directory.", Some(e.to_string())))?;
    let raw = serde_json::to_string_pretty(store).map_err(|e| {
        chat_internal_err("Could not serialize chat store.", Some(e.to_string()))
    })?;
    fs::write(store_path(dir), raw)
        .map_err(|e| chat_fs_err("Could not save chat store.", Some(e.to_string())))
}

fn to_summary(session: &ChatSession) -> ChatSessionSummary {
    ChatSessionSummary {
        id: session.id.clone(),
        title: session.title.clone(),
        updated_at: session.updated_at,
        preview: preview_from_messages(&session.messages),
    }
}

fn ensure_default_session(store: &mut ChatStore) -> Result<(), String> {
    if !store.sessions.is_empty() {
        return Ok(());
    }
    let id = new_session_id();
    let now = now_secs();
    store.sessions.push(ChatSession {
        id: id.clone(),
        title: "New chat".into(),
        created_at: now,
        updated_at: now,
        messages: default_welcome_messages(),
    });
    store.active_id = Some(id);
    Ok(())
}

pub fn get_chat_store(paths: &DataPaths) -> Result<ChatStoreState, String> {
    let dir = chats_dir(paths);
    let mut store = load_store(&dir)?;
    ensure_default_session(&mut store)?;
    if store.active_id.is_none() {
        store.active_id = store.sessions.first().map(|s| s.id.clone());
    }
    save_store(&dir, &store)?;

    let mut summaries: Vec<ChatSessionSummary> = store.sessions.iter().map(to_summary).collect();
    summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(ChatStoreState {
        active_id: store.active_id,
        sessions: summaries,
    })
}

pub fn load_chat_session(paths: &DataPaths, id: &str) -> Result<ChatSession, String> {
    let dir = chats_dir(paths);
    let store = load_store(&dir)?;
    store
        .sessions
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| chat_internal_err(format!("Chat session not found: {id}"), None))
}

pub fn create_chat_session(paths: &DataPaths) -> Result<ChatSession, String> {
    let dir = chats_dir(paths);
    let mut store = load_store(&dir)?;
    let id = new_session_id();
    let now = now_secs();
    let session = ChatSession {
        id: id.clone(),
        title: "New chat".into(),
        created_at: now,
        updated_at: now,
        messages: default_welcome_messages(),
    };
    store.sessions.push(session.clone());
    store.active_id = Some(id);
    save_store(&dir, &store)?;
    Ok(session)
}

pub fn save_chat_session(
    paths: &DataPaths,
    id: &str,
    messages: Vec<StoredChatMessage>,
) -> Result<ChatSessionSummary, String> {
    let dir = chats_dir(paths);
    let mut store = load_store(&dir)?;
    let now = now_secs();
    let title = title_from_messages(&messages);

    let session = store
        .sessions
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| chat_internal_err(format!("Chat session not found: {id}"), None))?;

    session.messages = messages;
    session.title = title;
    session.updated_at = now;
    store.active_id = Some(id.to_string());
    let summary = to_summary(session);
    save_store(&dir, &store)?;
    Ok(summary)
}

pub fn delete_chat_session(paths: &DataPaths, id: &str) -> Result<ChatSession, String> {
    let dir = chats_dir(paths);
    let mut store = load_store(&dir)?;
    let idx = store
        .sessions
        .iter()
        .position(|s| s.id == id)
        .ok_or_else(|| chat_internal_err(format!("Chat session not found: {id}"), None))?;
    store.sessions.remove(idx);

    if store.sessions.is_empty() {
        ensure_default_session(&mut store)?;
    }

    if store.active_id.as_deref() == Some(id) {
        store.active_id = store.sessions.last().map(|s| s.id.clone());
    }

    let active_id = store
        .active_id
        .clone()
        .ok_or_else(|| chat_internal_err("No active chat session.", None))?;
    let active = store
        .sessions
        .iter()
        .find(|s| s.id == active_id)
        .cloned()
        .ok_or_else(|| chat_internal_err("Active chat session is missing from store.", None))?;

    save_store(&dir, &store)?;
    Ok(active)
}

pub fn set_active_chat_session(paths: &DataPaths, id: &str) -> Result<(), String> {
    let dir = chats_dir(paths);
    let mut store = load_store(&dir)?;
    if !store.sessions.iter().any(|s| s.id == id) {
        return Err(chat_internal_err(format!("Chat session not found: {id}"), None));
    }
    store.active_id = Some(id.to_string());
    save_store(&dir, &store)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_paths() -> DataPaths {
        let base = env::temp_dir().join(format!("gnomad-chat-test-{}", now_secs()));
        DataPaths::from_data_root(base)
    }

    #[test]
    fn creates_default_session() {
        let paths = temp_paths();
        let state = get_chat_store(&paths).expect("store");
        assert!(!state.sessions.is_empty());
        assert!(state.active_id.is_some());
    }

    #[test]
    fn round_trip_messages() {
        let paths = temp_paths();
        let state = get_chat_store(&paths).expect("store");
        let id = state.active_id.unwrap();
        let mut session = load_chat_session(&paths, &id).expect("load");
        session.messages.push(StoredChatMessage {
            role: "user".into(),
            text: "Hello".into(),
            command_executed: None,
            command_result: None,
            attachments: None,
        });
        let summary = save_chat_session(&paths, &id, session.messages).expect("save");
        assert_eq!(summary.title, "Hello");
    }
}
