# RFC MK-018 — Workflow-centred UX redesign

**Status.** Superseded by RFCs MK-021–MK-037 — workflow-centred redesign series
**Tracks.** Top-level UX architecture.
**Touches.** `shell/view.rs`, `shell/top_bar.rs`, `shell/left_rail.rs`, `screens/rule_builder.rs`, `screens/trace_strip.rs` (new), `screens/wizard.rs`, `screens/overview.rs` (removed), `selection.rs`

## Why this RFC exists

After shipping v0.2 and reviewing the screens against actual user workflow
(see [`docs/src/ux-redesign.md`](../../docs/src/ux-redesign.md)), the v0.1
information architecture revealed several mismatches with how the tool is
actually used:

1. Trace and rule editing were on separate tabs even though the user's
   core loop is *edit → send request → observe → iterate*.
2. The rule editor stacked WHEN above RESPOND vertically, hiding the
   response below the fold of any reasonable window height.
3. The Overview tab repeated information shown more efficiently in the
   top bar and command palette.
4. The wizard had 5 sequential steps for what is effectively one short
   form — most users hit "Next" four times without changing anything.

This RFC documents the redesign that addresses these issues, and the
specific items deferred to a future v0.4 RFC.

## Implemented changes

### 1. Removed Overview tab; Routes is now the default

- `WorkspaceTab::Overview` removed from the enum
- `screens/overview.rs` deleted
- Left rail dropped from 5 to 4 destinations: Routes / Trace / Scripts / Settings
- App opens directly to Routes when a workspace loads
- Command palette no longer lists "Go to Overview"

### 2. Two-column rule editor

The rule builder (`screens/rule_builder.rs`) now renders as:

```
┌─────────────────────────┬─────────┬──────────────────────────┐
│ WHEN — request matches  │    →    │ RESPOND — what to return │
│   URL path              │         │   ⦿ Inline  ○ Serve file │
│   Method                │         │   body editor            │
│   Headers               │         │   status + delay         │
│   Body conditions       │         │   [ Test rule ]          │
└─────────────────────────┴─────────┴──────────────────────────┘
```

The visual `→` divider reinforces the *if X then Y* mental model.
Both columns independently scroll.

### 3. Live Trace strip alongside Routes

New `screens/trace_strip.rs` — a compact 260px-wide panel showing the
most recent 15 events (newest first). Each row carries:

- Outcome glyph (✓ matched, ↩ fallback, ◯ miss, ! error)
- Method + URL path
- Timestamp + duration
- A ⟲ button to "Replay as test input" without leaving Routes

The strip lives in the right column of the Routes tab (when toggled on).
When the strip is hidden, the right inspector takes its place. The
toggle is the new `∿/∿●` button in the top bar (`Message::ToggleTraceStrip`).

Default state: **visible**. Users who want the full editor width can hide it.

The standalone Trace tab is retained for full-screen drill-in but is
no longer the primary trace surface.

### 4. Single-page wizard

`screens/wizard.rs` was rewritten from a 5-step sequential flow into one
scrolling form:

- **Required fields** (Workspace name, Parent folder) — always visible at top
- **Server** section — defaults visible inline, user can change inline
- **Starter content** section — radio + checkboxes
- **Trace** section — toggle + queue size

A sticky action bar at the bottom carries `Cancel` and `Create`.
`Message::WizardBack` / `WizardNext` are marked `#[allow(dead_code)]` but
retained for backward compatibility.

True per-section collapse-on-click is documented as a v0.4 follow-up
(needs per-section state, which is a small addition).

## Deferred to v0.4 (will be a separate RFC)

- **Auto-save with restart-needed-only prompt.** Removes the four-step
  save flow (dirty dots → Save button → save-diff drawer → reload banner).
- **Workspace switcher in top bar.** Replaces the standalone Dashboard
  screen with an inline dropdown listing recent workspaces.
- **"Remember last workspace" on launch.** Skip the Welcome screen when
  a workspace was open in the last session.
- **Keyboard shortcut hints in command palette.** `Esc` and `⌘K` exist
  but are not discoverable.
- **True collapsible sections in wizard.** Section open/close state.

## Measurement claim

v0.2: editing one rule and verifying it took at least 2 tab switches
(Routes → Trace → Routes).

v0.3: the same loop takes zero tab switches when the trace strip is
visible. The user can edit the rule, send a request from another window,
and see the new trace event appear in the strip alongside the editor.

## Visual diff summary

| Surface | v0.2 | v0.3 |
|---|---|---|
| Default tab on workspace open | Overview | Routes |
| Left rail destinations | 5 | 4 |
| Rule editor layout | 5 cards stacked vertically | WHEN \| → \| RESPOND two-column |
| Trace visibility while editing | Hidden (separate tab) or in bottom drawer | Persistent right strip on Routes |
| Wizard steps | 5 sequential | 1 page with sections |
| Top bar action group | save / reload / restart / start-stop / palette / locale | + trace-strip toggle |
