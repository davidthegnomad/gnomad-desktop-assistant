pub mod exec;
pub mod rules;
pub mod safety;
pub mod sandbox;
pub mod session;

pub use exec::{run_shell_command, validate_shell_command, ShellOutputSink, ShellRunResult};
pub use session::{
    run_shell_command_in_session, ShellSessionState, ShellSessionStatus,
};
pub use rules::looks_like_shell_command;
pub use rules::normalize_elevated_command;
pub use safety::{
    check_command_safety, elevation_command_rejected, execute_elevated_command,
    looks_like_file_write, SafetyCheckResult,
};
pub use sandbox::{
    escape_windows_batch_path, sandbox_level, sandbox_shell_available, sandboxed_shell_command,
};
