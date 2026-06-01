# GGUF setup — embedded local LLM

Gnomad can run **small GGUF models in-process** for the command planner and (optionally) local chat when built with the `embedded-llm` feature. Models are **not bundled** in the repo or default installers (size and licensing).

---

## Quick start

1. **Build with embedded LLM:**

   ```bash
   npm run tauri:dev:embedded
   ```

2. **Download a small model** (optional helper):

   ```bash
   npm run download:gguf
   ```

   Default: Qwen2.5-Coder 1.5B Instruct Q4 (~1 GB) into `~/.gnomad/models/`.

3. **Settings → Agent access → GGUF path** — paste the full path to the `.gguf` file.

4. Enable **Command planner** and/or **Use GGUF for local chat** (experimental).

---

## Custom download

```bash
bash scripts/download-gguf-model.sh /path/to/output/dir
# or with a custom URL:
GGUF_URL="https://huggingface.co/.../model.gguf" bash scripts/download-gguf-model.sh ~/models
```

After download, set the GGUF path in Settings to the printed file path.

---

## Recommended models (dev)

| Model | Size (approx) | Use case |
|-------|----------------|----------|
| Qwen2.5-Coder 1.5B Q4 | ~1 GB | Command planner, light local chat |
| Llama 3.2 3B Q4 | ~2 GB | General chat (slower on CPU) |

Use **Q4_K_M** or similar quantizations for laptop CPU inference.

---

## CI note

Default GitHub Actions builds **omit** `embedded-llm` to keep matrix fast. Enable locally or in a dedicated workflow when testing GGUF.

---

## Related

- [USER_GUIDE.md](USER_GUIDE.md) — embedded GGUF section  
- [WAVE_B_ROADMAP.md](WAVE_B_ROADMAP.md) — B2 design  
- [BUILD.md](BUILD.md) — build flags

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
