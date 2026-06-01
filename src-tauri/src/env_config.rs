use serde::Serialize;
use std::path::PathBuf;

/// Load `.env` from the project root (parent of `src-tauri`) and optionally cwd.
pub fn load_dotenv_files() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(root) = manifest_dir.parent() {
        let root_env = root.join(".env");
        if root_env.is_file() {
            let _ = dotenvy::from_path(&root_env);
        }
    }
    let _ = dotenvy::dotenv();
}

fn env_value(keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Ok(value) = std::env::var(key) {
            let trimmed = value.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

pub const DEFAULT_CLOUD_API_BASE: &str = "https://api.deepseek.com";

pub fn cloud_api_key_from_env() -> Option<String> {
    env_value(&[
        "DeepSeek_API_KEY",
        "DEEPSEEK_API_KEY",
        "deepseek_api_key",
        "OPENAI_API_KEY",
        "CLOUD_API_KEY",
    ])
}

pub fn deepseek_api_key_from_env() -> Option<String> {
    cloud_api_key_from_env()
}

pub fn cloud_api_base_url_from_env() -> Option<String> {
    env_value(&[
        "CLOUD_API_BASE_URL",
        "OPENAI_BASE_URL",
        "OPENAI_API_BASE_URL",
    ])
}

pub fn normalize_cloud_base_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

pub fn is_default_deepseek_base(url: &str) -> bool {
    normalize_cloud_base_url(url) == DEFAULT_CLOUD_API_BASE
}

pub fn resolved_cloud_api_base_url() -> String {
    if let Some(url) = cloud_api_base_url_from_env() {
        return normalize_cloud_base_url(&url);
    }
    if let Ok(url) = crate::keychain::get_credential_value("cloud_api_base_url") {
        let trimmed = url.trim();
        if !trimmed.is_empty() {
            return normalize_cloud_base_url(trimmed);
        }
    }
    DEFAULT_CLOUD_API_BASE.to_string()
}

pub fn resolved_env_path() -> Option<String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root_env = manifest_dir.parent()?.join(".env");
    if root_env.is_file() {
        return Some(root_env.to_string_lossy().to_string());
    }
    std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join(".env"))
        .filter(|p| p.is_file())
        .map(|p| p.to_string_lossy().to_string())
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvLlmConfig {
    /// True when a cloud API key is present in `.env` (value is never sent to the UI).
    pub deepseek_configured: bool,
    pub env_path: Option<String>,
    pub cloud_base_url_from_env: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudApiConfig {
    pub base_url: String,
    pub is_default_deepseek: bool,
    pub api_key_configured: bool,
    pub base_url_from_env: bool,
}

#[tauri::command]
pub fn get_env_llm_config() -> EnvLlmConfig {
    EnvLlmConfig {
        deepseek_configured: cloud_api_key_from_env().is_some(),
        env_path: resolved_env_path(),
        cloud_base_url_from_env: cloud_api_base_url_from_env().is_some(),
    }
}

#[tauri::command]
pub fn get_cloud_api_config() -> CloudApiConfig {
    let base_url = resolved_cloud_api_base_url();
    CloudApiConfig {
        is_default_deepseek: is_default_deepseek_base(&base_url),
        api_key_configured: cloud_api_key_from_env().is_some()
            || crate::keychain::get_credential_value("llm_api_key")
                .map(|k| !k.trim().is_empty())
                .unwrap_or(false),
        base_url_from_env: cloud_api_base_url_from_env().is_some(),
        base_url,
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CloudApiKeySource {
    Env,
    Keychain,
    None,
}

#[tauri::command]
pub fn get_cloud_api_key_source() -> CloudApiKeySource {
    if cloud_api_key_from_env().is_some() {
        return CloudApiKeySource::Env;
    }
    if let Ok(key) = crate::keychain::get_credential_value("llm_api_key") {
        if !key.trim().is_empty() {
            return CloudApiKeySource::Keychain;
        }
    }
    CloudApiKeySource::None
}

#[tauri::command]
pub fn has_llm_configured() -> Result<bool, String> {
    if cloud_api_key_from_env().is_some() {
        return Ok(true);
    }

    let key = crate::keychain::get_credential_value("llm_api_key")?;
    if !key.trim().is_empty() {
        return Ok(true);
    }

    let url = crate::keychain::get_credential_value("ollama_url")?;
    Ok(!url.trim().is_empty())
}

/// Cloud API key for requests: `.env` DeepSeek key takes precedence while testing.
#[tauri::command]
pub fn get_effective_cloud_api_key() -> Result<String, String> {
    if let Some(key) = cloud_api_key_from_env() {
        return Ok(key);
    }
    crate::keychain::get_credential_value("llm_api_key")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_base_url() {
        assert_eq!(
            normalize_cloud_base_url("https://api.openai.com/v1/"),
            "https://api.openai.com/v1"
        );
    }

    #[test]
    fn detects_default_deepseek_base() {
        assert!(is_default_deepseek_base("https://api.deepseek.com"));
        assert!(is_default_deepseek_base("https://api.deepseek.com/"));
        assert!(!is_default_deepseek_base("https://api.openai.com/v1"));
    }
}
