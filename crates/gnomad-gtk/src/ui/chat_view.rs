use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{mpsc, Arc};

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, DropDown, Label, ListBox, ListBoxRow, Orientation, Paned, ScrolledWindow,
    StringList, Switch, TextView,
};
use gnomad_core::config::paths::DataPaths;
use libadwaita as adw;

use crate::chat::{
    spawn_agent_loop, spawn_chat_completion, spawn_ollama_discovery, ChatController, CLOUD_MODELS,
    PROVIDER_LOCAL,
};
use crate::ui::approval::{show_path_approval, show_shell_approval};
use crate::ui::attachments::AttachmentTray;
use crate::ui::context_pills::ContextPills;
use crate::ui::help_dialog::show_help_dialog;
use crate::ui::messages::{append_message_bubble, append_thinking_indicator};
use crate::ui::onboarding::maybe_show_onboarding;
use crate::ui::settings::show_settings_dialog;
use crate::ui::terminal::TerminalPanel;
use crate::ui::tooltips;
use crate::state::{AppState, ChatEvent, UiCommand};
use crate::window::DisplayMode;

#[derive(Clone)]
struct ChatWidgets {
    main_col: GtkBox,
    session_list: ListBox,
    messages_box: GtkBox,
    composer: TextView,
    send_btn: Button,
    agent_switch: Switch,
    provider_dd: DropDown,
    model_dd: DropDown,
    status_label: Label,
    settings_btn: Button,
    help_btn: Button,
    context_pills: ContextPills,
    attachments: AttachmentTray,
    terminal: TerminalPanel,
    terminal_toggle: Button,
}

