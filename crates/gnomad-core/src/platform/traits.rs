use std::path::PathBuf;

use crate::config::paths::DataPaths;
use crate::events::EventBus;

/// Platform services the GTK (or Tauri bridge) shell must provide to core subsystems.
pub trait PlatformContext: Send + Sync {
    fn data_paths(&self) -> &DataPaths;
    fn events(&self) -> &EventBus;

    /// Open a URL in the user's default browser.
    fn open_url(&self, url: &str) -> Result<(), String>;

    /// Resolve a bundled resource path (skill packs, icons). None if unavailable.
    fn resource_path(&self, relative: &str) -> Option<PathBuf>;
}

/// Default in-process context for tests and the upcoming GTK shell.
pub struct GnomadCoreContext {
    pub paths: DataPaths,
    pub events: EventBus,
    pub resource_dir: Option<PathBuf>,
}

impl GnomadCoreContext {
    pub fn discover() -> Self {
        Self {
            paths: DataPaths::discover(),
            events: EventBus::default(),
            resource_dir: None,
        }
    }
}

impl PlatformContext for GnomadCoreContext {
    fn data_paths(&self) -> &DataPaths {
        &self.paths
    }

    fn events(&self) -> &EventBus {
        &self.events
    }

    fn open_url(&self, url: &str) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(url)
                .spawn()
                .map_err(|e| format!("open url: {e}"))?;
            return Ok(());
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(url)
                .spawn()
                .map_err(|e| format!("open url: {e}"))?;
            return Ok(());
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", url])
                .spawn()
                .map_err(|e| format!("open url: {e}"))?;
            return Ok(());
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            let _ = url;
            Err("open_url not implemented on this platform".into())
        }
    }

    fn resource_path(&self, relative: &str) -> Option<PathBuf> {
        self.resource_dir.as_ref().map(|base| base.join(relative))
    }
}
