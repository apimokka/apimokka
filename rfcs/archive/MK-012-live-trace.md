# RFC MK-012 — Live Trace panel and match-detail inspector

**Status.** Superseded by RFCs MK-021–MK-037 — workflow-centred redesign series
**Tracks.** Trace event list and detail.
**Touches.** `screens/trace.rs`, `screens/match_detail.rs`, `shell/bottom_drawer.rs`

## Implementation summary

| Acceptance criterion | Met? | Where |
|---|---|---|
| Users can observe live request outcomes | ✅ | Trace tab + bottom-drawer Live Trace mode |
| Each outcome type has clear detail view | ✅ | `match_detail.rs` handles all 4 outcome variants: matched / fallback / miss / error |
| Matched and fallback outcomes link back to relevant configuration | ✅ | "matched" outcome includes a button that fires `Message::SelectRule(rule_id)` |
| Dropped events and trace connection problems visible | ⚠️ Partial | Per-event `dropped_count` chip shown (▲N); no connection-state banner |

## Implementation notes

- Trace events are static `mock::sample_trace_events()` — 4 canned events covering all outcomes
- Filter input matches against method / path / outcome label
- Pause toggle preserves visible state (would gate channel reads in production)
- "Trace not connected" / "Max subscribers reached" banner copy exists as i18n keys (`TraceDisconnected`, `TraceMaxSubscribers`) but is not displayed yet