pub fn build_chat_shell(
    state: Arc<AppState>,
    chat_rx: mpsc::Receiver<ChatEvent>,
) -> adw::ToolbarView {
    load_css();

    let paths = state.core.paths.clone();
    let controller = Rc::new(RefCell::new(
        ChatController::load(paths.clone()).expect("chat store"),
    ));

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&Label::new(Some("Gnomad 🍄"))));

    let mode_box = GtkBox::new(Orientation::Horizontal, 6);
    mode_box.add_css_class("linked");
    for mode in DisplayMode::ALL {
        let btn = Button::with_label(mode.label());
        btn.add_css_class("pill");
        btn.set_tooltip_text(Some(tooltips::mode_tooltip(mode)));
        let tx = state.cmd_tx.clone();
        btn.connect_clicked(move |_| {
            let _ = tx.send(UiCommand::SetMode(mode));
            let _ = tx.send(UiCommand::ShowPanel);
        });
        mode_box.append(&btn);
    }
    header.pack_end(&mode_box);
    toolbar_view.add_top_bar(&header);

    let paned = Paned::new(Orientation::Horizontal);
    paned.set_position(220);
    paned.set_resize_start_child(false);
    paned.set_shrink_start_child(false);

    let (sidebar, widgets) = build_chat_widgets(paths.clone());
    paned.set_start_child(Some(&sidebar));
    paned.set_end_child(Some(&widgets.main_col));
    toolbar_view.set_content(Some(&paned));

    refresh_all(&controller, &widgets.inner);

    wire_new_chat(&controller, &sidebar, &widgets);
    wire_sidebar(&controller, &widgets);
    wire_composer(&controller, &widgets, Arc::clone(&state));
    wire_provider(&controller, &widgets);
    wire_settings(&widgets, Arc::clone(&state));
    wire_help(&widgets);
    widgets
        .inner
        .terminal
        .attach_app_state(Arc::clone(&state));

    spawn_ollama_discovery(state.chat_tx.clone());

    wire_onboarding(
        &toolbar_view,
        state.core.paths.clone(),
        Rc::clone(&controller),
        widgets.inner.clone(),
    );

    widgets.inner.context_pills.refresh();
    let pills_poll = widgets.inner.context_pills.clone();
    glib::timeout_add_seconds_local(2, move || {
        pills_poll.refresh();
        glib::ControlFlow::Continue
    });

    let ctrl_poll = Rc::clone(&controller);
    let widgets_poll = widgets.inner.clone();
    let state_poll = Arc::clone(&state);
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        while let Ok(event) = chat_rx.try_recv() {
            let parent = widgets_poll
                .main_col
                .root()
                .and_then(|r| r.downcast::<gtk4::Window>().ok());
            match event {
                ChatEvent::OllamaModels(models) => {
                    {
                        let mut ctrl = ctrl_poll.borrow_mut();
                        ctrl.ollama_models = models;
                        if ctrl.provider == PROVIDER_LOCAL && ctrl.model.trim().is_empty() {
                            ctrl.model = ctrl.default_model_for_provider();
                        }
                    }
                    refresh_all(&ctrl_poll, &widgets_poll);
                }
                ChatEvent::CompletionResult(result) => {
                    {
                        let mut ctrl = ctrl_poll.borrow_mut();
                        match result {
                            Ok(content) => {
                                let _ = ctrl.append_assistant_message(content);
                                ctrl.end_thinking();
                            }
                            Err(err) => ctrl.set_error(err),
                        }
                    }
                    refresh_messages(&ctrl_poll, &widgets_poll.messages_box);
                    update_status(&ctrl_poll, &widgets_poll.status_label);
                }
                ChatEvent::TerminalCommandStart { command } => {
                    widgets_poll.terminal.show_output_mode();
                    widgets_poll.terminal.begin_command(&command);
                    if !widgets_poll.terminal.is_panel_visible() {
                        widgets_poll.terminal.set_panel_visible(true);
                        widgets_poll.terminal_toggle.set_label("Hide terminal");
                    }
                }
                ChatEvent::TerminalOutput { chunk } => {
                    widgets_poll.terminal.feed_text(&chunk);
                    if !widgets_poll.terminal.is_panel_visible() {
                        widgets_poll.terminal.set_panel_visible(true);
                        widgets_poll.terminal_toggle.set_label("Hide terminal");
                    }
                }
                ChatEvent::AgentResult(result) => {
                    {
                        let mut ctrl = ctrl_poll.borrow_mut();
                        match result {
                            Ok(agent) => {
                                let _ = ctrl.append_agent_result(&agent);
                                ctrl.end_thinking();
                            }
                            Err(err) => ctrl.set_error(err),
                        }
                    }
                    refresh_messages(&ctrl_poll, &widgets_poll.messages_box);
                    update_status(&ctrl_poll, &widgets_poll.status_label);
                }
                ChatEvent::ShellApproval {
                    command,
                    reason,
                    elevated,
                    reply,
                } => {
                    show_shell_approval(parent.as_ref(), &command, &reason, elevated, reply);
                }
                ChatEvent::PathApproval {
                    path,
                    reason,
                    scope,
                    reply,
                } => {
                    show_path_approval(
                        parent.as_ref(),
                        &state_poll.agent_settings,
                        &path,
                        &reason,
                        scope,
                        reply,
                    );
                }
                ChatEvent::OpenSettings => {
                    show_settings_dialog(parent.as_ref(), Arc::clone(&state_poll));
                }
            }
        }
        glib::ControlFlow::Continue
    });

    toolbar_view
}

struct ChatWidgetsBuilt {
    main_col: GtkBox,
    inner: ChatWidgets,
}

