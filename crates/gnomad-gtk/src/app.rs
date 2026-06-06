use std::cell::RefCell;
use std::rc::Rc;

use std::sync::{mpsc, Arc, Mutex};

use gnomad_core::config::env_config;
use gnomad_core::config::paths::DataPaths;
use gnomad_core::config::ui_prefs::{load_ui_prefs, ThemeMode};
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::icons::{self, ICON_NAME};
use crate::shortcut;
use crate::state::{AppState, ChatEvent, UiCommand};
use crate::tray;
use crate::ui;
use crate::window::{apply_panel_position, DisplayMode, TrayAnchor, WindowManager};

const APP_ID: &str = "com.gnomadstudio.gnomad";

fn handle_command(
    mgr: &Rc<RefCell<Option<WindowManager>>>,
    state: &Arc<AppState>,
    cmd: UiCommand,
) {
    match cmd {
        UiCommand::Quit => std::process::exit(0),
        UiCommand::ToggleWindowed => {
            if let Some(m) = mgr.borrow_mut().as_mut() {
                m.toggle_windowed();
            }
        }
        UiCommand::TrayActivate { x, y } => {
            let anchor = TrayAnchor::from_tray_event(x, y);
            if let Ok(mut stored) = state.last_tray_anchor.lock() {
                *stored = Some(anchor);
            }
            if let Some(m) = mgr.borrow_mut().as_mut() {
                m.set_tray_anchor(Some(anchor));
                m.show_in_mode(DisplayMode::Windowed);
            }
        }
        UiCommand::ShowPanel => {
            if let Some(m) = mgr.borrow_mut().as_mut() {
                let anchor = state
                    .last_tray_anchor
                    .lock()
                    .ok()
                    .and_then(|g| *g);
                m.set_tray_anchor(anchor);
                m.show();
            }
        }
        UiCommand::ShowWindowed => {
            if let Some(m) = mgr.borrow_mut().as_mut() {
                m.show_in_mode(DisplayMode::Windowed);
            }
        }
        UiCommand::ShowFloating => {
            if let Some(m) = mgr.borrow_mut().as_mut() {
                m.show_in_mode(DisplayMode::Floating);
            }
        }
        UiCommand::HidePanel => {
            if let Some(m) = mgr.borrow_mut().as_ref() {
                m.hide();
            }
        }
        UiCommand::SetMode(mode) => {
            if let Some(m) = mgr.borrow_mut().as_mut() {
                let anchor = state
                    .last_tray_anchor
                    .lock()
                    .ok()
                    .and_then(|g| *g);
                m.set_tray_anchor(anchor);
                m.set_mode(mode);
                if mode == DisplayMode::Panel {
                    apply_panel_position(m.window(), anchor);
                }
            }
        }
        UiCommand::OpenSettings => {
            if let Some(m) = mgr.borrow_mut().as_mut() {
                m.show_in_mode(DisplayMode::Windowed);
            }
            let _ = state.chat_tx.send(ChatEvent::OpenSettings);
        }
        UiCommand::OpenUrl(url) => {
            let _ = std::process::Command::new("xdg-open").arg(url).spawn();
        }
    }
}

pub fn run() {
    // Avoid GL/ZINK EGL paths that crash on some NVIDIA + Wayland setups when widgets repaint.
    if std::env::var_os("GSK_RENDERER").is_none() {
        std::env::set_var("GSK_RENDERER", "cairo");
    }
    env_config::load_dotenv_files(&[]);

    let (cmd_tx, cmd_rx) = mpsc::channel::<UiCommand>();
    let (chat_tx, chat_rx) = mpsc::channel();
    let chat_rx = Rc::new(RefCell::new(Some(chat_rx)));
    let state = AppState::new(cmd_tx.clone(), chat_tx);
    let manager_cell: Rc<RefCell<Option<WindowManager>>> = Rc::new(RefCell::new(None));

    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    let icon_paths = DataPaths::discover();
    app.connect_startup(move |_| {
        apply_theme(load_ui_prefs(&icon_paths).theme);
        if let Err(e) = icons::install_icon_theme(&icon_paths) {
            eprintln!("gnomad-gtk: icon theme install failed: {e}");
        }
    });

    let mgr_activate = Rc::clone(&manager_cell);
    let state_activate = Arc::clone(&state);
    let chat_rx_activate = Rc::clone(&chat_rx);
    app.connect_activate(move |app| {
        if let Some(m) = mgr_activate.borrow_mut().as_mut() {
            m.show();
            return;
        }

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Gnomad")
            .icon_name(ICON_NAME)
            .default_width(crate::window::PANEL_WIDTH)
            .default_height(crate::window::PANEL_HEIGHT)
            .build();

        let chat_rx = chat_rx_activate
            .borrow_mut()
            .take()
            .expect("chat event receiver");
        let shell = ui::build_shell(Arc::clone(&state_activate), chat_rx);
        window.set_content(Some(&shell));

        window.set_visible(false);

        let gtk_window: gtk4::ApplicationWindow = window.upcast();
        let mut manager = WindowManager::new(gtk_window);
        manager.set_mode(DisplayMode::Panel);
        manager.bind_hide_on_close();
        *mgr_activate.borrow_mut() = Some(manager);

        shortcut::setup_global_shortcut_poll(cmd_tx.clone());
        shortcut::register_app_accel(app, cmd_tx.clone());

        tray::spawn_tray(cmd_tx.clone());
    });

    let cmd_rx = Arc::new(Mutex::new(cmd_rx));
    let mgr_poll = Rc::clone(&manager_cell);
    let state_poll = Arc::clone(&state);
    let rx_poll = Arc::clone(&cmd_rx);
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        if let Ok(rx) = rx_poll.lock() {
            while let Ok(cmd) = rx.try_recv() {
                handle_command(&mgr_poll, &state_poll, cmd);
            }
        }
        glib::ControlFlow::Continue
    });

    let code = app.run();
    std::process::exit(code.into());
}

fn apply_theme(theme: ThemeMode) {
    let scheme = match theme {
        ThemeMode::Light => adw::ColorScheme::ForceLight,
        ThemeMode::Dark => adw::ColorScheme::ForceDark,
        ThemeMode::System => adw::ColorScheme::Default,
    };
    adw::StyleManager::default().set_color_scheme(scheme);
}
