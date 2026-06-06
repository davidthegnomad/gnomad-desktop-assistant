use crate::config::{env_config, keychain};
use crate::error::{into_invoke_err, GnomadError};

use super::ollama::ollama_base_url;
use super::tools::{agent_tool_definitions, parse_text_tool_calls};
use super::types::{
    AgentToolCall, ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ChatTurnResponse,
};

fn llm_err(message: impl Into<String>, detail: Option<String>) -> String {
    into_invoke_err(GnomadError::Llm {
        message: message.into(),
        detail,
    })
}

fn cloud_api_key() -> Result<String, String> {
    if let Some(key) = env_config::cloud_api_key_from_env() {
        return Ok(key);
    }
    let key = keychain::get_credential_value("llm_api_key")?;
    if key.trim().is_empty() {
        return Err(into_invoke_err(GnomadError::Keychain {
            message: "No cloud API key configured. Add an API key to .env or set one in Settings."
                .into(),
        }));
    }
    Ok(key)
}

fn cloud_chat_url() -> String {
    format!(
        "{}/chat/completions",
        env_config::resolved_cloud_api_base_url()
    )
}

pub fn build_api_messages(
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

async fn openai_compat_chat(
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

    let url = cloud_chat_url();
    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| llm_err("Cloud LLM request failed.", Some(e.to_string())))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| llm_err("Failed to read cloud LLM response.", Some(e.to_string())))?;

    if !status.is_success() {
        return Err(cloud_api_error(status.as_u16(), &text));
    }

    let parsed: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        llm_err(
            "Invalid cloud LLM JSON.",
            Some(format!(
                "{e}; body: {}",
                text.chars().take(200).collect::<String>()
            )),
        )
    })?;

    parsed["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| {
            llm_err(
                "Unexpected cloud LLM response shape.",
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
        llm_err(
            "Invalid Ollama JSON.",
            Some(format!(
                "{e}; body: {}",
                text.chars().take(200).collect::<String>()
            )),
        )
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

fn cloud_api_error(status: u16, body: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(msg) = v["error"]["message"].as_str() {
            return llm_err(format!("Cloud LLM API error ({status}): {msg}"), None);
        }
    }
    llm_err(
        format!("Cloud LLM API error ({status})."),
        Some(body.chars().take(300).collect()),
    )
}

pub async fn chat_completion(
    req: ChatCompletionRequest,
) -> Result<ChatCompletionResponse, String> {
    if req.messages.is_empty() {
        return Err(llm_err("No messages to send.", None));
    }

    let api_messages = build_api_messages(req.system_context, req.messages);

    let content = if req.provider == "local" {
        let base = ollama_base_url(req.ollama_url);
        let model = if req.model.trim().is_empty() {
            "llama3.2".to_string()
        } else {
            req.model
        };
        ollama_chat(&base, &model, api_messages).await?
    } else {
        let api_key = cloud_api_key()?;
        let model = if req.model.trim().is_empty() {
            "deepseek-chat".to_string()
        } else {
            req.model
        };
        openai_compat_chat(&api_key, &model, api_messages).await?
    };

    Ok(ChatCompletionResponse { content })
}

fn parse_tool_calls(message: &serde_json::Value) -> Vec<AgentToolCall> {
    let mut tool_calls = Vec::new();
    if let Some(arr) = message["tool_calls"].as_array() {
        for tc in arr {
            let id = tc["id"].as_str().unwrap_or("").to_string();
            let name = tc["function"]["name"].as_str().unwrap_or("").to_string();
            let arguments = tc["function"]["arguments"]
                .as_str()
                .unwrap_or("{}")
                .to_string();
            if !name.is_empty() {
                tool_calls.push(AgentToolCall {
                    id,
                    name,
                    arguments,
                });
            }
        }
    }
    tool_calls
}

async fn openai_compat_chat_turn(
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

    let url = cloud_chat_url();
    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| llm_err("Cloud LLM request failed.", Some(e.to_string())))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| llm_err("Failed to read cloud LLM response.", Some(e.to_string())))?;

    if !status.is_success() {
        return Err(cloud_api_error(status.as_u16(), &text));
    }

    let parsed: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        llm_err(
            "Invalid cloud LLM JSON.",
            Some(format!(
                "{e}; body: {}",
                text.chars().take(200).collect::<String>()
            )),
        )
    })?;

    let message = &parsed["choices"][0]["message"];
    let content = message["content"].as_str().map(|s| s.to_string());
    Ok(ChatTurnResponse {
        content,
        tool_calls: parse_tool_calls(message),
    })
}

