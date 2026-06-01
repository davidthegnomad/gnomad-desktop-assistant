# Privacy Policy — Gnomad Desktop Assistant (Alpha)

**Effective:** June 2026  
**Applies to:** Gnomad Desktop Assistant v0.1.0-alpha and subsequent alpha releases until replaced  
**Operator:** Gnomad Studio — https://gnomadstudio.org

This is a **product privacy overview** for the alpha desktop app. It is not legal advice. A formal policy may replace this at general availability.

---

## Summary

- **Your chats and knowledge files stay on your device** by default.  
- **Cloud mode** sends prompts to **DeepSeek** under their API terms.  
- **Local mode** sends prompts only to **your Ollama server** (often localhost).  
- **Gnomad Studio does not operate a chat backend** that stores your conversations in the alpha release.  
- **API keys** are stored in your **operating system keychain**, not in our servers.

---

## Data stored locally

| Data type | Location | Purpose |
|-----------|----------|---------|
| Chat sessions | Application data / `gnomad/chats/` | History across restarts |
| Knowledge files | `gnomad/knowledge/` | Skills, agents, uploads |
| Agent settings | `gnomad/agent-settings.json` | Workspace, trust mode |
| Audit log | `gnomad/agent-audit.jsonl` | Record of agent actions |
| Preferences | `localStorage` in app | Theme, sidebar layout (non-secret) |

Alpha builds do **not** encrypt this data at rest. Protect your device with disk encryption and OS account security.

---

## Data sent to third parties

### DeepSeek (cloud mode)

When you use **Cloud** provider, message content—including optional attachments, context pills (app name, window title, clipboard snippet), and knowledge excerpts—is transmitted to **DeepSeek’s API** to generate responses.

- Review DeepSeek’s privacy policy and data processing terms before use.  
- Do not send regulated data (HIPAA, PCI, etc.) without organizational approval.

### Ollama (local mode)

When you use **Local** provider, data is sent to the **URL you configure** (default `http://localhost:11434`). If that server is on your machine, content does not leave your hardware except to that process. If you point to a remote Ollama instance, that operator receives your prompts.

### Gnomad Studio website

Opening **gnomadstudio.org** from the app uses normal HTTPS browsing. No chat content is transmitted by that action alone.

---

## Data we do not collect (alpha)

- No Gnomad account or login  
- No centralized chat sync to Gnomad Studio servers  
- No analytics SDK in the alpha build (subject to change with opt-in telemetry in beta)  
- No sale of personal data  

---

## Clipboard and window context

Gnomad reads the **frontmost application**, **window title**, and **clipboard text** locally to display context pills and optionally include them in prompts you send. This data is **not** uploaded except as part of a prompt you explicitly send to your chosen model provider.

---

## API keys

Keys are stored using:

- macOS Keychain  
- Windows Credential Manager  
- Linux Secret Service  

They are not displayed again in the UI after save. Developers may use a local `.env` file; never commit `.env` to version control.

---

## Your choices

| Action | How |
|--------|-----|
| Stop cloud transmission | Switch to Local provider or quit app |
| Delete chat history | Delete sessions in sidebar |
| Remove knowledge files | Knowledge panel → remove files |
| Revoke API key | Settings → API keys → Remove |
| Uninstall | Remove app; optionally delete app data folder |

---

## Children

Gnomad is not directed at children under 13. Do not use the app in contexts requiring parental consent without guardian oversight.

---

## Changes

Alpha policies may change. Material updates will be noted in `CHANGELOG.md` and this document’s date.

**Contact:** via https://gnomadstudio.org

---

Built with ❤️ by Gnomad Studio 🦙
