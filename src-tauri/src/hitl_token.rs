use crate::error::{into_invoke_err, GnomadError};
use crate::privilege::check_command_safety;
#[cfg(not(test))]
use crate::keychain::get_credential_value;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

const KEYCHAIN_HITL_SECRET: &str = "hitl-hmac-secret";
const TOKEN_VERSION: &str = "v1";
const TOKEN_TTL_SECS: u64 = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitlScope {
    ShellRun,
    Elevated,
}

impl HitlScope {
    pub fn as_str(self) -> &'static str {
        match self {
            HitlScope::ShellRun => "shell_run",
            HitlScope::Elevated => "elevated",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "shell_run" | "shell" => Some(HitlScope::ShellRun),
            "elevated" | "elevate" => Some(HitlScope::Elevated),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenPayload {
    v: String,
    command_hash: String,
    nonce: String,
    issued_at: u64,
    expires_at: u64,
    scope: String,
}

pub struct HitlTokenState {
    used_nonces: Mutex<HashSet<String>>,
}

impl Default for HitlTokenState {
    fn default() -> Self {
        Self {
            used_nonces: Mutex::new(HashSet::new()),
        }
    }
}

impl HitlTokenState {
    fn mark_nonce_used(&self, nonce: &str) -> Result<(), String> {
        let mut guard = self
            .used_nonces
            .lock()
            .map_err(|e| internal_token_err(&e.to_string()))?;
        if guard.contains(nonce) {
            return Err(into_invoke_err(GnomadError::SafetyBlocked {
                message: "Approval token was already used.".into(),
                detail: None,
                hint: Some("Approve the command again in Sudo Gate.".into()),
            }));
        }
        guard.insert(nonce.to_string());
        Ok(())
    }
}

fn internal_token_err(detail: &str) -> String {
    into_invoke_err(GnomadError::Internal {
        message: "HITL token store error.".into(),
        detail: Some(detail.to_string()),
    })
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Normalize command for hashing (trim whitespace).
pub fn normalize_command(command: &str) -> String {
    command.trim().to_string()
}

pub fn command_hash(command: &str) -> String {
    let normalized = normalize_command(command);
    let digest = Sha256::digest(normalized.as_bytes());
    hex::encode(digest)
}

#[cfg(test)]
fn load_or_create_secret() -> Result<Vec<u8>, String> {
    Ok(b"gnomad-hitl-test-secret-fixed-key!!".to_vec())
}

#[cfg(not(test))]
fn load_or_create_secret() -> Result<Vec<u8>, String> {
    let existing = get_credential_value(KEYCHAIN_HITL_SECRET)?;
    if !existing.trim().is_empty() {
        return Ok(existing.into_bytes());
    }
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).map_err(|e| {
        into_invoke_err(GnomadError::Internal {
            message: "Failed to generate HITL signing secret.".into(),
            detail: Some(e.to_string()),
        })
    })?;
    let encoded = hex::encode(bytes);
    if let Err(e) = crate::keychain::store_credential(KEYCHAIN_HITL_SECRET, &encoded) {
        let retry = get_credential_value(KEYCHAIN_HITL_SECRET)?;
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
    let sig = mac.finalize().into_bytes();
    Ok(hex::encode(sig))
}

fn verify_signature(secret: &[u8], payload_json: &str, sig_hex: &str) -> Result<(), String> {
    let sig_bytes = hex::decode(sig_hex).map_err(|_| invalid_token_err("Invalid token signature."))?;
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|e| internal_token_err(&e.to_string()))?;
    mac.update(payload_json.as_bytes());
    mac.verify_slice(&sig_bytes)
        .map_err(|_| invalid_token_err("Approval token signature mismatch."))
}

fn invalid_token_err(message: &str) -> String {
    into_invoke_err(GnomadError::SafetyBlocked {
        message: message.into(),
        detail: None,
        hint: Some("Approve the command again in Sudo Gate.".into()),
    })
}

fn random_nonce() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).map_err(|e| internal_token_err(&e.to_string()))?;
    Ok(hex::encode(bytes))
}

/// Issue a signed approval token after UI approval. Command must require HITL.
pub fn issue_approval_token(command: &str, scope: HitlScope) -> Result<String, String> {
    let normalized = normalize_command(command);
    if normalized.is_empty() {
        return Err(into_invoke_err(GnomadError::ShellValidation {
            message: "Command cannot be empty.".into(),
            detail: None,
        }));
    }

    let safety = check_command_safety(&normalized);
    if !safety.requires_hitl_approval {
        return Err(into_invoke_err(GnomadError::SafetyBlocked {
            message: "This command does not require safety approval.".into(),
            detail: None,
            hint: None,
        }));
    }

    if scope == HitlScope::Elevated && !safety.requires_admin {
        return Err(into_invoke_err(GnomadError::SafetyBlocked {
            message: "Elevated approval token requires an administrative command.".into(),
            detail: None,
            hint: Some("Use shell_run scope for non-admin commands.".into()),
        }));
    }

    let issued_at = unix_now();
    let expires_at = issued_at.saturating_add(TOKEN_TTL_SECS);
    let payload = TokenPayload {
        v: TOKEN_VERSION.into(),
        command_hash: command_hash(&normalized),
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
        return Err(invalid_token_err("Malformed approval token."));
    }
    let payload_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        parts[1],
    )
    .map_err(|_| invalid_token_err("Malformed approval token payload."))?;
    let payload_json = String::from_utf8(payload_bytes)
        .map_err(|_| invalid_token_err("Invalid approval token payload."))?;
    let payload: TokenPayload = serde_json::from_str(&payload_json)
        .map_err(|_| invalid_token_err("Invalid approval token payload."))?;
    Ok((payload, payload_json))
}

