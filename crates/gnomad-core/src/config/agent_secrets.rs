//! Encrypted vault for agent subprocess secrets (separate from cloud LLM API keys).

use std::collections::HashMap;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use serde::{Deserialize, Serialize};

use super::keychain;
use super::paths::DataPaths;

const MASTER_KEY_NAME: &str = "agent-secrets-master-key";
const VAULT_VERSION: u32 = 1;
const NONCE_LEN: usize = 12;

/// Reserved vault key for optional stored sudo password (never injected into shell env).
pub const SUDO_PASSWORD_KEY: &str = "SUDO_PASSWORD";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSecretStatus {
    pub name: String,
    pub configured: bool,
}

#[derive(Serialize, Deserialize)]
struct VaultPlain {
    entries: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct VaultFile {
    v: u32,
    nonce: String,
    ciphertext: String,
}

fn ensure_master_key() -> Result<[u8; 32], String> {
    let existing = keychain::get_credential_value(MASTER_KEY_NAME)?;
    if existing.len() == 64 {
        let mut key = [0u8; 32];
        hex::decode_to_slice(&existing, &mut key).map_err(|e| format!("decode master key: {e}"))?;
        return Ok(key);
    }
    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key).map_err(|e| format!("random master key: {e}"))?;
    keychain::store_credential(MASTER_KEY_NAME, &hex::encode(key))?;
    Ok(key)
}

fn validate_secret_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Secret name cannot be empty.".into());
    }
    if trimmed == SUDO_PASSWORD_KEY {
        return Ok(());
    }
    let ok = trimmed
        .chars()
        .enumerate()
        .all(|(i, c)| {
            if i == 0 {
                c.is_ascii_uppercase()
            } else {
                c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'
            }
        });
    if !ok {
        return Err(
            "Secret names must look like env vars (e.g. GITHUB_TOKEN, DATABASE_URL).".into(),
        );
    }
    Ok(())
}

fn encrypt_entries(key: &[u8; 32], entries: &HashMap<String, String>) -> Result<String, String> {
    let plain = VaultPlain {
        entries: entries.clone(),
    };
    let plain_json = serde_json::to_vec(&plain).map_err(|e| e.to_string())?;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    getrandom::getrandom(&mut nonce_bytes).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plain_json.as_ref())
        .map_err(|e| format!("encrypt vault: {e}"))?;
    let file = VaultFile {
        v: VAULT_VERSION,
        nonce: base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            nonce_bytes,
        ),
        ciphertext: base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            ciphertext,
        ),
    };
    serde_json::to_string_pretty(&file).map_err(|e| e.to_string())
}

fn decrypt_entries(key: &[u8; 32], text: &str) -> Result<HashMap<String, String>, String> {
    let file: VaultFile = serde_json::from_str(text).map_err(|e| format!("parse vault: {e}"))?;
    if file.v != VAULT_VERSION {
        return Err("Unsupported agent secrets vault version.".into());
    }
    let nonce_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        file.nonce,
    )
    .map_err(|e| format!("decode nonce: {e}"))?;
    if nonce_bytes.len() != NONCE_LEN {
        return Err("Invalid vault nonce length.".into());
    }
    let ciphertext = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        file.ciphertext,
    )
    .map_err(|e| format!("decode ciphertext: {e}"))?;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let plain_bytes = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("decrypt vault: {e}"))?;
    let plain: VaultPlain =
        serde_json::from_slice(&plain_bytes).map_err(|e| format!("parse vault payload: {e}"))?;
    Ok(plain.entries)
}

pub fn load_entries(paths: &DataPaths) -> Result<HashMap<String, String>, String> {
    let path = paths.agent_secrets_path();
    if !path.is_file() {
        return Ok(HashMap::new());
    }
    let text = std::fs::read_to_string(&path).map_err(|e| format!("read vault: {e}"))?;
    if text.trim().is_empty() {
        return Ok(HashMap::new());
    }
    let key = ensure_master_key()?;
    decrypt_entries(&key, &text)
}

