use std::process::Command;

#[tauri::command]
pub fn execute_shell_command(command: &str) -> Result<serde_json::Value, String> {
    // Run in user standard shell
    let shell = if cfg!(target_os = "windows") { "cmd" } else { "zsh" };
    let flag = if cfg!(target_os = "windows") { "/C" } else { "-c" };

    // If zsh fails, fallback to sh
    let mut cmd = Command::new(shell);
    cmd.arg(flag).arg(command);

    let output = cmd.output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let status_code = out.status.code().unwrap_or(if out.status.success() { 0 } else { 1 });

            Ok(serde_json::json!({
                "success": out.status.success(),
                "status_code": status_code,
                "stdout": stdout,
                "stderr": stderr
            }))
        }
        Err(e) => {
            // Fallback to "sh" on macOS/Linux if "zsh" isn't present or errors on spawn
            if shell == "zsh" {
                let mut fallback_cmd = Command::new("sh");
                fallback_cmd.arg("-c").arg(command);
                if let Ok(fallback_out) = fallback_cmd.output() {
                    let stdout = String::from_utf8_lossy(&fallback_out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&fallback_out.stderr).to_string();
                    let status_code = fallback_out.status.code().unwrap_or(if fallback_out.status.success() { 0 } else { 1 });

                    return Ok(serde_json::json!({
                        "success": fallback_out.status.success(),
                        "status_code": status_code,
                        "stdout": stdout,
                        "stderr": stderr
                    }));
                }
            }
            Err(format!("Failed to execute shell: {}", e))
        }
    }
}
