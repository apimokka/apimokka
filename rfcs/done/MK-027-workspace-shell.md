# RFC MK-027 — Workspace shell (top bar and left rail)

**Status.** Implemented (v0.6.0)
**Tracks.** S-03 workspace shell.
**Touches.** Top bar, left rail navigation, status chip group, global action group, view controls.
**Supersedes.** Top-bar portions of MK-001, MK-011, MK-020.

## Summary

The shell is the chrome that persists across Routes, Trace, Scripts, and Settings. It answers four questions at a glance: which workspace, is the server running, are there pending changes, and what can I do globally.

The shell visual budget is **quiet** — the user spends most of their attention inside the body, not on the chrome.

## Top bar

### Regions (left → right)

| Region | Contents |
|---|---|
| Identity | `apimokka · workspace-name ▼` (clickable; opens workspace menu) |
| Status | Server state chip, save state chip |
| Global actions | Save, Reload, Restart, Start/Stop server |
| View controls | Trace strip toggle, theme toggle, command palette |
| Locale | EN / JA picker |

### Layout sketch

```
┌──────────────────────────────────────────────────────────────────────┐
│ apimokka · payments-mock ▼   Running   Saved      [Save] [Reload]    │
│                                                  [⏻] [∿] [☾] [⌘K] EN │
└──────────────────────────────────────────────────────────────────────┘
```

The bar is one row tall. Identity is left-aligned; locale is right-aligned. Status chips and action buttons share the middle, with a flex spacer absorbing extra width.

### Visual treatment
- Uses `surface.panel` with a subtle bottom shadow (no hard border line).
- Vertical padding: `space.3` (12 px); horizontal padding: `space.5` (20 px).
- The identity row uses `section` token (17–18 px semibold) for the workspace name and `body` for the app name; a muted `·` separates them.

### Identity click

Clicking the identity opens the **Workspace menu** (MK-034 O-01) — a snora `header_menu` dropdown listing:
- Current workspace (label + path)
- Recent workspaces (clickable)
- "Open workspace…" and "Create new workspace…" actions

`Esc` or outside-click closes the menu. Implementation uses snora's `header_menu(element)` and `on_close_menus(message)`.

### Status chip group

Two chips, always visible (when relevant):

| Chip | When shown |
|---|---|
| Server state: `Running`, `Stopped`, `Reload pending`, `Restart required`, `Starting`, `Error` | Always |
| Save state: `Saved` or `Unsaved (N)` | Always |

If a third status is needed (e.g. trace paused), it is added to the chip group. If more than three chips would appear, the lower-priority ones collapse into a "Status" dropdown next to the chips (per MK-022).

### Global action buttons

| Button | Enabled when | Style |
|---|---|---|
| Save | unsaved changes exist | Secondary |
| Reload | `ServerState::ReloadPending` | Secondary |
| Restart | `ServerState::RestartRequired` | Secondary |
| Start/Stop | always (toggles based on state) | Primary or Secondary based on context |

Auto-save for rule edits applies (see MK-035 save state machine) so the **Save** button is usually disabled — it only lights up when a restart-class setting is pending or save fails.

### View controls

| Control | Action |
|---|---|
| Trace strip toggle (`∿`) | Toggle the Routes-screen live trace strip on/off; visible only on Routes tab |
| Theme toggle (`☾`/`☀`) | Switch light/dark |
| Command palette (`⌘K`) | Open the command palette |

Each carries a tooltip plus an accessible label. The trace toggle visibly indicates active state.

### Locale picker

Drop-down with EN and JA. Switching is instant and persists across sessions (implementation detail).

## Left rail

### Layout

```
┌───────┐
│ ⎘ Routes  │
│ ∿ Trace   │
│ ⚙ Scripts │
│ ⚒ Settings│
└───────┘
```

Width: **120 px** with labels (default at ≥ 1280 px window width), **72 px** icon-only below 900 px window width.

### Rules

- Always visible in Workspace state.
- The selected destination has a visible non-colour indicator (filled background tint **and** a left-accent strip) — not colour alone.
- Keyboard navigation: arrow keys move focus up/down; Enter activates.
- Each item is a button with an accessible label even in icon-only mode.

### Visual treatment

- Uses `surface.panel` with no hard right border (a subtle right-edge shadow separates rail from body).
- Each item: icon + label, vertically centred, `body` token.
- Selected: subtle background tint + a 3 px-wide left accent strip in `accent.primary`.
- Hover: subtle tint change (no animation required).

## Bottom drawer trigger

When validation issues exist, the count chip in the top bar (`3 issues`) is clickable and opens the validation drawer. Similarly for the unsaved chip → save-diff drawer.

This means the bottom drawer is reachable from the top bar without going to a dedicated tab.

## Acceptance criteria

- The shell is the same on every workspace tab; switching tabs changes only the body content.
- The workspace identity is keyboard-reachable and opens the dropdown via Enter / Space.
- All status chips render glyph + text (ABDD).
- The locale switcher is keyboard-reachable and announces the locale change to screen readers.
- The view controls visibly indicate their active state (trace toggle on/off, current theme).
- At window widths below 900 px, the global action group collapses to an overflow menu and the left rail collapses to icon-only — without losing any functionality.

## Out of scope

- Workspace menu detail (MK-034)
- Command palette detail (MK-033)
- Bottom drawer detail (MK-032)
- Save state semantics (MK-035)
