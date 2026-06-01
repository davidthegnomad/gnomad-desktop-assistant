use crate::error::{into_invoke_err, GnomadError};
use std::process::Command;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SafetyCheckResult {
    pub is_safe: bool,
    pub requires_hitl_approval: bool,
    pub requires_admin: bool,
    pub danger_reason: Option<String>,
    #[serde(default)]
    pub suggest_agent_fs: bool,
}

/// Detect shell patterns that are better handled via agent fs_write + Path Gate.
pub fn looks_like_file_write(command: &str) -> bool {
    let t = command.trim();
    if t.is_empty() {
        return false;
    }
    if t.contains('>') {
        return true;
    }
    let lower = t.to_lowercase();
    if lower.starts_with("tee ") || lower.contains(" tee ") {
        return true;
    }
    if lower.contains(" sed -i") || lower.starts_with("sed -i") {
        return true;
    }
    let parts: Vec<&str> = lower.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }
    matches!(parts[0], "cp" | "mv" | "install" | "touch")
}

/// Block shell metacharacters that enable injection in elevated paths.
pub fn elevation_command_rejected(command: &str) -> Option<String> {
    let t = command.trim();
    if t.is_empty() {
        return Some("Command cannot be empty.".into());
    }
    if t.len() > 4000 {
        return Some("Command is too long for elevated execution.".into());
    }
    if t.contains('\n') || t.contains('\r') || t.contains('\0') {
        return Some("Elevated commands cannot contain newlines.".into());
    }
    for bad in ["$(", "`", "${", "<(", ">("] {
        if t.contains(bad) {
            return Some("Elevated commands cannot contain command substitution.".into());
        }
    }
    for bad in ["&&", "||", ";", "|"] {
        if t.contains(bad) {
            return Some(
                "Elevated commands cannot chain operators (;, &&, ||, |). Run one command at a time."
                    .into(),
            );
        }
    }
    None
}

/// Parse a simple single command into argv (no shell). Fails on shell operators.
fn parse_simple_argv(command: &str) -> Result<Vec<String>, String> {
    if elevation_command_rejected(command).is_some() {
        return Err(into_invoke_err(GnomadError::SafetyBlocked {
            message: "Command not allowed for elevation.".into(),
            detail: Some(command.chars().take(200).collect()),
            hint: Some("Use agent file tools for writes, or run manually in Terminal.".into()),
        }));
    }
    let parts: Vec<String> = command
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    if parts.is_empty() {
        return Err(into_invoke_err(GnomadError::ShellValidation {
            message: "Command cannot be empty.".into(),
            detail: None,
        }));
    }
    Ok(parts)
}

#[tauri::command]
pub fn check_command_safety(command: &str) -> SafetyCheckResult {
    let lower_cmd = command.to_lowercase();

    let sub_commands: Vec<&str> = lower_cmd.split(|c| c == ';' || c == '&' || c == '|').collect();

    let mut requires_admin = false;
    let mut requires_hitl = false;
    let mut danger_reason = None;
    let suggest_agent_fs = looks_like_file_write(command);

    let admin_commands = [
        "sudo", "pkexec", "apt-get", "dnf", "yum", "pacman", "brew-services", "launchctl",
        "systemctl",
    ];

    for sub_cmd in sub_commands {
        let parts: Vec<&str> = sub_cmd.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let base_cmd = parts[0];

        if admin_commands.contains(&base_cmd) || parts.contains(&"sudo") || parts.contains(&"pkexec")
        {
            requires_admin = true;
            requires_hitl = true;
            danger_reason = Some("Administrative privilege escalation requested.".to_string());
        }

        if base_cmd == "rm" && (parts.contains(&"-rf") || parts.contains(&"-f") || parts.contains(&"-r"))
        {
            requires_hitl = true;
            danger_reason = Some("Destructive file deletion (rm -rf/rm -f) detected.".to_string());
        }

        if ["dd", "mkfs", "fdisk", "parted"].contains(&base_cmd) {
            requires_hitl = true;
            requires_admin = true;
            danger_reason =
                Some("Low-level partition or disk write operation detected.".to_string());
        }

        if ["chmod", "chown"].contains(&base_cmd)
            && (parts.contains(&"777") || parts.contains(&"root") || parts.contains(&"-r"))
        {
            requires_hitl = true;
            danger_reason = Some("Permission/ownership override detected.".to_string());
        }

        if base_cmd == "mv"
            && (parts.contains(&"/etc")
                || parts.contains(&"/var")
                || parts.contains(&"/system")
                || parts.contains(&"/usr"))
        {
            requires_hitl = true;
            requires_admin = true;
            danger_reason = Some("Moving files into core system directories detected.".to_string());
        }
    }

    SafetyCheckResult {
        is_safe: true,
        requires_hitl_approval: requires_hitl,
        requires_admin,
        danger_reason,
        suggest_agent_fs,
    }
}

#[tauri::command]
pub fn execute_elevated_command(
    hitl_state: tauri::State<'_, crate::hitl_token::HitlTokenState>,
    command: &str,
    approval_token: Option<String>,
    hitl_approved: Option<bool>,
) -> Result<String, String> {
    let _safety = check_command_safety(command);

    crate::hitl_token::enforce_hitl(
        hitl_state.inner(),
        command,
        crate::hitl_token::HitlScope::Elevated,
        approval_token.as_deref(),
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

    #[cfg(target_os = "macos")]
    {
        let argv = parse_simple_argv(command)?;
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
        let argv = parse_simple_argv(command)?;
        let mut cmd = Command::new("pkexec");
        cmd.args(&argv);
        let output = cmd.output();

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
                message: "Failed to trigger pkexec.".into(),
                detail: Some(e.to_string()),
            })),
        };
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

#[cfg(target_os = "macos")]
fn shell_escape_applescript(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_injection_patterns() {
        assert!(elevation_command_rejected("echo $(whoami)").is_some());
        assert!(elevation_command_rejected("ls && rm -rf /").is_some());
        assert!(elevation_command_rejected("echo ok").is_none());
    }

    #[test]
    fn detects_file_write_patterns() {
        assert!(looks_like_file_write("echo hello > file.txt"));
        assert!(looks_like_file_write("tee /tmp/out"));
        assert!(looks_like_file_write("cp src dest"));
        assert!(!looks_like_file_write("ls -la"));
    }
}
