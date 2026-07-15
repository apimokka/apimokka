# RFC MK-034 — Dialog layer (workspace menu, test rule, dotted-path, confirm)

**Status.** Implemented (v0.6.0)
**Tracks.** O-01 Workspace menu, O-03 Test Rule dialog, O-04 Dotted-Path Assistant, O-05 Confirm dialog.
**Touches.** All non-palette overlays.
**Supersedes.** MK-008 (dotted-path assistant), MK-013 (replay / match-test), MK-016 (destructive-action confirmations), workspace-menu portion of MK-020.

## Summary

Four small overlays that supplement the workspace shell. Each is small in scope but high in importance: they handle the moments where the user needs focused interaction without losing the workspace context. They share the dialog priority order from MK-021.

## O-01 Workspace menu

### Trigger
Click the workspace identity in the top bar (the `apimokka · payments-mock ▼` button).

### Snora primitive
`AppLayout::header_menu(element)` with `on_close_menus(message)`. The menu drops down below the header; outside-click dismisses.

### Content

```
┌────────────────────────────────────────────┐
│ Current workspace                          │
│ payments-mock                              │
│ /Users/me/dev/payments-mock                │
│ ──────────────────────────────────────────│
│ Project Alpha   /path/alpha   Today        │
│ Shop Mock       /path/shop    Yesterday    │
│ ──────────────────────────────────────────│
│ [Open workspace…] [Create new workspace…]  │
└────────────────────────────────────────────┘
```

| Section | Content |
|---|---|
| Header | Current workspace label + path (read-only) |
| List | Recent workspaces, click to switch |
| Footer | "Open workspace…" (system picker) and "Create new workspace…" (Wizard) |

### Behaviour

- Selecting a recent workspace closes the menu and loads that workspace.
- Switching workspaces with unsaved changes triggers a confirm dialog (see O-05) before switching.
- `Esc` and outside-click both close the menu.

## O-03 Test Rule dialog

### Purpose
Run a dry match check using synthetic or replayed request input. The result shows condition-by-condition pass/fail — the same explanatory pattern as MK-029's match detail.

### Trigger
- "Test rule" button in the rule editor (MK-028 RESPOND column)
- Replay icon on a trace event row (MK-028 trace strip, MK-029 event stream)
- Command palette: "Test current rule"
- `Ctrl/Cmd + Enter` when the rule editor is focused

### Layout

```
┌──────────────────────────────────────────────┐
│ Test rule                          [✕]       │
│ Dry-run match against the selected rule.     │
│                                              │
│ Method [POST ▼]                              │
│ Path   [/api/checkout]                       │
│ Headers [content-type: application/json…  ]  │
│ Body                                         │
│ ┌──────────────────────────────────────────┐ │
│ │ {"user":{"id":123}}                      │ │
│ └──────────────────────────────────────────┘ │
│                                              │
│ [Run test]                                   │
│                                              │
│ Result                                       │
│ ✓ Matched                                    │
│ Method matched · URL matched · Body matched │
└──────────────────────────────────────────────┘
```

### Input sources (prefill)

| Source | Prefill |
|---|---|
| Selected rule | Method + URL path copied from the rule's `WHEN` |
| Trace event replay | Method, path, headers, body copied from the event |
| Blank | `GET /` |

### Result states

| State | Display |
|---|---|
| Not run | Neutral hint: "Run the test to see condition-by-condition result." |
| Matched | Success banner with summary + per-condition list (all passed) |
| No match | Miss banner with the list of failed conditions |
| Error | Error banner with the message |

### Per-condition result

The result section shows the same condition-by-condition table as the match detail (MK-029):

```
Condition          Expected           Actual          Result
Method             POST               POST            Matched
URL path           equals /api/checkout  /api/checkout  Matched
Body               user.id equal 123  123             Matched
```

### Visual treatment

- Snora `Dialog`, ~520 px wide.
- Header: title + dismiss button.
- Footer: `[Close]` (ghost) and `[Run test]` (primary). Pressing `Ctrl/Cmd + Enter` runs the test.

