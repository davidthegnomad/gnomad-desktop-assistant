use serde::Serialize;
use std::io::Write;

use crate::config::paths::DataPaths;

#[derive(Serialize)]
struct AuditEntry {
    ts: u64,
    kind: String,
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sandboxed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    success: Option<bool>,
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn write_entry(paths: &DataPaths, entry: &AuditEntry) {
    if let Ok(line) = serde_json::to_string(entry) {
        let path = paths.agent_audit_path();
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

pub fn log_action(paths: &DataPaths, kind: &str, detail: &str) {
    let entry = AuditEntry {
        ts: now_secs(),
        kind: kind.to_string(),
        detail: detail.to_string(),
        sandboxed: None,
        success: None,
    };
    write_entry(paths, &entry);
}

pub fn log_shell_run(paths: &DataPaths, command: &str, sandboxed: bool, success: bool) {
    let preview: String = command.chars().take(240).collect();
    let entry = AuditEntry {
        ts: now_secs(),
        kind: "shell_run".to_string(),
        detail: preview,
        sandboxed: Some(sandboxed),
        success: Some(success),
    };
    write_entry(paths, &entry);
}
