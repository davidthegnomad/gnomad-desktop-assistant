//! Tauri → GTK data migration and compatibility audit.
//!
//! Chat, knowledge, agent settings, and keychain already share `com.gnomadstudio.gnomad`
//! XDG paths with the Tauri app. This module audits existing data, copies legacy trees
//! when found, and can import UI prefs from a localStorage JSON export.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::config::paths::{DataPaths, APP_ID};
use crate::config::ui_prefs::{load_ui_prefs, save_ui_prefs, ThemeMode};

const LEGACY_APP_IDS: &[&str] = &[
    "com.gnomadstudio.omni",
    "com.gnomadstudio.omni-taskbar-ai",
    "omni-taskbar-ai",
];

#[derive(Debug, Clone, Default)]
pub struct MigrationReport {
    pub actions: Vec<String>,
    pub warnings: Vec<String>,
}

impl MigrationReport {
    pub fn format_text(&self) -> String {
        let mut out = String::from("Gnomad data migration\n\n");
        if self.actions.is_empty() && self.warnings.is_empty() {
            out.push_str("Nothing to migrate — GTK uses the same XDG data dir as Tauri.\n");
            return out;
        }
        if !self.actions.is_empty() {
            out.push_str("Actions:\n");
            for a in &self.actions {
                out.push_str(&format!("  • {a}\n"));
            }
        }
        if !self.warnings.is_empty() {
            out.push_str("\nWarnings:\n");
            for w in &self.warnings {
                out.push_str(&format!("  • {w}\n"));
            }
        }
        out
    }
}

#[derive(Debug, Deserialize)]
struct LocalStorageExport {
    #[serde(default)]
    omni_onboarding_complete: Option<String>,
    #[serde(default)]
    omni_provider: Option<String>,
    #[serde(default)]
    omni_model: Option<String>,
    #[serde(default)]
    omni_local_model: Option<String>,
    #[serde(default)]
    omni_theme: Option<String>,
}

pub fn audit_data(paths: &DataPaths) -> MigrationReport {
    let mut report = MigrationReport::default();
    let gnomad = paths.gnomad_data_dir();

    let checks: [(&str, PathBuf); 5] = [
        ("chat store", paths.chats_dir().join("store.json")),
        ("agent settings", paths.agent_settings_path()),
        ("agent secrets", paths.agent_secrets_path()),
        ("knowledge library", paths.knowledge_root()),
        ("UI prefs", paths.config_dir().join("ui-prefs.json")),
    ];

    for (label, path) in checks {
        if path.exists() {
            report
                .actions
                .push(format!("Found existing {label}: {}", path.display()));
        }
    }

    if !gnomad.exists() {
        report.warnings.push(format!(
            "No data at {} — fresh install or run the Tauri app once to seed data.",
            gnomad.display()
        ));
    }

    for legacy_id in LEGACY_APP_IDS {
        if let Some(legacy_root) = legacy_share_dir(legacy_id) {
            let legacy_gnomad = legacy_root.join("gnomad");
            if legacy_gnomad.is_dir() {
                report.actions.push(format!(
                    "Legacy data tree at {} (use --migrate to copy into GTK paths)",
                    legacy_gnomad.display()
                ));
            }
        }
    }

    report
}

pub fn run_migration(
    paths: &DataPaths,
    import_prefs: Option<&Path>,
) -> Result<MigrationReport, String> {
    let mut report = audit_data(paths);
    paths.ensure_gnomad_data_dir()?;
    fs::create_dir_all(paths.chats_dir()).map_err(|e| e.to_string())?;
    fs::create_dir_all(paths.knowledge_root()).map_err(|e| e.to_string())?;

    let target = paths.gnomad_data_dir();
    for legacy_id in LEGACY_APP_IDS {
        let Some(legacy_root) = legacy_share_dir(legacy_id) else {
            continue;
        };
        let legacy_gnomad = legacy_root.join("gnomad");
        if !legacy_gnomad.is_dir() {
            continue;
        }
        if legacy_root == paths.data_dir() {
            continue;
        }
        merge_tree(&legacy_gnomad, &target, &mut report)?;
    }

    if let Some(export_path) = import_prefs {
        import_ui_prefs_export(paths, export_path, &mut report)?;
    }

    if report.actions.is_empty() {
        report.actions.push(format!(
            "Data directory ready at {} (shared with Tauri when identifier is {APP_ID})",
            target.display()
        ));
    }

    Ok(report)
}

