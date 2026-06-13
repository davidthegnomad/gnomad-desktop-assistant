//! Linux window management — separate from macOS tray-accessory model.
//!
//! GTK/WebKit on KDE Wayland crashes when mode switches burst-call:
//! set_decorations, set_always_on_top, set_resizable, set_size, center, unmaximize.
//! Linux modes are **UI layout only**; the native window chrome stays fixed after init.

use std::time::Duration;

use tauri::{Emitter, Manager, Runtime, WebviewWindow};

use crate::window_manager::{TrayAnchor, WindowDisplayMode, WindowRuntimeState};

fn main_window<R: Runtime>(app: &tauri::AppHandle<R>) -> Option<WebviewWindow<R>> {
    app.get_webview_window("main")
}

/// One-time window chrome — never toggle these at runtime on Linux.
pub fn init_window_chrome<R: Runtime>(app: &tauri::AppHandle<R>) {
    let Some(window) = main_window(app) else {
        return;
    };
    let _ = window.set_decorations(true);
    let _ = window.set_resizable(true);
    let _ = window.set_always_on_top(false);
}

fn position_panel_default<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());

    let Some(monitor) = monitor else {
        return window.center();
    };

    let screen = monitor.size();
    let outer = window.outer_size()?;
    let win_w = outer.width as f64;
    let win_h = outer.height as f64;
    let top = 8.0;
    let x = (screen.width as f64 - win_w - 12.0).max(8.0);
    let y = top;
    let max_y = (screen.height as f64 - win_h - 8.0).max(top);
    let y = y.min(max_y);

    window.set_position(tauri::PhysicalPosition::new(x.round() as i32, y.round() as i32))
}

fn position_panel_near_tray<R: Runtime>(
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
    let outer = window.outer_size()?;
    let win_w = outer.width as f64;

    let x = if let Some(a) = anchor {
        let icon_center_x = a.x + a.width / 2.0;
        (icon_center_x - win_w / 2.0)
            .max(8.0)
            .min(screen.width as f64 - win_w - 8.0)
    } else {
        ((screen.width as f64) - win_w) / 2.0
    };

    let y = if let Some(a) = anchor {
        (a.y + a.height + 4.0).max(8.0)
    } else {
        8.0
    };

    window.set_position(tauri::PhysicalPosition::new(x.round() as i32, y.round() as i32))
}

fn position_panel<R: Runtime>(
    window: &WebviewWindow<R>,
    anchor: Option<&TrayAnchor>,
) -> tauri::Result<()> {
    let has_real_anchor = anchor.is_some_and(|a| a.x > 1.0 || a.y > 1.0);
    if has_real_anchor {
        position_panel_near_tray(window, anchor)
    } else {
        position_panel_default(window)
    }
}

fn update_mode_state<R: Runtime>(app: &tauri::AppHandle<R>, mode: WindowDisplayMode) {
    if let Some(state) = app.try_state::<WindowRuntimeState>() {
        *state.user_resized.lock().unwrap() = false;
        *state.current_mode.lock().unwrap() = mode;
    }
}

/// Apply a display mode without reshaping GTK window properties (except fullscreen).
pub fn apply_mode<R: Runtime>(
    app: &tauri::AppHandle<R>,
    mode: WindowDisplayMode,
    anchor: Option<&TrayAnchor>,
) -> tauri::Result<WindowDisplayMode> {
    let Some(window) = main_window(app) else {
        return Ok(mode);
    };

    let was_fullscreen = window.is_fullscreen().unwrap_or(false);
    update_mode_state(app, mode);

    // UI can react immediately; GTK work is deferred and minimal.
    let _ = app.emit("window-mode-changed", mode);

    let app_handle = app.clone();
    let anchor_owned = anchor.cloned();

    if mode == WindowDisplayMode::Fullscreen {
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(32)).await;
            let Some(win) = main_window(&app_handle) else {
                return;
            };
            let _ = win.set_fullscreen(true);
        });
        return Ok(mode);
    }

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(32)).await;
        let Some(win) = main_window(&app_handle) else {
            return;
        };

        if was_fullscreen {
            let _ = win.set_fullscreen(false);
            tokio::time::sleep(Duration::from_millis(48)).await;
        }

        if mode == WindowDisplayMode::Panel {
            let _ = position_panel(&win, anchor_owned.as_ref());
        }
    });

    Ok(mode)
}

pub fn show<R: Runtime>(
    app: &tauri::AppHandle<R>,
    mode: Option<WindowDisplayMode>,
    anchor: Option<&TrayAnchor>,
) {
    let Some(window) = main_window(app) else {
        return;
    };

    let target = mode.unwrap_or(WindowDisplayMode::Panel);
    let _ = apply_mode(app, target, anchor);
    let _ = window.show();
}
