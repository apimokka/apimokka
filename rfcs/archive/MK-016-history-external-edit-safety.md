# RFC MK-016 — Local history, external edit reload, and non-destructive safety

**Status.** Superseded by RFCs MK-021–MK-037 — workflow-centred redesign series
**Tracks.** Undo, external file watcher, destructive-action confirmation.
**Touches.** `screens/confirm_dialog.rs`, `message.rs::ConfirmAction`, `app.rs`, `shell/right_inspector.rs`, `screens/rule_builder.rs`

## Implementation summary

| Acceptance criterion | Met? | Where |
|---|---|---|
| External changes are never silently overwritten | ❌ | File watcher not in scope for mockup (no real I/O) |
| Unsaved GUI edits not silently discarded by reload | ❌ | No undo/redo |
| Destructive actions clearly explain impact | ✅ | Confirmation dialog pattern implemented (see below) |
| Local undo/history behaviour honest about limits | ❌ | No history; deferred |

## Confirmation dialog pattern (new in v0.2.0)

Four destructive actions now route through `Message::ConfirmRequest(ConfirmAction)`:

| Trigger | `ConfirmAction` | Dialog text |
|---|---|---|
| Delete rule (inspector) | `DeleteRule(NodeId)` | `Key::ConfirmDeleteRule` |
| Delete rule set (inspector) | `DeleteRuleSet(RuleSetId)` | `Key::ConfirmDeleteRuleSet` |
| Clear all headers (rule builder) | `ClearAllHeaders` | `Key::ConfirmClearHeaders` |
| Clear all body conditions (rule builder) | `ClearAllBody` | `Key::ConfirmClearBody` |

`ConfirmProceed` re-dispatches the actual destructive message.
`ConfirmCancel` (or Esc) dismisses without action.

The dialog is shown as a snora `Dialog` with highest priority (above command
palette, test-rule, and path-assistant dialogs). `on_close_modals = ConfirmCancel`.

## Remaining gaps

- No undo/redo history
- No file watcher (would use `notify` crate in production)
- No "Are you sure?" for workspace deletion (wizard cancel, overwrite)
