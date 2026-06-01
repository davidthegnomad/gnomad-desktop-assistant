use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconEvent},
    ActivationPolicy, Emitter, Manager, PhysicalPosition, PhysicalSize, Runtime, WebviewWindow,
};

/// Default panel size — tuned to fit header, context pills, welcome, and input without clipping.
pub const PANEL_WIDTH: u32 = 1060;
pub const PANEL_HEIGHT: u32 = 780;
const FLOATING_WIDTH: u32 = PANEL_WIDTH;
const FLOATING_HEIGHT: u32 = PANEL_HEIGHT;
const WINDOWED_WIDTH: u32 = 1100;
const WINDOWED_HEIGHT: u32 = 820;
const MIN_WIDTH: u32 = 520;
const MIN_HEIGHT: u32 = 420;
const MAX_WIDTH: u32 = 1600;
const MAX_HEIGHT: u32 = 1200;
/// macOS titled window chrome above the webview
const TITLE_BAR_HEIGHT: u32 = 32;
const SIZE_PADDING_W: u32 = 16;
const SIZE_PADDING_H: u32 = 12;

pub struct WindowRuntimeState {
    pub user_resized: Mutex<bool>,
    pub current_mode: Mutex<WindowDisplayMode>,
}

impl Default for WindowRuntimeState {
    fn default() -> Self {
        Self {
            user_resized: Mutex::new(false),
            current_mode: Mutex::new(WindowDisplayMode::Panel),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WindowDisplayMode {
    Panel,
    Floating,
    Windowed,
    Fullscreen,
}

impl WindowDisplayMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "floating" => Self::Floating,
            "windowed" => Self::Windowed,
            "fullscreen" => Self::Fullscreen,
            _ => Self::Panel,
        }
    }
}

#[derive(Clone)]
pub struct TrayAnchor {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Default for TrayAnchor {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 22.0,
            height: 22.0,
        }
    }
}

fn main_window<R: Runtime>(app: &tauri::AppHandle<R>) -> Option<WebviewWindow<R>> {
    app.get_webview_window("main")
}

fn menu_bar_inset<R: Runtime>(window: &WebviewWindow<R>) -> f64 {
    #[cfg(target_os = "macos")]
    {
        if let Ok(Some(monitor)) = window.current_monitor().or_else(|_| window.primary_monitor()) {
            let scale = monitor.scale_factor();
            return (28.0 * scale).round();
        }
        28.0
    }
    #[cfg(not(target_os = "macos"))]
    {
        0.0
    }
}

/// Position panel flush under the macOS menu bar (centered, or under tray icon).
pub fn position_panel_dropdown<R: Runtime>(
    window: &WebviewWindow<R>,
    anchor: Option<&TrayAnchor>,
) -> tauri::Result<()> {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());

    let Some(monitor) = monitor else {
        return Ok(());
    };

    let screen = monitor.size();
    let scale = monitor.scale_factor();
    let outer = window.outer_size()?;
    let win_w = outer.width as f64;
    let _win_h = outer.height as f64;
    let top = menu_bar_inset(window);

    let x = if let Some(a) = anchor {
        // Drop down centered under tray icon
        let icon_center_x = a.x + a.width / 2.0;
        (icon_center_x - win_w / 2.0).max(8.0).min(screen.width as f64 - win_w - 8.0)
    } else {
        ((screen.width as f64) - win_w) / 2.0
    };

    let y = if let Some(a) = anchor {
        // Below tray icon in menu bar
        (a.y + a.height + 4.0).max(top)
    } else {
        top
    };

    window.set_position(PhysicalPosition::new(x.round() as i32, y.round() as i32))?;
    let _ = scale;
    Ok(())
}

fn apply_min_max_size<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    window.set_min_size(Some(PhysicalSize::new(MIN_WIDTH, MIN_HEIGHT)))?;
    window.set_max_size(Some(PhysicalSize::new(MAX_WIDTH, MAX_HEIGHT)))?;
    Ok(())
}

