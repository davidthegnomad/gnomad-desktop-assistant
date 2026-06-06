use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::agent::tokens::path::{PathScope, PathTokenState};
use crate::config::paths::DataPaths;
use crate::shell::sandbox::{sandbox_level, sandbox_shell_available};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrustMode {
    Standard,
    Yolo,
}

/// How elevated (sudo) commands authenticate on Linux.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SudoAuthMode {
    /// System polkit dialog via `pkexec` (default).
    #[default]
    Polkit,
    /// Pipe stored sudo password to `sudo -S` (requires agent secrets vault).
    StoredPassword,
}

impl SudoAuthMode {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "stored_password" | "stored" | "password" => Self::StoredPassword,
            _ => Self::Polkit,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Polkit => "polkit",
            Self::StoredPassword => "stored_password",
        }
    }
}

impl TrustMode {
    pub fn from_str(s: &str) -> Self {
        if s.eq_ignore_ascii_case("yolo") {
            Self::Yolo
        } else {
            Self::Standard
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSettingsDto {
    pub workspace_root: String,
    pub trust_mode: String,
    pub home_dir: String,
    pub command_planner_enabled: bool,
    pub command_planner_model: String,
    pub command_planner_use_chat_local_model: bool,
    pub command_planner_gguf_path: String,
    #[serde(default)]
    pub use_gguf_for_local_chat: bool,
    #[serde(default)]
    pub sandbox_shell_in_yolo: bool,
    #[serde(default = "default_sandbox_level")]
    pub sandbox_level: String,
    #[serde(default)]
    pub agent_secrets_enabled: bool,
    #[serde(default)]
    pub sudo_auth_mode: String,
}

fn default_sandbox_level() -> String {
    sandbox_level().to_string()
}

pub struct AgentSettingsState {
    pub inner: Mutex<AgentSettings>,
}

#[derive(Debug, Clone)]
pub struct AgentSettings {
    pub workspace_root: PathBuf,
    pub trust_mode: TrustMode,
    pub command_planner_enabled: bool,
    pub command_planner_model: String,
    pub command_planner_use_chat_local_model: bool,
    pub command_planner_gguf_path: String,
    pub use_gguf_for_local_chat: bool,
    pub sandbox_shell_in_yolo: bool,
    pub agent_secrets_enabled: bool,
    pub sudo_auth_mode: SudoAuthMode,
}

impl Default for AgentSettingsState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(default_agent_settings()),
        }
    }
}

pub fn default_workspace_root() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")))
}

fn default_agent_settings() -> AgentSettings {
    AgentSettings {
        workspace_root: default_workspace_root(),
        trust_mode: TrustMode::Standard,
        command_planner_enabled: false,
        command_planner_model: "llama3.2:1b".into(),
        command_planner_use_chat_local_model: false,
        command_planner_gguf_path: String::new(),
        use_gguf_for_local_chat: false,
        sandbox_shell_in_yolo: false,
        agent_secrets_enabled: false,
        sudo_auth_mode: SudoAuthMode::Polkit,
    }
}

pub fn load_persisted(paths: &DataPaths) -> AgentSettings {
    let path = paths.agent_settings_path();
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(dto) = serde_json::from_str::<AgentSettingsDto>(&text) {
            let root = PathBuf::from(&dto.workspace_root);
            let workspace_root = if root.is_absolute() && root.exists() {
                root
            } else {
                default_workspace_root()
            };
            return AgentSettings {
                workspace_root,
                trust_mode: TrustMode::from_str(&dto.trust_mode),
                command_planner_enabled: dto.command_planner_enabled,
                command_planner_model: if dto.command_planner_model.trim().is_empty() {
                    "llama3.2:1b".into()
                } else {
                    dto.command_planner_model
                },
                command_planner_use_chat_local_model: dto.command_planner_use_chat_local_model,
                command_planner_gguf_path: dto.command_planner_gguf_path,
                use_gguf_for_local_chat: dto.use_gguf_for_local_chat,
                sandbox_shell_in_yolo: dto.sandbox_shell_in_yolo,
                agent_secrets_enabled: dto.agent_secrets_enabled,
                sudo_auth_mode: SudoAuthMode::from_str(&dto.sudo_auth_mode),
            };
        }
    }
    default_agent_settings()
}

fn settings_to_dto(s: &AgentSettings) -> AgentSettingsDto {
    AgentSettingsDto {
        workspace_root: s.workspace_root.to_string_lossy().to_string(),
        trust_mode: match s.trust_mode {
            TrustMode::Standard => "standard".into(),
            TrustMode::Yolo => "yolo".into(),
        },
        home_dir: default_workspace_root().to_string_lossy().to_string(),
        command_planner_enabled: s.command_planner_enabled,
        command_planner_model: s.command_planner_model.clone(),
        command_planner_use_chat_local_model: s.command_planner_use_chat_local_model,
        command_planner_gguf_path: s.command_planner_gguf_path.clone(),
        use_gguf_for_local_chat: s.use_gguf_for_local_chat,
        sandbox_shell_in_yolo: s.sandbox_shell_in_yolo,
        sandbox_level: sandbox_level().to_string(),
        agent_secrets_enabled: s.agent_secrets_enabled,
        sudo_auth_mode: s.sudo_auth_mode.as_str().into(),
    }
}

pub fn save_persisted(paths: &DataPaths, settings: &AgentSettings) -> Result<(), String> {
    let path = paths.agent_settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let dto = settings_to_dto(settings);
    let text = serde_json::to_string_pretty(&dto).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())
}

pub fn init_agent_settings(paths: &DataPaths, state: &AgentSettingsState) {
    let loaded = load_persisted(paths);
    if let Ok(mut guard) = state.inner.lock() {
        *guard = loaded;
    }
}

pub fn read_settings(state: &AgentSettingsState) -> AgentSettings {
    state.inner.lock().unwrap().clone()
}

pub fn settings_dto(state: &AgentSettingsState) -> AgentSettingsDto {
    settings_to_dto(&read_settings(state))
}

pub fn local_gguf_chat_path(settings: &AgentSettings) -> Option<PathBuf> {
    if !settings.use_gguf_for_local_chat {
        return None;
    }
    let p = settings.command_planner_gguf_path.trim();
    if p.is_empty() {
        return None;
    }
    let path = PathBuf::from(p);
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

pub fn should_sandbox_shell(settings: &AgentSettings) -> bool {
    settings.trust_mode == TrustMode::Yolo
        && settings.sandbox_shell_in_yolo
        && sandbox_shell_available()
}

pub fn resolve_path_input(input: &str, workspace: &Path) -> PathBuf {
    let trimmed = input.trim();
    let path = if trimmed.is_empty() || trimmed == "." {
        workspace.to_path_buf()
    } else if Path::new(trimmed).is_absolute() {
        PathBuf::from(trimmed)
    } else {
        workspace.join(trimmed)
    };
    path.canonicalize().unwrap_or(path)
}

pub fn resolve_agent_path_with_token(
    input: &str,
    workspace: &Path,
    trust: TrustMode,
    path_state: &PathTokenState,
    scope: PathScope,
    path_approval_token: Option<&str>,
    path_approved: Option<bool>,
) -> Result<PathBuf, String> {
    crate::agent::tokens::path::resolve_agent_path(
        input,
        workspace,
        trust,
        path_state,
        scope,
        path_approval_token,
        path_approved,
    )
}
