pub use gnomad_core::shell::sandbox::{sandbox_level, sandbox_shell_available};

use gnomad_core::shell::sandbox::sandboxed_shell_command as sandboxed_shell_command_inner;
use std::path::Path;

pub fn sandboxed_shell_command(
    app: &tauri::AppHandle,
    workspace: &Path,
    shell: &str,
    shell_args: &[String],
) -> Result<(String, Vec<String>), String> {
    let paths = crate::core_bridge::data_paths_from_app(app)?;
    sandboxed_shell_command_inner(&paths, workspace, shell, shell_args)
}

#[cfg(target_os = "macos")]
pub fn write_macos_sandbox_profile_for_app(
    app: &tauri::AppHandle,
    workspace: &Path,
) -> Result<std::path::PathBuf, String> {
    let paths = crate::core_bridge::data_paths_from_app(app)?;
    write_macos_sandbox_profile(&paths, workspace)
}

#[cfg(not(target_os = "macos"))]
pub fn write_macos_sandbox_profile_for_app(
    _app: &tauri::AppHandle,
    _workspace: &Path,
) -> Result<std::path::PathBuf, String> {
    Err("macOS sandbox profile is only used on macOS.".into())
}