/// Align OS chrome with display mode (macOS menu bar, Linux/Windows focus).
pub fn sync_platform_shell<R: Runtime>(app: &tauri::AppHandle<R>, mode: WindowDisplayMode) {
    #[cfg(target_os = "macos")]
    {
        let policy = if mode == WindowDisplayMode::Windowed {
            ActivationPolicy::Regular
        } else {
            ActivationPolicy::Accessory
        };
        let _ = app.set_activation_policy(policy);
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        if mode == WindowDisplayMode::Windowed {
            if let Some(window) = main_window(app) {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }
}

pub fn apply_window_mode<R: Runtime>(
    app: &tauri::AppHandle<R>,
    mode: WindowDisplayMode,
    anchor: Option<&TrayAnchor>,
) -> tauri::Result<WindowDisplayMode> {
    let Some(window) = main_window(app) else {
        return Ok(mode);
    };

    if let Some(state) = app.try_state::<WindowRuntimeState>() {
        *state.user_resized.lock().unwrap() = false;
        *state.current_mode.lock().unwrap() = mode;
    }

    if window.is_fullscreen()? {
        window.set_fullscreen(false)?;
    }

    match mode {
        WindowDisplayMode::Panel => {
            window.set_decorations(true)?;
            window.set_resizable(true)?;
            window.set_always_on_top(true)?;
            apply_min_max_size(&window)?;
            window.set_size(PhysicalSize::new(PANEL_WIDTH, PANEL_HEIGHT))?;
            #[cfg(target_os = "macos")]
            position_panel_dropdown(&window, anchor)?;
            #[cfg(not(target_os = "macos"))]
            {
                let _ = anchor;
                window.center()?;
            }
        }
        WindowDisplayMode::Floating => {
            window.set_decorations(true)?;
            window.set_resizable(true)?;
            window.set_always_on_top(true)?;
            apply_min_max_size(&window)?;
            window.set_size(PhysicalSize::new(FLOATING_WIDTH, FLOATING_HEIGHT))?;
            if anchor.is_some() {
                position_panel_dropdown(&window, anchor)?;
            } else {
                window.center()?;
            }
        }
        WindowDisplayMode::Windowed => {
            window.set_decorations(true)?;
            window.set_resizable(true)?;
            window.set_always_on_top(false)?;
            apply_min_max_size(&window)?;
            window.set_size(PhysicalSize::new(WINDOWED_WIDTH, WINDOWED_HEIGHT))?;
            window.center()?;
        }
        WindowDisplayMode::Fullscreen => {
            window.set_decorations(false)?;
            window.set_resizable(false)?;
            window.set_always_on_top(false)?;
            window.set_fullscreen(true)?;
        }
    }

    sync_platform_shell(app, mode);
    let _ = app.emit("window-mode-changed", mode);
    Ok(mode)
}

/// Fit outer window size to webview content (unless the user has manually resized).
pub fn apply_fit_window_to_content<R: Runtime>(
    app: &tauri::AppHandle<R>,
    content_width: u32,
    content_height: u32,
    force: bool,
) -> tauri::Result<()> {
    let Some(window) = main_window(app) else {
        return Ok(());
    };

    if window.is_fullscreen().unwrap_or(false) {
        return Ok(());
    }

    if let Some(state) = app.try_state::<WindowRuntimeState>() {
        if *state.current_mode.lock().unwrap() == WindowDisplayMode::Fullscreen {
            return Ok(());
        }
        if *state.user_resized.lock().unwrap() && !force {
            return Ok(());
        }
    }

    let mut outer_w = content_width.saturating_add(SIZE_PADDING_W).clamp(MIN_WIDTH, MAX_WIDTH);
    let mut outer_h = content_height
        .saturating_add(TITLE_BAR_HEIGHT)
        .saturating_add(SIZE_PADDING_H)
        .clamp(MIN_HEIGHT, MAX_HEIGHT);

    // Autosize may only grow the window; never shrink below current size unless forced.
    if !force {
        if let Ok(current) = window.outer_size() {
            outer_w = outer_w.max(current.width);
            outer_h = outer_h.max(current.height);
        }
    }

    window.set_size(PhysicalSize::new(outer_w, outer_h))?;

    if let Some(state) = app.try_state::<WindowRuntimeState>() {
        if *state.current_mode.lock().unwrap() == WindowDisplayMode::Panel {
            position_panel_dropdown(&window, None)?;
        }
    }

    Ok(())
}

pub fn show_gnomad<R: Runtime>(
    app: &tauri::AppHandle<R>,
    mode: Option<WindowDisplayMode>,
    anchor: Option<&TrayAnchor>,
) {
    let Some(window) = main_window(app) else {
        return;
    };

    let target = mode.unwrap_or(WindowDisplayMode::Panel);
    let _ = apply_window_mode(app, target, anchor);
    let _ = window.show();
    let _ = window.set_focus();
    let _ = app.emit("window-fit-requested", true);
}

pub fn hide_main_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = main_window(app) {
        let _ = window.hide();
    }
}

pub fn toggle_gnomad<R: Runtime>(app: &tauri::AppHandle<R>, anchor: Option<&TrayAnchor>) {
    let Some(window) = main_window(app) else {
        return;
    };

    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        show_gnomad(app, Some(WindowDisplayMode::Panel), anchor);
    }
}

