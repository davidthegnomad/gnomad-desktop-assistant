use serde::Serialize;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::Manager;

use crate::platform;

const KWIN_PROBE_PLUGIN: &str = "gnomad_ctx_probe";
const KWIN_DAEMON_PLUGIN: &str = "gnomad_active_daemon";
const ACTIVE_WINDOW_TTL: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinuxIntegrationStatus {
    pub session_type: String,
    pub desktop_environment: String,
    pub active_window_backend: String,
    pub clipboard_backend: String,
    pub sandbox_level: String,
    pub wl_paste_available: bool,
    pub xdotool_available: bool,
    pub qdbus_available: bool,
    pub bwrap_available: bool,
    pub tray_hint: String,
}

pub struct LinuxContextState {
    active_window: Mutex<(String, String)>,
    last_refresh: Mutex<Option<Instant>>,
}

impl Default for LinuxContextState {
    fn default() -> Self {
        Self {
            active_window: Mutex::new((
                "Unknown".to_string(),
                "Unknown Window".to_string(),
            )),
            last_refresh: Mutex::new(None),
        }
    }
}

pub fn init(app: &tauri::AppHandle) {
    let state = app.state::<LinuxContextState>();
    refresh_active_window(&state);

    let handle = app.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(ACTIVE_WINDOW_TTL);
            let state = handle.state::<LinuxContextState>();
            refresh_active_window(&state);
        }
    });
}

pub fn shutdown() {
    unload_kwin_script(KWIN_PROBE_PLUGIN);
    unload_kwin_script(KWIN_DAEMON_PLUGIN);
}

pub fn get_active_window_cached(state: &LinuxContextState) -> (String, String) {
    let stale = state
        .last_refresh
        .lock()
        .unwrap()
        .map(|t| t.elapsed() > ACTIVE_WINDOW_TTL)
        .unwrap_or(true);
    if stale {
        refresh_active_window(state);
    }
    state.active_window.lock().unwrap().clone()
}

fn refresh_active_window(state: &LinuxContextState) {
    let pair = probe_active_window();
    *state.active_window.lock().unwrap() = pair;
    *state.last_refresh.lock().unwrap() = Some(Instant::now());
}

pub fn probe_active_window() -> (String, String) {
    let session = platform::linux_session_type();
    if session == "wayland" {
        match desktop_environment() {
            "kde" => {
                if let Some(pair) = probe_kde_wayland_active_window() {
                    return pair;
                }
            }
            "gnome" => {
                if let Some(pair) = probe_gnome_wayland_active_window() {
                    return pair;
                }
            }
            "hyprland" => {
                if let Some(pair) = probe_hyprland_active_window() {
                    return pair;
                }
            }
            _ => {}
        }
        return ("Unknown".to_string(), "Unknown Window".to_string());
    }

    if session == "x11" {
        return probe_x11_active_window();
    }

    ("Unknown".to_string(), "Unknown Window".to_string())
}

pub fn get_integration_status() -> LinuxIntegrationStatus {
    let session = platform::linux_session_type().to_string();
    let desktop = desktop_environment().to_string();
    let wl_paste = command_exists("wl-paste");
    let xdotool = command_exists("xdotool");
    let qdbus = command_exists("qdbus");
    let bwrap = command_exists("bwrap");

    let active_window_backend = if session == "wayland" {
        match desktop.as_str() {
            "kde" if qdbus => "kwin-scripting".to_string(),
            "gnome" if command_exists("gdbus") => "gnome-shell-eval".to_string(),
            "hyprland" if command_exists("hyprctl") => "hyprctl".to_string(),
            _ => "unavailable".to_string(),
        }
    } else if session == "x11" && xdotool {
        "xdotool".to_string()
    } else {
        "unavailable".to_string()
    };

    let clipboard_backend = if session == "wayland" {
        if wl_paste {
            "wl-paste".to_string()
        } else {
            "missing-wl-clipboard".to_string()
        }
    } else if wl_paste {
        "wl-paste".to_string()
    } else if command_exists("xclip") {
        "xclip".to_string()
    } else {
        "unavailable".to_string()
    };

    let sandbox_level = crate::shell_sandbox::sandbox_level().to_string();

    let tray_hint = if session == "wayland" {
        "Wayland: left-click tray icon for menu, right-click to toggle panel.".to_string()
    } else {
        "Left-click tray icon to toggle panel; right-click for menu.".to_string()
    };

    LinuxIntegrationStatus {
        session_type: session,
        desktop_environment: desktop,
        active_window_backend,
        clipboard_backend,
        sandbox_level,
        wl_paste_available: wl_paste,
        xdotool_available: xdotool,
        qdbus_available: qdbus,
        bwrap_available: bwrap,
        tray_hint,
    }
}

