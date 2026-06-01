use crate::agent_settings::{resolve_agent_path, read_settings, TrustMode, AgentSettingsState};
use serde::Serialize;
use std::fs;
use std::path::Path;

const MAX_READ_BYTES: usize = 256 * 1024;
const MAX_SEARCH_MATCHES: usize = 80;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FsListResult {
    pub path: String,
    pub entries: Vec<FsEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FsEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FsReadResult {
    pub path: String,
    pub content: String,
    pub truncated: bool,
    pub binary: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FsWriteResult {
    pub path: String,
    pub bytes_written: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FsSearchMatch {
    pub path: String,
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FsSearchResult {
    pub query: String,
    pub matches: Vec<FsSearchMatch>,
}

fn settings_ctx(state: &AgentSettingsState) -> (std::path::PathBuf, TrustMode) {
    let s = read_settings(state);
    (s.workspace_root, s.trust_mode)
}

pub fn fs_list_inner(
    state: &AgentSettingsState,
    path: Option<String>,
    path_approved: bool,
) -> Result<FsListResult, String> {
    let (workspace, trust) = settings_ctx(state);
    let resolved = resolve_agent_path(
        path.as_deref().unwrap_or("."),
        &workspace,
        trust,
        path_approved,
    )?;
    if !resolved.is_dir() {
        return Err(format!("Not a directory: {}", resolved.display()));
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(&resolved).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        entries.push(FsEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir: meta.is_dir(),
            size: meta.len(),
        });
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(FsListResult {
        path: resolved.to_string_lossy().to_string(),
        entries,
    })
}

#[tauri::command]
pub fn agent_fs_list(
    state: tauri::State<'_, AgentSettingsState>,
    path: Option<String>,
    path_approved: Option<bool>,
) -> Result<FsListResult, String> {
    fs_list_inner(state.inner(), path, path_approved.unwrap_or(false))
}

pub fn fs_read_inner(
    state: &AgentSettingsState,
    path: String,
    path_approved: bool,
) -> Result<FsReadResult, String> {
    let (workspace, trust) = settings_ctx(state);
    let resolved = resolve_agent_path(&path, &workspace, trust, path_approved)?;
    if resolved.is_dir() {
        return Err("Path is a directory; use fs_list.".into());
    }
    let data = fs::read(&resolved).map_err(|e| e.to_string())?;
    if data.iter().take(8192).any(|b| *b == 0) {
        return Ok(FsReadResult {
            path: resolved.to_string_lossy().to_string(),
            content: format!("(binary file, {} bytes — not shown)", data.len()),
            truncated: false,
            binary: true,
        });
    }
    let truncated = data.len() > MAX_READ_BYTES;
    let slice = if truncated {
        &data[..MAX_READ_BYTES]
    } else {
        &data[..]
    };
    let content = String::from_utf8_lossy(slice).to_string();
    Ok(FsReadResult {
        path: resolved.to_string_lossy().to_string(),
        content,
        truncated,
        binary: false,
    })
}

#[tauri::command]
pub fn agent_fs_read(
    state: tauri::State<'_, AgentSettingsState>,
    path: String,
    path_approved: Option<bool>,
) -> Result<FsReadResult, String> {
    fs_read_inner(state.inner(), path, path_approved.unwrap_or(false))
}

pub fn fs_write_inner(
    app: &tauri::AppHandle,
    state: &AgentSettingsState,
    path: String,
    content: String,
    path_approved: bool,
) -> Result<FsWriteResult, String> {
    let (workspace, trust) = settings_ctx(state);
    let resolved = resolve_agent_path(&path, &workspace, trust, path_approved)?;
    if let Some(parent) = resolved.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&resolved, &content).map_err(|e| e.to_string())?;
    crate::agent_audit::log_action(
        &app,
        "fs_write",
        &format!("{} ({} bytes)", resolved.display(), content.len()),
    );
    Ok(FsWriteResult {
        path: resolved.to_string_lossy().to_string(),
        bytes_written: content.len(),
    })
}

#[tauri::command]
pub fn agent_fs_write(
    app: tauri::AppHandle,
    state: tauri::State<'_, AgentSettingsState>,
    path: String,
    content: String,
    path_approved: Option<bool>,
) -> Result<FsWriteResult, String> {
    fs_write_inner(
        &app,
        state.inner(),
        path,
        content,
        path_approved.unwrap_or(false),
    )
}

pub fn fs_search_inner(
    state: &AgentSettingsState,
    query: String,
    path: Option<String>,
    path_approved: bool,
) -> Result<FsSearchResult, String> {
    let (workspace, trust) = settings_ctx(state);
    let root = resolve_agent_path(
        path.as_deref().unwrap_or("."),
        &workspace,
        trust,
        path_approved,
    )?;
    let q = query.trim();
    if q.is_empty() {
        return Err("Search query cannot be empty.".into());
    }
    let mut matches = Vec::new();
    search_dir(&root, q, &mut matches)?;
    Ok(FsSearchResult {
        query: q.to_string(),
        matches,
    })
}

#[tauri::command]
pub fn agent_fs_search(
    state: tauri::State<'_, AgentSettingsState>,
    query: String,
    path: Option<String>,
    path_approved: Option<bool>,
) -> Result<FsSearchResult, String> {
    fs_search_inner(
        state.inner(),
        query,
        path,
        path_approved.unwrap_or(false),
    )
}

fn search_dir(dir: &Path, query: &str, matches: &mut Vec<FsSearchMatch>) -> Result<(), String> {
    if matches.len() >= MAX_SEARCH_MATCHES {
        return Ok(());
    }
    let q_lower = query.to_lowercase();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        if matches.len() >= MAX_SEARCH_MATCHES {
            break;
        }
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        if path.is_dir() {
            search_dir(&path, query, matches)?;
        } else if path.is_file() {
            if let Ok(data) = fs::read(&path) {
                if data.len() > MAX_READ_BYTES {
                    continue;
                }
                if data.contains(&0) {
                    continue;
                }
                let text = String::from_utf8_lossy(&data);
                for (i, line) in text.lines().enumerate() {
                    if line.to_lowercase().contains(&q_lower) {
                        matches.push(FsSearchMatch {
                            path: path.to_string_lossy().to_string(),
                            line: i + 1,
                            text: line.chars().take(200).collect(),
                        });
                        if matches.len() >= MAX_SEARCH_MATCHES {
                            break;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
