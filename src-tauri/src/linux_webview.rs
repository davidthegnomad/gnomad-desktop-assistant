//! Linux WebKit/Wayland workarounds — must run before Tauri/GTK initializes.

/// Apply env vars that reduce WebKit2GTK crashes on KDE Wayland (esp. NVIDIA).
pub fn init_stability_env() {
    set_default_env("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    set_default_env("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    // XWayland avoids the worst KWin+GTK resize/mode-switch bugs on Wayland sessions.
    if session_is_wayland() {
        set_default_env("GDK_BACKEND", "x11");
    }
}

fn session_is_wayland() -> bool {
    std::env::var("XDG_SESSION_TYPE")
        .map(|v| v.eq_ignore_ascii_case("wayland"))
        .unwrap_or(false)
        || std::env::var("WAYLAND_DISPLAY").is_ok()
}

fn set_default_env(key: &str, value: &str) {
    if std::env::var_os(key).is_none() {
        std::env::set_var(key, value);
    }
}
