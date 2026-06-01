use crate::error::{into_invoke_err, GnomadError};
use crate::local_inference;
use crate::shell_session::looks_like_shell_command;
use serde::Serialize;
use std::path::PathBuf;

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

fn planner_err(message: impl Into<String>, detail: Option<String>) -> String {
    into_invoke_err(GnomadError::Llm {
        message: message.into(),
        detail,
    })
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
        .map_err(|e| {
            into_invoke_err(GnomadError::Internal {
                message: "Failed to create HTTP client for planner.".into(),
                detail: Some(e.to_string()),
            })
        })?;

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
        .map_err(|e| {
            planner_err(
                "Planner Ollama request failed. Is `ollama serve` running?",
                Some(format!("{url}: {e}")),
            )
        })?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| planner_err("Failed to read planner response.", Some(e.to_string())))?;

    if !status.is_success() {
        return Err(planner_err(
            format!("Planner Ollama error ({status})."),
            Some(text.chars().take(300).collect()),
        ));
    }

    let parsed: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        planner_err(
            "Invalid planner JSON.",
            Some(format!("{e}; body: {}", text.chars().take(200).collect::<String>())),
        )
    })?;

    parsed["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| {
            planner_err(
                "Unexpected planner response shape.",
                Some(text.chars().take(200).collect()),
            )
        })
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
        return Err(planner_err("Intent cannot be empty.", None));
    }

    if let Some(path) = gguf_path.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let p = PathBuf::from(path);
        if p.extension().and_then(|e| e.to_str()) == Some("gguf") || p.is_file() {
            let prompt = format!(
                "{PLANNER_SYSTEM}\n\nUser intent:\n{trimmed_intent}\n\nShell command:"
            );
            let raw = local_inference::complete_with_gguf(&p, &prompt, 96)?;
            let command = extract_command_line(&raw);
            if command.is_empty() {
                return Err(into_invoke_err(GnomadError::Llm {
                    message: "Embedded planner returned an empty command.".into(),
                    detail: Some(raw.chars().take(200).collect()),
                }));
            }
            if !looks_like_shell_command(&command) {
                return Err(into_invoke_err(GnomadError::Llm {
                    message: format!(
                        "Embedded planner produced invalid shell syntax: \"{}\". Try a small coding GGUF (e.g. Qwen2.5-Coder 1.5B).",
                        command.chars().take(120).collect::<String>()
                    ),
                    detail: None,
                }));
            }
            return Ok(command);
        }
    }

    let model = model.trim();
    if model.is_empty() {
        return Err(planner_err(
            "Planner model name is not configured.",
            Some("Set a planner model in Settings → Agent access.".into()),
        ));
    }

    let base = ollama_base(ollama_url);
    let raw = ollama_plan(&base, model, trimmed_intent).await?;
    let command = extract_command_line(&raw);

    if command.is_empty() {
        return Err(planner_err("Planner returned an empty command.", None));
    }
    if !looks_like_shell_command(&command) {
        return Err(planner_err(
            format!(
                "Planner produced invalid shell syntax: \"{}\"",
                command.chars().take(120).collect::<String>()
            ),
            None,
        ));
    }

    Ok(command)
}