pub fn tray_anchor_from_event(event: &TrayIconEvent) -> Option<TrayAnchor> {
    let position = match event {
        TrayIconEvent::Click { position, .. }
        | TrayIconEvent::DoubleClick { position, .. }
        | TrayIconEvent::Enter { position, .. }
        | TrayIconEvent::Move { position, .. }
        | TrayIconEvent::Leave { position, .. } => position,
        _ => return None,
    };
    Some(TrayAnchor {
        x: position.x,
        y: position.y,
        width: 22.0,
        height: 22.0,
    })
}

pub fn handle_tray_click<R: Runtime>(app: &tauri::AppHandle<R>, event: &TrayIconEvent) {
    if let TrayIconEvent::Click {
        button,
        button_state,
        ..
    } = event
    {
        if *button == MouseButton::Left && *button_state == MouseButtonState::Up {
            let anchor = tray_anchor_from_event(event);
            toggle_gnomad(app, anchor.as_ref());
        }
    }
}

#[tauri::command]
pub fn set_window_mode(
    app: tauri::AppHandle,
    mode: String,
    anchor_tray: Option<bool>,
) -> Result<String, String> {
    let display_mode = WindowDisplayMode::from_str(&mode);
    let anchor = if anchor_tray.unwrap_or(false) {
        Some(TrayAnchor::default())
    } else {
        None
    };
    apply_window_mode(&app, display_mode, anchor.as_ref()).map_err(|e| e.to_string())?;
    Ok(mode)
}

#[tauri::command]
pub fn get_window_mode(app: tauri::AppHandle) -> Result<String, String> {
    if let Some(state) = app.try_state::<WindowRuntimeState>() {
        let mode = *state.current_mode.lock().unwrap();
        return Ok(match mode {
            WindowDisplayMode::Panel => "panel".into(),
            WindowDisplayMode::Floating => "floating".into(),
            WindowDisplayMode::Windowed => "windowed".into(),
            WindowDisplayMode::Fullscreen => "fullscreen".into(),
        });
    }
    let window = main_window(&app).ok_or("main window not found")?;
    if window.is_fullscreen().map_err(|e| e.to_string())? {
        return Ok("fullscreen".into());
    }
    Ok("panel".into())
}

#[tauri::command]
pub fn fit_window_to_content(
    app: tauri::AppHandle,
    content_width: u32,
    content_height: u32,
    force: Option<bool>,
) -> Result<(), String> {
    apply_fit_window_to_content(&app, content_width, content_height, force.unwrap_or(false))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_window_user_resized(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(state) = app.try_state::<WindowRuntimeState>() {
        *state.user_resized.lock().unwrap() = true;
    }
    Ok(())
}
