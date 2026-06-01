# macOS Permissions — Omni Taskbar AI

Features that touch the OS may prompt for or require grants in **System Settings**.

| Feature | Permission / capability | How the app uses it |
|---------|-------------------------|---------------------|
| Active window title | **Automation** (System Events) | `osascript` in `get_active_window` |
| Clipboard | Usually none | `pbpaste` |
| Simulate click / type | **Accessibility** | Enigo + AppleScript fallback |
| Global shortcut | None extra | Tauri global-shortcut plugin |
| Screen capture | **Screen Recording** (may apply) | `screencapture` in `capture_screen` |
| Elevated shell | **Administrator** (password / Touch ID) | `osascript` “with administrator privileges” |

## In-app checks

- `check_accessibility_permissions` — `AXIsProcessTrusted()`
- `request_accessibility_permissions` — opens Privacy → Accessibility

The context bar shows **Missing Permissions** when Accessibility is not granted.

## Developer tip

After changing bundle ID or binary path, macOS may reset grants. Re-enable the app under Privacy settings and restart `tauri dev`.
