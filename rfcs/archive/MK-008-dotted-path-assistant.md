# RFC MK-008 — Dotted-path assistant and sample-JSON selector

**Status.** Superseded by RFC MK-021..MK-037 series (workflow-centred redesign)
**Tracks.** Body-condition path entry helper.
**Touches.** `screens/dotted_path.rs`

## Implementation summary

| Acceptance criterion | Met? | Where |
|---|---|---|
| User can generate a valid dotted path from sample JSON | ✅ | `dotted_path.rs` extracts paths via `walk_object`/`walk_array` |
| UI explicitly discourages JSONPath syntax | ✅ | Inline warning `Key::DottedPathJsonpathWarn` when path starts with `$.` in rule builder; assistant title clarifies dotted-path syntax |
| Assistant does not persist sample request bodies by default | ✅ | `PathAssistantState` is in-memory only; closed on `PathAssistantClose` |
| Inserted paths participate in Rule Builder validation | ✅ | Insert dispatches `BodySetPath` which goes through the same path as manual entry |

## Implementation note

The JSON parser is a naive ~80-line walker, not a real JSON parser. It
handles the canned sample (objects, arrays, strings, numbers) but is
not robust to escaped characters or deeply nested structures. Production
should use `serde_json`.
