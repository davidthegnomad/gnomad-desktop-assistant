pub mod audit;
pub mod fs;
pub mod runtime;
pub mod settings;
pub mod tokens;
pub mod tools;

pub use audit::{log_action, log_shell_run};
pub use runtime::{
    AgentActionRecord, AgentApprovals, AgentLoopResult, AgentRuntime, MAX_AGENT_STEPS,
};
pub use settings::{
    default_workspace_root, init_agent_settings, load_persisted, read_settings, resolve_agent_path_with_token,
    resolve_path_input, save_persisted, settings_dto, should_sandbox_shell, AgentSettings,
    AgentSettingsDto, AgentSettingsState, SudoAuthMode, TrustMode,
};
pub use tokens::{HitlScope, HitlTokenState, PathScope, PathTokenState};
