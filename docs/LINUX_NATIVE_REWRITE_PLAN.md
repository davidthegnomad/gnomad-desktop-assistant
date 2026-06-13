# Gnomad Linux-Native Rewrite Plan

**Status:** Proposal (June 2026)  
**Trigger:** Tauri 2 + WebKitGTK on KDE Wayland cannot reliably support resize, window mode switching, or stable tray-panel UX. Patching the macOS-oriented shell is not viable.

**Goal:** Same product capabilities and visual identity as the macOS build, implemented with a **Linux-first** stack that treats KDE/Wayland as the primary platform—not a port of macOS window semantics.

---

## 1. macOS product spec (source of truth)

The current macOS build is **Gnomad v0.2.0-beta.1** — a Tauri 2 + React 19 + Rust app (`com.gnomadstudio.gnomad`).

### 1.1 Core product identity

| Attribute | Value |
|-----------|--------|
| Brand | Gnomad 🍄 — gnomadstudio.org |
| Access pattern | Menu bar / system tray; close hides to tray; global shortcut toggles panel |
| Primary workflow | AI chat + optional local agent (shell + filesystem tools) |
| Design language | “Gemini-inspired Neural Expressive” — dark/light/system themes, 16px radius, blue accent, gradient bloom background |

### 1.2 Window modes (macOS behavior — target parity)

| Mode | Size | macOS behavior |
|------|------|----------------|
| **Panel** | 600×920 | Menu-bar accessory; always-on-top; drops below menu bar; **compact UI** |
| **Pop out** | 1283×858 | Always-on-top floating; centered or under tray |
| **Window** | 1283×858 | Normal app window; regular activation policy |
| **Fullscreen** | — | Native fullscreen |

Constraints: min 520×420, max 1600×1200. macOS uses overlay title bar + traffic lights; `fit_window_to_content` grows window from webview.

### 1.3 Feature inventory (must preserve on Linux)

**Access & shell**
- System tray with menu (Show, Pop Out, Settings, About, Quit)
- Global shortcut: `Ctrl+Shift+Space` (toggle)
- Native menus: File, Edit, View, Window, Help
- Four window mode chips in toolbar
- Theme cycle (light / dark / system)

**Chat & composer**
- Welcome state + 3 random suggestion chips (22-prompt pool)
- Multi-session chat history (sidebar rail, collapse, resize 160–420px, left/right)
- Provider select: Cloud vs Local Ollama
- Model select: live Ollama `/api/tags` list; DeepSeek/cloud presets
- Attachments: up to 10 files, 48KB text extract; images/pdf/text
- Voice input (opt-in, Web Speech–class API on macOS)
- Thinking state + live PTY stream in terminal view
- Shell command cards in chat history
- Context footer pills: active app, window title, clipboard snippet (~30 chars)

**LLM**
- Cloud: OpenAI-compatible (DeepSeek default); keychain + `.env`
- Local: Ollama chat + model discovery
- Optional: embedded GGUF via llama-cpp (feature flag)
- Multi-step agent loop (max 10 steps) with tools: `shell_run`, `workspace_info`, `fs_*`

**Safety**
- Standard vs YOLO trust modes
- Sudo Gate modal + HMAC-signed approval tokens
- Path Gate modal + signed path tokens
- Command safety check before execution
- Optional YOLO sandbox (bwrap on Linux)
- Agent audit log

**Knowledge**
- Categories: skills, agents, preferences, uploads
- Import/delete files; bundled skill packs
- Context bundle (12k chars) injected into prompts

**Settings**
- Model & API, API keys, cloud endpoint presets
- Agent access (workspace, trust, command planner, GGUF)
- Updates (stable/beta, auto-check)
- Privacy (error log, voice)
- Linux integration diagnostics (already started)
- Onboarding wizard (2-step)

**Backend surface**
- ~55 Tauri invoke commands across 30 Rust modules
- Persistence: keychain, `~/.local/share` or XDG data dir, localStorage prefs on web UI

### 1.4 macOS-only mechanisms (do not port literally)

| macOS mechanism | Linux replacement |
|-----------------|-------------------|
| `ActivationPolicy::Accessory` | Tray + optional Layer Shell / KDE dialog placement |
| Overlay title bar + traffic lights | Native SSD (Server-Side Decorations) |
| AppleScript active window | KWin D-Bus / `hyprctl` / X11 `xdotool` (existing `linux_context.rs`) |
| `pbpaste` clipboard | `wl-paste` / `xclip` |
| `sandbox-exec` YOLO | `bubblewrap` |
| `screencapture` | `grim` / portal screenshot |
| WebKit entire shell | Native GTK window + targeted embed only if needed |

---

## 2. Why Tauri is the wrong Linux shell

