use serde::{Deserialize, Serialize};

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
        return Err(
            "No cloud API key configured. Add DeepSeek_API_KEY to .env or set a key in Settings."
                .into(),
        );
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

async fn deepseek_chat_turn(
    api_key: &str,
    model: &str,
    api_messages: Vec<serde_json::Value>,
    use_tools: bool,
) -> Result<ChatTurnResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

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
        .map_err(|e| format!("DeepSeek request failed: {e}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read DeepSeek response: {e}"))?;

    if !status.is_success() {
        return Err(format_deepseek_error(status.as_u16(), &text));
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("Invalid DeepSeek JSON: {e}"))?;

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
        .map_err(|e| e.to_string())?;

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
        .map_err(|e| format!("DeepSeek request failed: {e}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read DeepSeek response: {e}"))?;

    if !status.is_success() {
        return Err(format_deepseek_error(status.as_u16(), &text));
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("Invalid DeepSeek JSON: {e}"))?;

    parsed["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Unexpected DeepSeek response shape: {text}"))
}

async fn ollama_chat(
    base_url: &str,
    model: &str,
    api_messages: Vec<serde_json::Value>,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| e.to_string())?;

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
        .map_err(|e| format!("Ollama request failed ({url}): {e}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read Ollama response: {e}"))?;

    if !status.is_success() {
        return Err(format!("Ollama error ({status}): {text}"));
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("Invalid Ollama JSON: {e}"))?;

    parsed["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Unexpected Ollama response shape: {text}"))
}

fn format_deepseek_error(status: u16, body: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(msg) = v["error"]["message"].as_str() {
            return format!("DeepSeek API error ({status}): {msg}");
        }
    }
    format!("DeepSeek API error ({status}): {body}")
}

#[tauri::command]
pub async fn chat_completion(
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
        return Err("No messages to send.".into());
    }

    let api_messages = build_api_messages(req.system_context, req.messages);

    let content = if req.provider == "local" {
        let base = ollama_base(req.ollama_url);
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
        deepseek_chat(&api_key, &model, api_messages).await?
    };

    Ok(ChatCompletionResponse { content })
}

#[tauri::command]
pub async fn chat_completion_turn(
    provider: String,
    model: String,
    messages: Vec<ChatMessage>,
    ollama_url: Option<String>,
    system_context: Option<String>,
    enable_tools: Option<bool>,
) -> Result<ChatTurnResponse, String> {
    if messages.is_empty() {
        return Err("No messages to send.".into());
    }

    let api_messages = build_api_messages(system_context, messages);
    let use_tools = enable_tools.unwrap_or(true);

    if provider == "local" {
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