pub fn supports_active_window_context() -> bool {
    let status = get_integration_status();
    status.active_window_backend != "unavailable"
}

pub fn supports_clipboard_context() -> bool {
    let status = get_integration_status();
    !status.clipboard_backend.starts_with("missing")
        && status.clipboard_backend != "unavailable"
}

fn desktop_environment() -> &'static str {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if desktop.contains("kde") {
        return "kde";
    }
    if desktop.contains("gnome") {
        return "gnome";
    }
    if desktop.contains("hyprland") || std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return "hyprland";
    }
    "other"
}

fn command_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_command_stdout(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn parse_window_pair(payload: &str) -> Option<(String, String)> {
    let mut parts = payload.splitn(2, "|||");
    let app_name = parts.next()?.trim();
    if app_name.is_empty() {
        return None;
    }
    let window_title = parts
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Unknown Window")
        .to_string();
    Some((app_name.to_string(), window_title))
}

fn probe_x11_active_window() -> (String, String) {
    if !command_exists("xdotool") {
        return (
            "Unknown".to_string(),
            "Install xdotool for X11 window context".to_string(),
        );
    }
    let app_name = run_command_stdout("xdotool", &["getactivewindow", "getwindowclassname"])
        .unwrap_or_else(|| "Unknown".to_string());
    let window_title = run_command_stdout("xdotool", &["getactivewindow", "getwindowname"])
        .unwrap_or_else(|| "Unknown Window".to_string());
    (app_name, window_title)
}

fn probe_kde_wayland_active_window() -> Option<(String, String)> {
    if !command_exists("qdbus") {
        return None;
    }

    let token = format!("gnomad_{}", std::process::id());
    let script_body = format!(
        "console.info(\"{token}: \" + (workspace.activeWindow ? workspace.activeWindow.resourceClass + \"|||\" + workspace.activeWindow.caption : \"Unknown|||Unknown Window\"));"
    );

    let script_path = std::env::temp_dir().join(format!("gnomad-kwin-probe-{token}.js"));
    std::fs::write(&script_path, script_body).ok()?;
    let script_path_str = script_path.to_string_lossy();

    unload_kwin_script(KWIN_PROBE_PLUGIN);

    let load = Command::new("qdbus")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.loadScript",
            script_path_str.as_ref(),
            KWIN_PROBE_PLUGIN,
        ])
        .output()
        .ok()?;
    if !load.status.success() {
        let _ = std::fs::remove_file(&script_path);
        return None;
    }

    let start = Command::new("qdbus")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.start",
        ])
        .output()
        .ok()?;
    if !start.status.success() {
        unload_kwin_script(KWIN_PROBE_PLUGIN);
        let _ = std::fs::remove_file(&script_path);
        return None;
    }

    std::thread::sleep(Duration::from_millis(180));

    let marker = format!("{token}: ");
    let journal = Command::new("journalctl")
        .args(["-b", "-t", "kwin_wayland", "-n", "80", "--no-pager", "-o", "cat"])
        .output()
        .ok()?;

    unload_kwin_script(KWIN_PROBE_PLUGIN);
    let _ = std::fs::remove_file(&script_path);

    if !journal.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&journal.stdout);
    let line = stdout.lines().rev().find(|line| line.contains(&marker))?;
    let payload = line.split(&marker).nth(1)?.trim();
    parse_window_pair(payload)
}

fn probe_gnome_wayland_active_window() -> Option<(String, String)> {
    let script = r#"(function(){var w=global.display.get_focus_window();if(!w)return 'Unknown|||Unknown Window';return w.get_wm_class()+'|||'+w.get_title();})()"#;
    let output = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/org/gnome/Shell",
            "--method",
            "org.gnome.Shell.Eval",
            script,
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for segment in stdout.split('\'') {
        if segment.contains("|||") {
            if let Some(pair) = parse_window_pair(segment) {
                return Some(pair);
            }
        }
    }
    None
}

fn probe_hyprland_active_window() -> Option<(String, String)> {
    let output = Command::new("hyprctl")
        .args(["-j", "activewindow"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let class = json
        .get("class")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let title = json
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Window")
        .to_string();
    Some((class, title))
}

fn unload_kwin_script(plugin: &str) {
    let _ = Command::new("qdbus")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.unloadScript",
            plugin,
        ])
        .output();
}

#[tauri::command]
pub fn get_linux_integration_status() -> Result<LinuxIntegrationStatus, String> {
    Ok(get_integration_status())
}
