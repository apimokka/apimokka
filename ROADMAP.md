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
- **snora is frozen at 0.42.0 through M6**, decided 2026-09-12 and recorded in
  MK-058. 0.48.0 is current; snora's own note states neither release since 0.46
  changes any API, rendered appearance, or feature resolution, and we are six
  unreviewed minors behind, so adopting it now is cost with no named benefit at
  the moment the only remaining work is human acceptance sessions. The freeze
  protects the build identity MK-056 requires: M8's capture and M10's appearance
  verification both describe 0.42.0. It lifts on M6 completion, on an advisory
  against snora's own code, or on an M6 defect whose fix needs a later snora.
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
M0 ─ M1 ─ M2 ─ M3 ─ M4 ─ M7 ─ R1 ─┬─ M5 (complete) ──────────────────────────────┬─ R2
                                   ├─ M8 ─ M11 ─ M9 ─ M10 ─ t021 ─ M12 ─ M6 ─ M13 ┤
                                   └─ production-integration roadmap (draft only)

                                   participant recruitment ─────────────┘
                                   (owner, runs across t021 and M12)
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
| 13 | task 021 — appearance remediation (D-13, D-14) | M10; **before M6**, since it changes appearance M6 would otherwise record |
| 14 | M12 — Keyboard path through the primary scenarios (D-12) | task 021, for a stable appearance baseline; **before M6's keyboard-only session** |
| 15 | M6 — UX acceptance evidence | R1 GO; M8 before its live runs; M11 before it validates an editing surface; M9 before its keyboard-only session; M10 before its readability probe; **task 021 and M12** |
| 16 | M13 — apimock-rs 6.2 adoption | **after M6** — it can move Test Rule, which the sessions exercise; **before R2**, so the specification describes the contract production will build against |
| 17 | R2 — Integration readiness | M5, M8, M11, M9, M10, M6, **task 021, M12 and M13** |

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

## Remaining sequence, priority, and delivery cycles

Added 2026-10-06. The programme had order and prerequisites but no stated
**cycle structure** and no priority among the work that is left. Both are the
architect's to set and the project owner's to authorize.

### What a delivery cycle is here

**This programme produces no release, so the cycle unit is an authorized
increment, not a version.** One cycle is:

```text
architect issues task  →  dev team implements  →  architect reviews
     →  OWNER AUTHORIZES  →  push  →  CI six legs  →  register + roadmap updated
```

**Each cycle has exactly one authorization point.** That is deliberate: it is
the smallest number that keeps the owner in control of what reaches the remote
without making them the bottleneck on engineering judgement. Everything before
it is mine; the authorization is theirs; everything after it is mechanical and
verified.

A cycle is **closed** only when the register and this file both describe the
tree. Four milestones were accepted with their RFC left in `proposed/`, which is
why that clause is written down rather than assumed.

### The remaining cycles

**Renamed 2026-11-08** from *"the five remaining cycles"*: C1 and C2 are
closed and **C6 was added on project-owner authorization of 2026-11-07**, so
four remain. The count was never the point; the order is.

**Amended 2026-11-09 — C7 added, and the labels no longer run in order.** The
rows below are sorted by the **Priority** column, which is what carries order
here; the `C` numbers are stable identifiers assigned when a cycle was created,
and renumbering them would break every reference in the reviews and the
register. **C7 — snora adoption — therefore sits between C4 and C5**: after M6
because a snora bump changes what a participant sees, and before R2 because
certifying a specification against an appearance layer fifteen minors behind
production is the error the C4-before-C5 clause already names for the engine.
R2 moves to P4 and publication to P5; neither changed in substance.

