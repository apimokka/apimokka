# apimokka — Stabilization roadmap

**Planning baseline:** 2026-07-15

**Current release:** v0.10.0

**Current decision:** NO-GO for production integration or a new release

**Permitted use:** internal UI/UX exploration with the mockup limitations disclosed

This roadmap turns the architecture preparation review into a dependency-ordered
stabilization programme. It defines schedule, milestones, review points, and the
RFC work that follows. Detailed design belongs in RFCs; this file remains the
programme-level source of truth.

The repository is a UI/UX mockup until the Integration Readiness milestone is
approved. It has no real workspace file I/O and no live apimock-rs server
connection. Do not present it as a production integration baseline before then.

## Outcome

The programme is complete when apimokka is a trustworthy executable
specification for a production GUI effort:

- match-test behavior is conformant with the supported apimock-rs operators;
- the editing boundary is either engine-isomorphic or has an explicitly designed,
  tested mapping;
- repository governance records agree and are mechanically checked;
- formatting, tests, builds, lints, and the security policy pass on the declared
  toolchains;
- central modules and tests follow the project's maintainability rules;
- the principal workflows have current visual, keyboard, and EN/JA evidence; and
- an independent architecture re-review records a GO decision.

## Planning assumptions

- One primary implementer is available for four delivery days per week. The
  fifth day is reserved for design, review response, gates, and evidence.
- Independent design review is planned to return within three working days.
  Longer review turnaround moves the dependent window rather than consuming
  implementation reserve.
- No planned leave is included. Reviewer, participant, or implementer absence
  reduces calendar confidence and must be recorded in the progress table.
- Independent review and UX preparation may run in parallel when they do not
  alter the same design baseline.
- Target dates are planning ranges, not release promises. An RFC review or failed
  gate may move later milestones.
- Rust 1.91 remains the declared MSRV until an approved RFC changes it. Current
  stable is an additional verification target.
- apimock-rs 5.10.1 is the authoritative integration contract unless a newer
  engine reference is explicitly adopted before the integration-boundary RFC.
- Existing v0.10.0 behavior remains usable for internal UX review while
  stabilization proceeds, but known match-test limitations must be disclosed.
- No production file I/O, subprocess control, trace socket, or Rhai editing is
  added by this programme. Those require a later production-integration roadmap.

Calendar confidence is **medium** through M2 and **low** from M3 onward until the
engine-reuse feasibility check, independent reviewer availability, and UX
participant availability are recorded. Each milestone includes design/review and
evidence time; implementation is not assumed to begin on its first day.

## Ownership and approval

These role assignments apply until the project owner records a replacement:

| Milestone | Accountable owner | Delivery owner | Independent reviewer | Evidence approver |
|---|---|---|---|---|
| M0 | Project owner (nabbisen) | Programme architect | Architecture auditor | Project owner |
| M1 | Project owner | RFC author / assigned implementer | Architecture auditor | Project owner |
| M2 | Project owner | RFC author / assigned implementer | Engine-conformance reviewer | Project owner |
| M3 | Project owner | RFC author / assigned implementer | Architecture auditor | Project owner |
| M4 | Project owner | Assigned implementer | Security/release reviewer | Project owner |
| R1 | Project owner | — | Architecture auditor | Project owner |
| M5 | Project owner | RFC author / assigned implementer | Architecture auditor | Project owner |
| M6 | Project owner and session coordinator | Assigned implementer / UX facilitator | UX/accessibility reviewer | Project owner |
| R2 | Project owner | — | Architecture auditor | Project owner |

The named delivery person must be recorded in the relevant RFC before
implementation begins. An author or implementer does not independently approve
their own milestone exit evidence.

## Schedule at a glance

