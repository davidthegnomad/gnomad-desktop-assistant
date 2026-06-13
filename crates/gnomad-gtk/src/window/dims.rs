/// Window geometry — matches macOS/Tauri `window_manager.rs` product spec.
pub const PANEL_WIDTH: i32 = 600;
pub const PANEL_HEIGHT: i32 = 920;
pub const EXPANDED_WIDTH: i32 = 1283;
pub const EXPANDED_HEIGHT: i32 = 858;
pub const MIN_WIDTH: i32 = 520;
pub const MIN_HEIGHT: i32 = 420;
pub const MAX_WIDTH: i32 = 1600;
pub const MAX_HEIGHT: i32 = 1200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Panel,
    Floating,
    Windowed,
    Fullscreen,
}

impl DisplayMode {
    pub const ALL: [DisplayMode; 4] = [
        DisplayMode::Panel,
        DisplayMode::Floating,
        DisplayMode::Windowed,
        DisplayMode::Fullscreen,
    ];

    pub fn label(self) -> &'static str {
        match self {
            DisplayMode::Panel => "Panel",
            DisplayMode::Floating => "Pop out",
            DisplayMode::Windowed => "Window",
            DisplayMode::Fullscreen => "Full",
        }
    }

    pub fn from_label(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "pop out" | "floating" | "popout" => DisplayMode::Floating,
            "window" | "windowed" => DisplayMode::Windowed,
            "full" | "fullscreen" => DisplayMode::Fullscreen,
            _ => DisplayMode::Panel,
        }
    }

    pub fn default_size(self) -> (i32, i32) {
        match self {
            DisplayMode::Panel => (PANEL_WIDTH, PANEL_HEIGHT),
            DisplayMode::Floating | DisplayMode::Windowed => (EXPANDED_WIDTH, EXPANDED_HEIGHT),
            DisplayMode::Fullscreen => (EXPANDED_WIDTH, EXPANDED_HEIGHT),
        }
    }

    pub fn resizable(self) -> bool {
        !matches!(self, DisplayMode::Panel | DisplayMode::Fullscreen)
    }
}
