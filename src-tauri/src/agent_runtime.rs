use crate::agent_audit;
use crate::agent_fs::{self, FsListResult, FsReadResult, FsSearchResult, FsWriteResult};
use crate::agent_settings::{read_settings, AgentSettingsState};
use crate::error::{into_invoke_err, GnomadError};
use crate::path_token::{PathScope, PathTokenState};
use crate::shell_session::{run_shell_command, ShellSessionState};
use serde::Serialize;
use serde_json::Value;
use tauri::AppHandle;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolExecutionResult {
    pub tool: String,
    pub success: bool,
    pub data: Value,
    pub error: Option<String>,
}

fn parse_args(arguments: &str) -> Result<Value, String> {
    if arguments.trim().is_empty() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    serde_json::from_str(arguments).map_err(|e| format!("Invalid tool arguments JSON: {e}"))
}

fn fs_scope(tool: &str) -> PathScope {
    if tool == "fs_write" {
        PathScope::Write
    } else {
        PathScope::Read
    }
}

#[tauri::command]
pub fn agent_execute_tool(
    app: AppHandle,
    shell_state: tauri::State<'_, ShellSessionState>,
    hitl_state: tauri::State<'_, crate::hitl_token::HitlTokenState>,
    path_state: tauri::State<'_, PathTokenState>,
    settings_state: tauri::State<'_, AgentSettingsState>,
    name: String,
    arguments: String,
    hitl_approved: Option<bool>,
    approval_token: Option<String>,
    path_approval_token: Option<String>,
    path_approved: Option<bool>,
    cwd: Option<String>,
) -> Result<ToolExecutionResult, String> {
    let tool = name.trim().to_string();
    let args = parse_args(&arguments)?;
    let path_token = path_approval_token.as_deref();

    agent_audit::log_action(&app, "tool", &format!("{tool} {arguments}"));

    let result = match tool.as_str() {
        "shell_run" | "shell" => {
            let command = args
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "shell_run requires command".to_string())?;
            let res = run_shell_command(
                &app,
                shell_state.inner(),
                settings_state.inner(),
                hitl_state.inner(),
                command.to_string(),
                cwd,
                None,
                None,
                hitl_approved,
                approval_token,
            )?;
            ToolExecutionResult {
                tool: tool.clone(),
                success: res.success,
                data: serde_json::to_value(res).map_err(|e| e.to_string())?,
                error: None,
            }
        }
        "workspace_info" => {
            let s = read_settings(settings_state.inner());
            let (shell_cwd, shell_active) =
                crate::shell_session::session_cwd_snapshot(shell_state.inner());
            ToolExecutionResult {
                tool: tool.clone(),
                success: true,
                data: serde_json::json!({
                    "workspaceRoot": s.workspace_root.to_string_lossy(),
                    "trustMode": match s.trust_mode {
                        crate::agent_settings::TrustMode::Standard => "standard",
                        crate::agent_settings::TrustMode::Yolo => "yolo",
                    },
                    "shellCwd": shell_cwd,
                    "shellActive": shell_active,
                }),
                error: None,
            }
        }
        "fs_list" => {
            let path = args.get("path").and_then(|v| v.as_str()).map(String::from);
            let res: FsListResult = agent_fs::fs_list_inner(
                path_state.inner(),
                settings_state.inner(),
                path,
                path_token,
                path_approved,
            )?;
            ToolExecutionResult {
                tool: tool.clone(),
                success: true,
                data: serde_json::to_value(res).map_err(|e| e.to_string())?,
                error: None,
            }
        }
        "fs_read" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "fs_read requires path".to_string())?;
            let res: FsReadResult = agent_fs::fs_read_inner(
                path_state.inner(),
                settings_state.inner(),
                path.to_string(),
                path_token,
                path_approved,
            )?;
            ToolExecutionResult {
                tool: tool.clone(),
                success: true,
                data: serde_json::to_value(res).map_err(|e| e.to_string())?,
                error: None,
            }
        }
        "fs_write" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "fs_write requires path".to_string())?;
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let res: FsWriteResult = agent_fs::fs_write_inner(
                &app,
                path_state.inner(),
                settings_state.inner(),
                path.to_string(),
                content.to_string(),
                path_token,
                path_approved,
            )?;
            ToolExecutionResult {
                tool: tool.clone(),
                success: true,
                data: serde_json::to_value(res).map_err(|e| e.to_string())?,
                error: None,
            }
        }
        "fs_search" => {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "fs_search requires query".to_string())?;
            let path = args.get("path").and_then(|v| v.as_str());
            let res: FsSearchResult = agent_fs::fs_search_inner(
                path_state.inner(),
                settings_state.inner(),
                query.to_string(),
                path.map(String::from),
                path_token,
                path_approved,
            )?;
            ToolExecutionResult {
                tool: tool.clone(),
                success: true,
                data: serde_json::to_value(res).map_err(|e| e.to_string())?,
                error: None,
            }
        }
        other => {
            let _ = fs_scope(other);
            return Err(into_invoke_err(GnomadError::Internal {
                message: format!("Unknown agent tool: {other}"),
                detail: None,
            }));
        }
    };

    Ok(result)
}