| Milestone | Target window | Purpose | Release state |
|---|---|---|---|
| M0 — Planning approval | 2026-07-15 to 2026-07-22 | Agree scope, order, gates, ownership, and RFC queue | No release |
| M1 — Repository truth | 2026-07-23 to 2026-08-05 | RFC/review (1 week), repair/evidence (1 week) | No release |
| M2 — Match-test conformance | 2026-08-06 to 2026-08-26 | RFC/review (1 week), implementation/evidence (2 weeks) | No release |
| M3 — Integration boundary | 2026-08-27 to 2026-10-07 | RFC/architecture review (3 weeks), seam preparation and implementation/rework (3 weeks) | No release |
| M4 — Quality and security gates | 2026-10-08 to 2026-10-21 | Close residual issues and capture final evidence | Stabilization candidate |
| R1 — Blocking re-review | 2026-10-22 to 2026-10-28 | Frozen-input re-review of architecture findings B1–B5 | Release decision |
| M5 — Maintainable structure | RFC: 2026-10-08 to 2026-10-21; delivery: 2026-10-29 to 2026-11-18 | Split oversized modules and tests safely | No new behavior |
| M6 — UX acceptance evidence | Preparation: 2026-10-08 to 2026-11-18; sessions/fix/re-test: 2026-11-19 to 2026-12-09 | Validate usability, visual, input, and EN/JA outcomes | Readiness candidate |
| R2 — Integration readiness | 2026-12-10 to 2026-12-16 | Frozen-input final evidence review and baseline decision | GO/NO-GO |

The critical path is M0 → M1 → M2 → M3 → M4 → R1 → M5/M6 → R2. M5 RFC
design and M6 protocol/participant preparation may overlap M4 and R1 after the
M3 boundary is stable. Their implementation and acceptance findings do not
alter frozen R1 inputs. R2 requires both M5 and M6.

R1 and R2 each reserve one full week. Inputs freeze at the end of the preceding
window; material changes during review invalidate affected evidence and require
targeted re-review.

## Programme-wide gate cadence

Quality and security checks start at M1; M4 is the final remediation and
evidence milestone, not the first time they run.

At every implementation milestone checkpoint and exit on current stable:

