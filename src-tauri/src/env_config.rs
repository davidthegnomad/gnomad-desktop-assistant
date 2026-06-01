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

pub fn deepseek_api_key_from_env() -> Option<String> {
    env_value(&["DeepSeek_API_KEY", "DEEPSEEK_API_KEY", "deepseek_api_key"])
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
    /// True when a DeepSeek key is present in `.env` (value is never sent to the UI).
    pub deepseek_configured: bool,
    pub env_path: Option<String>,
}

#[tauri::command]
pub fn get_env_llm_config() -> EnvLlmConfig {
    EnvLlmConfig {
        deepseek_configured: deepseek_api_key_from_env().is_some(),
        env_path: resolved_env_path(),
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
    if deepseek_api_key_from_env().is_some() {
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
    if deepseek_api_key_from_env().is_some() {
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
    if let Some(key) = deepseek_api_key_from_env() {
        return Ok(key);
    }
    crate::keychain::get_credential_value("llm_api_key")
}
