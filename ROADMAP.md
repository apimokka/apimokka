# apimokka — Stabilization roadmap

**Baseline revised:** 2026-08-01

**Current release:** v0.10.0 (no release is planned from this programme)

**Current decision:** NO-GO for production integration

**Permitted use:** internal UI/UX exploration with the mockup limitations disclosed

This roadmap is **dependency-ordered, not calendar-ordered**. It defines what
must be done, in what order, and what evidence closes each step. It does not
assign dates. Detailed design belongs in RFCs; this file remains the
programme-level source of truth.

The repository is a UI/UX mockup until the Integration Readiness milestone is
approved. It has no real workspace file I/O and no live apimock-rs server
connection. Do not present it as a production integration baseline before then.

## Outcome

The programme is complete when apimokka is a trustworthy executable
specification for a production GUI effort:

- match-test behavior is conformant with the supported apimock-rs operators;
- the editing boundary has a mapping verified against the real apimock-rs
  configuration contract, with every remaining divergence recorded;
- repository governance records agree and are mechanically checked;
- formatting, tests, builds, lints, and the security policy pass on the
  declared toolchains;
- central modules and tests follow the project's maintainability rules;
- the principal workflows have current visual, keyboard, and EN/JA evidence; and
- an independent architecture re-review records a GO decision.

## Working assumptions

- Rust 1.91 remains the declared MSRV until an approved RFC changes it. Current
  stable is an additional verification target.
- **`apimock-config` and `apimock-routing` 6.0.0 are the authoritative
  executable artifacts**, adopted 2026-09-05 (M11). 5.10.0 was authoritative
  until then, and the never-published 5.10.1 prose reference that MK-053 had to
  work from is superseded: 6.0.0 ships `docs/src/library/`, a documented API with
  a stability statement.
- **apimock-rs 6.0.0 is adopted** (M11 complete 2026-09-05, `6c12a07`) — RFC
  MK-060, decided by the project owner 2026-08-21 (spec at
  `.git-exclude/specs/6.0.0-for-gui-app-team.md`). Its `docs/src/library/`
  section states the library API was **shaped for a GUI**, and carries an
  additive-only API gate from 6.0.0 onward. The architect first recommended
  against adopting, on the cost of re-running MK-052 and MK-055; **that was
  wrong and is withdrawn.** It applied a cost argument to a correctness
  question: MK-055's value is that our mapping is checked against the *real*
  engine contract, and the production project will build against 6.x, so
  validating against 5.10.0 produces a specification for a contract nobody will
  use. Measured against the published API baselines, all **89** engine symbols
  this repository references are present at 6.x with **none removed**.
- MK-060 went 5.10.0 to 6.0.0 directly rather than stepping through the nine
  unused 5.x minors the caret requirement had left behind (the line reached
  5.19.1). `apimock-config` remains a **test-only dev-dependency**; our
  production surface on this crate family is four items from `apimock-routing`,
  and `apimock-server` is not depended on at all.
- No production file I/O, subprocess control, trace socket, or Rhai editing is
  added by this programme. Engine crates may be adopted as **test-only**
  dev-dependencies where that is the only way to verify a contract.
- **apimokka is intended to be cross-platform** across Linux, macOS, and
  Windows. Only Linux has been exercised to date. `README.md` currently states
  a Linux prerequisite and attributes it to iced 0.14; that attribution is
  incorrect and the statement must be corrected. Platform support is a project
  scope decision, not a framework constraint.
- mdBook documentation is **not** required of this mockup. It is a requirement
  of the production GUI project and is recorded in the deferred list.
- Existing v0.10.0 behavior remains usable for internal UX review while
  stabilization proceeds, but known match-test limitations must be disclosed.
- Independent review and UX preparation may run in parallel when they do not
  alter the same design baseline.

Implementation throughput has not been the programme's constraint. Review and
decision turnaround, and the availability of real UX participants, have been.
Sequencing below reflects that.

## Ownership and approval

These role assignments apply until the project owner records a replacement:

| Step | Accountable owner | Delivery owner | Independent reviewer | Evidence approver |
|---|---|---|---|---|
| M0 | Project owner (nabbisen) | Programme architect | Architecture auditor | Project owner |
| M1 | Project owner | RFC author / assigned implementer | Architecture auditor | Project owner |
| M2 | Project owner | RFC author / assigned implementer | Engine-conformance reviewer | Project owner |
| M3 | Project owner | RFC author / assigned implementer | Architecture auditor | Project owner |
| M4 | Project owner | Assigned implementer | Security/release reviewer | Project owner |
| M7 | Project owner | RFC author / assigned implementer | Engine-conformance reviewer | Project owner |
| R1 | Project owner | — | Architecture auditor | Project owner |
| M5 | Project owner | RFC author / assigned implementer | Architecture auditor | Project owner |
| M8 | Project owner | RFC author / assigned implementer | Architecture auditor | Project owner |
| M11 | Project owner | RFC author / assigned implementer | Engine-conformance reviewer | Project owner |
| M9 | Project owner | Assigned implementer | Architecture auditor | Project owner |
| M10 | Project owner | Assigned implementer | UX/accessibility reviewer | Project owner |
| M6 | Project owner and session coordinator | Assigned implementer / UX facilitator | UX/accessibility reviewer | Project owner |
| R2 | Project owner | — | Architecture auditor | Project owner |