fn build_chat_widgets(paths: DataPaths) -> (GtkBox, ChatWidgetsBuilt) {
    let sidebar = GtkBox::new(Orientation::Vertical, 8);
    sidebar.set_margin_top(8);
    sidebar.set_margin_bottom(8);
    sidebar.set_margin_start(8);
    sidebar.set_margin_end(8);
    sidebar.set_width_request(200);

    let new_btn = Button::with_label("New chat");
    new_btn.add_css_class("suggested-action");
    new_btn.set_tooltip_text(Some(tooltips::NEW_CHAT));
    sidebar.append(&new_btn);

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .build();

    let session_list = ListBox::new();
    session_list.add_css_class("navigation-sidebar");
    session_list.set_selection_mode(gtk4::SelectionMode::Single);
    scroll.set_child(Some(&session_list));
    sidebar.append(&scroll);

    let sidebar_footer = GtkBox::new(Orientation::Horizontal, 0);
    sidebar_footer.set_margin_top(4);
    sidebar_footer.set_halign(gtk4::Align::Start);
    sidebar_footer.set_valign(gtk4::Align::End);

    let help_btn = Button::builder()
        .icon_name("help-about-symbolic")
        .tooltip_text(tooltips::HELP)
        .build();
    help_btn.add_css_class("flat");
    help_btn.add_css_class("sidebar-help");
    sidebar_footer.append(&help_btn);

    let settings_btn = Button::builder()
        .icon_name("preferences-system-symbolic")
        .tooltip_text(tooltips::SETTINGS)
        .build();
    settings_btn.add_css_class("flat");
    settings_btn.add_css_class("sidebar-settings");
    sidebar_footer.append(&settings_btn);
    sidebar.append(&sidebar_footer);

    let main_col = GtkBox::new(Orientation::Vertical, 0);
    main_col.set_vexpand(true);
    main_col.set_hexpand(true);

    let msg_scroll = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(16)
        .margin_end(16)
        .build();

    let messages_box = GtkBox::new(Orientation::Vertical, 10);
    messages_box.set_valign(gtk4::Align::Start);
    msg_scroll.set_child(Some(&messages_box));
    main_col.append(&msg_scroll);

    let status_label = Label::new(None);
    status_label.add_css_class("dim-label");
    status_label.set_margin_start(16);
    status_label.set_margin_end(16);
    status_label.set_halign(gtk4::Align::Start);
    status_label.set_wrap(true);
    main_col.append(&status_label);

    let terminal = TerminalPanel::new();
    main_col.append(&terminal.root);

    let composer_box = GtkBox::new(Orientation::Vertical, 8);
    composer_box.add_css_class("composer");
    composer_box.set_margin_start(16);
    composer_box.set_margin_end(16);
    composer_box.set_margin_bottom(12);

    let context_pills = ContextPills::new();
    composer_box.append(&context_pills.root);

    let attachments = AttachmentTray::new(paths.clone());
    composer_box.append(&attachments.root);

    let input_scroll = ScrolledWindow::builder().height_request(72).build();
    let composer = TextView::new();
    composer.set_wrap_mode(gtk4::WrapMode::WordChar);
    composer.add_css_class("composer-input");
    input_scroll.set_child(Some(&composer));
    composer_box.append(&input_scroll);

    let footer_row = GtkBox::new(Orientation::Horizontal, 8);
    footer_row.set_margin_top(4);
    footer_row.set_valign(gtk4::Align::Center);

    let access_box = GtkBox::new(Orientation::Horizontal, 6);
    access_box.add_css_class("access-toggle");
    let agent_label = Label::new(Some("Access local files"));
    agent_label.add_css_class("caption-heading");
    agent_label.set_tooltip_text(Some(tooltips::ACCESS_LOCAL));
    let agent_switch = Switch::new();
    agent_switch.set_active(true);
    agent_switch.set_valign(gtk4::Align::Center);
    agent_switch.set_tooltip_text(Some(tooltips::ACCESS_LOCAL));
    access_box.append(&agent_label);
    access_box.append(&agent_switch);
    footer_row.append(&access_box);

    let provider_dd = pill_dropdown_from_strings(&["Local", "Cloud"]);
    provider_dd.set_selected(0);
    provider_dd.set_tooltip_text(Some(tooltips::PROVIDER));
    footer_row.append(&provider_dd);

    let model_list = StringList::new(&["Loading…"]);
    let model_dd = DropDown::new(Some(model_list), gtk4::Expression::NONE);
    model_dd.add_css_class("pill-combo");
    model_dd.set_width_request(140);
    model_dd.set_tooltip_text(Some(tooltips::MODEL));
    footer_row.append(&model_dd);

    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    footer_row.append(&spacer);

    let terminal_toggle = Button::with_label("Terminal");
    terminal_toggle.add_css_class("pill-button");
    terminal_toggle.set_tooltip_text(Some(tooltips::TERMINAL));
    footer_row.append(&terminal_toggle);

    let attach_btn = Button::from_icon_name("mail-attachment-symbolic");
    attach_btn.add_css_class("pill-button");
    attach_btn.set_tooltip_text(Some(tooltips::ATTACH));
    footer_row.append(&attach_btn);

    let ctrl_hint = Label::new(Some("Ctrl+Enter"));
    ctrl_hint.add_css_class("composer-hint");
    ctrl_hint.set_valign(gtk4::Align::Center);
    footer_row.append(&ctrl_hint);

    let send_btn = Button::with_label("Send");
    send_btn.add_css_class("suggested-action");
    send_btn.add_css_class("pill-button");
    send_btn.set_tooltip_text(Some(tooltips::SEND));
    footer_row.append(&send_btn);

    composer_box.append(&footer_row);

    main_col.append(&composer_box);

    {
        let term = terminal.clone();
        let toggle = terminal_toggle.clone();
        terminal_toggle.connect_clicked(move |_| {
            let visible = !term.is_panel_visible();
            term.set_panel_visible(visible);
            if visible {
                toggle.set_label("Hide terminal");
                toggle.set_tooltip_text(Some(tooltips::TERMINAL_HIDE));
            } else {
                toggle.set_label("Terminal");
                toggle.set_tooltip_text(Some(tooltips::TERMINAL));
            }
        });
    }

    let main_col_ref = main_col.clone();
    let built = ChatWidgetsBuilt {
        main_col,
        inner: ChatWidgets {
            main_col: main_col_ref,
            session_list,
            messages_box,
            composer,
            send_btn,
            agent_switch,
            provider_dd,
            model_dd,
            status_label,
            settings_btn,
            help_btn,
            context_pills,
            attachments,
            terminal,
            terminal_toggle,
        },
    };

    {
        let tray = built.inner.attachments.clone();
        let anchor = built.inner.main_col.clone();
        attach_btn.connect_clicked(move |_| {
            tray.pick_files(&anchor);
        });
    }

    (sidebar, built)
}

