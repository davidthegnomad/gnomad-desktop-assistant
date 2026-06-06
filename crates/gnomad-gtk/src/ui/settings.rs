use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, DropDown, Label, Orientation, PasswordEntry, ScrolledWindow, Switch,
};
use libadwaita as adw;
use libadwaita::prelude::AdwWindowExt;

use gnomad_core::agent::{
    read_settings, save_persisted, AgentSettings, AgentSettingsState, SudoAuthMode, TrustMode,
};
use gnomad_core::config::agent_secrets::{
    list_statuses, remove_entry, set_entry, sudo_password_configured, SUDO_PASSWORD_KEY,
};
use gnomad_core::config::keychain;
use gnomad_core::config::paths::DataPaths;
use gnomad_core::config::env_config;
use gnomad_core::knowledge::{list_files, open_knowledge_folder};

use crate::state::AppState;
use crate::ui::knowledge_panel::show_knowledge_panel;

const SUDO_DISCLAIMER: &str = "Warning: Storing your sudo password lets Gnomad run approved elevated commands without a system prompt. If you approve a malicious command, your password could be used to harm this machine. Only enable this if you understand the risk.";

struct SettingsUi {
    paths: DataPaths,
    agent_settings: Arc<AgentSettingsState>,
    workspace_label: Label,
    trust_dd: DropDown,
    secrets_switch: Switch,
    secrets_panel: GtkBox,
    sudo_mode_dd: DropDown,
    disclaimer: Label,
    sudo_pw_row: GtkBox,
    sudo_pw_status: Label,
    secrets_list: GtkBox,
    cloud_status: Label,
    knowledge_status: Label,
}

impl SettingsUi {
    fn refresh(&self) {
        let s = read_settings(&self.agent_settings);
        self.workspace_label.set_text(&format!(
            "Workspace: {}",
            s.workspace_root.display()
        ));
        self.trust_dd.set_selected(match s.trust_mode {
            TrustMode::Standard => 0,
            TrustMode::Yolo => 1,
        });
        self.secrets_switch.set_active(s.agent_secrets_enabled);
        self.secrets_panel.set_visible(s.agent_secrets_enabled);
        self.sudo_mode_dd.set_selected(match s.sudo_auth_mode {
            SudoAuthMode::Polkit => 0,
            SudoAuthMode::StoredPassword => 1,
        });
        let stored_pw = s.sudo_auth_mode == SudoAuthMode::StoredPassword;
        self.disclaimer.set_visible(stored_pw);
        self.sudo_pw_row.set_visible(stored_pw);
        self.sudo_pw_status.set_text(if sudo_password_configured(&self.paths) {
            "Sudo password is saved (not shown)."
        } else {
            "No sudo password saved."
        });
        let cloud_from_env = env_config::cloud_api_key_from_env().is_some();
        let cloud_from_keychain = keychain::has_credential("llm_api_key").unwrap_or(false);
        self.cloud_status.set_text(if cloud_from_env {
            "Cloud API key loaded from .env (not editable here)."
        } else if cloud_from_keychain {
            "Cloud API key saved in keychain (not shown)."
        } else {
            "No cloud API key configured."
        });
        match list_files(&self.paths) {
            Ok(files) => {
                self.knowledge_status
                    .set_text(&format!("{} file(s) in knowledge library.", files.len()));
            }
            Err(err) => self.knowledge_status.set_text(&err),
        }
        while let Some(child) = self.secrets_list.first_child() {
            self.secrets_list.remove(&child);
        }
        match list_statuses(&self.paths) {
            Ok(items) if items.is_empty() => {
                let empty = Label::new(Some("No secrets yet."));
                empty.add_css_class("dim-label");
                empty.set_halign(gtk4::Align::Start);
                self.secrets_list.append(&empty);
            }
            Ok(items) => {
                for item in items {
                    let row = GtkBox::new(Orientation::Horizontal, 8);
                    let name = Label::new(Some(&item.name));
                    name.set_hexpand(true);
                    name.set_halign(gtk4::Align::Start);
                    row.append(&name);
                    let remove = Button::with_label("Remove");
                    let paths = self.paths.clone();
                    let secret_name = item.name.clone();
                    let refresh = self.refresh_cb();
                    remove.connect_clicked(move |_| {
                        let _ = remove_entry(&paths, &secret_name);
                        refresh();
                    });
                    row.append(&remove);
                    self.secrets_list.append(&row);
                }
            }
            Err(_) => {
                let err = Label::new(Some("Could not load secrets."));
                err.add_css_class("dim-label");
                self.secrets_list.append(&err);
            }
        }
    }

