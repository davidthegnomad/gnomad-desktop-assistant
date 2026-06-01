════════════════════════════════════════════════════════════════════════
  GNOMAD DESKTOP ASSISTANT
  Gnomad Desktop Assistant · README.md
════════════════════════════════════════════════════════════════════════

GNOMAD DESKTOP ASSISTANT
========================

Alpha
Version
Platforms

Gnomad is a cross-platform desktop AI assistant that integrates with the operating system—system tray, global shortcut, live window and clipboard context—and executes real shell and filesystem work under explicit user approval, not simulated chat output.

Built with ❤️ by Gnomad Studio 🦙

Live project site: davidthegnomad.github.io/gnomad-desktop-assistant

────────────────────────────────────────

WHY THIS EXISTS
---------------

Desktop knowledge work is fragmented: chat in a browser tab, terminal in another, files elsewhere. Gnomad unifies conversation, OS awareness, and action in a native shell that stays out of the way until invoked—then anchors next to the menu bar or tray with a focused, Gemini-inspired interface.

The alpha demonstrates end-to-end delivery: multi-platform installers, CI/CD, credential hygiene, agent tooling with human-in-the-loop gates, and documentation suitable for technical review.

────────────────────────────────────────

CAPABILITIES (V0.1.0-ALPHA + MAIN)
----------------------------------

  Area          |  What you get                                                                                            
  Access        |  Menu bar / system tray, global shortcut, four window modes (panel, pop-out, windowed, fullscreen)       
  Intelligence  |  DeepSeek (cloud), Ollama (local), optional in-process GGUF (embedded-llm build)                         
  Agent         |  Multi-step tool loop (shell + filesystem), command planner, persistent PTY shell                        
  Safety        |  Cryptographic Sudo Gate (HITL) and Path Gate tokens; Standard vs YOLO trust; optional YOLO shell sandbox
  Terminal      |  xterm.js live stream + replay on command cards                                                          
  Context       |  Active application, window title, clipboard snippet in the footer                                       
  Memory        |  Chat history on disk; knowledge library (skills, agents, uploads)                                       
  Updates       |  In-app check (stable/beta) via Tauri updater — see docs/UPDATER.md                                      
  Platform      |  macOS (primary), Windows, Linux (.deb, .rpm, AppImage); Wayland tray improvements                       

────────────────────────────────────────

PLATFORMS
---------

  Platform  |  Status     |  Install                 |  Notes                                
  macOS     |  Primary    |  .dmg                    |  Menu-bar accessory; overlay title bar
  Windows   |  Supported  |  .msi                    |  System tray; Ctrl+Shift+Space        
  Linux     |  Supported  |  .deb · .rpm · AppImage  |  See Linux packages                   

Per-OS build instructions: docs/BUILD_PLATFORMS.md  
Adding features or UI? Use the docs/CROSS_PLATFORM_CHECKLIST.md so changes are verified on macOS, Windows, and Linux.

────────────────────────────────────────

QUICK START
-----------

End users

  1. Download the installer for your OS from Releases or the project site.
  2. Launch Gnomad; complete the setup wizard (cloud API key or local Ollama URL).
  3. Press ⌘⇧Space (macOS) or Ctrl+Shift+Space (Windows/Linux) to show the assistant.
  4. Read the full manual: docs/USER_GUIDE.html (recommended) or docs/USER_GUIDE.txt.

Developers

Prerequisites: Node.js LTS, Rust stable, Tauri v2 platform deps

  [bash]
    git clone https://github.com/davidthegnomad/gnomad-desktop-assistant.git
    cd gnomad-desktop-assistant
    npm ci
    cp .env.example .env   # optional: DeepSeek_API_KEY for local dev
    npm run tauri dev
    npm run test              # Vitest (error parsing)
    cd src-tauri && cargo test

Embedded GGUF (optional, in-process local LLM):

  [bash]
    npm run tauri:dev:embedded

Production build:

  [bash]
    npm run tauri:build:mac     # macOS
    npm run tauri:build:linux   # Linux (on Linux host)
    npm run tauri:build:win     # Windows (on Windows host)

Never commit .env or API keys.

────────────────────────────────────────

