# RFC MK-042 — Trace screen completeness

**Status.** Implemented (v0.11.0)
**Tracks.** Trace screen (S-11) and match detail panel (S-12). Makes both
production-quality by filling in the four deferred areas from the v0.6.0
implementation.
**Touches.** `screens/trace.rs`, `screens/routes.rs` (trace strip), `app.rs`
state + messages, `apimokka-model/mock.rs` (richer sample data), i18n.
**Follows.** MK-029 (original trace spec), MK-035 (state models).

## Context

The v0.6.0 trace screen (MK-029) was intentionally minimal. It rendered a
scrollable event list and a static match detail panel, but:

1. The filter text input was not wired — it rendered but had no effect.
2. The match detail panel showed the same layout regardless of outcome
   (`Matched`, `Fallback`, `Miss`, `Error` all looked the same).
3. Dropped-count warnings (when the trace queue overflowed) were not shown.
4. The rule editor's "Recent trace activity" strip used a URL path heuristic
   to decide which events matched a rule, not the `rule_index` that the engine
   actually reports.

## Changes

### 1. Live filter

Add `trace_filter: String` to `App` state, wired to the existing filter input.
Events are shown only when the filter string (case-insensitive) appears in
`url_path`, `method`, or the outcome label. Empty filter = all events shown.

### 2. Outcome-aware match detail

Each outcome variant renders distinct content:

| Outcome | Detail panel content |
|---|---|
| `Matched { rule_set_index, rule_index }` | Rule set file name + rule summary. "Jump to rule" button → `SelectRule(id)` + switch to Routes tab. |
| `Fallback { file_path, status }` | File name + status + "Jump to file" button → `SelectFileRoute(path)` + switch to Routes tab. |
| `Miss { status }` | Status code + "No rule matched this request." + "Create rule for this path" CTA. |
| `Error { kind, message }` | Error kind + full message in monospace. |

### 3. Dropped-count warning

When an event with `dropped_count > 0` is selected, a notice appears in the
detail panel: "N events were dropped before this one (queue full)." Uses the
existing `TraceDroppedEvents` i18n key.

### 4. Rule editor trace strip: match by rule index

`recent_matching_events` now matches by checking whether
`TraceOutcome::Matched { rule_set_index, rule_index }` corresponds to the
displayed rule in the snapshot, rather than comparing URL paths. URL path
comparison is kept as a fallback when the snapshot indices do not resolve
(e.g. the rule list was edited since the event was recorded).

## Acceptance criteria

- Typing in the filter field immediately filters the event list.
- Selecting a `Matched` event shows the rule name and "Jump to rule".
- Selecting a `Fallback` event shows the file path and "Jump to file".
- Selecting a `Miss` event shows the status and a "Create rule" CTA.
- Selecting an `Error` event shows kind + message.
- An event with `dropped_count > 0` shows the dropped count in the detail.
- The rule editor trace strip shows exactly the events that matched the
  displayed rule by index (not URL heuristic).
- Zero errors, zero warnings, new + existing tests pass.