/// Verify token, consume nonce, ensure command + scope match.
pub fn verify_approval_token(
    state: &HitlTokenState,
    command: &str,
    scope: HitlScope,
    token: &str,
) -> Result<(), String> {
    let normalized = normalize_command(command);
    let (payload, payload_json) = parse_token(token)?;

    if payload.v != TOKEN_VERSION {
        return Err(invalid_token_err("Unsupported approval token version."));
    }

    let now = unix_now();
    if now > payload.expires_at {
        return Err(into_invoke_err(GnomadError::SafetyBlocked {
            message: "Approval token expired.".into(),
            detail: None,
            hint: Some("Approve the command again in Sudo Gate.".into()),
        }));
    }

    if payload.scope != scope.as_str() {
        return Err(invalid_token_err("Approval token scope mismatch."));
    }

    if payload.command_hash != command_hash(&normalized) {
        return Err(invalid_token_err("Approval token does not match this command."));
    }

    let secret = load_or_create_secret()?;
    let parts: Vec<&str> = token.trim().split('.').collect();
    verify_signature(&secret, &payload_json, parts[2])?;

    state.mark_nonce_used(&payload.nonce)?;

    Ok(())
}

/// Enforce HITL: safe commands pass; risky commands require a valid token (boolean bypass rejected).
pub fn enforce_hitl(
    state: &HitlTokenState,
    command: &str,
    scope: HitlScope,
    approval_token: Option<&str>,
    hitl_approved: Option<bool>,
) -> Result<(), String> {
    let normalized = normalize_command(command);
    let safety = check_command_safety(&normalized);

    if !safety.requires_hitl_approval {
        return Ok(());
    }

    if let Some(token) = approval_token.filter(|t| !t.trim().is_empty()) {
        return verify_approval_token(state, &normalized, scope, token);
    }

    if hitl_approved == Some(true) {
        return Err(into_invoke_err(GnomadError::SafetyBlocked {
            message: "Unsigned HITL approval is not accepted.".into(),
            detail: Some(normalized.chars().take(200).collect()),
            hint: Some(
                "Approve via Sudo Gate in the app UI to obtain a signed approval token.".into(),
            ),
        }));
    }

    Err(into_invoke_err(GnomadError::SafetyBlocked {
        message: safety
            .danger_reason
            .clone()
            .unwrap_or_else(|| "Command requires safety approval.".into()),
        detail: Some(normalized),
        hint: Some("Review and approve in Sudo Gate.".into()),
    }))
}

#[tauri::command]
pub fn issue_hitl_approval_token(command: String, scope: String) -> Result<String, String> {
    let scope = HitlScope::parse(&scope).ok_or_else(|| {
        into_invoke_err(GnomadError::ShellValidation {
            message: format!("Invalid HITL scope: {scope}"),
            detail: Some("Use shell_run or elevated.".into()),
        })
    })?;
    issue_approval_token(&command, scope)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_verify_roundtrip() {
        let state = HitlTokenState::default();
        let cmd = "sudo rm -rf /tmp/gnomad-test";
        let token = issue_approval_token(cmd, HitlScope::ShellRun).expect("issue");
        verify_approval_token(&state, cmd, HitlScope::ShellRun, &token).expect("verify");
    }

    #[test]
    fn rejects_wrong_command() {
        let state = HitlTokenState::default();
        let token = issue_approval_token("sudo ls", HitlScope::ShellRun).unwrap();
        assert!(verify_approval_token(&state, "sudo ls -la", HitlScope::ShellRun, &token).is_err());
    }

    #[test]
    fn rejects_replay() {
        let state = HitlTokenState::default();
        let cmd = "chmod 777 /tmp/gnomad-test";
        let token = issue_approval_token(cmd, HitlScope::ShellRun).unwrap();
        verify_approval_token(&state, cmd, HitlScope::ShellRun, &token).unwrap();
        assert!(verify_approval_token(&state, cmd, HitlScope::ShellRun, &token).is_err());
    }

    #[test]
    fn rejects_boolean_bypass() {
        let state = HitlTokenState::default();
        let err = enforce_hitl(&state, "sudo ls", HitlScope::ShellRun, None, Some(true)).unwrap_err();
        assert!(err.contains("Unsigned HITL"));
    }

    #[test]
    fn safe_command_without_token_ok() {
        let state = HitlTokenState::default();
        enforce_hitl(&state, "echo hello", HitlScope::ShellRun, None, None).unwrap();
    }
}
