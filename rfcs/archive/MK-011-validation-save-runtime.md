# RFC MK-011 — Validation, diagnostics, save, and runtime action workflow

**Status.** Superseded by RFCs MK-021–MK-037 — workflow-centred redesign series
**Tracks.** Validation drawer, save flow, reload/restart banners.
**Touches.** `shell/bottom_drawer.rs`, `shell/view.rs`, `app.rs::simulate_save`, `app.rs::rebuild_validation`

## Implementation summary

| Acceptance criterion | Met? | Where |
|---|---|---|
| Users can tell whether changes are invalid / unsaved / saved / reload-pending / restart-required | ✅ | Top bar shows server state + save state with glyph+text; banner shows reload/restart prompts |
| Diagnostics visible inline AND in aggregated drawer | ✅ | Per-rule validation in inspector + workspace-wide in drawer (`Validation` mode) |
| Save results list affected files | ✅ | Save Diff drawer lists `DiffItem` paths with Created/Modified/Removed glyphs |
| Runtime action requirements clear and action-oriented | ✅ | Banner copy: "Server reload required" / "Server restart required" + dismiss button |

## Implementation notes

- `simulate_save()` produces a real `SaveResult` with `DiffItem` rows
- `ReloadHint` is computed by checking which `RootSettingKey`s changed
- Banner is dismissable via `DismissBanner` but reappears if changes
  remain unapplied
- Validation runs after every edit (cheap in mockup; would be debounced
  in production)