## O-04 Dotted-path assistant

### Purpose
Help users create valid body-condition paths without writing JSONPath. Many users reach for `$.user.id` instinctively; this dialog teaches the dotted-path syntax by giving them the right answer.

### Trigger
- The `…` button next to a body-condition path input (MK-028 body conditions card)
- Command palette (less common entry)

### Layout

```
┌───────────────────────────────────────────────┐
│ Dotted-path assistant              [✕]        │
│                                               │
│ Paste sample JSON                             │
│ ┌───────────────────────────────────────────┐ │
│ │ {"user":{"id":123,"name":"Aki"}}          │ │
│ └───────────────────────────────────────────┘ │
│                                               │
│ JSON tree                                     │
│ ▾ user                                        │
│   id            user.id          [Use]        │
│   name          user.name        [Use]        │
│                                               │
│ Selected path: user.id                        │
│ [Cancel] [Insert path]                        │
└───────────────────────────────────────────────┘
```

### Behaviour

- The user pastes (or types) sample JSON in the top input.
- The dialog parses it and renders a tree on the fly.
- For each leaf, the row shows the leaf value and the **dotted path** that targets it.
- The "Use" button on a row sets the selected path; "Insert path" closes the dialog and writes the path into the body-condition row that opened the assistant.

### Path syntax rules

- Object access: `a.b.c`
- Array access: `items.0.name` (numeric index)
- **Not JSONPath**: `$.foo` is not recognised; if the user types it, the dialog shows an inline hint "Use dotted path syntax, e.g. `user.id`. JSONPath is not supported."

### Validation

- Invalid JSON in the input shows an inline error under the input with the parser's column number.
- Empty JSON shows the empty-state "Paste sample JSON to build a path."

## O-05 Confirm dialog

### Purpose
Confirm destructive actions only. Used sparingly — most actions in apimokka are non-destructive and don't need confirmation.

### Triggers

| Action | Confirm text |
|---|---|
| Delete rule | "Delete rule? This removes the selected rule." |
| Delete rule set | "Delete rule set? This removes the file and its rules." |
| Discard unsaved changes | "Discard unsaved changes? Edits in N files will be lost." |
| Overwrite existing workspace folder (from Wizard) | "Overwrite contents of `/path`? Existing files will be replaced." |
| Switch workspace with unsaved changes | "Switch workspaces? Unsaved edits will be lost." |

### Layout

```
┌───────────────────────────────────┐
│ Delete rule?                      │
│ This removes the selected rule.   │
│                                   │
│              [Cancel] [Delete rule] │
└───────────────────────────────────┘
```

### Rules

| Rule | Reason |
|---|---|
| Safe action first | Cancel is on the left so muscle-memory clicks the safer button |
| Destructive action visually distinct | The right button uses `danger` style — colour is supplementary; the placement, label, and button variant carry the message |
| `Esc` cancels | Always |
| `Enter` does not accidentally confirm danger | The danger button must be **explicitly focused** for `Enter` to activate it |
| One destructive action per dialog | No multi-step or multi-option destructive flows |

### Visual treatment

- Snora `Dialog`, ~420 px wide.
- Title uses `section` token.
- Description uses `body` token, `text.secondary` colour.
- The `[Cancel] [Delete rule]` row has the safe action left, destructive right, with a flex spacer between.

## Acceptance criteria

- Each of the four overlays can be opened from at least two triggers (button + palette where applicable).
- All four overlays follow the dialog priority order from MK-021 — opening one closes any other open overlay first.
- The Test Rule dialog correctly pre-fills from a selected rule, a trace event, or blank.
- The dotted-path assistant rejects `$.` JSONPath input with an inline hint.
- The confirm dialog's destructive button is visually distinct via shape + label + colour (ABDD: never colour alone).
- All four overlays render correctly in light and dark themes.

## Out of scope

- Workspace menu pinning (pinning lives on the Dashboard, MK-025)
- Multi-rule batch testing (a v2 candidate)
- Real-time JSON syntax highlighting in the dotted-path input (v2 candidate)
