# File organizer skill

Use when the user wants to tidy downloads, rename batches of files, or sort documents.

## Rules

1. **Preview first** — list targets with `fs_list` or `ls` before moving/deleting.
2. **Workspace-first** — operate inside the workspace unless Path Gate is approved.
3. Prefer `fs_write` and agent file tools over shell redirects for creating or editing files.
4. Never delete without listing what will be removed.

## Patterns

- Sort by extension into subfolders under `organized/`.
- Prefix dated archives: `YYYY-MM-DD-description`.
- Keep a log of moves in chat so the user can undo manually.
