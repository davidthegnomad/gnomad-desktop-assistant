//! Concise tooltip copy for in-app controls (mirrors tray menu flavor text).

pub const MODE_PANEL: &str = "Panel — Slim docked strip along the screen edge";
pub const MODE_FLOATING: &str = "Pop out — Floating window that tries to stay above others";
pub const MODE_WINDOWED: &str = "Window — Standard resizable desktop window; use the tray to hide";
pub const MODE_FULLSCREEN: &str = "Full — Maximize Gnomad to fill the screen";

pub const SETTINGS: &str = "Settings — Agent, secrets, knowledge, and API keys";
pub const HELP: &str = "Help — Quick guide to setup, tray, and desktop quirks";
pub const NEW_CHAT: &str = "New chat — Start a fresh conversation";
pub const TERMINAL: &str = "Terminal — Show command output and shell session";
pub const TERMINAL_HIDE: &str = "Hide terminal — Collapse the output panel";
pub const ATTACH: &str = "Attach — Add up to 10 files (text is inlined for the model)";
pub const SEND: &str = "Send — Submit your message (Ctrl+Enter)";
pub const ACCESS_LOCAL: &str = "Access local files — Enable agent tools for files and shell";
pub const PROVIDER: &str = "Provider — Local (Ollama) or Cloud API";
pub const MODEL: &str = "Model — Pick the LLM for this chat";

pub fn mode_tooltip(mode: crate::window::DisplayMode) -> &'static str {
    use crate::window::DisplayMode;
    match mode {
        DisplayMode::Panel => MODE_PANEL,
        DisplayMode::Floating => MODE_FLOATING,
        DisplayMode::Windowed => MODE_WINDOWED,
        DisplayMode::Fullscreen => MODE_FULLSCREEN,
    }
}
