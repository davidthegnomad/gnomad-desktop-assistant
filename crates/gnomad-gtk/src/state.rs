use std::sync::{mpsc, Arc};
use std::sync::mpsc::SyncSender;

use gnomad_core::agent::tokens::path::PathScope;
use gnomad_core::agent::{AgentLoopResult, AgentSettingsState, HitlTokenState, PathTokenState};
use gnomad_core::shell::ShellSessionState;
use gnomad_core::GnomadCoreContext;

use crate::window::TrayAnchor;

#[derive(Debug, Clone)]
pub enum UiCommand {
    ToggleWindowed,
    ShowPanel,
    ShowWindowed,
    ShowFloating,
    HidePanel,
    SetMode(crate::window::DisplayMode),
    OpenSettings,
    OpenUrl(&'static str),
    TrayActivate { x: i32, y: i32 },
    Quit,
}

#[derive(Debug)]
pub enum ChatEvent {
    OllamaModels(Vec<String>),
    CompletionResult(Result<String, String>),
    AgentResult(Result<AgentLoopResult, String>),
    ShellApproval {
        command: String,
        reason: String,
        elevated: bool,
        reply: SyncSender<Option<String>>,
    },
    PathApproval {
        path: String,
        reason: String,
        scope: PathScope,
        reply: SyncSender<Option<String>>,
    },
    TerminalCommandStart {
        command: String,
    },
    TerminalOutput {
        chunk: String,
    },
    OpenSettings,
}

/// Send-safe app state (tray thread holds only `cmd_tx`).
pub struct AppState {
    pub core: GnomadCoreContext,
    pub agent_settings: Arc<AgentSettingsState>,
    pub hitl: Arc<HitlTokenState>,
    pub path_tokens: Arc<PathTokenState>,
    pub shell_session: Arc<ShellSessionState>,
    pub last_tray_anchor: std::sync::Mutex<Option<TrayAnchor>>,
    pub cmd_tx: mpsc::Sender<UiCommand>,
    pub chat_tx: mpsc::Sender<ChatEvent>,
}

impl AppState {
    pub fn new(cmd_tx: mpsc::Sender<UiCommand>, chat_tx: mpsc::Sender<ChatEvent>) -> Arc<Self> {
        let core = GnomadCoreContext::discover();
        let agent_settings = Arc::new(AgentSettingsState::default());
        gnomad_core::agent::init_agent_settings(&core.paths, &agent_settings);
        Arc::new(Self {
            core,
            agent_settings,
            hitl: Arc::new(HitlTokenState::default()),
            path_tokens: Arc::new(PathTokenState::default()),
            shell_session: Arc::new(ShellSessionState::default()),
            last_tray_anchor: std::sync::Mutex::new(None),
            cmd_tx,
            chat_tx,
        })
    }
}
