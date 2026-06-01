use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrustMode {
    Standard,
    Yolo,
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
}

pub struct AgentSettingsState {
    pub inner: Mutex<AgentSettings>,
}

#[derive(Debug, Clone)]
pub(crate) struct AgentSettings {
    pub workspace_root: PathBuf,
    pub trust_mode: TrustMode,
    pub command_planner_enabled: bool,
    pub command_planner_model: String,
    pub command_planner_use_chat_local_model: bool,
    pub command_planner_gguf_path: String,
}

impl Default for AgentSettingsState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(AgentSettings {
                workspace_root: default_workspace_root(),
                trust_mode: TrustMode::Standard,
                command_planner_enabled: false,
                command_planner_model: "llama3.2:1b".into(),
                command_planner_use_chat_local_model: false,
                command_planner_gguf_path: String::new(),
            }),
        }
    }
}

pub fn default_workspace_root() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")))
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    Ok(base.join("gnomad").join("agent-settings.json"))
}

fn load_persisted(app: &tauri::AppHandle) -> AgentSettings {
    let path = match settings_path(app) {
        Ok(p) => p,
        Err(_) => return default_agent_settings(),
    };
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
            };
        }
    }
    default_agent_settings()
}

fn default_agent_settings() -> AgentSettings {
    AgentSettings {
        workspace_root: default_workspace_root(),
        trust_mode: TrustMode::Standard,
        command_planner_enabled: false,
        command_planner_model: "llama3.2:1b".into(),
        command_planner_use_chat_local_model: false,
        command_planner_gguf_path: String::new(),
    }
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
    }
}

fn save_persisted(app: &tauri::AppHandle, settings: &AgentSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let dto = settings_to_dto(settings);
    let text = serde_json::to_string_pretty(&dto).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())
}

pub fn init_agent_settings(app: &tauri::AppHandle, state: &AgentSettingsState) {
    let loaded = load_persisted(app);
    if let Ok(mut guard) = state.inner.lock() {
        *guard = loaded;
    }
}

pub fn read_settings(state: &AgentSettingsState) -> AgentSettings {
    state.inner.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_agent_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, AgentSettingsState>,
) -> Result<AgentSettingsDto, String> {
    init_agent_settings(&app, &state);
    let s = read_settings(&state);
    Ok(settings_to_dto(&s))
}

#[tauri::command]
pub fn set_command_planner(
    app: tauri::AppHandle,
    state: tauri::State<'_, AgentSettingsState>,
    enabled: bool,
    model: String,
    use_chat_local_model: bool,
    gguf_path: String,
) -> Result<AgentSettingsDto, String> {
    {
        let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
        guard.command_planner_enabled = enabled;
        guard.command_planner_model = if model.trim().is_empty() {
            "llama3.2:1b".into()
        } else {
            model.trim().to_string()
        };
        guard.command_planner_use_chat_local_model = use_chat_local_model;
        guard.command_planner_gguf_path = gguf_path.trim().to_string();
        save_persisted(&app, &guard)?;
    }
    get_agent_settings(app, state)
}

#[tauri::command]
pub fn set_workspace_root(
    app: tauri::AppHandle,
    state: tauri::State<'_, AgentSettingsState>,
    path: String,
) -> Result<AgentSettingsDto, String> {
    let p = PathBuf::from(path.trim());
    let resolved = if p.is_absolute() {
        p
    } else {
        default_workspace_root().join(p)
    };
    let canonical = resolved
        .canonicalize()
        .map_err(|e| format!("Invalid workspace path: {e}"))?;
    if !canonical.is_dir() {
        return Err("Workspace path must be an existing directory.".into());
    }
    {
        let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
        guard.workspace_root = canonical;
        save_persisted(&app, &guard)?;
    }
    get_agent_settings(app, state)
}

#[tauri::command]
pub fn set_trust_mode(
    app: tauri::AppHandle,
    state: tauri::State<'_, AgentSettingsState>,
    mode: String,
) -> Result<AgentSettingsDto, String> {
    {
        let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
        guard.trust_mode = TrustMode::from_str(&mode);
        save_persisted(&app, &guard)?;
    }
    get_agent_settings(app, state)
}

/// Resolve a user/model path against workspace and trust policy.
pub fn resolve_agent_path(
    input: &str,
    workspace: &Path,
    trust: TrustMode,
    path_approved: bool,
) -> Result<PathBuf, String> {
    let trimmed = input.trim();
    let path = if trimmed.is_empty() || trimmed == "." {
        workspace.to_path_buf()
    } else if Path::new(trimmed).is_absolute() {
        PathBuf::from(trimmed)
    } else {
        workspace.join(trimmed)
    };

    let canonical = path
        .canonicalize()
        .unwrap_or_else(|_| path.clone());

    if trust == TrustMode::Yolo || path_approved {
        return Ok(canonical);
    }

    let workspace_canon = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());

    if canonical.starts_with(&workspace_canon) {
        return Ok(canonical);
    }

    Err(crate::error::into_invoke_err(crate::error::GnomadError::PathPolicy {
        message: "Path is outside workspace.".into(),
        detail: Some(canonical.display().to_string()),
        hint: Some("Approve access once or enable YOLO! in Settings.".into()),
    }))
}
