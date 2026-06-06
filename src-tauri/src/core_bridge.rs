//! Adapters between Tauri shell APIs and `gnomad-core` types.

use gnomad_core::config::paths::DataPaths;
use tauri::Manager;

pub fn data_paths_from_app(app: &tauri::AppHandle) -> Result<DataPaths, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    Ok(DataPaths::from_data_root(base))
}

pub fn dotenv_search_roots() -> Vec<std::path::PathBuf> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(|p| vec![p.to_path_buf()])
        .unwrap_or_default()
}
