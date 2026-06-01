# UI Design — Gemini-inspired

**Reference research:** May 2026 Google Gemini app redesign (“Neural Expressive”).

## Visual references (web)

- [9to5Google — full redesign](https://9to5google.com/2026/05/03/gemini-full-redesign/) — pill prompt, centered greeting, gradient mesh, model picker top-left
- [The Verge — Neural Expressive](https://www.theverge.com/tech/933699/google-gemini-redesign-ai-3-5-flash-io-2026) — “Ask Gemini” bar, bottom blue gradient
- [PhoneWorld first look](https://www.phoneworld.com.pk/google-gemini-app-full-redesign-ui-overhaul-2026/) — comparison table vs old UI

## Patterns adopted in Omni

| Gemini pattern | Omni implementation |
|----------------|---------------------|
| Pill-shaped prompt | `.pill-prompt` + `.pill-input` footer bar |
| Centered greeting | `.chat-empty-state` — “What’s on your mind?” |
| Subtle color gradient | `.gemini-gradient-bg` mesh (blue / purple / coral) |
| Model picker top-left | `.model-picker` dual dropdown (provider + model) |
| Dark ≈ pure black | `[data-theme="dark"]` → `#0a0a0a` base |
| Light soft surfaces | `[data-theme="light"]` → `#f8f9fc` + bottom blue wash |
| Thin outline icons | Lucide, stroke 1.5 on spark icons |
| Theme toggle | Header cycles light → dark → system |

## First-run onboarding

Shown when **no** cloud API key **and** no Ollama URL in keychain, and `omni_onboarding_complete` is not set.

1. Choose **Cloud API** or **Local model**
2. Enter API key + model, or Ollama URL + model name
3. Saves to keychain + `localStorage`; can re-open from Settings

## Theme storage

- `localStorage.omni_theme`: `light` | `dark` | `system`
- Inline script in `index.html` prevents flash of wrong theme on load
