//! Linux desktop diagnostics for `gnomad-gtk --doctor`.

use std::process::Command;

use crate::platform::{gather_desktop_context, DesktopContext};
use crate::shell::sandbox::{sandbox_level, sandbox_shell_available};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone)]
pub struct DoctorCheck {
    pub id: &'static str,
    pub label: &'static str,
    pub status: CheckStatus,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub session_type: String,
    pub desktop: String,
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    pub fn has_failures(&self) -> bool {
        self.checks
            .iter()
            .any(|c| c.status == CheckStatus::Fail)
    }

    pub fn format_text(&self) -> String {
        let mut out = String::from("Gnomad desktop doctor\n\n");
        out.push_str(&format!(
            "Session: {} · Desktop: {}\n\n",
            self.session_type, self.desktop
        ));
        for check in &self.checks {
            let icon = match check.status {
                CheckStatus::Pass => "✓",
                CheckStatus::Warn => "!",
                CheckStatus::Fail => "✗",
            };
            out.push_str(&format!(
                "  [{icon}] {} — {}\n",
                check.label, check.detail
            ));
        }
        let (pass, warn, fail) = self.counts();
        out.push_str(&format!(
            "\nSummary: {pass} passed, {warn} warnings, {fail} failed\n"
        ));
        if fail > 0 {
            out.push_str("Fix failed checks before expecting full functionality.\n");
        } else if warn > 0 {
            out.push_str("Warnings are optional tools or DE limitations — app should still run.\n");
        } else {
            out.push_str("Environment looks good for Gnomad.\n");
        }
        out
    }

    fn counts(&self) -> (usize, usize, usize) {
        let mut pass = 0;
        let mut warn = 0;
        let mut fail = 0;
        for c in &self.checks {
            match c.status {
                CheckStatus::Pass => pass += 1,
                CheckStatus::Warn => warn += 1,
                CheckStatus::Fail => fail += 1,
            }
        }
        (pass, warn, fail)
    }
}

pub fn run_linux_diagnostics() -> DoctorReport {
    let session_type = detect_session_type().to_string();
    let desktop = detect_desktop().to_string();
    let mut checks = Vec::new();

    checks.push(check_display_session(&session_type));
    checks.push(check_dbus_session());
    checks.push(check_status_notifier_watcher(&desktop));
    checks.push(check_clipboard_tools(&session_type));
    checks.push(check_window_probe_tools(&session_type, &desktop));
    checks.push(check_wmctrl_pop_out(&session_type));
    checks.push(check_bwrap_sandbox());
    checks.push(check_ollama());
    checks.push(check_context_probe());

    DoctorReport {
        session_type,
        desktop,
        checks,
    }
}

fn check(id: &'static str, label: &'static str, status: CheckStatus, detail: impl Into<String>) -> DoctorCheck {
    DoctorCheck {
        id,
        label,
        status,
        detail: detail.into(),
    }
}

fn check_display_session(session_type: &str) -> DoctorCheck {
    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let x11 = std::env::var_os("DISPLAY").is_some();
    if wayland || x11 {
        let via = if wayland && x11 {
            "Wayland + XWayland"
        } else if wayland {
            "Wayland"
        } else {
            "X11"
        };
        check(
            "display",
            "Graphical session",
            CheckStatus::Pass,
            format!("{via} (XDG_SESSION_TYPE={session_type})"),
        )
    } else {
        check(
            "display",
            "Graphical session",
            CheckStatus::Fail,
            "WAYLAND_DISPLAY and DISPLAY are unset — GUI cannot start",
        )
    }
}

