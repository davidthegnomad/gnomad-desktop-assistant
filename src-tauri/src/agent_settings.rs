pub use gnomad_core::agent::settings::{
    default_workspace_root, local_gguf_chat_path, read_settings, resolve_agent_path_with_token,
    should_sandbox_shell, AgentSettingsDto, AgentSettingsState, TrustMode,
};

use gnomad_core::agent::settings::{init_agent_settings as init_inner, save_persisted, settings_dto};

fn paths_from_app(app: &tauri::AppHandle) -> Result<gnomad_core::config::paths::DataPaths, String> {
    crate::core_bridge::data_paths_from_app(app)
}

pub fn init_agent_settings(app: &tauri::AppHandle, state: &AgentSettingsState) {
    if let Ok(paths) = paths_from_app(app) {
        init_inner(&paths, state);
    }
}

#[tauri::command]
pub fn get_agent_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, AgentSettingsState>,
) -> Result<AgentSettingsDto, String> {
    init_agent_settings(&app, &state);
    Ok(settings_dto(state.inner()))
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
    let paths = paths_from_app(&app)?;
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
        save_persisted(&paths, &guard)?;
    }
    get_agent_settings(app, state)
}

#[tauri::command]
pub fn set_workspace_root(
    app: tauri::AppHandle,
    state: tauri::State<'_, AgentSettingsState>,
    path: String,
) -> Result<AgentSettingsDto, String> {
    let paths = paths_from_app(&app)?;
    let p = std::path::PathBuf::from(path.trim());
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
        save_persisted(&paths, &guard)?;
    }
    get_agent_settings(app, state)
}

#[tauri::command]
pub fn set_trust_mode(
    app: tauri::AppHandle,
    state: tauri::State<'_, AgentSettingsState>,
    mode: String,
) -> Result<AgentSettingsDto, String> {
    let paths = paths_from_app(&app)?;
    {
        let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
        guard.trust_mode = TrustMode::from_str(&mode);
        save_persisted(&paths, &guard)?;
    }
    get_agent_settings(app, state)
}

#[tauri::command]
pub fn set_agent_experimental_flags(
    app: tauri::AppHandle,
    state: tauri::State<'_, AgentSettingsState>,
    use_gguf_for_local_chat: bool,
    sandbox_shell_in_yolo: bool,
) -> Result<AgentSettingsDto, String> {
    let paths = paths_from_app(&app)?;
    {
        let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
        guard.use_gguf_for_local_chat = use_gguf_for_local_chat;
        guard.sandbox_shell_in_yolo = sandbox_shell_in_yolo;
        save_persisted(&paths, &guard)?;
    }
    get_agent_settings(app, state)
}
