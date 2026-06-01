use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::Manager;

/// Whether sandboxed shell is requested and supported on this OS.
pub fn sandbox_shell_available() -> bool {
    #[cfg(target_os = "macos")]
    {
        return true;
    }
    #[cfg(target_os = "linux")]
    {
        return command_exists("bwrap");
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        false
    }
}

fn command_exists(name: &str) -> bool {
    let checker = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };
    Command::new(checker)
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn escape_sb_path(path: &str) -> String {
    path.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Write a sandbox-exec profile for YOLO shell sessions (macOS).
#[cfg(target_os = "macos")]
pub fn write_macos_sandbox_profile(
    app: &tauri::AppHandle,
    workspace: &Path,
) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    let dir = base.join("gnomad");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let profile_path = dir.join("yolo-shell.sb");

    let ws = escape_sb_path(&workspace.to_string_lossy());
    let tmp = escape_sb_path("/private/tmp");
    let var = escape_sb_path("/var/folders");

    let profile = format!(
        r#"(version 1)
; Gnomad YOLO shell sandbox — network off, writes limited to workspace + temp
(deny default)
(allow process-fork)
(allow process-exec)
(allow signal)
(allow file-read*)
(allow file-write* (subpath "{ws}"))
(allow file-write* (subpath "{tmp}"))
(allow file-write* (subpath "{var}"))
(allow sysctl-read)
(deny network*)
"#
    );

    std::fs::write(&profile_path, profile).map_err(|e| e.to_string())?;
    Ok(profile_path)
}

#[cfg(not(target_os = "macos"))]
pub fn write_macos_sandbox_profile(
    _app: &tauri::AppHandle,
    _workspace: &Path,
) -> Result<PathBuf, String> {
    Err("macOS sandbox profile is only used on macOS.".into())
}

/// Wrap default shell invocation for a sandboxed PTY (macOS sandbox-exec or Linux bwrap).
pub fn sandboxed_shell_command(
    app: &tauri::AppHandle,
    workspace: &Path,
    shell: &str,
    shell_args: &[String],
) -> Result<(String, Vec<String>), String> {
    #[cfg(target_os = "macos")]
    {
        let profile = write_macos_sandbox_profile(app, workspace)?;
        let mut args = vec![
            "-f".to_string(),
            profile.to_string_lossy().to_string(),
            shell.to_string(),
        ];
        args.extend(shell_args.iter().cloned());
        return Ok(("sandbox-exec".to_string(), args));
    }

    #[cfg(target_os = "linux")]
    {
        if !command_exists("bwrap") {
            return Err(
                "bubblewrap (bwrap) is not installed. Install bubblewrap or disable sandboxed shell."
                    .into(),
            );
        }
        let ws = workspace
            .canonicalize()
            .unwrap_or_else(|_| workspace.to_path_buf());
        let ws_s = ws.to_string_lossy().to_string();

        let mut args = vec![
            "--die-with-parent".to_string(),
            "--ro-bind".to_string(),
            "/usr".to_string(),
            "/usr".to_string(),
            "--ro-bind".to_string(),
            "/bin".to_string(),
            "/bin".to_string(),
            "--ro-bind".to_string(),
            "/lib".to_string(),
            "/lib".to_string(),
            "--bind".to_string(),
            ws_s.clone(),
            ws_s.clone(),
            "--proc".to_string(),
            "/proc".to_string(),
            "--dev".to_string(),
            "/dev".to_string(),
            "--tmpfs".to_string(),
            "/tmp".to_string(),
            "--setenv".to_string(),
            "HOME".to_string(),
            "--setenv".to_string(),
            ws_s,
            "--setenv".to_string(),
            "TERM".to_string(),
            "--setenv".to_string(),
            "xterm-256color".to_string(),
        ];

        if Path::new("/lib64").exists() {
            args.extend([
                "--ro-bind".to_string(),
                "/lib64".to_string(),
                "/lib64".to_string(),
            ]);
        }

        args.push(shell.to_string());
        args.extend(shell_args.iter().cloned());
        return Ok(("bwrap".to_string(), args));
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = (app, workspace, shell, shell_args);
        Err("Sandboxed shell is not supported on this platform.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_sb_path_quotes() {
        assert!(escape_sb_path("/tmp/foo").contains("/tmp/foo"));
    }
}