The named delivery person must be recorded in the relevant RFC before
implementation begins. An author or implementer does not independently approve
their own milestone exit evidence. When no reviewer independent of the author
is available for a step, the project owner performs the confirmation directly
and the deviation is recorded in `docs/src/development-and-gates.md`.

## Milestone numbering

Milestone identifiers are assigned on creation and are **never reused or
renumbered**, for the same reason RFC numbers are not: dozens of review records
reference them. An identifier therefore does not indicate sequence position.
**The work sequence below is the authority on order.**

## Work sequence

```text
M0 ─ M1 ─ M2 ─ M3 ─ M4 ─ M7 ─ R1 ─┬─ M5 (complete) ───────────┬─ R2
                                   ├─ M8 ─ M11 ─ M9 ─ M10 ─ M6 ┤
                                   └─ production-integration roadmap (draft only)
```

| Order | Step | Prerequisite |
|---:|---|---|
| 1 | M0 — Planning approval | — |
| 2 | M1 — Repository truth | M0 |
| 3 | M2 — Match-test conformance | M1 |
| 4 | M3 — Integration boundary | M2 |
| 5 | M4 — Quality and security gates | M3 |
| 6 | M7 — Engine contract conformance | M4 |
| 7 | R1 — Blocking re-review | M7 |
| 8 | M5 — Maintainable structure | R1 GO |
| 9 | M8 — snora adoption | R1 GO |
| 10 | M11 — apimock-rs 6.0.0 adoption | M8's capture; before M9, since it may move the editing surface they build on |
| 11 | M9 — Keyboard operability | M8's capture (which it would otherwise invalidate); M11 |
| 12 | M10 — Typography and readability | M9; last appearance change before M6 |
| 13 | M6 — UX acceptance evidence | R1 GO; M8 before its live runs; M11 before it validates an editing surface; M9 before its keyboard-only session; M10 before its readability probe |
| 14 | R2 — Integration readiness | M5, M8, M11, M9, M10 and M6 |

This table owns **order and prerequisites only**. Current state is owned solely
by the progress table at the end of this document. Two tables recording the same
fact is how the work-sequence column silently went stale while the progress
table stayed correct.

M5 and M6 run in parallel after R1. The production-integration roadmap is
drafted in parallel after R1 GO; drafting authorizes no features.

**M6 participants are the only prerequisite that cannot be satisfied from
inside this repository.** Identifying a Guided newcomer and an Expert
apimock-rs user should begin as soon as M6 protocol design starts, because that
recruitment — not implementation — will gate R2.

R1 and R2 operate on frozen inputs. Inputs freeze when the preceding step is
accepted; material changes during review invalidate affected evidence and
require targeted re-review.

## Programme-wide gate cadence

The canonical gate is the repository-owned script adopted in MK-054:

```sh
bash scripts/check-release-gates.sh
```

It runs the stable and Rust 1.91 test/build gates, strict Clippy, `cargo
audit`, the matcher-oracle and RFC-integrity checkers, and the whitespace gate,
stopping on first failure. Its self-test
(`bash scripts/check-release-gates-self-test.sh`) verifies the exact command
contract. Future CI should call this script rather than duplicate its command
list.

Run the canonical gate at every implementation checkpoint and at every step
exit. Documentation-only changes may record which gates were not rerun and why,
as M1–M4 closures did.

Two additions follow from decisions recorded after MK-054:

- **`cargo doc --workspace --no-deps --locked`** joins the gate. It ran once in
  M3 to validate model intra-doc links and was never repeated, so link rot goes
  undetected. `cargo test --workspace --doc` covers doctests, which is a
  different check.
- **Cross-platform verification.** The canonical gate proves one host platform.
  Because the project is intended to support Linux, macOS, and Windows, a
  build-and-test run on each supported platform is required before R2, and a
  platform must not be claimed as supported without a recorded run. `cargo
  audit` is unaffected, since `Cargo.lock` is target-independent.

Cross-platform verification is performed **manually and recorded as evidence**,
not through CI. MK-054's deferral of hosted CI stands, and cross-platform scope
does not overturn it:

- the mockup contains no filesystem, process, or network code, so a successful
  build on another platform carries little information;
- what actually varies across platforms here is rendering, font metrics, window
  decoration, layout overflow, and high-contrast behaviour, none of which a
  build gate can assess. Those are validated by human inspection in M6;
- the number of remaining verification events before this repository is
  absorbed into the production project is small and bounded, so CI setup and
  maintenance would not amortize.

CI belongs to the production GUI project, which will be long-lived and will
carry genuinely platform-dependent code — path resolution, subprocess control,
and a trace transport whose UDS option is Unix-only and needs a TCP path on
Windows. It is recorded in the deferred list on that basis.

Any approved advisory exception must be repository-owned and machine-checked.
Its record must include advisory ID, dependency path, exploitability analysis,
decision owner, approval date, expiry, and remediation trigger. The policy
check must fail for an unapproved or expired exception even when `cargo audit`
continues to print the advisory.

## Completed steps

Scope and exit criteria for M0–M4 are preserved in their RFCs and in
`docs/src/development-and-gates.md`. Summary of what each established:

