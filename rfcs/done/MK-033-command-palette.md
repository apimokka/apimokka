# RFC MK-033 — Command palette

**Status.** Implemented (v0.6.0)
**Tracks.** O-02 command palette.
**Touches.** Keyboard-first action launcher.
**Supersedes.** Command palette portion of MK-015.

## Summary

The command palette is the keyboard-first action launcher. It is the canonical fallback for any action that lacks a dedicated shortcut, and the discoverability surface for keyboard shortcuts that exist. Power users live here.

## Activation

- `Ctrl/Cmd + K` toggles the palette from anywhere in the workspace state.
- Click the `⌘K` button in the top bar (MK-027).
- The top-bar trigger has a tooltip displaying the shortcut.

## Layout

```
┌────────────────────────────────────────────┐
│ ⌘K toggle    Esc close         [✕]         │
│                                            │
│ [Search commands…                        ] │
│ ──────────────────────────────────────────│
│ Save workspace                       ⌘S    │
│ Add rule                                   │
│ Toggle live trace strip                    │
│ Open validation drawer                     │
│ Start server                               │
│ Switch workspace                           │
└────────────────────────────────────────────┘
```

| Region | Content |
|---|---|
| Header | Title + visible keyboard shortcut chips + close button |
| Search input | Auto-focused on open; filters command list as the user types |
| Command list | Scrollable; each row is a command + optional shortcut chip |

Width: ~520 px. Vertical size: ~360 px (search + ~12 rows visible).

## Commands (v1 set)

Grouped conceptually (groups are subtle visual hints, not hard headers):

### Workspace
- Save workspace · `⌘S`
- Switch workspace
- Open workspace
- Create new workspace
- Toggle theme
- Change locale

### Routes
- Add rule
- Add rule set
- Test current rule
- Toggle live trace strip

### Server
- Start server
- Stop server
- Reload config · `⌘R` (when available)
- Restart server

### Drawers
- Open validation drawer
- Open save diff

### Navigation
- Go to Routes
- Go to Trace
- Go to Scripts
- Go to Settings

## Filtering

- Substring match against command label (case-insensitive)
- Live as the user types
- No result: empty-state "No matching commands"

A future enhancement: fuzzy match (e.g. `ts` matches "Toggle live trace **s**trip"). Out of v1 scope.

## Keyboard behaviour

| Key | Action |
|---|---|
| `Ctrl/Cmd + K` | Toggle palette |
| `Esc` | Close palette |
| `↑` / `↓` | Move selection in the command list |
| `Enter` | Execute selected command |
| Any character | Append to search input |

When the palette opens, the search field is auto-focused and any partial query from the previous open is **discarded** (so opening it always starts fresh).

## Shortcut hints

Where a command has a dedicated shortcut (e.g. `⌘S` for Save), the shortcut appears as a chip on the right of the row. Shortcut chips use the `chip` token style.

The palette header itself shows two global shortcuts as hint chips: `⌘K toggle` and `Esc close`. This makes the palette self-documenting — users don't need to read a help page to learn how to operate it.

## Visual treatment

- The palette is a snora `Dialog` on `surface.raised`.
- Card padding: `space.6`.
- Search input is a full-width text input with placeholder "Search commands…".
- Command rows are subtle cards (no border) with `space.3` vertical / `space.4` horizontal padding.
- Selected row uses the same selected-card style as everywhere else (subtle accent background + left strip).

## Context awareness (v1 minimal)

Some commands are only available in certain contexts (e.g. "Test current rule" needs a rule selected). In v1, context-unavailable commands appear grey and disabled with a tooltip explaining why. A more dynamic context system (hiding commands entirely based on context) is a v2 candidate.

## Acceptance criteria

- `Ctrl/Cmd + K` reliably opens and closes the palette from every workspace view.
- The search input is auto-focused on open.
- `Esc` closes the palette.
- Selected-row navigation via arrow keys works correctly with the visible scroll.
- Every command on the v1 list works correctly when invoked.
- Disabled commands have a visible non-colour reason (tooltip + reduced contrast).
- The header shortcut hints are visible and accessible.

## Out of scope

- Fuzzy matching (v2)
- Custom user-defined commands (v2)
- Recent-commands list at the top (v2)
- Macros (out of scope)
