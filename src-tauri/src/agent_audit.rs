use gnomad_core::agent::audit::{log_action as log_action_inner, log_shell_run as log_shell_run_inner};

pub fn log_action(app: &tauri::AppHandle, kind: &str, detail: &str) {
    if let Ok(paths) = crate::core_bridge::data_paths_from_app(app) {
        log_action_inner(&paths, kind, detail);
    }
}

pub fn log_shell_run(app: &tauri::AppHandle, command: &str, sandboxed: bool, success: bool) {
    if let Ok(paths) = crate::core_bridge::data_paths_from_app(app) {
        log_shell_run_inner(&paths, command, sandboxed, success);
    }
}

#[tauri::command]
pub fn append_agent_audit(
    app: tauri::AppHandle,
    kind: String,
    detail: String,
) -> Result<(), String> {
    log_action(&app, &kind, &detail);
    Ok(())
}
