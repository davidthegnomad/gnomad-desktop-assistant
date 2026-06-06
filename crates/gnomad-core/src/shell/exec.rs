use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;

use crate::agent::tokens::hitl::{enforce_hitl, HitlScope, HitlTokenState};
use crate::agent::AgentSettingsState;
use crate::config::agent_secrets;
use crate::config::paths::DataPaths;
use crate::error::{into_invoke_err, GnomadError};
use crate::shell::rules::looks_like_shell_command;

pub type ShellOutputSink = Arc<dyn Fn(&str) + Send + Sync>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellRunResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<i32>,
    pub cwd: String,
    pub state: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

pub fn validate_shell_command(command: &str) -> bool {
    looks_like_shell_command(command.trim())
}

pub fn run_shell_command(
    paths: &DataPaths,
    settings_state: &AgentSettingsState,
    hitl_state: &HitlTokenState,
    command: String,
    cwd: Option<String>,
    hitl_approved: Option<bool>,
    approval_token: Option<String>,
    on_output: Option<ShellOutputSink>,
) -> Result<ShellRunResult, String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err(into_invoke_err(GnomadError::ShellValidation {
            message: "Command cannot be empty.".into(),
            detail: None,
        }));
    }

    if !looks_like_shell_command(trimmed) {
        let preview: String = trimmed.chars().take(120).collect();
        return Err(into_invoke_err(GnomadError::ShellValidation {
            message: format!("Not a valid shell command: \"{preview}\""),
            detail: Some(
                "Use a real CLI line, not English prose (e.g. ls -la or xdg-open https://…).".into(),
            ),
        }));
    }

    enforce_hitl(
        hitl_state,
        trimmed,
        HitlScope::ShellRun,
        approval_token.as_deref(),
        hitl_approved,
    )?;

    let settings = crate::agent::read_settings(settings_state);
    let work_dir = resolve_cwd(cwd.as_deref(), &settings.workspace_root);
    let started = Instant::now();

    #[cfg(windows)]
    {
        let output = Command::new("cmd")
            .args(["/C", trimmed])
            .current_dir(&work_dir)
            .output()
            .map_err(|e| {
                into_invoke_err(GnomadError::ShellExecution {
                    message: "Failed to start shell command.".into(),
                    detail: Some(e.to_string()),
                })
            })?;
        return finish_output(output.status.code(), output.status.success(), output.stdout, output.stderr, work_dir, started);
    }

    #[cfg(not(windows))]
    {
        let mut cmd = Command::new("sh");
        cmd.arg("-lc")
            .arg(trimmed)
            .current_dir(&work_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (key, value) in agent_secrets::shell_env_map(paths, settings.agent_secrets_enabled) {
            cmd.env(key, value);
        }

        if let Some(sink) = on_output {
            let mut child = cmd.spawn().map_err(|e| {
                into_invoke_err(GnomadError::ShellExecution {
                    message: "Failed to start shell command.".into(),
                    detail: Some(e.to_string()),
                })
            })?;
            let stdout_pipe = child.stdout.take();
            let stderr_pipe = child.stderr.take();
            let mut stdout_buf = Vec::new();
            let mut stderr_buf = Vec::new();

            if let Some(out) = stdout_pipe {
                let sink_out = Arc::clone(&sink);
                let mut out = out;
                stream_pipe(&mut out, &mut stdout_buf, sink_out);
            }
            if let Some(err) = stderr_pipe {
                let sink_err = Arc::clone(&sink);
                let mut err = err;
                stream_pipe(&mut err, &mut stderr_buf, sink_err);
            }

            let status = child.wait().map_err(|e| {
                into_invoke_err(GnomadError::ShellExecution {
                    message: "Failed to wait for shell command.".into(),
                    detail: Some(e.to_string()),
                })
            })?;
            return finish_output(
                status.code(),
                status.success(),
                stdout_buf,
                stderr_buf,
                work_dir,
                started,
            );
        }

        let output = cmd.output().map_err(|e| {
            into_invoke_err(GnomadError::ShellExecution {
                message: "Failed to start shell command.".into(),
                detail: Some(e.to_string()),
            })
        })?;
        return finish_output(
            output.status.code(),
            output.status.success(),
            output.stdout,
            output.stderr,
            work_dir,
            started,
        );
    }
}

#[cfg(not(windows))]
fn stream_pipe<R: Read>(reader: &mut R, capture: &mut Vec<u8>, sink: ShellOutputSink) {
    let mut buf = [0u8; 4096];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                capture.extend_from_slice(&buf[..n]);
                let chunk = String::from_utf8_lossy(&buf[..n]);
                if !chunk.is_empty() {
                    sink(&chunk);
                }
            }
            Err(_) => break,
        }
    }
}

fn finish_output(
    status_code: Option<i32>,
    success: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    work_dir: PathBuf,
    started: Instant,
) -> Result<ShellRunResult, String> {
    let duration_ms = started.elapsed().as_millis() as u64;
    let stdout = String::from_utf8_lossy(&stdout).to_string();
    let stderr = String::from_utf8_lossy(&stderr).to_string();
    let cwd_str = work_dir.to_string_lossy().to_string();
    Ok(ShellRunResult {
        success,
        stdout,
        stderr,
        status_code,
        cwd: cwd_str,
        state: if success {
            "completed".into()
        } else {
            "failed".into()
        },
        message: if success {
            "Command completed.".into()
        } else {
            format!("Command exited with code {}.", status_code.unwrap_or(-1))
        },
        duration_ms: Some(duration_ms),
    })
}

fn resolve_cwd(cwd: Option<&str>, workspace: &Path) -> PathBuf {
    if let Some(c) = cwd {
        let trimmed = c.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            if p.is_absolute() {
                return p;
            }
            return workspace.join(p);
        }
    }
    workspace.to_path_buf()
}
