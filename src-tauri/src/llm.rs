use serde::{Deserialize, Serialize};
use tauri::State;

use crate::agent_settings::{local_gguf_chat_path, AgentSettingsState};
use crate::error::{into_invoke_err, GnomadError};
use crate::local_inference::{self, EmbeddedLlmState};

fn llm_err(message: impl Into<String>, detail: Option<String>) -> String {
    into_invoke_err(GnomadError::Llm {
        message: message.into(),
        detail,
    })
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<AgentToolCall>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCompletionRequest {
    pub provider: String,
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub ollama_url: Option<String>,
    pub system_context: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCompletionResponse {
    pub content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatTurnResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<AgentToolCall>,
}

pub fn agent_tool_definitions() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "shell_run",
                "description": "Run a shell command on the user's machine. Returns stdout, exit code, and state.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "One-line shell command" }
                    },
                    "required": ["command"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "workspace_info",
                "description": "Get workspace root, trust mode, and shell cwd.",
                "parameters": { "type": "object", "properties": {} }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "fs_list",
                "description": "List files in a directory relative to workspace.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Relative path or . for workspace root" }
                    }
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "fs_read",
                "description": "Read a text file relative to workspace.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    },
                    "required": ["path"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "fs_write",
                "description": "Write or overwrite a file relative to workspace.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["path", "content"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "fs_search",
                "description": "Search for text in files under a directory.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "path": { "type": "string" }
                    },
                    "required": ["query"]
                }
            }
        }),
    ]
}

fn cloud_api_key() -> Result<String, String> {
    if let Some(key) = crate::env_config::deepseek_api_key_from_env() {
        return Ok(key);
    }
    let key = crate::keychain::get_credential_value("llm_api_key")?;
    if key.trim().is_empty() {
        return Err(into_invoke_err(GnomadError::Keychain {
            message: "No cloud API key configured. Add DeepSeek_API_KEY to .env or set a key in Settings.".into(),
        }));
    }
    Ok(key)
}

fn ollama_base(url: Option<String>) -> String {
    if let Some(u) = url {
        let trimmed = u.trim();
        if !trimmed.is_empty() {
            return trimmed.trim_end_matches('/').to_string();
        }
    }
    crate::keychain::get_credential_value("ollama_url")
        .ok()
        .filter(|u| !u.trim().is_empty())
        .map(|u| u.trim().trim_end_matches('/').to_string())
        .unwrap_or_else(|| "http://localhost:11434".to_string())
}

fn build_api_messages(
    system_context: Option<String>,
    messages: Vec<ChatMessage>,
) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    if let Some(ctx) = system_context {
        let trimmed = ctx.trim();
        if !trimmed.is_empty() {
            out.push(serde_json::json!({
                "role": "system",
                "content": trimmed
            }));
        }
    }
    for m in messages {
        let role = m.role.trim();
        let content = m.content.trim();
        if role == "tool" {
            if let Some(id) = &m.tool_call_id {
                out.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": id,
                    "content": content
                }));
            }
            continue;
        }
        if let Some(calls) = &m.tool_calls {
            if !calls.is_empty() {
                let tc: Vec<serde_json::Value> = calls
                    .iter()
                    .map(|c| {
                        serde_json::json!({
                            "id": c.id,
                            "type": "function",
                            "function": {
                                "name": c.name,
                                "arguments": c.arguments
                            }
                        })
                    })
                    .collect();
                out.push(serde_json::json!({
                    "role": "assistant",
                    "content": if content.is_empty() { serde_json::Value::Null } else { serde_json::json!(content) },
                    "tool_calls": tc
                }));
                continue;
            }
        }
        if content.is_empty() {
            continue;
        }
        let role = match role {
            "user" | "assistant" | "system" => role,
            _ => "user",
        };
        out.push(serde_json::json!({ "role": role, "content": content }));
    }
    out
}

fn build_gguf_chat_prompt(api_messages: &[serde_json::Value]) -> String {
    let mut parts = Vec::new();
    for msg in api_messages {
        let role = msg["role"].as_str().unwrap_or("user");
        let content = match msg["content"].as_str() {
            Some(c) if !c.trim().is_empty() => c.trim(),
            _ => continue,
        };
        let label = match role {
            "system" => "System",
            "assistant" => "Assistant",
            "tool" => "Tool",
            _ => "User",
        };
        parts.push(format!("### {label}:\n{content}\n"));
    }
    parts.push("### Assistant:\n".to_string());
    parts.join("\n")
}

