pub mod completion;
pub mod ollama;
pub mod tools;
pub mod types;

pub use completion::{build_api_messages, chat_completion, chat_completion_turn};
pub use ollama::{list_ollama_models, ollama_base_url, OllamaModelOption};
pub use tools::{
    agent_tool_definitions, parse_text_tool_calls, AGENT_SYSTEM_TOOLS, LOCAL_AGENT_JSON_INSTRUCTION,
};
pub use types::{
    AgentToolCall, ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ChatTurnResponse,
};