    fn refresh_cb(&self) -> Rc<dyn Fn()> {
        let ui = self.clone_handles();
        Rc::new(move || ui.refresh())
    }

    fn clone_handles(&self) -> SettingsUi {
        SettingsUi {
            paths: self.paths.clone(),
            agent_settings: Arc::clone(&self.agent_settings),
            workspace_label: self.workspace_label.clone(),
            trust_dd: self.trust_dd.clone(),
            secrets_switch: self.secrets_switch.clone(),
            secrets_panel: self.secrets_panel.clone(),
            sudo_mode_dd: self.sudo_mode_dd.clone(),
            disclaimer: self.disclaimer.clone(),
            sudo_pw_row: self.sudo_pw_row.clone(),
            sudo_pw_status: self.sudo_pw_status.clone(),
            secrets_list: self.secrets_list.clone(),
            cloud_status: self.cloud_status.clone(),
            knowledge_status: self.knowledge_status.clone(),
        }
    }

    fn persist(&self, patch: impl FnOnce(&mut AgentSettings)) {
        let mut s = read_settings(&self.agent_settings);
        patch(&mut s);
        if let Ok(mut guard) = self.agent_settings.inner.lock() {
            *guard = s.clone();
        }
        let _ = save_persisted(&self.paths, &s);
        self.refresh();
    }
}

