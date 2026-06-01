use crate::agent_settings::{resolve_path_input, TrustMode};
use crate::error::{into_invoke_err, GnomadError};
#[cfg(not(test))]
use crate::keychain::get_credential_value;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

const KEYCHAIN_PATH_SECRET: &str = "path-gate-hmac-secret";
const TOKEN_VERSION: &str = "pv1";
const TOKEN_TTL_SECS: u64 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathScope {
    Read,
    Write,
}

impl PathScope {
    pub fn as_str(self) -> &'static str {
        match self {
            PathScope::Read => "read",
            PathScope::Write => "write",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "read" | "fs_read" | "fs_list" | "fs_search" => Some(PathScope::Read),
            "write" | "fs_write" => Some(PathScope::Write),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenPayload {
    v: String,
    path_hash: String,
    nonce: String,
    issued_at: u64,
    expires_at: u64,
    scope: String,
}

pub struct PathTokenState {
    used_nonces: Mutex<HashSet<String>>,
}

impl Default for PathTokenState {
    fn default() -> Self {
        Self {
            used_nonces: Mutex::new(HashSet::new()),
        }
    }
}

impl PathTokenState {
    fn mark_nonce_used(&self, nonce: &str) -> Result<(), String> {
        let mut guard = self
            .used_nonces
            .lock()
            .map_err(|e| internal_token_err(&e.to_string()))?;
        if guard.contains(nonce) {
            return Err(into_invoke_err(GnomadError::PathPolicy {
                message: "Path approval token was already used.".into(),
                detail: None,
                hint: Some("Approve the path again in Path Gate.".into()),
            }));
        }
        guard.insert(nonce.to_string());
        Ok(())
    }
}

fn internal_token_err(detail: &str) -> String {
    into_invoke_err(GnomadError::Internal {
        message: "Path token store error.".into(),
        detail: Some(detail.to_string()),
    })
}

fn invalid_token_err(message: &str) -> String {
    into_invoke_err(GnomadError::PathPolicy {
        message: message.into(),
        detail: None,
        hint: Some("Approve the path again in Path Gate.".into()),
    })
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn path_hash(canonical: &Path) -> String {
    let digest = Sha256::digest(canonical.to_string_lossy().as_bytes());
    hex::encode(digest)
}

pub fn path_needs_approval(input: &str, workspace: &Path, trust: TrustMode) -> bool {
    if trust == TrustMode::Yolo {
        return false;
    }
    let canonical = resolve_path_input(input, workspace);
    let workspace_canon = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    !canonical.starts_with(&workspace_canon)
}

#[cfg(test)]
fn load_or_create_secret() -> Result<Vec<u8>, String> {
    Ok(b"gnomad-path-gate-test-secret-key!".to_vec())
}

#[cfg(not(test))]
fn load_or_create_secret() -> Result<Vec<u8>, String> {
    let existing = get_credential_value(KEYCHAIN_PATH_SECRET)?;
    if !existing.trim().is_empty() {
        return Ok(existing.into_bytes());
    }
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).map_err(|e| {
        into_invoke_err(GnomadError::Internal {
            message: "Failed to generate path gate signing secret.".into(),
            detail: Some(e.to_string()),
        })
    })?;
    let encoded = hex::encode(bytes);
    if let Err(e) = crate::keychain::store_credential(KEYCHAIN_PATH_SECRET, &encoded) {
        let retry = get_credential_value(KEYCHAIN_PATH_SECRET)?;
        if retry.trim().is_empty() {
            return Err(e);
        }
        return Ok(retry.into_bytes());
    }
    Ok(encoded.into_bytes())
}

fn sign_payload(secret: &[u8], payload_json: &str) -> Result<String, String> {
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|e| internal_token_err(&e.to_string()))?;
    mac.update(payload_json.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

fn verify_signature(secret: &[u8], payload_json: &str, sig_hex: &str) -> Result<(), String> {
    let sig_bytes = hex::decode(sig_hex).map_err(|_| invalid_token_err("Invalid path token signature."))?;
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|e| internal_token_err(&e.to_string()))?;
    mac.update(payload_json.as_bytes());
    mac.verify_slice(&sig_bytes)
        .map_err(|_| invalid_token_err("Path approval token signature mismatch."))
}

fn random_nonce() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).map_err(|e| internal_token_err(&e.to_string()))?;
    Ok(hex::encode(bytes))
}

