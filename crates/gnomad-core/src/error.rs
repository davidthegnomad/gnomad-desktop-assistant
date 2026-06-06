use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentErrorPayload {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    pub retryable: bool,
}

#[derive(Debug, Error)]
pub enum GnomadError {
    #[error("{message}")]
    ShellValidation {
        message: String,
        detail: Option<String>,
    },
    #[error("{message}")]
    ShellExecution {
        message: String,
        detail: Option<String>,
    },
    #[error("{message}")]
    SafetyBlocked {
        message: String,
        detail: Option<String>,
        hint: Option<String>,
    },
    #[error("{message}")]
    PathPolicy {
        message: String,
        detail: Option<String>,
        hint: Option<String>,
    },
    #[error("{message}")]
    ElevationUnsupported {
        message: String,
        hint: Option<String>,
    },
    #[error("{message}")]
    Fs {
        message: String,
        detail: Option<String>,
    },
    #[error("{message}")]
    Llm {
        message: String,
        detail: Option<String>,
    },
    #[error("{message}")]
    Keychain {
        message: String,
    },
    #[error("{message}")]
    Internal {
        message: String,
        detail: Option<String>,
    },
}

impl GnomadError {
    pub fn to_payload(&self) -> AgentErrorPayload {
        match self {
            GnomadError::ShellValidation { message, detail } => AgentErrorPayload {
                code: "shell_validation".into(),
                message: message.clone(),
                detail: detail.clone(),
                hint: Some(
                    "Use a real CLI command, e.g. command -v brew or brew install <pkg>.".into(),
                ),
                retryable: false,
            },
            GnomadError::ShellExecution { message, detail } => AgentErrorPayload {
                code: "shell_execution".into(),
                message: message.clone(),
                detail: detail.clone(),
                hint: None,
                retryable: true,
            },
            GnomadError::SafetyBlocked {
                message,
                detail,
                hint,
            } => AgentErrorPayload {
                code: "safety_blocked".into(),
                message: message.clone(),
                detail: detail.clone(),
                hint: hint.clone(),
                retryable: false,
            },
            GnomadError::PathPolicy {
                message,
                detail,
                hint,
            } => AgentErrorPayload {
                code: "path_policy".into(),
                message: message.clone(),
                detail: detail.clone(),
                hint: hint
                    .clone()
                    .or(Some("Approve the path or enable YOLO! in Settings.".into())),
                retryable: false,
            },
            GnomadError::ElevationUnsupported { message, hint } => AgentErrorPayload {
                code: "elevation_unsupported".into(),
                message: message.clone(),
                detail: None,
                hint: hint.clone(),
                retryable: false,
            },
            GnomadError::Fs { message, detail } => AgentErrorPayload {
                code: "fs".into(),
                message: message.clone(),
                detail: detail.clone(),
                hint: None,
                retryable: false,
            },
            GnomadError::Llm { message, detail } => AgentErrorPayload {
                code: "llm".into(),
                message: message.clone(),
                detail: detail.clone(),
                hint: None,
                retryable: true,
            },
            GnomadError::Keychain { message } => AgentErrorPayload {
                code: "keychain".into(),
                message: message.clone(),
                detail: None,
                hint: Some("Check API keys in Settings or .env.".into()),
                retryable: false,
            },
            GnomadError::Internal { message, detail } => AgentErrorPayload {
                code: "internal".into(),
                message: message.clone(),
                detail: detail.clone(),
                hint: None,
                retryable: true,
            },
        }
    }
}

/// Serialize error for UI invoke boundaries (`Result<_, String>`).
pub fn into_invoke_err(e: GnomadError) -> String {
    serde_json::to_string(&e.to_payload()).unwrap_or_else(|_| e.to_string())
}

pub fn internal_err(msg: impl Into<String>, detail: Option<String>) -> String {
    into_invoke_err(GnomadError::Internal {
        message: msg.into(),
        detail,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_codes_are_stable() {
        let e = GnomadError::SafetyBlocked {
            message: "blocked".into(),
            detail: None,
            hint: None,
        };
        let p = e.to_payload();
        assert_eq!(p.code, "safety_blocked");
        let json = into_invoke_err(e);
        assert!(json.contains("safety_blocked"));
    }
}
