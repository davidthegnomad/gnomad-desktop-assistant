use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;

use crate::state::UiCommand;

/// Poll global-hotkey events (works on X11; on pure Wayland use app accel fallback).
pub fn setup_global_shortcut_poll(cmd_tx: std::sync::mpsc::Sender<UiCommand>) {
    if let Ok(manager) = GlobalHotKeyManager::new() {
        let hotkey = HotKey::new(
            Some(Modifiers::CONTROL | Modifiers::SHIFT),
            Code::Space,
        );
        match manager.register(hotkey) {
            Ok(_) => std::mem::forget(manager),
            Err(e) => eprintln!(
                "gnomad-gtk: global shortcut unavailable (install input group or use XWayland): {e}"
            ),
        }
    }

    glib::timeout_add_local(std::time::Duration::from_millis(80), move || {
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state == global_hotkey::HotKeyState::Pressed {
                let _ = cmd_tx.send(UiCommand::ToggleWindowed);
            }
        }
        glib::ControlFlow::Continue
    });
}

pub fn register_app_accel(app: &adw::Application, cmd_tx: std::sync::mpsc::Sender<UiCommand>) {
    let toggle = gtk4::gio::SimpleAction::new("toggle", None);
    toggle.connect_activate(move |_, _| {
        let _ = cmd_tx.send(UiCommand::ToggleWindowed);
    });
    app.add_action(&toggle);
    app.set_accels_for_action("app.toggle", &["<Control><Shift>space"]);
}