async fn gguf_local_chat(
    settings_state: &AgentSettingsState,
    embedded_state: &EmbeddedLlmState,
    api_messages: Vec<serde_json::Value>,
) -> Result<String, String> {
    let path = {
        let guard = settings_state.inner.lock().map_err(|e| {
            into_invoke_err(GnomadError::Internal {
                message: "Agent settings lock poisoned.".into(),
                detail: Some(e.to_string()),
            })
        })?;
        local_gguf_chat_path(&guard).ok_or_else(|| {
            llm_err(
                "Local GGUF chat is not configured. Enable it in Settings → Agent access and set a GGUF path.",
                None,
            )
        })?
    };

    let prompt = build_gguf_chat_prompt(&api_messages);
    let path_for_record = path.clone();
    let content = tokio::task::spawn_blocking(move || {
        local_inference::complete_with_gguf(&path, &prompt, 1024)
    })
    .await
    .map_err(|e| {
        into_invoke_err(GnomadError::Internal {
            message: "Embedded inference task failed.".into(),
            detail: Some(e.to_string()),
        })
    })??;

    local_inference::record_loaded(embedded_state, path_for_record);
    Ok(content.trim().to_string())
}

async fn deepseek_chat_turn(
    api_key: &str,
    model: &str,
    api_messages: Vec<serde_json::Value>,
    use_tools: bool,
) -> Result<ChatTurnResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| {
            into_invoke_err(GnomadError::Internal {
                message: "Failed to create HTTP client.".into(),
                detail: Some(e.to_string()),
            })
        })?;

    let mut body = serde_json::json!({
        "model": model,
        "messages": api_messages,
        "stream": false
    });
    if use_tools {
        body["tools"] = serde_json::json!(agent_tool_definitions());
        body["tool_choice"] = serde_json::json!("auto");
    }

    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| llm_err("DeepSeek request failed.", Some(e.to_string())))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| llm_err("Failed to read DeepSeek response.", Some(e.to_string())))?;

    if !status.is_success() {
        return Err(deepseek_api_error(status.as_u16(), &text));
    }

    let parsed: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        llm_err("Invalid DeepSeek JSON.", Some(format!("{e}; body: {}", text.chars().take(200).collect::<String>())))
    })?;

    let message = &parsed["choices"][0]["message"];
    let content = message["content"].as_str().map(|s| s.to_string());
    let mut tool_calls = Vec::new();
    if let Some(arr) = message["tool_calls"].as_array() {
        for tc in arr {
            let id = tc["id"].as_str().unwrap_or("").to_string();
            let name = tc["function"]["name"].as_str().unwrap_or("").to_string();
            let arguments = tc["function"]["arguments"].as_str().unwrap_or("{}").to_string();
            if !name.is_empty() {
                tool_calls.push(AgentToolCall {
                    id,
                    name,
                    arguments,
                });
            }
        }
    }

    Ok(ChatTurnResponse {
        content,
        tool_calls,
    })
}

async fn deepseek_chat(
    api_key: &str,
    model: &str,
    api_messages: Vec<serde_json::Value>,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| {
            into_invoke_err(GnomadError::Internal {
                message: "Failed to create HTTP client.".into(),
                detail: Some(e.to_string()),
            })
        })?;

    let body = serde_json::json!({
        "model": model,
        "messages": api_messages,
        "stream": false
    });

    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| llm_err("DeepSeek request failed.", Some(e.to_string())))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| llm_err("Failed to read DeepSeek response.", Some(e.to_string())))?;

    if !status.is_success() {
        return Err(deepseek_api_error(status.as_u16(), &text));
    }

    let parsed: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        llm_err("Invalid DeepSeek JSON.", Some(format!("{e}; body: {}", text.chars().take(200).collect::<String>())))
    })?;

    parsed["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| {
            llm_err(
                "Unexpected DeepSeek response shape.",
                Some(text.chars().take(200).collect()),
            )
        })
}

