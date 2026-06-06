use serde::Serialize;

use crate::config::keychain;
use crate::error::{into_invoke_err, GnomadError};

fn llm_err(message: impl Into<String>, detail: Option<String>) -> String {
    into_invoke_err(GnomadError::Llm {
        message: message.into(),
        detail,
    })
}

pub fn ollama_base_url(url: Option<String>) -> String {
    if let Some(u) = url {
        let trimmed = u.trim();
        if !trimmed.is_empty() {
            return trimmed.trim_end_matches('/').to_string();
        }
    }
    keychain::get_credential_value("ollama_url")
        .ok()
        .filter(|u| !u.trim().is_empty())
        .map(|u| u.trim().trim_end_matches('/').to_string())
        .unwrap_or_else(|| "http://localhost:11434".to_string())
}

pub fn is_likely_chat_model(name: &str) -> bool {
    let lower = name.to_lowercase();
    !(lower.contains("embed")
        || lower.contains("bge-")
        || lower.contains("mxbai-embed")
        || lower.ends_with("-embed"))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModelOption {
    pub value: String,
    pub label: String,
}

pub async fn list_ollama_models(ollama_url: Option<String>) -> Result<Vec<OllamaModelOption>, String> {
    let base = ollama_base_url(ollama_url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| llm_err("Failed to create HTTP client.", Some(e.to_string())))?;

    let url = format!("{base}/api/tags");
    let response = client.get(&url).send().await.map_err(|e| {
        llm_err(
            "Could not reach Ollama. Is `ollama serve` running?",
            Some(format!("{url}: {e}")),
        )
    })?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| llm_err("Failed to read Ollama model list.", Some(e.to_string())))?;

    if !status.is_success() {
        return Err(llm_err(
            format!("Ollama model list error ({status})."),
            Some(text.chars().take(300).collect()),
        ));
    }

    let parsed: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        llm_err(
            "Invalid Ollama model list JSON.",
            Some(format!(
                "{e}; body: {}",
                text.chars().take(200).collect::<String>()
            )),
        )
    })?;

    let mut models: Vec<OllamaModelOption> = parsed["models"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let name = item["name"]
                        .as_str()
                        .or_else(|| item["model"].as_str())?
                        .trim()
                        .to_string();
                    if name.is_empty() || !is_likely_chat_model(&name) {
                        return None;
                    }
                    Some(OllamaModelOption {
                        label: name.clone(),
                        value: name,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    models.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));
    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_embed_models() {
        assert!(!is_likely_chat_model("nomic-embed-text"));
        assert!(!is_likely_chat_model("bge-large-en"));
        assert!(is_likely_chat_model("qwen2.5:7b"));
    }

    #[test]
    fn normalizes_explicit_ollama_url() {
        assert_eq!(
            ollama_base_url(Some("http://127.0.0.1:11434/".into())),
            "http://127.0.0.1:11434"
        );
    }
}
