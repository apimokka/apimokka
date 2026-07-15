# RFC MK-014 — Rhai middleware script surface

**Status.** Superseded by RFCs MK-021–MK-037 — workflow-centred redesign series
**Tracks.** Middleware script discovery and inspection.
**Touches.** `screens/scripts.rs`

## Implementation summary

| Acceptance criterion | Met? | Where |
|---|---|---|
| Users can find and inspect middleware scripts | ✅ | Scripts tab in left rail; script list in left column |
| UI explains when scripts are appropriate | ⚠️ Partial | Context-help panel lists `ctx.headers` / `ctx.method` / etc. APIs; no "when to use a script" rationale text |
| Read-only versus editable behavior is unambiguous | ✅ | Read-only banner `Key::ScriptsReadOnly` shown below stub viewer |
| Script surface does not obscure simpler file/rule workflows | ✅ | Scripts tab is the 4th of 5 destinations; not promoted on Welcome / Overview |

## Implementation notes

- The viewer shows hardcoded stub Rhai code, not actual file contents
  (the mockup has no file I/O)
- Two middleware scripts are listed (`middleware/auth.rhai`,
  `middleware/log.rhai`) from `mock::shop_api_mock()`
- Per the RFC §"Non-goals", editing is explicitly out of scope
