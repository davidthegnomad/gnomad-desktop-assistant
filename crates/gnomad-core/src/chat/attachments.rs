//! Chat file attachments — stage copies under app data and inline text for prompts.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::paths::DataPaths;

pub const MAX_TEXT_BYTES: usize = 48_000;
pub const MAX_FILES: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAttachment {
    pub id: String,
    pub name: String,
    pub path: String,
    pub mime: String,
    pub kind: String,
    pub size_bytes: u64,
}

pub fn staging_dir(paths: &DataPaths) -> Result<PathBuf, String> {
    let dir = paths.gnomad_data_dir().join("chat-staging");
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

pub fn stage_chat_attachments(
    paths: &DataPaths,
    source_paths: &[String],
) -> Result<Vec<ChatAttachment>, String> {
    if source_paths.is_empty() {
        return Ok(vec![]);
    }
    if source_paths.len() > MAX_FILES {
        return Err(format!("Attach at most {MAX_FILES} files at once."));
    }

    let staging = staging_dir(paths)?;
    let mut out = Vec::new();

    for (i, src) in source_paths.iter().enumerate() {
        let source = PathBuf::from(src);
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

pub fn format_attachments_for_prompt(paths: &[String]) -> Result<String, String> {
    if paths.is_empty() {
        return Ok(String::new());
    }

    let mut sections = Vec::new();
    sections.push("--- Attached files ---".to_string());

    for path in paths {
        let p = PathBuf::from(path);
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

pub fn remove_staged_attachments(paths: &[String]) {
    for path in paths {
        let p = PathBuf::from(path);
        if p.is_file() {
            let _ = fs::remove_file(p);
        }
    }
}

pub fn format_bytes(n: u64) -> String {
    if n < 1024 {
        format!("{n} B")
    } else if n < 1024 * 1024 {
        format!("{:.1} KB", n as f64 / 1024.0)
    } else {
        format!("{:.1} MB", n as f64 / (1024.0 * 1024.0))
    }
}

pub fn display_text_for_user_message(text: &str, attachments: &[ChatAttachment]) -> String {
    let trimmed = text.trim();
    if attachments.is_empty() {
        return trimmed.to_string();
    }
    let names = attachments
        .iter()
        .map(|a| a.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    if trimmed.is_empty() {
        format!("📎 {names}")
    } else {
        format!("{trimmed}\n\n📎 {names}")
    }
}

pub fn api_content_for_message(text: &str, attachments: Option<&[ChatAttachment]>) -> String {
    let base = text.trim();
    let Some(atts) = attachments.filter(|a| !a.is_empty()) else {
        return base.to_string();
    };
    let paths: Vec<String> = atts.iter().map(|a| a.path.clone()).collect();
    let block = format_attachments_for_prompt(&paths).unwrap_or_default();
    if base.is_empty() {
        block
    } else if block.is_empty() {
        base.to_string()
    } else {
        format!("{base}\n\n{block}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_text_adds_clip_prefix() {
        let atts = vec![ChatAttachment {
            id: "1".into(),
            name: "notes.md".into(),
            path: "/tmp/x".into(),
            mime: "text/plain".into(),
            kind: "text".into(),
            size_bytes: 10,
        }];
        assert_eq!(
            display_text_for_user_message("hello", &atts),
            "hello\n\n📎 notes.md"
        );
    }
}
