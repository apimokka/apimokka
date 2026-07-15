# Architecture

This page describes the current v0.10.0 workspace. Historical design documents
and RFC bodies remain useful decision records but are not inventories of the
live source tree.

## Workspace and dependencies

```text
crates/app (apimokka-app)
  ├── crates/model (apimokka-model)
  ├── crates/i18n (apimokka-i18n)
  ├── iced 0.14
  └── snora 0.25

crates/model
  ├── serde_json
  └── uuid

crates/i18n
  └── no external dependencies
```

The root manifest requests snora 0.25 from crates.io; the lockfile currently
resolves the snora family to 0.25.2. No `vendor/` tree is part of the current
workspace.

## Application structure

`crates/app/src/app.rs` owns the central `App` state and message reducer.
`message.rs` defines user and system events, `selection.rs` holds navigation
selection types, and `theme.rs` maps theme choices to iced/snora styling.

Views are split between:

- `screens/` for mode selection, welcome, dashboard, wizard, routes, trace,
  settings, scripts, and dialogs;
- `shell/` for the workspace top bar, rail, tabs, body composition, and bottom
  drawer;
- `widgets/` for shared view helpers.

The executable starts at the mode picker. The user then reaches Welcome and may
open Dashboard, start the wizard, or enter a workspace. Workspace tabs dispatch
to Routes, Trace, Settings, or Scripts views.

## Data and message flow

```text
user input
  → iced dispatches Message
  → App::update mutates mock state
  → validation/save/trace state is recomputed as required
  → App::view selects a screen and composes shell/dialog/drawer views
```

This repository remains a UI/UX mockup: workspace edits are in-memory and there
is no live apimock-rs server integration. The stabilization roadmap governs work
to establish those production boundaries.

## Internationalization

`crates/i18n/src/keys.rs` defines the `Key` enum. English and Japanese locale
modules match every key, so missing match arms are compile errors. The app uses
`Tr { locale }` to resolve static display strings and changes locale through a
`Message`.

## Source-size baseline

The project guideline is to aim below 300 lines per Rust file and split files
above 500 lines. A 2026-07-15 physical line count identified the principal
exceptions:

| File | Lines |
|---|---:|
| `crates/app/src/app.rs` | 3,665 |
| `crates/app/src/screens/routes.rs` | 1,519 |
| `crates/model/src/mock.rs` | 513 |
| `crates/app/src/theme.rs` | 420 |
| `crates/i18n/src/keys.rs` | 412 |
| `crates/i18n/src/en.rs` | 406 |
| `crates/i18n/src/ja.rs` | 386 |
| `crates/app/src/screens/trace.rs` | 356 |
| `crates/app/src/shell/bottom_drawer.rs` | 327 |

These are recorded facts, not exceptions to the guideline. Structural
remediation is scheduled for roadmap milestone M5.
