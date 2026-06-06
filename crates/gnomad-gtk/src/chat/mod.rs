pub mod approvals;
pub mod controller;

pub use controller::{
    spawn_agent_loop, spawn_chat_completion, spawn_ollama_discovery, ChatController, CLOUD_MODELS,
    PROVIDER_LOCAL,
};
