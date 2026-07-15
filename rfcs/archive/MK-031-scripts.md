# RFC MK-031 — Scripts

**Status.** Withdrawn — scripts tab deferred; see ROADMAP.md
**Tracks.** S-14 Scripts screen.
**Touches.** Read-only inspection of Rhai middleware scripts.
**Supersedes.** MK-014 (Rhai middleware script surface).

## Summary

Scripts is a low-frequency, **read-only** screen for inspecting Rhai middleware files that ship with the workspace. Editing scripts is out of v1 scope — opening a Rhai editor here would create a false affordance (users would expect autocomplete, linting, debugger). The screen exists so the user can read what middleware exists and locate the file to edit externally.

## Layout

```
┌────────────────┬────────────────────────────────────────────┐
│ Scripts        │ auth.rhai                                  │
│                │                                            │
│ ▸ auth.rhai    │ 1  fn before_request(req) {                │
│   rewrite.rhai │ 2      if !req.has_auth() {                │
│                │ 3          return rejected("401");         │
│                │ 4      }                                   │
│                │ 5      req                                 │
│                │ 6  }                                       │
│                │                                            │
│ Copy path      │ /workspace/middleware/auth.rhai            │
└────────────────┴────────────────────────────────────────────┘
```

| Column | Width |
|---|---:|
| Script list | 220–280 px |
| Code viewer | flexible |

## Requirements

| Requirement | Reason |
|---|---|
| Read-only viewer | Avoid false editing affordance |
| Monospace font (`mono` token) | Code legibility |
| Line numbers | Error-location readability |
| Copy path action | Developer convenience — easier to paste into a terminal or editor |
| Empty state | "No middleware scripts in this workspace." |
| Selection state on script list | Same selection treatment as Routes sidebar |

## Code viewer

- Monospace font from `mono` token (~13–14 px).
- Line numbers in a gutter on the left, `caption` token, `text.secondary`.
- Whole-content scrollable both vertically and horizontally.
- No syntax highlighting in v1 (a `Rhai` lexer would be useful but not blocking — flagged as a v2 candidate).
- Selecting text is permitted; copy works via the system clipboard.

## Empty state

When the workspace has no middleware scripts:
```
No middleware scripts in this workspace.
Middleware scripts run before rule matching and can transform requests.
```

The empty state explains what scripts do so users who never see one still understand their place in the request-handling pipeline.

## Acceptance criteria

- The list of scripts shows every Rhai file in the workspace's middleware directory.
- Selecting a script displays its contents in the viewer.
- Line numbers render correctly even for files of 500+ lines.
- The "Copy path" button copies the absolute path of the currently-viewed script.
- The screen renders correctly in light and dark themes (mono font is legible on both).

## Out of scope

- Editing
- Linting / error checking
- Syntax highlighting (v2 candidate)
- Debugger / breakpoint UI
- Hot-reload of script changes from disk (v2 candidate, alongside external-edit detection)
