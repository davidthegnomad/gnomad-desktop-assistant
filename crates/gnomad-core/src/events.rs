use serde::Serialize;
use tokio::sync::broadcast;

/// Application events emitted by core services (replaces Tauri `emit()` for GTK shell).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum GnomadEvent {
    ShellOutput {
        chunk: String,
        stream: String,
    },
    ShellRunProgress {
        command: String,
        phase: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        exit_code: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    AgentStep {
        step: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
}

/// Broadcast bus for UI subscribers (GTK main loop or Tauri bridge).
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<GnomadEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity.max(16));
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<GnomadEvent> {
        self.sender.subscribe()
    }

    pub fn emit(&self, event: GnomadEvent) {
        let _ = self.sender.send(event);
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscriber_receives_emitted_event() {
        let bus = EventBus::new(8);
        let mut rx = bus.subscribe();
        bus.emit(GnomadEvent::ShellOutput {
            chunk: "hello".into(),
            stream: "stdout".into(),
        });
        let event = rx.try_recv().expect("event");
        match event {
            GnomadEvent::ShellOutput { chunk, .. } => assert_eq!(chunk, "hello"),
            _ => panic!("wrong variant"),
        }
    }
}