fn legacy_share_dir(app_id: &str) -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let share = home.join(".local/share").join(app_id);
    if share.is_dir() {
        Some(share)
    } else {
        None
    }
}

fn merge_tree(src: &Path, dest: &Path, report: &mut MigrationReport) -> Result<(), String> {
    if !src.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let from = entry.path();
        let to = dest.join(&name);
        if from.is_dir() {
            fs::create_dir_all(&to).map_err(|e| e.to_string())?;
            merge_tree(&from, &to, report)?;
        } else if !to.exists() {
            fs::copy(&from, &to).map_err(|e| e.to_string())?;
            report.actions.push(format!("Copied {}", to.display()));
        }
    }
    Ok(())
}

fn import_ui_prefs_export(
    paths: &DataPaths,
    export_path: &Path,
    report: &mut MigrationReport,
) -> Result<(), String> {
    let raw = fs::read_to_string(export_path).map_err(|e| format!("read prefs export: {e}"))?;
    let export: LocalStorageExport =
        serde_json::from_str(&raw).map_err(|e| format!("parse prefs export: {e}"))?;

    let mut prefs = load_ui_prefs(paths);
    let mut changed = false;

    if export.omni_onboarding_complete.as_deref() == Some("true") && !prefs.onboarding_complete {
        prefs.onboarding_complete = true;
        changed = true;
    }
    if let Some(provider) = export.omni_provider.as_ref() {
        let mapped = if provider == "local" {
            "local"
        } else {
            "cloud"
        };
        if prefs.provider.as_deref() != Some(mapped) {
            prefs.provider = Some(mapped.to_string());
            changed = true;
        }
    }
    if let Some(model) = export.omni_model.as_ref().filter(|m| !m.is_empty()) {
        if prefs.model.as_deref() != Some(model.as_str()) {
            prefs.model = Some(model.clone());
            changed = true;
        }
    }
    if let Some(local) = export.omni_local_model.as_ref().filter(|m| !m.is_empty()) {
        if prefs.provider.as_deref() == Some("local") && prefs.model.as_deref() != Some(local.as_str())
        {
            prefs.model = Some(local.clone());
            changed = true;
        }
    }
    if let Some(theme) = export.omni_theme.as_ref().filter(|t| !t.is_empty()) {
        let mapped = ThemeMode::from_storage(theme);
        if prefs.theme != mapped {
            prefs.theme = mapped;
            changed = true;
        }
    }

    if changed {
        save_ui_prefs(paths, &prefs)?;
        report.actions.push(format!(
            "Imported UI prefs from {} → {}",
            export_path.display(),
            paths.config_dir().join("ui-prefs.json").display()
        ));
    } else {
        report.warnings.push(format!(
            "No UI pref changes from {}",
            export_path.display()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_paths() -> DataPaths {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let base = std::env::temp_dir().join(format!("gnomad-migrate-test-{pid}-{n}"));
        let _ = std::fs::remove_dir_all(&base);
        DataPaths::for_test(base)
    }

    #[test]
    fn imports_theme_from_localstorage() {
        let paths = test_paths();
        let export = paths.config_dir().join("ls-export.json");
        fs::create_dir_all(paths.config_dir()).unwrap();
        fs::write(&export, r#"{"omni_theme":"dark"}"#).unwrap();
        let report = run_migration(&paths, Some(&export)).unwrap();
        assert!(report.actions.iter().any(|a| a.contains("Imported UI prefs")));
        assert_eq!(load_ui_prefs(&paths).theme, ThemeMode::Dark);
    }

    #[test]
    fn imports_localstorage_export() {
        let paths = test_paths();
        let export = paths.config_dir().join("ls-export.json");
        fs::create_dir_all(paths.config_dir()).unwrap();
        fs::write(
            &export,
            r#"{"omni_onboarding_complete":"true","omni_provider":"local","omni_local_model":"llama3.2"}"#,
        )
        .unwrap();

        let report = run_migration(&paths, Some(&export)).unwrap();
        assert!(report
            .actions
            .iter()
            .any(|a| a.contains("Imported UI prefs")));
        let prefs = load_ui_prefs(&paths);
        assert!(prefs.onboarding_complete);
        assert_eq!(prefs.provider.as_deref(), Some("local"));
        assert_eq!(prefs.model.as_deref(), Some("llama3.2"));
    }
}