async fn ollama_chat(
    base_url: &str,
    model: &str,
    api_messages: Vec<serde_json::Value>,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| {
            into_invoke_err(GnomadError::Internal {
                message: "Failed to create HTTP client.".into(),
                detail: Some(e.to_string()),
            })
        })?;

    let url = format!("{base_url}/api/chat");
    let body = serde_json::json!({
        "model": model,
        "messages": api_messages,
        "stream": false
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            llm_err(
                "Ollama request failed. Is `ollama serve` running?",
                Some(format!("{url}: {e}")),
            )
        })?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| llm_err("Failed to read Ollama response.", Some(e.to_string())))?;

    if !status.is_success() {
        return Err(llm_err(
            format!("Ollama error ({status})."),
            Some(text.chars().take(300).collect()),
        ));
    }

    let parsed: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        llm_err("Invalid Ollama JSON.", Some(format!("{e}; body: {}", text.chars().take(200).collect::<String>())))
    })?;

    parsed["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| {
            llm_err(
                "Unexpected Ollama response shape.",
                Some(text.chars().take(200).collect()),
            )
        })
}

fn deepseek_api_error(status: u16, body: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(msg) = v["error"]["message"].as_str() {
            return llm_err(format!("DeepSeek API error ({status}): {msg}"), None);
        }
    }
    llm_err(
        format!("DeepSeek API error ({status})."),
        Some(body.chars().take(300).collect()),
    )
}

#[tauri::command]
pub async fn chat_completion(
    settings_state: State<'_, AgentSettingsState>,
    embedded_state: State<'_, EmbeddedLlmState>,
    provider: String,
    model: String,
    messages: Vec<ChatMessage>,
    ollama_url: Option<String>,
    system_context: Option<String>,
) -> Result<ChatCompletionResponse, String> {
    let req = ChatCompletionRequest {
        provider,
        model,
        messages,
        ollama_url,
        system_context,
    };

    if req.messages.is_empty() {
        return Err(llm_err("No messages to send.", None));
    }

    let api_messages = build_api_messages(req.system_context, req.messages);

    let content = if req.provider == "local" {
        let use_gguf = settings_state
            .inner
            .lock()
            .ok()
            .and_then(|s| local_gguf_chat_path(&s))
            .is_some();
        if use_gguf {
            gguf_local_chat(settings_state.inner(), embedded_state.inner(), api_messages).await?
        } else {
            let base = ollama_base(req.ollama_url);
            let model = if req.model.trim().is_empty() {
                "llama3.2".to_string()
            } else {
                req.model
            };
            ollama_chat(&base, &model, api_messages).await?
        }
    } else {
        let api_key = cloud_api_key()?;
        let model = if req.model.trim().is_empty() {
            "deepseek-chat".to_string()
        } else {
            req.model
        };
        deepseek_chat(&api_key, &model, api_messages).await?
    };

    Ok(ChatCompletionResponse { content })
}

#[tauri::command]
pub async fn chat_completion_turn(
    settings_state: State<'_, AgentSettingsState>,
    embedded_state: State<'_, EmbeddedLlmState>,
    provider: String,
    model: String,
    messages: Vec<ChatMessage>,
    ollama_url: Option<String>,
    system_context: Option<String>,
    enable_tools: Option<bool>,
) -> Result<ChatTurnResponse, String> {
    if messages.is_empty() {
        return Err(llm_err("No messages to send.", None));
    }

    let api_messages = build_api_messages(system_context, messages);
    let use_tools = enable_tools.unwrap_or(true);

    if provider == "local" {
        let use_gguf = settings_state
            .inner
            .lock()
            .ok()
            .and_then(|s| local_gguf_chat_path(&s))
            .is_some();
        if use_gguf {
            let content =
                gguf_local_chat(settings_state.inner(), embedded_state.inner(), api_messages)
                    .await?;
            return Ok(ChatTurnResponse {
                content: Some(content),
                tool_calls: vec![],
            });
        }
        let base = ollama_base(ollama_url);
        let model = if model.trim().is_empty() {
            "llama3.2".to_string()
        } else {
            model
        };
        let content = ollama_chat(&base, &model, api_messages).await?;
        return Ok(ChatTurnResponse {
            content: Some(content),
            tool_calls: vec![],
        });
    }

    let api_key = cloud_api_key()?;
    let model = if model.trim().is_empty() {
        "deepseek-chat".to_string()
    } else {
        model
    };
    deepseek_chat_turn(&api_key, &model, api_messages, use_tools).await
}
