//! `gnomad-gtk --doctor` — GTK build + desktop environment diagnostics.

use gnomad_core::platform::{run_linux_diagnostics, CheckStatus, DoctorCheck};

pub fn run_and_exit() -> ! {
    let mut report = run_linux_diagnostics();
    report.checks.extend(gtk_build_checks());
    report.checks.sort_by_key(|c| c.id);

    print!("{}", report.format_text());
    print!("{}", gtk_notes());

    if report.has_failures() {
        std::process::exit(1);
    }
    std::process::exit(0);
}

fn gtk_build_checks() -> Vec<DoctorCheck> {
    let mut checks = Vec::new();

    let gsk = std::env::var("GSK_RENDERER").unwrap_or_else(|_| "(unset → cairo at startup)".into());
    checks.push(DoctorCheck {
        id: "gsk_renderer",
        label: "GSK renderer",
        status: if gsk.contains("cairo") || gsk.contains("(unset") {
            CheckStatus::Pass
        } else {
            CheckStatus::Warn
        },
        detail: format!(
            "GSK_RENDERER={gsk} — cairo avoids NVIDIA/Wayland repaint crashes"
        ),
    });

    #[cfg(feature = "vte")]
    checks.push(DoctorCheck {
        id: "vte",
        label: "Embedded terminal (VTE)",
        status: CheckStatus::Pass,
        detail: "built with vte4 — Shell mode in terminal panel".into(),
    });
    #[cfg(not(feature = "vte"))]
    checks.push(DoctorCheck {
        id: "vte",
        label: "Embedded terminal (VTE)",
        status: CheckStatus::Warn,
        detail: "built without vte — terminal output view only".into(),
    });

    #[cfg(feature = "x11-above")]
    checks.push(DoctorCheck {
        id: "x11_above",
        label: "Pop Out X11 hints",
        status: CheckStatus::Pass,
        detail: "gdk4-x11 enabled — wmctrl stay-above on X11 surfaces".into(),
    });
    #[cfg(not(feature = "x11-above"))]
    checks.push(DoctorCheck {
        id: "x11_above",
        label: "Pop Out X11 hints",
        status: CheckStatus::Warn,
        detail: "built without x11-above feature".into(),
    });

    let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_default().to_ascii_lowercase();
    if session.contains("wayland") {
        checks.push(DoctorCheck {
            id: "global_hotkey",
            label: "Global shortcut",
            status: CheckStatus::Warn,
            detail: "Ctrl+Shift+Space may only work when Gnomad is focused on pure Wayland".into(),
        });
    } else {
        checks.push(DoctorCheck {
            id: "global_hotkey",
            label: "Global shortcut",
            status: CheckStatus::Pass,
            detail: "Ctrl+Shift+Space should work globally on X11/XWayland".into(),
        });
    }

    checks
}

fn gtk_notes() -> String {
    let mut notes = String::from("\nGTK build notes:\n");
    notes.push_str("  • Re-run after changing DE or installing optional packages.\n");
    notes.push_str("  • See crates/gnomad-gtk/README.md for the full QA matrix.\n");
    notes
}
