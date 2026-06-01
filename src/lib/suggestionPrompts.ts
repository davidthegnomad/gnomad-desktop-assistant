/** Full pool of welcome suggestion chips — a random subset is shown each time. */
export const ALL_SUGGESTION_PROMPTS: readonly string[] = [
  "Summarize what I'm working on",
  "Help me automate a small task",
  "Explain this in simpler terms",
  "Draft a quick reply from my clipboard",
  "What's my active window about?",
  "Suggest a shell command for this job",
  "Turn this note into action items",
  "Plan my next three steps",
  "Debug why my script failed",
  "Write a one-liner for this repetitive task",
  "Compare these two approaches",
  "Polish this message before I send it",
  "Extract todos from this paragraph",
  "Explain this error message in plain English",
  "Brainstorm names for my new project",
  "What should I focus on right now?",
  "Help me learn this concept fast",
  "Organize my messy folder structure",
  "Walk me through a tool I've never used",
  "Start an adventure from my clipboard",
];

const DEFAULT_CHIP_COUNT = 3;

/** Pick `count` unique prompts at random from the pool. */
export function pickRandomSuggestions(
  count = DEFAULT_CHIP_COUNT
): string[] {
  const pool = [...ALL_SUGGESTION_PROMPTS];
  for (let i = pool.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [pool[i], pool[j]] = [pool[j], pool[i]];
  }
  return pool.slice(0, Math.min(count, pool.length));
}
