use serde::Serialize;
use std::path::PathBuf;

use super::keychain;

/// Load `.env` from optional project roots and the current working directory.
pub fn load_dotenv_files(extra_roots: &[PathBuf]) {
    for root in extra_roots {
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
    if let Ok(url) = keychain::get_credential_value("cloud_api_base_url") {
        let trimmed = url.trim();
        if !trimmed.is_empty() {
            return normalize_cloud_base_url(trimmed);
        }
    }
    DEFAULT_CLOUD_API_BASE.to_string()
}

pub fn resolved_env_path(search_roots: &[PathBuf]) -> Option<String> {
    for root in search_roots {
        let root_env = root.join(".env");
        if root_env.is_file() {
            return Some(root_env.to_string_lossy().to_string());
        }
    }
    std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join(".env"))
        .filter(|p| p.is_file())
        .map(|p| p.to_string_lossy().to_string())
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvLlmConfig {
    pub deepseek_configured: bool,
    pub env_path: Option<String>,
    pub cloud_base_url_from_env: bool,
}

pub fn build_env_llm_config(search_roots: &[PathBuf]) -> EnvLlmConfig {
    EnvLlmConfig {
        deepseek_configured: cloud_api_key_from_env().is_some(),
        env_path: resolved_env_path(search_roots),
        cloud_base_url_from_env: cloud_api_base_url_from_env().is_some(),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudApiConfig {
    pub base_url: String,
    pub is_default_deepseek: bool,
    pub api_key_configured: bool,
    pub base_url_from_env: bool,
}

pub fn build_cloud_api_config() -> CloudApiConfig {
    let base_url = resolved_cloud_api_base_url();
    CloudApiConfig {
        is_default_deepseek: is_default_deepseek_base(&base_url),
        api_key_configured: cloud_api_key_from_env().is_some()
            || keychain::get_credential_value("llm_api_key")
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

pub fn cloud_api_key_source() -> CloudApiKeySource {
    if cloud_api_key_from_env().is_some() {
        return CloudApiKeySource::Env;
    }
    if let Ok(key) = keychain::get_credential_value("llm_api_key") {
        if !key.trim().is_empty() {
            return CloudApiKeySource::Keychain;
        }
    }
    CloudApiKeySource::None
}

pub fn has_llm_configured() -> Result<bool, String> {
    if cloud_api_key_from_env().is_some() {
        return Ok(true);
    }
    let key = keychain::get_credential_value("llm_api_key")?;
    if !key.trim().is_empty() {
        return Ok(true);
    }
    let url = keychain::get_credential_value("ollama_url")?;
    Ok(!url.trim().is_empty())
}

pub fn effective_cloud_api_key() -> Result<String, String> {
    if let Some(key) = cloud_api_key_from_env() {
        return Ok(key);
    }
    keychain::get_credential_value("llm_api_key")
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
