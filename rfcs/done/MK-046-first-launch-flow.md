# RFC MK-046 — First-launch flow

**Status.** Implemented (v0.9.15)
**Tracks.** App launch sequence: Welcome screen first, mode picker as the
opening question, workspace only after explicit navigation.
**Touches.** `app.rs` (init, view), `screens/mode_picker.rs`,
`screens/welcome.rs`, test helpers.
**Follows.** MK-025 (Welcome), MK-040 (audience modes).

## Problem

Since v0.9.1 the app initialises with `AppView::Workspace` and pre-loads the
mock snapshot. The first thing a user sees is the Routes workbench — which is
the right target screen for a returning user, but wrong for a first-launch.

The audience mode picker (MK-040) appears as a modal on top of the workspace,
so the user simultaneously sees mock rules in the background and a "how would
you like to be guided?" question in the foreground. This is incoherent: the
picker should be answered before the user has any context to be confused by.

## New launch sequence

```
App launch
   │
   ▼
audience_mode is None?
   │  Yes
   ├──▶  Mode picker (full-screen, no backdrop content)
   │        │  user chooses Guided or Expert
   │        ▼
   │     Welcome screen
   │        │  "Open workspace" → Dashboard → click workspace → Workspace
   │        │  "Create workspace" → Wizard → fill in → Workspace
   │
   │  No (returning user, mode already set)
   ▼
   Welcome screen  (or, if a "last workspace" preference existed, Workspace)
```

In the mockup (no persistence), every launch is a "first launch" — but the
picker shows only once per session (once the user has chosen, the mode is
stored in `App.audience_mode`).

## Implementation

- `App::new()` starts at `AppView::Welcome` with no snapshot loaded.
- `App::view()` gates: if `audience_mode.is_none()`, render
  `screens::mode_picker::view(self)` as the full-screen content (no snora
  overlay needed — the picker fills the screen with nothing behind it).
- The snora-dialog picker in `shell/view.rs` is kept as a fallback for the
  case where a user navigates back to Welcome mid-session (rare, but correct).
- Test helpers updated: `fresh()` and `expert()` now navigate to Workspace
  after choosing a mode (one line: `a.view = AppView::Workspace`).

## Acceptance criteria

- Fresh app (no mode set): first screen is the mode picker, full-screen.
- After choosing: Welcome screen appears.
- Welcome → "Open workspace" → Dashboard → click workspace → Workspace.
- Welcome → "Create workspace" → Wizard → Create → Workspace.
- Existing tests unchanged in behaviour (helpers updated to set view).
- Zero errors, zero warnings, all tests pass.