pub fn show_settings_dialog(parent: Option<&impl IsA<gtk4::Window>>, state: Arc<AppState>) {
    let win = adw::Window::new();
    win.set_title(Some("Settings"));
    win.set_default_size(520, 640);
    if let Some(p) = parent {
        win.set_transient_for(Some(p));
        win.set_modal(true);
    }

    let root = GtkBox::new(Orientation::Vertical, 0);
    let header = adw::HeaderBar::new();
    let close = Button::with_label("Close");
    {
        let win = win.clone();
        close.connect_clicked(move |_| win.close());
    }
    header.set_title_widget(Some(&Label::new(Some("Settings"))));
    header.pack_end(&close);
    root.append(&header);

    let scroll = ScrolledWindow::builder()
        .vexpand(true)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(16)
        .margin_end(16)
        .build();

    let body = GtkBox::new(Orientation::Vertical, 18);
    scroll.set_child(Some(&body));
    root.append(&scroll);
    win.set_content(Some(&root));

    body.append(&section_title("Cloud LLM API"));
    let cloud_intro = Label::new(Some(
        "API key for cloud chat (DeepSeek, OpenAI-compatible). Separate from agent secrets vault.",
    ));
    cloud_intro.set_wrap(true);
    cloud_intro.add_css_class("dim-label");
    body.append(&cloud_intro);
    let cloud_status = Label::new(None);
    cloud_status.set_halign(gtk4::Align::Start);
    cloud_status.add_css_class("dim-label");
    body.append(&cloud_status);
    let cloud_key_entry = PasswordEntry::new();
    cloud_key_entry.set_placeholder_text(Some("Paste cloud API key"));
    body.append(&cloud_key_entry);
    let cloud_save = Button::with_label("Save cloud API key");
    cloud_save.add_css_class("suggested-action");
    body.append(&cloud_save);

    body.append(&section_title("Knowledge library"));
    let knowledge_intro = Label::new(Some(
        "Skills, agent briefs, and reference files the model can use. Drop files into the knowledge folder.",
    ));
    knowledge_intro.set_wrap(true);
    knowledge_intro.add_css_class("dim-label");
    body.append(&knowledge_intro);
    let knowledge_status = Label::new(None);
    knowledge_status.set_halign(gtk4::Align::Start);
    knowledge_status.add_css_class("dim-label");
    body.append(&knowledge_status);
    let knowledge_actions = GtkBox::new(Orientation::Horizontal, 8);
    let manage_knowledge_btn = Button::with_label("Manage knowledge…");
    manage_knowledge_btn.add_css_class("suggested-action");
    knowledge_actions.append(&manage_knowledge_btn);
    let open_knowledge_btn = Button::with_label("Open knowledge folder");
    knowledge_actions.append(&open_knowledge_btn);
    body.append(&knowledge_actions);

    body.append(&section_title("Agent access"));
    let workspace_label = Label::new(None);
    workspace_label.set_halign(gtk4::Align::Start);
    workspace_label.set_wrap(true);
    workspace_label.add_css_class("dim-label");
    body.append(&workspace_label);

    let trust_row = settings_row("Trust mode");
    let trust_dd = DropDown::from_strings(&["Standard", "YOLO"]);
    trust_row.append(&trust_dd);
    body.append(&trust_row);

    body.append(&section_title("Agent secrets"));
    let secrets_intro = Label::new(Some(
        "Optional encrypted vault for API keys and passwords used by agent shell commands. Separate from cloud chat API keys.",
    ));
    secrets_intro.set_wrap(true);
    secrets_intro.set_halign(gtk4::Align::Start);
    secrets_intro.add_css_class("dim-label");
    body.append(&secrets_intro);

    let secrets_enable_row = settings_row("Enable agent secrets");
    let secrets_switch = Switch::new();
    secrets_enable_row.append(&secrets_switch);
    body.append(&secrets_enable_row);

    let secrets_panel = GtkBox::new(Orientation::Vertical, 12);
    secrets_panel.set_margin_start(12);
    body.append(&secrets_panel);

    let sudo_mode_row = settings_row("Sudo authentication");
    let sudo_mode_dd = DropDown::from_strings(&[
        "System dialog (pkexec)",
        "Saved sudo password",
    ]);
    sudo_mode_row.append(&sudo_mode_dd);
    secrets_panel.append(&sudo_mode_row);

    let disclaimer = Label::new(Some(SUDO_DISCLAIMER));
    disclaimer.set_wrap(true);
    disclaimer.set_halign(gtk4::Align::Start);
    disclaimer.add_css_class("warning");
    disclaimer.add_css_class("dim-label");
    secrets_panel.append(&disclaimer);

    let sudo_pw_row = GtkBox::new(Orientation::Vertical, 6);
    sudo_pw_row.append(&Label::new(Some("Sudo password")));
    let sudo_pw_entry = PasswordEntry::new();
    sudo_pw_entry.set_placeholder_text(Some("Enter sudo password"));
    sudo_pw_row.append(&sudo_pw_entry);
    let sudo_pw_status = Label::new(None);
    sudo_pw_status.set_halign(gtk4::Align::Start);
    sudo_pw_status.add_css_class("dim-label");
    sudo_pw_row.append(&sudo_pw_status);
    let sudo_pw_save = Button::with_label("Save sudo password");
    sudo_pw_save.add_css_class("suggested-action");
    sudo_pw_row.append(&sudo_pw_save);
    secrets_panel.append(&sudo_pw_row);

    secrets_panel.append(&section_title("Stored secrets"));
    let secrets_list = GtkBox::new(Orientation::Vertical, 6);
    secrets_panel.append(&secrets_list);
    let add_secret_btn = Button::with_label("Add secret");
    secrets_panel.append(&add_secret_btn);

    let ui = Rc::new(RefCell::new(SettingsUi {
        paths: state.core.paths.clone(),
        agent_settings: Arc::clone(&state.agent_settings),
        workspace_label,
        trust_dd: trust_dd.clone(),
        secrets_switch: secrets_switch.clone(),
        secrets_panel,
        sudo_mode_dd: sudo_mode_dd.clone(),
        disclaimer,
        sudo_pw_row,
        sudo_pw_status,
        secrets_list,
        cloud_status,
        knowledge_status,
    }));

    manage_knowledge_btn.connect_clicked({
        let paths = state.core.paths.clone();
        let ui = Rc::clone(&ui);
        let win = win.clone();
        move |_| {
            show_knowledge_panel(Some(&win), paths.clone());
            ui.borrow().refresh();
        }
    });

    open_knowledge_btn.connect_clicked({
        let paths = state.core.paths.clone();
        let ui = Rc::clone(&ui);
        move |_| {
            let _ = open_knowledge_folder(&paths);
            ui.borrow().refresh();
        }
    });

    cloud_save.connect_clicked({
        let ui = Rc::clone(&ui);
        let cloud_key_entry = cloud_key_entry.clone();
        move |_| {
            if env_config::cloud_api_key_from_env().is_some() {
                return;
            }
            let key = cloud_key_entry.text().to_string();
            if key.trim().is_empty() {
                return;
            }
            if keychain::store_credential("llm_api_key", key.trim()).is_ok() {
                cloud_key_entry.set_text("");
                ui.borrow().refresh();
            }
        }
    });

    trust_dd.connect_selected_notify({
        let ui = Rc::clone(&ui);
        move |dd| {
            let mode = if dd.selected() == 1 {
                TrustMode::Yolo
            } else {
                TrustMode::Standard
            };
            ui.borrow().persist(|s| s.trust_mode = mode);
        }
    });

    secrets_switch.connect_active_notify({
        let ui = Rc::clone(&ui);
        move |sw| {
            let on = sw.is_active();
            ui.borrow().persist(|s| s.agent_secrets_enabled = on);
        }
    });

    sudo_mode_dd.connect_selected_notify({
        let ui = Rc::clone(&ui);
        move |dd| {
            let mode = if dd.selected() == 1 {
                SudoAuthMode::StoredPassword
            } else {
                SudoAuthMode::Polkit
            };
            ui.borrow().persist(|s| s.sudo_auth_mode = mode);
        }
    });

    sudo_pw_save.connect_clicked({
        let ui = Rc::clone(&ui);
        let sudo_pw_entry = sudo_pw_entry.clone();
        move |_| {
            let pw = sudo_pw_entry.text().to_string();
            if pw.trim().is_empty() {
                return;
            }
            let paths = ui.borrow().paths.clone();
            if set_entry(&paths, SUDO_PASSWORD_KEY, &pw).is_ok() {
                sudo_pw_entry.set_text("");
                ui.borrow().refresh();
            }
        }
    });

    add_secret_btn.connect_clicked({
        let ui = Rc::clone(&ui);
        let win_ref = win.clone();
        move |_| {
            let paths = ui.borrow().paths.clone();
            let refresh = ui.borrow().refresh_cb();
            let parent_win: &gtk4::Window = win_ref.upcast_ref();
            show_add_secret_dialog(Some(parent_win), &paths, move || refresh());
        }
    });

    ui.borrow().refresh();
    win.present();
}

