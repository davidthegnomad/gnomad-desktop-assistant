use serde::Serialize;
use std::io::Write;
use tauri::Manager;

#[derive(Serialize)]
struct AuditEntry {
    ts: u64,
    kind: String,
    detail: String,
}

fn audit_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    Ok(base.join("gnomad").join("agent-audit.jsonl"))
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn log_action(app: &tauri::AppHandle, kind: &str, detail: &str) {
    let entry = AuditEntry {
        ts: now_secs(),
        kind: kind.to_string(),
        detail: detail.to_string(),
    };
    if let Ok(line) = serde_json::to_string(&entry) {
        if let Ok(path) = audit_path(app) {
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
pub fn append_agent_audit(
    app: tauri::AppHandle,
    kind: String,
    detail: String,
) -> Result<(), String> {
    log_action(&app, &kind, &detail);
    Ok(())
}