fn wire_settings(widgets: &ChatWidgetsBuilt, state: Arc<AppState>) {
    let main_col = widgets.inner.main_col.clone();
    let settings = widgets.inner.settings_btn.clone();
    settings.connect_clicked(move |_| {
        let parent = main_col
            .root()
            .and_then(|r| r.downcast::<gtk4::Window>().ok());
        show_settings_dialog(parent.as_ref(), Arc::clone(&state));
    });
}

fn wire_help(widgets: &ChatWidgetsBuilt) {
    let main_col = widgets.inner.main_col.clone();
    let help = widgets.inner.help_btn.clone();
    help.connect_clicked(move |_| {
        let parent = main_col
            .root()
            .and_then(|r| r.downcast::<gtk4::Window>().ok());
        show_help_dialog(parent.as_ref());
    });
}

fn wire_onboarding(
    toolbar_view: &adw::ToolbarView,
    paths: gnomad_core::config::paths::DataPaths,
    controller: Rc<RefCell<ChatController>>,
    widgets: ChatWidgets,
) {
    let shell = toolbar_view.clone();
    glib::idle_add_local_once(move || {
        let parent = shell
            .root()
            .and_then(|r| r.downcast::<gtk4::Window>().ok());
        maybe_show_onboarding(
            parent.as_ref(),
            paths,
            Rc::new(move |provider, model| {
                let local = provider == crate::chat::PROVIDER_LOCAL;
                {
                    let mut ctrl = controller.borrow_mut();
                    ctrl.set_provider(local);
                    ctrl.set_model(model);
                }
                refresh_all(&controller, &widgets);
            }),
        );
    });
}

fn refresh_all(controller: &Rc<RefCell<ChatController>>, widgets: &ChatWidgets) {
    sync_provider_dropdown(controller, &widgets.provider_dd);
    refresh_sidebar(controller, &widgets.session_list);
    refresh_messages(controller, &widgets.messages_box);
    refresh_model_dropdown(controller, &widgets.model_dd);
    update_status(controller, &widgets.status_label);
}

fn sync_provider_dropdown(controller: &Rc<RefCell<ChatController>>, provider_dd: &DropDown) {
    let local = controller.borrow().provider == crate::chat::PROVIDER_LOCAL;
    provider_dd.set_selected(if local { 0 } else { 1 });
}

