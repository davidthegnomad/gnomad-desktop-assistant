pub use gnomad_core::shell::{check_command_safety, SafetyCheckResult};

use gnomad_core::shell::execute_elevated_command as execute_elevated_inner;

#[tauri::command(rename = "check_command_safety")]
pub fn check_command_safety_cmd(command: &str) -> SafetyCheckResult {
    check_command_safety(command)
}

#[tauri::command]
pub fn execute_elevated_command(
    app: tauri::AppHandle,
    hitl_state: tauri::State<'_, crate::hitl_token::HitlTokenState>,
    settings_state: tauri::State<'_, crate::agent_settings::AgentSettingsState>,
    command: &str,
    approval_token: Option<String>,
    hitl_approved: Option<bool>,
) -> Result<String, String> {
    let paths = crate::core_bridge::data_paths_from_app(&app)?;
    execute_elevated_inner(
        hitl_state.inner(),
        settings_state.inner(),
        &paths,
        command,
        approval_token.as_deref(),
        hitl_approved,
    )
}
