//! User knowledge library (skills, agents, preferences, uploads).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::paths::DataPaths;

pub const CATEGORIES: &[&str] = &["skills", "agents", "preferences", "uploads"];

const DEFAULT_BUNDLE_MAX_CHARS: usize = 12_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeFileEntry {
    pub id: String,
    pub name: String,
    pub category: String,
    pub path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillPackInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub skill_count: u32,
}

#[derive(Debug, Deserialize)]
struct SkillPackManifest {
    id: String,
    name: String,
    description: String,
}

pub fn knowledge_root(paths: &DataPaths) -> PathBuf {
    paths.knowledge_root()
}

pub fn ensure_layout(paths: &DataPaths) -> Result<PathBuf, String> {
    let root = knowledge_root(paths);
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    for cat in CATEGORIES {
        fs::create_dir_all(root.join(cat)).map_err(|e| e.to_string())?;
    }
    let prefs = root.join("preferences").join("user-preferences.md");
    if !prefs.exists() {
        fs::write(
            &prefs,
            r#"# Gnomad user preferences

Agents read and update this file to remember how you like to work.

## Notes

- Add preferences in Settings → Knowledge, or let Gnomad learn from your chats.

"#,
        )
        .map_err(|e| e.to_string())?;
    }
    let index = root.join("INDEX.md");
    if !index.exists() {
        fs::write(
            &index,
            r#"# Gnomad knowledge base

| Folder | Purpose |
|--------|---------|
| `skills/` | Reusable skill instructions |
| `agents/` | Agent personas |
| `preferences/` | User preferences |
| `uploads/` | Reference files |

"#,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(root)
}

pub fn list_files(paths: &DataPaths) -> Result<Vec<KnowledgeFileEntry>, String> {
    list_files_filtered(paths, None)
}

pub fn list_files_filtered(
    paths: &DataPaths,
    category: Option<&str>,
) -> Result<Vec<KnowledgeFileEntry>, String> {
    let root = ensure_layout(paths)?;
    let mut out = Vec::new();
    for cat in CATEGORIES {
        if let Some(filter) = category {
            if filter != *cat {
                continue;
            }
        }
        let dir = root.join(cat);
        if !dir.is_dir() {
            continue;
        }
        scan_dir(&dir, cat, &root, &mut out)?;
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn scan_dir(
    dir: &Path,
    category: &str,
    root: &Path,
    out: &mut Vec<KnowledgeFileEntry>,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, category, root, out)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if name == "user-preferences.md" && category == "preferences" {
            // Still listed; agents also read it in the bundle header.
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
        out.push(KnowledgeFileEntry {
            id: rel.clone(),
            name,
            category: category.to_string(),
            path: path.to_string_lossy().to_string(),
            size_bytes: meta.len(),
        });
    }
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn file_extension(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}

pub fn import_files(
    paths: &DataPaths,
    sources: &[PathBuf],
    category: &str,
) -> Result<Vec<KnowledgeFileEntry>, String> {
    let cat = if CATEGORIES.contains(&category) {
        category.to_string()
    } else {
        "uploads".to_string()
    };
    let root = ensure_layout(paths)?;
    let dest_dir = root.join(&cat);
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

    let mut imported = Vec::new();
    for source in sources {
        if !source.is_file() {
            continue;
        }
        let file_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .map(sanitize_filename)
            .unwrap_or_else(|| "upload".into());
        let dest = dest_dir.join(&file_name);
        fs::copy(source, &dest).map_err(|e| e.to_string())?;
        let meta = fs::metadata(&dest).map_err(|e| e.to_string())?;
        let rel = dest
            .strip_prefix(&root)
            .unwrap_or(&dest)
            .to_string_lossy()
            .to_string();
        imported.push(KnowledgeFileEntry {
            id: rel,
            name: file_name,
            category: cat.clone(),
            path: dest.to_string_lossy().to_string(),
            size_bytes: meta.len(),
        });
    }
    Ok(imported)
}

pub fn delete_file(paths: &DataPaths, id: &str) -> Result<(), String> {
    if id.is_empty() || id.contains("..") || id.starts_with('/') {
        return Err("Invalid file id.".into());
    }
    let root = ensure_layout(paths)?;
    let path = root.join(id);
    if !path.starts_with(&root) {
        return Err("Invalid file id.".into());
    }
    if path.is_file() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn trim_context_bundle(bundle: &str, max_chars: usize) -> String {
    let trimmed = bundle.trim();
    if trimmed.len() <= max_chars {
        return trimmed.to_string();
    }
    format!(
        "{}\n\n…(knowledge truncated)",
        &trimmed[..max_chars]
    )
}

pub fn get_agent_context_bundle(paths: &DataPaths) -> Result<String, String> {
    let root = ensure_layout(paths)?;
    let mut out = String::from("# Gnomad agent context bundle\n\n");

    let index = root.join("INDEX.md");
    if index.exists() {
        out.push_str(&fs::read_to_string(&index).map_err(|e| e.to_string())?);
        out.push_str("\n\n---\n\n");
    }

    let prefs = root.join("preferences").join("user-preferences.md");
    if prefs.exists() {
        out.push_str("## User preferences\n\n");
        out.push_str(&fs::read_to_string(&prefs).map_err(|e| e.to_string())?);
        out.push_str("\n\n---\n\n");
    }

    let files = list_files(paths)?;
    for cat in CATEGORIES {
        let cat_files: Vec<_> = files.iter().filter(|f| f.category == *cat).collect();
        if cat_files.is_empty() {
            continue;
        }
        out.push_str(&format!("## Category: {cat}\n\n"));
        for f in cat_files {
            let ext = file_extension(Path::new(&f.path));
            if ext == "md" || ext == "txt" {
                if let Ok(body) = fs::read_to_string(&f.path) {
                    if f.path == prefs.to_string_lossy() {
                        continue;
                    }
                    out.push_str(&format!("### {}\n\n{body}\n\n", f.name));
                }
            } else {
                out.push_str(&format!("- {} ({} bytes)\n", f.name, f.size_bytes));
            }
        }
    }

    Ok(out)
}

/// Merge desktop context pills with trimmed knowledge bundle for chat/agent system prompts.
pub fn build_full_session_context(paths: &DataPaths, desktop_context: &str) -> String {
    let mut parts = Vec::new();
    let desktop = desktop_context.trim();
    if !desktop.is_empty() {
        parts.push(desktop.to_string());
    }
    if let Ok(bundle) = get_agent_context_bundle(paths) {
        let trimmed = trim_context_bundle(&bundle, DEFAULT_BUNDLE_MAX_CHARS);
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }
    let help = crate::help::context_snippet();
    if !help.is_empty() {
        parts.push(format!("## Gnomad quick help\n\n{help}"));
    }
    parts.join("\n\n---\n\n")
}

pub fn open_knowledge_folder(paths: &DataPaths) -> Result<(), String> {
    let root = ensure_layout(paths)?;
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&root)
            .spawn()
            .map_err(|e| format!("open knowledge folder: {e}"))?;
        return Ok(());
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = root;
        Err("open_knowledge_folder is only implemented on Linux".into())
    }
}

pub fn skill_packs_root() -> Option<PathBuf> {
    if let Ok(raw) = std::env::var("GNOMAD_SKILL_PACKS") {
        let p = PathBuf::from(raw);
        if p.is_dir() {
            return Some(p);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let bundled = dir.join("resources").join("skill-packs");
            if bundled.is_dir() {
                return Some(bundled);
            }
        }
    }
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../src-tauri/resources/skill-packs");
    if dev.is_dir() {
        return Some(dev);
    }
    None
}

fn read_pack_manifest(pack_dir: &Path) -> Result<SkillPackManifest, String> {
    let path = pack_dir.join("pack.json");
    let raw = fs::read_to_string(&path).map_err(|e| format!("pack.json: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Invalid pack.json: {e}"))
}

pub fn list_skill_packs() -> Result<Vec<SkillPackInfo>, String> {
    let root = skill_packs_root().ok_or_else(|| "Skill packs not bundled.".to_string())?;
    let mut packs = Vec::new();
    for entry in fs::read_dir(&root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let pack_dir = entry.path();
        let manifest = match read_pack_manifest(&pack_dir) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let skill_count = fs::read_dir(&pack_dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .and_then(|x| x.to_str())
                            .map(|x| x.eq_ignore_ascii_case("md"))
                            .unwrap_or(false)
                    })
                    .count() as u32
            })
            .unwrap_or(0);
        packs.push(SkillPackInfo {
            id: manifest.id,
            name: manifest.name,
            description: manifest.description,
            skill_count,
        });
    }
    packs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(packs)
}

