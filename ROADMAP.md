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
- `apimock-config`, `apimock-routing`, and `apimock-server` 5.10.0 are published
  on crates.io with MSRV 1.91.0. The documented 5.10.1 GUI integration reference
  describes the intended contract but was never published; 5.10.0 is the
  authoritative executable artifact unless a newer release is explicitly
  adopted.
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
                                   ├─ M8 ─ M9 ─ M10 ─ M6 ──────┤
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
| 10 | M9 — Keyboard operability | M8's capture (which it would otherwise invalidate) |
| 11 | M10 — Typography and readability | M9; last appearance change before M6 |
| 12 | M6 — UX acceptance evidence | R1 GO; M8 before its live runs; M9 before its keyboard-only session; M10 before its readability probe |
| 13 | R2 — Integration readiness | M5, M8, M9, M10 and M6 |

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
| M8 — snora adoption | Blocked | **Code-complete at 0.28.0, one version bump and one human pass outstanding.** MK-058 Phases 1-3 implemented, reviewed and accepted as code-complete 2026-08-04; evidence: `.git-exclude/reviewed/2026-08-04-mk058-snora-0-28-adoption-implementation-review.md`. Commits `265e9b6` (P1), `8caa826` (P2), `2c191aa` (P3), `b0f62a2` (docs), against baseline `cef5e32`. Gate green; CI run `31229529161` green on all six legs. Phase 2 additionally fixed two pre-existing theme-detection bugs, one of which had no test coverage. **Blocking condition:** the acceptance evidence requires the live GUI, and `scripts/ux/probe.sh` returned `PROBE_RESULT=incapable` — the window is a native-Wayland surface invisible to `xdotool`, and niri's IPC exposes no input injection, so nothing *currently installed* can drive the app except a person. That is a provisioning statement, not an impossibility: niri does support `zwp_virtual_keyboard_manager_v1` and `zwlr_virtual_pointer_manager_v1`, so `wtype` plus a pointer tool would work here. Considered and deliberately deferred 2026-08-15 — the manual pass is what produces the reference screenshots any scripted pass would have to be validated against; reasoning in `.git-exclude/reviewed/2026-08-15-m8-visual-capture-approach-review.md` §3. **Phase 5 complete 2026-08-15:** snora 0.29.0 adopted in `5aeee57` — one line, no source change, whole `Cargo.lock` diff is four versions and four checksums with no transitive dependency moved; reviewed `.git-exclude/reviewed/2026-08-15-snora-0-29-upgrade-review.md`; CI run `31869542368` green on all six legs. Sequencing reversed 2026-08-15 on evidence from snora's migration guide, which established appearance-neutrality by semantic diff of the two tags; reasoning in MK-058 §7. **Task 015 commit 1 complete 2026-08-18:** snora 0.37.1 adopted in `c5f3cf2` — no source change, five crates including the new `snora-style`, nothing outside the family moved in `Cargo.lock`; 289 tests unchanged on both toolchains; reviewed `.git-exclude/reviewed/2026-08-18-snora-0-37-upgrade-commit-1-review.md`; CI run `32141644318` green on all six legs. **Sole remaining step: owner task 001** — roughly 10 minutes at the machine, with the fillable form at `.git-exclude/release-evidence/2026-08-12-m8-visual-capture/RESULTS.md`. Task 015 commit 2 (dropping `snora-widgets`) is deliberately held behind the capture. **The bump now precedes the capture.** snora 0.34.0 raises the dialog-card border from 1.28:1 to 3.12:1 in `light` (1.19:1 to 3.17:1 in `dark`; both high-contrast presets unchanged) after classifying the old value as an accessibility defect — and that border is the boundary of the dialog card Pass A photographs. Capturing first would record a surface we are about to replace, and the task's own "expected, not a fault" note was teaching the owner to accept the defect. **Re-sequenced four times, and now terminated.** A stop rule was written into the owner task after the third; snora triggered its exception clause once more with 0.37.0, which raised `DIM_ALPHA` 0.40 → 0.44 to repair a `light`-preset failure where the dialog card was distinguishable from its own dimmed backdrop at only 2.85:1. **snora now holds the matching commitment** — they will state when a release contains a rendered change, and a note that does not mention one leaves our baseline valid; they record 0.37.0 as the last, with nothing in `proposed/` touching a rendered surface. That is the terminating condition the sequence lacked: our stop rule only ever bound us. The capture also gained a required case: our new-workspace wizard is the one dialog not wrapped by snora's card, uses a border-less style with an 18%-black shadow, and in `high_contrast_dark` may repeat the compositing failure M8 exists to fix. **Owner:** dev team for (1), project owner for (2). No participant recruitment required — this is not a UX session. |
| M9 — Keyboard operability | Not started | **Authorized by the project owner 2026-08-15.** Task: `.git-exclude/tasks/dev-team/014-mk033-keyboard-operable-palette.md`. Implements RFC MK-033 as written — auto-focused search field, `Enter` executes, arrow-key row navigation — plus keyboard reachability of the mode picker, which closes MK-023's first-screen gap. **Why it is a milestone and not a bug fix:** a keyboard-only user cannot pass the application's first screen today, and MK-056 plans a keyboard-only session (line 267) rating accessibility failures that prevent completion as S1 Blocking (line 210). M6 cannot run its own protocol until this lands. **Scope is deliberately narrow:** MK-023 line 17 permits reachability "via Tab *or* the command palette", so the palette is the route; full Tab traversal, focus rings on every control and screen-reader landmarks stay with the production project, and MK-056 already lists iced-imposed accessibility gaps as a non-goal. **Sequenced after M8's capture**, which it would otherwise invalidate — auto-focus adds a visible cursor and ring to a Phase 3 capture surface. Evidence: `.git-exclude/reviewed/2026-08-15-wtype-probe-result-and-keyboard-contract-gap.md`. |
| M10 — Typography and readability | Not started | **RFC MK-059 accepted by the project owner 2026-08-15**, not yet implemented; task issued when its window opens. Adopts snora's six-role text scale, present in the pinned 0.29.0 and reachable with no upgrade. **The finding that motivated it:** 152 of 294 text-sizing call sites render at `CAPTION` 12.0 — snora's stated readability floor — because our scale jumps 16.0 to 12.0 with nothing between, and snora supplies two 14.0 roles for exactly that gap. Line-height is set nowhere in `crates/app/src`. Resolutions: `DISPLAY` stays 36.0 as a recorded divergence (a prior pass raised it from 32 for comfort); triage covers prose-bearing surfaces only, leaving genuine metadata at 12.0 so M6 can judge it on evidence; MK-056 gains a readability probe, **owed before M6's sessions run**. **Main implementation risk:** line-height 1.4 makes prose blocks taller, so fixed-height cards, dense lists and the bottom drawer need visual checking, not just compilation. |
| M6 — UX acceptance evidence | Implementing | MK-056 design accepted and implementation authorized by the project owner 2026-08-02, including resolution of all five review questions. Delivered as three units. **L1 cross-platform CI complete 2026-08-03**: all six legs green (Linux/macOS/Windows x stable/1.91) on run `30824280662` at commit `469e6cf`; evidence in `docs/src/development-and-gates.md`. Remaining: the preparation gate (`.git-exclude/tasks/dev-team/006-mk056-preparation-gate.md`), then L2 scripted GUI verification and L3 human sessions. Participant recruitment is the project owner's and is the critical path to R2. |
| R2 — Integration readiness | Not started | — |

## Deferred beyond this roadmap

These remain outside the mockup stabilization programme:

- real `apimock_config::Workspace` file I/O and persistence in production
  targets;
- helper subprocess start/stop/reload/restart control;
- live trace UDS/TCP connection and reconnection;
- external-edit detection and conflict handling;
- remembered workspace/theme/locale/audience preferences;
- editable Rhai scripts and runtime validation;
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
