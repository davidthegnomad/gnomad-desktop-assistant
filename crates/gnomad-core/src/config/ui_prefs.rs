//! GTK shell UI preferences (onboarding, default provider/model).

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::env_config;
use super::paths::DataPaths;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn from_storage(value: &str) -> Self {
        match value {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::System,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UiPrefs {
    #[serde(default)]
    pub onboarding_complete: bool,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub theme: ThemeMode,
}

fn prefs_path(paths: &DataPaths) -> PathBuf {
    paths.config_dir().join("ui-prefs.json")
}

pub fn load_ui_prefs(paths: &DataPaths) -> UiPrefs {
    let path = prefs_path(paths);
    let Ok(raw) = fs::read_to_string(&path) else {
        return UiPrefs::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save_ui_prefs(paths: &DataPaths, prefs: &UiPrefs) -> Result<(), String> {
    fs::create_dir_all(paths.config_dir()).map_err(|e| e.to_string())?;
    let raw = serde_json::to_string_pretty(prefs).map_err(|e| e.to_string())?;
    fs::write(prefs_path(paths), raw).map_err(|e| e.to_string())
}

/// Whether to show first-run onboarding (auto-skips if LLM already configured).
pub fn should_show_onboarding(paths: &DataPaths) -> bool {
    let mut prefs = load_ui_prefs(paths);
    if prefs.onboarding_complete {
        return false;
    }
    if env_config::has_llm_configured().unwrap_or(false) {
        prefs.onboarding_complete = true;
        let _ = save_ui_prefs(paths, &prefs);
        return false;
    }
    true
}

pub fn mark_onboarding_complete(paths: &DataPaths, provider: &str, model: &str) -> Result<(), String> {
    let mut prefs = load_ui_prefs(paths);
    prefs.onboarding_complete = true;
    prefs.provider = Some(provider.to_string());
    prefs.model = Some(model.to_string());
    save_ui_prefs(paths, &prefs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_paths() -> DataPaths {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let base = std::env::temp_dir().join(format!("gnomad-ui-prefs-test-{pid}-{n}"));
        let _ = std::fs::remove_dir_all(&base);
        DataPaths::for_test(base)
    }

    #[test]
    fn round_trip_onboarding_flag() {
        let paths = test_paths();
        assert!(!load_ui_prefs(&paths).onboarding_complete);
        mark_onboarding_complete(&paths, "local", "llama3.2").unwrap();
        let prefs = load_ui_prefs(&paths);
        assert!(prefs.onboarding_complete);
        assert_eq!(prefs.provider.as_deref(), Some("local"));
        assert_eq!(prefs.model.as_deref(), Some("llama3.2"));
    }
}
