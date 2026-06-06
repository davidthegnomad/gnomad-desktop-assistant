use gnomad_core::agent::{
    AgentActionRecord, AgentLoopResult, AgentRuntime, HitlTokenState, PathTokenState,
    AgentSettingsState,
};
use gnomad_core::chat::{
    api_content_for_message, create_chat_session, get_chat_store, load_chat_session,
    save_chat_session, set_active_chat_session, ChatAttachment, ChatSession, ChatStoreState,
    StoredChatMessage,
};
use gnomad_core::config::paths::DataPaths;
use gnomad_core::config::load_ui_prefs;
use gnomad_core::llm::{
    chat_completion, list_ollama_models, ChatCompletionRequest, ChatMessage, AGENT_SYSTEM_TOOLS,
    LOCAL_AGENT_JSON_INSTRUCTION,
};
use gnomad_core::shell::ShellOutputSink;
use std::sync::Arc;

use crate::chat::approvals::ChannelApprovals;
use crate::state::ChatEvent;

pub const PROVIDER_LOCAL: &str = "local";
pub const PROVIDER_CLOUD: &str = "cloud";

pub const CLOUD_MODELS: &[&str] = &["deepseek-chat", "deepseek-reasoner"];

pub struct ChatController {
    pub paths: DataPaths,
    pub store: ChatStoreState,
    pub session: ChatSession,
    pub provider: String,
    pub model: String,
    pub ollama_models: Vec<String>,
    pub agent_enabled: bool,
    pub thinking: bool,
    pub error: Option<String>,
}

impl ChatController {
    pub fn load(paths: DataPaths) -> Result<Self, String> {
        let store = get_chat_store(&paths)?;
        let active_id = store
            .active_id
            .clone()
            .or_else(|| store.sessions.first().map(|s| s.id.clone()))
            .ok_or_else(|| "No chat sessions available.".to_string())?;
        let session = load_chat_session(&paths, &active_id)?;
        let prefs = load_ui_prefs(&paths);
        let provider = prefs
            .provider
            .as_deref()
            .unwrap_or(PROVIDER_LOCAL)
            .to_string();
        let model = prefs.model.unwrap_or_default();
        Ok(Self {
            paths,
            store,
            session,
            provider,
            model,
            ollama_models: Vec::new(),
            agent_enabled: true,
            thinking: false,
            error: None,
        })
    }

    pub fn refresh_store(&mut self) -> Result<(), String> {
        self.store = get_chat_store(&self.paths)?;
        Ok(())
    }

    pub fn switch_session(&mut self, id: &str) -> Result<(), String> {
        set_active_chat_session(&self.paths, id)?;
        self.session = load_chat_session(&self.paths, id)?;
        self.refresh_store()?;
        self.error = None;
        Ok(())
    }

    pub fn new_session(&mut self) -> Result<(), String> {
        self.session = create_chat_session(&self.paths)?;
        self.refresh_store()?;
        self.error = None;
        Ok(())
    }

    pub fn set_provider(&mut self, local: bool) {
        self.provider = if local {
            PROVIDER_LOCAL.into()
        } else {
            PROVIDER_CLOUD.into()
        };
        self.model = self.default_model_for_provider();
    }

    pub fn set_model(&mut self, model: String) {
        self.model = model;
    }

    pub fn default_model_for_provider(&self) -> String {
        if self.provider == PROVIDER_LOCAL {
            self.ollama_models
                .first()
                .cloned()
                .unwrap_or_else(|| "llama3.2".into())
        } else {
            CLOUD_MODELS[0].to_string()
        }
    }

    pub fn effective_model(&self) -> String {
        if self.model.trim().is_empty() {
            self.default_model_for_provider()
        } else {
            self.model.clone()
        }
    }

    pub fn append_user_message(
        &mut self,
        text: String,
        attachments: &[ChatAttachment],
    ) -> Result<(), String> {
        let trimmed = text.trim();
        if trimmed.is_empty() && attachments.is_empty() {
            return Ok(());
        }
        let stored_attachments = if attachments.is_empty() {
            None
        } else {
            Some(attachments.to_vec())
        };
        self.session.messages.push(StoredChatMessage {
            role: "user".into(),
            text: trimmed.to_string(),
            command_executed: None,
            command_result: None,
            attachments: stored_attachments,
        });
        let _ = save_chat_session(&self.paths, &self.session.id, self.session.messages.clone())?;
        self.refresh_store()?;
        Ok(())
    }

    pub fn append_action_message(&mut self, action: &AgentActionRecord) -> Result<(), String> {
        let text = match &action.command_executed {
            None => action.label.clone(),
            Some(cmd) if action.label == *cmd => String::new(),
            Some(_) if action.error.is_some() => action.label.clone(),
            Some(_) => String::new(),
        };
        self.session.messages.push(StoredChatMessage {
            role: "assistant".into(),
            text,
            command_executed: action.command_executed.clone(),
            command_result: action.command_result.clone(),
            attachments: None,
        });
        let _ = save_chat_session(&self.paths, &self.session.id, self.session.messages.clone())?;
        self.refresh_store()?;
        Ok(())
    }

