pub use gnomad_core::agent::tokens::hitl::*;

#[tauri::command]
pub fn issue_hitl_approval_token(command: String, scope: String) -> Result<String, String> {
    let scope = HitlScope::parse(&scope).ok_or_else(|| {
        crate::error::into_invoke_err(crate::error::GnomadError::ShellValidation {
            message: format!("Invalid HITL scope: {scope}"),
            detail: Some("Use shell_run or elevated.".into()),
        })
    })?;
    issue_approval_token(&command, scope)
}
