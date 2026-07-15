# RFC MK-028 — Routes workbench

**Status.** Implemented (v0.6.0)
**Tracks.** S-05 Routes, S-06 Add new endpoint, S-07 Live trace strip, S-08 Rule inspector.
**Touches.** The product's identity screen.
**Supersedes.** MK-005 (file-route browser), MK-006 (rule-set manager), MK-007 (visual rule builder), MK-009 (respond editor), MK-010 (service strategy controls), MK-012 (live trace + match detail) partially.

## Summary

Routes is the workbench. **Edit a rule and observe traffic at the same time.** Three columns: a tree of rule sets and other workspace objects on the left, a visual rule editor in the middle, and either a live trace strip or a rule inspector on the right.

## Layout

```
┌───────────────┬──────────────────────────────────────┬────────────────┐
│ Rule Sets     │ Rule Editor                          │ Live Trace     │
│               │                                      │                │
│ ▾ checkout    │ WHEN                   →  RESPOND    │ ✓ POST /pay    │
│   GET /cart   │ ┌ URL Path ┐             ┌ Resp. ┐   │   Matched 12ms │
│   POST /pay   │ │ /api/pay │             │ 500   │   │                │
│               │ └──────────┘             └───────┘   │ ◯ GET /cart    │
│ Fallback files│ ┌ Headers ┐                          │   Miss 3ms     │
│ Middleware    │ └─────────┘                          │                │
│               │ ┌ Body conditions ┐                  │ ↩ GET /users   │
│ [+ Rule set]  │ └─────────────────┘                  │   Fallback     │
│               │                                      │ [Pause][Clear] │
└───────────────┴──────────────────────────────────────┴────────────────┘
```

| Column | Responsibility | Default width |
|---|---|---:|
| Left | What exists (rule sets / rules / fallback files / scripts) — selection, create | 260–300 px |
| Center | What this rule does — visual editor | flexible, ≥ 520 px |
| Right | What just happened (live trace) OR what this rule's metadata is (inspector) | 260–320 px |

The right column has a mode toggle in its header: **Live trace** (default for editing) or **Rule inspector** (for strategy fields, validation drilldown, quick actions).

## Left sidebar

### Content groups (top to bottom)

```
Rule sets                       ← group header
  checkout.toml ●               ← rule set (dirty marker)
    ✓ GET /api/cart → 200       ← rule (status glyph + summary)
    ! POST /api/checkout → 500
  auth.toml
    GET /oauth/callback → 302

Fallback files                  ← group header
  users.json      /users        ← file name + URL hint
  health.json     /health

Middleware scripts              ← group header
  auth.rhai                     ← script (read-only)
```

### Item states

| State | Visual treatment |
|---|---|
| Selected rule | Highlighted row background + 3 px left accent strip |
| Unsaved rule set | Dirty dot (●) next to the filename |
| Validation error | Error glyph + accessible label inside the row |
| Recently matched | Subtle "Matched" marker for ~5 s after a trace event |
| Disabled / unavailable | Muted text with explanation on focus |

### Rule summary text

Each rule row shows a compact one-line summary:
- Status glyph (✓ if validation clean and matched recently, ✕/⚠ if validation issues)
- Method
- URL path
- Arrow + status code

Example: `✓ POST /api/checkout → 500`

The summary updates as the user edits the rule.

### Actions

| Action | Location |
|---|---|
| Add rule set | "+ Rule set" button pinned at the bottom of the sidebar |
| Add rule (within rule set) | "+" affordance on each rule-set group header, plus right-click → "Add rule", plus palette command |
| Reorder rules | Drag-and-drop within a rule set, or right-click → Move up/down, or palette command |
| Delete rule / rule set | Right-click → Delete (with confirm dialog from MK-034) |

## Center rule editor

The editor reads as a sentence:

> **WHEN** request matches these conditions → **RESPOND** with this output.

The arrow between the columns is a substantial visual element (`section` token size) — not a tiny chevron.

### WHEN cards (left half)

#### URL Path card

| Field | Behaviour |
|---|---|
| Operator | Equal · Starts With · Contains · Ends With · Wildcard · Not Equal |
| Path input | Required when an operator is selected |
| Hint | "Use `/api/orders`, not the full URL." |

#### Method card (segmented buttons)

Segmented: `Any` · `GET` · `POST` · `PUT` · `PATCH` · `DELETE` · `Other…`. The selected method has visible non-colour state. `Any` means no method constraint.