async fn ollama_chat_turn(
    base_url: &str,
    model: &str,
    api_messages: Vec<serde_json::Value>,
    use_tools: bool,
) -> Result<ChatTurnResponse, String> {
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
    let mut body = serde_json::json!({
        "model": model,
        "messages": api_messages,
        "stream": false
    });
    if use_tools {
        body["tools"] = serde_json::json!(agent_tool_definitions());
    }

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
        llm_err(
            "Invalid Ollama JSON.",
            Some(format!(
                "{e}; body: {}",
                text.chars().take(200).collect::<String>()
            )),
        )
    })?;

    let message = &parsed["message"];
    let content = message["content"].as_str().map(|s| s.to_string());
    Ok(ChatTurnResponse {
        content,
        tool_calls: parse_tool_calls(message),
    })
}

/// Ollama does not accept OpenAI-style `tool` messages; flatten them for chat history.
fn flatten_messages_for_ollama(messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
    let mut out = Vec::new();
    for m in messages {
        match m.role.as_str() {
            "tool" => {
                let label = m.tool_call_id.as_deref().unwrap_or("tool");
                out.push(ChatMessage {
                    role: "user".into(),
                    content: format!("Tool result ({label}): {}", m.content),
                    tool_call_id: None,
                    tool_calls: None,
                });
            }
            "assistant" if m.tool_calls.as_ref().is_some_and(|c| !c.is_empty()) => {
                let names: Vec<&str> = m
                    .tool_calls
                    .as_ref()
                    .map(|calls| calls.iter().map(|c| c.name.as_str()).collect())
                    .unwrap_or_default();
                let content = if m.content.trim().is_empty() {
                    format!("[Called tools: {}]", names.join(", "))
                } else {
                    format!("{}\n[Called tools: {}]", m.content, names.join(", "))
                };
                out.push(ChatMessage {
                    role: "assistant".into(),
                    content,
                    tool_call_id: None,
                    tool_calls: None,
                });
            }
            _ => out.push(m),
        }
    }
    out
}

pub async fn chat_completion_turn(
    req: ChatCompletionRequest,
    enable_tools: bool,
) -> Result<ChatTurnResponse, String> {
    if req.messages.is_empty() {
        return Err(llm_err("No messages to send.", None));
    }

    if req.provider == "local" {
        let base = ollama_base_url(req.ollama_url);
        let model = if req.model.trim().is_empty() {
            "llama3.2".to_string()
        } else {
            req.model
        };
        let flat = flatten_messages_for_ollama(req.messages);
        let api_messages = build_api_messages(req.system_context, flat);
        // Most Ollama models (incl. Gemma) reject native tools — use JSON-in-text parsing.
        let mut turn = ollama_chat_turn(&base, &model, api_messages, false).await?;
        if enable_tools && turn.tool_calls.is_empty() {
            if let Some(content) = &turn.content {
                let parsed = parse_text_tool_calls(content);
                if !parsed.is_empty() {
                    turn.tool_calls = parsed;
                }
            }
        }
        return Ok(turn);
    }

    let api_messages = build_api_messages(req.system_context, req.messages);
    let use_tools = enable_tools;

    let api_key = cloud_api_key()?;
    let model = if req.model.trim().is_empty() {
        "deepseek-chat".to_string()
    } else {
        req.model
    };
    openai_compat_chat_turn(&api_key, &model, api_messages, use_tools).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_user_assistant_messages() {
        let msgs = vec![
            ChatMessage {
                role: "user".into(),
                content: "Hi".into(),
                tool_call_id: None,
                tool_calls: None,
            },
            ChatMessage {
                role: "assistant".into(),
                content: "Hello".into(),
                tool_call_id: None,
                tool_calls: None,
            },
        ];
        let api = build_api_messages(None, msgs);
        assert_eq!(api.len(), 2);
        assert_eq!(api[0]["role"], "user");
        assert_eq!(api[1]["role"], "assistant");
    }
}
