use std::cell::Cell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, DropDown, Label, Orientation, PasswordEntry, Stack, TextView,
};
use libadwaita as adw;
use libadwaita::prelude::AdwWindowExt;

use gnomad_core::config::env_config;
use gnomad_core::config::keychain;
use gnomad_core::config::paths::DataPaths;
use gnomad_core::config::{mark_onboarding_complete, should_show_onboarding};

use crate::chat::{CLOUD_MODELS, PROVIDER_LOCAL};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProviderChoice {
    Cloud,
    Local,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    ChooseProvider,
    Configure,
}

pub fn maybe_show_onboarding(
    parent: Option<&gtk4::Window>,
    paths: DataPaths,
    on_complete: Rc<dyn Fn(String, String)>,
) {
    if !should_show_onboarding(&paths) {
        return;
    }
    show_onboarding_dialog(parent, paths, on_complete);
}

pub fn show_onboarding_dialog(
    parent: Option<&gtk4::Window>,
    paths: DataPaths,
    on_complete: Rc<dyn Fn(String, String)>,
) {
    let provider = Rc::new(Cell::new(ProviderChoice::Cloud));
    let step = Rc::new(Cell::new(Step::ChooseProvider));

    let dialog = adw::Window::builder()
        .title("Welcome to Gnomad")
        .default_width(480)
        .default_height(520)
        .modal(true)
        .resizable(false)
        .build();
    if let Some(p) = parent {
        dialog.set_transient_for(Some(p));
    }

    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&Label::new(Some("Welcome to Gnomad 🍄"))));
    toolbar.add_top_bar(&header);

    let body = GtkBox::new(Orientation::Vertical, 16);
    body.set_margin_top(20);
    body.set_margin_bottom(20);
    body.set_margin_start(24);
    body.set_margin_end(24);
    body.set_vexpand(true);

    let subtitle = Label::new(Some(
        "Connect a cloud API or local Ollama. You can change this anytime in Settings.",
    ));
    subtitle.set_wrap(true);
    subtitle.add_css_class("dim-label");
    body.append(&subtitle);

    let stack = Stack::new();
    stack.set_vexpand(true);

    // Step 1 — provider cards
    let step1 = GtkBox::new(Orientation::Vertical, 12);
    let cloud_btn = Button::builder()
        .label("Cloud API — OpenAI-compatible (DeepSeek, etc.)")
        .build();
    cloud_btn.add_css_class("pill-button");
    cloud_btn.add_css_class("suggested-action");
    let local_btn = Button::builder()
        .label("Local model — Ollama on this machine")
        .build();
    local_btn.add_css_class("pill-button");
    step1.append(&cloud_btn);
    step1.append(&local_btn);
    stack.add_named(&step1, Some("choose"));

    // Step 2 — configuration
    let step2 = GtkBox::new(Orientation::Vertical, 10);
    let config_title = Label::new(Some("Configure your provider"));
    config_title.add_css_class("title-4");
    config_title.set_halign(gtk4::Align::Start);
    step2.append(&config_title);

    let cloud_panel = GtkBox::new(Orientation::Vertical, 8);
    let env_note = Label::new(None);
    env_note.set_wrap(true);
    env_note.add_css_class("dim-label");
    env_note.set_visible(false);
    cloud_panel.append(&env_note);

    let api_key_entry = PasswordEntry::new();
    api_key_entry.set_placeholder_text(Some("Paste cloud API key"));
    cloud_panel.append(&api_key_entry);

    let cloud_model_dd = DropDown::from_strings(CLOUD_MODELS);
    cloud_panel.append(&Label::new(Some("Default model")));
    cloud_panel.append(&cloud_model_dd);

    let local_panel = GtkBox::new(Orientation::Vertical, 8);
    let ollama_url = TextView::new();
    ollama_url.set_height_request(36);
    ollama_url.buffer().set_text("http://localhost:11434");
    local_panel.append(&Label::new(Some("Ollama server URL")));
    local_panel.append(&ollama_url);
    let local_model = TextView::new();
    local_model.set_height_request(36);
    local_model.buffer().set_text("llama3.2");
    local_panel.append(&Label::new(Some("Model name")));
    local_panel.append(&local_model);
    let ollama_hint = Label::new(Some("Run `ollama serve` and `ollama pull <model>` if needed."));
    ollama_hint.set_wrap(true);
    ollama_hint.add_css_class("dim-label");
    local_panel.append(&ollama_hint);

    let config_stack = Stack::new();
    config_stack.add_named(&cloud_panel, Some("cloud"));
    config_stack.add_named(&local_panel, Some("local"));
    step2.append(&config_stack);

    let error_label = Label::new(None);
    error_label.add_css_class("error");
    error_label.set_wrap(true);
    error_label.set_visible(false);
    step2.append(&error_label);

    stack.add_named(&step2, Some("configure"));
    body.append(&stack);

    let footer = GtkBox::new(Orientation::Horizontal, 8);
    footer.set_halign(gtk4::Align::End);

    let skip_btn = Button::with_label("Skip for now");
    skip_btn.add_css_class("pill-button");
    let back_btn = Button::with_label("Back");
    back_btn.add_css_class("pill-button");
    back_btn.set_visible(false);
    let next_btn = Button::with_label("Continue");
    next_btn.add_css_class("suggested-action");
    next_btn.add_css_class("pill-button");

    footer.append(&skip_btn);
    footer.append(&back_btn);
    footer.append(&next_btn);
    body.append(&footer);

    toolbar.set_content(Some(&body));
    dialog.set_content(Some(&toolbar));

    if env_config::cloud_api_key_from_env().is_some() {
        env_note.set_text("Cloud API key is set via environment (.env). You do not need to paste it here.");
        env_note.set_visible(true);
        api_key_entry.set_visible(false);
    }

    let show_step = {
        let stack = stack.clone();
        let back_btn = back_btn.clone();
        let next_btn = next_btn.clone();
        let config_stack = config_stack.clone();
        let provider = Rc::clone(&provider);
        move |s: Step| {
            match s {
                Step::ChooseProvider => {
                    stack.set_visible_child_name("choose");
                    back_btn.set_visible(false);
                    next_btn.set_label("Continue");
                }
                Step::Configure => {
                    stack.set_visible_child_name("configure");
                    back_btn.set_visible(true);
                    next_btn.set_label("Get started");
                    let name = if provider.get() == ProviderChoice::Cloud {
                        "cloud"
                    } else {
                        "local"
                    };
                    config_stack.set_visible_child_name(name);
                }
            }
        }
    };

    {
        let provider = Rc::clone(&provider);
        cloud_btn.connect_clicked(move |_| {
            provider.set(ProviderChoice::Cloud);
        });
    }
    {
        let provider = Rc::clone(&provider);
        local_btn.connect_clicked(move |_| {
            provider.set(ProviderChoice::Local);
        });
    }

    let step_rc = Rc::clone(&step);
    let show_step_continue = show_step.clone();
    next_btn.connect_clicked({
        let step = Rc::clone(&step_rc);
        let show_step = show_step_continue.clone();
        let error_label = error_label.clone();
        let provider = Rc::clone(&provider);
        let paths = paths.clone();
        let on_complete = Rc::clone(&on_complete);
        let dialog = dialog.clone();
        let api_key_entry = api_key_entry.clone();
        let cloud_model_dd = cloud_model_dd.clone();
        let ollama_url = ollama_url.clone();
        let local_model = local_model.clone();
        move |_| {
            error_label.set_visible(false);
            match step.get() {
                Step::ChooseProvider => {
                    step.set(Step::Configure);
                    show_step(Step::Configure);
                }
                Step::Configure => {
                    let (prov, model) = if provider.get() == ProviderChoice::Cloud {
                        if env_config::cloud_api_key_from_env().is_none() {
                            let key = api_key_entry.text().to_string();
                            if key.trim().is_empty() {
                                error_label.set_text(
                                    "Enter an API key or set OPENAI_API_KEY / DEEPSEEK_API_KEY in .env",
                                );
                                error_label.set_visible(true);
                                return;
                            }
                            if keychain::store_credential("llm_api_key", key.trim()).is_err() {
                                error_label.set_text("Failed to save API key to keychain.");
                                error_label.set_visible(true);
                                return;
                            }
                        }
                        let idx = cloud_model_dd.selected() as usize;
                        let model = CLOUD_MODELS
                            .get(idx)
                            .unwrap_or(&CLOUD_MODELS[0])
                            .to_string();
                        ("cloud", model)
                    } else {
                        let start = ollama_url.buffer().start_iter();
                        let end = ollama_url.buffer().end_iter();
                        let url = ollama_url
                            .buffer()
                            .text(&start, &end, true)
                            .trim()
                            .to_string();
                        if url.is_empty() {
                            error_label.set_text("Enter your Ollama server URL.");
                            error_label.set_visible(true);
                            return;
                        }
                        if keychain::store_credential("ollama_url", &url).is_err() {
                            error_label.set_text("Failed to save Ollama URL.");
                            error_label.set_visible(true);
                            return;
                        }
                        let start = local_model.buffer().start_iter();
                        let end = local_model.buffer().end_iter();
                        let model = local_model
                            .buffer()
                            .text(&start, &end, true)
                            .trim()
                            .to_string();
                        let model = if model.is_empty() {
                            "llama3.2".to_string()
                        } else {
                            model
                        };
                        (PROVIDER_LOCAL, model)
                    };
                    if mark_onboarding_complete(&paths, prov, &model).is_err() {
                        error_label.set_text("Failed to save preferences.");
                        error_label.set_visible(true);
                        return;
                    }
                    on_complete(prov.to_string(), model);
                    dialog.close();
                }
            }
        }
    });

    back_btn.connect_clicked({
        let step = Rc::clone(&step_rc);
        let show_step = show_step.clone();
        let error_label = error_label.clone();
        move |_| {
            step.set(Step::ChooseProvider);
            show_step(Step::ChooseProvider);
            error_label.set_visible(false);
        }
    });

    skip_btn.connect_clicked({
        let paths = paths.clone();
        let on_complete = Rc::clone(&on_complete);
        let dialog = dialog.clone();
        move |_| {
            let _ = mark_onboarding_complete(&paths, PROVIDER_LOCAL, "llama3.2");
            on_complete(PROVIDER_LOCAL.to_string(), "llama3.2".to_string());
            dialog.close();
        }
    });

    show_step(Step::ChooseProvider);
    dialog.present();
}
