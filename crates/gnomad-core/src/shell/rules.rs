use crate::error::{into_invoke_err, GnomadError};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SafetyCheckResult {
    pub is_safe: bool,
    pub requires_hitl_approval: bool,
    pub requires_admin: bool,
    pub danger_reason: Option<String>,
    #[serde(default)]
    pub suggest_agent_fs: bool,
}

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

/// Strip leading `sudo` / `pkexec` wrappers before elevated execution.
pub fn normalize_elevated_command(command: &str) -> String {
    let mut t = command.trim().to_string();
    loop {
        let lower = t.to_lowercase();
        if lower.starts_with("sudo ") {
            t = t[5..].trim().to_string();
        } else if lower.starts_with("pkexec ") {
            t = t[7..].trim().to_string();
        } else {
            break;
        }
    }
    t
}

pub fn parse_simple_argv(command: &str) -> Result<Vec<String>, String> {
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

pub fn looks_like_shell_command(command: &str) -> bool {
    let t = command.trim();
    if t.is_empty() || t.len() > 2000 {
        return false;
    }
    if is_probably_natural_language(t) {
        return false;
    }
    let first = t.split_whitespace().next().unwrap_or("");
    !first.is_empty()
        && first.len() <= 128
        && first.chars().all(|c| {
            c.is_ascii_alphanumeric()
                || matches!(c, '.' | '-' | '_' | '/' | '\\' | '@' | '$' | '~')
        })
}

fn is_probably_natural_language(command: &str) -> bool {
    let lower = command.to_lowercase();
    let prose_markers = [
        "please ",
        "could you",
        "can you",
        "would you",
        "i want",
        "i need",
        "help me",
        "how do",
        "what is",
    ];
    prose_markers.iter().any(|m| lower.contains(m))
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

    #[test]
    fn flags_sudo_as_hitl() {
        let r = check_command_safety("sudo apt update");
        assert!(r.requires_hitl_approval);
        assert!(r.requires_admin);
    }
}
