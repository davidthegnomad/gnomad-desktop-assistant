use std::io::Write;
use std::process::{Command, Stdio};

use crate::agent::settings::{read_settings, AgentSettingsState, SudoAuthMode};
use crate::agent::tokens::hitl::{enforce_hitl, HitlScope, HitlTokenState};
use crate::config::agent_secrets;
use crate::config::paths::DataPaths;
use crate::error::{into_invoke_err, GnomadError};

pub use super::rules::{
    check_command_safety, elevation_command_rejected, looks_like_file_write,
    normalize_elevated_command, parse_simple_argv, SafetyCheckResult,
};

pub fn execute_elevated_command(
    hitl_state: &HitlTokenState,
    settings_state: &AgentSettingsState,
    paths: &DataPaths,
    command: &str,
    approval_token: Option<&str>,
    hitl_approved: Option<bool>,
) -> Result<String, String> {
    let _safety = check_command_safety(command);

    enforce_hitl(
        hitl_state,
        command,
        HitlScope::Elevated,
        approval_token,
        hitl_approved,
    )?;

    if let Some(reason) = elevation_command_rejected(command) {
        return Err(into_invoke_err(GnomadError::SafetyBlocked {
            message: reason,
            detail: Some(command.chars().take(200).collect()),
            hint: Some(
                "Prefer fs_write in the agent, or run elevated commands manually in Terminal."
                    .into(),
            ),
        }));
    }

    let settings = read_settings(settings_state);

    #[cfg(target_os = "macos")]
    {
        let inner = normalize_elevated_command(command);
        let argv = parse_simple_argv(&inner)?;
        let mut script = String::new();
        for (i, arg) in argv.iter().enumerate() {
            if i > 0 {
                script.push(' ');
            }
            script.push_str(&shell_escape_applescript(arg));
        }
        let apple_script = format!("do shell script {script} with administrator privileges");
        let output = Command::new("osascript").arg("-e").arg(&apple_script).output();
        return match output {
            Ok(out) => {
                if out.status.success() {
                    Ok(String::from_utf8_lossy(&out.stdout).to_string())
                } else {
                    Err(into_invoke_err(GnomadError::ShellExecution {
                        message: "Elevated command failed.".into(),
                        detail: Some(String::from_utf8_lossy(&out.stderr).to_string()),
                    }))
                }
            }
            Err(e) => Err(into_invoke_err(GnomadError::Internal {
                message: "Failed to trigger administrator authentication.".into(),
                detail: Some(e.to_string()),
            })),
        };
    }

    #[cfg(target_os = "linux")]
    {
        if settings.agent_secrets_enabled && settings.sudo_auth_mode == SudoAuthMode::StoredPassword
        {
            let password = agent_secrets::sudo_password(paths).ok_or_else(|| {
                into_invoke_err(GnomadError::ShellExecution {
                    message: "Stored sudo password is not configured.".into(),
                    detail: Some(
                        "Enable agent secrets and save your sudo password in Settings.".into(),
                    ),
                })
            })?;
            return execute_linux_sudo_password(&password, command);
        }
        return execute_linux_polkit(command);
    }

    #[cfg(target_os = "windows")]
    {
        return Err(into_invoke_err(GnomadError::ElevationUnsupported {
            message: "Administrative elevation is not supported on Windows.".into(),
            hint: Some("Run the command in an elevated PowerShell or Terminal window.".into()),
        }));
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(into_invoke_err(GnomadError::ElevationUnsupported {
            message: "Administrative elevation is not supported on this platform.".into(),
            hint: None,
        }))
    }
}

#[cfg(target_os = "linux")]
fn execute_linux_polkit(command: &str) -> Result<String, String> {
    let inner = normalize_elevated_command(command);
    let argv = parse_simple_argv(&inner)?;
    let output = Command::new("pkexec").args(&argv).output();
    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(String::from_utf8_lossy(&out.stdout).to_string())
            } else {
                Err(into_invoke_err(GnomadError::ShellExecution {
                    message: "Elevated command failed.".into(),
                    detail: Some(String::from_utf8_lossy(&out.stderr).to_string()),
                }))
            }
        }
        Err(e) => Err(into_invoke_err(GnomadError::Internal {
            message: "Failed to trigger pkexec.".into(),
            detail: Some(e.to_string()),
        })),
    }
}

#[cfg(target_os = "linux")]
fn execute_linux_sudo_password(password: &str, command: &str) -> Result<String, String> {
    let inner = normalize_elevated_command(command);
    if inner.is_empty() {
        return Err(into_invoke_err(GnomadError::ShellValidation {
            message: "Command cannot be empty.".into(),
            detail: None,
        }));
    }
    let mut child = Command::new("sudo")
        .args(["-S", "-p", ""])
        .arg("sh")
        .arg("-lc")
        .arg(&inner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            into_invoke_err(GnomadError::Internal {
                message: "Failed to start sudo.".into(),
                detail: Some(e.to_string()),
            })
        })?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(password.as_bytes());
        let _ = stdin.write_all(b"\n");
    }
    let output = child.wait_with_output().map_err(|e| {
        into_invoke_err(GnomadError::Internal {
            message: "Failed to wait for sudo.".into(),
            detail: Some(e.to_string()),
        })
    })?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(into_invoke_err(GnomadError::ShellExecution {
            message: "Elevated command failed.".into(),
            detail: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        }))
    }
}

#[cfg(target_os = "macos")]
fn shell_escape_applescript(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}