pub fn issue_path_approval_token(
    input_path: &str,
    scope: PathScope,
    workspace: &Path,
    trust: TrustMode,
) -> Result<String, String> {
    let trimmed = input_path.trim();
    if trimmed.is_empty() {
        return Err(into_invoke_err(GnomadError::PathPolicy {
            message: "Path cannot be empty.".into(),
            detail: None,
            hint: None,
        }));
    }

    if !path_needs_approval(trimmed, workspace, trust) {
        return Err(into_invoke_err(GnomadError::PathPolicy {
            message: "This path is already allowed (inside workspace or YOLO mode).".into(),
            detail: Some(trimmed.to_string()),
            hint: None,
        }));
    }

    let canonical = resolve_path_input(trimmed, workspace);
    let issued_at = unix_now();
    let expires_at = issued_at.saturating_add(TOKEN_TTL_SECS);
    let payload = TokenPayload {
        v: TOKEN_VERSION.into(),
        path_hash: path_hash(&canonical),
        nonce: random_nonce()?,
        issued_at,
        expires_at,
        scope: scope.as_str().into(),
    };

    let payload_json = serde_json::to_string(&payload).map_err(|e| internal_token_err(&e.to_string()))?;
    let secret = load_or_create_secret()?;
    let sig = sign_payload(&secret, &payload_json)?;
    let payload_b64 = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        payload_json.as_bytes(),
    );
    Ok(format!("{TOKEN_VERSION}.{payload_b64}.{sig}"))
}

fn parse_token(token: &str) -> Result<(TokenPayload, String), String> {
    let parts: Vec<&str> = token.trim().split('.').collect();
    if parts.len() != 3 || parts[0] != TOKEN_VERSION {
        return Err(invalid_token_err("Malformed path approval token."));
    }
    let payload_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        parts[1],
    )
    .map_err(|_| invalid_token_err("Malformed path token payload."))?;
    let payload_json = String::from_utf8(payload_bytes)
        .map_err(|_| invalid_token_err("Invalid path token payload."))?;
    let payload: TokenPayload = serde_json::from_str(&payload_json)
        .map_err(|_| invalid_token_err("Invalid path token payload."))?;
    Ok((payload, payload_json))
}

pub fn verify_path_approval_token(
    state: &PathTokenState,
    canonical: &Path,
    scope: PathScope,
    token: &str,
) -> Result<(), String> {
    let (payload, payload_json) = parse_token(token)?;

    if payload.v != TOKEN_VERSION {
        return Err(invalid_token_err("Unsupported path token version."));
    }

    let now = unix_now();
    if now > payload.expires_at {
        return Err(into_invoke_err(GnomadError::PathPolicy {
            message: "Path approval token expired.".into(),
            detail: None,
            hint: Some("Approve the path again in Path Gate.".into()),
        }));
    }

    if payload.scope != scope.as_str() {
        return Err(invalid_token_err("Path approval token scope mismatch."));
    }

    if payload.path_hash != path_hash(canonical) {
        return Err(invalid_token_err("Path approval token does not match this path."));
    }

    let secret = load_or_create_secret()?;
    let parts: Vec<&str> = token.trim().split('.').collect();
    verify_signature(&secret, &payload_json, parts[2])?;
    state.mark_nonce_used(&payload.nonce)?;
    Ok(())
}

