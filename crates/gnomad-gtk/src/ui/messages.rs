use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Expander, Label, Orientation};

use gnomad_core::chat::{display_text_for_user_message, StoredChatMessage, StoredCommandResult};

pub fn append_message_bubble(messages_box: &GtkBox, msg: &StoredChatMessage) {
    let bubble = GtkBox::new(Orientation::Vertical, 6);
    bubble.set_margin_top(4);
    bubble.set_margin_bottom(4);
    bubble.set_halign(if msg.role == "user" {
        gtk4::Align::End
    } else {
        gtk4::Align::Start
    });
    bubble.set_width_request(300);
    bubble.add_css_class(if msg.role == "user" {
        "message-user"
    } else {
        "message-assistant"
    });

    let role = Label::new(Some(if msg.role == "user" {
        "You"
    } else {
        "Gnomad"
    }));
    role.add_css_class("caption-heading");
    role.set_halign(gtk4::Align::Start);
    role.set_xalign(0.0);
    bubble.append(&role);

    let body_trimmed = if msg.role == "user" {
        display_text_for_user_message(
            &msg.text,
            msg.attachments.as_deref().unwrap_or(&[]),
        )
    } else {
        msg.text.trim().to_string()
    };
    if !body_trimmed.is_empty() && msg.command_executed.is_none() {
        let body = Label::new(Some(&body_trimmed));
        body.set_wrap(true);
        body.set_halign(gtk4::Align::Start);
        body.set_xalign(0.0);
        body.set_selectable(true);
        bubble.append(&body);
    }

    if let Some(cmd) = &msg.command_executed {
        bubble.append(&build_command_card(cmd, msg.command_result.as_ref(), &msg.text));
    }

    messages_box.append(&bubble);
}

pub fn append_thinking_indicator(messages_box: &GtkBox) {
    let thinking = Label::new(Some("Thinking…"));
    thinking.add_css_class("dim-label");
    thinking.set_halign(gtk4::Align::Start);
    messages_box.append(&thinking);
}

fn build_command_card(
    command: &str,
    result: Option<&StoredCommandResult>,
    label: &str,
) -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 4);
    card.add_css_class("command-card");
    if let Some(res) = result {
        if res.success {
            card.add_css_class("command-success");
        } else {
            card.add_css_class("command-failed");
        }
    }

    let header = GtkBox::new(Orientation::Horizontal, 8);
    let status = Label::new(Some(command_status_icon(result)));
    status.add_css_class("command-status-icon");
    header.append(&status);

    let cmd_label = Label::new(Some(&format!("$ {command}")));
    cmd_label.add_css_class("command-chip");
    cmd_label.set_hexpand(true);
    cmd_label.set_halign(gtk4::Align::Start);
    cmd_label.set_xalign(0.0);
    cmd_label.set_wrap(true);
    cmd_label.set_selectable(true);
    header.append(&cmd_label);

    if let Some(res) = result {
        if let Some(code) = res.status_code {
            let exit = Label::new(Some(&format!("exit {code}")));
            exit.add_css_class("command-meta");
            header.append(&exit);
        }
        if let Some(ms) = res.duration_ms.filter(|&d| d > 0) {
            let dur = Label::new(Some(&format!("{:.1}s", ms as f64 / 1000.0)));
            dur.add_css_class("command-meta");
            header.append(&dur);
        }
    }
    card.append(&header);

    if let Some(res) = result {
        if let Some(state) = &res.state {
            let state_label = Label::new(Some(state));
            state_label.add_css_class("command-meta");
            state_label.set_halign(gtk4::Align::Start);
            card.append(&state_label);
        }
        if let Some(cwd_path) = res.cwd.as_ref().filter(|c| !c.is_empty()) {
            let cwd = Label::new(Some(&format!("cwd: {cwd_path}")));
            cwd.add_css_class("command-meta");
            cwd.set_halign(gtk4::Align::Start);
            cwd.set_wrap(true);
            card.append(&cwd);
        }
        if let Some(msg) = &res.message {
            if !msg.is_empty() && res.stdout.is_empty() && res.stderr.is_empty() {
                let message = Label::new(Some(msg));
                message.add_css_class("dim-label");
                message.set_halign(gtk4::Align::Start);
                message.set_wrap(true);
                card.append(&message);
            }
        }
    } else if !label.trim().is_empty() {
        let hint = Label::new(Some(label.trim()));
        hint.add_css_class("dim-label");
        hint.set_halign(gtk4::Align::Start);
        hint.set_wrap(true);
        card.append(&hint);
    }

    if let Some(res) = result {
        let output = command_output_text(res);
        if !output.is_empty() {
            let lines: Vec<&str> = output.lines().collect();
            let preview: String = lines.iter().take(8).copied().collect::<Vec<_>>().join("\n");
            if lines.len() <= 8 {
                let out = Label::new(Some(&preview));
                out.add_css_class("command-output");
                out.set_wrap(true);
                out.set_halign(gtk4::Align::Start);
                out.set_xalign(0.0);
                out.set_selectable(true);
                card.append(&out);
            } else {
                let expander = Expander::new(Some("Show output"));
                expander.set_expanded(false);
                let out = Label::new(Some(&output));
                out.add_css_class("command-output");
                out.set_wrap(true);
                out.set_halign(gtk4::Align::Start);
                out.set_xalign(0.0);
                out.set_selectable(true);
                expander.set_child(Some(&out));
                card.append(&expander);
            }
        }
    }

    card
}

fn command_status_icon(result: Option<&StoredCommandResult>) -> &'static str {
    match result {
        Some(r) if r.success => "✓",
        Some(_) => "✗",
        None => "›",
    }
}

fn command_output_text(res: &StoredCommandResult) -> String {
    if !res.stderr.trim().is_empty() {
        res.stderr.trim().to_string()
    } else {
        res.stdout.trim().to_string()
    }
}
