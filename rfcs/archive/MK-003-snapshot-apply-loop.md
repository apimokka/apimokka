# RFC MK-003 — Snapshot-apply loop and in-memory workspace adapter

**Status.** Superseded by RFCs MK-021–MK-037 — workflow-centred redesign series
**Tracks.** State management / data layer.
**Touches.** `crates/apimokka-model/src/mock.rs`, `crates/apimokka-app/src/app.rs`

## Summary

The mockup simulates the engine's `Workspace::apply(EditCommand) → ApplyResult`
contract without real file I/O. The in-memory adapter:

1. Holds a `WorkspaceSnapshot` that mirrors the engine's read-only snapshot.
2. On each `EditCommand`-equivalent `Message`, mutates the snapshot directly
   (in production this would call `Workspace::apply` then re-snapshot).
3. Sets `file.dirty = true` on the owning rule-set file after any rule edit.
4. Re-runs a trivial `rebuild_validation()` after each edit to simulate the
   engine's post-apply `ValidationReport`.

## Save flow simulation

`App::simulate_save()`:
1. Clears all `dirty` markers.
2. Builds a `SaveResult` with `DiffItem` rows and a `ReloadHint`.
3. Sets `server_state` to `ReloadPending` or `RestartRequired` based on
   which `RootSettingKey` values changed.
4. Opens the `SaveDiff` bottom drawer.

The `ReloadHint` logic mirrors the engine reference: listener/TLS/log-file
keys require restart; all others require reload.

## Mock data

`apimokka_model::mock::shop_api_mock()` returns a pre-built `WorkspaceSnapshot`
matching the external design §36 mock workspace:
- Two rule sets (`rules/main.toml` dirty=true, `rules/error-scenarios.toml`)
- Three fallback files with route hints
- Two middleware scripts
- One deliberate `$.user.id` JSONPath-syntax body condition for validation demo
- Strategy: `WeightedRandom` so the "rule without weight" warning fires
