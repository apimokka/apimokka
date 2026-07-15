# RFC MK-013 — Replay and match-test workflow

**Status.** Superseded by RFCs MK-021–MK-037 — workflow-centred redesign series
**Tracks.** Dry-run rule testing from inside the GUI.
**Touches.** `screens/match_test.rs`, `screens/rule_builder.rs`, `screens/trace.rs`, `screens/match_detail.rs`, `app.rs::run_stub_match_test`

## Implementation summary

| Acceptance criterion | Met? | Where |
|---|---|---|
| Users can test a rule without leaving the GUI | ✅ | "Test rule" button at the bottom of the rule builder opens `screens/match_test.rs` dialog |
| Trace events can be used as dry-run inputs | ✅ | "Replay as test input" button on each trace row + in match-detail panel; fires `Message::ReplayAsTestInput(event_id)` which pre-fills the dialog |
| Results distinguish match, no match, and error | ✅ | `TestRuleResult::{Matched{rule_summary}, NoMatch, Error}` — each renders a distinct colour-free banner |
| UI does not misrepresent dry-run testing as real traffic | ✅ | `Key::TestRuleHint` explains "no real network traffic" |

## Implementation notes

The matcher (`App::run_stub_match_test`) is a simplified in-memory check
covering method + URL path (Equal / StartsWith / Contains / NotEqual).
It does NOT evaluate header or body conditions — those require the full
`apimock-routing` crate which is out of scope for the mockup.

The dialog pre-fills from the selected rule's `method` and `url_path`
on open, and from the trace event on replay.
