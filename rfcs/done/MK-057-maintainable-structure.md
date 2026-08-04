# RFC MK-057 — Maintainable structure

**Status.** Implemented (Unreleased)
**Tracks.** Stabilization roadmap M5 — Maintainable structure.
**Touches.** `crates/app/src/app.rs`, `screens/routes.rs`, `model/mock.rs`, the
three implementation files carrying inline test bodies, the source-size record,
and a new structure checker.

## Summary

Bring the codebase within the project's file-organization rules without changing
behaviour, and replace the hand-maintained source-size record — which has gone
stale twice — with a generated one.

**Module boundary is the criterion; line count is only the surface signal that
tells you where to look.** A file is split because it holds more than one
responsibility, never because it crossed a number. Files grow through ordinary
maintenance, and a rule that fires on size alone would demand churn for its own
sake while missing a 200-line file that does three unrelated things.

## Problem and motivation

The project rule recommends splitting above 300 lines, strongly recommends it
above 500, and requires test modules to live in separate files. Measured at
`3c72374`:

- **8 implementation files and 7 test files exceed 500 lines.** The roadmap's M5
  scope names three, so the surface signal is roughly five times the named scope.
- **Three implementation files carry inline test bodies** — a flat rule
  violation, not a judgement call: `model/audience.rs:27`,
  `model/friendly_error.rs:91`, `model/mock.rs:416`.
- **The source-size table in `docs/src/architecture.md` has gone stale twice**,
  most recently at R1 (finding R1-2), where its top entry read 4,343 for a file
  that was 4,321 and MK-055's files were missing entirely.

Correcting a transcribed table at closure points does not work: the next commit
invalidates it. This is an instance of a general pattern — **transcribed facts
go stale, generated and checked facts do not** — and the fix should follow the
general rule, not patch the one table.

**Why now.** R1-1's closure means the canonical gate runs the full workspace test
surface on both toolchains. "Run the gate after every split" is therefore a real
guarantee rather than an aspiration, which it was not while the conformance
suite sat outside the gate. M5 would have been materially less safe a day ago.

## Goals

1. Eliminate the three inline test bodies.
2. Split files that hold more than one responsibility, at their real boundaries.
3. Record an explicit boundary decision for every file the surface signal flags.
4. Replace the stale table with a generated record.
5. Change no behaviour.

## Non-goals

- Splitting every file above a threshold on principle.
- Changing public crate boundaries, the `WorkspacePort` contract, message
  vocabulary, or any observable behaviour.
- Renaming types, reworking APIs, or improving code encountered during a move.
- UX or localization work; that is M6 / MK-056.

## Boundary audit

Performed at `3c72374` by inspecting structure, not line counts.

### `crates/app/src/app.rs` — 4,321 lines

The line count is not the finding. The structure is:

| Region | Lines | Content |
|---|---:|---|
| 37–477 | ~440 | State type definitions — `ThemeChoice`, `AppView`, `WizardState`, dialog states, `HistoryEntry`, `App` |
| **477–4035** | **~3,558** | **A single `impl App` block** |
| 4035–4181 | ~146 | Free helpers — history rebinding, mutation classification, subtree bindings |

**Correction (2026-08-04, slice 1 review).** An earlier draft of this audit
claimed fifteen `app/` sibling modules established the extraction pattern. That
was wrong, and it was the stated evidence for this mandate. Only **three** are
production siblings — `workspace_session`, `global_save`, `runtime` — of which
only `runtime.rs` extends `impl App` from a child module. The rest are
`#[cfg(test)] #[path = "app/X.rs"] mod tests_mkNNN;` test files whose filenames
read as production modules: `app/history.rs` is 198 lines, 8 `#[test]`
functions, zero `pub fn`. The precedent is `runtime.rs` alone, which is
sufficient and correct.

The residual `impl App` block is the domains nobody has pulled out yet, visible
in its own method names: root settings (`update_root_setting`), rule prototype,
rule core, response drafts, header drafts, body drafts.

**Additional finding (slice 1 review).** That the error was easy to make is
itself a defect worth fixing. Eight files under `app/` — `history`, `density`,
`trace`, `strategy`, `navigation`, `workspace_creation`, `rule_set_creation`,
`rule_duplication` — read as production modules and are pure test files reached
under different module names. Rename them to the `_tests.rs` suffix three
siblings already use (`workspace_session_tests.rs`, `runtime_tests.rs`,
`global_save_tests.rs`), or make the `mod` name match the file.

**Boundary:** the draft-editing domains. Extract them as siblings following the
existing pattern. The state types and free helpers may follow if they read
better separately, but the impl block is the defect.

### `crates/app/src/screens/routes.rs` — 1,542 lines

The cleanest case in the repository. Its functions already sit on the six
boundaries the roadmap named, at clean seams:

| Boundary | Functions | Lines |
|---|---|---|
| Sidebar | `left_sidebar`, `rule_set_group` | 24–318 |
| Rule-set configuration | `centre_panel`, `rule_set_config` | 318–569 |
| Rule editor | `rule_editor` + card helpers | 569–882, 1204–1532 |
| Fallback editor | `fallback_file_editor` | 882–1050 |
| Script viewer | `script_viewer` | 1050–1098 |
| Trace activity | `trace_activity_section`, `recent_matching_events` | 1098–1204 |

This is six modules in one file, separable at function boundaries with no shared
mutable state to untangle.

### `crates/app/src/app/workspace_session.rs` — 1,307 lines

Holds eleven distinct types — `WorkspaceIdentity`, `DraftBinding`,
`ConditionFamily`, `ConditionFocus`, `ContractFaultAdoption`, `RuleEditorDraft`,
`RootSettingDrafts`, `RulePrototype`, `TracePrototypeSettings`,
`PrototypeState`, `WorkspaceSession` — spanning identity, drafts, prototype
state, and the session itself. Plausibly three or four boundaries, but the bulk
is one impl block and the seams are less obvious than in `routes.rs`.

**Assessed during implementation, not mandated here.** Recording a guess as a
mandate would be delegating the decision while appearing to make it.

### Everything else the signal flags

**This list is superseded and is retained only as the audit's original state.**
It went stale during the milestone that was fixing stale lists: it omits
`app/drafts.rs`, `routes/rule_editor.rs`, and
`workspace_session_tests/edit_history_round_trips.rs`, created by slices 1, 4
and 5, and still names `mock.rs`, `routes.rs`, and `workspace_session_tests.rs`,
all now sub-threshold hub files. That is precisely the failure decision 4 exists
to end, demonstrated on this document.

**`scripts/check-source-size.sh` is the authoritative inventory.** Run it.

Each flagged file receives a recorded boundary decision under decision 3.

## Decision

### 1. Mandatory splits

- **`app.rs`** — extract the draft-editing domains from the `impl App` block as
  `app/` siblings, following the established pattern.
- **`screens/routes.rs`** — split at the six audited boundaries.
- **`app/workspace_session_tests.rs`** — at 2,216 lines it is the second-largest
  file in the repository and larger than `routes.rs`. "Tests for one module" is
  a statement about location, not cohesion; split by the scenario domains it
  covers.
- **`model/mock.rs`** — separate fixture constructors, and move its inline test
  body out per decision 2.

### 2. Inline test bodies move out

Three files carry `mod tests { … }` inline, which the rule prohibits outright:

| File | Lines | Test module begins |
|---|---:|---:|
| `model/audience.rs` | 35 | 27 |
| `model/friendly_error.rs` | 119 | 91 |
| `model/mock.rs` | 543 | 416 |

Each becomes `<module>/tests.rs` with a `#[cfg(test)] mod tests;` declaration,
matching `selection.rs`, `accelerator.rs`, `match_test.rs`, `app.rs`, and
`workspace_port.rs`. The first two are small; they are in scope because the rule
is about layout, not size.

### 3. Every flagged file gets a recorded boundary decision

For each remaining file the signal flags, record **split** or **single
responsibility**. The second must state positively what the one responsibility
is — that the contents are one thing which separation would harm. "Splitting is
inconvenient" is not a boundary argument.

Two observations offered as input, not conclusions: the two `engine_conformance`
files each mirror one contract tier and may well be single-responsibility; and
the `i18n` locale files, though below the threshold, would be actively harmed by
splitting, since their value is the compile-time exhaustiveness of one match.

### 4. Structure checker replaces the size table

Add `scripts/check-source-size.sh` and its self-test, following the existing
checker pattern. It:

- enumerates `.rs` files above the 500-line signal threshold;
- **fails when a flagged file has no recorded boundary decision** — it does not
  fail on size;
- prints the current inventory, so the record is generated rather than
  transcribed.

**The checker enforces that a decision exists, never that files are small.**
Files grow; a size gate would demand churn on every growth and would be routed
around within a month.

Gate at 500 only. Twenty-two files currently exceed 300; a check printing
twenty-two warnings each run trains people to skim its output, costing more than
it gains. The 300 guideline stays a review norm.

Wire it into `scripts/check-release-gates.sh` and update the gate self-test.

**Delete `architecture.md`'s size table**, leaving one line pointing at the
checker. Do not generate a summary into the docs — that recreates staleness one
step removed, which is the failure being fixed.

### 5. Slices follow boundaries

Each split lands as its own reviewable unit, with the canonical gate run after
each. `app.rs` goes first: largest, highest-risk, and doing it first means later
slices land against a tree that has absorbed the biggest move.