| Problem | Root cause |
|---------|------------|
| Resize crashes | WebKitGTK + GTK window property churn on Wayland |
| Mode switch crashes | Burst `set_size` / `always_on_top` / `decorations` calls |
| Vertical resize blocked | GTK min-size / compositor fights with webview layout |
| “Linux lane” in same binary | macOS accessory model baked into `window_manager.rs` |
| EGL/ZINK spam | NVIDIA multi-GPU + WebKit compositing on Wayland |

**Conclusion:** Keep the **Rust business logic**. Replace **Tauri + React shell** with a **native Linux UI layer**.

---

## 3. Recommended Linux-first stack

### Primary recommendation: **Rust + GTK 4 + Libadwaita**

| Layer | Technology | Rationale |
|-------|------------|-----------|
| Language | **Rust** | 30 existing backend modules; memory safety; single static binary |
| UI toolkit | **GTK 4 + Libadwaita** | First-class on KDE & GNOME Wayland; native SSD; no WebKit for chrome |
| Tray | **`ksni` / `libappindicator` via gtk-rs** | StatusNotifierItem — same as current tray |
| Async | **tokio + glib async** | PTY streaming, Ollama HTTP, LLM calls |
| Secrets | **`keyring` crate** → Secret Service | Already implemented |
| PTY | **`portable-pty`** | Already implemented |
| HTTP | **`reqwest`** | Already implemented |
| Optional chat rich text | **`gtk4::TextView` + `pulldown-cmark`** OR small WebKitGTK embed **only for message pane** | Avoid full-window WebKit |

### Why not other “Linux-first” options?

| Option | Verdict |
|--------|---------|
| **Python + PyQt6** | Faster UI iteration but GIL + packaging weight; rewrite all Rust backend |
| **C++ / Qt6 / Kirigami** | Excellent on KDE; throws away entire Rust codebase |
| **Electron / NW.js** | Same WebKit/Chromium resize class of problems |
| **Tauri (keep patching)** | Proven failure on your machine |
| **Pure Wayland layer-shell only** | Great for panel; insufficient for full settings/knowledge/agent UI |
| **Slint / Iced** | Portable Rust UI; weaker KDE tray/Wayland integration vs GTK |

### UI fidelity strategy

Exact pixel parity with React is expensive. Target **functional + visual parity**:

1. Port CSS design tokens → **GTK CSS** (`gtk.css`) — colors, radii, typography
2. Rebuild layout with `AdwNavigationView`, `AdwToolbarView`, `GtkPaned` (sidebar)
3. Chat messages: markdown → GTK widgets; code blocks in `GtkSourceView` or monospace `TextView`
4. **Cursor bloom:** optional `GtkDrawingArea` shader or static gradient (disable on low-end GPU)
5. xterm.js → **`vte4` crate** (native terminal widget) for live PTY — better Linux fit

---

## 4. Target architecture

```
gnomad-desktop-assistant/
├── crates/
│   ├── gnomad-core/              # Extracted from src-tauri/src (no Tauri deps)
│   │   ├── llm/                  # Ollama, cloud, tools, agent loop
│   │   ├── shell/                # PTY session, sandbox, safety
│   │   ├── agent/                # fs tools, runtime, audit, tokens
│   │   ├── knowledge/            # store, skill packs
│   │   ├── chat/                 # session persistence
│   │   ├── context/              # active window, clipboard (linux_context)
│   │   ├── config/               # keychain, env, settings JSON
│   │   └── platform/             # traits: Tray, Window, Screenshot, Elevate
│   │
│   └── gnomad-gtk/               # Linux-first application binary
│       ├── main.rs
│       ├── app.rs                # AdwApplication lifecycle
│       ├── tray/                 # StatusNotifierItem + menu
│       ├── window/
│       │   ├── panel.rs          # Layer-shell or positioned popup
│       │   ├── floating.rs
│       │   ├── windowed.rs
│       │   └── fullscreen.rs
│       ├── ui/
│       │   ├── chat_view.rs
│       │   ├── composer.rs
│       │   ├── sidebar.rs
│       │   ├── settings.rs
│       │   ├── knowledge.rs
│       │   └── modals/           # gates, onboarding, about
│       ├── state.rs              # glib::Sendable app state bridge
│       └── resources/            # gresource XML, icons, css
│
├── gnomad-mac/                   # (later) thin Tauri or Swift wrapper — out of scope
└── docs/
```

### Linux window model (correct semantics)

