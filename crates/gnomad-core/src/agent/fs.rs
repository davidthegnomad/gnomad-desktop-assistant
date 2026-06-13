use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::agent::settings::{read_settings, resolve_agent_path_with_token, AgentSettingsState};
use crate::agent::tokens::path::PathScope;
use crate::agent::tokens::PathTokenState;
use crate::config::paths::DataPaths;
use crate::error::{into_invoke_err, GnomadError};

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

fn settings_ctx(state: &AgentSettingsState) -> (std::path::PathBuf, crate::agent::TrustMode) {
    let s = read_settings(state);
    (s.workspace_root, s.trust_mode)
}

fn resolve_fs_path(
    path_state: &PathTokenState,
    settings_state: &AgentSettingsState,
    input: &str,
    scope: PathScope,
    path_approval_token: Option<&str>,
    path_approved: Option<bool>,
) -> Result<std::path::PathBuf, String> {
    let (workspace, trust) = settings_ctx(settings_state);
    resolve_agent_path_with_token(
        input,
        &workspace,
        trust,
        path_state,
        scope,
        path_approval_token,
        path_approved,
    )
}

pub fn fs_list_inner(
    path_state: &PathTokenState,
    state: &AgentSettingsState,
    path: Option<String>,
    path_approval_token: Option<&str>,
    path_approved: Option<bool>,
) -> Result<FsListResult, String> {
    let resolved = resolve_fs_path(
        path_state,
        state,
        path.as_deref().unwrap_or("."),
        PathScope::Read,
        path_approval_token,
        path_approved,
    )?;
    if !resolved.is_dir() {
        return Err(into_invoke_err(GnomadError::Fs {
            message: format!("Not a directory: {}", resolved.display()),
            detail: None,
        }));
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(&resolved).map_err(|e| {
        into_invoke_err(GnomadError::Fs {
            message: "Failed to read directory.".into(),
            detail: Some(e.to_string()),
        })
    })? {
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

pub fn fs_read_inner(
    path_state: &PathTokenState,
    state: &AgentSettingsState,
    path: String,
    path_approval_token: Option<&str>,
    path_approved: Option<bool>,
) -> Result<FsReadResult, String> {
    let resolved = resolve_fs_path(
        path_state,
        state,
        &path,
        PathScope::Read,
        path_approval_token,
        path_approved,
    )?;
    if resolved.is_dir() {
        return Err(into_invoke_err(GnomadError::Fs {
            message: "Path is a directory; use fs_list.".into(),
            detail: None,
        }));
    }
    let data = fs::read(&resolved).map_err(|e| {
        into_invoke_err(GnomadError::Fs {
            message: "Failed to read file.".into(),
            detail: Some(e.to_string()),
        })
    })?;
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

pub fn fs_write_inner(
    paths: &DataPaths,
    path_state: &PathTokenState,
    state: &AgentSettingsState,
    path: String,
    content: String,
    path_approval_token: Option<&str>,
    path_approved: Option<bool>,
) -> Result<FsWriteResult, String> {
    let resolved = resolve_fs_path(
        path_state,
        state,
        &path,
        PathScope::Write,
        path_approval_token,
        path_approved,
    )?;
    if let Some(parent) = resolved.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&resolved, &content).map_err(|e| e.to_string())?;
    crate::agent::audit::log_action(
        paths,
        "fs_write",
        &format!("{} ({} bytes)", resolved.display(), content.len()),
    );
    Ok(FsWriteResult {
        path: resolved.to_string_lossy().to_string(),
        bytes_written: content.len(),
    })
}

pub fn fs_search_inner(
    path_state: &PathTokenState,
    state: &AgentSettingsState,
    query: String,
    path: Option<String>,
    path_approval_token: Option<&str>,
    path_approved: Option<bool>,
) -> Result<FsSearchResult, String> {
    let root = resolve_fs_path(
        path_state,
        state,
        path.as_deref().unwrap_or("."),
        PathScope::Read,
        path_approval_token,
        path_approved,
    )?;
    let q = query.trim();
    if q.is_empty() {
        return Err(into_invoke_err(GnomadError::Fs {
            message: "Search query cannot be empty.".into(),
            detail: None,
        }));
    }
    let mut matches = Vec::new();
    search_dir(&root, q, &mut matches)?;
    Ok(FsSearchResult {
        query: q.to_string(),
        matches,
    })
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
                if data.len() > MAX_READ_BYTES || data.contains(&0) {
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
