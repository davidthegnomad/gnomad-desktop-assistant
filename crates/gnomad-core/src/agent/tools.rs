use serde::Serialize;
use serde_json::Value;

use crate::agent::fs::{
    fs_list_inner, fs_read_inner, fs_search_inner, fs_write_inner, FsListResult, FsReadResult,
    FsSearchResult, FsWriteResult,
};
use crate::agent::settings::{read_settings, AgentSettingsState};
use crate::agent::tokens::{HitlTokenState, PathTokenState};
use crate::agent::TrustMode;
use crate::config::agent_secrets;
use crate::config::paths::DataPaths;
use crate::error::{into_invoke_err, GnomadError};
use crate::shell::{check_command_safety, run_shell_command, ShellRunResult};

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

pub struct ToolContext<'a> {
    pub paths: &'a DataPaths,
    pub settings: &'a AgentSettingsState,
    pub hitl: &'a HitlTokenState,
    pub path_tokens: &'a PathTokenState,
    pub shell_cwd: Option<String>,
    pub hitl_token: Option<String>,
    pub path_token: Option<String>,
}

pub fn execute_tool(name: &str, arguments: &str, ctx: ToolContext<'_>) -> Result<ToolExecutionResult, String> {
    let tool = name.trim().to_string();
    let args = parse_args(arguments)?;
    crate::agent::audit::log_action(ctx.paths, "tool", &format!("{tool} {arguments}"));

    match tool.as_str() {
        "shell_run" | "shell" => {
            let command = args
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "shell_run requires command".to_string())?;
            let res: ShellRunResult = run_shell_command(
                ctx.paths,
                ctx.settings,
                ctx.hitl,
                command.to_string(),
                ctx.shell_cwd.clone(),
                None,
                ctx.hitl_token.clone(),
                None,
            )?;
            crate::agent::audit::log_shell_run(ctx.paths, command, false, res.success);
            Ok(ToolExecutionResult {
                tool,
                success: res.success,
                data: serde_json::to_value(res).map_err(|e| e.to_string())?,
                error: None,
            })
        }
        "workspace_info" => {
            let s = read_settings(ctx.settings);
            let secret_names: Vec<String> = if s.agent_secrets_enabled {
                agent_secrets::list_statuses(ctx.paths)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|st| st.name)
                    .collect()
            } else {
                Vec::new()
            };
            Ok(ToolExecutionResult {
                tool,
                success: true,
                data: serde_json::json!({
                    "workspaceRoot": s.workspace_root.to_string_lossy(),
                    "trustMode": match s.trust_mode {
                        TrustMode::Standard => "standard",
                        TrustMode::Yolo => "yolo",
                    },
                    "shellCwd": ctx.shell_cwd.clone().unwrap_or_else(|| s.workspace_root.to_string_lossy().to_string()),
                    "agentSecretsEnabled": s.agent_secrets_enabled,
                    "agentSecretNames": secret_names,
                    "sudoAuthMode": s.sudo_auth_mode.as_str(),
                }),
                error: None,
            })
        }
        "fs_list" => {
            let path = args.get("path").and_then(|v| v.as_str()).map(String::from);
            let res: FsListResult = fs_list_inner(
                ctx.path_tokens,
                ctx.settings,
                path,
                ctx.path_token.as_deref(),
                None,
            )?;
            Ok(ToolExecutionResult {
                tool,
                success: true,
                data: serde_json::to_value(res).map_err(|e| e.to_string())?,
                error: None,
            })
        }
        "fs_read" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "fs_read requires path".to_string())?;
            let res: FsReadResult = fs_read_inner(
                ctx.path_tokens,
                ctx.settings,
                path.to_string(),
                ctx.path_token.as_deref(),
                None,
            )?;
            Ok(ToolExecutionResult {
                tool,
                success: true,
                data: serde_json::to_value(res).map_err(|e| e.to_string())?,
                error: None,
            })
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
            let res: FsWriteResult = fs_write_inner(
                ctx.paths,
                ctx.path_tokens,
                ctx.settings,
                path.to_string(),
                content.to_string(),
                ctx.path_token.as_deref(),
                None,
            )?;
            Ok(ToolExecutionResult {
                tool,
                success: true,
                data: serde_json::to_value(res).map_err(|e| e.to_string())?,
                error: None,
            })
        }
        "fs_search" => {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "fs_search requires query".to_string())?;
            let path = args.get("path").and_then(|v| v.as_str());
            let res: FsSearchResult = fs_search_inner(
                ctx.path_tokens,
                ctx.settings,
                query.to_string(),
                path.map(String::from),
                ctx.path_token.as_deref(),
                None,
            )?;
            Ok(ToolExecutionResult {
                tool,
                success: true,
                data: serde_json::to_value(res).map_err(|e| e.to_string())?,
                error: None,
            })
        }
        other => Err(into_invoke_err(GnomadError::Internal {
            message: format!("Unknown agent tool: {other}"),
            detail: None,
        })),
    }
}

pub fn shell_safety(command: &str) -> crate::shell::SafetyCheckResult {
    check_command_safety(command)
}
