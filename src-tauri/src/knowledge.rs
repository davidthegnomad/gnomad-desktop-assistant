use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::Manager;

const CATEGORIES: &[&str] = &["skills", "agents", "preferences", "uploads"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeFileEntry {
    pub id: String,
    pub name: String,
    pub category: String,
    pub path: String,
    pub size_bytes: u64,
    pub updated_at: String,
    pub extension: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KnowledgeManifest {
    version: u32,
    files: Vec<KnowledgeFileEntry>,
}

fn knowledge_root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    Ok(base.join("gnomad").join("knowledge"))
}

fn manifest_path(root: &Path) -> PathBuf {
    root.join("manifest.json")
}

fn ensure_layout(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    for cat in CATEGORIES {
        fs::create_dir_all(root.join(cat)).map_err(|e| e.to_string())?;
    }

    let prefs = root.join("preferences").join("user-preferences.md");
    if !prefs.exists() {
        let seed = r#"# Gnomad user preferences

Agents read and update this file to remember how you like to work.

## Notes

- Add preferences by using Settings → Knowledge, or let Gnomad learn from your chats.

"#;
        fs::write(&prefs, seed).map_err(|e| e.to_string())?;
    }

    let index = root.join("INDEX.md");
    if !index.exists() {
        let seed = r#"# Gnomad knowledge base

Upload skills, agent briefs, and reference files so models stay customized.

| Folder | Purpose |
|--------|---------|
| `skills/` | Reusable skill instructions (`.md`, `.txt`) |
| `agents/` | Agent personas and knowledge |
| `preferences/` | Auto-updated user preferences |
| `uploads/` | General reference files |

"#;
        fs::write(&index, seed).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn load_manifest(root: &Path) -> Result<KnowledgeManifest, String> {
    let path = manifest_path(root);
    if !path.exists() {
        return Ok(KnowledgeManifest {
            version: 1,
            files: vec![],
        });
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn save_manifest(root: &Path, manifest: &KnowledgeManifest) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
    fs::write(manifest_path(root), raw).map_err(|e| e.to_string())
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

fn register_file(
    manifest: &mut KnowledgeManifest,
    category: &str,
    dest: &Path,
) -> KnowledgeFileEntry {
    let meta = fs::metadata(dest).ok();
    let name = dest
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();
    let ext = dest
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mut hasher = DefaultHasher::new();
    dest.to_string_lossy().hash(&mut hasher);
    let id = format!("{}-{:x}", category, hasher.finish());
    let entry = KnowledgeFileEntry {
        id: id.clone(),
        name: name.clone(),
        category: category.to_string(),
        path: dest.to_string_lossy().to_string(),
        size_bytes: meta.map(|m| m.len()).unwrap_or(0),
        updated_at: now_iso(),
        extension: ext,
    };
    manifest.files.retain(|f| f.path != entry.path);
    manifest.files.push(entry.clone());
    entry
}

pub fn init_knowledge_store(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let root = knowledge_root(app)?;
    ensure_layout(&root)?;
    Ok(root)
}

#[tauri::command]
pub fn get_knowledge_root(app: tauri::AppHandle) -> Result<String, String> {
    let root = init_knowledge_store(&app)?;
    Ok(root.to_string_lossy().to_string())
}

#[tauri::command]
pub fn list_knowledge_files(
    app: tauri::AppHandle,
    category: Option<String>,
) -> Result<Vec<KnowledgeFileEntry>, String> {
    let root = init_knowledge_store(&app)?;
    let manifest = load_manifest(&root)?;
    Ok(manifest
        .files
        .into_iter()
        .filter(|f| {
            category
                .as_ref()
                .map(|c| &f.category == c)
                .unwrap_or(true)
        })
        .collect())
}

#[tauri::command]
pub fn import_knowledge_files(
    app: tauri::AppHandle,
    source_paths: Vec<String>,
    category: String,
) -> Result<Vec<KnowledgeFileEntry>, String> {
    let cat = if CATEGORIES.contains(&category.as_str()) {
        category
    } else {
        "uploads".to_string()
    };

    let root = init_knowledge_store(&app)?;
    let dest_dir = root.join(&cat);
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

    let mut manifest = load_manifest(&root)?;
    let mut imported = Vec::new();

    for src in source_paths {
        let source = PathBuf::from(&src);
        if !source.exists() {
            continue;
        }
        let file_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .map(sanitize_filename)
            .unwrap_or_else(|| "upload".into());
        let dest = dest_dir.join(&file_name);
        fs::copy(&source, &dest).map_err(|e| e.to_string())?;
        let entry = register_file(&mut manifest, &cat, &dest);
        imported.push(entry);
    }

    save_manifest(&root, &manifest)?;
    touch_index(&root, &format!("Imported {} file(s) to `{cat}/`", imported.len()))?;
    Ok(imported)
}

#[tauri::command]
pub fn read_knowledge_file(app: tauri::AppHandle, id: String) -> Result<String, String> {
    let root = init_knowledge_store(&app)?;
    let manifest = load_manifest(&root)?;
    let entry = manifest
        .files
        .iter()
        .find(|f| f.id == id)
        .ok_or_else(|| format!("Unknown file id: {id}"))?;
    fs::read_to_string(&entry.path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_knowledge_file(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let root = init_knowledge_store(&app)?;
    let mut manifest = load_manifest(&root)?;
    let Some(idx) = manifest.files.iter().position(|f| f.id == id) else {
        return Err(format!("Unknown file id: {id}"));
    };
    let entry = manifest.files.remove(idx);
    let _ = fs::remove_file(&entry.path);
    save_manifest(&root, &manifest)?;
    touch_index(&root, &format!("Removed `{}`", entry.name))?;
    Ok(())
}

#[tauri::command]
pub fn append_user_preference(
    app: tauri::AppHandle,
    note: String,
    source: Option<String>,
) -> Result<(), String> {
    let root = init_knowledge_store(&app)?;
    let prefs = root.join("preferences").join("user-preferences.md");
    let stamp = now_iso();
    let src = source.unwrap_or_else(|| "user".into());
    let block = format!("\n### {stamp} ({src})\n\n{note}\n");
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&prefs)
        .map_err(|e| e.to_string())?;
    file.write_all(block.as_bytes())
        .map_err(|e| e.to_string())?;
    touch_index(&root, &format!("Preference note added ({src})"))?;
    Ok(())
}

#[tauri::command]
pub fn get_agent_context_bundle(app: tauri::AppHandle) -> Result<String, String> {
    let root = init_knowledge_store(&app)?;
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

    let manifest = load_manifest(&root)?;
    for cat in CATEGORIES {
        let cat_files: Vec<_> = manifest
            .files
            .iter()
            .filter(|f| f.category == *cat)
            .collect();
        if cat_files.is_empty() {
            continue;
        }
        out.push_str(&format!("## Category: {cat}\n\n"));
        for f in cat_files {
            if f.extension == "md" || f.extension == "txt" {
                if let Ok(body) = fs::read_to_string(&f.path) {
                    out.push_str(&format!("### {}\n\n{body}\n\n", f.name));
                }
            } else {
                out.push_str(&format!("- {} ({} bytes)\n", f.name, f.size_bytes));
            }
        }
    }

    Ok(out)
}

fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn touch_index(root: &Path, line: &str) -> Result<(), String> {
    let index = root.join("INDEX.md");
    let stamp = now_iso();
    let entry = format!("\n- **{stamp}**: {line}\n");
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&index)
        .map_err(|e| e.to_string())?;
    file.write_all(entry.as_bytes())
        .map_err(|e| e.to_string())?;
    Ok(())
}
