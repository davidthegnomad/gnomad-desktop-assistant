pub const AGENT_SYSTEM_TOOLS: &str = r#"## Agent tools
You can call tools to run commands and manage files on the user's Linux machine. Use tools instead of only describing actions.
- shell_run: run a one-line shell command (open apps, xdg-open URLs/files, list dirs, etc.)
- workspace_info: get workspace root and trust mode
- fs_list, fs_read, fs_write, fs_search: file operations relative to workspace
After tools run, summarize results for the user. Do not claim success without tool output."#;

/// Extra instructions for Ollama models that do not support native tool calling.
pub const LOCAL_AGENT_JSON_INSTRUCTION: &str = r#"## Tool calling (local model)
When you need to run a command or access files, reply with a single JSON line (no markdown):
{"tool":"TOOL_NAME","arguments":{...}}
Examples:
{"tool":"fs_list","arguments":{"path":"."}}
{"tool":"shell_run","arguments":{"command":"ls -la"}}
Use tool names: shell_run, workspace_info, fs_list, fs_read, fs_write, fs_search.
After you receive tool results in a follow-up message, summarize for the user."#;

pub fn parse_text_tool_calls(content: &str) -> Vec<crate::llm::AgentToolCall> {
    use crate::llm::AgentToolCall;
    let mut calls = Vec::new();
    let mut id = 0u64;
    for line in content.lines() {
        let trimmed = line
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) else {
            continue;
        };
        let name = v
            .get("tool")
            .or_else(|| v.get("name"))
            .and_then(|x| x.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let Some(name) = name else {
            continue;
        };
        let arguments = v
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        id += 1;
        calls.push(AgentToolCall {
            id: format!("local-{id}"),
            name: name.to_string(),
            arguments: arguments.to_string(),
        });
    }
    calls
}

pub fn agent_tool_definitions() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "shell_run",
                "description": "Run a shell command on the user's machine. Returns stdout, exit code, and state.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "One-line shell command" }
                    },
                    "required": ["command"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "workspace_info",
                "description": "Get workspace root, trust mode, and shell cwd.",
                "parameters": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "fs_list",
                "description": "List files in a directory relative to workspace.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Relative path or . for workspace root" }
                    }
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "fs_read",
                "description": "Read a text file relative to workspace.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    },
                    "required": ["path"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "fs_write",
                "description": "Write or overwrite a file relative to workspace.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["path", "content"]
                }
            }
        }),
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "fs_search",
                "description": "Search for text in files under a directory.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "path": { "type": "string" }
                    },
                    "required": ["query"]
                }
            }
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_tool_line() {
        let calls = parse_text_tool_calls(r#"{"tool":"fs_list","arguments":{"path":"."}}"#);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "fs_list");
    }
}