fn wire_new_chat(controller: &Rc<RefCell<ChatController>>, sidebar: &GtkBox, widgets: &ChatWidgetsBuilt) {
    let ctrl = Rc::clone(controller);
    let w = widgets.inner.clone();
    let list = w.session_list.clone();

    let new_btn = sidebar
        .first_child()
        .and_then(|child| child.downcast::<Button>().ok())
        .expect("new chat button");

    new_btn.connect_clicked(move |_| {
        if ctrl.borrow_mut().new_session().is_ok() {
            w.attachments.clear();
            refresh_sidebar(&ctrl, &list);
            refresh_messages(&ctrl, &w.messages_box);
            refresh_model_dropdown(&ctrl, &w.model_dd);
            update_status(&ctrl, &w.status_label);
        }
    });
}

fn wire_sidebar(controller: &Rc<RefCell<ChatController>>, widgets: &ChatWidgetsBuilt) {
    let ctrl = Rc::clone(controller);
    let w = widgets.inner.clone();
    let list = w.session_list.clone();

    list.connect_row_activated(move |lb, row| {
        let name = row.widget_name();
        let Some(id) = name.as_str().strip_prefix("session-") else {
            return;
        };
        if ctrl.borrow_mut().switch_session(id).is_ok() {
            w.attachments.clear();
            refresh_messages(&ctrl, &w.messages_box);
            update_status(&ctrl, &w.status_label);
            refresh_sidebar(&ctrl, lb);
        }
    });
}

fn wire_provider(controller: &Rc<RefCell<ChatController>>, widgets: &ChatWidgetsBuilt) {
    let ctrl = Rc::clone(controller);
    let model_dd = widgets.inner.model_dd.clone();
    let status = widgets.inner.status_label.clone();
    widgets.inner.provider_dd.connect_selected_notify(move |dd| {
        let local = dd.selected() == 0;
        ctrl.borrow_mut().set_provider(local);
        refresh_model_dropdown(&ctrl, &model_dd);
        update_status(&ctrl, &status);
    });
}

