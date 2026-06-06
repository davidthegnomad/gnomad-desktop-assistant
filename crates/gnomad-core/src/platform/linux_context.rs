//! Linux desktop context (clipboard + active window) for GTK shell.

use std::process::Command;

#[derive(Debug, Clone)]
pub struct DesktopContext {
    pub app_name: String,
    pub window_title: String,
    pub clipboard_preview: String,
}

pub fn gather_desktop_context(clipboard_max_chars: usize) -> DesktopContext {
    let (app_name, window_title) = probe_active_window();
    DesktopContext {
        app_name,
        window_title,
        clipboard_preview: probe_clipboard_preview(clipboard_max_chars),
    }
}

pub fn format_context_block(ctx: &DesktopContext) -> String {
    format!(
        "Active window: {} — {}\nClipboard preview: {}",
        ctx.app_name, ctx.window_title, ctx.clipboard_preview
    )
}

fn probe_clipboard_preview(max_chars: usize) -> String {
    if command_exists("wl-paste") {
        if let Some(text) = run_command_stdout("wl-paste", &["-n"]) {
            return truncate_preview(&text, max_chars);
        }
        if let Some(text) = run_command_stdout("wl-paste", &[]) {
            return truncate_preview(&text, max_chars);
        }
    }
    if command_exists("xclip") {
        if let Some(text) = run_command_stdout("xclip", &["-o", "-selection", "clipboard"]) {
            return truncate_preview(&text, max_chars);
        }
    }
    "Empty".into()
}

fn truncate_preview(text: &str, max_chars: usize) -> String {
    let collapsed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return "Empty".into();
    }
    if collapsed.chars().count() <= max_chars {
        collapsed
    } else {
        let short: String = collapsed.chars().take(max_chars).collect();
        format!("{short}…")
    }
}

pub fn probe_active_window() -> (String, String) {
    let session = linux_session_type();
    if session == "wayland" {
        return match desktop_environment() {
            "kde" => probe_kde_wayland_active_window()
                .unwrap_or_else(|| ("Unknown".into(), "Unknown Window".into())),
            "gnome" => probe_gnome_wayland_active_window()
                .unwrap_or_else(|| ("Unknown".into(), "Unknown Window".into())),
            "cosmic" => probe_cosmic_wayland_active_window()
                .unwrap_or_else(|| ("Unknown".into(), "Unknown Window".into())),
            "hyprland" => probe_hyprland_active_window()
                .unwrap_or_else(|| ("Unknown".into(), "Unknown Window".into())),
            _ => ("Unknown".into(), "Unknown Window".into()),
        };
    }
    if session == "x11" && command_exists("xdotool") {
        return probe_x11_active_window();
    }
    ("Unknown".into(), "Unknown Window".into())
}

fn linux_session_type() -> &'static str {
    let value = std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if value.contains("wayland") {
        "wayland"
    } else if value.contains("x11") || value == "xorg" {
        "x11"
    } else {
        "unknown"
    }
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
    if desktop.contains("cosmic") {
        return "cosmic";
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

fn probe_x11_active_window() -> (String, String) {
    let app_name = run_command_stdout("xdotool", &["getactivewindow", "getwindowclassname"])
        .unwrap_or_else(|| "Unknown".into());
    let window_title = run_command_stdout("xdotool", &["getactivewindow", "getwindowname"])
        .unwrap_or_else(|| "Unknown Window".into());
    (app_name, window_title)
}

fn probe_hyprland_active_window() -> Option<(String, String)> {
    let line = run_command_stdout("hyprctl", &["activewindow", "-j"])?;
    let class = extract_json_string(&line, "class").unwrap_or_else(|| "Unknown".into());
    let title = extract_json_string(&line, "title").unwrap_or_else(|| "Unknown Window".into());
    Some((class, title))
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{key}\"");
    let start = json.find(&pattern)? + pattern.len();
    let rest = json.get(start..)?;
    let after_colon = rest.find(':')? + 1;
    let value_part = rest.get(after_colon..)?.trim();
    if !value_part.starts_with('"') {
        return None;
    }
    let inner = value_part.trim_start_matches('"');
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}

/// GNOME Shell Eval probe (Wayland). May fail on newer GNOME if Eval is restricted.
fn probe_gnome_wayland_active_window() -> Option<(String, String)> {
    if !command_exists("gdbus") {
        return None;
    }
    let script = "global.display.focus_window ? global.display.focus_window.get_wm_class() + '|||' + global.display.focus_window.get_title() : 'Unknown|||Unknown Window'";
    let out = run_command_stdout(
        "gdbus",
        &[
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/org/gnome/Shell",
            "--method",
            "org.gnome.Shell.Eval",
            script,
            "true",
        ],
    )?;
    parse_gdbus_eval_result(&out)
}

/// COSMIC Wayland: best-effort via XWayland xdotool when DISPLAY is set.
fn probe_cosmic_wayland_active_window() -> Option<(String, String)> {
    if std::env::var_os("DISPLAY").is_some() && command_exists("xdotool") {
        return Some(probe_x11_active_window());
    }
    None
}

fn parse_gdbus_eval_result(raw: &str) -> Option<(String, String)> {
    if raw.contains("(false") {
        return None;
    }
    let start = raw.find('\'')? + 1;
    let rest = raw.get(start..)?;
    let end = rest.find('\'')?;
    parse_window_pair(&rest[..end])
}

/// Lightweight KDE Wayland probe via qdbus + KWin scripting (same approach as Tauri shell).
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

    let _ = Command::new("qdbus")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.unloadScript",
            "gnomad_ctx_probe",
        ])
        .output();

    let load = Command::new("qdbus")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.loadScript",
            script_path_str.as_ref(),
            "gnomad_ctx_probe",
        ])
        .output()
        .ok()?;
    if !load.status.success() {
        let _ = std::fs::remove_file(&script_path);
        return None;
    }
    let _ = Command::new("qdbus")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.start",
        ])
        .output();
    std::thread::sleep(std::time::Duration::from_millis(180));

    let marker = format!("{token}: ");
    let journal = Command::new("journalctl")
        .args(["-b", "-t", "kwin_wayland", "-n", "80", "--no-pager", "-o", "cat"])
        .output()
        .ok()?;
    let _ = Command::new("qdbus")
        .args([
            "org.kde.KWin",
            "/Scripting",
            "org.kde.kwin.Scripting.unloadScript",
            "gnomad_ctx_probe",
        ])
        .output();
    let _ = std::fs::remove_file(&script_path);

    if !journal.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&journal.stdout);
    let line = stdout.lines().rev().find(|line| line.contains(&marker))?;
    let payload = line.split(&marker).nth(1)?.trim();
    parse_window_pair(payload)
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
