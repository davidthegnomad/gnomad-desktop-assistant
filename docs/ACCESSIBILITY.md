# Accessibility — Gnomad Desktop Assistant

**Status:** Alpha improvements (June 2026) — not a full WCAG 2.2 audit  
**Target:** v1.0 general availability pass

---

## Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| **⌘⇧Space** / **Ctrl+Shift+Space** | Show/hide Gnomad (global, OS-level) |
| **Escape** | Close topmost overlay (gate modal → about → onboarding → settings → knowledge) |
| **⌘K** / **Ctrl+K** or **⌘L** / **Ctrl+L** | Focus message composer |
| **⌘,** / **Ctrl+,** | Open Settings |
| **⌘N** / **Ctrl+N** | New chat |

Shortcuts are disabled while typing in inputs unless combined with ⌘/Ctrl. Gate modals (Sudo / Path) only handle **Escape**.

---

## Implemented (alpha)

- **Focus visible** — `:focus-visible` ring on interactive controls
- **Composer** — `aria-label="Message Gnomad"`, stable `#gnomad-composer-input` id
- **Modals** — `role="dialog"`, `aria-modal="true"`, focus trap on Sudo Gate, Path Gate, About, **Onboarding**
- **Skip link** — windowed/fullscreen: “Skip to main content” jumps to `#gnomad-main-content`
- **Terminal** — debounced `aria-live` summaries for xterm output (full stream not read line-by-line)
- **Knowledge tabs** — `role="tablist"` / `role="tab"` / `aria-selected`
- **Existing labels** — provider/model selects, send button, tray actions (see component `aria-label` attributes)

---

## Known gaps (v1.0)

- Full WCAG 2.2 AA contrast audit not complete
- Screen reader testing limited (VoiceOver / NVDA / Orca)
- Live terminal: debounced summaries only (not full stream narration)

---

## Testing checklist

```
[ ] Tab through Settings without mouse
[ ] Escape closes settings and knowledge overlay
[ ] ⌘K focuses composer from chat view
[ ] Sudo Gate: Tab cycles Deny / Approve; Escape denies
[ ] Focus ring visible on buttons and tabs (keyboard only)
[ ] Onboarding: Tab through provider cards and form fields
[ ] Windowed mode: Tab to “Skip to main content” → main chat
```

---

## Related

- [USER_GUIDE.md](USER_GUIDE.md)
- [ROADMAP.md](ROADMAP.md) — v1.0 WCAG pass
- [CROSS_PLATFORM_CHECKLIST.md](CROSS_PLATFORM_CHECKLIST.md)

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
