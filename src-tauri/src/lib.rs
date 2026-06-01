mod context;
mod privilege;
mod keychain;
mod automation;
mod shell_executor;
mod window_manager;
mod knowledge;
mod env_config;
mod llm;
mod chat_history;
mod menu_shell;

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, RunEvent, WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code, ShortcutState};
use tauri_plugin_opener::OpenerExt;
use window_manager::{
    handle_tray_click, hide_main_window, show_gnomad, toggle_gnomad, TrayAnchor,
    WindowDisplayMode, WindowRuntimeState,
};

#[cfg(target_os = "macos")]
use std::process::Command;

const STUDIO_URL: &str = "https://gnomadstudio.org";

fn load_tray_icon(app: &tauri::App) -> tauri::image::Image<'static> {
    let icon = tauri::include_image!("icons/tray-mushroom.png").to_owned();
    if icon.width() > 0 && icon.height() > 0 {
        return icon;
    }
    app.default_window_icon()
        .map(|i| i.clone().to_owned())
        .unwrap_or(icon)
}

struct AppTrayState {
    last_anchor: Mutex<Option<TrayAnchor>>,
}

/// Set to true before `app.exit()` so close-to-tray does not block real quit.
struct AppLifecycleState {
    quit_requested: Mutex<bool>,
}

#[tauri::command]
fn check_accessibility_permissions() -> bool {
    #[cfg(target_os = "macos")]
    {
        extern "C" {
            fn AXIsProcessTrusted() -> bool;
        }
        unsafe { AXIsProcessTrusted() }
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

#[tauri::command]
fn request_accessibility_permissions() {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn();
    }
}

fn store_tray_anchor(state: &AppTrayState, anchor: Option<TrayAnchor>) {
    if let Some(a) = anchor {
        *state.last_anchor.lock().unwrap() = Some(a);
    }
}

fn last_anchor(state: &AppTrayState) -> Option<TrayAnchor> {
    state.last_anchor.lock().unwrap().clone()
}

fn open_studio_website(app: &tauri::AppHandle) {
    let _ = app.opener().open_url(STUDIO_URL, None::<&str>);
}

fn open_settings_panel(app: &tauri::AppHandle, state: &AppTrayState) {
    show_gnomad(
        app,
        Some(WindowDisplayMode::Panel),
        last_anchor(state).as_ref(),
    );
    let _ = app.emit("menu-open-settings", ());
}

fn build_tray_menu(app: &tauri::AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "show", "🦙 Show Panel", true, None::<&str>)?,
            &MenuItem::with_id(app, "mode_floating", "Pop Out Window", true, None::<&str>)?,
            &MenuItem::with_id(app, "settings", "🍄 Settings", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "visit_website", "Visit gnomadstudio.org", true, None::<&str>)?,
            &MenuItem::with_id(app, "about", "About Gnomad", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", "Quit Gnomad", true, None::<&str>)?,
        ],
    )
}

fn handle_menu_action(app: &tauri::AppHandle, state: &AppTrayState, id: &str) {
    let stored_anchor = last_anchor(state);
    let anchor = stored_anchor.as_ref();
    match id {
        "show" | "show_overlay" | "show_overlay_view" | "mode_panel" => {
            show_gnomad(app, Some(WindowDisplayMode::Panel), anchor);
        }
        "mode_floating" => {
            show_gnomad(app, Some(WindowDisplayMode::Floating), anchor);
        }
        "mode_windowed" => {
            show_gnomad(app, Some(WindowDisplayMode::Windowed), None);
        }
        "mode_fullscreen" => {
            show_gnomad(app, Some(WindowDisplayMode::Fullscreen), None);
        }
        "toggle_overlay" => toggle_gnomad(app, anchor),
        "new_chat" => {
            let _ = app.emit("menu-new-chat", ());
        }
        "open_knowledge" => {
            let _ = app.emit("menu-open-knowledge", ());
        }
        "cycle_theme" => {
            let _ = app.emit("menu-cycle-theme", ());
        }
        "open_settings" | "settings" => open_settings_panel(app, state),
        "visit_website" | "visit_website_help" => open_studio_website(app),
        "about" => {
            let _ = app.emit("menu-show-about", ());
        }
        "hide_to_tray" => hide_main_window(app),
        "quit" => request_app_quit(app),
        _ => {}
    }
}

