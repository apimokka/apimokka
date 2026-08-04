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

RFC MK-055 adopted `apimock-config` 5.10.0 as a **test-only dev-dependency**
of `crates/model` (`crates/model/tests/engine_conformance/`), pinned by
`scripts/check-engine-oracle.sh`. 5.10.1 — the version the documented GUI
integration reference describes — was never published; 5.10.0 is the
reproducible artifact the programme has adopted, on crates.io since
2026-05-16 with MSRV 1.91.0, matching this workspace. The MK-053 mapping and
in-memory contract suite are now additionally verified by executing against
this real artifact, not designed against prose alone. `MemoryWorkspace`
remains the application's only implementation; no production dependency,
filesystem access, or production code path was added by this verification.
A future production adapter must still implement the same port, pass the
same behavior suite, and undergo its own explicit artifact/version/source
adoption review — M7 proves the mapping is faithful, it does not build that
adapter.

The contract distinguishes:

| Provenance | Meaning |
|---|---|
| Documented Reference | Behavior stated by the 5.10.1 integration reference |
| Verified Against 5.10.0 | Behavior confirmed by executing the real `apimock-config` 5.10.0 crate (RFC MK-055) |
| Local M3 Decision | Deterministic behavior required by this UI boundary without an engine-equivalence claim |
| Accepted Divergence | Confirmed different from 5.10.0, intentionally, with a documented conversion/translation rule |
| `ReferenceGap` | No usable reference command or executable artifact establishes the behavior |

Executing against 5.10.0 confirmed three places where the 5.10.1 prose
reference was simply wrong about the real payload shapes (`RespondPayload`
fields; see the accepted divergences below), and found two more divergences
the prose reference could not have revealed because it never showed the
affected fields at all: `BodyConditionPayload.value` is a mandatory
`serde_json::Value` where our model's `Option<Value>` assumed an optional
one, and `RootSettingKey::ServiceStrategy`/`LogFormat` require lowercase
snake_case wire values (`"first_match"`, `"text"`, …) where our canonical
values match this UI's own PascalCase/`"plain"` labels. None of these
contradict an MK-053 **decision** — MK-053's per-condition `NodeId`
addressing, `Option<Vec<_>>` preserve/clear/replace semantics, apply-error
handling, and undo/redo ownership were all exercised against the real engine
and held. No MK-053 amendment was required.

### Accepted divergences (confirmed against `apimock-config` 5.10.0)