/// Resolve path and enforce workspace policy. Outside-workspace access requires a signed token.
pub fn resolve_agent_path(
    input: &str,
    workspace: &Path,
    trust: TrustMode,
    path_state: &PathTokenState,
    scope: PathScope,
    path_approval_token: Option<&str>,
    path_approved: Option<bool>,
) -> Result<PathBuf, String> {
    let canonical = resolve_path_input(input, workspace);

    if trust == TrustMode::Yolo {
        return Ok(canonical);
    }

    let workspace_canon = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());

    if canonical.starts_with(&workspace_canon) {
        return Ok(canonical);
    }

    if let Some(token) = path_approval_token.filter(|t| !t.trim().is_empty()) {
        verify_path_approval_token(path_state, &canonical, scope, token)?;
        return Ok(canonical);
    }

    if path_approved == Some(true) {
        return Err(into_invoke_err(GnomadError::PathPolicy {
            message: "Unsigned path approval is not accepted.".into(),
            detail: Some(canonical.display().to_string()),
            hint: Some(
                "Approve via Path Gate in the app UI to obtain a signed approval token.".into(),
            ),
        }));
    }

    Err(into_invoke_err(GnomadError::PathPolicy {
        message: "Path is outside workspace.".into(),
        detail: Some(canonical.display().to_string()),
        hint: Some("Approve access once or enable YOLO! in Settings.".into()),
    }))
}

#[tauri::command]
pub fn issue_path_gate_token(
    path_state: tauri::State<'_, PathTokenState>,
    settings_state: tauri::State<'_, crate::agent_settings::AgentSettingsState>,
    path: String,
    scope: String,
) -> Result<String, String> {
    let _ = path_state.inner();
    let settings = crate::agent_settings::read_settings(settings_state.inner());
    let scope = PathScope::parse(&scope).ok_or_else(|| {
        into_invoke_err(GnomadError::PathPolicy {
            message: format!("Invalid path scope: {scope}"),
            detail: Some("Use read or write.".into()),
            hint: None,
        })
    })?;
    issue_path_approval_token(
        &path,
        scope,
        &settings.workspace_root,
        settings.trust_mode,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_workspace() -> PathBuf {
        std::env::temp_dir().join("gnomad-path-token-test")
    }

    #[test]
    fn issue_verify_roundtrip() {
        let ws = test_workspace();
        let _ = std::fs::create_dir_all(&ws);
        let outside = std::env::temp_dir().join("gnomad-outside-path-test");
        let state = PathTokenState::default();
        let token = issue_path_approval_token(
            &outside.to_string_lossy(),
            PathScope::Read,
            &ws,
            TrustMode::Standard,
        )
        .expect("issue");
        let canonical = resolve_path_input(&outside.to_string_lossy(), &ws);
        verify_path_approval_token(&state, &canonical, PathScope::Read, &token).expect("verify");
    }

    #[test]
    fn rejects_boolean_bypass() {
        let ws = test_workspace();
        let outside = "/etc/hosts";
        let state = PathTokenState::default();
        let err = resolve_agent_path(
            outside,
            &ws,
            TrustMode::Standard,
            &state,
            PathScope::Read,
            None,
            Some(true),
        )
        .unwrap_err();
        assert!(err.contains("Unsigned path approval"));
    }

    #[test]
    fn rejects_replay() {
        let ws = test_workspace();
        let outside = std::env::temp_dir().join("gnomad-replay-path");
        let state = PathTokenState::default();
        let token = issue_path_approval_token(
            &outside.to_string_lossy(),
            PathScope::Write,
            &ws,
            TrustMode::Standard,
        )
        .unwrap();
        let canonical = resolve_path_input(&outside.to_string_lossy(), &ws);
        verify_path_approval_token(&state, &canonical, PathScope::Write, &token).unwrap();
        assert!(verify_path_approval_token(&state, &canonical, PathScope::Write, &token).is_err());
    }
}