#### Headers card

Per row:

| Column | Behaviour |
|---|---|
| Name | Lowercase suggested; do not destroy user input mid-typing |
| Operator | Equal · Contains · Starts With · Ends With · Regex · Exists · Absent · Not Equal · Wildcard |
| Value | Hidden (or disabled) for Exists / Absent operators |
| Row actions | Duplicate · Delete (icon buttons) |

Card footer: `+ Header` button.

#### Body conditions card

Per row:

| Column | Behaviour |
|---|---|
| Dotted path | `a.b.c`, `items.0.name` — NOT JSONPath |
| Assistant button (`…`) | Opens dotted-path assistant (MK-034 O-04) |
| Operator | Changes the value input type below |
| Value | JSON editor / string / number / hidden depending on operator |

Card footer: `+ Body condition` button.

A JSONPath-flavoured input (`$.user.id`) shows an inline warning: "Use dotted path syntax, e.g. `user.id`. JSONPath is not supported."

### RESPOND card (right half)

| Field | Behaviour |
|---|---|
| Mode tabs | **Inline text** · **Serve file** (mutually exclusive) |
| Body editor | Multiline monospace area when Inline; file path picker when Serve file |
| Status | Picker with common codes (200, 201, 204, 400, 401, 403, 404, 500, 502, 503) + custom text input |
| Delay | Number input in milliseconds; optional |
| Test rule button | Bottom-right, primary style, opens MK-034 O-03 Test Rule dialog |

Mutex hint under the mode tabs: "Inline text and a served file cannot both be set on one rule."

### Empty state (no rule selected)

```
No rule selected

Choose a rule from the left, or create a new endpoint.

[Add rule]
```

## Right column

### Mode A — Live trace strip (default)

```
Live Trace
[Pause] [Clear]

✓ POST /api/checkout
  Matched · 12 ms · checkout.toml

◯ GET /api/cart
  Miss · 3 ms

↩ GET /users
  Fallback · users.json
```

| Action | Result |
|---|---|
| Click event | Opens match detail (MK-029 S-12 pane, or expands inline) |
| Replay icon (per row) | Opens Test Rule dialog with the event request pre-filled |
| Pause | Stops UI stream updates only — server keeps running |
| Clear | Clears the visible list only; does not delete event history |

Event row layout: outcome glyph + label, method + path, time + duration. Two-line layout; metadata on the second line is `caption` token and `text.secondary` colour.

The strip is capped at the most recent ~15–30 events visible at a time, with scrolling for older events kept in memory.

### Mode B — Rule inspector

```
Rule Inspector

Validation
✓ No issues

Strategy
Priority: [10]

Quick actions
[Duplicate]
[Move up] [Move down]
[Delete]
```

| Section | Content |
|---|---|
| Validation | Per-rule issues with severity glyph + message |
| Strategy fields | `Priority` (when strategy is Priority); `Weight` (when WeightedRandom); empty otherwise |
| Quick actions | Duplicate, Move up, Move down, Delete (last is danger button + confirm) |

The mode switch is a small segmented toggle in the right-column header.

## Add new endpoint flow

### Entry points
- "+" affordance on the rule set group header
- Empty-state CTA when no rule selected
- Command palette: "Add rule"
- Right-click / context menu on a rule set

### Draft defaults

| Field | Default |
|---|---|
| Method | GET |
| URL operator | Equal |
| URL path | empty, focused |
| Headers | none |
| Body conditions | none |
| Response mode | Inline text |
| Status | `200 OK` |
| Body | `{}` |
| Delay | empty / 0 |

After creation, focus jumps to the URL path field so the user types the path next.

## Acceptance criteria

- The user can edit a rule and watch the live trace in the same view without switching tabs.
- Adding a new rule lands the user on a focused URL path field with sensible defaults pre-filled.
- The right column toggles cleanly between trace strip and inspector.
- Per-rule validation appears both in the left-sidebar rule row (glyph) and the inspector.
- The replay icon on a trace event row opens Test Rule pre-filled.
- All four card types (URL path / method / headers / body) render correctly with the design tokens from MK-022.
- All screen content visible in Routes is reachable by keyboard following the focus order in MK-023.

## Out of scope

- Match detail panel itself (MK-029)
- Test Rule dialog (MK-034)
- Dotted-path assistant (MK-034)
- Trace event schema details (MK-035)
- Strategy semantics (apimock-rs concern, surfaced via the Settings screen, MK-030)
