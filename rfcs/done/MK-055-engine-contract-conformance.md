# RFC MK-055 — Engine contract conformance

**Status.** Implemented (Unreleased)
**Tracks.** Stabilization roadmap M7 — Engine contract conformance.
**Touches.** `crates/model` dev-dependencies and conformance tests, the
`WorkspacePort` mapping types, the `ReferenceGap` inventory, the matcher-oracle
guard, and architecture/model documentation.

## Summary

Verify the MK-053 editing boundary against the real `apimock-config` crate
instead of against a prose reference. Implementation adopts `apimock-config`
5.10.0 as a **test-only dev-dependency**, adds a conformance suite that drives a
real `Workspace` on disk in a temporary directory, classifies every divergence
found, corrects our side where our side is wrong, and reduces the
`ReferenceGap` inventory to what the artifact genuinely does not establish.

This RFC changes no production code path, adds no production dependency, and
introduces no filesystem, process, or network access to any non-test target.
`MemoryWorkspace` remains the application's workspace implementation. M7 proves
the mapping is faithful; it does not build the production adapter.

## Problem and motivation

MK-053 was designed on the premise that no reproducible engine artifact was
available. `docs/src/architecture.md` and `crates/model/README.md` both record
that the evidence proves "the local mapping and in-memory contract suite only."

That premise was true only of version 5.10.1, which was never published.
`apimock-config` 5.10.0 has been on crates.io since 2026-05-16 with MSRV
1.91.0 — matching this workspace exactly — and exposes the complete
`Workspace::load / snapshot / apply / validate / save / list_directory` surface
plus the `EditCommand` enum.

The programme therefore applied two different standards to the same engine
family. M2 adopted `apimock-routing` 5.10.0 as an executable oracle for Test
Rule, and that was independently accepted. M3 declined an executable oracle for
the editing boundary and produced a designed mapping instead. Architecture
finding B2 is consequently closed against our own design rather than against the
contract we must integrate with, and R1 cannot honestly verify it as it stands.

### Observed divergences — preliminary, not exhaustive

Reading `apimock-config` 5.10.0 sources during design already contradicts the
5.10.1 reference in three places, and two of them contradict our implementation:

| Surface | 5.10.1 reference | `apimock-config` 5.10.0 | Our model |
|---|---|---|---|
| `RespondPayload.status` | `Option<String>` (`"200 OK"`) | `Option<u16>` | `String`, default `"200 OK"` |
| `RespondPayload.delay_milliseconds` | `Option<u64>` | `Option<u32>` | `u64` |
| `EditCommand::AddRuleSet.path` | `PathBuf` | `String` | `RuleSetPath` wrapping a workspace-relative string |
| `EditCommand` | (not stated) | `#[non_exhaustive]` | — |

The status divergence is material: a free-text `"200 OK"` cannot be handed to a
`u16` field without a lossy parse, and our respond editor currently accepts
values the engine cannot represent. The delay divergence permits values above
`u32::MAX` that the engine will reject.

`RulePayload`'s three-state `Option<Vec<_>>` semantics, by contrast, match both
the reference and our `ConditionEdit::into_reference_option`. Part of the
mapping is right; the point of M7 is to find out which part, by execution rather
than by reading.

These findings are design-time evidence that the milestone is warranted. They
are not a complete inventory, and implementation must not treat them as one.

## Goals

1. Establish an executable oracle for the editing boundary, as M2 did for
   matching.
2. Determine, by execution, which parts of the MK-053 mapping are faithful.
3. Correct our side wherever it is wrong against the artifact.
4. Record every remaining divergence explicitly, with its reason.
5. Reduce `ReferenceGap` to what the artifact genuinely does not establish.
6. Leave R1 able to verify B2 against engine behavior.

## Non-goals

- Building the production filesystem adapter. `MemoryWorkspace` remains the
  application's implementation.
- Adding any production dependency, file I/O, subprocess, or socket.
- Replacing, deleting, or re-architecting `WorkspacePort`. This RFC tests it and
  corrects defects; a structural redesign would be an MK-053 amendment.
