use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
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
