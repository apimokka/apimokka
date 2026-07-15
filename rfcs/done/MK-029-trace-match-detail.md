# RFC MK-029 — Trace and match detail

**Status.** Implemented (v0.6.0)
**Tracks.** S-11 Trace screen, S-12 Match detail panel.
**Touches.** Full diagnostic view for request history.
**Supersedes.** MK-012 (live trace + match detail).

## Summary

The Trace tab is the deep diagnostic view. It supplements the Routes-screen live strip with filtering, full request/response detail, and a condition-by-condition match explanation. The key UX commitment: when a request didn't match, show **why** — not just "miss".

## S-11 Trace screen

### Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Trace                                                                   │
│ [Method ▼] [Outcome ▼] [Path contains…]              [Pause] [Clear]    │
├───────────────────────────────────────────────┬─────────────────────────┤
│ Event Stream                                  │ Match Detail            │
│ ✓ POST /api/checkout · 12 ms · matched        │ Request                 │
│ ◯ GET  /api/cart · 3 ms · miss                │ POST /api/checkout      │
│ ↩ GET  /users · 2 ms · fallback               │ Headers                 │
│                                               │ Body                    │
│                                               │                         │
│                                               │ Outcome                 │
│                                               │ Matched checkout.toml #2│
│                                               │ [Replay as test input]  │
└───────────────────────────────────────────────┴─────────────────────────┘
```

| Column | Width |
|---|---:|
| Event stream | flexible, ≥ 480 px |
| Match detail | 360–520 px |

### Filter bar

| Filter | Type | Behaviour |
|---|---|---|
| Method | Compact dropdown or segmented buttons | Filters event stream live |
| Outcome | Dropdown: All / Matched / Fallback / Miss / Error | Filters event stream live |
| Path contains | Text input | Filters event stream live (case-insensitive substring) |
| Pause | Toggle button | Pauses UI stream updates (does not affect server) |
| Clear | Ghost button — danger only if "irreversible" hint applies | Clears visible list; does not delete history |

Filters apply as the user changes them; no Apply button.

### Event row

| Field | Required |
|---|---|
| Outcome glyph + label | Yes |
| Method | Yes |
| Path | Yes |
| Time | Yes (e.g. `14:32:08`) |
| Duration | Yes (e.g. `12 ms`) |
| Matched rule or fallback file | When applicable |
| Dropped events warning | When `dropped_count > 0` since the last visible event |

Outcome glyphs from MK-022: `✓` matched, `↩` fallback, `◯` miss, `!` error.

A dropped-events row appears as a thin notice between events:
```
⚠ 12 events dropped (queue full)
```

### Event row interactions

| Action | Result |
|---|---|
| Click event | Loads it into the match detail panel |
| Replay icon | Opens Test Rule dialog (MK-034 O-03) with the event request pre-filled |
| Keyboard arrow keys | Move selection through visible events |

### Selected-event state
The currently-selected event has a subtle highlighted background and a left accent strip — the same selection treatment used in the Routes sidebar (MK-028) for consistency.

## S-12 Match detail panel

The right panel of the Trace screen. Also rendered as a side-out when the user clicks an event in the Routes-screen trace strip (compact variant).

### Sections (top → bottom)

| Section | Content |
|---|---|
| **Request** | Method, path, headers (table), body (monospace, scrollable) |
| **Outcome** | One of: matched / fallback / miss / error — with glyph and label |
| **Response** | Status code + label, headers, body preview |
| **Match reasoning** | Either the winning rule or the closest-rule explanation (see below) |
| **Actions** | Replay as test input · Open matched rule · Copy request |

### Match reasoning — matched case

```
Matched
checkout.toml · Rule #2

Condition          Expected           Actual          Result
Method             POST               POST            Matched
URL path           equals /api/pay    /api/pay        Matched
Header             content-type exists   present       Matched
Body               user.id equal 123  123             Matched
```

### Match reasoning — closest-rule case (when outcome = miss)

```
Closest rule
checkout.toml · POST /api/checkout → 500
2 of 4 conditions matched

Condition          Expected           Actual          Result
Method             POST               POST            Matched
URL path           equals /api/checkout  /api/cart    Failed
Header             content-type exists   present       Matched
Body               user.id equal 123  missing         Failed

[Open this rule] [Replay against this rule]
```

This explanatory pattern is the most important diagnostic affordance in the app. It turns "miss" from a dead end into a debugging path.

### Match reasoning — fallback case

```
Fallback
Served users.json from fallback files

(No rule matched. The path /users was handled by the dynamic-route
fallback that maps /users to users.json.)

[Open users.json] [Replay as test input]
```

### Match reasoning — error case

```
Error
Middleware script auth.rhai threw an error.

Message
"Token expired at line 12"

[Open auth.rhai] [Copy request]
```

## Debug-a-miss flow

```
User sees miss in trace
  → Click the miss event
  → Match detail shows the request + closest-rule explanation
  → User clicks "Replay against this rule"
  → Test Rule dialog opens, pre-filled with the request + the closest rule selected
  → User adjusts the rule (e.g. fix the URL operator)
  → Triggers the request again from their app
  → Trace shows ✓ matched
```

The whole loop happens without leaving the Trace tab until the rule adjustment, which switches to Routes briefly (the user can switch back to Trace to verify).

## Acceptance criteria

- Filtering by method, outcome, and path is instant (no submit button).
- Selecting an event populates the match detail panel.
- The closest-rule explanation appears for every miss with at least one rule defined in the workspace.
- "Replay as test input" is reachable from both the event row and the detail panel.
- The dropped-events notice is visible and non-blocking.
- All outcome glyphs + labels are ABDD-compliant (no colour-only state).
- The whole screen renders correctly in light and dark themes.

## Out of scope

- Network-level packet inspection (not in product scope)
- Live editing of historical events (events are immutable)
- Trace history persistence across server restarts (deferred to a v2 RFC)