| Surface | Our canonical value | Engine's actual value | Handling rule |
|---|---|---|---|
| `RespondDefinition.status` | Validated `"<3-digit code>[ reason]"` string | `Option<u16>` | Total: parse the leading 3 digits; the reason phrase has no engine representation and is dropped |
| `RespondDefinition.delay_milliseconds` | `u64`, now range-checked to `0..=u32::MAX` (RFC MK-055 correction to `map_response`) | `Option<u32>` | Values above `u32::MAX` are rejected at mapping time — our defect, fixed; see `crates/model/src/workspace_port/mapping.rs` |
| `EditCommand::AddRuleSet.path` | `RuleSetPath` (validated workspace-relative, `.toml` suffix) | `String` | Total: `RuleSetPath::as_relative().as_str().to_owned()`; every value our type admits is already a valid engine string |
| `BodyConditionPayload.value` | `Option<serde_json::Value>` (`None` for `Exists`/`Absent`) | Mandatory `serde_json::Value` | `None` → `Value::Null`. Safe by construction of our own type, not merely by observed engine behavior: `BodyCondition::expected()` returns `None` **only** for `Exists`/`Absent`, so the substitution is unreachable for any operator that would read the value. (The engine's presence matchers also do not read this field — confirmed by execution.) Candidate default for a production adapter, proven only for the presence operators; an adapter that widens where `None` can occur would need to revisit it |
| `RootSettingKey::ServiceStrategy` | PascalCase labels from `apimokka_model::settings::Strategy` (`"FirstMatch"`, …), sent verbatim by `app.rs` today | Lowercase snake_case (`"first_match"`, …); anything else is `ApplyError::InvalidPayload` | Not corrected in `map_root_setting`: MK-053 established the port as an **explicit UI mapping, not an isomorphic copy**, so carrying this UI's own vocabulary internally is correct by design, not a deferred defect. What is missing is a translation at a production-engine boundary that does not exist yet — the future production-adapter RFC owns that translation, the same way it owns every other engine-facing conversion this document records. (This is a narrower boundary than "no production code path changes": the `delay_milliseconds` correction above does change one, because it is a contained validation tightening with no caller-visible ripple. This change would instead propagate through `app.rs`, `settings.rs`, and UI labels, which is what actually distinguishes the two cases.) |
| `RootSettingKey::LogFormat` | `"plain"` (this UI's default) | `"text"` / `"json"` | Same reasoning and same disposition as `ServiceStrategy` |
| `RootSettingKey::ListenerPort` | Rejects `0` (existing test: `root_mapping_rejects_type_range_and_enum_errors`) | Accepts `0..=65535` | We are stricter than the engine, never more permissive; no correction needed |
| `EditCommand::AddRuleSet` file existence | `MemoryWorkspace` has no filesystem; the intent always succeeds if the parent exists | Requires the referenced file to already exist on disk and parse as a valid rule set | Structural, not a mapping defect: a production adapter's `AddRuleSet` needs a real pre-flight file check ours cannot model |
| `ApplyResult.changed_nodes` for `AddRule` | Reports parent + new rule (2) — `WorkspaceNodeKind` has no `Respond` variant | Reports parent + new rule + new respond (3) — the engine addresses each rule's respond block by its own `NodeKind::Respond` id | Both report the parent and the new rule; the engine's extra entry reflects an addressable node our port does not expose. A production adapter needs to track both ids per rule |

## Reference gap inventory

These gaps are deliberately outside canonical workspace edits. Each was
re-evaluated against `apimock-config` 5.10.0 by RFC MK-055.

| Surface | Disposition | Current treatment | Evidence from 5.10.0 |
|---|---|---|---|
| Rule weight and priority | **Confirmed** | Typed app-owned prototype state; no port edit, dirty file, save effect, or runtime hint | `apimock_config::RulePayload` still has no `weight`/`priority` field (`view.rs:321-332`); genuinely not established by this artifact |
| Strategy seed and priority tiebreaker | **Confirmed** | Only the five visible strategy labels map through the port | `cmd_update_root_setting`'s `ServiceStrategy` arm hardcodes `seed: None` and `tiebreaker: PriorityTiebreaker::FirstMatch` for every request (`workspace/edit.rs:557-573`); no `EditValue` shape can set either parameter |
| Trace settings and transport | **Confirmed** | Session-scoped prototype state and canned trace events; no socket/subprocess implementation | `apimock-config` 5.10.0 has no trace module at all (its `lib.rs` exports `config`, `error`, `path_util`, `view`, `workspace` only); trace remains `apimock-server`'s concern, outside this artifact entirely |
| Dotted body paths for object keys containing dots | **Confirmed** | Dotted mini-syntax only; such keys cannot be addressed | `BodyConditionPayload.path` is a raw `String` with no escaping scheme defined at this layer; matching semantics (and any future escaping) remain `apimock-routing`'s concern, covered by the MK-052 oracle, not `apimock-config` |
| Body/header condition payload shapes omitted by the reference | **Closed** (headers) / **Narrowed** (body) | Local typed mapping, including optional presence values and JSON conversion | `HeaderConditionPayload { name, op, value: Option<String> }` matches our provenance table exactly — closed. `BodyConditionPayload { kind, path, op, value: serde_json::Value }` is now known but its `value` field is mandatory where ours is optional — narrowed to the one documented accepted divergence above, not an open unknown |
| Workspace-relative lexical paths | **Narrowed** | Platform-independent UTF-8 component grammar, with no host path resolution | `cmd_add_rule_set` resolves via `Path::new(&relative_dir).join(&path)` on the raw string, with no canonicalization, `..`-rejection, or collision handling of its own (`workspace/edit.rs:139`) — the engine performs no lexical validation at all at this layer, so our stricter grammar is not redundant; engine/platform collision behavior for a real filesystem remains undefined by this artifact |
| Symlinks and filesystem containment | **Confirmed**, with stronger evidence | No filesystem exists in `MemoryWorkspace` | `apimock-config` 5.10.0 does not resolve symlinks or reject traversal/escape itself — `resolve_root` (`workspace/path_helpers.rs:23-41`) uses plain `Path::is_file`/`is_dir`, and `cmd_add_rule_set` joins the untrusted path string directly. A production adapter cannot rely on the engine for this; MK-053 section 3's rule (validate components before joining to an opened root handle) is confirmed necessary, not precautionary |

Reference-gap values must not be described as persisted engine state. A future
artifact may close a gap only through a fresh mapping/adoption decision; it
must not silently change the local contract.

### Named follow-ups for the production-adapter RFC

RFC MK-055's implementation review identified three items that a future
production-adapter RFC must inherit explicitly rather than rediscover:

1. **Path containment and traversal protection.** `apimock-config` 5.10.0
   provides none (see "Symlinks and filesystem containment" above). A
   production adapter must implement MK-053 section 3's rule (validate
   components, then join to an opened root handle) itself; this is now an
   evidence-backed security requirement, not a precaution.
2. **`ServiceStrategy`/`LogFormat` wire-vocabulary translation.** The adapter
   must translate this UI's PascalCase/`"plain"` values to the engine's
   snake_case/`"text"` values at its own boundary (see the accepted
   divergences above). Without this translation, every strategy and
   log-format change fails silently at the engine boundary.
3. **`ValidationReport` diagnostic-content comparison.** RFC MK-055 verified
   only that a clean workspace validates cleanly on both sides; it did not
   compare diagnostic *content* for a workspace with real issues. Since the
   GUI surfaces engine diagnostics directly to users, content fidelity
   (not just presence/absence) is a real production-adapter concern and
   should be verified before that adapter ships.

## Internationalization

`crates/i18n/src/keys.rs` defines the `Key` enum. English and Japanese locale
modules match every key, so missing match arms are compile errors. The app uses
`Tr { locale }` to resolve static display strings and changes locale through a
`Message`.

## Source-size baseline

The project guideline is to aim below 300 lines per Rust file. Files above the
500-line signal threshold are enumerated and checked for a recorded boundary
decision by `scripts/check-source-size.sh` (RFC MK-057) — run it for the
current inventory rather than reading a transcribed count here, which is the
staleness this checker replaces.
