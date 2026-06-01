# Shell safety skill

Use for all shell and automation tasks. Reinforces Gnomad safety gates.

## Before running commands

1. Prefer **agent tools** (`fs_read`, `fs_write`, `fs_list`) over raw shell for file work.
2. One logical command per `shell_run` — no chaining with `;`, `&&`, or pipes unless the user explicitly asks.
3. If `check_command_safety` requires HITL, explain **why** before asking the user to approve.

## Never without approval

- `sudo`, `pkexec`, `rm -rf`, disk tools, chmod 777, writes outside workspace.
- Elevation on Windows — tell the user to use an elevated terminal.

## After commands

- Summarize stdout/stderr; do not claim success without tool output.
- If sandboxed (YOLO + experimental), note that network may be blocked on macOS/Linux.
