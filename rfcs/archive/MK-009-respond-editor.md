# RFC MK-009 — Respond editor and response preview

**Status.** Superseded by RFC MK-021..MK-037 series (workflow-centred redesign)
**Tracks.** Respond block editing.
**Touches.** `screens/rule_builder.rs` (respond_card)

## Implementation summary

| Acceptance criterion | Met? | Where |
|---|---|---|
| Exactly one response source active after applying changes | ✅ | `RespondMode` enum (`InlineText` vs `ServeFile`) is exclusive; tab switch changes mode |
| Status and delay visible in rule summaries | ⚠️ Partial | Status/delay shown in editor; not aggregated into the rule-tree summary string |
| Response edits dispatch structured respond payloads | ✅ | `RespondSetText`/`SetFilePath`/`SetStatus`/`SetDelay` → `update_respond()` |
| Users can preview response effect before applying | ❌ | **Not implemented** — no preview panel |

## Deferred

The response preview surface (RFC §"Mockup Surface") is not built. It
would render a faux HTTP response — status line, headers, body — in a
side panel beside the editor. Tracked as a future addition; can use
the same `RespondPayload` snapshot.