- Adopting 5.10.1 or any later version. 5.10.0 is the adopted artifact.
- Module splitting or test reorganization; that remains M5.
- Trace, subprocess, Rhai, persistence, or external-edit work.

## Decision

### 1. Adopt `apimock-config` 5.10.0 as a test-only dev-dependency

Added to `crates/model` under `[dev-dependencies]` only. A production
dependency is out of scope and would breach the mockup boundary.

The resolved graph must be verified before the suite is written: `apimock-config`
5.10.0 declares `apimock-routing >= 5.6.0`, and the workspace already pins
`apimock-routing` 5.10.0 through the MK-052 oracle contract. Implementation must
confirm a single resolved `apimock-routing` package, not two. If Cargo resolves
a second copy, stop and report rather than working around it.

No sign-off beyond acceptance of this RFC is required for the adoption itself.
The precedent is heavier in the other direction: M2 adopted `apimock-routing` as
a **production** dependency on its RFC alone. Same author, same Apache-2.0
licence, same 5.10.0 line, MSRV-compatible, and dev-only is strictly less
invasive.

One consequence is recorded in advance. `cargo audit` scans `Cargo.lock`, which
includes dev-dependencies, so pulling in `toml`, `indexmap`, `thiserror`,
`console`, `log`, and `tempfile` may grow the allowed-warning inventory. That is
expected and is not a gate regression; a **vulnerability** in the added surface
would be, and blocks under the standing MK-054 policy.

### 2. Add a sibling oracle guard

Add `scripts/check-engine-oracle.sh` and its self-test, pinning `apimock-config`
version, registry source, checksum, and activated features on the same terms
`scripts/check-matcher-oracle.sh` applies to `apimock-routing` and `http`. A
compatible lockfile bump must not silently change the oracle, exactly as MK-052
established. The canonical gate calls both checkers.

A sibling rather than an extension, because `check-matcher-oracle.sh` is named
for the matcher contract and `apimock-config` is not a matcher; renaming it to
something broader would break the references in M2, M3, and M4's accepted
evidence. Do **not** factor shared logic into a common helper: that is an
abstraction for two consumers in a repository whose remaining life is a handful
of steps. Accept the modest duplication.

### 3. Conformance harness

Tests construct a real workspace on disk in a temporary directory and drive it
through `Workspace::load`. `Workspace::load` resolves either a config file or a
directory containing `apimock.toml`, and loads through the same `Config::new`
path the running server uses, so fixtures must be real TOML the server would
accept.

A `tempfile` dev-dependency is permitted for this purpose. Temporary
directories must be created per-test and cleaned up by scope; no test may write
outside its own temporary directory, and none may depend on execution order.

### 4. What conformance means

This is the crux and is decided here rather than left to implementation.
Conformance has two tiers, and both are required.

**Tier 1 — mapping totality (exhaustive).** For every `WorkspacePort` type that
corresponds to an engine type, a conversion to the engine type exists, is total
over our type's domain, and is tested. Where the conversion is not total —
`status: String` to `Option<u16>` is the known example — the partiality is
enumerated, tested at its boundary, and recorded as an accepted divergence with
its handling rule. "It usually works" is not a result.

**Tier 2 — behavioural equivalence (representative).** For a defined set of
scenarios, the same logical edit applied to both `MemoryWorkspace` and a real
`Workspace` produces equivalent observable state. Full behavioural equivalence
across the entire command surface is not required; the scenario set must cover
at least:

- add / update / delete / move rule, including `MoveRule` index semantics;
- `UpdateRule` three-state `Option<Vec<_>>` preserve, clear, and replace, for
  both header and body conditions;
- per-condition add / update / remove addressed by `NodeId`;
- `UpdateRespond` across both inline-text and serve-file modes, including the
  status and delay boundaries above;
- `UpdateRootSetting` for at least one variant of each `EditValue` shape;
- `AddRuleSet` / `RemoveRuleSet`, including that removal does not delete the
  file from disk;
- apply-error paths, including the RFC 013 `url_path: None` with
  `url_path_op: Some(_)` rejection;
- `save()` producing the expected `DiffItem` set, and being a no-op when clean;
- `ApplyResult.changed_nodes` for a representative edit, correlated to the nodes
  our port reports as changed;