fn wire_composer(
    controller: &Rc<RefCell<ChatController>>,
    widgets: &ChatWidgetsBuilt,
    state: Arc<AppState>,
) {
    let ctrl = Rc::clone(controller);
    widgets.inner.model_dd.connect_selected_notify(move |dd| {
        if let Some(model) = dropdown_string_at(dd, dd.selected() as u32) {
            ctrl.borrow_mut().set_model(model);
        }
    });

    let ctrl_send = Rc::clone(controller);
    let w = widgets.inner.clone();
    let state_send = Arc::clone(&state);
    w.send_btn.clone().connect_clicked(move |_| {
        submit_message(&ctrl_send, &w, &state_send);
    });

    let ctrl_key = Rc::clone(controller);
    let w_key = widgets.inner.clone();
    let state_key = Arc::clone(&state);
    let key = gtk4::EventControllerKey::new();
    key.connect_key_pressed(move |_, keyval: gdk4::Key, _, modifier| {
        if keyval == gdk4::Key::Return
            && modifier.contains(gdk4::ModifierType::CONTROL_MASK)
        {
            submit_message(&ctrl_key, &w_key, &state_key);
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    widgets.inner.composer.add_controller(key);
}

fn submit_message(
    controller: &Rc<RefCell<ChatController>>,
    widgets: &ChatWidgets,
    state: &Arc<AppState>,
) {
    let buffer = widgets.composer.buffer();
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    let text = buffer.text(&start, &end, true).to_string();
    let attachments = widgets.attachments.pending();
    if text.trim().is_empty() && attachments.is_empty() {
        return;
    }

    let (api_messages, provider, model, agent_enabled) = {
        let mut ctrl = controller.borrow_mut();
        if ctrl.thinking {
            return;
        }
        let local = widgets.provider_dd.selected() == 0;
        ctrl.set_provider(local);
        if let Some(model) = dropdown_string_at(&widgets.model_dd, widgets.model_dd.selected() as u32)
        {
            ctrl.set_model(model);
        }
        ctrl.agent_enabled = widgets.agent_switch.is_active();
        if ctrl
            .append_user_message(text.clone(), &attachments)
            .is_err()
        {
            return;
        }
        ctrl.begin_thinking();
        (
            ctrl.api_messages(),
            ctrl.provider.clone(),
            ctrl.effective_model(),
            ctrl.agent_enabled,
        )
    };

    widgets.composer.buffer().set_text("");
    widgets.attachments.clear();
    widgets.context_pills.refresh();
    let desktop_context = widgets.context_pills.prompt_context();
    let session_context =
        gnomad_core::knowledge::build_full_session_context(&state.core.paths, &desktop_context);
    refresh_messages(controller, &widgets.messages_box);
    update_status(controller, &widgets.status_label);

    let chat_tx = state.chat_tx.clone();
    if agent_enabled {
        spawn_agent_loop(
            chat_tx,
            state.core.paths.clone(),
            Arc::clone(&state.agent_settings),
            Arc::clone(&state.hitl),
            Arc::clone(&state.path_tokens),
            Arc::clone(&state.shell_session),
            provider,
            model,
            api_messages,
            session_context,
        );
    } else {
        spawn_chat_completion(chat_tx, provider, model, api_messages, session_context);
    }
}

fn refresh_sidebar(controller: &Rc<RefCell<ChatController>>, list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    let ctrl = controller.borrow();
    let active = ctrl.session.id.clone();
    for summary in &ctrl.store.sessions {
        let row = ListBoxRow::new();
        row.set_widget_name(&format!("session-{}", summary.id));

        let row_box = GtkBox::new(Orientation::Vertical, 2);
        row_box.set_margin_top(6);
        row_box.set_margin_bottom(6);
        row_box.set_margin_start(8);
        row_box.set_margin_end(8);

        let title = Label::new(Some(&summary.title));
        title.add_css_class("heading");
        title.set_halign(gtk4::Align::Start);
        title.set_xalign(0.0);
        row_box.append(&title);

        if !summary.preview.is_empty() {
            let preview = Label::new(Some(&summary.preview));
            preview.add_css_class("dim-label");
            preview.set_halign(gtk4::Align::Start);
            preview.set_xalign(0.0);
            preview.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            preview.set_max_width_chars(24);
            row_box.append(&preview);
        }

        row.set_child(Some(&row_box));
        if summary.id == active {
            list.select_row(Some(&row));
        }
        list.append(&row);
    }
}

fn refresh_messages(controller: &Rc<RefCell<ChatController>>, messages_box: &GtkBox) {
    while let Some(child) = messages_box.first_child() {
        messages_box.remove(&child);
    }

    let ctrl = controller.borrow();
    for msg in &ctrl.session.messages {
        append_message_bubble(messages_box, msg);
    }
    if ctrl.thinking {
        append_thinking_indicator(messages_box);
    }
}

fn model_names(controller: &ChatController) -> Vec<String> {
    if controller.provider == PROVIDER_LOCAL {
        if controller.ollama_models.is_empty() {
            vec!["llama3.2".to_string()]
        } else {
            controller.ollama_models.clone()
        }
    } else {
        CLOUD_MODELS.iter().map(|s| (*s).to_string()).collect()
    }
}

fn refresh_model_dropdown(controller: &Rc<RefCell<ChatController>>, model_dd: &DropDown) {
    let (models, selected) = {
        let ctrl = controller.borrow();
        let models = model_names(&ctrl);
        let selected = models
            .iter()
            .position(|m| m == &ctrl.effective_model())
            .unwrap_or(0) as u32;
        (models, selected)
    };
    let refs: Vec<&str> = models.iter().map(|s| s.as_str()).collect();
    let list = StringList::new(&refs);
    model_dd.set_model(Some(&list));
    model_dd.set_selected(selected);
    model_dd.add_css_class("pill-combo");
}

fn pill_dropdown_from_strings(labels: &[&str]) -> DropDown {
    let dd = DropDown::from_strings(labels);
    dd.add_css_class("pill-combo");
    dd
}

fn dropdown_string_at(dd: &DropDown, index: u32) -> Option<String> {
    let model = dd.model()?;
    let item = model.item(index)?;
    let string_obj = item.downcast_ref::<gtk4::StringObject>()?;
    Some(string_obj.string().to_string())
}

fn update_status(controller: &Rc<RefCell<ChatController>>, status_label: &Label) {
    let ctrl = controller.borrow();
    if let Some(err) = &ctrl.error {
        status_label.set_label(&format!("Error: {err}"));
        status_label.add_css_class("error");
        return;
    }
    status_label.remove_css_class("error");
    if ctrl.thinking {
        status_label.set_label("Waiting for model response…");
        return;
    }
    let provider = if ctrl.provider == PROVIDER_LOCAL {
        "Ollama"
    } else {
        "Cloud"
    };
    let model = ctrl.effective_model();
    let agent = if ctrl.agent_enabled {
        " · local files on"
    } else {
        ""
    };
    if ctrl.provider == PROVIDER_LOCAL && ctrl.ollama_models.is_empty() {
        status_label.set_label(&format!(
            "{provider} · {model}{agent} (start `ollama serve` if models fail to load)"
        ));
    } else {
        status_label.set_label(&format!("{provider} · {model}{agent}"));
    }
}

fn load_css() {
    const CSS: &str = r#"
.composer {
  border-top: 1px solid alpha(@window_fg_color, 0.12);
  padding-top: 8px;
}
.composer-input {
  padding: 8px;
  border-radius: 12px;
}
.access-toggle {
  padding-right: 4px;
}
.composer-hint {
  font-size: 0.82em;
  opacity: 0.55;
}
.attachment-chips {
  margin-bottom: 2px;
}
.attachment-chip {
  border-radius: 999px;
  padding: 2px 4px 2px 10px;
  background: alpha(@window_fg_color, 0.08);
  border: 1px solid alpha(@window_fg_color, 0.12);
}
button.attachment-remove {
  min-width: 20px;
  min-height: 20px;
  padding: 0;
}
dropdown.pill-combo {
  border-radius: 999px;
  padding: 2px 4px;
  background: alpha(@window_fg_color, 0.06);
}
dropdown.pill-combo button {
  border-radius: 999px;
  padding: 4px 14px;
  min-height: 28px;
}
dropdown.pill-combo popover {
  border-radius: 12px;
}
button.pill-button {
  border-radius: 999px;
  padding: 6px 18px;
}
.message-user {
  background: alpha(@accent_bg_color, 0.35);
  border-radius: 12px;
  padding: 8px 12px;
}
.message-assistant {
  background: alpha(@window_fg_color, 0.06);
  border-radius: 12px;
  padding: 8px 12px;
}
button.pill {
  border-radius: 16px;
  padding: 4px 12px;
}
label.error {
  color: @destructive_color;
}
.command-card {
  border-radius: 8px;
  padding: 8px;
  background: alpha(@window_fg_color, 0.05);
  border: 1px solid alpha(@window_fg_color, 0.1);
}
.command-card.command-success {
  border-color: alpha(@success_color, 0.45);
}
.command-card.command-failed {
  border-color: alpha(@destructive_color, 0.45);
}
.command-status-icon {
  font-weight: bold;
  min-width: 1.2em;
}
.command-meta {
  font-size: 0.78em;
  opacity: 0.7;
}
.command-chip {
  font-family: monospace;
  font-size: 0.85em;
}
.command-output {
  font-family: monospace;
  font-size: 0.8em;
  opacity: 0.9;
  background: alpha(@window_fg_color, 0.06);
  border-radius: 6px;
  padding: 6px 8px;
}
.context-pills {
  /* GTK4 CSS has no flex-wrap; pills ellipsize on one row */
}
.context-pill {
  border-radius: 999px;
  padding: 4px 10px;
  background: alpha(@window_fg_color, 0.08);
}
.context-pill-title {
  font-size: 0.75em;
  opacity: 0.65;
}
.context-pill-value {
  font-size: 0.82em;
  font-family: monospace;
}
.terminal-panel {
  border-top: 1px solid alpha(@window_fg_color, 0.1);
  padding-top: 6px;
  margin-top: 4px;
}
.terminal-view {
  font-size: 0.82em;
  padding: 6px;
  background: alpha(@window_fg_color, 0.04);
}
.vte-terminal {
  padding: 4px;
  border-radius: 6px;
  background: alpha(@window_fg_color, 0.06);
}
button.sidebar-settings {
  min-width: 36px;
  min-height: 36px;
  padding: 6px;
  border-radius: 999px;
  opacity: 0.85;
}
button.sidebar-settings:hover {
  opacity: 1;
}
label.warning {
  color: @warning_color;
}
"#;
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(CSS);
    if let Some(display) = gdk4::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
