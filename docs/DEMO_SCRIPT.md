# Demo Script — Gnomad Desktop Assistant

**Duration:** 5–7 minutes  
**Audience:** Portfolio review, interview, or investor technical screen  
**Prerequisites:** Installed alpha build, DeepSeek key **or** Ollama running

---

## Setup (before you share screen)

1. Quit other assistants/tray clutter.  
2. Confirm Gnomad is in tray; window hidden.  
3. Pre-stage: one text file in Downloads, Ollama model pulled if local demo.  
4. Optional: add a short `skills/demo-skill.md` in knowledge (“Prefer concise answers”).  
5. Close sensitive windows; copy a harmless clipboard string (e.g. “Q2 roadmap draft”).

---

## Act 1 — Native presence (45 sec)

**Say:** “Gnomad is a native desktop assistant—not a browser tab. It lives in the menu bar / tray and respects OS conventions.”

1. Press **⌘⇧Space** (or **Ctrl+Shift+Space**) → panel appears.  
2. Point out **context footer**: active app, window title, clipboard.  
3. Click tray → show **Pop out** vs **Windowed** (brief).  
4. Close window → “It hides to tray; it doesn’t quit.”

---

## Act 2 — Onboarding & models (60 sec)

**Say:** “Users choose cloud or local inference; secrets go to the system keychain.”

1. Open **Settings** (gear).  
2. Show **API keys** — configured, not visible.  
3. Toggle **Provider**: Cloud vs Local; change **model** dropdown.  
4. Mention `.env` for developers (optional one line).

---

## Act 3 — Conversation (90 sec)

**Say:** “Chat is grounded in what you’re working on.”

1. New chat from sidebar.  
2. Click a **suggestion chip** or type:  
   *“Summarize what I’m working on based on my window and clipboard.”*  
3. Show assistant reply referencing context pills.  
4. Attach a small file → send → note attachment pill in thread.

---

## Act 4 — Agent & safety (2 min) — highlight reel

**Say:** “This is an agent that executes—not theater. Safety is explicit.”

1. **Safe shell:**  
   *“Run `echo Hello from Gnomad` and show the output.”*  
   → Point at **shell block** with stdout.

2. **Persistent cwd:**  
   *“Run `pwd` then `cd Desktop` and `pwd` again.”*  
   → Show cwd continuity in settings/shell block.

3. **Sudo Gate (careful):**  
   *“Try `sudo echo test`”* or use a flagged pattern.  
   → **Deny** first (“user stays in control”), then optionally approve on a safe test machine.

4. **Stop:** Start a longer command; click **Stop** during thinking.

---

## Act 5 — Knowledge (45 sec)

**Say:** “Skills and docs live on disk and inject into every prompt.”

1. Open **Knowledge & skills** (book icon).  
2. Show folder structure / add file.  
3. Ask a question that only your staged skill can answer.

---

## Act 6 — Engineering story (30 sec close)

**Say:** “Under the hood: Tauri 2, React 19, Rust for shell and FS. CI builds macOS, Windows, and Linux installers.”

Show (optional, 10 sec): GitHub Actions or `docs/ARCHITECTURE.md` diagram.

**Close:** “Alpha today; roadmap includes hardened HITL, tests, and broader packaging.”

---

## Fallbacks if something breaks

| Issue | Fallback line |
|-------|----------------|
| Cloud API down | Switch to local Ollama |
| Ollama down | Show pre-recorded screenshot of shell block |
| Context empty (Linux) | “Optional packages xdotool/wl-paste; core chat still works” |
| Permission banner | “macOS Accessibility—one-time grant” |

---

## Do not demo on a live stream

- Real `rm -rf`  
- Production API keys in view  
- Customer data paths  
- YOLO mode on a machine with secrets  

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
