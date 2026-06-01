use std::process::Command;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SafetyCheckResult {
    pub is_safe: bool,
    pub requires_hitl_approval: bool,
    pub requires_admin: bool,
    pub danger_reason: Option<String>,
}

#[tauri::command]
pub fn check_command_safety(command: &str) -> SafetyCheckResult {
    let lower_cmd = command.to_lowercase();
    
    // Split the command string by shell operators to catch chained shell exploits
    let sub_commands: Vec<&str> = lower_cmd.split(|c| c == ';' || c == '&' || c == '|').collect();
    
    let mut requires_admin = false;
    let mut requires_hitl = false;
    let mut danger_reason = None;

    let admin_commands = ["sudo", "pkexec", "apt-get", "dnf", "yum", "pacman", "brew-services", "launchctl", "systemctl"];

    for sub_cmd in sub_commands {
        let parts: Vec<&str> = sub_cmd.split_whitespace().collect();
        if parts.is_empty() { continue; }
        
        let base_cmd = parts[0];
        
        // Check if any sub-command requires admin elevation
        if admin_commands.contains(&base_cmd) || parts.contains(&"sudo") || parts.contains(&"pkexec") {
            requires_admin = true;
            requires_hitl = true;
            danger_reason = Some("Administrative privilege escalation requested.".to_string());
        }

        // Destructive rm -rf or rm -f
        if base_cmd == "rm" && (parts.contains(&"-rf") || parts.contains(&"-f") || parts.contains(&"-r")) {
            requires_hitl = true;
            danger_reason = Some("Destructive file deletion (rm -rf/rm -f) detected.".to_string());
        }

        // Low-level disk commands
        if ["dd", "mkfs", "fdisk", "parted"].contains(&base_cmd) {
            requires_hitl = true;
            requires_admin = true;
            danger_reason = Some("Low-level partition or disk write operation detected.".to_string());
        }

        // Permission ownership change overrides
        if ["chmod", "chown"].contains(&base_cmd) && (parts.contains(&"777") || parts.contains(&"root") || parts.contains(&"-r")) {
            requires_hitl = true;
            danger_reason = Some("Permission/ownership override detected.".to_string());
        }

        // Moving files into key system areas
        if base_cmd == "mv" && (parts.contains(&"/etc") || parts.contains(&"/var") || parts.contains(&"/system") || parts.contains(&"/usr")) {
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
    }
}

#[tauri::command]
pub fn execute_elevated_command(command: &str) -> Result<String, String> {
    // Safety check again before running!
    let _safety = check_command_safety(command);
    
    // We enforce that the frontend should have acquired HITL confirmation.
    // If it requires admin, let's run it with native privileges!
    #[cfg(target_os = "macos")]
    {
        // For macOS, we execute via AppleScript 'do shell script ... with administrator privileges'
        // Escape quotes in the command
        let escaped_command = command.replace('\\', "\\\\").replace('"', "\\\"");
        let apple_script = format!(
            "do shell script \"{}\" with administrator privileges",
            escaped_command
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&apple_script)
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    Ok(String::from_utf8_lossy(&out.stdout).to_string())
                } else {
                    Err(String::from_utf8_lossy(&out.stderr).to_string())
                }
            }
            Err(e) => Err(format!("Failed to trigger AppleScript authentication: {}", e)),
        }
    }

    #[cfg(target_os = "linux")]
    {
        // For Linux, we run via pkexec
        // We split the command to run with pkexec bash -c "command"
        let output = Command::new("pkexec")
            .args(["bash", "-c", command])
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    Ok(String::from_utf8_lossy(&out.stdout).to_string())
                } else {
                    Err(String::from_utf8_lossy(&out.stderr).to_string())
                }
            }
            Err(e) => Err(format!("Failed to trigger Polkit (pkexec) authentication: {}", e)),
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err("Administrative elevation is not supported on this OS platform.".to_string())
    }
}