```sh
cargo fmt --check
cargo test --workspace --lib --bins --locked
cargo build --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Rust 1.91 build and test checks run at the M1 baseline, after every dependency or
feature change, and at each implementation milestone exit:

```sh
cargo +1.91 test --workspace --lib --bins --locked
cargo +1.91 build --workspace --locked
```

`cargo audit` runs at the M1 baseline, whenever `Cargo.lock` or dependency
features change, and at milestone exits. Any approved advisory exception must be
repository-owned and machine-checked. Its record must include advisory ID,
dependency path, exploitability analysis, decision owner, approval date, expiry,
and remediation trigger. The policy check must fail for an unapproved or expired
exception even when `cargo audit` continues to print the advisory.

## Milestones

### M0 — Planning approval

**Goal:** establish one agreed stabilization plan before detailed RFC design.

Deliverables:

- this roadmap approved or revised by the project owner;
- milestone scope, ordering, and gate policy accepted;
- RFC topics ordered for detailed design;
- accountable owner, delivery role, independent reviewer, and evidence approver
  assigned for every milestone;
- reviewer and UX-participant availability risks recorded.

Exit gate:

- the project owner approves the roadmap review request;
- open planning objections are recorded here rather than left implicit.

### M1 — Repository truth

**Goal:** make the repository a coherent source of truth before behavior and
architecture changes accumulate.

Scope:

- create the next-numbered M1 RFC in `proposed/` as the bootstrap record; its
  implementation repairs the index that will then include the RFC itself;
- rebuild `rfcs/README.md` from the files on disk, including every implemented
  and archived RFC;
- correct shipped versions and RFC Status fields against `CHANGELOG.md` and the
  workspace version;
- record the snora de-vendoring transition;
- update stale roadmap, architecture metrics, and historical-document warnings;
- add a small automated RFC integrity check for index coverage, unique numbers,
  status/folder agreement, and resolvable local links;
- capture programme baseline results for current stable, Rust 1.91, and the
  dependency audit according to the programme-wide cadence.

Exit gate:

- governance documents agree on versions and current status;
- the integrity check passes locally;
- documentation review confirms that historical material is visibly labeled;
- baseline stable/MSRV/audit evidence is recorded without overclaiming passes;
- no product behavior changes are included.

### M2 — Match-test conformance

**Goal:** ensure the Test Rule workflow never reports a false match or false
non-match for an unsupported or incorrectly evaluated condition.

Scope:

- choose real matcher reuse or exact local equivalence in a detailed RFC;
- implement URL wildcard, header regex/wildcard, body regex, and exact i64
  semantics, or explicitly mark an operator unsupported and return a distinct
  `Unsupported`/`Indeterminate` outcome or precise `TestRuleResult::Error`;
- add positive and negative conformance cases for every supported URL, header,
  and body operator, including integers above 2^53;
- compare conformance results against the adopted apimock-rs version rather than
  treating a local reimplementation as its own oracle;
- reconcile README and UI claims with the supported behavior;
- disable or explain any intentionally unavailable operation.

Exit gate:

- the operator conformance matrix has executable coverage;
- no skipped or best-effort branch can produce `Matched` or `NoMatch`;
- supported operators have positive and negative results verified against the
  adopted engine; unsupported operators produce only the explicit
  indeterminate/error outcome;
- test-rule limitations, if any, are visible in both UI and documentation;
- existing screen-flow tests remain green.

### M3 — Integration boundary

**Goal:** resolve the mismatch between the mock reducer and the authoritative
apimock-rs editing contract before production integration begins.

Decision required:

1. **Engine-isomorphic adapter:** implement an in-memory workspace adapter whose
   `apply(EditCommand)` boundary, condition identities, payload semantics,
   diagnostics, changed nodes, and reload hints mirror apimock-rs; or
2. **Explicit UI mapping:** retain a UI-specific model and define a tested mapping
   to and from the engine contract, documenting every non-isomorphic conversion.

Required design coverage:

- stable per-condition `NodeId` addressing;
- `Option<Vec<_>>` preserve/clear/replace semantics;
- snapshot refresh and selection stability;
- apply errors, diagnostics, and reload/restart hints;
- undo/redo ownership and inverse-command behavior;
- external-edit and session-lifetime boundaries for later production work.

Preparation slice before adapter implementation:

- extract the inline `app.rs` test modules into the project-required test-file
  structure;
- create only the reducer/adapter module seam required by the approved boundary;
- keep routes, fixtures, and general structural cleanup in M5;
- preserve behavior and run the continuous gates after each mechanical move.

Exit gate:

- the approved RFC decision is implemented with mapping/adapter contract tests;
- app mutations use the approved boundary rather than undocumented direct
  snapshot mutation;
- model documentation makes no unsupported compatibility claim;
- Add Rule Set and undo/redo behavior have honest, tested semantics.

### M4 — Quality and security gates

**Goal:** close residual gate issues and capture release-decision evidence after
continuous checking throughout M1–M3.

Scope:

- clear all warnings in library, binary, and test targets;
- make clippy pass with warnings denied;
- verify the final candidate on Rust 1.91 and current stable with locked
  dependency resolution;
- resolve the `quick-xml` advisories through dependency changes, feature-surface
  reduction, or a time-bounded exception with path-specific exploitability
  analysis and owner/expiry;
- decide whether iced's `highlighter` feature is justified;
- finalize exact local gate commands and expected evidence;
- add CI or a repository-owned repeatable gate script.

Minimum blocking commands:

```sh
cargo fmt --check
cargo test --workspace --lib --bins --locked
cargo build --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo audit
```

Exit gate:

- all applicable commands pass on the required toolchains, or an explicitly
  approved, machine-checked security exception is current and scoped;
- evidence is captured in the repository or review package;
- documentation does not claim a broader gate than was actually observed.

### R1 — Blocking architecture re-review

**Goal:** independently verify that findings B1–B5 are resolved.

Required inputs:

- M1 governance evidence;
- M2 operator conformance matrix and test output;
- M3 adapter/mapping RFC, implementation, and contract tests;
- M4 toolchain and security gate evidence.

Decision:

- **GO:** a clearly labeled `v0.11.0-stabilization.N` pre-release may be
  prepared and M5/M6 continue toward an integration-ready baseline. Its release
  notes must state `NOT INTEGRATION READY`;
- **CONDITIONAL GO:** only listed non-production work may proceed;
- **NO-GO:** return unresolved blockers to the owning milestone.

### M5 — Maintainable structure

**Goal:** bring the codebase back within the project's file-organization rules
without changing behavior.

Scope:

- complete the `app.rs` split by state/reducer domain, building on the narrow
  test extraction and reducer/adapter seam completed in M3;
- split `screens/routes.rs` by sidebar, rule-set configuration, editor,
  fallback editor, script viewer, and trace activity boundaries;
- split `model/mock.rs` fixtures and tests;
- remove dead placeholders and misleading fake commands;
- preserve public crate boundaries and behavior through focused tests.

Exit gate:

- no implementation file exceeds 500 ELOC without a reviewed written reason;
- files above 300 ELOC have an explicit cohesion justification or follow-up;
- implementation files contain no inline test modules where the project rule
  requires separate test files;
- M4 gates remain green after every split.

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
  window sizes, and input methods. Full Cartesian coverage is not required, but
  every dimension must appear in a risk-based combination.

Outcome criteria:

- each primary scenario records completion, facilitator rescue, critical-error
  count, elapsed time where useful, and participant observations;
- no release-blocking scenario requires facilitator rescue;
- severity rules distinguish blocking workflow failure, serious confusion or
  accessibility failure, and non-blocking polish;
- fixes receive targeted re-test with the same scenario and build identity;
- deferral is allowed only for a named owner, rationale, user impact, and target
  milestone, and never for a release-blocking workflow failure.

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
- the no-I/O mockup boundary and the next production-integration risks are
  documented;
- the project owner approves the release/review package.

A GO decision permits creation of a separate production-integration roadmap for
file I/O, subprocess lifecycle, trace transport, external edits, persistence,
and Rhai editing. It does not itself authorize those features.

## RFC design queue

RFC identifiers are assigned only when files are created. After M0 approval,
prepare detailed RFCs in this order:

1. **Repository governance repair and automated RFC integrity** — M1.
2. **Test-rule matcher conformance** — M2.
3. **GUI editing boundary and apimock-rs mapping/adapter** — M3.
4. **Release gates, dependency policy, and security exceptions** — M4.
5. **Reducer, routes, fixtures, and test modularization** — M5.
6. **UX and accessibility acceptance protocol** — M6.

The first four RFCs form the blocking re-review package. RFCs 5 and 6 may be
designed while earlier implementation proceeds, but must not assume an
unapproved editing boundary.

Optional developer handoffs should be created only where the RFC is too large
to implement safely from the design alone. The integration-boundary and
modularization RFCs are expected to benefit from task breakdowns and acceptance
checklists; the governance repair likely does not. M3's package should include a
decision log, engine/UI mapping matrix, task breakdown, and acceptance checklist.
M5's package should include a file-move sequence and regression checklist. M2
should include an operator-conformance matrix, with a full developer handoff only
if the accepted matcher-reuse approach needs one.

## Release policy during stabilization

- No release is cut from the current NO-GO baseline.
- A stabilization pre-release is considered only after R1 records GO and all
  release gates applicable to that revision are observed passing. Its reserved
  label is `v0.11.0-stabilization.N`, and its release notes must prominently say
  `NOT INTEGRATION READY`.
- Integration-ready labeling is reserved for R2 GO.
- Release archives must follow the project rule: files at archive root, version
  suffix in the archive name, and no `.git`, `.git-exclude`, or `target` content.
- Do not create commits, tags, archives, or pushes without explicit project-owner
  authorization for that task.

## Risks and schedule controls

| Risk | Schedule effect | Control |
|---|---|---|
| Engine semantics differ from the 5.10.1 reference | M2/M3 rework | Confirm the authoritative engine version at RFC start and test against it |
| Adapter decision expands into production I/O | M3 overrun | Keep I/O and live server control explicitly out of scope |
| Transitive advisories have no immediate upstream fix | M4 delay | Evaluate feature removal first; otherwise require a scoped exception with expiry |
| Structural split overlaps behavior changes | Regression risk | Complete M2/M3 behavior first; keep M5 behavior-neutral |
| Reviewer or UX participant availability | R1/M6/R2 delay | Confirm R1/R2 reviewer before M3 implementation; book M6 roles during protocol design |
| MSRV dependency incompatibility | M4 delay | Run Rust 1.91 checks early, not only at milestone exit |

## Progress tracking

Milestone status is one of `Not started`, `Designing`, `Implementing`,
`In review`, `Complete`, or `Blocked`. Update this table whenever a milestone
changes state; detailed task progress belongs in its RFC or handoff.

Allowed transitions are:

```text
Not started → Designing → Implementing → In review → Complete
                       ↘ Blocked ← any active state