- `ReloadHint` restart-versus-reload classification across the `RootSettingKey`
  groups, against the hints our port produces;
- `Workspace::validate()` and its `ValidationReport`, against our diagnostics.

The last three exist because architecture finding B2 named
`ApplyResult.changed_nodes`, diagnostics, and reload hints specifically as
surfaces that were "not the actual mutation boundary." R1 will verify B2
sub-claim by sub-claim, so each needs its own scenario rather than incidental
coverage.

Equivalence is judged on observable state — resulting rules, conditions,
respond blocks, diagnostics, and dirty/save outcomes — not on internal
representation. Our `NodeId` values are session-scoped and will not equal the
engine's; correlate by position and content, never by identity.

### 5. Divergence disposition

Every divergence found gets exactly one disposition, recorded with evidence:

- **Our defect** — our side is wrong against the artifact. Correct our side.
  The status and delay findings above are expected to land here.
- **Accepted divergence** — intentional and UI-specific, with a documented
  conversion rule and a test proving the conversion. Requires a stated reason,
  not merely a note that it exists.
  - Correcting a mapping is in scope; changing the **editor's input affordance**
    is not. A Tier 1 totality test for `String` to `Option<u16>` cannot be
    written without deciding what happens to `"200 OK"`, so the conversion rule
    belongs here. Replacing the status combobox with a constrained numeric
    control, and its validation copy, is UX work with no UX review behind it and
    belongs with M6. This keeps M7 evidential: a failing test then means the
    mapping is wrong, not that new validation is wrong.
- **Design conflict** — an MK-053 **decision** is contradicted, as distinct from
  an input assumption being wrong. **Stop and raise an MK-053 amendment.** M7
  must not silently rewrite an accepted design to make it appear conformant.

The decision-versus-assumption boundary decides which disposition applies, so it
is stated precisely. MK-053 decided to map to the documented contract; the
documented contract was wrong about `status`. Our implementation faithfully
implemented a bad input, and correcting `String` to `u16` overturns no MK-053
reasoning — that is **our defect**. A genuine design conflict would be
discovering that the engine has no per-condition `NodeId` addressing at all,
which would invalidate MK-053's condition-identity design rather than one of its
inputs.

Without that distinction every divergence looks amendable and the rule loses
force. With it, the third disposition catches the case that matters: if
implementation finds itself reshaping the port to fit, that is a design
conflict, not an implementation detail.

### 6. ReferenceGap reconciliation

Each of the seven `ReferenceGap` entries in `docs/src/architecture.md` is
re-evaluated against the artifact and moved to one of:

- **closed** — the artifact establishes the behavior; adopt it and test it;
- **narrowed** — partially established; record precisely what remains open;
- **confirmed** — genuinely not established by 5.10.0; keep, with the reason
  restated in terms of the artifact rather than its absence.

Rule weight/priority, strategy seed/tiebreaker, and the condition payload shapes
are the entries most likely to close or narrow. Trace transport and filesystem
containment are expected to remain confirmed, since they are not
`apimock-config` concerns.

### 7. Documentation corrections

Correct the statements asserting no reproducible artifact exists, in
`docs/src/architecture.md`, `crates/model/README.md`, and
`docs/src/match-test-conformance.md`. The corrected text must state what is now
true: 5.10.0 is adopted and executable, 5.10.1 was never published, and the
production adapter remains unbuilt.

Add `cargo doc --workspace --no-deps --locked` to the canonical gate, per the
roadmap's recorded gate addition.

## Implementation sequence

1. Add the dev-dependency; verify single `apimock-routing` resolution on stable
   and Rust 1.91 before writing any test.
2. Extend the oracle guard and its self-test.
3. Build the fixture helper and one end-to-end `load → apply → snapshot` test to
   prove the harness before scaling it.
4. Implement Tier 1 mapping totality tests.
5. Implement Tier 2 scenario tests.
6. Classify divergences; correct our defects; raise any design conflict as an
   MK-053 amendment before proceeding.
7. Reconcile `ReferenceGap`; correct documentation; run the canonical gate.

Steps 1–3 should be submitted as one review checkpoint before steps 4–7, because
a harness that cannot load a fixture invalidates everything built on it.

