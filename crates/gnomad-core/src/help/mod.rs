//! Bundled quick-help text for users and agent context injection.

const QUICK_HELP: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../gnomad-gtk/resources/GNOMAD_HELP.md"));

const CONTEXT_MAX_CHARS: usize = 2_000;

/// Full quick-help markdown (bundled at compile time).
pub fn quick_help_text() -> &'static str {
    QUICK_HELP
}

/// Trimmed snippet for LLM system context (caps size to avoid bloating prompts).
pub fn context_snippet() -> String {
    let trimmed = QUICK_HELP.trim();
    if trimmed.chars().count() <= CONTEXT_MAX_CHARS {
        return trimmed.to_string();
    }
    let short: String = trimmed.chars().take(CONTEXT_MAX_CHARS).collect();
    format!("{short}…")
}
