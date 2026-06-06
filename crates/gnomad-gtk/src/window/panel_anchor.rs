//! Panel placement — default top-right and tray-anchored positioning (KDE-first).

use gdk4::prelude::DisplayExt;
use gtk4::prelude::{MonitorExt, NativeExt, WidgetExt};
use gtk4::ApplicationWindow;

use super::dims::{PANEL_HEIGHT, PANEL_WIDTH};

#[derive(Debug, Clone, Copy, Default)]
pub struct TrayAnchor {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl TrayAnchor {
    pub fn from_tray_event(x: i32, y: i32) -> Self {
        Self {
            x: x as f64,
            y: y as f64,
            width: 22.0,
            height: 22.0,
        }
    }

    pub fn is_meaningful(self) -> bool {
        self.x > 1.0 || self.y > 1.0
    }
}

pub fn apply_panel_position(window: &ApplicationWindow, anchor: Option<TrayAnchor>) {
    let Some((x, y)) = panel_xy(window, anchor) else {
        return;
    };
    if kde_kwin_move_window("Gnomad", x, y, PANEL_WIDTH, PANEL_HEIGHT) {
        return;
    }
    let _ = wmctrl_move_window("Gnomad", x, y, PANEL_WIDTH, PANEL_HEIGHT);
}

fn panel_xy(window: &ApplicationWindow, anchor: Option<TrayAnchor>) -> Option<(i32, i32)> {
    let display = WidgetExt::display(window);
    let surface = NativeExt::surface(window)?;
    let monitor = display.monitor_at_surface(&surface)?;
    let geometry = monitor.geometry();
    let scale = monitor.scale_factor() as f64;
    let screen_w = geometry.width() as f64 / scale;
    let screen_h = geometry.height() as f64 / scale;
    let win_w = PANEL_WIDTH as f64;
    let win_h = PANEL_HEIGHT as f64;

    let (x, y) = if anchor.filter(|a| a.is_meaningful()).is_some() {
        let a = anchor.unwrap();
        let icon_center_x = a.x + a.width / 2.0;
        let x = (icon_center_x - win_w / 2.0)
            .max(8.0)
            .min(screen_w - win_w - 8.0);
        let y = (a.y + a.height + 4.0).max(8.0).min(screen_h - win_h - 8.0);
        (x.round() as i32, y.round() as i32)
    } else {
        let x = (screen_w - win_w - 12.0).max(8.0).round() as i32;
        let y = 8;
        (x, y)
    };
    Some((x, y))
}

fn wmctrl_move_window(title: &str, x: i32, y: i32, w: i32, h: i32) -> bool {
    if !command_exists("wmctrl") {
        return false;
    }
    std::process::Command::new("wmctrl")
        .args([
            "-r",
            title,
            "-e",
            &format!("0,{x},{y},{w},{h}"),
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// KDE Plasma: reposition via KWin scripting (Wayland-safe on primary dev target).
fn kde_kwin_move_window(title: &str, x: i32, y: i32, w: i32, h: i32) -> bool {
    if !command_exists("qdbus") {
        return false;
    }
    let token = format!("gnomad_place_{}", std::process::id());
    let script = format!(
        r#"
var target = "{title}";
var wins = workspace.windowList();
for (var i = 0; i < wins.length; i++) {{
  var win = wins[i];
  if (win.caption.indexOf(target) >= 0) {{
    win.setGeometry({{x: {x}, y: {y}, width: {w}, height: {h}}});
    console.info("{token}: placed");
    break;
  }}
}}
"#
    );
    let script_path = std::env::temp_dir().join(format!("gnomad-kwin-place-{token}.js"));
    if std::fs::write(&script_path, script).is_err() {
        return false;
    }
    let path = script_path.to_string_lossy();
    let plugin = "gnomad_panel_place";

    let _ = std::process::Command::new("qdbus")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.unloadScript",
            plugin,
        ])
        .output();

    let load = std::process::Command::new("qdbus")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.loadScript",
            path.as_ref(),
            plugin,
        ])
        .output();
    let ok = load.map(|o| o.status.success()).unwrap_or(false);
    if !ok {
        let _ = std::fs::remove_file(&script_path);
        return false;
    }

    let _ = std::process::Command::new("qdbus")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.start",
        ])
        .output();
    std::thread::sleep(std::time::Duration::from_millis(120));
    let _ = std::process::Command::new("qdbus")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.unloadScript",
            plugin,
        ])
        .output();
    let _ = std::fs::remove_file(&script_path);
    ok
}

fn command_exists(cmd: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|path| {
        std::env::split_paths(&path).any(|dir| dir.join(cmd).is_file())
    })
}
