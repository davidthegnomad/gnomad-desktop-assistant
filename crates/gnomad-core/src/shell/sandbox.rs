use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::paths::DataPaths;

pub fn sandbox_level() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        return "full";
    }
    #[cfg(target_os = "linux")]
    {
        return if command_exists("bwrap") {
            "full"
        } else {
            "none"
        };
    }
    #[cfg(target_os = "windows")]
    {
        return "workspace";
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        "none"
    }
}

pub fn sandbox_shell_available() -> bool {
    sandbox_level() != "none"
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

pub fn escape_windows_batch_path(path: &str) -> String {
    path.replace('%', "%%").replace('"', "\"\"")
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn sandbox_data_dir(paths: &DataPaths) -> Result<PathBuf, String> {
    paths.ensure_gnomad_data_dir()
}

#[cfg(target_os = "macos")]
fn escape_sb_path(path: &str) -> String {
    path.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(target_os = "macos")]
pub fn write_macos_sandbox_profile(
    paths: &DataPaths,
    workspace: &Path,
) -> Result<PathBuf, String> {
    let dir = sandbox_data_dir(paths)?;
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
    _paths: &DataPaths,
    _workspace: &Path,
) -> Result<PathBuf, String> {
    Err("macOS sandbox profile is only used on macOS.".into())
}

#[cfg(target_os = "windows")]
pub fn write_windows_sandbox_init(
    paths: &DataPaths,
    workspace: &Path,
    shell: &str,
    shell_args: &[String],
) -> Result<PathBuf, String> {
    let dir = sandbox_data_dir(paths)?;
    let init_path = dir.join("yolo-shell-init.cmd");

    let ws = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    let ws_s = escape_windows_batch_path(&ws.to_string_lossy());
    let tmp = escape_windows_batch_path(
        &ws.join(".gnomad-sandbox-tmp")
            .to_string_lossy()
            .to_string(),
    );

    let mut shell_cmd = format!("\"{}\"", escape_windows_batch_path(shell));
    for arg in shell_args {
        shell_cmd.push(' ');
        shell_cmd.push_str(&format!("\"{}\"", escape_windows_batch_path(arg)));
    }

    let script = format!(
        "@echo off\r\n\
         cd /d \"{ws_s}\"\r\n\
         set \"TEMP={tmp}\"\r\n\
         set \"TMP={tmp}\"\r\n\
         if not exist \"{tmp}\" mkdir \"{tmp}\"\r\n\
         {shell_cmd}\r\n"
    );

    std::fs::write(&init_path, script).map_err(|e| e.to_string())?;
    Ok(init_path)
}

#[cfg(not(target_os = "windows"))]
pub fn write_windows_sandbox_init(
    _paths: &DataPaths,
    _workspace: &Path,
    _shell: &str,
    _shell_args: &[String],
) -> Result<PathBuf, String> {
    Err("Windows sandbox init is only used on Windows.".into())
}

pub fn sandboxed_shell_command(
    #[allow(unused_variables)] paths: &DataPaths,
    workspace: &Path,
    shell: &str,
    shell_args: &[String],
) -> Result<(String, Vec<String>), String> {
    #[cfg(target_os = "macos")]
    {
        let profile = write_macos_sandbox_profile(paths, workspace)?;
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

    #[cfg(target_os = "windows")]
    {
        let init = write_windows_sandbox_init(paths, workspace, shell, shell_args)?;
        return Ok((
            "cmd.exe".to_string(),
            vec!["/k".to_string(), init.to_string_lossy().to_string()],
        ));
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = (paths, workspace, shell, shell_args);
        Err("Sandboxed shell is not supported on this platform.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn escape_sb_path_quotes() {
        assert!(escape_sb_path("/tmp/foo").contains("/tmp/foo"));
    }

    #[test]
    fn escape_windows_batch_path_percent() {
        assert_eq!(
            escape_windows_batch_path("C:\\work\\%TEMP%"),
            "C:\\work\\%%TEMP%%"
        );
    }

    #[test]
    fn sandbox_level_is_defined() {
        let level = sandbox_level();
        assert!(matches!(level, "full" | "workspace" | "none"));
    }
}
