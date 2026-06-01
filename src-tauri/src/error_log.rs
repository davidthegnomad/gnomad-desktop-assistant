use serde::Serialize;
use std::io::Write;
use tauri::Manager;

#[derive(Serialize)]
struct ErrorLogEntry {
    ts: u64,
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
}

fn log_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    Ok(base.join("gnomad").join("error-log.jsonl"))
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn append_error(
    app: &tauri::AppHandle,
    code: &str,
    message: &str,
    detail: Option<&str>,
    source: Option<&str>,
) {
    let entry = ErrorLogEntry {
        ts: now_secs(),
        code: code.to_string(),
        message: message.to_string(),
        detail: detail.map(|s| s.to_string()),
        source: source.map(|s| s.to_string()),
    };
    if let Ok(line) = serde_json::to_string(&entry) {
        if let Ok(path) = log_path(app) {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(f, "{line}");
            }
        }
    }
}

#[tauri::command]
pub fn append_error_log(
    app: tauri::AppHandle,
    code: String,
    message: String,
    detail: Option<String>,
    source: Option<String>,
) -> Result<(), String> {
    append_error(
        &app,
        &code,
        &message,
        detail.as_deref(),
        source.as_deref(),
    );
    Ok(())
}