Slices are drawn at **boundaries, not per file**: the two small inline-test-body
moves (`audience.rs`, `friendly_error.rs`) are one slice, being identical in
shape and trivial in risk. `routes.rs`'s six boundaries may land as one slice if
the moves are purely mechanical, or as more if any requires untangling.

Do not batch unrelated boundaries into one changeset. A behaviour-neutral
refactor's safety argument is that each step is individually verifiable.

## Behaviour neutrality

Moves only: no logic edits, no signature changes, no renames, no reordering that
could change evaluation. Visibility widens only where a move requires it, never
beyond the crate.

The canonical gate is the verification and is now sufficient: full workspace test
surface — 202 app, 56 model, 26 conformance, 4 doctests — on both stable and
Rust 1.91, plus strict Clippy, `cargo doc`, audit, and the governance and oracle
checkers.

If a split cannot be made behaviour-neutral, stop and report. A behaviour change
discovered mid-refactor is a design question, not a refactoring detail.

## Acceptance evidence

- The three inline test bodies relocated, gate green after each slice.
- The four mandatory splits complete, each as its own reviewed slice, with the
  boundary each was drawn on stated.
- A recorded boundary decision for every flagged file.
- `scripts/check-source-size.sh` and its self-test passing and wired into the
  canonical gate, with the gate self-test updated.
- `architecture.md`'s size table removed, checker referenced in its place.
- Full canonical gate output after the final slice.
- An explicit statement that no behaviour changed, and that no test was modified
  except where a move required its path to change.

## Alternatives considered

**Split every file above 500.** Rejected as churn for its own sake, and as the
wrong criterion: it would demand splitting a cohesive 600-line file while
ignoring a 200-line file doing three unrelated things.

**Keep the size table, correct it more carefully.** Rejected. It has gone stale
twice under exactly that intention; the failure is the method, not the diligence.

**Defer M5; production will restructure anyway.** Rejected. The mockup is meant
to be an executable specification a production team reads, and a 3,558-line impl
block imposes that cost on them.

**A Cargo xtask instead of a shell checker.** Rejected for consistency — every
repository-owned check here is a shell script with a self-test.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| A "pure move" silently changes behaviour | One boundary per slice, gate after each, no logic edits in a move |
| `routes.rs` splitting collides with M6 preparation | M6 preparation touches the same file; sequence `app.rs` first and coordinate before the `routes.rs` slice |
| "Single responsibility" becomes a way to avoid work | The claim must state the responsibility positively; review evaluates the reasoning, not the count |
| The checker becomes a size gate by drift | Its failure condition is a missing decision, never a line count; the self-test asserts this |
| Churn makes M6 findings hard to re-test | M5 is behaviour-neutral; re-tests cite build SHAs regardless |

## Resolved review questions

Raised at design review and settled by the project owner on 2026-08-02:

1. **Criterion.** Module boundary decides; ELOC is the surface signal that
   indicates where to look. Files grow through maintenance, so a size-based rule
   is the wrong instrument. This reshaped decisions 3, 4, and 5.
2. **Scope.** Housekeeping judged against future maintenance cost, which is why
   `workspace_session_tests.rs` joins the mandatory set and why cohesive large
   files may stay whole.
3. **The checker** enforces that a boundary decision exists, not smallness.
4. **Documentation staleness** is addressed as a general rule — transcribed
   facts go stale, generated and checked facts do not — rather than by patching
   the one table.
5. **Slicing** follows boundaries rather than file counts.

## Status

Design accepted by the project owner on 2026-08-02, including the five
resolutions above. Under the four-folder lifecycle this RFC remains `Proposed`
until its implementation ships, at which point it moves to `done/`; design
acceptance is not a folder transition.

Design accepted and implementation authorized by the project owner on
2026-08-02, assigned to the dev team.

Delivered in seven independently reviewed slices between 2026-08-02 and
2026-08-04: four mandatory splits, three inline test bodies relocated, the
structure checker and its self-test wired into the canonical gate, and a
recorded boundary decision for every flagged file. No behaviour changed; every
relocated test kept its name and body. Reviews are recorded as
`.git-exclude/reviewed/2026-08-04-mk057-*.md`.

Three factual corrections were made to this document during delivery, each
prompted by the implementer checking its claims rather than accepting them: the
sibling-module count, the misleading test-filename finding, and this audit's own
stale inventory. All are marked in place above.

**Exit taken under an amended gate.** Four implementation files carry a recorded
`split` decision that was not executed — `app.rs`, `app/workspace_session.rs`,
`shell/bottom_drawer.rs`, `workspace_port/mapping.rs` — together with a fourth
inline test body at `app/workspace_session.rs:1304`. The project owner amended
M5's 500-ELOC exit clause on 2026-08-04 to admit a recorded boundary decision
with a named follow-up, and deferred these. Each has an analysed boundary, so
the follow-up is execution rather than design.

This RFC moved to `done/` at its lifecycle closure on 2026-08-04.
