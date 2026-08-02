# RFC MK-057 — Maintainable structure

**Status.** Proposed
**Tracks.** Stabilization roadmap M5 — Maintainable structure.
**Touches.** `crates/app/src/app.rs`, `screens/routes.rs`, `model/mock.rs`, the
three implementation files carrying inline test bodies, the source-size record,
and a new source-size checker.

## Summary

Bring the codebase within the project's file-organization rules without changing
behaviour, and replace the hand-maintained source-size record — which has gone
stale twice — with a mechanical check.

The work is deliberately not "split everything above the threshold." Fifteen
files exceed 500 lines; splitting all of them in one behaviour-neutral milestone
would be high-risk churn in a repository whose remaining life is short. This RFC
mandates splits where cohesion is genuinely poor, and requires a recorded
decision for every other threshold crosser.

## Problem and motivation

The project rule recommends splitting above 300 lines and strongly recommends it
above 500, and requires test modules to live in separate files rather than
inline in implementation files. The current tree, measured at `d2530ea`:

- **8 implementation files and 7 test files exceed 500 lines.** The roadmap's M5
  scope names three (`app.rs`, `screens/routes.rs`, `model/mock.rs`), so the
  actual inventory is roughly five times the named scope.
- **Three implementation files carry inline test bodies**, which is a flat rule
  violation rather than a judgement call: `model/audience.rs:27`,
  `model/friendly_error.rs:91`, and `model/mock.rs:416`.
- **The source-size baseline in `docs/src/architecture.md` has gone stale
  twice** — once caught at the start of the 2026-08-01 session against the
  2026-07-22 table, and again at R1 (finding R1-2), where its top entry read
  4,343 for a file that was 4,321 and MK-055's new files were absent entirely.

Correcting a hand-maintained table at closure points demonstrably does not work,
because the next commit invalidates it. M5 plans its splits from that table, so
a wrong number is worse than no number.

**Why now is the right time.** R1-1's closure means the canonical gate runs the
full workspace test surface on both stable and MSRV. "Run the gate after every
split" is therefore a real guarantee rather than an aspiration — which it was
not before, when the conformance suite sat outside the gate. M5 would have been
materially less safe a day ago.

## Goals

1. Eliminate the three inline test bodies.
2. Split the files whose size reflects poor cohesion rather than genuine scope.
3. Record an explicit, reviewed decision for every other file above 500 lines.
4. Replace the stale size table with a check that cannot go stale.
5. Change no behaviour.

## Non-goals

- Splitting every file above the threshold on principle. The roadmap's own exit
  gate permits exceeding 500 "without a reviewed written reason" — meaning a
  reasoned exception is a valid outcome, not a failure.
- Changing public crate boundaries, the `WorkspacePort` contract, message
  vocabulary, or any observable behaviour.
- Renaming types, reworking APIs, or "improving" code encountered during a move.
- UX or localization work; that is M6.
- Production integration or any deferred feature.

## Decision

### 1. Measure physical lines, not ELOC

The project rule is stated in effective lines of code; every record this
programme has kept counts physical lines. Rather than leave the two silently
diverging, **physical lines is the measure**, because it needs no ELOC
definition, is reproducible by `wc -l`, and is strictly more conservative than
the rule's intent. Record the choice so a future reader does not assume ELOC.

### 2. Mandatory splits — poor cohesion

These three are split because their size reflects accumulation, not scope:

- **`crates/app/src/app.rs` (4,321).** The central reducer has absorbed
  unrelated domains. Split by state/reducer domain along the seams M3 already
  established, continuing the direction of `app/workspace_session.rs`,
  `app/runtime.rs`, `app/history.rs`, and the other existing `app/` modules.
- **`crates/app/src/screens/routes.rs` (1,542).** Split along the six boundaries
  the roadmap already identifies: sidebar, rule-set configuration, rule editor,
  fallback editor, script viewer, and trace activity.
- **`crates/model/src/mock.rs` (543).** Separate fixture constructors from each
  other, and move its inline test body out per decision 3.

### 3. Mandatory: inline test bodies move to separate files

Three files carry `mod tests { … }` inline, which the project rule prohibits
outright:

| File | Lines | Test module begins |
|---|---:|---:|
| `crates/model/src/audience.rs` | 35 | 27 |
| `crates/model/src/friendly_error.rs` | 119 | 91 |
| `crates/model/src/mock.rs` | 543 | 416 |

Each becomes `<module>/tests.rs` with a `#[cfg(test)] mod tests;` declaration,
matching the pattern already used by `selection.rs`, `accelerator.rs`,
`match_test.rs`, `app.rs`, and `workspace_port.rs`.

Note that `audience.rs` and `friendly_error.rs` are small — 35 and 119 lines.
They are in scope because the rule is about *layout*, not size, and leaving two
known violations in place while writing a milestone about file organization
would be incoherent.

### 4. Every other threshold crosser gets a recorded decision

For each remaining file above 500 lines, the implementation records **split** or
**reasoned exception**. An exception must state why the file is cohesive — that
its contents are one thing that would be harmed by separation — not merely that
splitting is inconvenient.

Current inventory at `d2530ea`, for disposition:

| File | Lines | Kind |
|---|---:|---|
| `app/workspace_session_tests.rs` | 2,216 | Test |
| `model/workspace_port/memory_tests.rs` | 1,463 | Test |
| `app/workspace_session.rs` | 1,307 | Implementation |
| `model/workspace_port/memory.rs` | 1,272 | Implementation |
| `app/global_save_tests.rs` | 893 | Test |
| `model/workspace_port.rs` | 878 | Implementation |
| `app/runtime_tests.rs` | 840 | Test |
| `model/tests/engine_conformance/tier2_scenarios.rs` | 831 | Test |
| `model/workspace_port/tests.rs` | 547 | Test |
| `app/shell/bottom_drawer.rs` | 526 | Implementation |
| `model/workspace_port/mapping.rs` | 515 | Implementation |
| `model/tests/engine_conformance/tier1_mapping.rs` | 501 | Test |

I am not pre-judging these. Cohesion cannot be assessed from a line count, and
an RFC that guessed would be delegating a decision while appearing to make one.
The implementer assesses each and records the reasoning; the review evaluates
the reasoning, not the count.

Two observations offered as input rather than conclusions: the two
`engine_conformance` files each mirror one contract tier and may well be
cohesive; and the `i18n` locale files, though below the threshold, would be
actively harmed by splitting, since their value is the compile-time
exhaustiveness of a single match.

### 5. Replace the size table with a checker

Add `scripts/check-source-size.sh` and its self-test, following the structure of
the existing `check-rfcs.sh` and oracle guards. It:

- enumerates `.rs` files above the 500-line threshold;
- fails when a file exceeds it without an entry in a recorded exception list;
- prints the current inventory so the record is generated, never transcribed.

The exception list — file, line count at time of review, and the reason — becomes
the durable record, replacing `architecture.md`'s hand-maintained table. **Delete
that table.** A number that is wrong is worse than a check that is authoritative,
and this record has now gone stale twice.

Wire the checker into `scripts/check-release-gates.sh` alongside the other
repository-owned checks, and update the gate self-test's `write_expected()`.

This closes R1-2 mechanically rather than by another manual correction.

### 6. One reviewed slice per file

Splits land one file at a time, each its own reviewable unit, with the canonical
gate run after each. `app.rs` goes first: it is the largest and the highest-risk,
and doing it first means the remaining slices land against a tree that has
already absorbed the biggest move.

Do not batch splits into one changeset. A behaviour-neutral refactor's entire
safety argument is that each step is individually verifiable; a batched diff
across four files forfeits that.

## Behaviour neutrality

Moves only: no logic edits, no signature changes, no renames, no reordering of
match arms that could change evaluation. Visibility widens only where a move
requires it, and never beyond the crate.

The canonical gate is the verification, and it is now sufficient for the purpose:
it runs the full workspace test surface — 202 app, 56 model, 26 conformance, 4
doctests — on both stable and Rust 1.91, plus strict Clippy, `cargo doc`, audit,
and the governance and oracle checkers.

If a split cannot be made behaviour-neutral, stop and report. A behaviour change
discovered mid-refactor is a design question, not a refactoring detail.

## Acceptance evidence

- The three inline test bodies relocated, with the gate green after each.
- The mandatory splits complete, each as its own reviewed slice.
- A disposition for every file above 500 lines: split, or exception with stated
  cohesion reasoning.
- `scripts/check-source-size.sh` and its self-test passing, wired into the
  canonical gate, with the gate self-test updated.
- `architecture.md`'s size table removed and the checker referenced in its place.
- Full canonical gate output after the final slice.
- An explicit statement that no behaviour changed, and that no existing test was
  modified except where a move required its path to change.

## Alternatives considered

**Split every file above 500.** Rejected as disproportionate. Fifteen files of
churn in a short-lived repository, to satisfy a rule whose own wording is
"strongly recommended" and whose roadmap exit gate explicitly admits reasoned
exceptions.

**Defer M5 entirely; the production project will restructure anyway.** Tempting,
and the honest counter is that the mockup is meant to be an *executable
specification* a production effort reads. A 4,321-line reducer is hard to read,
and the cost of that lands on the production team, not here.

**Keep the size table and correct it more carefully.** Rejected. It has gone
stale twice under exactly that intention. The failure is the method, not the
diligence.

**Mechanize with a Cargo xtask instead of a shell checker.** Rejected for
consistency: every other repository-owned check here is a shell script with a
self-test, and introducing a second mechanism for one check is not worth the
divergence.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| A "pure move" silently changes behaviour | One file per slice, gate after each, no logic edits permitted in a move |
| Splitting `app.rs` conflicts with M6 preparation work | M6's preparation touches `screens/` and i18n; sequence `app.rs` first and coordinate before the `routes.rs` slice |
| Exceptions become a way to avoid the work | An exception must state cohesion positively; the review evaluates reasoning, not counts |
| The new checker duplicates effort at R2 | It replaces a manual step that has already failed twice; net reduction |
| Churn makes M6 findings hard to re-test | M5 is behaviour-neutral, so scenarios remain valid; re-tests cite build SHAs regardless |

## Review questions

1. Are the three mandatory splits the right set, or should any threshold crosser
   in decision 4 be mandated outright rather than left to a recorded decision?
2. Is "record split-or-exception per file" the right instrument, or too
   permissive — should a hard cap be enforced with no exceptions?
3. Should the size checker gate at 500 only, or also warn at 300?
4. Is deleting `architecture.md`'s size table correct, or should a generated
   summary remain in the docs for readers who will not run the checker?
5. Is one-file-per-slice proportionate, or unnecessarily slow for moves this
   mechanical?

Creation of this Proposed RFC does not authorize implementation.
