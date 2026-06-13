//! Platform-agnostic Gnomad backend — extracted from the Tauri shell for Linux-native GTK.
//!
//! Phase 0: error handling, config paths, secrets, safety tokens, events bus, and LLM helpers.
//! Tauri and GTK shells depend on this crate; neither should duplicate business logic.

pub mod agent;
pub mod chat;
pub mod config;
pub mod help;
pub mod knowledge;
pub mod error;
pub mod events;
pub mod llm;
pub mod migrate;
pub mod platform;
pub mod shell;

pub use error::{into_invoke_err, AgentErrorPayload, GnomadError};
pub use events::{EventBus, GnomadEvent};
pub use platform::{GnomadCoreContext, PlatformContext};
