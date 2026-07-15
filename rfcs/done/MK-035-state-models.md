# RFC MK-035 — State models for UI

**Status.** Implemented (v0.6.0)
**Tracks.** Server state machine, save state machine, trace stream state machine.
**Touches.** Top-bar status chips (MK-027), action button availability, drawer triggers.
**Supersedes.** State portions of MK-003 and MK-011.

## Summary

Three small finite state machines drive most of the chrome's behaviour: **server state** (is the mock server up?), **save state** (are there unsaved edits?), and **trace stream state** (is the UI receiving live events?). This RFC defines each state machine, its transitions, and the UI affordances tied to each state.

The state machines are intentionally small. They live in `App` and are visible directly via the top-bar status chips. The UI never displays a state outside this set.

## 9.1 Server state

```
[*] → Stopped
Stopped → Starting       (Start)
Starting → Running       (success)
Starting → Error         (failure)
Running → ReloadPending  (config changed, reload-class)
Running → RestartRequired (restart-class setting changed)
ReloadPending → Running  (Reload)
RestartRequired → Running (Restart)
Running → Stopped        (Stop)
Error → Stopped          (Dismiss / Stop)
```

### State semantics

| State | Meaning | Chip label | Glyph |
|---|---|---|---|
| Stopped | Server is not running | `Stopped` | ■ |
| Starting | Server boot in progress | `Starting` | ◔ |
| Running | Server is up and serving | `Running` | ● |
| ReloadPending | Saved changes need an in-process reload | `Reload pending` | ↻ |
| RestartRequired | Saved changes need a full restart | `Restart required` | ⏻ |
| Error | Boot failure or runtime error | `Error` | ! |

### Action availability

| Button | Visible when |
|---|---|
| Start | `Stopped`, `Error` |
| Stop | `Running`, `ReloadPending`, `RestartRequired`, `Starting` |
| Reload | `ReloadPending` |
| Restart | `RestartRequired` |

Buttons that aren't currently visible should be **disabled with a tooltip** (not hidden) so the user understands why they can't click. Exception: Start and Stop are toggle-style; they swap labels rather than coexist.

## 9.2 Save state

```
[*] → Saved
Saved → Unsaved          (edit)
Unsaved → Saving         (save action)
Saving → Saved           (success)
Saving → SaveError       (failure)
SaveError → Unsaved      (continue editing)
Unsaved → Saved          (discard changes)
```

### State semantics

| State | Meaning | Chip label | Glyph |
|---|---|---|---|
| Saved | All changes are written | `Saved` | ✓ |
| Unsaved | At least one file has unwritten edits | `Unsaved (N)` where N is file count | ● |
| Saving | Write in progress | `Saving…` | … |
| SaveError | A save attempt failed | `Save error` | ! |

### Auto-save policy

Rule content edits (URL, method, headers, body, respond) **auto-save** after a short debounce. After the debounce fires, the state transitions Unsaved → Saving → Saved automatically without an explicit Save click.

Restart-class setting changes do **not** auto-save — they require explicit save (the user reviews what they're committing before the server has to restart).

Save state therefore typically only shows `Unsaved (N)` when:
- A restart-class setting has been changed
- A save attempt failed

In all other normal editing, the chip shows `Saved`.

### Top-bar Save button

The top-bar Save button is:
- **Disabled** when the state is `Saved` (nothing to do)
- **Enabled** when the state is `Unsaved` (restart-class) or `SaveError`
- **Loading-style** (with a spinner glyph) when the state is `Saving`

## 9.3 Trace stream state

```
[*] → Disconnected
Disconnected → Connecting  (workspace open / server start)
Connecting → Streaming     (connected)
Connecting → TraceError    (failed)
Streaming → Paused         (pause UI)
Paused → Streaming         (resume)
Streaming → Disconnected   (server stopped)
TraceError → Connecting    (retry)
```

### State semantics

| State | Meaning | Top-bar chip (if shown) |
|---|---|---|
| Disconnected | No trace connection | (omit chip; server is also stopped) |
| Connecting | Attempting to subscribe | `Trace connecting` |
| Streaming | Receiving events | (omit chip; this is the default) |
| Paused | UI is not updating (user paused) | `Trace paused` |
| TraceError | Subscription failed | `Trace error` |

A "Trace paused" chip is the only one that adds noise to the top bar in the normal path; it disappears when streaming resumes. "Trace connecting" is briefly visible during workspace open.

### UI affordances

- The pause / resume button on the trace strip (MK-028) and trace screen (MK-029) toggles between `Streaming` and `Paused`.
- Clear button does not change state; it only empties the visible list.
- On `TraceError`, the trace screen shows a retry CTA with the underlying error message.

## Cross-state interactions

| Trigger | Effect |
|---|---|
| Server transitions Stopped → Starting | Trace transitions Disconnected → Connecting |
| Server transitions Running → Stopped | Trace transitions Streaming → Disconnected |
| Save state transitions to Saved with restart-class change | Server state transitions to RestartRequired |
| Save state transitions to Saved with reload-class change | Server state transitions to ReloadPending |

## Implementation notes

- All three machines are pure enums in `App`. Transitions happen via `update()` message handlers.
- The chip rendering reads state directly from `App`; no derived state.
- Save state should track which file caused the dirty state, since the Save Diff drawer (MK-032) needs that information.
- Trace state's `Streaming` state contains an event buffer (Vec or bounded ring buffer); pause does not flush the buffer.

## Acceptance criteria

- Every transition above is reachable in the UI (manually or via test).
- The top-bar chips correctly reflect state for every combination.
- Auto-save kicks in for rule edits within a debounce window (suggested: 500 ms after the last edit).
- Restart-class changes do not auto-save.
- The trace strip's pause/resume button correctly toggles `Streaming` ↔ `Paused`.

## Out of scope

- Crash recovery (e.g. resume an in-progress save after app restart) — v2
- Offline workspace editing without the apimock-rs binary present — v2
- Multi-server connections (one server per workspace in v1)