ARCHITECTURE (SUMMARY)
----------------------

  [code]
    ┌─────────────────────────────────────────────────────────┐
    │  React 19 UI (chat, settings, knowledge, gates)         │
    └───────────────────────────┬─────────────────────────────┘
                                │ Tauri IPC (invoke / events)
    ┌───────────────────────────▼─────────────────────────────┐
    │  Rust: tray, window modes, shell PTY, agent FS,         │
    │        keychain, context, safety, audit, chat store      │
    └───────────────────────────┬─────────────────────────────┘
                                │
             macOS / Windows / Linux native APIs

Deep dive: docs/TECH_STACK.md · Delivery narrative: docs/BUILD.md

────────────────────────────────────────

DOCUMENTATION INDEX
-------------------

All docs ship as Markdown (source), HTML (browser), and TXT (Notepad/Word). Full index: docs/DOCS_INDEX.md. Regenerate: npm run docs:export.

  Document          |  MD                       |  HTML                       |  TXT                     
  All docs (index)  |  docs/DOCS_INDEX.md       |  docs/DOCS_INDEX.html       |  docs/DOCS_INDEX.txt     
  Project site      |  —                        |  docs/index.html            |  —                       
  User Guide        |  docs/USER_GUIDE.md       |  docs/USER_GUIDE.html       |  docs/USER_GUIDE.txt     
  Tech Stack        |  docs/TECH_STACK.md       |  docs/TECH_STACK.html       |  docs/TECH_STACK.txt     
  Build Narrative   |  docs/BUILD.md            |  docs/BUILD.html            |  docs/BUILD.txt          
  Architecture      |  docs/ARCHITECTURE.md     |  docs/ARCHITECTURE.html     |  docs/ARCHITECTURE.txt   
  Security Model    |  docs/SECURITY_MODEL.md   |  docs/SECURITY_MODEL.html   |  docs/SECURITY_MODEL.txt 
  Wave B Roadmap    |  docs/WAVE_B_ROADMAP.md   |  docs/WAVE_B_ROADMAP.html   |  docs/WAVE_B_ROADMAP.txt 
  Auto-updater      |  docs/UPDATER.md          |  docs/UPDATER.html          |  docs/UPDATER.txt        
  Privacy           |  docs/PRIVACY.md          |  docs/PRIVACY.html          |  docs/PRIVACY.txt        
  Roadmap           |  docs/ROADMAP.md          |  docs/ROADMAP.html          |  docs/ROADMAP.txt        
  Demo Script       |  docs/DEMO_SCRIPT.md      |  docs/DEMO_SCRIPT.html      |  docs/DEMO_SCRIPT.txt    
  QA Checklists     |  docs/QA_CHECKLIST.md     |  docs/QA_CHECKLIST.html     |  docs/QA_CHECKLIST.txt   
  Build Platforms   |  docs/BUILD_PLATFORMS.md  |  docs/BUILD_PLATFORMS.html  |  docs/BUILD_PLATFORMS.txt
  Changelog         |  CHANGELOG.md             |  CHANGELOG.html             |  CHANGELOG.txt           

────────────────────────────────────────

SECURITY & PRIVACY (ALPHA)
--------------------------

  • API keys are stored in the OS keychain, not in chat logs or localStorage.
  • Destructive or privileged shell operations require Sudo Gate approval with signed HITL tokens (unsigned IPC bypass rejected).
  • Filesystem access outside the workspace requires Path Gate approval with signed path tokens (Standard mode).
  • Optional YOLO shell sandbox (macOS/Linux experimental) limits network and writes when enabled.
  • Agent actions are appended to a local audit log under application data.

Alpha software: review CHANGELOG.md for known limitations before production use.

────────────────────────────────────────

CI / RELEASES
-------------

GitHub Actions builds macOS, Linux, and Windows on every push to main / master. Tagged v* releases attach installers to GitHub Releases.

────────────────────────────────────────

LICENSE
-------

Private project — see repository settings. Contact Gnomad Studio for licensing inquiries.

────────────────────────────────────────

ACKNOWLEDGMENTS
---------------

UI patterns informed by Google Gemini’s 2026 “Neural Expressive” redesign; implementation is original to Gnomad Studio. Built with Tauri, React, and Rust.

════════════════════════════════════════════════════════════════════════
Built with ❤️ by Gnomad Studio 🦙
https://gnomadstudio.org
════════════════════════════════════════════════════════════════════════
