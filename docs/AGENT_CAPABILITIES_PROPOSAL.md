# Agent Terminal & Filesystem Capabilities

**Status:** Implemented in v0.2 agent runtime (Phases 0–3).

## Summary

Gnomad is a **command-and-filesystem agent**, not a chatbot that pretends to run shell commands. Cloud chat uses **LLM tool calls** (`shell_run`, `fs_*`, `workspace_info`). Local Ollama falls back to `<gnomad-run>` tags.

## Trust modes

| Mode | Behavior |
|------|----------|
| **Standard** (default) | Read/write/search under workspace (`$HOME` until changed). Paths outside workspace need approval. Sudo Gate for risky shell. |
| **YOLO!** | Full-machine file access; fewer path prompts. Sudo Gate may still apply for destructive shell. |

## Architecture

- **Rust:** `agent_runtime`, `agent_fs`, `agent_settings`, `shell_session` (PTY + cwd tracking)
- **Frontend:** `agentLoop.ts` orchestrates up to 10 tool steps per message
- **Audit:** `{app_data}/gnomad/agent-audit.jsonl`

## User actions

- **Settings → Agent access:** workspace folder, trust mode
- **Stop** while thinking: interrupts PTY
- Paste `$ command` or use terminal button for direct run

See plan history in `.cursor/plans/` for full approach comparison.

## Local command planner (optional)

Settings → **Agent access** → **Local command planner**:

- Checkbox to enable
- **Planner model (Ollama)** — small/fast tag (e.g. `llama3.2:1b`), separate from main chat model
- Optional **Use chat local model** when in Ollama mode
- **GGUF path** — stored for future direct inference; v1 uses Ollama only

Runs when the main model emits prose instead of a valid shell command.
