use tauri::{
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu},
    AppHandle, Runtime,
};

const STUDIO_URL: &str = "https://gnomadstudio.org";

pub fn about_metadata() -> AboutMetadata<'static> {
    AboutMetadata {
        name: Some("Gnomad".into()),
        version: Some("0.1.0".into()),
        copyright: Some("Built with ❤️ by Gnomad Studio 🦙".into()),
        credits: Some(format!("{STUDIO_URL}\n\n🦙 Local llama models · 🍄 Desktop assistant")),
        ..Default::default()
    }
}

fn hide_to_tray_label() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Hide to Menu Bar"
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Hide to System Tray"
    }
}

fn build_app_submenu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Submenu<R>> {
    #[cfg(target_os = "macos")]
    {
        return Submenu::with_items(
            app,
            "Gnomad",
            true,
            &[
                &PredefinedMenuItem::about(app, Some("About Gnomad"), Some(about_metadata()))?,
                &MenuItem::with_id(app, "visit_website", "Visit gnomadstudio.org…", true, None::<&str>)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::services(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::hide(app, None)?,
                &PredefinedMenuItem::hide_others(app, None)?,
                &PredefinedMenuItem::show_all(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "quit", "Quit Gnomad", true, Some("CmdOrCtrl+Q"))?,
            ],
        );
    }

    #[cfg(not(target_os = "macos"))]
    {
        Submenu::with_items(
            app,
            "Gnomad",
            true,
            &[
                &PredefinedMenuItem::about(app, Some("About Gnomad"), Some(about_metadata()))?,
                &MenuItem::with_id(app, "visit_website", "Visit gnomadstudio.org", true, None::<&str>)?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "quit", "Quit Gnomad", true, Some("CmdOrCtrl+Q"))?,
            ],
        )
    }
}

/// Full desktop menu bar (File, Edit, View, Window, Help) — used on macOS, Linux, and Windows.
pub fn build_app_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let hide_label = hide_to_tray_label();
    let app_submenu = build_app_submenu(app)?;

    let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &MenuItem::with_id(app, "new_chat", "New Chat", true, Some("CmdOrCtrl+N"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(
                app,
                "show_overlay",
                "Show Gnomad",
                true,
                Some("CmdOrCtrl+Shift+Space"),
            )?,
            &MenuItem::with_id(
                app,
                "hide_to_tray",
                hide_label,
                true,
                Some("CmdOrCtrl+W"),
            )?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "open_knowledge", "Knowledge & Skills…", true, None::<&str>)?,
            &MenuItem::with_id(app, "open_settings", "Settings…", true, Some("CmdOrCtrl+,"))?,
        ],
    )?;

    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;

    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &MenuItem::with_id(app, "mode_panel", "Menu Bar Panel", true, None::<&str>)?,
            &MenuItem::with_id(app, "mode_floating", "Pop Out (Floating)", true, None::<&str>)?,
            &MenuItem::with_id(app, "mode_windowed", "Windowed", true, None::<&str>)?,
            &MenuItem::with_id(app, "mode_fullscreen", "Fullscreen", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "toggle_overlay", "Toggle Visibility", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "cycle_theme", "Cycle Theme", true, None::<&str>)?,
        ],
    )?;

    let window_menu = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &MenuItem::with_id(app, "hide_to_tray", hide_label, true, Some("CmdOrCtrl+W"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::minimize(app, None)?,
        ],
    )?;

    let help_menu = Submenu::with_items(
        app,
        "Help",
        true,
        &[
            &MenuItem::with_id(app, "visit_website_help", "Visit gnomadstudio.org", true, None::<&str>)?,
            &PredefinedMenuItem::about(app, Some("About Gnomad"), Some(about_metadata()))?,
        ],
    )?;

    Menu::with_items(
        app,
        &[
            &app_submenu,
            &file_menu,
            &edit_menu,
            &view_menu,
            &window_menu,
            &help_menu,
        ],
    )
}
