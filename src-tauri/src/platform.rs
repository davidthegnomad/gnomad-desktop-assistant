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
        };
    }

    #[cfg(target_os = "linux")]
    {
        return PlatformInfo {
            os: "linux",
            tray_region_label: "System tray".into(),
            panel_mode_menu_label: "System Tray Panel".into(),
            hide_to_tray_label: "Hide to System Tray".into(),
            tray_tooltip: "Gnomad — click to open from the system tray".into(),
            uses_overlay_titlebar: false,
            hide_in_app_titlebar_when_windowed: true,
            supports_active_window_context: true,
            supports_clipboard_context: true,
            supports_accessibility_settings: false,
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
        }
    }
}
