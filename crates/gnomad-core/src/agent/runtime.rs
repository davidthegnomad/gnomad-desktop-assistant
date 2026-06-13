use serde::Serialize;
use serde_json::Value;

use crate::agent::settings::AgentSettingsState;
use crate::agent::tokens::path::PathScope;
use crate::agent::tokens::{HitlTokenState, PathTokenState};
use crate::agent::tools::{execute_tool, shell_safety, ToolContext, ToolExecutionResult};
use crate::chat::StoredCommandResult;
use crate::config::paths::DataPaths;
use crate::llm::{
    chat_completion_turn, AgentToolCall, ChatCompletionRequest, ChatMessage, ChatTurnResponse,
};
use crate::shell::{
    execute_elevated_command, run_shell_command, run_shell_command_in_session,
    validate_shell_command, ShellOutputSink, ShellRunResult, ShellSessionState,
};

pub const MAX_AGENT_STEPS: usize = 10;

pub trait AgentApprovals: Send + Sync {
    /// Request HITL approval. Use `elevated: true` for sudo/admin commands (Elevated token scope).
    fn request_hitl(&self, command: &str, reason: &str, elevated: bool) -> Option<String>;
    fn request_path(&self, path: &str, reason: &str, scope: PathScope) -> Option<String>;
    /// Live terminal: command is about to run.
    fn terminal_command_start(&self, _command: &str) {}
    /// Live terminal: stdout/stderr chunk while command runs.
    fn terminal_output(&self, _chunk: &str) {}
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentActionRecord {
    pub tool: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_executed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_result: Option<StoredCommandResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentLoopResult {
    pub final_text: String,
    pub actions: Vec<AgentActionRecord>,
}

pub struct AgentRuntime<'a> {
    pub paths: &'a DataPaths,
    pub settings: &'a AgentSettingsState,
    pub hitl: &'a HitlTokenState,
    pub path_tokens: &'a PathTokenState,
    pub approvals: &'a dyn AgentApprovals,
    pub terminal_sink: Option<ShellOutputSink>,
    pub shell_session: Option<&'a ShellSessionState>,
}

impl<'a> AgentRuntime<'a> {
    pub async fn run_loop(
        &self,
        req: ChatCompletionRequest,
        enable_tools: bool,
    ) -> Result<AgentLoopResult, String> {
        let mut api_messages = req.messages.clone();
        let mut actions = Vec::new();
        let mut shell_cwd: Option<String> = None;

        for _step in 0..MAX_AGENT_STEPS {
            let turn_req = ChatCompletionRequest {
                provider: req.provider.clone(),
                model: req.model.clone(),
                messages: api_messages.clone(),
                ollama_url: req.ollama_url.clone(),
                system_context: req.system_context.clone(),
            };
            let turn: ChatTurnResponse =
                chat_completion_turn(turn_req, enable_tools).await?;

            if turn.tool_calls.is_empty() {
                return Ok(AgentLoopResult {
                    final_text: turn.content.unwrap_or_else(|| "Done.".into()).trim().to_string(),
                    actions,
                });
            }

            api_messages.push(ChatMessage {
                role: "assistant".into(),
                content: turn.content.unwrap_or_default(),
                tool_call_id: None,
                tool_calls: Some(turn.tool_calls.clone()),
            });

            for tc in &turn.tool_calls {
                let record = self.run_one_tool(tc, &mut shell_cwd)?;
                let tool_content = serde_json::to_string(
                    &record
                        .tool_data
                        .clone()
                        .unwrap_or_else(|| Value::String(record.label.clone())),
                )
                .unwrap_or_else(|_| record.label.clone());
                api_messages.push(ChatMessage {
                    role: "tool".into(),
                    content: tool_content,
                    tool_call_id: Some(tc.id.clone()),
                    tool_calls: None,
                });
                actions.push(record);
            }
        }

        Ok(AgentLoopResult {
            final_text:
                "Reached the maximum number of agent steps. Review the actions above.".into(),
            actions,
        })
    }

    fn run_one_tool(
        &self,
        tc: &AgentToolCall,
        shell_cwd: &mut Option<String>,
    ) -> Result<AgentActionRecord, String> {
        let args: Value = serde_json::from_str(&tc.arguments).unwrap_or(Value::Object(
            serde_json::Map::new(),
        ));

        if tc.name == "shell_run" {
            return self.run_shell_tool(&args, shell_cwd);
        }

        self.run_fs_tool(&tc.name, &tc.arguments)
    }

