use gtk4::prelude::*;
use gtk4::Window;
use libadwaita as adw;
use libadwaita::prelude::MessageDialogExt;

use gnomad_core::agent::settings::read_settings;
use gnomad_core::agent::tokens::path::{issue_path_approval_token, PathScope};
use gnomad_core::agent::tokens::hitl::{issue_approval_token, HitlScope};
use gnomad_core::agent::AgentSettingsState;

pub fn show_shell_approval(
    parent: Option<&Window>,
    command: &str,
    reason: &str,
    elevated: bool,
    reply: std::sync::mpsc::SyncSender<Option<String>>,
) {
    let title = if elevated {
        "Approve elevated command?"
    } else {
        "Approve shell command?"
    };
    let intro = if elevated {
        "Gnomad wants to run an elevated (sudo/admin) command:"
    } else {
        "Gnomad wants to run:"
    };
    let body = format!(
        "{intro}\n\n{command}\n\nReason: {reason}\n\nAllow this command?"
    );
    let dialog = adw::MessageDialog::new(parent, Some(title), Some(&body));
    dialog.add_response("deny", "Deny");
    dialog.add_response("approve", "Allow");
    dialog.set_response_appearance("approve", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("deny"));
    dialog.set_close_response("deny");
    let cmd = command.to_string();
    let scope = if elevated {
        HitlScope::Elevated
    } else {
        HitlScope::ShellRun
    };
    dialog.connect_response(None::<&str>, move |_, response| {
        let token = if response == "approve" {
            issue_approval_token(&cmd, scope).ok()
        } else {
            None
        };
        let _ = reply.send(token);
    });
    dialog.present();
}

pub fn show_path_approval(
    parent: Option<&Window>,
    settings: &AgentSettingsState,
    path: &str,
    reason: &str,
    scope: PathScope,
    reply: std::sync::mpsc::SyncSender<Option<String>>,
) {
    let action = match scope {
        PathScope::Write => "write to",
        PathScope::Read => "access",
    };
    let body = format!(
        "Gnomad wants to {action}:\n\n{path}\n\n{reason}\n\nAllow this path?"
    );
    let dialog = adw::MessageDialog::new(parent, Some("Approve path access?"), Some(&body));
    dialog.add_response("deny", "Deny");
    dialog.add_response("approve", "Allow");
    dialog.set_response_appearance("approve", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("deny"));
    dialog.set_close_response("deny");
    let path_owned = path.to_string();
    let s = read_settings(settings);
    let workspace = s.workspace_root;
    let trust = s.trust_mode;
    dialog.connect_response(None::<&str>, move |_, response| {
        let token = if response == "approve" {
            issue_path_approval_token(&path_owned, scope, &workspace, trust).ok()
        } else {
            None
        };
        let _ = reply.send(token);
    });
    dialog.present();
}
