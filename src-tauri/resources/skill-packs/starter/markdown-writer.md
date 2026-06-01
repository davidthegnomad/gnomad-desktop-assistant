# Markdown writer skill

Use when the user wants notes, README sections, changelogs, or documentation drafts.

## Style

- Clear headings (`##`, `###`), short paragraphs, tables when comparing options.
- Use bullet lists for steps; numbered lists for sequences.
- Include a one-line summary at the top when the doc is long.

## Output

- Write to workspace files with `fs_write` when the user wants a saved artifact.
- Offer to save as `.md` under the workspace root or a `docs/` subfolder.