    fn run_shell_tool(
        &self,
        args: &Value,
        shell_cwd: &mut Option<String>,
    ) -> Result<AgentActionRecord, String> {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if command.is_empty() {
            return Ok(AgentActionRecord {
                tool: "shell_run".into(),
                label: "Empty shell command".into(),
                command_executed: None,
                command_result: None,
                tool_data: None,
                error: None,
            });
        }
        if !validate_shell_command(&command) {
            return Ok(AgentActionRecord {
                tool: "shell_run".into(),
                label: format!("Invalid shell command: {command}"),
                command_executed: Some(command),
                command_result: None,
                tool_data: None,
                error: Some("validation".into()),
            });
        }

        let safety = shell_safety(&command);
        let mut hitl_token = None;
        if safety.requires_hitl_approval {
            let reason = safety
                .danger_reason
                .clone()
                .unwrap_or_else(|| "Safety review requested.".into());
            let token = self
                .approvals
                .request_hitl(&command, &reason, safety.requires_admin);
            if token.is_none() {
                return Ok(AgentActionRecord {
                    tool: "shell_run".into(),
                    label: "Command blocked by user".into(),
                    command_executed: Some(command),
                    command_result: None,
                    tool_data: None,
                    error: Some("blocked".into()),
                });
            }
            hitl_token = token;
        }

        if safety.requires_admin {
            self.approvals.terminal_command_start(&command);
            match execute_elevated_command(
                self.hitl,
                self.settings,
                self.paths,
                &command,
                hitl_token.as_deref(),
                None,
            ) {
                Ok(stdout) => {
                    self.approvals.terminal_output(&stdout);
                    crate::agent::audit::log_shell_run(self.paths, &command, false, true);
                    let stored = StoredCommandResult {
                        success: true,
                        stdout: stdout.clone(),
                        stderr: String::new(),
                        status_code: Some(0),
                        cwd: shell_cwd.clone(),
                        state: Some("completed".into()),
                        message: Some("Elevated command completed.".into()),
                        duration_ms: None,
                    };
                    return Ok(AgentActionRecord {
                        tool: "shell_run".into(),
                        label: command.clone(),
                        command_executed: Some(command),
                        command_result: Some(stored),
                        tool_data: Some(serde_json::json!({
                            "success": true,
                            "stdout": stdout,
                            "stderr": "",
                            "state": "completed",
                            "message": "Elevated command completed.",
                        })),
                        error: None,
                    });
                }
                Err(err) => {
                    return Ok(AgentActionRecord {
                        tool: "shell_run".into(),
                        label: err.clone(),
                        command_executed: Some(command),
                        command_result: None,
                        tool_data: None,
                        error: Some(err),
                    });
                }
            }
        }

        self.approvals.terminal_command_start(&command);
        let run_result = if let Some(session) = self.shell_session {
            run_shell_command_in_session(
                self.paths,
                self.settings,
                self.hitl,
                session,
                command.clone(),
                shell_cwd.clone(),
                None,
                hitl_token,
                self.terminal_sink.clone(),
                None,
                None,
            )
        } else {
            run_shell_command(
                self.paths,
                self.settings,
                self.hitl,
                command.clone(),
                shell_cwd.clone(),
                None,
                hitl_token,
                self.terminal_sink.clone(),
            )
        };
        match run_result {
            Ok(res) => {
                if !res.cwd.is_empty() {
                    *shell_cwd = Some(res.cwd.clone());
                }
                crate::agent::audit::log_shell_run(self.paths, &command, false, res.success);
                Ok(AgentActionRecord {
                    tool: "shell_run".into(),
                    label: command.clone(),
                    command_executed: Some(command),
                    command_result: Some(shell_result_to_stored(&res)),
                    tool_data: serde_json::to_value(&res).ok(),
                    error: None,
                })
            }
            Err(err) => Ok(AgentActionRecord {
                tool: "shell_run".into(),
                label: err.clone(),
                command_executed: Some(command),
                command_result: None,
                tool_data: None,
                error: Some(err),
            }),
        }
    }

    fn run_fs_tool(&self, name: &str, arguments: &str) -> Result<AgentActionRecord, String> {
        let args: Value = serde_json::from_str(arguments).unwrap_or(Value::Object(
            serde_json::Map::new(),
        ));
        let scope = if name == "fs_write" {
            PathScope::Write
        } else {
            PathScope::Read
        };

        let run = |path_token: Option<String>| {
            execute_tool(
                name,
                arguments,
                ToolContext {
                    paths: self.paths,
                    settings: self.settings,
                    hitl: self.hitl,
                    path_tokens: self.path_tokens,
                    shell_cwd: None,
                    hitl_token: None,
                    path_token,
                },
            )
        };

        match run(None) {
            Ok(res) => Ok(action_from_tool(name, &res)),
            Err(err) => {
                if is_path_policy_err(&err) {
                    let path_hint = args
                        .get("path")
                        .or_else(|| args.get("query"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let token = self.approvals.request_path(
                        path_hint,
                        &err,
                        scope,
                    );
                    if let Some(tok) = token {
                        return match run(Some(tok)) {
                            Ok(res) => Ok(action_from_tool(name, &res)),
                            Err(e) => Ok(fs_error_record(name, &e)),
                        };
                    }
                    return Ok(AgentActionRecord {
                        tool: name.into(),
                        label: "Path access blocked by user".into(),
                        command_executed: None,
                        command_result: None,
                        tool_data: None,
                        error: Some("blocked".into()),
                    });
                }
                Ok(fs_error_record(name, &err))
            }
        }
    }
}

fn is_path_policy_err(err: &str) -> bool {
    err.contains("path_policy") || err.contains("outside workspace") || err.contains("Path policy")
}

fn action_from_tool(name: &str, res: &ToolExecutionResult) -> AgentActionRecord {
    AgentActionRecord {
        tool: name.into(),
        label: format!("{name} completed"),
        command_executed: None,
        command_result: None,
        tool_data: Some(res.data.clone()),
        error: res.error.clone(),
    }
}

fn fs_error_record(name: &str, err: &str) -> AgentActionRecord {
    AgentActionRecord {
        tool: name.into(),
        label: err.to_string(),
        command_executed: None,
        command_result: None,
        tool_data: None,
        error: Some(err.to_string()),
    }
}

fn shell_result_to_stored(res: &ShellRunResult) -> StoredCommandResult {
    StoredCommandResult {
        success: res.success,
        stdout: res.stdout.clone(),
        stderr: res.stderr.clone(),
        status_code: res.status_code,
        cwd: Some(res.cwd.clone()),
        state: Some(res.state.clone()),
        message: Some(res.message.clone()),
        duration_ms: res.duration_ms,
    }
}