## Acceptance evidence

- Resolved-graph output showing one `apimock-config` 5.10.0 and one
  `apimock-routing` 5.10.0, with crates.io sources and checksums, and no patch,
  Git, vendored, or alternate-registry override.
- Confirmation that no production target references `apimock-config` —
  a dependency inspection of non-dev targets, not an assertion.
- Tier 1 and Tier 2 test output.
- The divergence table, each row with its disposition and evidence.
- The reconciled `ReferenceGap` inventory with per-entry justification.
- Full `bash scripts/check-release-gates.sh` output, including the extended
  oracle guard.
- Explicit statement of which MK-053 decisions were confirmed by execution and
  which were not exercised.

## Alternatives considered

**Keep the designed mapping and defer verification to production.** Rejected.
It leaves B2 unverifiable at R1 and moves the discovery of mapping defects to
the point where they are most expensive. Two defects were already found by
reading alone.

**Adopt `apimock-config` as a production dependency and replace
`MemoryWorkspace`.** Rejected for M7. That is the production adapter, requires
filesystem I/O, and is deferred past R2 with its own threat modeling.

**Wait for a published 5.10.1.** Rejected. It does not exist, no release is
promised, and 5.10.0 is the version the programme already adopted for matching.

**Test against the engine's own test suite rather than writing ours.** Rejected
as insufficient. The engine's tests prove the engine; they say nothing about
whether our mapping onto it is faithful, which is the actual question.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| Divergences are numerous enough to invalidate MK-053's design | Disposition 3 requires stopping for an amendment rather than absorbing them silently |
| `apimock-config` pulls a second `apimock-routing` copy | Verify the resolved graph in step 1, before any test is written |
| Fixture TOML does not load, blocking the suite | Prove the harness end-to-end in step 3 before scaling |
| Test-only dependency drifts into production | Dev-dependency only; acceptance requires a non-dev dependency inspection |
| Scope creeps toward the production adapter | Non-goals are explicit; filesystem access confined to per-test temporary directories |
| Engine's own docs are internally inconsistent | Observed already: `UpdateRule`'s doc comment describes a payload shape the struct contradicts. Treat executable behavior as authoritative over any doc comment, and record where they disagree |

## Review questions and their resolution

All five were resolved by the project owner on 2026-08-01, and the decisions are
folded into the text above.

1. **Are the two conformance tiers the right contract, and is the Tier 2
   scenario set sufficient for R1 to verify B2?** Tiers accepted. The scenario
   set was **not** sufficient: `ApplyResult.changed_nodes`, `ReloadHint`, and
   `validate()` were named by B2 but appeared only as incidental prose. Three
   scenarios added.
2. **Is the three-way divergence disposition correct?** Accepted, with the
   design-conflict boundary sharpened to *a decision contradicted, not an input
   assumption wrong*, so the rule keeps its force.
3. **Should the status/delay corrections split out?** Split, along the
   mapping/affordance line rather than by leaving them whole in either
   milestone: M7 decides and tests the conversion rule, M6 changes the editor
   control and its copy.
4. **Does adopting the crate need sign-off beyond this RFC?** No — M2's
   production adoption of `apimock-routing` is the heavier precedent. The
   dev-dependency audit-surface consequence is recorded in decision 1.
5. **Extend the oracle guard or add a second checker?** Sibling checker, no
   shared-helper refactor.

## Status

Design accepted by the project owner on 2026-08-01, and implementation
authorized separately on the same date, assigned to the dev team. Authorization
covered the scope defined here and nothing beyond it.

Implementation was delivered in two reviewed stages — a harness checkpoint
(steps 1–3) and the full suite (steps 4–7) — accepted in
`.git-exclude/reviewed/2026-08-02-rfc-mk055-harness-checkpoint-review.md` and
`.git-exclude/reviewed/2026-08-02-rfc-mk055-implementation-review.md`, and
committed as `fec0fbf`. Nine divergences were classified, one was a genuine
defect and was corrected, and **no MK-053 decision was contradicted**, so
decision 5's design-conflict path was never taken and no amendment was required.

This RFC moved to `done/` at its lifecycle closure on 2026-08-02.