- **M0 — Planning approval.** Agreed scope, ordering, gate policy, ownership,
  and the RFC queue.
- **M1 — Repository truth (MK-051).** Rebuilt the RFC index from disk,
  reconciled versions and Status fields, recorded the snora de-vendoring, and
  added the executable RFC-integrity checker.
- **M2 — Match-test conformance (MK-052).** Made Test Rule fail-closed against
  real `apimock-routing` 5.10.0 matcher primitives, with a published capability
  matrix and a matcher-oracle guard against unreviewed dependency drift.
- **M3 — Integration boundary (MK-053).** Replaced direct snapshot mutation
  with the `WorkspacePort` mapping boundary: typed atomic transactions, stable
  condition identity, canonical/render correlation, semantic undo/redo, runtime
  correlation, and typed Global Save reporting.
- **M4 — Quality and security gates (MK-054).** Cleared the workspace warning
  backlog without suppression, removed the iced highlighter dependency chain,
  resolved both `quick-xml` advisories through compatible updates, and added
  the canonical release-gate script and its self-test.

## Remaining steps

### M7 — Engine contract conformance

**Goal:** verify the M3 editing boundary against the real apimock-rs
configuration contract instead of against a locally designed mapping.

**Why this exists.** M3 was designed on the premise that no reproducible
`apimock-config` artifact was available. That is true only of 5.10.1, which was
never published. `apimock-config` 5.10.0 has been on crates.io since
2026-05-16 with MSRV 1.91.0 and exposes the full `Workspace::load / snapshot /
apply / validate / save / list_directory` surface and the `EditCommand` enum.
M2 had already adopted `apimock-routing` 5.10.0 as an executable oracle, so the
programme applied two different standards to the same engine family. B2 is
currently closed against our own mapping, not against the contract we must
integrate with.

Scope:

- adopt `apimock-config` 5.10.0 as a **test-only dev-dependency**; production
  targets gain no filesystem, process, or network code;
- execute the MK-053 port contract suite against a real `Workspace` in a
  temporary directory, covering `EditCommand` shapes, per-condition `NodeId`
  addressing, `Option<Vec<_>>` preserve/clear/replace semantics, apply
  diagnostics, changed nodes, and reload/restart hints;
- record every mapping divergence with executable evidence;
- reduce the `ReferenceGap` inventory in `docs/src/architecture.md` to items the
  real artifact genuinely does not establish;
- correct the "no reproducible artifact" statements in
  `docs/src/architecture.md`, `crates/model/README.md`, and
  `docs/src/match-test-conformance.md`;
- extend the oracle guard to pin `apimock-config` version, source, checksum,
  and activated features.

Exit gate:

- the contract suite runs against real `apimock-config` 5.10.0 and passes, or
  each failure is recorded as an accepted, documented divergence;
- no production (non-dev) dependency on `apimock-config` exists;
- the `ReferenceGap` inventory reflects the artifact rather than its absence;
- no document claims an unavailable artifact where 5.10.0 establishes the
  behavior;
- the canonical gate passes.

### R1 — Blocking architecture re-review

**Goal:** independently verify that findings B1–B5 are resolved.

Required inputs: M1 governance evidence; M2 operator conformance matrix and
test output; M3 adapter/mapping RFC, implementation, and contract tests; M4
toolchain and security gate evidence; **M7 engine-conformance evidence**.

Decision:

- **GO:** M5 and M6 proceed toward an integration-ready baseline;
- **CONDITIONAL GO:** only listed non-production work may proceed;
- **NO-GO:** return unresolved blockers to the owning step.

### M5 — Maintainable structure

**Goal:** bring the codebase back within the project's file-organization rules
without changing behavior.

Scope:

- split `app.rs` by state/reducer domain, building on the test extraction and
  reducer/adapter seam completed in M3;
- split `screens/routes.rs` by sidebar, rule-set configuration, editor,
  fallback editor, script viewer, and trace activity boundaries;
- split `model/mock.rs` fixtures and tests;
- remove any remaining dead placeholders. The fake discarded
  `UndoCommand::AddRule` recorded in the 2026-07-15 architecture review was
  already removed by MK-053 and is not outstanding;
- preserve public crate boundaries and behavior through focused tests.

Exit gate:

- no implementation file exceeds 500 ELOC without either a reviewed written
  reason, or a recorded boundary decision naming a specific follow-up. The
  second disposition was added by project-owner decision on 2026-08-04, aligning
  this clause with the 300-ELOC clause below, which already accepted a
  follow-up. The deferred items it admits are named in the progress table;
- files above 300 ELOC have an explicit cohesion justification or follow-up;
- implementation files contain no inline test modules where the project rule
  requires separate test files;
- the canonical gate remains green after every split.

### M6 — UX acceptance evidence

**Goal:** validate the mockup for its actual purpose: stakeholder review of
workflows and interaction design.

Preparation and scope:

- inventory user-visible literals before acceptance; migrate hard-coded English
  to i18n keys or record an explicit reviewed exemption;
- name participant personas or stakeholder roles, including at least a Guided
  newcomer and an Expert apimock-rs user;
- define scenario scripts, issue-severity definitions, facilitator rules, and
  the exact environment/build identity;
