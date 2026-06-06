use std::sync::mpsc;

use gnomad_core::agent::tokens::path::PathScope;
use gnomad_core::agent::AgentApprovals;

use crate::state::ChatEvent;

pub struct ChannelApprovals {
    pub chat_tx: mpsc::Sender<ChatEvent>,
}

impl AgentApprovals for ChannelApprovals {
    fn request_hitl(&self, command: &str, reason: &str, elevated: bool) -> Option<String> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        let _ = self.chat_tx.send(ChatEvent::ShellApproval {
            command: command.to_string(),
            reason: reason.to_string(),
            elevated,
            reply: reply_tx,
        });
        reply_rx.recv().ok().flatten()
    }

    fn terminal_command_start(&self, command: &str) {
        let _ = self.chat_tx.send(ChatEvent::TerminalCommandStart {
            command: command.to_string(),
        });
    }

    fn terminal_output(&self, chunk: &str) {
        if chunk.is_empty() {
            return;
        }
        let _ = self.chat_tx.send(ChatEvent::TerminalOutput {
            chunk: chunk.to_string(),
        });
    }

    fn request_path(&self, path: &str, reason: &str, scope: PathScope) -> Option<String> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        let _ = self.chat_tx.send(ChatEvent::PathApproval {
            path: path.to_string(),
            reason: reason.to_string(),
            scope,
            reply: reply_tx,
        });
        reply_rx.recv().ok().flatten()
    }
}
