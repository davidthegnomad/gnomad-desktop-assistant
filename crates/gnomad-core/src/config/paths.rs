use std::path::{Path, PathBuf};

/// Application identifier — matches Tauri `tauri.conf.json` for shared data dirs on Linux.
pub const APP_ID: &str = "com.gnomadstudio.gnomad";

/// XDG-backed (or fallback) filesystem roots for Gnomad persistence.
#[derive(Debug, Clone)]
pub struct DataPaths {
    data_dir: PathBuf,
    config_dir: PathBuf,
}

impl DataPaths {
    /// Resolve standard data/config dirs for this app (Linux XDG, macOS Application Support, etc.).
    pub fn discover() -> Self {
        match xdg::BaseDirectories::with_prefix(APP_ID) {
            Ok(xdg) => Self {
                data_dir: xdg.get_data_home(),
                config_dir: xdg.get_config_home(),
            },
            Err(_) => Self::fallback(),
        }
    }

    /// Build from an explicit data root (e.g. Tauri `app_data_dir()` during transition).
    pub fn from_data_root(data_dir: PathBuf) -> Self {
        let config_dir = data_dir
            .parent()
            .and_then(|share| share.parent())
            .map(|home_child| home_child.join("config").join(APP_ID))
            .unwrap_or_else(|| data_dir.join("config"));
        Self {
            data_dir,
            config_dir,
        }
    }

    /// Isolated data + config roots for unit tests and migration tests.
    pub fn for_test(base: PathBuf) -> Self {
        Self {
            data_dir: base.join("data"),
            config_dir: base.join("config"),
        }
    }

    fn fallback() -> Self {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        Self {
            data_dir: home.join(".local/share").join(APP_ID),
            config_dir: home.join(".config").join(APP_ID),
        }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// `…/gnomad` under the app data dir (agent settings, audit, knowledge, sandbox profiles).
    pub fn gnomad_data_dir(&self) -> PathBuf {
        self.data_dir.join("gnomad")
    }

    pub fn ensure_gnomad_data_dir(&self) -> Result<PathBuf, String> {
        let dir = self.gnomad_data_dir();
        std::fs::create_dir_all(&dir).map_err(|e| format!("create data dir: {e}"))?;
        Ok(dir)
    }

    pub fn agent_settings_path(&self) -> PathBuf {
        self.gnomad_data_dir().join("agent-settings.json")
    }

    pub fn agent_audit_path(&self) -> PathBuf {
        self.gnomad_data_dir().join("agent-audit.jsonl")
    }

    pub fn knowledge_root(&self) -> PathBuf {
        self.gnomad_data_dir().join("knowledge")
    }

    /// Chat session store (`store.json`) — same path as the Tauri app for compatibility.
    pub fn chats_dir(&self) -> PathBuf {
        self.gnomad_data_dir().join("chats")
    }

    /// AES-GCM encrypted agent secrets vault (API keys, passwords for shell tools).
    pub fn agent_secrets_path(&self) -> PathBuf {
        self.gnomad_data_dir().join("agent-secrets.enc")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gnomad_subdir_under_data_root() {
        let paths = DataPaths::from_data_root(PathBuf::from("/tmp/com.gnomadstudio.gnomad"));
        assert!(paths
            .agent_settings_path()
            .ends_with("gnomad/agent-settings.json"));
    }

    #[test]
    fn discover_has_nonempty_dirs() {
        let paths = DataPaths::discover();
        assert!(!paths.data_dir().as_os_str().is_empty());
        assert!(!paths.config_dir().as_os_str().is_empty());
    }
}