```
┌─────────────────────────────────────────────────────────┐
│  KStatusNotifierItem (tray)                             │
│    ├─ left-click (Wayland): menu                        │
│    ├─ right-click: toggle panel                         │
│    └─ global shortcut: toggle panel                     │
├─────────────────────────────────────────────────────────┤
│  Panel mode:                                            │
│    • GtkWindow (popup) OR wlr-layer-shell overlay       │
│    • Fixed 600×920 default; compact GTK layout          │
│    • Position: top-right under tray (KDE)               │
│    • NOT always-on-top hack — use layer-shell exclusive │
├─────────────────────────────────────────────────────────┤
│  Pop out: GtkWindow, floating, resizable, always-on-top │
│  Window:  GtkApplicationWindow, SSD, resizable           │
│  Full:    gtk_window_fullscreen()                       │
└─────────────────────────────────────────────────────────┘
```

**KDE-specific enhancement (Phase 2+):** optional `plasma6` pattern — register as KWin script assisted placement for panel anchor.

---

## 5. Feature parity matrix & phases

| Feature | macOS today | Linux GTK target | Phase |
|---------|-------------|------------------|-------|
| Tray + menu | ✅ | `ksni` + GMenu | **P0** |
| Global shortcut | ✅ | `global-hotkey` crate or portal | **P0** |
| Panel 600×920 compact | ✅ | Layer-shell / popup GtkWindow | **P0** |
| Pop out / Window / Full | ✅ | Separate GtkWindow policies | **P0** |
| Resize all modes | ✅ | Native GTK — no WebKit shell | **P0** |
| Chat + sessions | ✅ | `gnomad-core` + GTK list | **P1** |
| Ollama + cloud LLM | ✅ | `gnomad-core` direct | **P1** |
| Agent + PTY stream | ✅ | `gnomad-core` + `vte4` | **P1** |
| Sudo/Path gates | ✅ | AdwDialog modals | **P1** |
| Context pills | ✅ | `linux_context` + GTK labels | **P1** |
| Attachments | ✅ | `gnomad-core` + GTK file chooser | **P2** |
| Knowledge library | ✅ | GtkListView + file manager | **P2** |
| Settings (all sections) | ✅ | AdwPreferencesWindow | **P2** |
| Onboarding | ✅ | AdwDialog wizard | **P2** |
| Updater | ✅ | AppImage/deb + `pkexec` or GitHub API check | **P2** |
| Themes light/dark | ✅ | `AdwStyleManager` + custom CSS | **P1** |
| Voice input | ✅ | `speech-dispatcher` / vosk optional | **P3** |
| Embedded GGUF | ✅ | `gnomad-core` feature flag | **P3** |
| Cursor bloom | ✅ | GtkDrawingArea (optional) | **P3** |
| Automation (enigo) | ✅ | `gnomad-core` | **P3** |

---

## 6. Implementation phases (estimated)

### Phase 0 — Extract core (1–2 weeks)
- [x] Create `crates/gnomad-core` from `src-tauri/src/*` minus Tauri types (foundation modules)
- [x] Replace `tauri::AppHandle` with `PlatformContext` trait + `DataPaths` (Tauri uses `core_bridge`)
- [x] Replace `emit()` events with `tokio::broadcast` `EventBus` (GTK shell will subscribe in Phase 1)
- [x] Unit tests for LLM, tokens, shell safety (22 tests, `cargo test -p gnomad-core`)
- [x] Shared config paths via `xdg` crate (`com.gnomadstudio.gnomad`)
- [x] `chat_history` → `gnomad-core/src/chat/store.rs` (shared `store.json` path)
- [x] Basic `chat_completion` (Ollama + cloud) → `gnomad-core/src/llm/completion.rs`
- [ ] Remaining modules in `src-tauri` (shell_session, agent_runtime, knowledge) — Phase 0.5

### Phase 1 — Linux shell MVP (2–3 weeks)
- [x] `gnomad-gtk` binary: AdwApplication + ksni tray (code complete; needs `gtk4-devel` to build)
- [x] Panel popup 600×920 with placeholder chat UI
- [x] Four mode switches with **real** GTK window policies
- [x] Resize verified on KDE Wayland (QA passed)
- [x] Global shortcut + hide-to-tray (`Ctrl+Shift+Space` + close hides)
- [x] RPM spec template (`crates/gnomad-gtk/packaging/`) — no webkit2gtk

### Phase 2 — Chat & agent parity (3–4 weeks)
- [x] Session sidebar + composer + provider/model selects
- [x] Ollama model discovery on startup
- [x] Light/dark/system theme (`AdwStyleManager` follows system)
- [x] **Agent computer access** — terminal (`shell_run`), open/run programs (`xdg-open`, PATH), filesystem (`fs_*`), workspace info; safety + audit
- [x] Full agent loop (multi-step tool calls, max 10 steps) → `gnomad-core/agent/runtime`
- [x] Sudo/Path gate dialogs (GTK `AdwMessageDialog`)
- [x] Command result cards in chat UI
- [ ] VTE live terminal (PTY stream)
- [ ] Context pills + Linux integration settings