```

`Complete` means the independent reviewer accepted the exit-gate evidence and
the evidence approver recorded acceptance. Finishing implementation alone moves
the milestone to `In review`, not `Complete`. A blocked milestone records the
blocking condition, owner, and next decision date.

| Milestone | Status | Decision/evidence |
|---|---|---|
| M0 — Planning approval | Complete | Approved 2026-07-15; evidence: `.git-exclude/reviewed/2026-07-15-apimokka-stabilization-roadmap-m0-confirmation-review.md` |
| M1 — Repository truth | Complete | Accepted 2026-07-15; MK-051 Implemented (Unreleased); evidence: `.git-exclude/reviewed/2026-07-15-rfc-mk051-repository-truth-closure-confirmation-review.md` |
| M2 — Match-test conformance | Complete | Accepted 2026-07-16; MK-052 Implemented (Unreleased); implementation evidence: `.git-exclude/reviewed/2026-07-16-rfc-mk052-test-rule-matcher-conformance-implementation-second-rereview.md`; dependency-policy evidence: `.git-exclude/reviewed/2026-07-16-rfc-mk052-compatible-manifest-lockfile-authority-amendment-rereview.md`; closure evidence: `.git-exclude/reviewed/2026-07-16-rfc-mk052-closure-confirmation-rereview.md` |
| M3 — Integration boundary | Complete | Accepted 2026-07-23; MK-053 Implemented (Unreleased); integrated implementation evidence: `.git-exclude/reviewed/2026-07-22-rfc-mk053-integrated-implementation-review.md`; closure evidence: `.git-exclude/reviewed/2026-07-23-rfc-mk053-closure-confirmation-review.md` |
| M4 — Quality and security gates | Not started | — |
| R1 — Blocking re-review | Not started | — |
| M5 — Maintainable structure | Not started | — |
| M6 — UX acceptance evidence | Not started | — |
| R2 — Integration readiness | Not started | — |

## Deferred beyond this roadmap

These remain outside the mockup stabilization programme:

- real `apimock_config::Workspace` file I/O and persistence;
- helper subprocess start/stop/reload/restart control;
- live trace UDS/TCP connection and reconnection;
- external-edit detection and conflict handling;
- remembered workspace/theme/locale/audience preferences;
- editable Rhai scripts and runtime validation;
- drag-and-drop rule ordering and transition animation;
- multi-user synchronization.

They must be reconsidered only after R2, with threat modeling for new file,
process, socket, and script-execution data flows.