pub fn install_skill_pack(paths: &DataPaths, pack_id: &str) -> Result<Vec<KnowledgeFileEntry>, String> {
    if pack_id.is_empty()
        || pack_id.contains("..")
        || pack_id.contains('/')
        || pack_id.contains('\\')
    {
        return Err("Invalid skill pack id.".into());
    }
    let packs_root = skill_packs_root().ok_or_else(|| "Skill packs not bundled.".to_string())?;
    let pack_dir = packs_root.join(pack_id);
    if !pack_dir.is_dir() {
        return Err(format!("Skill pack not found: {pack_id}"));
    }

    let mut sources = Vec::new();
    for entry in fs::read_dir(&pack_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file()
            && path.extension().and_then(|e| e.to_str()) == Some("md")
        {
            sources.push(path);
        }
    }
    import_files(paths, &sources, "skills")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::DataPaths;

    #[test]
    fn trim_context_bundle_truncates() {
        let long = "a".repeat(20_000);
        let out = trim_context_bundle(&long, 100);
        assert!(out.len() < long.len());
        assert!(out.contains("truncated"));
    }

    #[test]
    fn bundle_includes_imported_skill() {
        let base = std::env::temp_dir().join(format!(
            "gnomad-knowledge-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&base);
        let paths = DataPaths::from_data_root(base.clone());
        let src = std::env::temp_dir().join("gnomad-test-skill.md");
        fs::write(&src, "# Test skill\nDo the thing.").unwrap();
        import_files(&paths, &[src], "skills").unwrap();
        let bundle = get_agent_context_bundle(&paths).unwrap();
        assert!(bundle.contains("Test skill"));
        assert!(bundle.contains("Do the thing."));
        let _ = fs::remove_dir_all(&base);
    }
}
