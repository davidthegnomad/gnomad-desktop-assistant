use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, DropDown, Label, Orientation, ScrolledWindow, Stack, TextView,
};

#[cfg(not(feature = "vte"))]
use gtk4::Entry;

#[cfg(feature = "vte")]
use vte4::prelude::*;

use gnomad_core::agent::AgentActionRecord;

use crate::state::AppState;

#[cfg(feature = "vte")]
struct VteShellPane {
    scroll: ScrolledWindow,
    terminal: vte4::Terminal,
}

/// Live terminal panel: agent output replay + native VTE shell (lazy-loaded on first use).
#[derive(Clone)]
pub struct TerminalPanel {
    pub root: GtkBox,
    stack: Stack,
    text_view: TextView,
    #[cfg(feature = "vte")]
    vte_shell: Rc<RefCell<Option<VteShellPane>>>,
    shell_mode: Rc<Cell<bool>>,
    mode_dd: DropDown,
    #[cfg(not(feature = "vte"))]
    shell_row: GtkBox,
    #[cfg(not(feature = "vte"))]
    shell_entry: Entry,
}

impl TerminalPanel {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Vertical, 4);
        root.add_css_class("terminal-panel");
        root.set_visible(false);

        let header = GtkBox::new(Orientation::Horizontal, 8);
        let title = Label::new(Some("Terminal"));
        title.add_css_class("caption-heading");
        title.set_halign(gtk4::Align::Start);
        title.set_hexpand(true);
        header.append(&title);

        let mode_dd = DropDown::from_strings(&["Output", "Shell"]);
        mode_dd.add_css_class("pill-combo");
        header.append(&mode_dd);

        let clear_btn = Button::with_label("Clear");
        clear_btn.add_css_class("flat");
        header.append(&clear_btn);
        root.append(&header);

        let stack = Stack::new();
        stack.set_vexpand(false);
        stack.set_hexpand(true);

        let output_scroll = ScrolledWindow::builder()
            .height_request(160)
            .vexpand(false)
            .hexpand(true)
            .build();
        let text_view = TextView::new();
        text_view.set_editable(false);
        text_view.set_cursor_visible(false);
        text_view.set_monospace(true);
        text_view.add_css_class("terminal-view");
        text_view.set_wrap_mode(gtk4::WrapMode::None);
        output_scroll.set_child(Some(&text_view));
        stack.add_named(&output_scroll, Some("output"));

        #[cfg(feature = "vte")]
        {
            let placeholder = GtkBox::new(Orientation::Vertical, 0);
            placeholder.set_height_request(160);
            stack.add_named(&placeholder, Some("shell"));
        }

        #[cfg(not(feature = "vte"))]
        {
            let shell_scroll = ScrolledWindow::builder()
                .height_request(160)
                .vexpand(false)
                .hexpand(true)
                .build();
            let fallback = TextView::new();
            fallback.set_editable(false);
            fallback.set_monospace(true);
            fallback.add_css_class("terminal-view");
            shell_scroll.set_child(Some(&fallback));
            stack.add_named(&shell_scroll, Some("shell"));
        }

        stack.set_visible_child_name("output");
        root.append(&stack);

        let shell_mode = Rc::new(Cell::new(false));

        #[cfg(feature = "vte")]
        let vte_shell = Rc::new(RefCell::new(None::<VteShellPane>));

        #[cfg(not(feature = "vte"))]
        let (shell_row, shell_entry) = {
            let row = GtkBox::new(Orientation::Horizontal, 6);
            row.set_margin_start(4);
            row.set_margin_end(4);
            row.set_margin_bottom(4);
            row.set_visible(false);
            row.append(&Label::new(Some("$")));
            let entry = Entry::new();
            entry.set_hexpand(true);
            entry.set_placeholder_text(Some("Interactive shell — Enter to run"));
            entry.add_css_class("monospace");
            row.append(&entry);
            root.append(&row);
            (row, entry)
        };

        {
            let text_view = text_view.clone();
            #[cfg(feature = "vte")]
            let vte_shell = Rc::clone(&vte_shell);
            clear_btn.connect_clicked(move |_| {
                text_view.buffer().set_text("");
                #[cfg(feature = "vte")]
                if let Some(pane) = vte_shell.borrow().as_ref() {
                    pane.terminal.reset(true, true);
                }
            });
        }

        {
            let stack = stack.clone();
            let shell_mode = Rc::clone(&shell_mode);
            let mode_dd = mode_dd.clone();
            #[cfg(feature = "vte")]
            let vte_shell = Rc::clone(&vte_shell);
            #[cfg(not(feature = "vte"))]
            let shell_row = shell_row.clone();
            mode_dd.connect_notify_local(Some("selected"), move |dd, _| {
                let shell = dd.selected() == 1;
                shell_mode.set(shell);
                if shell {
                    #[cfg(feature = "vte")]
                    ensure_vte_shell(&stack, &vte_shell);
                    stack.set_visible_child_name("shell");
                } else {
                    stack.set_visible_child_name("output");
                }
                #[cfg(not(feature = "vte"))]
                shell_row.set_visible(shell);
            });
        }

        Self {
            root,
            stack,
            text_view,
            #[cfg(feature = "vte")]
            vte_shell,
            shell_mode,
            mode_dd,
            #[cfg(not(feature = "vte"))]
            shell_row,
            #[cfg(not(feature = "vte"))]
            shell_entry,
        }
    }

    pub fn attach_app_state(&self, state: Arc<AppState>) {
        let chat_tx = state.chat_tx.clone();
        state.shell_session.set_live_sink(Some(Arc::new(move |chunk: &str| {
            if chunk.is_empty() {
                return;
            }
            let _ = chat_tx.send(crate::state::ChatEvent::TerminalOutput {
                chunk: chunk.to_string(),
            });
        })));

        #[cfg(not(feature = "vte"))]
        {
            let shell_session_mode = Arc::clone(&state.shell_session);
            let paths_mode = state.core.paths.clone();
            let agent_settings_mode = Arc::clone(&state.agent_settings);
            let mode_dd = self.mode_dd.clone();
            mode_dd.connect_notify_local(Some("selected"), move |dd, _| {
                if dd.selected() == 1 {
                    let _ =
                        shell_session_mode.ensure_session(&paths_mode, &agent_settings_mode, None);
                }
            });
        }

        #[cfg(feature = "vte")]
        {
            let commit_wired = Rc::new(Cell::new(false));
            let shell_session = Arc::clone(&state.shell_session);
            let paths = state.core.paths.clone();
            let agent_settings = Arc::clone(&state.agent_settings);
            let vte_shell = Rc::clone(&self.vte_shell);
            let stack = self.stack.clone();
            let mode_dd = self.mode_dd.clone();
            mode_dd.connect_notify_local(Some("selected"), move |dd, _| {
                if dd.selected() != 1 {
                    return;
                }
                ensure_vte_shell(&stack, &vte_shell);
                let _ = shell_session.ensure_session(&paths, &agent_settings, None);
                if commit_wired.get() {
                    return;
                }
                let term = vte_shell
                    .borrow()
                    .as_ref()
                    .map(|p| p.terminal.clone())
                    .expect("vte shell pane");
                commit_wired.set(true);
                let shell_session = Arc::clone(&shell_session);
                let paths = paths.clone();
                let agent_settings = Arc::clone(&agent_settings);
                term.connect_commit(move |_term, text, _| {
                    let line = text.trim();
                    if line.is_empty() {
                        return;
                    }
                    let _ = shell_session.ensure_session(&paths, &agent_settings, None);
                    let _ = shell_session.write_line(line);
                });
            });
        }

        #[cfg(not(feature = "vte"))]
        {
            let shell_session_entry = Arc::clone(&state.shell_session);
            let paths_entry = state.core.paths.clone();
            let agent_settings_entry = Arc::clone(&state.agent_settings);
            let shell_entry = self.shell_entry.clone();
            let text_view = self.text_view.clone();
            shell_entry.connect_activate(move |entry| {
                let line = entry.text().to_string();
                if line.trim().is_empty() {
                    return;
                }
                let _ = shell_session_entry.ensure_session(
                    &paths_entry,
                    &agent_settings_entry,
                    None,
                );
                let mut end = text_view.buffer().end_iter();
                text_view
                    .buffer()
                    .insert(&mut end, &format!("\n$ {line}\n"));
                if let Err(err) = shell_session_entry.write_line(&line) {
                    let mut end = text_view.buffer().end_iter();
                    text_view.buffer().insert(&mut end, &format!("{err}\n"));
                }
                entry.set_text("");
            });
        }
    }

    pub fn set_panel_visible(&self, visible: bool) {
        self.root.set_visible(visible);
    }

    pub fn is_panel_visible(&self) -> bool {
        self.root.is_visible()
    }

    pub fn show_output_mode(&self) {
        self.shell_mode.set(false);
        self.mode_dd.set_selected(0);
        self.stack.set_visible_child_name("output");
    }

    pub fn begin_command(&self, command: &str) {
        self.feed_text(&format!("\n$ {command}\n"));
    }

    pub fn feed_text(&self, text: &str) {
        if text.is_empty() {
            return;
        }
        if self.shell_mode.get() {
            #[cfg(feature = "vte")]
            if let Some(pane) = self.vte_shell.borrow().as_ref() {
                pane.terminal.feed(text.as_bytes());
                return;
            }
        }
        self.append_to_text_view(text);
    }

    fn append_to_text_view(&self, text: &str) {
        let buffer = self.text_view.buffer();
        let mut end = buffer.end_iter();
        buffer.insert(&mut end, text);
        self.text_view.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
    }

    pub fn append_agent_actions(&self, actions: &[AgentActionRecord]) {
        for action in actions {
            if let Some(cmd) = &action.command_executed {
                self.begin_command(cmd);
                if let Some(res) = &action.command_result {
                    if !res.stdout.is_empty() {
                        self.feed_text(&res.stdout);
                    }
                    if !res.stderr.is_empty() {
                        self.feed_text(&res.stderr);
                    }
                }
            }
        }
    }

    pub fn clear(&self) {
        self.text_view.buffer().set_text("");
        #[cfg(feature = "vte")]
        if let Some(pane) = self.vte_shell.borrow().as_ref() {
            pane.terminal.reset(true, true);
        }
    }
}

#[cfg(feature = "vte")]
fn ensure_vte_shell(stack: &Stack, holder: &Rc<RefCell<Option<VteShellPane>>>) {
    if holder.borrow().is_some() {
        return;
    }
    let shell_scroll = ScrolledWindow::builder()
        .height_request(160)
        .vexpand(false)
        .hexpand(true)
        .build();
    let term = vte4::Terminal::new();
    term.set_input_enabled(true);
    term.set_hexpand(true);
    term.set_vexpand(true);
    term.set_can_focus(true);
    term.add_css_class("vte-terminal");
    term.set_font_scale(0.9);
    shell_scroll.set_child(Some(&term));

    if let Some(old) = stack.child_by_name("shell") {
        stack.remove(&old);
    }
    stack.add_named(&shell_scroll, Some("shell"));

    *holder.borrow_mut() = Some(VteShellPane {
        scroll: shell_scroll,
        terminal: term,
    });
}
