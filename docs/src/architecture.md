# Architecture

This page describes the current v0.10.0 workspace. Historical design documents
and RFC bodies remain useful decision records but are not inventories of the
live source tree.

## Workspace and dependencies

```text
crates/app (apimokka)
  ├── crates/model (apimokka-model)
  ├── crates/i18n (apimokka-i18n)
  ├── iced 0.14
  └── snora 0.25

crates/model
  ├── http 1.4
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
  → App::update maps configuration messages to typed EditIntent values
  → WorkspaceSession dispatches an atomic EditTransaction to WorkspacePort
  → the returned complete PortSnapshot is validated and adopted
  → selection, diagnostics, history, dirty state, and runtime phase reconcile
  → App::view selects a screen and composes shell/dialog/drawer views
```

This repository remains a UI/UX mockup: workspace edits are in-memory and there
is no filesystem adapter, file watcher, merge implementation, trace transport,
helper subprocess, or live apimock-rs server integration.

## Workspace boundary and ownership

RFC MK-053 establishes `crates/model/src/workspace_port.rs` as the
application-facing configuration boundary. It is an explicit UI mapping, not
a source- or binary-isomorphic copy of an unavailable engine crate.

`MemoryWorkspace` owns canonical workspace state. `WorkspacePort::apply`
accepts a nonempty ordered transaction and commits it atomically. Apply, save,
and runtime acknowledgement return a complete `PortSnapshot`. Each snapshot
contains:

- a legacy `WorkspaceSnapshot` render projection;
- one ordered canonical `PortRuleView` per rendered rule, including stable
  condition identities;
- a sorted workspace dirty vector; and
- separate unsaved and saved-but-runtime-pending effects.

`WorkspaceSession` in the app owns the admitted workspace identity, snapshot
adoption, drafts, semantic undo/redo, saved configuration revision, and
post-adoption contract-fault state. Every returned snapshot is checked for
node uniqueness and complete canonical/render correlation. A post-attempt
structural or identity fault still adopts the returned state, clears unsafe
draft/history state, and makes the session read-only.

The app owns presentation concerns that are outside the port:

- route, fallback, script, and condition-focus selection reconciliation;
- `ReferenceGap` prototype values such as rule weight/priority and trace
  controls;
- durable snapshot diagnostics versus transient operational problems;
- runtime request correlation and acknowledgement; and
- the historical typed Global Save report combining workspace progress with
  ordered, per-item-atomic fallback attempts.

The older `edit_command`, `save`, and render payload modules remain local
mockup vocabulary. They are not the authoritative mutation/save boundary, and
the lossy render projection is never used to reconstruct canonical history.

## Contract provenance

The intended production semantics come from the documented apimock-rs 5.10.1
GUI integration reference. A reproducible `apimock-config` 5.10.1 artifact is
not available, so the current evidence proves the local mapping and in-memory
contract suite only. A future production adapter must implement the same port,
pass the same behavior suite, and undergo an explicit artifact/version/source
adoption review.

The contract distinguishes:

| Provenance | Meaning |
|---|---|
| Documented Reference | Behavior stated by the 5.10.1 integration reference |
| Local M3 Decision | Deterministic behavior required by this UI boundary without an engine-equivalence claim |
| `ReferenceGap` | No usable reference command or executable artifact establishes the behavior |

## Reference gap inventory

These gaps are deliberately outside canonical workspace edits:

| Surface | Current treatment | Future adoption requirement |
|---|---|---|
| Rule weight and priority | Typed app-owned prototype state; no port edit, dirty file, save effect, or runtime hint | Adopt an explicit engine command/payload and persistence semantics |
| Strategy seed and priority tiebreaker | Only the five visible strategy labels map through the port | Establish parameter types, defaults, and effects from an executable artifact |
| Trace settings and transport | Session-scoped prototype state and canned trace events; no socket/subprocess implementation | Define command ownership and independently design transport/lifecycle behavior |
| Dotted body paths for object keys containing dots | Dotted mini-syntax only; such keys cannot be addressed | Adopt an escaping or structured path representation |
| Body/header condition payload shapes omitted by the reference | Local typed mapping, including optional presence values and JSON conversion | Reconcile against the real payload types and rerun the mapping suite |
| Workspace-relative lexical paths | Platform-independent UTF-8 component grammar, with no host path resolution | Define engine/platform collision behavior and root-handle resolution |
| Symlinks and filesystem containment | No filesystem exists in `MemoryWorkspace` | Production adapter must resolve roots/parents, reject traversal and resolved escapes, and receive separate security review |

Reference-gap values must not be described as persisted engine state. A future
artifact may close a gap only through a fresh mapping/adoption decision; it
must not silently change the local contract.

## Internationalization

`crates/i18n/src/keys.rs` defines the `Key` enum. English and Japanese locale
modules match every key, so missing match arms are compile errors. The app uses
`Tr { locale }` to resolve static display strings and changes locale through a
`Message`.

## Source-size baseline

The project guideline is to aim below 300 lines per Rust file and split files
above 500 lines. The M3 implementation materially changed the 2026-07-15
baseline, so the current physical counts were refreshed on 2026-07-22 before
M5 planning:

| File | Lines | Kind |
|---|---:|---|
| `crates/app/src/app.rs` | 4,363 | Implementation |
| `crates/app/src/app/workspace_session_tests.rs` | 2,216 | Test |
| `crates/app/src/screens/routes.rs` | 1,547 | Implementation |
| `crates/model/src/workspace_port/memory_tests.rs` | 1,463 | Test |
| `crates/app/src/app/workspace_session.rs` | 1,307 | Implementation |
| `crates/model/src/workspace_port/memory.rs` | 1,272 | Implementation |
| `crates/app/src/app/global_save_tests.rs` | 893 | Test |
| `crates/model/src/workspace_port.rs` | 878 | Implementation |
| `crates/app/src/app/runtime_tests.rs` | 840 | Test |
| `crates/model/src/mock.rs` | 537 | Implementation |
| `crates/model/src/workspace_port/tests.rs` | 530 | Test |
| `crates/app/src/shell/bottom_drawer.rs` | 526 | Implementation |
| `crates/model/src/workspace_port/mapping.rs` | 506 | Implementation |
| `crates/i18n/src/keys.rs` | 453 | Implementation |
| `crates/i18n/src/en.rs` | 449 | Implementation |
| `crates/i18n/src/ja.rs` | 429 | Implementation |
| `crates/app/src/theme.rs` | 420 | Implementation |
| `crates/app/src/app/tests.rs` | 381 | Test |
| `crates/app/src/screens/trace.rs` | 356 | Implementation |
| `crates/app/src/match_test/tests/body.rs` | 307 | Test |

These are recorded facts, not exceptions to the guideline. Structural
remediation is scheduled for roadmap milestone M5.