fn save_entries_map(paths: &DataPaths, entries: &HashMap<String, String>) -> Result<(), String> {
    paths.ensure_gnomad_data_dir()?;
    let path = paths.agent_secrets_path();
    if entries.is_empty() {
        if path.is_file() {
            std::fs::remove_file(&path).map_err(|e| format!("remove vault: {e}"))?;
        }
        return Ok(());
    }
    let key = ensure_master_key()?;
    let text = encrypt_entries(&key, entries)?;
    std::fs::write(&path, text).map_err(|e| format!("write vault: {e}"))?;
    Ok(())
}

pub fn set_entry(paths: &DataPaths, name: &str, value: &str) -> Result<(), String> {
    validate_secret_name(name)?;
    let trimmed_value = value.trim();
    if trimmed_value.is_empty() {
        return Err("Secret value cannot be empty.".into());
    }
    let mut entries = load_entries(paths)?;
    entries.insert(name.trim().to_string(), trimmed_value.to_string());
    save_entries_map(paths, &entries)
}

pub fn remove_entry(paths: &DataPaths, name: &str) -> Result<(), String> {
    let mut entries = load_entries(paths)?;
    entries.remove(name.trim());
    save_entries_map(paths, &entries)
}

pub fn has_entry(paths: &DataPaths, name: &str) -> bool {
    load_entries(paths)
        .ok()
        .and_then(|e| e.get(name.trim()).cloned())
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

pub fn list_statuses(paths: &DataPaths) -> Result<Vec<AgentSecretStatus>, String> {
    let entries = load_entries(paths)?;
    let mut names: Vec<String> = entries
        .keys()
        .filter(|k| *k != SUDO_PASSWORD_KEY)
        .cloned()
        .collect();
    names.sort();
    Ok(names
        .into_iter()
        .map(|name| AgentSecretStatus {
            configured: entries.get(&name).map(|v| !v.is_empty()).unwrap_or(false),
            name,
        })
        .collect())
}

/// Environment variables injected into agent shell subprocesses (excludes sudo password).
pub fn shell_env_map(paths: &DataPaths, enabled: bool) -> HashMap<String, String> {
    if !enabled {
        return HashMap::new();
    }
    load_entries(paths)
        .unwrap_or_default()
        .into_iter()
        .filter(|(k, v)| k != SUDO_PASSWORD_KEY && !v.is_empty())
        .collect()
}

pub fn sudo_password(paths: &DataPaths) -> Option<String> {
    load_entries(paths)
        .ok()
        .and_then(|e| e.get(SUDO_PASSWORD_KEY).cloned())
        .filter(|v| !v.is_empty())
}

pub fn sudo_password_configured(paths: &DataPaths) -> bool {
    has_entry(paths, SUDO_PASSWORD_KEY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn lock_test() -> MutexGuard<'static, ()> {
        TEST_LOCK.lock().unwrap()
    }

    fn temp_paths() -> DataPaths {
        let dir = std::env::temp_dir().join(format!(
            "gnomad-secrets-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        DataPaths::from_data_root(dir)
    }

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let _guard = lock_test();
        let paths = temp_paths();
        set_entry(&paths, "GITHUB_TOKEN", "ghp_test").expect("set");
        set_entry(&paths, SUDO_PASSWORD_KEY, "secret").expect("sudo");
        let entries = load_entries(&paths).expect("load");
        assert_eq!(entries.get("GITHUB_TOKEN").map(String::as_str), Some("ghp_test"));
        let env = shell_env_map(&paths, true);
        assert!(env.contains_key("GITHUB_TOKEN"));
        assert!(!env.contains_key(SUDO_PASSWORD_KEY));
        remove_entry(&paths, "GITHUB_TOKEN").expect("remove");
        assert!(!has_entry(&paths, "GITHUB_TOKEN"));
    }

    #[test]
    fn rejects_bad_secret_names() {
        let _guard = lock_test();
        let paths = temp_paths();
        assert!(set_entry(&paths, "bad-name", "x").is_err());
    }
}
