use serde::{Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::Manager;

const MAX_TEXT_BYTES: usize = 48_000;
const MAX_FILES: usize = 10;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAttachment {
    pub id: String,
    pub name: String,
    pub path: String,
    pub mime: String,
    pub kind: String,
    pub size_bytes: u64,
}

fn staging_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    let dir = base.join("gnomad").join("chat-staging");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn mime_for_ext(ext: &str) -> (&'static str, &'static str) {
    match ext.to_lowercase().as_str() {
        "png" => ("image/png", "image"),
        "jpg" | "jpeg" => ("image/jpeg", "image"),
        "gif" => ("image/gif", "image"),
        "webp" => ("image/webp", "image"),
        "heic" => ("image/heic", "image"),
        "pdf" => ("application/pdf", "file"),
        "txt" | "md" | "markdown" => ("text/plain", "text"),
        "json" => ("application/json", "text"),
        "csv" => ("text/csv", "text"),
        "xml" => ("application/xml", "text"),
        "html" | "htm" => ("text/html", "text"),
        _ => ("application/octet-stream", "file"),
    }
}

fn is_text_mime(mime: &str) -> bool {
    mime.starts_with("text/") || mime.contains("json") || mime.contains("xml")
}

#[tauri::command]
pub fn stage_chat_attachments(
    app: tauri::AppHandle,
    source_paths: Vec<String>,
) -> Result<Vec<ChatAttachment>, String> {
    if source_paths.is_empty() {
        return Ok(vec![]);
    }
    if source_paths.len() > MAX_FILES {
        return Err(format!("Attach at most {MAX_FILES} files at once."));
    }

    let staging = staging_dir(&app)?;
    let mut out = Vec::new();

    for (i, src) in source_paths.into_iter().enumerate() {
        let source = PathBuf::from(&src);
        if !source.is_file() {
            continue;
        }
        let meta = fs::metadata(&source).map_err(|e| e.to_string())?;
        let name = source
            .file_name()
            .and_then(|n| n.to_str())
            .map(sanitize_filename)
            .unwrap_or_else(|| format!("file_{i}"));
        let ext = Path::new(&name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        let (mime, kind) = mime_for_ext(&ext);
        let id = format!("{i}_{}", meta.len());
        let dest = staging.join(format!("{id}_{name}"));
        fs::copy(&source, &dest).map_err(|e| format!("Failed to copy {name}: {e}"))?;
        out.push(ChatAttachment {
            id,
            name,
            path: dest.to_string_lossy().to_string(),
            mime: mime.to_string(),
            kind: kind.to_string(),
            size_bytes: meta.len(),
        });
    }

    Ok(out)
}

#[tauri::command]
pub fn format_attachments_for_prompt(paths: Vec<String>) -> Result<String, String> {
    if paths.is_empty() {
        return Ok(String::new());
    }

    let mut sections = Vec::new();
    sections.push("--- Attached files ---".to_string());

    for path in paths {
        let p = PathBuf::from(&path);
        if !p.exists() {
            continue;
        }
        let name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("attachment");
        let meta = fs::metadata(&p).map_err(|e| e.to_string())?;
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        let (mime, kind) = mime_for_ext(ext);
        let size_kb = meta.len() as f64 / 1024.0;

        if kind == "image" {
            sections.push(format!(
                "\n### {name} ({mime}, {size_kb:.1} KB)\n[Image attached — describe or analyze based on the user's message; vision may be limited on this model.]"
            ));
            continue;
        }

        if is_text_mime(mime) || kind == "text" {
            let bytes = fs::read(&p).map_err(|e| e.to_string())?;
            let truncated = bytes.len() > MAX_TEXT_BYTES;
            let slice = if truncated {
                &bytes[..MAX_TEXT_BYTES]
            } else {
                &bytes[..]
            };
            let content = String::from_utf8_lossy(slice);
            let tail = if truncated {
                "\n…(file truncated)"
            } else {
                ""
            };
            sections.push(format!(
                "\n### {name} ({mime}, {size_kb:.1} KB)\n```\n{content}{tail}\n```"
            ));
        } else {
            sections.push(format!(
                "\n### {name} ({mime}, {size_kb:.1} KB)\n[Binary file attached — content not inlined.]"
            ));
        }
    }

    sections.push("\n--- End attachments ---".to_string());
    Ok(sections.join("\n"))
}

#[tauri::command]
pub fn remove_staged_attachments(paths: Vec<String>) -> Result<(), String> {
    for path in paths {
        let p = PathBuf::from(&path);
        if p.exists() {
            let _ = fs::remove_file(p);
        }
    }
    Ok(())
}
