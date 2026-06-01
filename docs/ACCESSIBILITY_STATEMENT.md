# Accessibility Statement — Gnomad Desktop Assistant

**Version:** 0.2.0-beta.1  
**Last updated:** June 2026  
**Standard:** Working toward [WCAG 2.2](https://www.w3.org/TR/WCAG22/) Level AA

---

## Commitment

Gnomad Studio aims to make Gnomad Desktop Assistant usable with keyboard and assistive technologies. This is a **beta** product; a full third-party WCAG audit is planned before v1.0 general availability.

---

## Conformance status

**Partially conformant** — some content does not yet meet WCAG 2.2 AA (contrast audit incomplete; terminal output summarized via `aria-live` rather than full narration).

---

## Supported features (beta)

- Keyboard shortcuts for show/hide, composer focus, settings, new chat, and Escape to close overlays
- Visible focus indicators (`:focus-visible`) on interactive controls
- Modal dialogs with focus traps and `role="dialog"`
- Skip link to main content in windowed and fullscreen modes
- Labeled form controls and composer (`aria-label`, stable ids)
- Debounced `aria-live` announcements for terminal activity

See [ACCESSIBILITY.md](ACCESSIBILITY.md) for the full shortcut list and testing checklist.

---

## Known limitations

- Color contrast not yet certified to WCAG AA across all themes
- Limited testing with VoiceOver, NVDA, and Orca
- xterm.js canvas is not directly readable; summaries are announced periodically
- Voice dictation depends on OS/browser speech services

---

## Feedback

Report accessibility barriers:

- [GitHub Issues](https://github.com/davidthegnomad/gnomad-desktop-assistant/issues) (label: `accessibility`)
- [Gnomad Studio](https://gnomadstudio.org)

We aim to respond within 10 business days.

---

## Related

- [ACCESSIBILITY.md](ACCESSIBILITY.md) — developer checklist
- [USER_GUIDE.md](USER_GUIDE.md)

---

Built with ❤️ by [Gnomad Studio](https://gnomadstudio.org) 🦙