fn section_title(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class("title-4");
    label.set_halign(gtk4::Align::Start);
    label.set_margin_top(8);
    label
}

fn settings_row(label_text: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    let label = Label::new(Some(label_text));
    label.set_hexpand(true);
    label.set_halign(gtk4::Align::Start);
    row.append(&label);
    row
}

fn show_add_secret_dialog(
    parent: Option<&impl IsA<gtk4::Window>>,
    paths: &DataPaths,
    on_saved: impl Fn() + 'static,
) {
    let dialog = adw::Window::new();
    dialog.set_title(Some("Add secret"));
    dialog.set_default_size(420, 220);
    dialog.set_modal(true);
    if let Some(p) = parent {
        dialog.set_transient_for(Some(p));
    }

    let root = GtkBox::new(Orientation::Vertical, 12);
    root.set_margin_top(16);
    root.set_margin_bottom(16);
    root.set_margin_start(16);
    root.set_margin_end(16);

    let hint = Label::new(Some("Name must look like an env var (e.g. GITHUB_TOKEN)."));
    hint.set_wrap(true);
    hint.add_css_class("dim-label");
    root.append(&hint);

    root.append(&Label::new(Some("Name")));
    let name_entry = gtk4::Entry::new();
    name_entry.set_placeholder_text(Some("GITHUB_TOKEN"));
    root.append(&name_entry);

    root.append(&Label::new(Some("Value")));
    let value_entry = PasswordEntry::new();
    root.append(&value_entry);

    let actions = GtkBox::new(Orientation::Horizontal, 8);
    actions.set_halign(gtk4::Align::End);
    let cancel = Button::with_label("Cancel");
    let save = Button::with_label("Save");
    save.add_css_class("suggested-action");
    actions.append(&cancel);
    actions.append(&save);
    root.append(&actions);
    dialog.set_content(Some(&root));

    let paths_owned = paths.clone();
    {
        let dialog = dialog.clone();
        cancel.connect_clicked(move |_| dialog.close());
    }
    {
        let dialog = dialog.clone();
        save.connect_clicked(move |_| {
            let name = name_entry.text().to_string();
            let value = value_entry.text().to_string();
            if set_entry(&paths_owned, &name, &value).is_ok() {
                on_saved();
                dialog.close();
            }
        });
    }

    dialog.present();
}
