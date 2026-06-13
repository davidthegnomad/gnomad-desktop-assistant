# gnomad-gtk

Linux-native Gnomad shell (GTK 4 + Libadwaita). Cargo package `gnomad-gtk`; shipping binary **`gnomad`**.

## Build (Nobara/Fedora)

```bash
sudo dnf install gtk4-devel libadwaita-devel
# Optional: Pop Out stay-above on X11
sudo dnf install wmctrl
# Optional: embedded terminal (Shell mode)
sudo dnf install vte291-gtk4-devel
cargo build -p gnomad-gtk --release
```

## Run

Build output is at the **workspace root** (not `src-tauri/target/`):

```bash
# from repo root
./target/release/gnomad

# or from anywhere in the repo
npm run gtk:run
```

### Desktop doctor

Check session type, desktop environment, tray host, optional tools, and live context probes:

```bash
./target/release/gnomad --doctor
# or: npm run gtk:doctor
```

Exit code `0` = no failures (warnings OK); `1` = fix failed checks before expecting full functionality.

### Data migration (Tauri → GTK)

Chat, knowledge, agent settings, and keychain already live under `~/.local/share/com.gnomadstudio.gnomad/gnomad/` for both apps. To audit or copy legacy trees and import UI prefs from a Tauri localStorage export:

```bash
./target/release/gnomad --migrate-audit
./target/release/gnomad --migrate
./target/release/gnomad --migrate --import-prefs /path/to/localStorage-export.json
# or: npm run gtk:migrate
```

Export format (DevTools → Application → Local Storage, or manual JSON):

```json
{"omni_onboarding_complete":"true","omni_provider":"local","omni_local_model":"llama3.2","omni_theme":"dark"}
```

Theme is stored in `ui-prefs.json` as `theme`: `system` | `light` | `dark` and applied at startup via Libadwaita.

### KDE smoke + packaging

```bash
npm run gtk:smoke:kde       # preflight + tray registration (graphical session)
npm run gtk:package:linux   # .deb + AppImage (Ubuntu 24.04+ recommended)
npm run gtk:package:rpm     # Fedora/Nobara RPM when rpmbuild is installed
npm run gtk:release:smoke   # build packages + extract .deb + run --doctor
```

**First GTK release tag** (after local smoke passes):

```bash
npm run gtk:release:smoke
git tag v0.2.0-gtk
git push origin v0.2.0-gtk
```

CI `release.yml` attaches `gnomad_0.2.0_*.deb` and `gnomad_0.2.0_*.AppImage` to the GitHub Release.

CI: `.github/workflows/gtk-build.yml` builds deb + AppImage on `ubuntu-24.04` and `ubuntu-24.04-arm`.

- **Tray:** StatusNotifierItem (ksni) — **left-click** opens Window mode; **right-click** opens the menu.
- **Shortcut:** `Ctrl+Shift+Space` toggles Window mode (global-hotkey on X11/XWayland; GTK accel when focused on pure Wayland).
- **Modes:** Panel 600×920, Pop out / Window 1283×858, Fullscreen — native GTK policies.
- **Help:** `?` icon in the sidebar footer; bundled `resources/GNOMAD_HELP.md` is injected into agent context.
- **Onboarding:** First-run wizard for cloud API or Ollama; prefs in `~/.config/com.gnomadstudio.gnomad/ui-prefs.json`.

## Desktop environment notes

| Topic | KDE Plasma | GNOME | COSMIC |
|-------|------------|-------|--------|
| Tray | Native SNI | Needs AppIndicator (or equivalent) | Enable status-area applet |
| Left-click tray | Opens window | Usually opens window | Usually opens window (verify cosmic-applets version) |
| Pop Out above | Best on XWayland + `wmctrl` | May not stay above on pure Wayland | Verify on your build |
| Global shortcut | X11 global; Wayland when focused | Same | Same |
| Context pills | KDE Wayland title probe | GNOME Shell Eval (may be restricted) | XWayland xdotool fallback |

## QA matrix

**Automated gate (any DE):** `npm run gtk:preflight` — core tests, release build, `--doctor`.

**KDE Wayland (primary):** doctor verified 12 pass / 1 warn / 0 fail on Nobara 43 (SNI, qdbus+KWin, wl-paste, bwrap, wmctrl, ollama, live context). Remaining items need interactive GUI smoke.

**GNOME / COSMIC:** deferred until hardware access — run the same doctor + manual matrix when available.

| # | Check | KDE | GNOME | COSMIC |
|---|-------|-----|-------|--------|
| 1 | Tray icon appears | doctor ✓ SNI | AppIndicator required | status-area applet |
| 2 | Left click → Windowed + focus | manual | manual | manual |
| 3 | Right click → menu; all items work | manual | manual | retest menu clicks |
| 4 | Pop Out stays above a normal window | doctor ✓ wmctrl | may fail Wayland | verify |
| 5 | Settings from tray opens dialog | manual | manual | manual |
| 6 | Gnomad Studio opens browser | manual | manual | manual |
| 7 | Ctrl+Shift+Space unfocused | doctor ! Wayland | same | same |
| 8 | Ctrl+Shift+Space focused | manual | manual | manual |
| 9 | Help `?` dialog; agent uses injected help | manual | manual | manual |
| 10 | Composer click — no GPU crash | doctor ✓ cairo | same | same |
| 11 | Context pill window title | doctor ✓ live | GNOME may Unknown | COSMIC may Unknown |
| 12 | App usable if tray spawn fails | `.desktop` / shortcut | same | same |

### KDE Wayland interactive smoke

1. Left-click tray → Window mode appears, composer clickable.
2. Right-click tray → menu items work; Quit exits.
3. Pop Out → floating window; stays above when XWayland + wmctrl available.
4. Ctrl+Shift+Space toggles Window mode (focused on Wayland).
5. `?` help dialog opens; chat can answer “what does Gnomad need?”
