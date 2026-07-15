# RFC MK-010 — Service strategy and rule selection controls

**Status.** Superseded by RFCs MK-021–MK-037 — workflow-centred redesign series
**Tracks.** Strategy picker + per-rule strategy-specific fields.
**Touches.** `screens/settings.rs` (strategy_form), `shell/right_inspector.rs`

## Implementation summary

| Acceptance criterion | Met? | Where |
|---|---|---|
| Users can understand the active rule selection strategy | ✅ | Strategy pick-list in Settings → Strategy with `strategy.help()` text |
| Irrelevant strategy-specific fields hidden or disabled | ✅ | Inspector shows `weight` field only when `Strategy::WeightedRandom`, `priority` only when `Strategy::Priority` |
| Strategy changes produce reload feedback | ✅ | `SetSettingStrategy` triggers `app.simulate_save()` → ReloadHint reload-pending → banner |
| Trace detail does not leave users guessing why a rule won | ⚠️ Partial | Match detail shows matched rule + indices; does NOT show "strategy=weighted, weight=2 vs other matches" reasoning |

## Deferred

A future addition: per-event reasoning explainer in the match-detail
panel (e.g. "Matched rule #2 (weight=2); rule #4 also matched (weight=1)").
This requires engine support for richer trace events.