| Cycle | Content | Priority | Exit evidence | The one authorization |
|---:|---|---|---|---|
| ~~**C1**~~ | task 021 — D-13 muted-text contrast, D-14 duplicate severity glyph | — | **CLOSED 2026-10-31.** AA asserted in all four presets; one glyph source in `crates/model`; captured by task 022 with no owner clicks. | done |
| ~~**C2**~~ | M12 — the keyboard interaction model (D-12). **CLOSED 2026-11-03.** External design accepted 2026-10-31, recorded in MK-023. **All ten slices complete** (`dbf947c`, `926a4fc`, `cc56256`, `4ba6552`, `556102b`, `1258ce9`, `0af9e09`, `7df87ad`, `83b5ffc`, `9d405e1`) — the plan named six and evidence extended it four times, each time from a slice looking where it was told to look and finding the layer above it. D-6 closed, both dialogs given a safe default action, and the region/cursor model built — `Tab` between regions, arrows within, the tab bar keyboard-operable and the mode picker's special case removed. D-15 and D-17 closed early by tasks 023 and 025. **Slice 3 closed D-12's headline symptom**: a keyboard-only user now reaches an open workspace with **no pointer event at all**, which also retired the mixed-backend evidence tension carried since task 026. **All six primary scenarios are now keyboard-completable** — slice 6 closed the last two, which the original slice plan had never assigned. **Slice 7 closed D-16, D-18, D-19 and D-20 and added the palette routes, but found scenario 5 broken** — so MK-023 did not move to `done/`. **Slice 8 (`7df87ad`) closed D-12**: the fallback editor, the rule-set configuration pane, the drawer and the confirm dialog became regions, the three pre-existing scroll dispatches were folded into one tested pattern, and **scenario 5 was captured end to end, keyboard only**. **Its census is what ends the accumulation** — walked from source, it found seven further surfaces holding controls no key reaches, **three of them named in MK-023's own acceptance clause** (*"every control it lists must be reachable"*): Test Rule's method buttons, and the first two and the last entries of the Routes focus order. An eighth was found in review. **Slice 9 (`83b5ffc`) closed D-23**: every control the acceptance clause names is now reachable, the census became a test, `scripts.rs` was deleted, and `App::topmost_overlay()` became the one overlay-precedence source — which also **corrected the capture geometry for the whole cycle** (the nested compositor's own column width and output scale, not the host-side resize, were what slice 8's half-width frames came from). **Slice 10 (`9d405e1`) closed D-22 and D-26** and ended the cycle: `topmost_overlay()` now reports only overlays the current view draws, and the confirm dialog is drawn wherever it is raised — it had never been drawn on the Dashboard or in the Wizard at all, for pointer users as much as keyboard ones. **MK-023 is `done/`.** Ten slices: the plan named six and evidence extended it four times, each time from a slice looking where it was told to look and finding the layer above it — region inside overlay inside view, with `App::view` the root and no fourth container. **Slice 10's capture was run by the architect in review**, the dev team's rig having been blocked by niri writing its screenshots asynchronously, so a reader that did not wait saw the previous frame and read it as a frozen application. | — | **Met.** All six scenarios captured by injected keys with **no pointer click**; D-6, D-12, D-16, D-22, D-23 and D-26 closed; the census holds as a test (`app/control_census.rs`); MK-023 in `done/`. | done |
| **C3** | M6 — the critical path. **The appearance layer is clear** (task 036 closed D-21, D-25, D-27 on 2026-11-04), and MK-056's readability probe was amended 2026-11-04 — **accepted by the project owner** — to record the text *role* as well as the size tier, so the sessions answer both halves of the elder-readability question without being asked a leading one. **Prerequisites, and only the first is the owner's**: participant recruitment (task 002, owner); **task 037**, the four mandatory L2 combinations; and **task 038**, the defect task 037 found. **Task 037 ran 2026-11-07** (`1634aed`) — the first time any of the four had been exercised — and produced three passes, one fail and one incomplete: **no clipping at 880×700 in any configuration, Japanese included**, which was the risk named first; **combination 2 failed and is now D-29 (S1)** — the keyboard cursor paints nothing in either high-contrast preset, found by the combination that exists for exactly that; and **combination 4 has not yet exercised its own purpose**, decision 7's *"Expert shows every control at once, and this is where that density fails"*, because both narrow runs captured Routes in its empty state and the rule editor has never been rendered below 1280. Five harness bugs were found and fixed in the building, each of which would have produced silently wrong evidence. **Booking waits on D-29's fix and on those two re-runs; recruiting does not.** The session pack's §0, withdrawn 2026-09-17 over D-12, is **restored**. **Authorized 2026-11-07:** task 038 proceeds, and the four local commits may be pushed. Nothing else in C3 changed — the S1 is still what stands between this cycle and its exit evidence. **2026-11-08:** **task 038 accepted** (`3ac0e20`) and **D-29 closed** — the cursor ring is visible in both high-contrast presets, verified in the decoded pixels of its own evidence as well as by a guard that fails without it. **Task 037's two re-runs are now unblocked** and are what C3 waits on besides recruitment. The review's own measurement opened **D-30 (S2)** — in high contrast *selection* is now the barely-drawn state — which does **not** block booking and is task 039. **Task 039 accepted 2026-11-08** (`7f26ba1`) and **D-30 closed** — high-contrast selection now carries its state by width (3 px token against 1.5 px), verified in the pixels alongside the cursor ring. **C3 now waits on exactly two things:** recruitment (task 002, owner) and task 037's two re-runs, of which the 880×700 editor surface needs nothing and **combination 2 needs the host rebooted** (see the risk table). **Task 037 round 2 (`43d3cad`) accepted 2026-11-08 for §9.2 and §9.3** — the rule editor and Test Rule captured at 880×700 in both narrow combinations, and the frame manifest built: every kept frame's MD5 and measured ground pixel, failing the run on a duplicate or a theme its pixels contradict. The architect tested that mechanism against task 039's actual bad frames and it catches both halves of that failure; task 039's directory is cleaned and its fifteen manifest rows re-verified against the files. **The editor at 880×700 found what decision 7 said it would**: **D-31 (S1)** — the Method group clips PATCH and DELETE, unreachable by pointer — and **D-32 (S2)**. **Task 040** issued; D-31 blocks booking. **§9.1, combination 2's wgpu re-run, stays open and is blocked on the host reboot**, verified by the architect (kernel `7.3.1` running, its module tree replaced at 09:14). **Task 040 accepted 2026-11-08** (`c046485`): **D-31 and D-32 closed** — the method and mode groups wrap, all six methods drawn at 880×700 in both locales, and the manifest self-test now runs **inside the release gate**, the first capture-integrity mechanism in this programme that is itself guarded. The row walk the task required found a worse defect than the two it fixed: **D-33 (S1)**, the condition rows collapse and a header value the rule matches on is not drawn — **task 041**, option (a) decided. **Booking is now blocked on D-33, not D-31.** **The host was rebooted 2026-11-08, so §9.1's wgpu re-run is unblocked** and task 041's captures run on wgpu. **Task 041 accepted 2026-11-08** (`6958496`): **D-33 and D-34 closed**, verified on nine **wgpu** runs — the first since the reboot — with 55 manifest rows re-checked and the control census passing unmodified. **No defect above S2 is open on any surface M6 exercises** (D-24 S2, D-28 S3, D-35 S3 all sit after the sessions). **C3 now waits on exactly two things, neither a defect:** task 037 §9.1 — combination 2's `wgpu` re-run, the one mandatory combination that has never recorded a pass — and participant recruitment (task 002, the owner's). **Task 037 §9.1 accepted 2026-11-08** (`7439269`) and **task 037 is complete**: combination 2 records a pass on **wgpu**, so **all four of decision 7's mandatory combinations now have a recorded L2 pass** and MK-056's L2 acceptance criterion is met. The ring is proved against round 1's own frames — the same screen after one Tab went 14 249 → 15 093 primary pixels, **+844**, the switcher's ring that round 1 could not produce — and `06` shows cursored, selected and unselected rows in one frame in both HC presets. The rig's strict HC check now fails a Tab that changes nothing, and the 880×700 condition evidence task 041 could not take is now captured through the application's own `scroll_snap_target`. **One finding, D-36 (S2)**: text-field focus is colour-only in all four presets and clears the 3:1 state floor only in Light — task 042, to land before the first session but not blocking booking. **C3's only remaining prerequisite is participant recruitment (task 002), the project owner's.** | **P1** | two session logs, findings classified, no unresolved S1, readability probe answered | participants confirmed, and sessions begin |
| **C4** | M13 — apimock-rs 6.2 adoption | **P2** | MK-052 and MK-055 re-run, both oracle guards re-pinned, divergences re-classified | push |
| **C7** | **snora adoption beyond 0.42**, as **RFC MK-062** (number reserved; drafted after M6's findings are classified, not before). **snora 0.57.0 shipped 2026-11-08 and we are fifteen minors behind.** Nothing in it breaks us — assessed 2026-11-09 against our actual surface, which is `design::render`, `AppLayout`/`Dialog`/`Sheet`, `design::theme`, `Tokens` and two style helpers; 0.57's two breaking removals are toast APIs and both live deprecations are outside it. **Two things in 0.43–0.57 do matter**: 0.54's **two AAA presets**, which bear directly on the owner's *"kinder to elder people"* question that MK-059 left to evidence and MK-056's probe was amended to answer; and 0.55–0.57's overlay/keyboard semantics — *"overlays block the keyboard as they block the pointer"*, focus recovery, and an Escape message per overlay — **which this programme built for itself** in `App::topmost_overlay()` and MK-023. So adoption is a design review against our own model, not a bump. **Runs after M6 because a snora bump changes what a participant sees** (the C1/C2-before-C3 clause, and snora's own 2026-10-20 withdrawal: *"releases since 0.42.0 changed default-path appearance"*), and **before R2** because certifying a specification whose appearance layer is fifteen minors behind production reproduces exactly the error the C4-before-C5 clause names. Assessment: `.git-exclude/reviewed/2026-11-09-snora-0-57-0-assessment.md`. **Authorized by the project owner 2026-11-09, conditionally — *"if the upgrade is properly in control in schedule"* — and the control is this, in three steps with two owner gates.** **Step 1, the survey (read-only).** One dev-team task reads each guide's *"Who is affected"* table from 0.43 to the current release against the surface list above, and reports **the size**: which rows apply, what each costs, and which of snora's new overlay/keyboard semantics collide with `App::topmost_overlay()` and MK-023. It changes no file under `crates/` and no pin. It can run **during recruitment**, when it competes with nothing on M6's path, or after M6 — the owner's call, and it is **gate 1**. **Step 2, the decision.** MK-062 is drafted *from the survey's findings plus M6's*, and goes to the owner with the size attached. **It is allowed to say "not yet"**: if the survey finds the adoption would replace our keyboard/overlay model rather than sit beside it, that is a design question with a number on it, and it returns to the owner instead of being decided inside a bump — the same stop the C2 audit was given, which used it twice. **Gate 2** is the owner authorizing the implementation. **Step 3, implementation in slices**, each with its own review, ending in M8's and M10's capture cases re-run against the new pixels. **The drift control between now and then:** every new snora minor gets a one-paragraph assessment against the surface list, appended to MK-058 on receipt of its notice — the architect does it without a relay unless it contains a decision. That is what keeps "fifteen behind" from silently becoming twenty. | **P3** | the survey's size report; then the guides' rows answered, the four presets re-measured, and M8/M10's capture cases re-run against the new pixels | **two**: the survey, then the implementation once its size is known |
| **C5** | R2 — integration readiness | **P4** | GO or NO-GO against R2's listed criteria | the decision itself |
| **C6** | **Publication readiness** — the packaging programme, as **RFC MK-061** (number reserved; drafted after R2 GO, not before). **Authorized in order, not in position: it runs after production integration, never from this mockup.** Its ordered deliverables, each a task: the version decision and the first changelog cut; licence and metadata completeness (LICENSE and NOTICE inside every package — today they are in none, which Apache-2.0 §4 does not permit in a distribution — plus `keywords`, `categories`, `homepage`, `documentation`, and the stale snora version claims corrected — `README.md` lines 16 and 68 and `docs/src/architecture.md` lines 14 and 25 say **0.28**, `docs/src/designer-brief.md:369` says **0.8**, against a tree pinned at **0.42.0** since 2026-09-04); the declared runtime dependency set measured on a clean container rather than asserted; desktop integration (`.desktop` entry, icon set, man page — none exist); a tag-triggered release workflow producing the `apimokka-bin` input with checksums, plus the Clippy, `fmt` and gate legs CI does not run today; a packaging validation check under `scripts/`, so *"the package installs and the installed thing runs"* is gated like every other claim here; and only then the tag, crates.io, and AUR. **Rehearsed 2026-11-07, so the gaps above are measured:** `cargo package` succeeds for `apimokka-i18n` (9 files) and `apimokka-model` (38 files) and fails for `apimokka` only because its path dependencies are not yet on the registry — the ordinary publish order `i18n → model → app`, which means **the first publication is also the first test of its third step**. MK-017 (`archive/`, superseded) is the prior art and is not a substitute. | **P5** | the packaging validation check passes against a built package, and the owner authorizes the tag | the tag, and each registry push |

**Participant recruitment is not a cycle. It is an unblocked prerequisite**, and
it runs alongside C1 and C2 rather than after them. It is first in priority
despite producing no artifact, for two reasons that are both about dependency,
not duration:

- **nothing blocks it.** It has no prerequisite in this repository, so there is
  never a reason for it to be waiting;
- **nothing substitutes for it.** It is the only prerequisite in the programme
  that cannot be produced from inside the repository, so no amount of
  engineering progress moves it, and every other cycle can be closed with it
  still outstanding.

**Priority here means order of attention, never a calendar position.** This file
assigns no dates, durations, or windows — see its opening paragraph. A cycle is
ready when its prerequisites are closed and not before; it is finished when its
exit evidence exists and not before. Nothing in it is scheduled against time.

### Why this order and not another

- **C1 before C2** — both touch `theme.rs` and the shell views. C1 is a style
  fix with measured targets; C2 is a feature. Landing the small one first gives
  C2 a stable appearance baseline and keeps two changes out of one review.
- **C1 and C2 before C3** — both change what a participant sees. Running
  sessions first would record evidence against a build we then changed, and
  MK-056 requires findings to be re-tested against a recorded build identity. It
  would also spend participants rediscovering D-12, which is the exact waste the
  preparation gate exists to prevent.
- **C3 before C4** — `apimock-routing` supplies Test Rule's matcher primitives,
  and *"edit a rule's conditions and verify the change with Test Rule"* is a
  primary scenario. An engine change between the evidence and the sessions would
  make a finding unattributable. The engine cannot change a pixel, so unlike
  snora it poses no risk to M8's or M10's evidence — only to M6's.
- **C4 before C5** — R2 certifies a specification. Certifying one validated
  against 6.0.0 when production will build against 6.2 or later reproduces the
  error MK-060 was adopted to avoid.

### The D-12 scope decision, decided

D-12 is S1 and MK-056 forbids deferring an S1, so the choice was between
building a keyboard path and declaring keyboard-only operation out of scope for
this mockup. **The recommendation is to build it, bounded to MK-056's six
primary scenarios** — not to general keyboard operability.

- MK-023 line 17 already permits this shape: every control reachable *"via Tab
  **or the command palette**"*. Tab is not available — iced 0.14 gives focus only
  to `text_input` and `text_editor`, so no traversal we could wire would reach a
  `button` or a `pick_list`. The palette and M9's selection-plus-`Enter` idiom are
  the route, and they already exist.
- Scoping by **scenario** rather than by control is what makes it bounded. The
  known blocks are Welcome's two actions, Dashboard's workspace rows, and
  Settings' locale and audience-mode controls; the rest comes from the audit.
- **C2's first deliverable is that audit, and it is allowed to stop the cycle.**
  If the per-scenario control inventory comes back unbounded, the finding is
  reported and the scope question returns to the owner with a size attached
  instead of a guess.

**Settled 2026-11-03.** It was built, and **D-12 closed** with MK-023 slice 8
(`7df87ad`): all six primary scenarios captured keyboard-only, zero pointer
events. The audit did not come back unbounded, and it did stop the cycle twice
— slice 5's omissions and slice 6's two unassigned screens — exactly as this
clause allowed for. **Scoping by scenario was right and it was not the whole
of MK-023's acceptance**: that clause also requires every control the audit
inventory lists to be reachable, and slice 8's census found seven surfaces
where it is not. That remainder is **D-23**, at its own severity, closing with
slice 9. The lesson to carry out of C2 is the one the census embodies:
coverage accumulated slice by slice is not a claim anyone can check, so slice
9 turns the census into a test.

### Release cycles, concretely

There are none, and the policy below is unchanged. What replaces them:

- **the cycle above is the delivery rhythm** — four more of them, then the
  programme is finished;
- **the terminal artifact is R2's review package, and the deliverable is this
  repository**, not an archive. No snapshot is produced: the production project
  absorbs the repository, and `## Deferred beyond this roadmap` records that no
  validation check exists for an archive. Building one to ship an artifact nobody
  asked for would be work with no reader.
- **`v0.10.0` stays frozen** through C1–C5. Completed RFCs read
  `Implemented (Unreleased)`. R2 GO is a readiness label, not a version.

If the owner later wants a distributable snapshot, it is a separate authorized
task and the missing packaging validation is its first deliverable, not an
afterthought.

**Amended 2026-11-08 — publication is now authorized, and it is still not this
programme's to perform.** The project owner asked on 2026-11-07 whether a
GitHub tag, a crates.io publication, and AUR `apimokka` / `apimokka-bin` were
near, and authorized the answer: **R2 GO → production integration → C6 →
publication**, in that order and no other. Three things follow, and the first is
the reason for the rest:

- **what the tree is, measured rather than remembered:** `crates/app` and
  `crates/model` contain **no file I/O, no subprocess control, and no network**
  — re-verified 2026-11-07, the single `std::fs` call being a test that reads
  `src/` to audit itself. Publishing it would hand a user a GUI for apimock-rs
  that cannot open their config, cannot save an edit, and cannot start their
  server, while looking finished. That is the mockup working exactly as
  designed, and it is why **the mockup is not what gets published**;
- **R2 GO does not change that.** Its own clause says a GO *"permits adoption of
  the separately drafted production-integration roadmap"* and *"does not itself
  authorize those features"*. Publication sits on the far side of them, so C6 is
  the integrated application's packaging cycle;
- **C1–C5 are unchanged and nothing was reordered to accommodate C6.** It is
  last in priority (P4) because it is last in dependency.

**AUR preparation is held** until C6's version decision, desktop assets, release
assets, and declared runtime dependency set exist, because those four are its
inputs: PKGBUILDs written before them would be written against a target that
changes. The one item worth measuring ahead of C6 is **the runtime dependency
set on a clean Arch container** — needed whatever is published and whenever, and
the only entry on the list this repository cannot answer for itself.

**If a public artifact is wanted sooner, the one honest shape is a GitHub tag
and release on this repository only**, labelled a UI/UX prototype with the
README's *"mockup only — no file I/O, no live server connection"* line promoted
above the fold. A tag claims nothing. `cargo install` and `pacman -S` both claim
*"this is the application"*, and that claim cannot be made yet. **Not
authorized; recorded as the available option.**

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

> **Overtaken 2026-08-03, recorded 2026-09-16.** The paragraphs below are kept
> as the reasoning that was current until M6's L1 unit delivered hosted CI —
> GitHub Actions, three platforms × two toolchains, six legs, run
> `30824280662` at `469e6cf`, evidence in
> `docs/src/development-and-gates.md`. **Every milestone accepted since (M8,
> M11, M9, M10) and every defect task (018, 019, 020) was accepted on "CI green
> on all six legs", so CI, not manual capture, is now the cross-platform
> record.** The three bullets below remain individually true and are why CI was
> not built earlier; they are no longer why it does not exist. Manual capture
> retains the role the second bullet describes — rendering, font metrics and
> high-contrast behaviour, which no build gate can assess and which M6 judges.

Cross-platform verification was, until M6's L1, performed **manually and
recorded as evidence** rather than through CI. MK-054's deferral of hosted CI
stood on three grounds, and cross-platform scope did not by itself overturn
them:

- the mockup contains no filesystem, process, or network code, so a successful
  build on another platform carries little information;
- what actually varies across platforms here is rendering, font metrics, window
  decoration, layout overflow, and high-contrast behaviour, none of which a
  build gate can assess. Those are validated by human inspection in M6;
- the number of remaining verification events before this repository is
  absorbed into the production project is small and bounded, so CI setup and
  maintenance would not amortize.

What changed that calculus was M6 itself: MK-056 requires a recorded
build-and-test run on every platform claimed as supported, and repeating that
by hand at each of the remaining verification events cost more than the
workflow did. The production GUI project will need CI on far stronger grounds —
path resolution, subprocess control, and a trace transport whose UDS option is
Unix-only and needs a TCP path on Windows.

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
  real `apimock-routing` matcher primitives — 5.10.0 then, **6.0.0 since M11** —
  with a published capability matrix and a matcher-oracle guard against
  unreviewed dependency drift.
- **M3 — Integration boundary (MK-053).** Replaced direct snapshot mutation
  with the `WorkspacePort` mapping boundary: typed atomic transactions, stable
  condition identity, canonical/render correlation, semantic undo/redo, runtime
  correlation, and typed Global Save reporting.
- **M4 — Quality and security gates (MK-054).** Cleared the workspace warning
  backlog without suppression, removed the iced highlighter dependency chain,
  resolved both `quick-xml` advisories through compatible updates, and added
  the canonical release-gate script and its self-test.

## Step definitions

Scope and exit criteria for the steps not summarised above. **This section is
not a statement of what remains** — the progress table below is the authority
on status, and each subsection carries its own marker. Completed definitions
are kept rather than deleted: they are what the exit evidence was judged
against.

**Of these, only M6 and R2 remain.**

### M7 — Engine contract conformance

> **Complete**, accepted 2026-08-02. Superseded in one respect: M11 later
> carried the engine from 5.10.0 to 6.0.0, so the version named throughout this
> subsection is the one M7 was judged against, not the one in the tree.

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

> **Complete** — **CONDITIONAL GO** recorded 2026-08-02; condition R1-1 closed.

**Goal:** independently verify that findings B1–B5 are resolved.

Required inputs: M1 governance evidence; M2 operator conformance matrix and
test output; M3 adapter/mapping RFC, implementation, and contract tests; M4
toolchain and security gate evidence; **M7 engine-conformance evidence**.

Decision:

- **GO:** M5 and M6 proceed toward an integration-ready baseline;
- **CONDITIONAL GO:** only listed non-production work may proceed;
- **NO-GO:** return unresolved blockers to the owning step.

### M5 — Maintainable structure

> **Complete**, accepted 2026-08-04 (MK-057, seven reviewed slices).

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

> **Implementing** — L1, the preparation gate and L2 are closed; **L3 human
> sessions remain, blocked only on participant recruitment.**

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
- correct platform-conditional user-facing copy. **Largely closed 2026-08-02 by
  dev-team task 002**, which built `accelerator.rs` as the single source of both
  the matcher and the display label and gave every advertised binding a
  `(macos, other)` pair; the command palette now renders
  `accelerator::display(...)`, and `README.md:64` names both notations. Accelerator
  notation is platform key-cap notation, not prose, and is exempt from i18n by
  that module's recorded decision — so it does *not* belong to the localization
  inventory above, as an earlier revision of this bullet claimed. **One site was
  missed and is now D-11**: `shell/view.rs:196` hard-codes `⌘Z` in the undo
  banner. It remains independently fixable ahead of M6.

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

> **Not started** — follows M6.

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
| 5 | Engine contract conformance against real `apimock-config` | M7 | MK-055 Implemented |
| 6 | Reducer, routes, fixtures, and test modularization | M5 | MK-057 Implemented |
| 7 | UX and accessibility acceptance protocol | M6 | MK-056 Proposed — the one open design |
| 8 | snora design-system adoption | M8 | MK-058 Implemented |
| 9 | apimock-rs 6.0.0 adoption | M11 | MK-060 Implemented |
| 10 | Typography and readability | M10 | MK-059 Implemented |

M9 is the one milestone with no RFC of its own, by design: it implements
MK-033 and MK-023's first-screen half as written, and dev-team task 014 says so
explicitly. MK-033 remains `Proposed` because its context-aware disabled
commands were never built; MK-023 because Tab traversal was not taken.

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
- tags, archives, version bumps, and registry publication are created only on
  explicit project-owner authorization for that task;
- commits and pushes follow the rule reconciled below, not this clause's
  earlier blanket form.

If the project owner later wants a distributable snapshot for archival or
handoff, it follows the project rule — files at archive root, version suffix in
the archive name, and no `.git`, `.git-exclude`, or `target` content — and is
requested explicitly rather than produced on a cadence.

**Amended 2026-11-08 — the push rule, reconciled, and audit finding B3 is
closed.** This policy read *"do not create commits, tags, archives, or pushes
without explicit project-owner authorization for that task"*, which was
**stricter than the rule that authorizes both models** (*"Both the
High-Capability Model and the Mid-Capability Model are authorized to commit and
push"*) and stricter than the owner's own practice. The architect propagated the
stricter text into every dev-team handoff, which is why it was recorded as
**finding B3** of `.git-exclude/reviewed/2026-10-06-roles-and-rules-compliance-audit.md`
as *"a contradiction that should not stand"*, with the correction left as a
proposal because §13 of the workflow document requires a project-specific
override to be documented and approved rather than assumed.

**The project owner settled it on 2026-11-08:** *"You may commit and push…
Besides, the dev team may commit and push only what they make and get as
approved."* So:

- **the architect may commit and push** without asking for each task;
- **the dev team may commit and push only work they authored and that has been
  reviewed and approved** — an unreviewed or rejected change does not reach the
  remote, which is what the branch-sharing risk in *Risks and controls* is
  about;
- **tags, version bumps, archives, and registry publication still require
  explicit authorization for that task.** Nothing above touches them, and C6 is
  where they live.

Dev-team handoffs issued after this date carry the reconciled rule, not the
blanket one.

**Amended 2026-11-08.** Publication — tag, crates.io, AUR — is **authorized as
cycle C6**, which runs after R2 GO and after production integration. Every
clause above continues to bind **this** programme: no version bump, tag,
registry publication, or archive is produced by C1–C5, `v0.10.0` stays the
recorded version, and nothing is committed, tagged, archived, or pushed without
explicit authorization for that task. What changed is only that the programme
now has a named, ordered destination for the work the policy previously left as
*"a separate authorized task"* — see **C6** in the cycle table for its
deliverables, and the amendment under *"Release cycles, concretely"* for why the
mockup itself is not what gets published.

## Risks and controls

| Risk | Effect | Control |
|---|---|---|
| Real `apimock-config` reveals material mapping divergence | M7 rework, possible M3 amendment | Test against the real artifact before R1, not after; record divergences rather than hiding them |
| `apimock-config` pulls a conflicting `apimock-routing` resolution | M7 blocked | Verify the resolved graph and extend the oracle guard before accepting |
| Test-only engine dependency leaks into production targets | Scope breach | Dev-dependency only; verify no production target references it |
| Adapter work expands into production I/O | M7 overrun | Keep filesystem and live server control explicitly out of scope |
| Structural split overlaps behavior changes | Regression risk | Keep M5 behavior-neutral; run the canonical gate after each split |
| UX participants unavailable | R2 blocked | Identify participants at M6 protocol design, not at session time |
| **Standing still on snora does not keep the jump the same size** (added 2026-11-09) | Each release removes items deprecated three minors earlier, and appearance has moved since 0.42.0 by snora's own account, so the cost of C7 grows with every minor we skip | **Standing practice, set 2026-11-09:** every snora notice that arrives gets a one-paragraph assessment against the surface list, appended to MK-058 on receipt, so the delta is always known and never discovered at adoption time. The pin is held through M6 deliberately rather than by inattention; C7's survey converts the remaining unknown into a number before any code moves |
| ~~**The capture rig cannot use `wgpu` until the host reboots**~~ — **cleared 2026-11-08**, the host now runs `7.3.3-1-cachyos` with its module tree present | Task 037's combination-2 re-run cannot produce comparable evidence — MK-056 decision 7 records the renderer, and its first run was `wgpu` | The running kernel is `7.3.1-1-cachyos` and its module tree was replaced by the `7.3.3-1` upgrade at 09:14 on 2026-11-08, so no unloaded kernel module can load and the nested compositor's dmabuf import fails. The GPU is healthy (`vulkaninfo`: RADV, Mesa 26.2.6). **Remedy: reboot the host — the project owner's action, not an engineering task.** The 880×700 editor surface needs no GPU and can run first |
| **A capture rig proves a file exists, not that it is this run's frame** (added 2026-11-08) | Evidence that reads as genuine but is stale, duplicated, or from another theme — three instances in three tasks (async write, assumed geometry, duplicate files) | Task 037's re-run adds a per-run manifest: every kept frame's MD5 and measured ground pixel, and the run fails on a duplicate MD5 or a ground that does not match the requested theme |
| Reviewer independence unavailable for a step | Weakened evidence | Project owner confirms directly and the deviation is recorded |
| New advisory disclosed after M4 | Gate regression | Canonical gate runs `cargo audit` at every step exit |
| MSRV dependency incompatibility | Late failure | Rust 1.91 checks run in the canonical gate, not only at exits |
| **A wrong sentence in a task, RFC or evidence file outlives the reasoning that produced it** | Work is scoped, skipped or justified on a premise nobody re-checks; it is then restated more strongly by the next reader | **Observed four times** (snora's `find_focused` constraint that does not bind us, twice; the 0.41.0 chrome-border scoping; task 020's "not observable on this host"). Correct the source document in place with a dated correction block rather than only the review — three of the four were propagated by someone reading the stale file, not the correction. Prefer citing a section over a line number: `ROADMAP.md:400` for the push rule has drifted twice and cost a correction each time. |
| **A push carries every commit beneath it, so "commit but do not push" cannot hold on a shared branch** | Reviewed-but-unapproved work reaches the remote invisibly; a dev-team status report becomes wrong through no fault of its author | **Observed 2026-10-06**: the architect's documentation pushes (`a032aba`, `d5a7754`, `246d324`) carried the whole D-13/D-14 implementation to the remote, including `2b828bf` at the moment its review said *Corrections Required*. **Control: run `git log origin/<branch>..HEAD` before every push and read what it would carry.** The instruction in every handoff assumes a property the branch does not have; reconciling `ROADMAP.md`'s per-task push rule with the rules file's standing authorization is finding B3 of the 2026-10-06 compliance audit and is owed. |
| **`lru` 0.16.4 carries an unfixable use-after-free on a runtime path** (RUSTSEC-2026-0253) | Memory corruption is reachable in principle wherever the application renders with wgpu, which is iced's default | **Accepted, not ignored.** `cargo update -p lru` moves zero packages: `cryoglyph` pins below the patched 0.18.2 and has published no release that lifts it, so no consumer of iced can clear it today. The failure needs a stored key whose `Drop` panics with unwinding enabled — a path this application does not create. **Re-check whenever `cryoglyph` or `iced` releases**; it is the only advisory in our graph that is both unsound and compiled. Distinguish it from `paste` and `anyhow`, which are lockfile-only entries never built for any supported platform. |

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

**Moving a step to `Complete` has three parts, not one.** Update the row below;
**move the step's RFC to `rfcs/done/` and set its Status field**; and update any
prose the step made untrue. Added 2026-09-16 because the first was done four
times and the second never — MK-033, MK-058, MK-059 and MK-060 all sat in
`rfcs/proposed/` after their milestones were accepted, and M11 left two
documents describing an engine version it had replaced. `check-rfcs.sh` cannot
catch either: it verifies that a folder and a Status field agree, which a
shipped-but-unmoved RFC satisfies perfectly. **This is a review obligation, not
a gate.**

**Qualified 2026-11-08:** the lifecycle now has the fourth folder the project's
own policy had always specified. `rfcs/accepted/` exists — *"review complete;
implementer may start"* — so an RFC whose design the owner has accepted no
longer sits in `proposed/` asserting the opposite, and the step above reads
*"move the RFC to `rfcs/done/`"* from whichever folder it is in. `check-rfcs.sh`
requires the folder and validates its `Accepted (authorized YYYY-MM-DD)` status
form; MK-056 is its first and currently only occupant.

**Evidence geometry, qualified 2026-11-03.** Task 034 established that the
nested compositor's own output scale was left at its winit default (`1.0`)
and the app's own column at niri's default 50 % preset, neither of which the
host-side float-and-resize the recipe issued could reach. So capture frames
from **tasks 022–033** are 1536×864 physical as stated, but were laid out
with **more logical space than the 1280×720 they claim** — and task 033's
were half that width again. Conclusions drawn from them about colour,
contrast, glyphs and keyboard reachability are unaffected; **conclusions
about layout fit at 1280×720 are not established by those frames**, which is
precisely how task 033's withdrawn "footer overflows" finding arose.
Captures from task 034 onward add `niri msg output winit scale` and
`maximize-column` against the nested socket, and record the **measured**
size. A re-measurement of any earlier appearance claim is a review
obligation if one is ever relied on for layout; none is today.

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
| M8 — snora adoption | **Complete** | **Accepted 2026-09-04.** Capture run collaboratively: the architect drove launch, `wtype` keystrokes and screenshots; the project owner supplied the pointer clicks, which are the only actions the application cannot accept from a keyboard until M9 lands. Evidence: 14 artefacts in `.git-exclude/release-evidence/2026-08-12-m8-visual-capture/`, results in its `RESULTS.md`. **Phase 3 (A1–A7): all PASS.** Dialog card intact in all four presets. **A6 — the defect M8 exists for — is proven by comparison, not judgement**: an undimmed `high_contrast_dark` reference is pure black where the dimmed backdrop is plainly lighter; the shipped defect was black-on-black. **A8 judgement confirmed by the project owner**: the card reads correctly and the dim is not heavy — the answer snora has asked for twice and had from nobody. **Phase 2 (B1–B4): all PASS.** Stock inputs, dropdown and checkbox follow the palette in all four presets; `Dark` is the case Phase 2 existed for. Scrollbars unexercised (content fits). **Pass C measured rather than eyeballed:** backdrop RGB 153 at snora 0.28.0 versus 143 at 0.42.0 over a white page — exactly 255x(1-0.40) and 255x(1-0.44), an independent pixel-level confirmation of snora's `DIM_ALPHA` change. **Pass D closed a 2x2 that had been open since 2026-08-04:** an absolute path checked immediately after `niri msg action screenshot-window` returns is absent and present 2s later, so the write is asynchronous regardless of path type. **Half-corrected 2026-10-31:** Pass D went on to call `f7356c6`'s "requires an absolute path" *a misdiagnosis*, and that was wrong. `niri msg action screenshot-window --help` states plainly: *"The path must be absolute, otherwise an error is returned."* Both facts are true and independent — niri **does** require an absolute path, **and** the write is asynchronous. Pass D found the real cause of the missing-file symptom and then over-reached by retracting a correct constraint, which is the two-true-things-conflated error this programme has flagged in others' work three times. The asynchronous finding stands. **B2-1 FAILED and does not hold this milestone**, by project-owner decision: it was added opportunistically on 2026-08-18 to check a surface outside MK-058's scope, and every case testing Phases 1-3 passed. See the defect list below. Prior state: code-complete at 0.28.0 (`265e9b6`, `8caa826`, `2c191aa`, `b0f62a2` against baseline `cef5e32`), bumped to 0.29.0 (`5aeee57`) and 0.42.0 (`93815e7`), CI green on six legs at each. **Task 015 closed 2026-09-04** with commit 2 (`5d5388e`): `snora-widgets` dropped, `default-features = false, features = ["design"]`, CI run `33837173928` green; reviewed `.git-exclude/reviewed/2026-09-04-snora-0-42-upgrade-commit-2-review.md`. **We are the first consumer anywhere running design-without-widgets**, which produced two measurements worth keeping: the stripped binary is **byte-identical** either way (17,899,656 bytes — the linker already eliminated the crate when it was merely unused, so an unused optional dependency and a binary cost are not the same thing), and **rendered output is byte-identical** to M8's capture, so that evidence still describes the shipped build. **Near-miss recorded:** pruning the now-stale `snora-widgets` entry from `Cargo.lock` needs a bare `cargo update`, which would also carry `apimock-config`/`apimock-routing` 5.10.0 → 5.19.1 — invalidating MK-052 and MK-055 and pre-empting MK-060's deliberate 5.10.0 → 6.x decision. Run, inspected, reverted; the entry is correctly left in place. |
| M11 — apimock-rs 6.0.0 adoption | **Complete** | **Accepted 2026-09-05.** `6c12a07`; reviewed `.git-exclude/reviewed/2026-09-05-mk060-apimock-6-adoption-review.md`; CI run `33929641568` green on all six legs. **Production source changed by zero lines.** The diff is Cargo files, the test-only conformance suite, and the four oracle scripts whose `5.10.0` pins the task predicted would trip. 290 tests pass on both toolchains (203/56/27/4). **All nine MK-055 divergences re-classified against 6.0.0 and all nine persist**, none closed. **A tenth was found by execution, not by reading:** `TlsCertFile`/`TlsKeyFile` moved from `restart()` to `reload()` in apimock-config 6.0.0 (their RFC 020 — cert rotation uses the reloadable resolver), which our `map_root_setting` still classifies as `Restart`. Verified at source in both vendored crates. Caught by a pre-existing Tier 2 test failing — **the first time MK-055's executable oracle has detected an engine change on its own**, and the justification for having built it as an oracle rather than a document. **Accepted as a stricter divergence, deliberately not corrected here:** we say "restart" where "reload" would now do — never the reverse — and fixing it would land an unvalidated, user-visible behaviour change between M8's capture and M6's sessions. See the production-integration inputs below; it is owed, not settled. **The `library/` delta confirmed every MK-053 decision** — no amendment raised — while surfacing more new engine surface than MK-060's symbol-presence sizing could bound, exactly as that sizing warned. |
| M9 — Keyboard operability | **Complete** | **Accepted 2026-09-05.** `74e7f75`; reviewed `.git-exclude/reviewed/2026-09-05-task014-mk033-keyboard-palette-review.md`; CI run `33943676852` green on all six legs. MK-033's three keyboard requirements built — search field auto-focused on open, `Enter` executes the selected row, arrow-key navigation — with the command table extracted to `palette_commands.rs` following `accelerator.rs`'s precedent so `view` and `update` cannot disagree about which row is selected. The mode picker gained the same idiom, **closing MK-023's first-screen gap: a keyboard-only user can now pass the application's first screen** — and is stopped at the second. *Qualified 2026-09-17:* this row previously implied the S1 condition blocking M6's keyboard-only session was cleared. It was not: Welcome has no keyboard path, which the dev team declared in the run's own evidence and which is now **D-12**. The run's keyboard chain was steps 1–4 and 8–11, bridged by two owner pointer clicks. D-4 fixed (option 2 — the palette toggle is inert outside `AppView::Workspace`, turning a silent no-op into an honest one). 306 tests pass on both toolchains; `update` remains a pure reducer, its 486 existing call sites untouched, with the one genuine `Task` isolated in a wrapper `main.rs` alone calls. **The live `wtype` run found a defect this task's own requirement created**, and no unit test could reach: `Escape` needed two presses to close the palette. Root-caused at source — `iced_widget`'s `text_input` captures `Escape` unconditionally when focused (`text_input.rs:1235`), and `iced::keyboard::listen()` only ever sees events with `Status::Ignored`, so auto-focus made the first press invisible to the global handler. Fixed at the root by moving the subscription to `iced::event::listen_with`, which reports capture status rather than dropping captured events, and re-verified live. **Third time the live-run instruction has paid** — after task 010's three execution-only script defects and M8's four. |
| M10 — Typography and readability | **Complete** | **Accepted 2026-09-05.** `4deadc2`; reviewed `.git-exclude/reviewed/2026-09-05-task017-m10-typography-and-defects-review.md`; CI run `33963484000` green on all six legs. MK-059 implemented per its own resolutions: **102 of 152 `CAPTION` sites triaged** — 32 to the new `BODY_SMALL` with line-height, 71 to the new `LABEL` without — and **50 left at 12.0 deliberately**, which is decision 2 working as intended: it leaves M6 something real to judge rather than closing the question by assumption. Line-height applied only where it is a gain: `body` (1.4, looser than iced's silent 1.3 default) and `body_small` (1.35). `title` is a no-op at 1.3 and `heading`/`display` are tighter, so they are applied nowhere; the accessors exist, unused, with the reason recorded. `DISPLAY` stays 36.0 per resolution 1. **All four appearance defects fixed and verified live in four presets.** D-1/D-2 were one fix: `theme::dialog_style` now takes `width: 1.0` and `tokens_for(t).palette.border`, so we inherit snora's 0.34.0 contrast repair instead of overpainting it — confirmed on D-2's exact B2-1 failure case, the wizard in `high_contrast_dark`, which M8's capture recorded as having no discernible edge at all. D-3 fixed with `.style(theme::naked)` and `.width(Length::Fill)`. D-5's auto-scroll implemented in `update_and_dispatch` and verified **against the real scrollbar**, not only against index math — iced 0.14 exposes no scroll-into-view for a specific child, so it maps filtered index to relative offset and says so. `update` remains a pure reducer. `theme.rs` crossed the 500-line threshold and carries a recorded boundary decision. **A fifth defect surfaced live and is recorded as D-7.** |
| M6 — UX acceptance evidence | Implementing | MK-056 design accepted and implementation authorized by the project owner 2026-08-02, including resolution of all five review questions. Delivered as three units. **L1 cross-platform CI complete 2026-08-03**: all six legs green (Linux/macOS/Windows x stable/1.91) on run `30824280662` at commit `469e6cf`; evidence in `docs/src/development-and-gates.md`. **Preparation gate closed 2026-08-04** (`d46b1e9`, task 006) — no participant's time is now spent rediscovering a defect we already knew. **L2 landed 2026-08-04** (`22b6739`, task 008: `scripts/ux/`, inert and deliberately outside `check-release-gates.sh`), rescoped after the capability probe (`887ab1b`) and since exercised three times against the live application — the mode-picker size matrix (2026-08-04), M8's capture (2026-09-04) and M9's `wtype` run (2026-09-05), all under `.git-exclude/release-evidence/`. **Only L3 human sessions remained, and they were blocked on nothing technical** — true when written; *superseded 2026-11-04*, see the end of this row. Every milestone M6 waited on is Complete: M8, M11, M9, M10. **No defect is open above S3 (latent), and none is queued for the dev team.** **Participant recruitment is the project owner's and is the one remaining prerequisite that cannot be produced inside this repository** — the other, task 037, can be. *Qualified 2026-10-06:* this row previously called it the item "nobody can compress", which framed it as a duration. It is not — it is an unblocked prerequisite with no substitute. Two defects found after this row was written, D-13 and D-14, now also precede the sessions. **2026-11-04:** the readability-probe amendment (record the role, not only the tier) accepted; the acceptance criterion naming L2 **resolved as written** now that D-12 is closed — every wall the 2026-09-17 defect note listed was removed by MK-023; the session pack's §0 restored; **task 037 issued** for the four mandatory L2 combinations. **2026-11-07:** task 037 ran them for the first time (`1634aed`, `scripts/ux/` only — no `crates/` change): three pass, **combination 2 fails as D-29 (S1)** and **combination 4 is incomplete** against decision 7's own density wording; **task 038** issued for D-29. `scripts/ux/` is now the durable home of the capture recipe, with the capture-landed check in code and the host output mode restored and verified. **2026-11-07, authorized by the project owner:** **task 038** (the D-29 fix) and the push of the four local docs/test commits. Recruitment (task 002) is unaffected by D-29 and remains the only prerequisite this repository cannot produce; booking a session waits on D-29's fix and on task 037's two re-runs, recruiting does not. **2026-11-08:** task 038 accepted (`3ac0e20`), **D-29 closed**; the review measured the fix independently, verified the new guard fails on the pre-fix code, and isolated the ring in the evidence frames by decoding and diffing them. D-30 (S2) opened from that review and task 039 issued; neither blocks this row. **What M6 now waits on: recruitment (owner) and task 037's combination-2 re-run plus the 880×700 editor surface.** **2026-11-08:** task 039 accepted, D-30 closed; **no defect above S3 is open that touches M6's surfaces.** The review found the capture rig's `wgpu` break is a host kernel upgrade (module tree of the running kernel replaced at 09:14), not a rig defect — a reboot clears it, and only combination 2's re-run depends on it. **2026-11-08, round 2:** §9.2 and §9.3 accepted; **D-31 (S1) and D-32 (S2) opened** from the 880×700 editor captures and task 040 issued. M6's path is now three items: task 040 (blocks booking), task 037 §9.1 (blocked on a host reboot), and recruitment (owner, unblocked). **2026-11-08, task 040 accepted:** D-31 and D-32 closed; D-33 (S1) and D-34 (S3) opened from its row walk; task 041 issued. M6's path is unchanged in shape — one S1 fix (task 041), task 037 §9.1 (now unblocked by the reboot), and recruitment (owner). **2026-11-08, task 041 accepted:** D-33 and D-34 closed, D-35 (S3, latent) recorded. The defect queue is clear of anything that blocks a session; what remains is MK-056's own criterion — *"L2 recorded pass for every mandatory combination"* — which combination 2 still lacks, and recruitment. **2026-11-08, §9.1 accepted — L2 is complete.** Four mandatory combinations, four recorded passes; task 037 closes after four rounds that produced six harness repairs and six defects (D-29, D-30, D-31, D-32, D-33, D-36 — four of them S1), none of which had ever been seen because none of the four combinations had ever been run. **Nothing in this repository now blocks a session; the one open prerequisite is recruitment.** |
| task 021 — appearance remediation | **Complete** | **Accepted 2026-10-31** (cycle C1). D-13 and D-14 closed; AA asserted in all four presets; one severity-glyph source in `crates/model`; captured by task 022 with no owner clicks. |
| task 036 — banner contrast and labels | **Complete** | **Accepted 2026-11-04** in two rounds (`33505c7`, `1248793`). **D-21, D-25, D-27 closed**; the banner's every colour measured 5.43 / 10.63 / 8.42 / 14.85 in all four presets, with a census test over `banner_style`'s own call sites so a sixth user cannot appear unguarded. Round 1's own test passed while the surface still failed — the lesson recorded in D-21's closure. Reviews: `.git-exclude/reviewed/2026-11-04-task036-d21-d25-d27-review.md`, `…-round2-review.md`. |
| M12 — Keyboard path through the primary scenarios | **Complete** | **Accepted 2026-11-03** (cycle C2). MK-023's external design accepted 2026-10-31 and implemented in **ten reviewed slices** (`dbf947c`, `926a4fc`, `cc56256`, `4ba6552`, `556102b`, `1258ce9`, `0af9e09`, `7df87ad`, `83b5ffc`, `9d405e1`); **MK-023 moved to `done/`** as `Implemented (Unreleased)`. **D-6, D-12, D-15, D-16, D-17, D-18, D-19, D-20, D-22, D-23, D-26 closed**, and D-24's confirmation half. All six primary scenarios captured keyboard-only with zero pointer events, every control in the audit inventory reachable or recorded with the reason it is not, and the census held by a test. Four defects were found by capturing rather than reasoning (D-18, D-20, the slice-9 `Enter` precedence bug, D-25's second instance) and one claimed defect was withdrawn when the geometry was measured. Reviews: `.git-exclude/reviewed/2026-10-31-task025-and-026-review.md` through `2026-11-03-task035-slice10-review.md`. |
| R2 — Integration readiness | Not started | — |

## Defect register

Defects found by executing surfaces nobody had run — M8's capture (2026-09-04),
M9's live `wtype` run and M10's visual pass (2026-09-05). **None is a regression
from the milestone that found it**; every one is a pre-existing surface that had
never been exercised. Recorded here rather than folded silently into another
milestone.

D-9, D-10 and D-11 arrived differently — by reading source while working on a
neighbouring change, not by executing anything. Both routes are kept in one
register: what matters to M6 is that a participant would meet the defect, not
how we came to know about it.

### Open

| # | Defect | Severity | Disposition |
|---|---|---|---|
| **D-36** | **Text-field focus is carried by colour alone, and the colour clears the state floor in one preset of four.** Text inputs are the one widget iced 0.14 gives native focus to, so they carry no region cursor and their indication is the field's own border. `theme::text_input_style`'s `Focused` arm sets **only the placeholder colour**, leaving iced's default focused border, while the `Active` arm overrides the resting border to `border_token` — the two were never compared with each other. Focused-versus-resting: **light 3.74**, **dark 2.63**, **hc_light 1.40**, **hc_dark 1.19**, with **no width change in any preset**. So the cue is a hue shift and nothing else, and it clears WCAG 1.4.11's 3:1 only in Light. **Found 2026-11-08 by task 037 §9.1** — combination 2, whose stated purpose is *"focus indication against high-contrast borders"*; the dev team measured the two HC presets and the architect's own probe extended it to all four. **The third instance of one pattern**: a role measured against the page (task 023's D-15 guard asserts the focus ring ≥3:1 against the page, and still holds) rather than against the thing the eye compares it with — as D-21 and D-30 were. | **S2** — a degrading accessibility failure, not one that prevents completion: the caret and the typed text still answer *"where am I"*, which is MK-056 §5's own test for S1 and the reading already applied to D-30. **Does not block M6 booking**, and did not hold combination 2's recorded pass, since §5 places S2 beside completion rather than against it. | **Task 042**, issued 2026-11-08: `rfcs/handoffs/MK-022-visual-design-system/042-d36-text-field-focus.md`. **Give focus a width cue and keep the colour** — `border.width = 2.0` against the resting 1.0 in every preset, the move that closed D-30, so the state is carried by width (MK-028's non-colour rule) and no new colour needs measuring against four grounds. **Architect's judgement: it should land before the first session**, not after — one match arm, and the preparation gate exists so that no participant's time is spent rediscovering a defect we already knew. The task also deletes `ux_app_show_region_cursor`'s dead `soft` branch, whose comment still asserts D-29 in the present tense. |
| **D-35** | **The top bar has the least slack in the application, and a workspace name can push its buttons off the edge.** Measured by task 041's bounded audit at 880×700: ~120–130 logical px spare in both locales, and the third button (Reload/Restart, shown when a restart is pending) costs ~75–85 of it, leaving ~40. Workspace names are user text drawn without truncation, so a long one consumes that margin and pushes *Save all* / *Stop server* off the right edge. **Latent — found by reading and measuring, not by execution**, which the register has carried before (D-9, D-10 and D-11 arrived the same way). | **S3** — unobserved, no participant will meet it with the mock's own short names, and nothing is lost when it happens. | After M6, with D-24 and D-28. The measurement is recorded here so it is not re-derived; whoever fixes it should capture a long name first rather than trusting the arithmetic. |
| **D-24** | **The save-diff drawer's `Discard` discards less than the drawer lists** (and was unconfirmed until slice 9). `widgets::ghost_btn(…, Message::DiscardChanges)` runs `discard_all_changes()` immediately — while **`ConfirmAction::DiscardChanges` exists, is handled in both the `ConfirmRequest` and `ConfirmProceed` arms, owns two i18n keys, and is dispatched by nothing**: a confirmation was designed for this exact action and is dead. Separately, `save_diff_total`/`save_diff_content` count and list dirty **rule sets** and dirty **fallback files**, but `discard_all_changes` iterates `fallback_drafts` only, so a dirty rule set listed in the drawer survives the button beneath it. Slice 8's own capture shows the one dirty item as `main.toml`, a rule set. Found 2026-11-03 reviewing slice 8, while checking the activation message it gave a keyboard route to. Pre-existing. | **S2** — the user completes, with a wrong mental model; the edits lost are not recoverable (`discard_all_changes` records no history entry). | **Confirmation half closed** by slice 9 (`83b5ffc`) — both the drawer's button and `DrawerField::Discard`'s own `activation_message` now dispatch `ConfirmRequest(ConfirmAction::DiscardChanges)`, so the confirmation that already existed is finally used. **The scope half stays open and is the owner's**: the dialog now says *"Discard unsaved changes? Edits will be lost."* for an action that still reverts fallback drafts only, which makes the mismatch more visible rather than less. Originally scheduled as: **confirmation half in MK-023 slice 9** — route it through `ConfirmRequest(ConfirmAction::DiscardChanges)`; the strings are already written. **The scope half is a product decision for the owner**: either discard what the drawer lists, or relabel the control to what it does. |
| **D-28** | **Eleven history variants announce the wrong action when undone.** `HistoryEntry::banner_key` maps ten variants that are not a move to `UndoRuleMoved` — "Moved rule" — (`RulePrototype`, `TracePrototype`, `HeaderAdd`, `HeaderUpdate`, `HeaderRemove`, `HeadersClear`, `BodyAdd`, `BodyUpdate`, `BodyRemove`, `BodiesClear`), and maps `Respond` to `UndoUrlPathEdited` — "URL path changed" — so undoing a response status, delay or body edit announces a path change. **Found by the dev team's own mapping audit** in task 036 (which task 036 asked for and which correctly reported rather than fixed: naming eleven labels is judgement work, not the mechanical shape that task authorised). Counted from source in review: the audit said eight in one document and nine in another, both lists held ten, and the eleventh hid under the other key. | **S3** — the same class as D-25, at eleven times the scale: a wrong label on the one affordance that offers to undo. | **Not scheduled.** A task of its own, after M6: eleven labels is a content exercise (MK-036's territory), not a mechanical mapping fix, and M6 may say which of them a user ever actually sees. |

### Closed

| # | Defect | Closed by |
|---|---|---|
| D-33 | **The editor's condition rows collapse at 880×700 — and a header value the rule matches on is not drawn at all.** `headers_card` (`rule_editor.rs:589`) builds `row![name Fixed(110), op Fixed(110), value Fill, ✕]` inside a card ~273 logical px wide; the fixed children and spacings take essentially all of it and **the value field resolves to zero width**. The walk frame shows `content-type` · `Contains` · ✕ with nothing between, while the mock's value is **`"application/json"`** (`workspace_fixtures.rs:89`) — so the editor hides a condition the rule actually matches on, which reads as data loss even though nothing is lost. `body_card` (`:672`) has two `Fill` children and both collapse to ~28–31 px (`ac`, `ite`, `cr`, `0.(`). **Found 2026-11-08 by task 040's row walk** — the dev team reported it and asked for the design call rather than choosing silently, as the task required; confirmed by the architect at source and in the frame. | **Task 041**, `6958496`, accepted 2026-11-08 — option (a) built as decided: each condition is two explicit rows in a column, `[name][op]` over `[value Fill][✕]` and `[path Fill][…][op]` over `[value Fill][✕]`. The value keeps `Fill` (≈230 px at 880, still growing at 1280), the shape is identical at every width, and **the control census passes unmodified** — the signal MK-023 slice 9 built it to give, that cursor order and the control surface did not move. Verified in the frames: `application/json` legible at 880 wide in English Expert and Japanese Expert, both body values with it. **Reachability at the mandated 700 px height was checked in the application, not the rig**: `App::scroll_snap_target`'s `Region::Editor` arm snaps the WHEN/RESPOND scrollable to the cursor, so the fold is a density observation, not a reachability one. Evidence: nine runs, all **wgpu** (the first since the host reboot), 55 manifest rows re-verified by the architect. Review: `.git-exclude/reviewed/2026-11-08-task041-d33-review.md`. |
| D-34 | **Japanese 「インラインテキスト」 wraps inside its own RESPOND tab at 880×700** ("…キス / ト"): the label plus `mode_tab`'s `S4` horizontal padding needs ~160 logical px against ~155 available — about six short. Measured and reported by task 040, confirmed in `combo1-04-editor.png`. | **Task 041**, `6958496` — `mode_tab`'s horizontal padding `S4 → S3` (16 → 12 logical): 8 px off a tab that measured ~6 px over. Two call sites, both the RESPOND mode group; Japanese now sits on one line at 880×700, English is unchanged, and the D-32 wrap is undisturbed at both widths. **No string was shortened** — microcopy stayed MK-036's. |
| D-31 | **The rule editor's Method group clips PATCH and DELETE at 880×700 — and says nothing about it.** `method_card` (`crates/app/src/screens/routes/rule_editor.rs:540`) builds `row(btns)` over `regions::editor_method_group`'s six entries with no wrap and no horizontal scroll; at this window the card is 308 logical px and renders **Any / GET / POST / PUT** only. At 1280 all six appear. **A pointer user cannot select PATCH or DELETE**, and no ellipsis or scrollbar says the list is truncated, so the four visible choices read as the whole set. **A keyboard user can still set them, blind**: each button is `cursor_wrap_group`-wrapped, so cursor indices 4 and 5 exist and `Enter` dispatches `RuleSetMethod("PATCH")` onto a control that is not drawn. **Found 2026-11-08 by task 037 round 2**, in the combinations decision 7 added for exactly this (*"Expert shows every control at once, and this is where that density fails"*), confirmed by the architect in both runs' frames and traced to source. | **Task 040**, `c046485`, accepted 2026-11-08 — the group is now `row(...).wrap().vertical_spacing(S1)`, so every choice stays drawn, reachable and in cursor order at any width with no breakpoint; `iced_widget 0.14.2`'s `Wrapping` was read at source before being relied on. Verified in the frames: **Any/GET/POST/PUT then PATCH/DELETE** in English, **指定なし/GET/POST then PUT/PATCH/DELETE** in Japanese, at 880×700. **The measured clipping width of the old layout is the number to keep: ~1120 logical px** — 880 showed four choices, 1000 lost DELETE, 1100 drew DELETE with its edge cut, 1130 and 1160 were complete. This was never a small-window curiosity; an ordinary 1024- or 1100-px window silently lost a method. Guarded structurally (`method_group_builds_all_six_choices`) and evidentially (two manifested runs, 21 of 21 rows re-verified by the architect). Review: `.git-exclude/reviewed/2026-11-08-task040-d31-d32-review.md`. |
| D-32 | **The RESPOND mode group collapses at 880×700, worst in Japanese.** `respond_card` (`rule_editor.rs:766`) builds `row![inline, serve_file]` of two `mode_tab` buttons — the same unwrapped-row mechanism as D-31. In Japanese the second collapses to a **~29 logical-px column** with 「ファイル提供」 stacked vertically at the panel edge, beside 「インラインテキスト」 which itself wraps; English Expert wraps it to two lines. Confirmed by the architect in `combo1-04-editor.png`. | **Task 040**, `c046485`, accepted 2026-11-08 — same mechanism, same fix: the mode group wraps, so 「ファイル提供」 sits on its own line at its natural width. The Status/Delay row was wrapped in the same visit (Delay had been squeezed to ~36 px with its `ms` clipped — overflow, so in scope). Guarded by `respond_mode_group_builds_both_modes` for both active states. |
| D-30 | **In high contrast, *selection* is the state that is barely drawn — and the selected row is the one whose border is missing.** `card_selected_style`'s high-contrast branch carries selection by a 10–18% primary wash with its border at `..Default::default()`, while unselected `card_style` carries a **1.5px token border at ~21:1**. Measured: selected-versus-unselected fill **1.60:1 (HC Light)** and **1.35:1 (HC Dark)**, border width **1.5 → 0.0**. `card_parent_selected_style` is the same shape (**1.47:1** / **1.28:1**, border 1.5 → 0.0). Light and Dark carry the same information properly — the border doubles in width *and* takes primary at 6.19–6.70:1, MK-028's non-colour rule satisfied. **Confirmed in the rendered pixels** of task 038's own frames: in `hcdark-4`, the selected row is a flat (30,37,48) against unselected (3,3,3), and the next row down opens with a (193,193,193)→(255,255,255) edge the selected row does not have; no left accent strip exists on the row in either preset. **Found 2026-11-08 by the task 038 review's own measurement** — the dev team reported the qualitative half (*"in HC Dark the selection tint is subtle against black"*) and the review measured it. | **Task 039**, `7f26ba1`, accepted 2026-11-08 — both selected styles take a shared `hc_selected_border`: **3 px in the token colour** where unselected `card_style` carries 1.5 px, so the cue is **width** (MK-028's non-colour rule) and the colour channel stays the cursor's. Measured independently on both grounds in all four presets: the token border reads **13.13 / 14.29 (HC Light)** and **15.24 / 16.11 (HC Dark)** against the selected fills and 20.59–21.00 against the unselected ones, with muted text *gaining* margin (7.11–8.85). The guard is a rule rather than a number — fill ≥3:1 **or** a border ≥1 px wider whose colour clears 3:1 — and the review verified it fails on the pre-fix styles with eight high-contrast rows, Light and Dark passing. **Verified in the pixels:** in `hcdark-3`/`hcdark-4` the selected row shows 3 px white edges against unselected rows' 1 px, with the 2 px primary cursor ring outside — three states, one frame. Review: `.git-exclude/reviewed/2026-11-08-task039-d30-selection-review.md`, whose §4 records an evidence-directory correction (twelve undocumented files, nine duplicates, four naming a theme their pixels contradict) carried into task 037 as a manifest check, and whose §6 diagnoses the capture rig's GPU break as a host kernel upgrade, not a rig defect. |
| D-29 | **The keyboard cursor paints nothing in either high-contrast preset.** `theme::cursor_style` — the cursor's appearance for every region that is not a text field — borrows **only the border** from `card_selected_style`, whose high-contrast branch deliberately carries *selection* by a 10–18% alpha tint and a shadow and leaves its border at `..Default::default()`. `cursor_style` discards the tint and the shadow and keeps the one part that carries nothing. Measured per preset: Light and Dark **2.0px, opaque**; **HC Light and HC Dark 0.0px, alpha 0.00, no background, no shadow** — nothing is painted at all. **Application-wide**: the top bar, tab bar, sidebar, rule editor, rule-set config, dashboard, welcome, settings, Test Rule, the path assistant, the drawer and the notice banners are all `cursor_style` callers. The only indicators that survive are text fields' native focus rings and the error banner's three controls, which escape because `banner_cursor_style` (D-21, task 036 round 2) sets its own explicit border. **Found 2026-11-07 by task 037's combination 2** — MK-056 decision 7's mandatory *"high-contrast theme with keyboard-only input, focus against high-contrast borders"*, the combination that exists for exactly this and had never been run. Confirmed three ways: read at source, measured in a probe, and pixel-identical frames after repeated `Tab` in the delivered evidence. **Why ten slice reviews missed it**, architect included: MK-023's design requires *"every region's indicator is rendered when the cursor is in it"*, the test asserts that **in the model**, and every acceptance capture in cycle C2 was taken in Light or Dark. The same shape as D-21 — our tests assert our roles on our surfaces, and none of them looked here. | **Task 038**, `3ac0e20`, accepted 2026-11-08 — `cursor_style` owns its border (2px, `primary.base`, radius `LG`) instead of borrowing `card_selected_style`'s, and one test walks both cursor treatments in all four presets asserting non-zero width, full opacity and ≥3:1 against each style's real grounds. Measured twice independently — **10.25/10.25/10.25 (HC Light)** and **11.75/11.75/11.08 (HC Dark)** against page, `surface` and `surface_raised` — and the review verified the guard fails on the pre-fix code with exactly the ten high-contrast rows at 1.00:1. Non-HC is unchanged to the value: the new literal is the same `width: 2.0` and `primary.base` the old branch returned, so cycle C2's Light and Dark captures still describe the shipped build. **The ring is verified in the pixels, not only in the test**: decoding the evidence frames and diffing consecutive ones isolates it as the only thing that moves — 708 pixels of (156,196,255) in `hc_dark`, (0,48,176) in `hc_light`, each the preset's own `primary.base` to the byte, on the top bar, the sidebar and inside the rule editor. Review: `.git-exclude/reviewed/2026-11-08-task038-d29-cursor-ring-review.md`. |
| D-1 | `theme::dialog_style` overpainted snora's dialog card — fill, radius and an 18%-black shadow with **zero border width**, so snora's 0.34.0 `palette.border` repair never rendered for us. Measured: byte-identical card edges at snora 0.28.0 and 0.42.0 across that repair. | **M10**, `4deadc2` — `width: 1.0` and `tokens_for(t).palette.border`, so the token's contrast work is inherited rather than overpainted. |
| D-2 | The new-workspace wizard's card had **no discernible edge** in `high_contrast_dark` (capture case B2-1, the one required case that failed). Same root cause as D-1: a full screen with no dim behind it, so the fill could not separate it and the 18% shadow composited to nothing on black. | **M10**, `4deadc2` — same fix as D-1. Verified on the exact failure case. |
| D-3 | The wizard's three collapsible section headers rendered as **primary-filled buttons** with near-unreadable hint text and a fill that did not wrap its content — `wizard.rs` built `section_head` with no `.style(...)` and no `.width(...)`. **Found by the project owner during M8's capture, with no case covering it.** | **M10**, `4deadc2` — `.style(theme::naked)` and `.width(Length::Fill)`. `theme::muted()` left untouched: the hint colour was never wrong, only what sat underneath it. |
| D-4 | The command palette could not be opened outside a workspace: `Ctrl+K` set `open = true` while `command_palette::view` renders only inside `AppView::Workspace`, so the state flipped and nothing drew it. | **M9**, `74e7f75` — option 2: the toggle is inert outside its one rendering context, turning a silent no-op into an honest one. |
| D-7 | Two overlays could stack: executing the palette's *Switch workspace* row opened the workspace menu without closing the palette. `AddRuleFromPalette` was the only row that self-closed. | **task 018**, `5391789` — the same flag set in the four palette-reachable arms that open another overlay (`ToggleWorkspaceMenu`, `TestRuleOpen`, `OpenValidationDrawer`, `OpenSaveDiffDrawer`). No general modal-ordering mechanism built; that stays recorded for the production project. |
| D-8 | The palette listed two rows dispatching `SwitchTab(Settings)` — "Open settings" and "Go to Settings" — with identical behaviour in both locales. | **task 018**, `5391789` — `PaletteCmdSettings` removed and its i18n key deleted as genuinely orphaned, keeping "Go to Settings" so the three tab-navigation commands read as one set. |
| D-5 | Arrow-key navigation did not scroll the selection into view; MK-033 line 120's "correctly with the visible scroll" was met for index correctness only. **Declared by the implementer rather than found later.** | **M10**, `4deadc2` — `snap_to` from `update_and_dispatch`, mapping filtered index to relative offset since iced 0.14 exposes no scroll-into-view for a specific child. Verified against the real scrollbar, not only the index math. |
| D-9 | The command palette's server row was statically labelled `Key::PaletteCmdStartServer` while dispatching `Message::StartStopServer`, a toggle — so with the server running the row read "Start server" and executing it stopped the server. | **task 019**, `083666b` — `Command.label` is now `fn(&App) -> Key`, mirroring the `fn() -> Message` already beside it, and the server row matches on `app.server_state` exactly as `shell/top_bar.rs:130` does. The conditional stays inside the row rather than in every reader of the table, and filtering reads the live label, so the palette cannot show a row whose text contradicts what executing it does. |
| D-11 | The undo banner advertised a macOS key on every platform: `shell/view.rs:196` hard-coded `text(" ⌘Z")` beside the Undo button, so a Linux or Windows participant was shown a key their keyboard does not have, immediately after their first reversible action. The one call site task 002 missed when it made `accelerator.rs` the single source of accelerator notation. | **task 020**, `d2e0209` — `accelerator::display(Accelerator::Undo)`, the same function and binding the command palette already reads; three call sites now, one source. **No test added, and that is the finding worth keeping**: `accelerator/tests.rs:161` already asserted `notation(Undo, Other) == "Ctrl+Z"` and **passed while the banner was wrong**, because the banner never consulted the table. Asserting a table is correct says nothing about which call sites read it, and no reducer test can reach a view literal — declared plainly rather than papered over with a test that would not have failed. |
| D-10 | No palette route to locale switching; recorded as S3 on the premise that locale was "reachable from Settings". | **Superseded by D-12**, 2026-09-17 — the premise was wrong for keyboard users: Settings' locale control is a `pick_list`, which iced 0.14 cannot focus, so locale was reachable by pointer only. Not a fix; a reclassification. |
| D-13 | Muted text was below WCAG AA on every surface painting an iced-derived slot rather than a snora token — `panel_style` and `chip_style` (3.12–4.22:1 in Light and Dark) and, after the task's own audit, `card_style`, `card_selected_style` and `card_parent_selected_style`. snora's `text_muted` guarantee is scoped to *their* three surfaces; we painted iced's `background.weak`/`strong`, so the guarantee did not come with us. | **task 021** (`2b828bf`, `c27a586`, `a04616c`, `2465443`) — one rule for every muted surface in Light and Dark: snora `surface` fill plus a token border, the selected parent on `surface_raised` with the card's shadow so selection reads as raised, high contrast byte-identical. **Asserted in `crates/app/src/theme/tests.rs`** across all four presets, so the pairing cannot regress silently. Captured by **task 022** — nine frames, both presets, XWayland, `wgpu`; the tenth is structurally unreachable; **the first capture in this programme needing no owner clicks**. **Architect's judgement: the repair reads correctly and nothing reads worse** — panels still read as panels at a 1.08:1 fill step because the border carries the edge. Reviewed `.git-exclude/reviewed/2026-10-31-task022-capture-complete-review.md`. |
| D-17 | The locale picker was labelled with `Key::NavSettings` — "Settings" in English, "設定" in Japanese — so the control changing interface language carried the navigation label, **and in Japanese it duplicated the page title directly above it**. | **task 025**, `0ebed9f` — new `Key::SettingsLocale` ("Language" / "言語") at the one call site; `Key::NavSettings` untouched at its navigation sites. No test added, correctly: `en.rs` and `ja.rs` carry **zero** catch-all arms, so a missing value is a compile error and a test would restate a constant. Captured in both locales. CI run `41193214523` green on six legs. |
| D-6 | `iced::event::listen_with` was handed a capture status the subscription discarded, so the application saw key events a focused widget had already consumed. Inert while `Escape` was the only overlap, and due to wake the moment any `text_input` gained `.on_submit` or a global shortcut landed on a captured key. | **task 026** (M12 slice 1), `dbf947c` — the closure now reads the status and drops captured events, with `Escape` deliberately excepted **before** the check so M9's single-press fix survives. The hazard is closed at the root rather than avoided: `.on_submit` is forbidden by the accepted design, and a test walks **every** source file under `crates/app/src` to enforce it — the dev team replaced the architect's suggested list-equality assertion with a tree walk, which is strictly better since the hazard is a file nobody has written yet. Verified nothing was silenced: `accelerator.rs:136` destructures `Key::Character`, so no accelerator is ever a `Named` key. |
| D-15 | The stock `text_input` style failed WCAG in four of its five colour roles, in every preset — placeholder 1.00–1.09, disabled value 1.02–1.35, disabled placeholder 1.14–1.46, border 1.54–1.58 against a 3:1 floor; only the focus ring passed. We styled no `text_input`, so iced's default applied and derived every colour from the emitted palette. Two Settings fields are disabled inputs that were rendering real values invisibly. | **task 023**, `1e5bd89` — `theme::text_input_style` starts from `iced::widget::text_input::default` and overrides only the failing roles, so everything M8's Pass B validated keeps its derivation; wired at all 28 call sites across 9 screens. Placeholder takes `text_muted` (4.93 / 5.44 / 11.37 / 11.54); **disabled value, placeholder and border all take `text_secondary`** (6.53 / 7.10 / 12.91 / 16.50) after the dev team found, unprompted, that D-13's `border_token` clears 3:1 on `base` but only 2.51 / 2.95 on Disabled's `weak` fill — **the same defect class one token over, and the third appearance of it after D-1 and D-13**. Five tests assert every role in four presets, including a pin on the untouched focus ring and an equality check against iced's default. Captured in four presets with no owner clicks. CI run `41181638485` green on six legs. Reviewed `.git-exclude/reviewed/2026-10-31-task023-d15-review.md`. |
| D-14 | Two disagreeing sources for the severity glyph: `apimokka_model::Severity::glyph()` returned `!`/`△`/`i` and was documented as the non-colour cue in use, while the validation drawer rendered `widgets::severity_glyph()`'s `✕`/`⚠`/`ℹ`. A specification defect in the crate the production project builds from. | **task 021**, `2b828bf` — one source in `crates/model`, returning the previously rendered values; the duplicate deleted, both call sites repointed, values asserted. Behaviour-neutral by construction. *Row moved to Closed 2026-10-31: its disposition had said closed since 2026-10-06 while the row sat in Open — the architect edited the cell and not the table.* |
| D-12 | A keyboard-only user was stopped at the second screen, and every one of MK-056's six primary scenarios was uncompletable on the keyboard path — at its first step. Narrowed 2026-11-03 to scenario 5 alone once slices 1–7 had carried the other five. | **MK-023 slices 1–8**, closed by slice 8 (`7df87ad`) 2026-11-03. The region/cursor model — `Tab` between regions, arrows within, `Enter` activating — built across nine surfaces, and **all six scenarios captured keyboard-only with zero pointer events**. Scenario 5 is the last: a fallback file selected from the sidebar, edited, saved, and reverted through the confirm dialog, where a bare `Enter` cancels and only a deliberate `ArrowDown` then `Enter` proceeds. **Closed on its own terms, not by redefinition** — the controls still unreached are off the scenario paths and are D-23, a distinct defect at its own severity. One correction against the evidence: the capture ran at roughly **half** the required window width (the app occupied 50.3 % of the nested output against 100 % in slices 3–7) though `RESULTS.md` recorded 1280×720, so the frames' geometry claim is void while their reachability is not; the slice-8 "footer overflows at 1280×720" finding was an artifact of it and is **withdrawn**, disproved by slice 7's own frame. Re-capture is required with slice 9. |
| D-16 | MK-027's left rail was specified in a `done/` RFC and rendered nowhere — `left_rail::view` invoked by nothing, no RFC recording the substitution. R1 finding B3's class. | **MK-023 slice 7** (`0af9e09`) 2026-11-03. Deleted, per the disposition MK-023's accepted design decided: adding an unrendered surface to the keyboard model would have meant designing navigation for something users never see. Verified in review: gone from `crates/` entirely, not merely unreferenced. MK-027 keeps its dated correction in `done/`. |
| D-18 | Pointer focus and region focus could diverge: `focused_region` was set only by `enter_region`, which no pointer path called, so a click gave a field iced's native focus while the region model believed focus was elsewhere — and two focus indications could show at once. | **MK-023 slice 7** (`0af9e09`) 2026-11-03. A click landing in a region's item now enters that region: `mouse_area(...).on_release(…)` dispatching `RegionClicked`, which goes through `enter_region` rather than setting the two fields directly, so a click and a `Tab` onto the same region cannot disagree about what entering it means. `on_release`, not `on_press`, because a focused `text_input` captures `ButtonPressed`. |
| D-19 | A value-cycling `pick_list` gave the keyboard no affordance and `Enter` was inert on it: the control still rendered its ▾ chevron, promising a popup iced 0.14 cannot open, and `activation_message` returned `None`. The gap was in the architect's design, not the implementation. | **MK-023 slice 7** (`0af9e09`) 2026-11-03. `Enter` cycles forward, the same message the dropdown's own selection dispatches, and the handle becomes `⇅` while the cursor is on the control so it stops promising a popup. Both halves, as required — either alone was partial. Slice 8 added a fifth instance (the rule set's strategy) on the same mechanism rather than a second one. |
| D-20 | Six scrollables had no scroll-into-view, so the keyboard cursor could leave the viewport silently — the trace event list sharpest, being a list that grows while the server runs. It passed two reviews because slices 5 and 6 each substituted other rows for the design's *scroll visibility* edge case, **and the architect accepted the substitution reviewing slice 5**, his own handoff having invited it. | **MK-023 slice 7** (`0af9e09`) 2026-11-03, all six in one change. Slice 8 then folded the three pre-existing dispatches into the same `scroll_snap_target` rather than leaving two patterns side by side — with a regression test for each of the three, since a silently dropped dispatch is precisely how this defect reached live the first time. |
| D-23 | Controls the MK-023 audit inventory lists that no key reached — Test Rule's method buttons and fields, the top bar's switcher and global actions, the trace-activity footer links, the path assistant's five controls, the workspace menu's rows, the feedback banner's actions, and the Routes centre panel's own empty-state call to action. Three of them were named in MK-023's acceptance clause itself. | **MK-023 slice 9** (`83b5ffc`) 2026-11-03. Every one is a region, or recorded with the reason it is not (the palette's `✕`, covered by `Escape` like every other dialog's). Verified by an independent walk of `screens/` and `shell/` in review, including the `widgets::*_btn` helpers a raw `button(`/`on_press(` sweep misses. `screens/scripts.rs`, dead since before the cycle, was deleted. **And the census became a test** (`app/control_census.rs`): it counts each file's own control call sites and fails when the recorded table drifts — deliberately a literal-occurrence count, whose job is not to count controls correctly but to make a file's control surface changing impossible to do silently. |
| D-21 | **The feedback banner's colours failed WCAG AA in every preset.** `theme::banner_style` painted `ep.warning.weak.color` with `ep.background.base.text` — one iced-derived slot and one token chosen for a different surface — measuring **2.54 / 1.31 / 1.94 / 1.25** against a 4.5 floor, on the surface that announces what the application just did. **Found by the project owner by eye**, from watching the slice captures, and confirmed by measurement. Round 1 then showed the defect was wider than the register said: `banner_style` has **five** users, and a container's `text_color` does not reach a `button`, so the banner's own two controls, the rule editor's validation header, and the region cursor ring were all still unguaranteed — the header **worse than before**, because a shared style's fill changed under text nobody re-measured. | **Task 036, two rounds** (`33505c7`, `1248793`) 2026-11-04. Every colour this surface specifies now comes from snora's `warning`/`warning_text` — the one pairing guaranteed for this fill — measuring **5.43 / 10.63 / 8.42 / 14.85** against the 4.5 text floor and the 3:1 non-text floor: the message text, **both button labels in every button status** (a new `banner_text_button`, deliberately not reusing `button::text`'s own `scale_alpha` dimming, which was measured at **4.10:1** in Light *before* being rejected), the validation header and its icon (explicit colours dropped so they inherit — proven live, not assumed), and a per-surface `banner_cursor_style` for the ring. **A census test derived from the style's own call sites** fails if a sixth user appears, which is the guard the first round lacked: its test asserted the roles we had touched while the surface still failed. Verified in review by independent measurement of all four colours in all four statuses and all four presets, by planting a sixth site to confirm the census fires, and by capturing the one frame round 2 reported as uncapturable — its diagnosis was wrong (`refresh_validation` runs on edit, not at admission, and the seeded issue is in `error-scenarios.toml`), and the frame proves the fix. **Third and fourth instance of D-13's root cause, closed the same way: measure the pairing on our surface, never inherit a token's claim.** One note, not a defect: a thin-stroke glyph renders below its specified colour (3.64 where 5.43 is specified) because antialiasing never fills a pixel — it still clears the 3:1 icon floor. |
| D-25 | Two surfaces named the wrong action: a strategy edit's undo banner said "Moved rule", and the Create-workspace confirmation was titled "Switch workspaces?". The second was invisible until MK-023 slice 10 made the dialog draw on the Wizard at all. | **Task 036 round 1** (`33505c7`) 2026-11-04. Three new i18n keys in both locales: `UndoSettingChanged` ("Setting changed") and `ConfirmCreateWorkspace`/`…Body` ("Create workspace? / Unsaved edits will be lost."). The undo label was deliberately made **generic rather than strategy-specific** — `HistoryEntry::RootSetting` covers listener IP, port, TLS and log level too, so the handoff's own strategy-specific framing would have been wrong for four of five callers; the dev team read the variant rather than the instruction. Tests assert the key, not the string. The audit the task also required found eleven further mislabelled variants — **D-28**. |
| D-27 | The save-diff drawer's `Discard`/`Save all` row was clipped at 1280×720: `SheetSize::Ratio(0.32)` is shorter than its content, and D-5's relative-offset scroll-snap put position 1 of 3 at offset 0.5, which is not a visibility guarantee. | **Task 036 round 1** (`33505c7`) 2026-11-04. The action row moved **out of the drawer's one scrollable** into a sibling position below a divider — the same shape `Close` already had in `drawer_header` above it — and both fields were excluded from `scroll_snap_target`, so visibility no longer depends on scroll position at all. The sheet ratio is unchanged: the layout remedy was preferred to a taller sheet precisely because a taller sheet would only have postponed the same failure. Frame at the measured 1280×720 (`drawer-discard-whole.png`). |
| D-22 | An overlay could own the whole keyboard model on a screen that never drew it: `shell/view.rs`'s z-order, `order_for`'s override chain and `EscapePressed`'s dismissal chain each wrote the precedence out by hand and had drifted, and the one source that replaced them still answered *which overlay state is set* rather than *which overlay is drawn*. | **MK-023 slices 9 and 10** (`83b5ffc`, `9d405e1`) 2026-11-03. Slice 9 made `App::topmost_overlay()` the one source for the view, `order_for`, `EnterPressed` and `EscapePressed`, with exactly one overlay construct attached per frame — which also settled, at source, that `snora-0.42.0` pushes `dialog` then `sheet` into one `Stack`, so a sheet paints **over** a dialog (this register and `view.rs`'s own comment had both said the reverse; corrected). Slice 10 gated the four workspace-only overlays on `self.view != AppView::Workspace` — the same single test `App::view`'s own match makes, so the two cannot disagree — and drew the confirm dialog, the one overlay legitimately raised elsewhere, over Welcome, Dashboard and the Wizard. Verified by probe and by frame: the four-keystroke Dashboard trap gone (`a1-04`), the dialog drawn over the Dashboard (`b1-02`) and the Wizard (`c1-02`), and cancel restoring the cursor byte-for-byte (`b1-03` ≡ `b1-01`). **Accepted behaviour, recorded so it is not re-filed:** a workspace menu left open survives a trip through the Wizard and reappears on return — *derive, do not clear on transition* was chosen over enumerating six `self.view = …` sites. |
| D-26 | Two sources for each HTTP-method table — `test_rule.rs` and `regions.rs` for the five-method list, `rule_editor.rs` and two more `regions.rs` sites for the six-entry one — so a method added to a view would have had no keyboard activation, silently. | **MK-023 slice 10** (`9d405e1`) 2026-11-03. One `regions::HTTP_METHODS`; `editor_method_group` derives the editor's `"Any"`- and `""`-prefixed variants from it rather than spelling the names again; `len`/`active_index`/`activation_message` and both views read the same constant, with a test per table asserting they agree. |

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

**A dependency-reduction input, recorded 2026-10-21.** Three unmaintained
advisories — `RUSTSEC-2026-0370/0371/0372` on `zvariant`, `zbus_names` and
`zvariant_derive` — reach us on Linux through iced's **default**
`linux-theme-detection` feature (`iced_winit` → `mundy` → `zbus` 5). apimokka has
no use for it: it offers four explicit presets and reads no system appearance.
Turning the feature off would clear all three and drop the subtree, but
**MK-054's non-goals rule out disabling iced defaults**, so it is out of scope
here. The production project should decide deliberately — it is a free reduction
if it has no system-theme requirement, and a breaking one if it does.

**One further handover input, not from M11.** The production project should
adopt **current** snora rather than inheriting our 0.42.0 pin, and re-verify
appearance when it does. Our pin exists to hold M6's evidence still, which is a
reason with no force once M6 is over. As of 2026-09-12 that means 0.48.0 or
later, which additionally brings compiler-enforced `#![forbid(unsafe_code)]`
across all five snora crates and a published threat model (both 0.47.0) — worth
citing in a production assurance record, and neither worth an upgrade here,
where no release is produced.

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
- ~~continuous integration, including a cross-platform build matrix~~ —
  **no longer deferred; delivered 2026-08-03 by M6's L1 unit** (GitHub Actions,
  three platforms × two toolchains). Struck rather than deleted because the
  entry was cited in M4's and MK-054's reasoning. The production project still
  needs its own, on stronger grounds: path resolution, subprocess lifecycle,
  and a trace transport whose UDS option is unavailable on Windows;
- packaging reproducibility and source-archive-matches-tree validation. These
  are moot under the no-release policy; if an archival snapshot is ever
  requested, note that no validation check exists for it.

They must be reconsidered only after R2, with threat modeling for new file,
process, socket, and script-execution data flows.
