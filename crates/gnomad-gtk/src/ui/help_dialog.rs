use std::path::PathBuf;

use gtk4::prelude::*;
use gtk4::{Button, ScrolledWindow, TextView, Window};
use libadwaita as adw;
use libadwaita::prelude::AdwWindowExt;

fn help_editor_path() -> PathBuf {
    std::env::temp_dir().join(format!("gnomad-help-{}.md", std::process::id()))
}

/// Show the bundled quick-help guide in a small dialog.
pub fn show_help_dialog(parent: Option<&Window>) {
    let help = gnomad_core::help::quick_help_text();

    let dialog = adw::Window::builder()
        .title("Gnomad Help")
        .default_width(520)
        .default_height(480)
        .modal(true)
        .build();

    if let Some(p) = parent {
        dialog.set_transient_for(Some(p));
    }

    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&gtk4::Label::new(Some("Gnomad Help"))));

    let close_btn = Button::with_label("Close");
    close_btn.add_css_class("pill-button");
    let dlg = dialog.clone();
    close_btn.connect_clicked(move |_| dlg.close());
    header.pack_end(&close_btn);
    toolbar.add_top_bar(&header);

    let scroll = ScrolledWindow::builder()
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(16)
        .margin_end(16)
        .vexpand(true)
        .hexpand(true)
        .build();

    let text = TextView::new();
    text.set_editable(false);
    text.set_monospace(true);
    text.set_wrap_mode(gtk4::WrapMode::WordChar);
    text.buffer().set_text(help);
    scroll.set_child(Some(&text));
    toolbar.set_content(Some(&scroll));

    let footer = adw::HeaderBar::new();
    footer.add_css_class("bottom");

    let open_btn = Button::with_label("Open in editor");
    open_btn.set_tooltip_text(Some(
        "Save this guide to a temp file and open it in your default editor",
    ));
    open_btn.add_css_class("pill-button");
    let help_copy = help.to_string();
    open_btn.connect_clicked(move |_| {
        let path = help_editor_path();
        if std::fs::write(&path, &help_copy).is_ok() {
            let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
        }
    });
    footer.pack_start(&open_btn);

    let site_btn = Button::with_label("Gnomad Studio");
    site_btn.add_css_class("pill-button");
    site_btn.connect_clicked(|_| {
        let _ = std::process::Command::new("xdg-open")
            .arg("https://gnomadstudio.org")
            .spawn();
    });
    footer.pack_end(&site_btn);

    toolbar.add_bottom_bar(&footer);

    dialog.set_content(Some(&toolbar));
    dialog.present();
}