fn check_dbus_session() -> DoctorCheck {
    if std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some() {
        check(
            "dbus",
            "D-Bus session",
            CheckStatus::Pass,
            "DBUS_SESSION_BUS_ADDRESS is set (tray + DE probes)",
        )
    } else if command_exists("busctl") {
        let ok = Command::new("busctl")
            .args(["--user", "status"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ok {
            check(
                "dbus",
                "D-Bus session",
                CheckStatus::Pass,
                "user session bus reachable via busctl",
            )
        } else {
            check(
                "dbus",
                "D-Bus session",
                CheckStatus::Fail,
                "no session bus — tray and GTK integration may fail",
            )
        }
    } else {
        check(
            "dbus",
            "D-Bus session",
            CheckStatus::Warn,
            "DBUS_SESSION_BUS_ADDRESS unset; install busctl to verify",
        )
    }
}

fn check_status_notifier_watcher(desktop: &str) -> DoctorCheck {
    if dbus_name_active("org.kde.StatusNotifierWatcher") {
        return check(
            "sni",
            "System tray (SNI)",
            CheckStatus::Pass,
            "StatusNotifierWatcher is running",
        );
    }

    let hint = match desktop {
        "gnome" => "Install AppIndicator (or equivalent) for tray icons",
        "cosmic" => "Enable the COSMIC status-area applet; update cosmic-applets",
        "kde" => "KDE system tray applet may be disabled",
        _ => "Tray host may be missing; launch Gnomad from the app menu instead",
    };
    check(
        "sni",
        "System tray (SNI)",
        CheckStatus::Warn,
        format!("StatusNotifierWatcher not found — {hint}"),
    )
}

fn check_clipboard_tools(session_type: &str) -> DoctorCheck {
    let wl = command_exists("wl-paste");
    let xclip = command_exists("xclip");
    if session_type == "wayland" && wl {
        check(
            "clipboard",
            "Clipboard context",
            CheckStatus::Pass,
            "wl-paste available (Wayland)",
        )
    } else if xclip {
        check(
            "clipboard",
            "Clipboard context",
            CheckStatus::Pass,
            "xclip available",
        )
    } else if session_type == "wayland" {
        check(
            "clipboard",
            "Clipboard context",
            CheckStatus::Warn,
            "install wl-clipboard (wl-paste) for clipboard context pills",
        )
    } else {
        check(
            "clipboard",
            "Clipboard context",
            CheckStatus::Warn,
            "install xclip or wl-clipboard for clipboard context pills",
        )
    }
}

fn check_window_probe_tools(session_type: &str, desktop: &str) -> DoctorCheck {
    let tool = match desktop {
        "kde" if session_type == "wayland" => command_exists("qdbus").then_some("qdbus+KWin"),
        "gnome" if session_type == "wayland" => command_exists("gdbus").then_some("gdbus+Shell.Eval"),
        "hyprland" => command_exists("hyprctl").then_some("hyprctl"),
        _ if session_type == "x11" && command_exists("xdotool") => Some("xdotool"),
        _ if command_exists("xdotool") => Some("xdotool (XWayland)"),
        _ => None,
    };

    if let Some(name) = tool {
        check(
            "window_probe",
            "Active window title",
            CheckStatus::Pass,
            format!("{name} available for {desktop} on {session_type}"),
        )
    } else {
        check(
            "window_probe",
            "Active window title",
            CheckStatus::Warn,
            format!(
                "no probe tool for {desktop}/{session_type} — context pill may show Unknown"
            ),
        )
    }
}

fn check_wmctrl_pop_out(session_type: &str) -> DoctorCheck {
    if command_exists("wmctrl") {
        check(
            "wmctrl",
            "Pop Out stay-above",
            CheckStatus::Pass,
            "wmctrl installed (best on X11/XWayland)",
        )
    } else if session_type == "wayland" {
        check(
            "wmctrl",
            "Pop Out stay-above",
            CheckStatus::Warn,
            "wmctrl not installed; pure Wayland may not keep Pop Out above other windows",
        )
    } else {
        check(
            "wmctrl",
            "Pop Out stay-above",
            CheckStatus::Warn,
            "install wmctrl for Pop Out stay-above on X11",
        )
    }
}

fn check_bwrap_sandbox() -> DoctorCheck {
    if !command_exists("bwrap") {
        return check(
            "bwrap",
            "YOLO shell sandbox",
            CheckStatus::Warn,
            "bubblewrap (bwrap) not installed — sandboxed agent shell unavailable",
        );
    }
    if !user_namespaces_enabled() {
        return check(
            "bwrap",
            "YOLO shell sandbox",
            CheckStatus::Warn,
            "bwrap present but user namespaces may be disabled (kernel hardening)",
        );
    }
    let level = sandbox_level();
    if sandbox_shell_available() {
        check(
            "bwrap",
            "YOLO shell sandbox",
            CheckStatus::Pass,
            format!("bubblewrap ready (sandbox_level={level})"),
        )
    } else {
        check(
            "bwrap",
            "YOLO shell sandbox",
            CheckStatus::Warn,
            "bwrap found but sandbox not available",
        )
    }
}

fn check_ollama() -> DoctorCheck {
    if command_exists("ollama") {
        check(
            "ollama",
            "Local LLM (Ollama)",
            CheckStatus::Pass,
            "ollama binary on PATH — run `ollama serve` for Local provider",
        )
    } else {
        check(
            "ollama",
            "Local LLM (Ollama)",
            CheckStatus::Warn,
            "ollama not on PATH — use Cloud provider or install Ollama",
        )
    }
}

fn check_context_probe() -> DoctorCheck {
    let ctx: DesktopContext = gather_desktop_context(40);
    let window_ok = ctx.app_name != "Unknown" || ctx.window_title != "Unknown Window";
    let clip_ok = ctx.clipboard_preview != "Empty";

    if window_ok && clip_ok {
        check(
            "context_live",
            "Live context sample",
            CheckStatus::Pass,
            format!(
                "window={} / \"{}\"; clipboard={}",
                ctx.app_name,
                truncate(&ctx.window_title, 36),
                truncate(&ctx.clipboard_preview, 36)
            ),
        )
    } else if window_ok || clip_ok {
        check(
            "context_live",
            "Live context sample",
            CheckStatus::Warn,
            format!(
                "partial — app=\"{}\" title=\"{}\"; clipboard={}",
                ctx.app_name,
                truncate(&ctx.window_title, 28),
                truncate(&ctx.clipboard_preview, 28)
            ),
        )
    } else {
        check(
            "context_live",
            "Live context sample",
            CheckStatus::Warn,
            "active window and clipboard both unavailable (probes or permissions)",
        )
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

fn detect_session_type() -> &'static str {
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

fn detect_desktop() -> &'static str {
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
    if desktop.is_empty() {
        "unknown"
    } else {
        "other"
    }
}

fn command_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn dbus_name_active(name: &str) -> bool {
    if command_exists("busctl") {
        return Command::new("busctl")
            .args(["--user", "status", name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
    }
    if command_exists("qdbus") {
        return Command::new("qdbus")
            .args([name, "/StatusNotifierWatcher"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
    }
    if command_exists("gdbus") {
        return Command::new("gdbus")
            .args([
                "introspect",
                "--session",
                "--dest",
                name,
                "--object-path",
                "/StatusNotifierWatcher",
            ])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
    }
    false
}

fn user_namespaces_enabled() -> bool {
    match std::fs::read_to_string("/proc/sys/kernel/unprivileged_userns_clone") {
        Ok(v) => v.trim() == "1",
        Err(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_format_includes_summary() {
        let report = DoctorReport {
            session_type: "wayland".into(),
            desktop: "kde".into(),
            checks: vec![check(
                "display",
                "Graphical session",
                CheckStatus::Pass,
                "test",
            )],
        };
        let text = report.format_text();
        assert!(text.contains("Gnomad desktop doctor"));
        assert!(text.contains("Summary:"));
        assert!(!report.has_failures());
    }
}
