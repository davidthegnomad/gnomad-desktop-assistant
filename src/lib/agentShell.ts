/** Machine-readable command blocks the model should emit (app executes these). */
const GNOMAD_RUN_RE =
  /<gnomad-run>([\s\S]*?)<\/gnomad-run>/gi;

const FENCED_SHELL_RE =
  /```(?:bash|sh|zsh|shell|console)\s*\n([\s\S]*?)```/gi;

const RUN_INTENT_RE =
  /\b(run|execute|exec|install|uninstall|brew\s+install|npm\s+i|npm\s+install|pip\s+install|cargo\s+install|terminal|shell|command line|cli|check if|verify|homebrew)\b/i;

const PASTE_COMMAND_RE =
  /^(?:run\s+)?[`']?([a-zA-Z0-9_./\\=\-\s|&><'"$]+)[`']?\s*$/i;

const INTERNAL_WRAPPER_RE =
  /__gnomad|__GNOMAD|printf\s+'%s|2>\/dev\/null\s*\|\|\s*true;\s*$/;

/** Mirrors Rust `is_probably_natural_language` in shell_session.rs */
function isProbablyNaturalLanguage(command: string): boolean {
  const lower = command.toLowerCase();
  const phrases = [
    "try to",
    "install it again",
    "please",
    "could you",
    "would you",
    "let me",
    "make sure",
    "check if",
    "is installed",
    "are installed",
    "not installed",
    "run the",
    "you can",
    "you should",
    "want to",
    "need to",
    "whether",
    "installed or not",
    "homebrew is",
  ];
  if (phrases.some((p) => lower.includes(p))) return true;
  if (lower.includes("__gnomad")) return true;

  const words = command.trim().split(/\s+/);
  if (words.length < 2) return false;

  const first = words[0].toLowerCase();
  if (
    ["try", "please", "could", "would", "check", "verify", "see", "find", "tell", "help", "ensure", "make"].includes(
      first
    )
  ) {
    const second = words[1].toLowerCase();
    if (["to", "if", "the", "that", "whether", "out", "it", "me", "sure"].includes(second)) {
      return true;
    }
  }

  if (
    words.length >= 4 &&
    !/[|&;$`<>=\\]/.test(command) &&
    !command.includes("/") &&
    !command.includes("-")
  ) {
    const alphaWords = words.filter((w) => /^[a-zA-Z]+$/.test(w)).length;
    if (alphaWords >= 4) return true;
  }

  return false;
}

function hasPlausibleCommandToken(command: string): boolean {
  const first = command.trim().split(/\s+/)[0] ?? "";
  if (!first || first.length > 128) return false;
  return /^[a-zA-Z0-9@$~./\\_-]+$/.test(first);
}

/** User message looks like a request to run something on the machine. */
export function userWantsShellExecution(message: string): boolean {
  const t = message.trim();
  if (!t) return false;
  if (RUN_INTENT_RE.test(t)) return true;
  if (t.startsWith("$ ")) return true;
  if (/^[`'].*[`']$/.test(t) && t.includes(" ")) return false;
  return false;
}

/** True only for strings that are safe to pass to eval in the PTY (aligned with Rust). */
export function isValidShellCommand(cmd: string): boolean {
  const t = cmd.trim();
  if (t.length < 1 || t.length > 2000) return false;
  if (t.includes("\n\n")) return false;
  if (INTERNAL_WRAPPER_RE.test(t)) return false;
  if (isProbablyNaturalLanguage(t)) return false;
  return hasPlausibleCommandToken(t);
}

/** User pasted a single shell command (optional leading "run"). */
export function extractUserShellCommand(message: string): string | null {
  const t = message.trim();
  if (t.startsWith("$ ")) {
    const cmd = t.slice(2).trim();
    return isValidShellCommand(cmd) ? cmd : null;
  }
  const m = t.match(PASTE_COMMAND_RE);
  if (m?.[1] && isValidShellCommand(m[1])) return m[1].trim();
  return null;
}

function normalizeCommand(raw: string): string | null {
  const lines = raw
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter((l) => l.length > 0 && !l.startsWith("#"));
  if (lines.length === 0) return null;
  const cmd = lines.join(" && ").trim();
  if (!isValidShellCommand(cmd)) return null;
  return cmd;
}

/** Tags the model wrote that are not valid shell (for user-visible skip messages). */
export function collectInvalidGnomadRunTags(reply: string): string[] {
  const invalid: string[] = [];
  let m: RegExpExecArray | null;
  GNOMAD_RUN_RE.lastIndex = 0;
  while ((m = GNOMAD_RUN_RE.exec(reply)) !== null) {
    const raw = m[1].trim();
    if (raw && !isValidShellCommand(raw)) {
      invalid.push(raw);
    }
  }
  return invalid;
}

/** Pull commands from an assistant reply (tags + fenced blocks when run was requested). */
export function extractCommandsFromAssistantReply(
  reply: string,
  runRequested: boolean
): string[] {
  const found: string[] = [];
  const seen = new Set<string>();

  const add = (raw: string) => {
    const cmd = normalizeCommand(raw);
    if (!cmd || seen.has(cmd)) return;
    seen.add(cmd);
    found.push(cmd);
  };

  let m: RegExpExecArray | null;
  GNOMAD_RUN_RE.lastIndex = 0;
  while ((m = GNOMAD_RUN_RE.exec(reply)) !== null) {
    add(m[1]);
  }

  if (runRequested || found.length > 0) {
    FENCED_SHELL_RE.lastIndex = 0;
    while ((m = FENCED_SHELL_RE.exec(reply)) !== null) {
      add(m[1]);
    }
  }

  return found.slice(0, 5);
}

/** Intro text without executable blocks (shown above command output cards). */
export function stripExecutableBlocksFromReply(reply: string): string {
  let text = reply
    .replace(GNOMAD_RUN_RE, "")
    .replace(FENCED_SHELL_RE, "")
    .trim();
  text = text.replace(/\n{3,}/g, "\n\n");
  return text;
}

export const AGENT_SHELL_SYSTEM_APPEND = `
## Terminal execution (important)
You have a real shell on the user's machine via the app — not a simulated terminal.
When you need to run a command:
1. Put ONLY a valid one-line shell command inside: <gnomad-run>command -v brew</gnomad-run>
2. NEVER put English inside the tag (wrong: <gnomad-run>check if brew is installed</gnomad-run>).
3. Use concrete CLI: command -v brew, brew --version, brew install foo, npm install, cd path, etc.
4. Do NOT use markdown code fences for commands you want executed.
5. Do NOT claim success until the app returns state, exit_code, and terminal output.
6. If state is stalled, the command may need a password — ask the user.
Examples:
- Check Homebrew: <gnomad-run>command -v brew</gnomad-run>
- Install package: <gnomad-run>brew install wget</gnomad-run>
`.trim();