- cover both pointer and keyboard paths for open/create workspace,
  add/edit/test rule, inspect trace outcome, save/revert fallback draft, and
  change settings;
- visible focus, non-color status communication, high-contrast themes, and 200%
  text-scale checks where the platform supports them;
- select a representative matrix across Guided/Expert, EN/JA, themes, supported
  window sizes, input methods, **and supported platforms**. Full Cartesian
  coverage is not required, but every dimension must appear in a risk-based
  combination;
- correct platform-conditional user-facing copy. `screens/command_palette.rs`
  displays macOS-only shortcut notation (`⌘Z`, `⌘⇧Z`, `⌘S`, `⌘R`) while the
  handler at `app.rs:4266-4286` correctly accepts Ctrl as well, so Linux and
  Windows users are shown a key their keyboard does not have. `README.md:64`
  repeats the notation. These literals also sit outside the i18n system, so
  they belong to the localization inventory above. This defect is independently
  fixable ahead of M6 and need not wait for the acceptance sessions.

Outcome criteria:

- each primary scenario records completion, facilitator rescue, critical-error
  count, elapsed time where useful, and participant observations;
- no release-blocking scenario requires facilitator rescue;
- severity rules distinguish blocking workflow failure, serious confusion or
  accessibility failure, and non-blocking polish;
- fixes receive targeted re-test with the same scenario and build identity;
- deferral is allowed only for a named owner, rationale, user impact, and target
  step, and never for a release-blocking workflow failure.

Exit gate:

- localization inventory has no unexplained user-visible English literals;
- findings, outcomes, screenshots/notes, personas, scenarios, and environment
  details are captured;
- primary scenarios complete without facilitator rescue and without unresolved
  critical errors;
- blocking clipping, focus, navigation, or misleading-affordance issues are fixed;
- enabled no-op controls are removed, disabled with a reason, or implemented;
- required fixes are re-tested and any permitted deferrals satisfy the recorded
  deferral rule;
- remaining accessibility gaps are explicit production requirements.

### R2 — Integration readiness

**Goal:** decide whether the stabilized mockup can become the executable
specification for production GUI integration.

GO requires:

- R1 did not leave blocking findings;
- M5 and M6 exit gates pass;
- current docs, RFC index, changelog, gate evidence, and archive contents agree;
- every platform claimed as supported has a recorded build-and-test run;
- the no-I/O mockup boundary and the next production-integration risks are
  documented;
- the project owner approves the review package.

A GO decision permits adoption of the separately drafted production-integration
roadmap for file I/O, subprocess lifecycle, trace transport, external edits,
persistence, and Rhai editing. It does not itself authorize those features.

## RFC design queue

RFC identifiers are assigned only when files are created.

| Order | Topic | Step | State |
|---:|---|---|---|
| 1 | Repository governance repair and automated RFC integrity | M1 | MK-051 Implemented |
| 2 | Test-rule matcher conformance | M2 | MK-052 Implemented |
| 3 | GUI editing boundary and apimock-rs mapping/adapter | M3 | MK-053 Implemented |
| 4 | Release gates, dependency policy, and security exceptions | M4 | MK-054 Implemented |
| 5 | **Engine contract conformance against apimock-config 5.10.0** | M7 | MK-055 Proposed, design accepted |
| 6 | Reducer, routes, fixtures, and test modularization | M5 | To be created |
| 7 | UX and accessibility acceptance protocol | M6 | To be created |

Optional developer handoffs should be created only where the RFC is too large
to implement safely from the design alone. M5's package should include a
file-move sequence and regression checklist. M7's package should include a
divergence matrix if the conformance run finds more than a few differences.

## Release and delivery policy

This programme produces **no release**. apimokka is a mockup whose delivery is
integration into the production GUI project, not publication. Accordingly:

- no version bump, tag, push, registry publication, or release archive is
  produced by this programme;
- `v0.10.0` remains the recorded version; completed RFCs are recorded as
  `Implemented (Unreleased)`;
- integration-ready labeling is reserved for R2 GO and refers to specification
  readiness, not to a released artifact;
- do not create commits, tags, archives, or pushes without explicit
  project-owner authorization for that task.

If the project owner later wants a distributable snapshot for archival or
handoff, it follows the project rule — files at archive root, version suffix in
the archive name, and no `.git`, `.git-exclude`, or `target` content — and is
requested explicitly rather than produced on a cadence.

## Risks and controls

| Risk | Effect | Control |
|---|---|---|
| Real `apimock-config` reveals material mapping divergence | M7 rework, possible M3 amendment | Test against the real artifact before R1, not after; record divergences rather than hiding them |
| `apimock-config` pulls a conflicting `apimock-routing` resolution | M7 blocked | Verify the resolved graph and extend the oracle guard before accepting |
| Test-only engine dependency leaks into production targets | Scope breach | Dev-dependency only; verify no production target references it |
| Adapter work expands into production I/O | M7 overrun | Keep filesystem and live server control explicitly out of scope |
| Structural split overlaps behavior changes | Regression risk | Keep M5 behavior-neutral; run the canonical gate after each split |
| UX participants unavailable | R2 blocked | Identify participants at M6 protocol design, not at session time |
| Reviewer independence unavailable for a step | Weakened evidence | Project owner confirms directly and the deviation is recorded |
| New advisory disclosed after M4 | Gate regression | Canonical gate runs `cargo audit` at every step exit |
| MSRV dependency incompatibility | Late failure | Rust 1.91 checks run in the canonical gate, not only at exits |

