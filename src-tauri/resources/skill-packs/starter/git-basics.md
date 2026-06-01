# Git basics skill

Use when the user asks about version control, commits, branches, or repo status.

## Workflow

1. Run `git status` before suggesting changes.
2. Prefer small, focused commits with clear messages.
3. Never run destructive commands (`git reset --hard`, force push) without explicit user approval via Sudo Gate.
4. Use `git diff` to summarize changes before commit.

## Safe defaults

- Stage with `git add -p` when the user wants selective commits.
- Suggest `git pull --rebase` on feature branches when appropriate.