### Phase 3 — Knowledge, settings, polish (2–3 weeks)
- [ ] Settings panels (port from React spec)
- [ ] Knowledge library + skill packs
- [ ] Onboarding wizard
- [ ] Attachments
- [ ] Update checker

### Phase 4 — Release & migration (1 week)
- [ ] Data migration from Tauri app paths (chat JSON, knowledge, prefs)
- [ ] Desktop file, MIME, Nobara/Fedora CI
- [ ] Rename binary: `gnomad` (drop `omni-taskbar-ai` legacy)
- [ ] GitHub release artifacts: rpm, deb, AppImage

**Total estimate:** ~10–14 weeks focused work for feature parity MVP.

---

## 7. Data & config compatibility

| Data | Current path | Linux GTK path |
|------|--------------|----------------|
| Chat sessions | Tauri app data `gnomad/chat/` | `$XDG_DATA_HOME/gnomad/chat/` (same schema) |
| Knowledge | `gnomad/knowledge/` | Same |
| Agent settings | `agent-settings.json` | Same |
| Keychain | service `com.gnomadstudio.gnomad` | Same (Secret Service) |
| UI prefs | `localStorage` `omni_*` | `glib` GSettings or `~/.config/gnomad/prefs.json` — **migration tool reads localStorage export** |

---

## 8. Packaging (Linux-first)

```spec
# Fedora/Nobara RPM deps (no WebKit!)
gnomad-gtk
  depends: gtk4 >= 4.14, libadwaita >= 1.5, libvte-2.91-gtk4
  depends: libappindicator-gtk3 OR libayatana-appindicator3-1
  recommends: wl-clipboard, bubblewrap, ollama
```

AppImage and Flatpak follow same GTK4 base — **eliminates WebKitGTK dependency entirely** if chat uses VTE + markdown widgets.

---

## 9. macOS repo relationship

| Strategy | Description |
|----------|-------------|
| **Recommended** | Monorepo: `gnomad-core` shared; `gnomad-gtk` (Linux); keep Tauri app as `gnomad-mac` until macOS migrates to Swift or retains Tauri |
| **Not recommended** | Continue “Linux lane” patches inside Tauri `window_manager.rs` |

macOS build continues shipping from current Tauri tree; Linux ships `gnomad-gtk` when Phase 1 passes QA.

---

## 10. Immediate next steps

1. **Approve stack:** Rust + GTK4/Libadwaita + VTE (confirm or choose Qt6 if you prefer QML/KDE-native)
2. **Phase 0 kickoff:** extract `gnomad-core` crate; CI `cargo test` without Tauri
3. **Spike (2 days):** minimal `gnomad-gtk` panel window on your Nobara box — resize + mode switch must pass before any chat UI
4. **Freeze Tauri Linux fixes:** maintenance mode only; no more window_manager patches

---

## 11. Success criteria (Linux QA gate)

On **Nobara 43, KDE Plasma, Wayland, NVIDIA**:

- [ ] Panel opens from tray; survives 50 resize cycles without crash
- [ ] All four window modes switch without crash
- [ ] Ollama models populate in composer on cold start
- [ ] Agent shell command streams to VTE widget
- [ ] Sudo gate blocks `sudo` until approved
- [ ] Chat sessions persist across restart
- [ ] CPU/GPU: no ZINK/EGL error spam on launch (no WebKit shell)

---

## Appendix A — Rust modules to extract verbatim

`llm.rs`, `shell_session.rs`, `shell_sandbox.rs`, `privilege.rs`, `hitl_token.rs`, `path_token.rs`, `agent_settings.rs`, `agent_runtime.rs`, `agent_fs.rs`, `command_planner.rs`, `agent_audit.rs`, `chat_history.rs`, `attachments.rs`, `knowledge.rs`, `keychain.rs`, `env_config.rs`, `context.rs`, `linux_context.rs`, `error.rs`, `error_log.rs`, `local_inference.rs`, `automation.rs`, `updater.rs` (adapt)

## Appendix B — React components → GTK mapping

| React | GTK 4 |
|-------|-------|
| `AppToolbar` | `AdwHeaderBar` + `GtkToggle` mode group |
| `ChatSidebar` | `GtkPaned` + `GtkListBox` |
| `ChatView` | `GtkScrolledWindow` + message `ListBox` |
| `LiveTerminal` | `vte4::Terminal` |
| `SettingsPanel` | `AdwPreferencesWindow` |
| `OnboardingModal` | `AdwDialog` multi-page |
| `WindowModeBar` | `AdwViewSwitcher` or segmented `GtkToggle` |
| `CursorBloomBackground` | `GtkDrawingArea` (optional) |

---

*Document generated from macOS build audit of `gnomad-desktop-assistant` v0.2.0-beta.1.*
