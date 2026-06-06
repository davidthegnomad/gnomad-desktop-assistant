pub use gnomad_core::config::env_config::{
    build_cloud_api_config, build_env_llm_config, cloud_api_key_from_env, cloud_api_key_source,
    effective_cloud_api_key, has_llm_configured, resolved_cloud_api_base_url, CloudApiConfig,
    CloudApiKeySource, EnvLlmConfig,
};

use crate::core_bridge;

pub fn load_dotenv_files() {
    gnomad_core::config::env_config::load_dotenv_files(&core_bridge::dotenv_search_roots());
}

#[tauri::command]
pub fn get_env_llm_config() -> EnvLlmConfig {
    build_env_llm_config(&core_bridge::dotenv_search_roots())
}

#[tauri::command]
pub fn get_cloud_api_config() -> CloudApiConfig {
    build_cloud_api_config()
}

#[tauri::command]
pub fn get_cloud_api_key_source() -> CloudApiKeySource {
    cloud_api_key_source()
}

#[tauri::command(rename = "has_llm_configured")]
pub fn has_llm_configured_cmd() -> Result<bool, String> {
    has_llm_configured()
}

#[tauri::command]
pub fn get_effective_cloud_api_key() -> Result<String, String> {
    effective_cloud_api_key()
}
