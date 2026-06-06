pub use gnomad_core::agent::tokens::path::*;

#[tauri::command]
pub fn issue_path_gate_token(
    path_state: tauri::State<'_, PathTokenState>,
    settings_state: tauri::State<'_, crate::agent_settings::AgentSettingsState>,
    path: String,
    scope: String,
) -> Result<String, String> {
    let _ = path_state.inner();
    let settings = crate::agent_settings::read_settings(settings_state.inner());
    let scope = PathScope::parse(&scope).ok_or_else(|| {
        crate::error::into_invoke_err(crate::error::GnomadError::PathPolicy {
            message: format!("Invalid path scope: {scope}"),
            detail: Some("Use read or write.".into()),
            hint: None,
        })
    })?;
    issue_path_approval_token(
        &path,
        scope,
        &settings.workspace_root,
        settings.trust_mode,
    )
}