## Progress tracking

Step status is one of `Not started`, `Designing`, `Implementing`, `In review`,
`Complete`, or `Blocked`. Update this table whenever a step changes state;
detailed task progress belongs in its RFC or handoff.

Allowed transitions are:

```text
Not started → Designing → Implementing → In review → Complete
                       ↘ Blocked ← any active state
```

`Complete` means the independent reviewer accepted the exit-gate evidence and
the evidence approver recorded acceptance. Finishing implementation alone moves
the step to `In review`, not `Complete`. A blocked step records the blocking
condition, owner, and next decision.

| Step | Status | Decision/evidence |
|---|---|---|
| M0 — Planning approval | Complete | Approved 2026-07-15; evidence: `.git-exclude/reviewed/2026-07-15-apimokka-stabilization-roadmap-m0-confirmation-review.md` |
| M1 — Repository truth | Complete | Accepted 2026-07-15; MK-051 Implemented (Unreleased); evidence: `.git-exclude/reviewed/2026-07-15-rfc-mk051-repository-truth-closure-confirmation-review.md` |
| M2 — Match-test conformance | Complete | Accepted 2026-07-16; MK-052 Implemented (Unreleased); implementation evidence: `.git-exclude/reviewed/2026-07-16-rfc-mk052-test-rule-matcher-conformance-implementation-second-rereview.md`; dependency-policy evidence: `.git-exclude/reviewed/2026-07-16-rfc-mk052-compatible-manifest-lockfile-authority-amendment-rereview.md`; closure evidence: `.git-exclude/reviewed/2026-07-16-rfc-mk052-closure-confirmation-rereview.md` |
| M3 — Integration boundary | Complete | Accepted 2026-07-23; MK-053 Implemented (Unreleased); integrated implementation evidence: `.git-exclude/reviewed/2026-07-22-rfc-mk053-integrated-implementation-review.md`; closure evidence: `.git-exclude/reviewed/2026-07-23-rfc-mk053-closure-confirmation-review.md` |
| M4 — Quality and security gates | Complete | Accepted 2026-08-01; MK-054 Implemented (Unreleased); implementation evidence: `.git-exclude/reviewed/2026-08-01-rfc-mk054-quality-and-security-gates-implementation-rereview.md`, committed `160456c`; lifecycle closure prepared by the programme architect and confirmed directly by the project owner on 2026-08-01, without the separate independent closure-confirmation session used for M1–M3 |
| M7 — Engine contract conformance | Complete | Accepted 2026-08-02; MK-055 Implemented (Unreleased), committed `fec0fbf`, closure `5534192`; checkpoint evidence: `.git-exclude/reviewed/2026-08-02-rfc-mk055-harness-checkpoint-review.md`; implementation evidence: `.git-exclude/reviewed/2026-08-02-rfc-mk055-implementation-review.md`. Lifecycle closure was prepared by the programme architect and confirmed directly by the project owner, without a separate independent closure session, as recorded for M4. |
| R1 — Blocking re-review | Complete | **CONDITIONAL GO** recorded 2026-08-02 on frozen input `5534192`; evidence: `.git-exclude/reviewed/2026-08-02-r1-blocking-architecture-re-review.md`. B1–B5 all resolved. Condition R1-1 — the canonical gate did not run integration-test targets, leaving the MK-055 conformance suite unguarded — **closed 2026-08-02**; both toolchains now run the full workspace test surface inside the gate. Evidence: `.git-exclude/reviewed/2026-08-02-r1-1-gate-integration-coverage-review.md` and `.git-exclude/reviewed/2026-08-02-r1-1-msrv-extension-review.md`. M5 implementation is unblocked. Non-blocking findings R1-2 (stale source-size baseline) and R1-3 (two conformance test files above the split threshold) carry into M5 planning. Conducted as a **self-audit** by the programme architect at the project owner's direction, with the reviewer-independence conflict recorded in the verdict; it is not an independent review. |
| M5 — Maintainable structure | Complete | Accepted 2026-08-04. MK-057 delivered in seven reviewed slices; four mandatory splits, three inline test bodies relocated, `scripts/check-source-size.sh` wired into the canonical gate, and a recorded boundary decision for all 15 flagged files. Evidence: `.git-exclude/reviewed/2026-08-04-mk057-*.md`. **Exit taken under the amended 500-ELOC clause**: four implementation files carry a recorded `split` decision that was not executed, deferred by project-owner decision on 2026-08-04 — `crates/app/src/app.rs` (3,497), `crates/app/src/app/workspace_session.rs` (1,320), `crates/app/src/shell/bottom_drawer.rs` (535), `crates/model/src/workspace_port/mapping.rs` (522). A fourth inline test body at `crates/app/src/app/workspace_session.rs:1304` is deferred with them. Each has a named boundary already analysed, so the follow-up is execution rather than design. |
| M8 — snora adoption | **Complete** | **Accepted 2026-09-04.** Capture run collaboratively: the architect drove launch, `wtype` keystrokes and screenshots; the project owner supplied the pointer clicks, which are the only actions the application cannot accept from a keyboard until M9 lands. Evidence: 14 artefacts in `.git-exclude/release-evidence/2026-08-12-m8-visual-capture/`, results in its `RESULTS.md`. **Phase 3 (A1–A7): all PASS.** Dialog card intact in all four presets. **A6 — the defect M8 exists for — is proven by comparison, not judgement**: an undimmed `high_contrast_dark` reference is pure black where the dimmed backdrop is plainly lighter; the shipped defect was black-on-black. **A8 judgement confirmed by the project owner**: the card reads correctly and the dim is not heavy — the answer snora has asked for twice and had from nobody. **Phase 2 (B1–B4): all PASS.** Stock inputs, dropdown and checkbox follow the palette in all four presets; `Dark` is the case Phase 2 existed for. Scrollbars unexercised (content fits). **Pass C measured rather than eyeballed:** backdrop RGB 153 at snora 0.28.0 versus 143 at 0.42.0 over a white page — exactly 255x(1-0.40) and 255x(1-0.44), an independent pixel-level confirmation of snora's `DIM_ALPHA` change. **Pass D closed a 2x2 that had been open since 2026-08-04:** an absolute path checked immediately after `niri msg action screenshot-window` returns is absent and present 2s later, so the write is asynchronous regardless of path type and `f7356c6`'s "requires an absolute path" was a misdiagnosis. **B2-1 FAILED and does not hold this milestone**, by project-owner decision: it was added opportunistically on 2026-08-18 to check a surface outside MK-058's scope, and every case testing Phases 1-3 passed. See the defect list below. Prior state: code-complete at 0.28.0 (`265e9b6`, `8caa826`, `2c191aa`, `b0f62a2` against baseline `cef5e32`), bumped to 0.29.0 (`5aeee57`) and 0.42.0 (`93815e7`), CI green on six legs at each. **Task 015 closed 2026-09-04** with commit 2 (`5d5388e`): `snora-widgets` dropped, `default-features = false, features = ["design"]`, CI run `33837173928` green; reviewed `.git-exclude/reviewed/2026-09-04-snora-0-42-upgrade-commit-2-review.md`. **We are the first consumer anywhere running design-without-widgets**, which produced two measurements worth keeping: the stripped binary is **byte-identical** either way (17,899,656 bytes — the linker already eliminated the crate when it was merely unused, so an unused optional dependency and a binary cost are not the same thing), and **rendered output is byte-identical** to M8's capture, so that evidence still describes the shipped build. **Near-miss recorded:** pruning the now-stale `snora-widgets` entry from `Cargo.lock` needs a bare `cargo update`, which would also carry `apimock-config`/`apimock-routing` 5.10.0 → 5.19.1 — invalidating MK-052 and MK-055 and pre-empting MK-060's deliberate 5.10.0 → 6.x decision. Run, inspected, reverted; the entry is correctly left in place. |
| M11 — apimock-rs 6.0.0 adoption | **Complete** | **Accepted 2026-09-05.** `6c12a07`; reviewed `.git-exclude/reviewed/2026-09-05-mk060-apimock-6-adoption-review.md`; CI run `33929641568` green on all six legs. **Production source changed by zero lines.** The diff is Cargo files, the test-only conformance suite, and the four oracle scripts whose `5.10.0` pins the task predicted would trip. 290 tests pass on both toolchains (203/56/27/4). **All nine MK-055 divergences re-classified against 6.0.0 and all nine persist**, none closed. **A tenth was found by execution, not by reading:** `TlsCertFile`/`TlsKeyFile` moved from `restart()` to `reload()` in apimock-config 6.0.0 (their RFC 020 — cert rotation uses the reloadable resolver), which our `map_root_setting` still classifies as `Restart`. Verified at source in both vendored crates. Caught by a pre-existing Tier 2 test failing — **the first time MK-055's executable oracle has detected an engine change on its own**, and the justification for having built it as an oracle rather than a document. **Accepted as a stricter divergence, deliberately not corrected here:** we say "restart" where "reload" would now do — never the reverse — and fixing it would land an unvalidated, user-visible behaviour change between M8's capture and M6's sessions. See the production-integration inputs below; it is owed, not settled. **The `library/` delta confirmed every MK-053 decision** — no amendment raised — while surfacing more new engine surface than MK-060's symbol-presence sizing could bound, exactly as that sizing warned. |
| M9 — Keyboard operability | **Complete** | **Accepted 2026-09-05.** `74e7f75`; reviewed `.git-exclude/reviewed/2026-09-05-task014-mk033-keyboard-palette-review.md`; CI run `33943676852` green on all six legs. MK-033's three keyboard requirements built — search field auto-focused on open, `Enter` executes the selected row, arrow-key navigation — with the command table extracted to `palette_commands.rs` following `accelerator.rs`'s precedent so `view` and `update` cannot disagree about which row is selected. The mode picker gained the same idiom, **closing MK-023's first-screen gap: a keyboard-only user can now pass the application's first screen**, which was the S1 condition blocking M6's keyboard-only session. D-4 fixed (option 2 — the palette toggle is inert outside `AppView::Workspace`, turning a silent no-op into an honest one). 306 tests pass on both toolchains; `update` remains a pure reducer, its 486 existing call sites untouched, with the one genuine `Task` isolated in a wrapper `main.rs` alone calls. **The live `wtype` run found a defect this task's own requirement created**, and no unit test could reach: `Escape` needed two presses to close the palette. Root-caused at source — `iced_widget`'s `text_input` captures `Escape` unconditionally when focused (`text_input.rs:1235`), and `iced::keyboard::listen()` only ever sees events with `Status::Ignored`, so auto-focus made the first press invisible to the global handler. Fixed at the root by moving the subscription to `iced::event::listen_with`, which reports capture status rather than dropping captured events, and re-verified live. **Third time the live-run instruction has paid** — after task 010's three execution-only script defects and M8's four. |
| M10 — Typography and readability | **Complete** | **Accepted 2026-09-05.** `4deadc2`; reviewed `.git-exclude/reviewed/2026-09-05-task017-m10-typography-and-defects-review.md`; CI run `33963484000` green on all six legs. MK-059 implemented per its own resolutions: **102 of 152 `CAPTION` sites triaged** — 32 to the new `BODY_SMALL` with line-height, 71 to the new `LABEL` without — and **50 left at 12.0 deliberately**, which is decision 2 working as intended: it leaves M6 something real to judge rather than closing the question by assumption. Line-height applied only where it is a gain: `body` (1.4, looser than iced's silent 1.3 default) and `body_small` (1.35). `title` is a no-op at 1.3 and `heading`/`display` are tighter, so they are applied nowhere; the accessors exist, unused, with the reason recorded. `DISPLAY` stays 36.0 per resolution 1. **All four appearance defects fixed and verified live in four presets.** D-1/D-2 were one fix: `theme::dialog_style` now takes `width: 1.0` and `tokens_for(t).palette.border`, so we inherit snora's 0.34.0 contrast repair instead of overpainting it — confirmed on D-2's exact B2-1 failure case, the wizard in `high_contrast_dark`, which M8's capture recorded as having no discernible edge at all. D-3 fixed with `.style(theme::naked)` and `.width(Length::Fill)`. D-5's auto-scroll implemented in `update_and_dispatch` and verified **against the real scrollbar**, not only against index math — iced 0.14 exposes no scroll-into-view for a specific child, so it maps filtered index to relative offset and says so. `update` remains a pure reducer. `theme.rs` crossed the 500-line threshold and carries a recorded boundary decision. **A fifth defect surfaced live and is recorded as D-7.** |
| M6 — UX acceptance evidence | Implementing | MK-056 design accepted and implementation authorized by the project owner 2026-08-02, including resolution of all five review questions. Delivered as three units. **L1 cross-platform CI complete 2026-08-03**: all six legs green (Linux/macOS/Windows x stable/1.91) on run `30824280662` at commit `469e6cf`; evidence in `docs/src/development-and-gates.md`. Remaining: the preparation gate (`.git-exclude/tasks/dev-team/006-mk056-preparation-gate.md`), then L2 scripted GUI verification and L3 human sessions. Participant recruitment is the project owner's and is the critical path to R2. |
| R2 — Integration readiness | Not started | — |

## Defect register

Defects found by executing surfaces nobody had run — M8's capture (2026-09-04),
M9's live `wtype` run and M10's visual pass (2026-09-05). **None is a regression
from the milestone that found it**; every one is a pre-existing surface that had
never been exercised. Recorded here rather than folded silently into another
milestone.

### Open

| # | Defect | Severity | Disposition |
|---|---|---|---|
| **D-7** | **Two overlays can stack.** Executing the command palette's *Switch workspace* row opens the workspace menu **without closing the palette**; `AddRuleFromPalette` is the only row that self-closes. `Escape` resolves them one at a time in the coded priority order, but which overlay is on top is not something a user should have to reason about. Found live during M10's visual pass. | **S2** | **Fix before M6, minimally** — close the palette in the palette-reachable arms that open another overlay, exactly as `AddRuleFromPalette` already does. A few match arms, no new abstraction. **Do not** build a general single-modal-layer ordering here; that is recorded for the production project, where the overlay set is larger and a real total order earns its keep. Per MK-056's preparation-gate principle: known defects are fixed before sessions so participant time finds unknown ones. |
| D-6 | **`iced::event::listen_with` ignores capture status by design**, so the application sees key events a widget already consumed. Correct for `Escape`, which is why M9 adopted it, and **inert for everything else today**: the only overlap between our global key set and `text_input`'s captured set (`ArrowLeft`, `ArrowRight`, `Backspace`, `Delete`, `End`, `Enter` conditionally, `Escape`, `Home`) is `Escape` itself. It stops being inert if `.on_submit` is ever added to the palette's search field — `Enter` would then be captured *and* dispatched globally, executing the selected row twice — or if a future global shortcut lands on any other key in that set. | S3 (latent) | Recorded only. **Re-check whenever the global key set changes.** |

### Closed

| # | Defect | Closed by |
|---|---|---|
| D-1 | `theme::dialog_style` overpainted snora's dialog card — fill, radius and an 18%-black shadow with **zero border width**, so snora's 0.34.0 `palette.border` repair never rendered for us. Measured: byte-identical card edges at snora 0.28.0 and 0.42.0 across that repair. | **M10**, `4deadc2` — `width: 1.0` and `tokens_for(t).palette.border`, so the token's contrast work is inherited rather than overpainted. |
| D-2 | The new-workspace wizard's card had **no discernible edge** in `high_contrast_dark` (capture case B2-1, the one required case that failed). Same root cause as D-1: a full screen with no dim behind it, so the fill could not separate it and the 18% shadow composited to nothing on black. | **M10**, `4deadc2` — same fix as D-1. Verified on the exact failure case. |
| D-3 | The wizard's three collapsible section headers rendered as **primary-filled buttons** with near-unreadable hint text and a fill that did not wrap its content — `wizard.rs` built `section_head` with no `.style(...)` and no `.width(...)`. **Found by the project owner during M8's capture, with no case covering it.** | **M10**, `4deadc2` — `.style(theme::naked)` and `.width(Length::Fill)`. `theme::muted()` left untouched: the hint colour was never wrong, only what sat underneath it. |
| D-4 | The command palette could not be opened outside a workspace: `Ctrl+K` set `open = true` while `command_palette::view` renders only inside `AppView::Workspace`, so the state flipped and nothing drew it. | **M9**, `74e7f75` — option 2: the toggle is inert outside its one rendering context, turning a silent no-op into an honest one. |
| D-5 | Arrow-key navigation did not scroll the selection into view; MK-033 line 120's "correctly with the visible scroll" was met for index correctness only. **Declared by the implementer rather than found later.** | **M10**, `4deadc2` — `snap_to` from `update_and_dispatch`, mapping filtered index to relative offset since iced 0.14 exposes no scroll-into-view for a specific child. Verified against the real scrollbar, not only the index math. |

## Production-integration inputs from M11

Recorded because this repository exists to hand over a trustworthy specification,
not because any of it is scheduled here. Detail:
`.git-exclude/reviewed/2026-09-05-mk060-apimock-6-adoption-review.md`.

- **`TlsCertFile`/`TlsKeyFile` reload classification is owed a correction.** Our
  mapping says `Restart`; apimock-config 6.0.0 says `reload()`. Accepted here as
  stricter-not-wrong, but cert/key rotation is routine in production and their
  RFC 020 removed the restart precisely because it is. An unnecessary restart
  there is a real operational cost, unlike `ListenerPort = 0` which nobody does.
- **New engine capability with no `WorkspacePort` equivalent**, none of it a
  divergence — wholesale gaps: a fifteenth `EditCommand`
  (`UpdateRuleSetStrategy`, per-rule-set strategy override); **sixteen new match
  operators** (`UrlPathOp` 6→11, `HeaderOp` 9→13, `BodyOp` 18→25 — the four
  `Not*` families plus `Regex`/`NotRegex`, `MapHasKey`/`MapDoesNotHaveKey`,
  `StructuralContains`); `RespondPayload.json` as a third response mode; and
  `FileTreeRespectGitignore`. Our operator enums are **strict subsets** of the
  engine's, so the UI cannot express anything the engine will reject — the gap is
  unrealised capability, not risk.
- **`RulePayload.priority: Option<i32>` now matches our own prototype field
  exactly**, narrowing a long-standing `ReferenceGap` to an exact type match. A
  production adapter could wire it through `AddRule`/`UpdateRule` today.
  `weight` remains absent from the engine.
- **`apimock-routing` grew eleven production transitive crates** at 6.0.0 — a
  glob and gitignore-aware directory-walking stack plus a relaxed-JSON parser,
  traced to real new capability. `cargo audit` clean. Our production use of that
  crate is **four items**, none of which needs them; a shipped GUI carries the
  rest regardless. Recorded as a supply-chain measurement, not a complaint.

## Deferred beyond this roadmap

These remain outside the mockup stabilization programme:

- real `apimock_config::Workspace` file I/O and persistence in production
  targets;
- helper subprocess start/stop/reload/restart control;
- live trace UDS/TCP connection and reconnection;
- external-edit detection and conflict handling;
- remembered workspace/theme/locale/audience preferences;
- editable Rhai scripts and runtime validation. **Security constraint recorded
  2026-08-21:** apimock-rs's threat model notes that `service.middlewares` lists
  Rhai scripts the server compiles and runs, so **a configuration file is
  executable content**. The mockup does not open real files, so this does not
  arise here — but "open a workspace" becomes "open a directory someone sent
  you" in production, which is opening executable content by another name. The
  production integration must settle this deliberately, and apimock-rs has asked
  to be told what it decides;
- drag-and-drop rule ordering and transition animation;
- multi-user synchronization;
- mdBook documentation build and link validation. `docs/src` has no `book.toml`
  or `SUMMARY.md` and is therefore not a buildable book. This is accepted for
  the mockup and is a requirement of the production GUI project;
- continuous integration, including a cross-platform build matrix. Deferred
  here because this repository is short-lived and has no platform-specific
  code; required by the production project, which will be long-lived and will
  carry path resolution, subprocess lifecycle, and a trace transport whose UDS
  option is unavailable on Windows;
- packaging reproducibility and source-archive-matches-tree validation. These
  are moot under the no-release policy; if an archival snapshot is ever
  requested, note that no validation check exists for it.

They must be reconsidered only after R2, with threat modeling for new file,
process, socket, and script-execution data flows.
