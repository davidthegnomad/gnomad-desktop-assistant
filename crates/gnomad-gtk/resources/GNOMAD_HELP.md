# Gnomad quick help

Gnomad is your Linux-native AI desktop assistant. This guide covers setup, capabilities, and desktop quirks.

## What Gnomad needs

On first launch, Gnomad shows a short setup wizard (cloud API or Ollama). To see it again, delete `~/.config/com.gnomadstudio.gnomad/ui-prefs.json` and restart.

- **Local models:** Run `ollama serve`, then pick **Local** in the composer.
- **Cloud models:** Add an API key in **Settings** (gear icon) and choose **Cloud**.
- **Agent access:** Turn on **Access local files** to let Gnomad read/write files and run shell commands (with your approval).
- **Optional:** Configure agent secrets and sudo policy in Settings.

## Capabilities

- Chat with local or cloud LLMs
- Agent tools: `shell_run`, file read/write, knowledge library
- Built-in terminal (Shell mode) for command output
- Trust modes and approval gates for shell and path access
- Context pills: active window title and clipboard preview

## How to ask Gnomad

- "Summarize this clipboard text and suggest next steps."
- "List files in my workspace and explain the project structure."
- "Run `git status` and tell me what changed."
- Add skills and preferences in **Settings → Knowledge**.

## Tray and window modes

| Action | Result |
|--------|--------|
| **Tray left-click** | Opens the standard **Window** mode and focuses it |
| **Pop Out** | Floating window; tries to stay above other windows |
| **Hide** | Hides the window; app keeps running in the tray |
| **Quit** | Exits Gnomad completely |
| **Ctrl+Shift+Space** | Same as tray left-click (toggle Window mode) |

Mode chips in the header: **Panel**, **Pop out**, **Window**, **Full**.

## Desktop environments (GNOME, KDE, COSMIC)

- **Tray icon:** Uses the StatusNotifier (SNI) protocol. On **GNOME**, you may need an AppIndicator extension for the tray to appear. On **COSMIC**, enable the status-area applet and keep `cosmic-applets` updated.
- **Tray left-click vs menu:** Left-click opens the window; right-click opens the menu. If your desktop opens the menu on left-click, check for an outdated tray host.
- **Pop Out stay-above:** Works best on **X11/XWayland** with `wmctrl`. On **pure Wayland**, compositors may not keep Pop Out above all windows — this is a platform limitation, not a Gnomad bug.
- **Global shortcut:** `Ctrl+Shift+Space` is captured globally on X11. On pure Wayland it usually works only when Gnomad is focused (GTK accelerator fallback).
- **Context pills:** Active window titles are reliable on KDE Wayland. GNOME and COSMIC Wayland may show "Unknown" until your compositor exposes focus info.

## Environment check

Run the desktop doctor before reporting tray or DE issues:

```bash
gnomad --doctor
```

It reports session type, desktop environment, tray host, optional tools, and a live context sample.

## Links

- Website: https://gnomadstudio.org
- Full user guide: see `docs/USER_GUIDE.md` in the Gnomad repository
