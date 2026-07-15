# apimokka — Roadmap

This file tracks deferred decisions and future development candidates. Items that
have been implemented are reflected in CHANGELOG.md and in `rfcs/done/`.

---

## Implemented (through v0.9.15)

All RFCs MK-021 through MK-046 are implemented. See `rfcs/done/` for design
records and CHANGELOG.md for the release history.

Notable capabilities:
- Full Routes workbench: WHEN/RESPOND editing, per-rule weight/priority,
  strategy dropdown, validation issues strip
- Trace screen: live filter, outcome-aware match detail, jump-to-rule/file,
  dropped-count warning
- Audience modes: Guided (inline hints, layout density) and Expert; first-run
  picker, Settings toggle
- Undo / redo: typed command log (delete, add, move, URL-path edit); ⌘Z / ⌘⇧Z
- Bottom drawer: validation grouped by rule set with jump navigation; save-diff
  with rule summaries
- MK-038 two-buffer fallback file lifecycle: explicit save, revert, JSON validity
- Command palette: all 17 commands wired (including Undo / Redo)
- First-launch flow: Welcome screen first, mode picker full-screen before workspace
- Snora Design (snora 0.25): four WCAG-AA contrast-tested themes including High
  Contrast Light/Dark; high-contrast borders on cards/panels (MK-050)

---

## Deferred — needs persistence layer

These require reading/writing files or persisting state across sessions, which
the mockup does not support.

| Feature | Notes |
|---|---|
| "Remember last workspace" on launch | Would read a preferences file on startup |
| Persist theme / locale / audience mode | Same preferences file |
| File watcher (external edit detection) | `notify` crate; reload workspace on change |
| Real workspace I/O | Connect to `apimock_config::Workspace` API |

---

## Deferred — v2 candidates

| Feature | Rationale |
|---|---|
| Transition animations on dialogs | iced Animated API; not stable in 0.14 |
| Compact / comfortable density toggle | snora `Density::Compact` is deferred upstream; would wire to a per-user preference |
| Drag-and-drop rule reordering | Requires custom mouse-event handling; drag handles present |
| Screen-reader / ARIA support | iced accessibility API is experimental |
| Scripts tab (Rhai editor) | Needs syntax highlighting + live error surfacing; MK-031 withdrawn |

---

## Testing approach

The project uses **plain `#[test]` unit + smoke tests**. No `iced_test` dependency.

Rationale: `iced_test` 0.14 is built around a rendering `Simulator` with PNG
snapshot hashing. Its selector/interaction API is still maturing. The
highest-value tests here exercise the pure `App::update` reducer and MK-038
lifecycle — no rendering needed.

99 tests at v0.10.0 (92 app + 7 model): selection/accordion invariants,
MK-038 two-buffer lifecycle, undo/redo round-trips, audience mode behaviour,
layout density toggles, strategy/weight/priority round-trips, trace filter,
jump actions, first-launch flow, view smoke tests for every screen and every
centre-panel branch.

If `iced_test` matures, a small set of interaction-level smoke tests could be
added. It is intentionally not a dependency today.