fn request_app_quit(app: &tauri::AppHandle) {
    if let Some(state) = app.try_state::<AppLifecycleState>() {
        *state.quit_requested.lock().unwrap() = true;
    }
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppTrayState {
            last_anchor: Mutex::new(None),
        })
        .manage(AppLifecycleState {
            quit_requested: Mutex::new(false),
        })
        .manage(WindowRuntimeState::default())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new().with_handler(
                |app: &tauri::AppHandle,
                 _shortcut: &tauri_plugin_global_shortcut::Shortcut,
                 event: tauri_plugin_global_shortcut::ShortcutEvent| {
                    if event.state() == ShortcutState::Pressed {
                        let state = app.state::<AppTrayState>();
                        toggle_gnomad(app, last_anchor(&state).as_ref());
                    }
                },
            )
            .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            env_config::load_dotenv_files();
            let _ = knowledge::init_knowledge_store(&app.handle());

            let global_shortcut_manager = app.global_shortcut();

            #[cfg(target_os = "macos")]
            let shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Space);

            #[cfg(not(target_os = "macos"))]
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);

            let _ = global_shortcut_manager.register(shortcut);

            let app_menu = menu_shell::build_app_menu(app.handle())?;
            app.set_menu(app_menu)?;

            let tray_menu = build_tray_menu(app.handle())?;

            let tray_icon = load_tray_icon(app);
            let mut tray_builder = TrayIconBuilder::new()
                .icon(tray_icon)
                .tooltip("Gnomad 🦙 — click to open panel");
            #[cfg(target_os = "macos")]
            {
                tray_builder = tray_builder.icon_as_template(false);
            }
            let _tray = tray_builder
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    let app = tray.app_handle();
                    let state = app.state::<AppTrayState>();
                    if let Some(anchor) = window_manager::tray_anchor_from_event(&event) {
                        store_tray_anchor(&state, Some(anchor));
                    }
                    handle_tray_click(app, &event);
                })
                .on_menu_event(|app, event| {
                    let state = app.state::<AppTrayState>();
                    handle_menu_action(app, &state, event.id.as_ref());
                })
                .build(app)?;

            window_manager::sync_platform_shell(
                &app.handle(),
                window_manager::WindowDisplayMode::Panel,
            );

            Ok(())
        })
        .on_menu_event(|app, event| {
            let state = app.state::<AppTrayState>();
            handle_menu_action(app, &state, event.id.as_ref());
        })
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            check_accessibility_permissions,
            request_accessibility_permissions,
            context::get_active_window,
            context::get_clipboard_text,
            privilege::check_command_safety,
            privilege::execute_elevated_command,
            keychain::store_credential,
            keychain::get_credential,
            keychain::delete_credential,
            automation::capture_screen,
            automation::simulate_click,
            automation::simulate_typing,
            shell_executor::execute_shell_command,
            window_manager::set_window_mode,
            window_manager::get_window_mode,
            window_manager::fit_window_to_content,
            window_manager::mark_window_user_resized,
            knowledge::get_knowledge_root,
            knowledge::list_knowledge_files,
            knowledge::import_knowledge_files,
            knowledge::read_knowledge_file,
            knowledge::delete_knowledge_file,
            knowledge::append_user_preference,
            knowledge::get_agent_context_bundle,
            env_config::get_env_llm_config,
            env_config::has_llm_configured,
            env_config::get_effective_cloud_api_key,
            llm::chat_completion,
            chat_history::get_chat_store,
            chat_history::load_chat_session,
            chat_history::create_chat_session,
            chat_history::save_chat_session,
            chat_history::delete_chat_session,
            chat_history::set_active_chat_session,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let RunEvent::ExitRequested { api, .. } = event {
                let allow_quit = app
                    .try_state::<AppLifecycleState>()
                    .map(|s| *s.quit_requested.lock().unwrap())
                    .unwrap_or(false);
                if !allow_quit {
                    api.prevent_exit();
                    hide_main_window(app);
                }
            }
        });
}
