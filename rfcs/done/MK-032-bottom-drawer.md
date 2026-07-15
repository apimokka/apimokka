# RFC MK-032 — Bottom drawer (validation + save diff)

**Status.** Implemented (v0.6.0)
**Tracks.** D-01 Validation drawer, D-02 Save-diff drawer.
**Touches.** Secondary detail that supplements the workspace shell.
**Supersedes.** MK-011 (validation/save/runtime workflow), drawer portion of MK-001.

## Summary

The bottom drawer is a sliding sheet anchored to the bottom of the workspace shell. It provides secondary detail without losing the user's current context. Two modes ship in v1: **Validation** and **Save diff**. Both are reachable from the top-bar status chips (clicking `3 issues` opens validation; clicking the unsaved chip opens save diff).

## Drawer behaviour

| Trigger | Opens mode |
|---|---|
| Click validation chip in top bar | Validation |
| Click unsaved chip in top bar | Save diff |
| Save-all action while validation issues exist | Validation (forces a look before save) |
| Command palette: "Open validation drawer" | Validation |
| Command palette: "Open save diff" | Save diff |

The drawer takes 30–40% of window height. It does not block interaction with the body above it (the user can still click into the rule editor while the drawer is open).

`Esc` closes the drawer. Outside-click — on the workspace body — does not close it (the body is still interactive). The user dismisses explicitly via a close button in the drawer header, or by pressing the trigger chip again.

Snora implementation: the drawer maps to snora's bottom sheet primitive. Only one of the two modes is active at a time; switching modes closes the current one and opens the new one without animation flicker.

## Validation mode

```
Validation                              [✕]
Errors 1 · Warnings 2

checkout.toml
✕ Rule 2: URL operator requires URL path
⚠ Rule 4: Header value ignored for Exists operator

auth.toml
⚠ Rule 1: TLS cert path is empty

[Jump to first issue]
```

### Layout

- Header: section title + dismiss button on the right
- Counts row: `Errors N · Warnings N · Info N` (omit zero counts)
- Grouped by file: file name as a `bodyStrong` header, then issue rows under it
- Each issue row: severity glyph + label + jump action

### Issue row

| Field | Required |
|---|---|
| Severity glyph + accessible label | Yes |
| Location text | Yes (e.g. "Rule 2", "Header `content-type`") |
| Message | Yes — concise, explanatory |
| Jump action | Yes — clicking the row navigates to the offending element |

### Jump-to action

Clicking an issue row:
- Switches to Routes tab
- Selects the relevant rule set + rule
- Scrolls the rule editor to the offending field
- Focuses the field if applicable

### Empty state
```
No validation issues.
```

## Save-diff mode

```
Save diff                               [✕]
3 files will be written

Modified  checkout.toml         [View diff]
Created   edge-cases.toml       [View diff]
Modified  apimock.toml          [View diff]

[Discard] [Save all]
```

### Layout

- Header: section title + dismiss button
- Counts row: "N files will be written"
- File rows: change kind (`Modified` / `Created` / `Removed`) + file path + per-file `[View diff]` action
- Footer: `[Discard]` (ghost) and `[Save all]` (primary)

### Per-file diff view

Clicking `[View diff]` on a row replaces the file list with a diff view for that file (still inside the drawer). The diff is line-by-line with `+` / `-` markers; colour is supplementary to the markers (ABDD).

A back arrow at the top of the diff view returns to the file list.

### Empty state
```
No unsaved changes.
```

## Persistence and re-open

The drawer remembers which mode it was last in within the current session. Re-opening it (via any trigger) re-renders that mode with fresh data.

## Acceptance criteria

- Both modes are reachable from at least two triggers (top-bar chip + command palette).
- Clicking an issue jumps to the offending rule and focuses the field.
- The save diff shows every file that will be written; missing files are a bug.
- The diff view distinguishes added / removed / modified lines using glyph + colour (ABDD: glyph is required, colour is additive).
- The drawer opens and closes without flicker; switching modes is also clean.
- `Esc` closes the drawer; outside-clicking the body does not close it.

## Out of scope

- A persistent "always-open" drawer mode (the drawer is on-demand only)
- Inline editing of files in the drawer
- Real-time validation pulses (a v2 candidate; v1 validates on edit and on save)
- Workspace-level history of saves (a v2 candidate)