    pub fn append_agent_result(&mut self, result: &AgentLoopResult) -> Result<(), String> {
        for action in &result.actions {
            let _ = self.append_action_message(action);
        }
        self.append_assistant_message(result.final_text.clone())
    }

    pub fn append_assistant_message(&mut self, text: String) -> Result<(), String> {
        self.session.messages.push(StoredChatMessage {
            role: "assistant".into(),
            text,
            command_executed: None,
            command_result: None,
            attachments: None,
        });
        let _ = save_chat_session(&self.paths, &self.session.id, self.session.messages.clone())?;
        self.refresh_store()?;
        Ok(())
    }

    pub fn api_messages(&self) -> Vec<ChatMessage> {
        self.session
            .messages
            .iter()
            .filter_map(|m| {
                let role = m.role.as_str();
                if role != "user" && role != "assistant" {
                    return None;
                }
                let content = api_content_for_message(&m.text, m.attachments.as_deref());
                if content.trim().is_empty() {
                    return None;
                }
                Some(ChatMessage {
                    role: m.role.clone(),
                    content,
                    tool_call_id: None,
                    tool_calls: None,
                })
            })
            .collect()
    }

    pub fn begin_thinking(&mut self) {
        self.thinking = true;
        self.error = None;
    }

    pub fn end_thinking(&mut self) {
        self.thinking = false;
    }

    pub fn set_error(&mut self, message: String) {
        self.error = Some(message);
        self.thinking = false;
    }
}

pub fn spawn_ollama_discovery(chat_tx: std::sync::mpsc::Sender<crate::state::ChatEvent>) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        let models = rt
            .block_on(list_ollama_models(None))
            .map(|opts| opts.into_iter().map(|o| o.value).collect::<Vec<_>>())
            .unwrap_or_default();
        let _ = chat_tx.send(crate::state::ChatEvent::OllamaModels(models));
    });
}

fn merge_system_context(provider: &str, session_context: &str) -> Option<String> {
    let base = if provider == PROVIDER_LOCAL {
        format!(
            "{AGENT_SYSTEM_TOOLS}\n\n{LOCAL_AGENT_JSON_INSTRUCTION}"
        )
    } else {
        AGENT_SYSTEM_TOOLS.to_string()
    };
    let trimmed = session_context.trim();
    if trimmed.is_empty() {
        Some(base)
    } else {
        Some(format!("{base}\n\n--- Session context ---\n{trimmed}"))
    }
}

pub fn spawn_agent_loop(
    chat_tx: std::sync::mpsc::Sender<crate::state::ChatEvent>,
    paths: DataPaths,
    settings: std::sync::Arc<AgentSettingsState>,
    hitl: std::sync::Arc<HitlTokenState>,
    path_tokens: std::sync::Arc<PathTokenState>,
    shell_session: std::sync::Arc<gnomad_core::shell::ShellSessionState>,
    provider: String,
    model: String,
    messages: Vec<ChatMessage>,
    session_context: String,
) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        let _ = shell_session.ensure_session(&paths, &settings, None);
        let approvals = ChannelApprovals {
            chat_tx: chat_tx.clone(),
        };
        let stream_tx = chat_tx.clone();
        let terminal_sink: ShellOutputSink = Arc::new(move |chunk: &str| {
            if chunk.is_empty() {
                return;
            }
            let _ = stream_tx.send(ChatEvent::TerminalOutput {
                chunk: chunk.to_string(),
            });
        });
        let runtime = AgentRuntime {
            paths: &paths,
            settings: &settings,
            hitl: &hitl,
            path_tokens: &path_tokens,
            approvals: &approvals,
            terminal_sink: Some(Arc::clone(&terminal_sink)),
            shell_session: Some(shell_session.as_ref()),
        };
        let system_context = merge_system_context(&provider, &session_context);
        let result = rt.block_on(runtime.run_loop(
            ChatCompletionRequest {
                provider,
                model,
                messages,
                ollama_url: None,
                system_context,
            },
            true,
        ));
        let _ = chat_tx.send(crate::state::ChatEvent::AgentResult(result));
    });
}

pub fn spawn_chat_completion(
    chat_tx: std::sync::mpsc::Sender<crate::state::ChatEvent>,
    provider: String,
    model: String,
    messages: Vec<ChatMessage>,
    session_context: String,
) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        let system_context = {
            let trimmed = session_context.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(format!("--- Session context ---\n{trimmed}"))
            }
        };
        let result = rt.block_on(chat_completion(ChatCompletionRequest {
            provider,
            model,
            messages,
            ollama_url: None,
            system_context,
        }));
        let mapped = result.map(|r| r.content);
        let _ = chat_tx.send(crate::state::ChatEvent::CompletionResult(mapped));
    });
}
