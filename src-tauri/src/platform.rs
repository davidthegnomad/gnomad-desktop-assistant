use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    pub os: &'static str,
    /// User-facing name for the tray / status area (menu bar vs system tray).
    pub tray_region_label: String,
    pub panel_mode_menu_label: String,
    pub hide_to_tray_label: String,
    pub tray_tooltip: String,
    /// macOS overlay title bar with traffic lights; Win/Linux use native caption when decorated.
    pub uses_overlay_titlebar: bool,
    /// Hide duplicate in-app toolbar when the OS already provides menu + caption (windowed).
    pub hide_in_app_titlebar_when_windowed: bool,
    /// Context pills can show active window title (needs OS integration).
    pub supports_active_window_context: bool,
    /// Clipboard preview pill (needs OS integration).
    pub supports_clipboard_context: bool,
    /// macOS Accessibility permission flow in settings.
    pub supports_accessibility_settings: bool,
    /// Linux session type: wayland, x11, or unknown.
    pub linux_session_type: Option<String>,
    /// When true, left-click on tray opens menu (Wayland-friendly).
    pub tray_left_click_opens_menu: bool,
}

#[cfg(target_os = "linux")]
pub fn linux_session_type() -> &'static str {
    match std::env::var("XDG_SESSION_TYPE") {
        Ok(v) if v.eq_ignore_ascii_case("wayland") => "wayland",
        Ok(v) if v.eq_ignore_ascii_case("x11") => "x11",
        Ok(v) if !v.is_empty() => "unknown",
        _ => {
            if std::env::var("WAYLAND_DISPLAY").is_ok() {
                "wayland"
            } else if std::env::var("DISPLAY").is_ok() {
                "x11"
            } else {
                "unknown"
            }
        }
    }
}

#[tauri::command]
pub fn get_platform_info() -> PlatformInfo {
    #[cfg(target_os = "macos")]
    {
        return PlatformInfo {
            os: "macos",
            tray_region_label: "Menu bar".into(),
            panel_mode_menu_label: "Menu Bar Panel".into(),
            hide_to_tray_label: "Hide to Menu Bar".into(),
            tray_tooltip: "Gnomad — click to open panel from the menu bar".into(),
            uses_overlay_titlebar: true,
            hide_in_app_titlebar_when_windowed: false,
            supports_active_window_context: true,
            supports_clipboard_context: true,
            supports_accessibility_settings: true,
            linux_session_type: None,
            tray_left_click_opens_menu: false,
        };
    }

    #[cfg(target_os = "windows")]
    {
        return PlatformInfo {
            os: "windows",
            tray_region_label: "System tray".into(),
            panel_mode_menu_label: "System Tray Panel".into(),
            hide_to_tray_label: "Hide to System Tray".into(),
            tray_tooltip: "Gnomad — click to open from the system tray".into(),
            uses_overlay_titlebar: false,
            hide_in_app_titlebar_when_windowed: true,
            supports_active_window_context: true,
            supports_clipboard_context: true,
            supports_accessibility_settings: false,
            linux_session_type: None,
            tray_left_click_opens_menu: false,
        };
    }

    #[cfg(target_os = "linux")]
    {
        let session = linux_session_type().to_string();
        let wayland = session == "wayland";
        return PlatformInfo {
            os: "linux",
            tray_region_label: "System tray".into(),
            panel_mode_menu_label: "System Tray Panel".into(),
            hide_to_tray_label: "Hide to System Tray".into(),
            tray_tooltip: if wayland {
                "Gnomad — left-click tray icon for menu (Wayland)".into()
            } else {
                "Gnomad — click to open from the system tray".into()
            },
            uses_overlay_titlebar: false,
            hide_in_app_titlebar_when_windowed: true,
            supports_active_window_context: true,
            supports_clipboard_context: true,
            supports_accessibility_settings: false,
            linux_session_type: Some(session),
            tray_left_click_opens_menu: wayland,
        };
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        PlatformInfo {
            os: "unknown",
            tray_region_label: "Tray".into(),
            panel_mode_menu_label: "Panel".into(),
            hide_to_tray_label: "Hide".into(),
            tray_tooltip: "Gnomad".into(),
            uses_overlay_titlebar: false,
            hide_in_app_titlebar_when_windowed: false,
            supports_active_window_context: false,
            supports_clipboard_context: false,
            supports_accessibility_settings: false,
            linux_session_type: None,
            tray_left_click_opens_menu: false,
        }
    }
}
