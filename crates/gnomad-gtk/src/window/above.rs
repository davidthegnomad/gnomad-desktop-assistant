//! Best-effort stay-above hint for Pop Out (Floating) mode.
//!
//! GTK4 has no cross-platform keep-above API. On X11/XWayland we set `_NET_WM_STATE_ABOVE`
//! via `wmctrl` when available; on pure Wayland compositors may ignore this.

use gtk4::ApplicationWindow;

fn command_exists(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(feature = "x11-above")]
fn window_xid(window: &ApplicationWindow) -> Option<u64> {
    use gdk4_x11::X11Surface;
    use glib::object::Cast;
    let surface = super::surface::window_surface(window)?;
    let x11 = surface.downcast::<X11Surface>().ok()?;
    let xid = x11.xid();
    if xid == 0 {
        None
    } else {
        Some(xid)
    }
}

#[cfg(feature = "x11-above")]
fn wmctrl_above(xid: u64, add: bool) -> bool {
    if !command_exists("wmctrl") {
        return false;
    }
    let op = if add { "add,above" } else { "remove,above" };
    std::process::Command::new("wmctrl")
        .args(["-i", "-r", &format!("{xid}"), "-b", op])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn try_stay_above(window: &ApplicationWindow) -> bool {
    #[cfg(feature = "x11-above")]
    {
        if let Some(xid) = window_xid(window) {
            if wmctrl_above(xid, true) {
                return true;
            }
        }
    }
    false
}

fn try_clear_above(window: &ApplicationWindow) {
    #[cfg(feature = "x11-above")]
    {
        if let Some(xid) = window_xid(window) {
            let _ = wmctrl_above(xid, false);
        }
    }
}

/// Request that a floating window stay above normal windows (best-effort).
pub fn apply_floating_above(window: &ApplicationWindow) {
    if !try_stay_above(window) {
        eprintln!(
            "gnomad-gtk: stay-above hint unavailable (Wayland or missing X11/wmctrl)"
        );
    }
}

/// Clear stay-above when leaving Floating mode.
pub fn clear_above(window: &ApplicationWindow) {
    try_clear_above(window);
}
