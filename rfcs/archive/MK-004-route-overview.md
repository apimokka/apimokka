# RFC MK-004 — Route overview and three-layer navigator

**Status.** Superseded by RFC MK-021..MK-037 series (workflow-centred redesign)
**Tracks.** Overview screen.
**Touches.** `screens/overview.rs`

## Implementation summary

| Acceptance criterion | Met? | Where |
|---|---|---|
| Overview clearly explains all three routing layers | ✅ | `overview.rs` script-cards stacked: Script → Rules → Files |
| Users can navigate from each layer summary to its detailed surface | ⚠️ Partial | Quick-action buttons for Routes / Trace; no direct "go to Scripts/Files" tile |
| Empty states encourage next action | ✅ | `EmptyNoWorkspace` shown when snapshot absent |
| Diagnostics + runtime issues visible but not overwhelming | ✅ | Health grid summarises validation / dirty / server / trace |

## Deferred

Per-layer click-through (script card → Scripts screen) is a one-line
addition wired to `Message::SwitchTab`; not done because the quick-action
buttons cover the same destinations.
