use crate::shell_session::looks_like_shell_command;
use serde::Serialize;
use std::path::Path;

const PLANNER_SYSTEM: &str = r#"You convert user intent into exactly ONE executable shell command for macOS/Linux.
Rules:
- Output ONLY the command line. No markdown, no quotes around the whole line, no explanation.
- Use real CLI: brew, npm, command -v, ls, cat, git, etc.
- Never output English prose like "check if brew is installed".
Examples:
- intent: check if homebrew is installed -> command -v brew
- intent: install wget with brew -> brew install wget
- intent: list files here -> ls -la
"#;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandPlannerSettings {
    pub enabled: bool,
    pub model: String,
    pub use_chat_local_model: bool,
    pub gguf_path: String,
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

fn extract_command_line(raw: &str) -> String {
    let mut t = raw.trim().to_string();
    if t.starts_with("```") {
        t = t
            .trim_start_matches('`')
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n");
        if let Some(end) = t.rfind("```") {
            t.truncate(end);
        }
    }
    t.lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .unwrap_or("")
        .trim()
        .to_string()
}

async fn ollama_plan(
    base_url: &str,
    model: &str,
    intent: &str,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("{base_url}/api/chat");
    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": PLANNER_SYSTEM },
            { "role": "user", "content": intent }
        ],
        "stream": false,
        "options": { "temperature": 0.1 }
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Planner Ollama request failed: {e}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read planner response: {e}"))?;

    if !status.is_success() {
        return Err(format!("Planner Ollama error ({status}): {text}"));
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("Invalid planner JSON: {e}"))?;

    parsed["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Unexpected planner response: {text}"))
}

/// Turn natural-language intent into one shell command via a small local model (Ollama).
#[tauri::command]
pub async fn plan_shell_command(
    intent: String,
    model: String,
    ollama_url: Option<String>,
    gguf_path: Option<String>,
) -> Result<String, String> {
    let trimmed_intent = intent.trim();
    if trimmed_intent.is_empty() {
        return Err("Intent cannot be empty.".into());
    }

    if let Some(path) = gguf_path.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let p = Path::new(path);
        if p.extension().and_then(|e| e.to_str()) == Some("gguf") || p.exists() {
            return Err(
                "Direct GGUF inference is not enabled yet. Use an Ollama model name in Settings, or run `ollama create` from your GGUF file."
                    .into(),
            );
        }
    }

    let model = model.trim();
    if model.is_empty() {
        return Err("Planner model name is not configured.".into());
    }

    let base = ollama_base(ollama_url);
    let raw = ollama_plan(&base, model, trimmed_intent).await?;
    let command = extract_command_line(&raw);

    if command.is_empty() {
        return Err("Planner returned an empty command.".into());
    }
    if !looks_like_shell_command(&command) {
        return Err(format!(
            "Planner produced invalid shell syntax: \"{}\"",
            command.chars().take(120).collect::<String>()
        ));
    }

    Ok(command)
}
