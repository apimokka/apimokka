# RFC MK-056 — UX and accessibility acceptance protocol

**Status.** Proposed
**Tracks.** Stabilization roadmap M6 — UX acceptance evidence.
**Touches.** User-visible string localization, known no-op controls, the
acceptance protocol and its recorded evidence, platform verification scope, and
the accessibility requirements inherited by production.

## Summary

Define how apimokka is validated for the purpose it exists to serve:
stakeholder review of workflows and interaction design. This RFC specifies the
personas, scenarios, severity taxonomy, facilitator rules, environment sampling,
and evidence format — and the preparation work that must land *before* any
session runs, so participant time is spent finding unknown problems rather than
rediscovering known ones.

M6 is the only milestone whose critical input cannot be produced inside this
repository. Everything else in the programme is work; this needs people.

## Problem and motivation

The mockup's stated purpose is to validate screen flows and interaction design
before production integration cost is committed. That validation has never
happened. Every milestone so far has improved correctness, governance, or
verifiable contracts; none has put the interface in front of a person.

R1 recorded this plainly: the programme's largest unretired question is not
technical. A GO at R2 without UX evidence would certify a specification nobody
has exercised.

## Goals

1. Produce recorded, reproducible evidence that the principal workflows are
   completable by their intended audiences.
2. Distinguish blocking workflow failure from confusion from polish, by a rule
   applied consistently rather than by judgement in the moment.
3. Cover pointer and keyboard paths, both audience modes, both locales, the
   accessible themes, and the declared platforms — by risk-based sampling, not
   exhaustive combination.
4. Convert what cannot be validated here into explicit production requirements
   rather than silent gaps.

## Non-goals

- Redesigning the interface. M6 measures; it does not re-architect. A finding
  that implies redesign is recorded and escalated, not fixed in place.
- Fixing accessibility gaps that iced 0.14 does not expose (see decision 7).
- Production integration, file I/O, or any deferred feature.
- Module splitting; that is M5.

## Decision

### 1. Preparation gate — known defects are fixed before sessions run

No session runs until the following are closed. Participant time is the
scarcest resource in this programme, and spending it on defects already known to
the team produces noise rather than findings.

**a. Enabled no-op controls.** `crates/app/src/screens/routes.rs:132` and `:191`
dispatch `Message::Noop` behind enabled buttons ("Add fallback file", "Add
.rhai"). An enabled control that does nothing is the single most reliable way to
destroy a participant's model of what the application can do. Each must be
removed, disabled with a visible reason, or implemented. Given the no-I/O
boundary, **disabled-with-reason is the expected resolution**; implementing them
would require file I/O and is out of scope.

**b. Localization inventory.** A repeatable sweep of user-visible string
literals in `screens/`, `shell/`, and `widgets/`. At the time of writing it
returns **14 candidates across three files** (`screens/routes.rs`,
`screens/wizard.rs`, `shell/bottom_drawer.rs`) — a bounded task, not an open
audit. Each literal is either migrated to an `apimokka_i18n::Key` or given a
recorded exemption. The accelerator-notation exemption established by handoff
002 is the precedent for the exemption form: platform or protocol notation is
exempt; prose is not.

The sweep command must be recorded in the evidence so the inventory is
reproducible rather than a one-time manual pass.

**Preparation exit gate:** no enabled no-op controls remain; the inventory
returns no unexplained user-visible English; the canonical gate passes.

### 2. Personas — two floors, not two targets

At minimum:

- **Guided newcomer.** Has not used apimock-rs. Understands HTTP APIs and JSON.
  Has never seen this interface. Recruited to exercise the Guided audience mode,
  the first-launch flow, and the wizard.
- **Expert apimock-rs user.** Has written apimock-rs TOML by hand. Knows rule
  sets, matching, and fallback behaviour. Recruited to exercise Expert mode, the
  rule builder, and Test Rule — and to judge whether the model the GUI presents
  matches the model the engine actually has.

Two is the floor. A third participant in either role materially improves
confidence, because a single participant's confusion is indistinguishable from
that participant's idiosyncrasy. Where only two are available, findings from a
single participant are recorded as **unreplicated** and cannot alone establish a
blocking severity.

Participants must not have contributed to this repository.

### 3. Primary scenarios

These five are release-blocking. Each is scripted as a goal, never as
instructions:

1. Open an existing workspace and find the rule that serves a given path.
2. Create a workspace from the wizard and add a rule matching a stated request.
3. Edit a rule's conditions and verify the change with Test Rule.
4. Inspect a trace event and determine why a request did not match.
5. Save and then revert a fallback-file draft.

Plus one settings task: change theme and locale, and switch audience mode.

Each is run on both pointer and keyboard paths — not necessarily by the same
participant, but every scenario must have both paths covered somewhere in the
session set.

### 4. Severity taxonomy

Applied by rule, recorded per finding:

| Severity | Definition |
|---|---|
| **S1 — Blocking** | The participant cannot complete a primary scenario without facilitator rescue; or state is lost, corrupted, or silently discarded; or an accessibility failure prevents completion on a supported configuration. |
| **S2 — Serious** | The scenario completes, but with a wrong mental model, significant hesitation or backtracking, or an accessibility failure that degrades without preventing. |
| **S3 — Polish** | Cosmetic or preference-level; does not affect completion or comprehension. |

S1 findings block R2. S2 findings are fixed or explicitly deferred under
decision 8. S3 findings are recorded and may be deferred freely.

### 5. Facilitator rules — "rescue" is defined, not judged

Without a precise definition, "no scenario required facilitator rescue" is
unmeasurable and will be graded generously by the person who wants to pass.

**A rescue is any facilitator utterance or gesture that conveys task-relevant
information the participant had not derived.** Naming a control, pointing at
one, confirming a choice ("yes, that one"), or correcting a wrong path all count.

**Not rescues:** reading the scenario script verbatim; repeating it unchanged;
encouraging think-aloud ("what are you looking at?"); answering questions about
the session itself; and answering domain questions unrelated to the interface
("what is a rule set?" — answerable, since domain ignorance is a recruiting
variable, not an interface defect).

When in doubt, record it as a rescue. The count matters more than any single
judgement.

### 6. Environment sampling

Full Cartesian coverage across audience mode (2), locale (2), theme (4),
platform (3), window size (2), and input method (2) is 192 configurations and is
not required. The rule instead:

- **Every value of every dimension appears at least once** across the session
  set.
- These high-risk combinations are **mandatory**, because this is where layout
  and contrast defects cluster:
  - Japanese locale at the smallest supported window size — text expansion and
    clipping;
  - High-contrast theme with keyboard-only input — focus visibility against
    high-contrast borders;
  - Guided mode at 200% text scale — the mode with the most inline hint text.
- The exact build identity (commit SHA), platform, theme, locale, mode, window
  size, and input method are recorded per session. A finding without its
  configuration is not reproducible and does not count as evidence.

### 7. Accessibility scope — verify what is verifiable, inherit the rest

iced 0.14 exposes no button focus status and no assistive-technology API. These
are upstream gaps, and failing the mockup for them would be recording an
upstream limitation as a project defect.

**Verified here:** non-colour status communication (every status carries glyph
and label), high-contrast theme contrast in practice, 200% text scale where the
platform supports it, keyboard reachability of every primary scenario, and
visible focus wherever iced provides it.

**Inherited by production, recorded explicitly as requirements:** screen-reader
and assistive-technology support, and custom focus-ring rendering. These join
the production-adapter inheritance list in `docs/src/architecture.md` rather
than being logged as M6 failures.

### 8. Platform verification is separate from UX sessions

Cross-platform is declared scope, but only Linux has ever been built or run.
These are two different questions and conflating them will produce a dishonest
result.

- **UX sessions** may be Linux-primary. Interaction design does not vary
  materially by platform, and participant availability should not be constrained
  by hardware.
- **Platform verification** — build, run, and a visual smoke pass of each
  primary screen — must be performed on every platform claimed as supported.

**If a platform cannot be exercised, it must be removed from the supported
claim** in `README.md` and the roadmap, not silently carried. A supported
platform with no evidence is a claim, and this programme does not make claims it
has not tested. This is a decision the project owner must make explicitly if
macOS or Windows hardware is unavailable.

### 9. Deferral rule

A finding may be deferred only with a named owner, a stated rationale, the user
impact, and a target step. **S1 findings may never be deferred.** Fixes receive
targeted re-test using the same scenario script and a recorded build identity;
an untested fix does not close a finding.

## Evidence format

Per session: participant role, build SHA, full configuration, per-scenario
completion, rescue count, critical-error count, elapsed time where useful, and
verbatim participant observations where informative.

Per finding: severity, scenario, configuration, reproduction steps, whether
replicated across participants, disposition, and — where fixed — the re-test
record.

Stored under `.git-exclude/release-evidence/`, following the existing dated
convention.

## Acceptance criteria

- The preparation gate (decision 1) closed before the first session.
- Every primary scenario completed by at least one participant of the intended
  persona, on both pointer and keyboard paths across the session set.
- No unresolved S1 finding.
- Every dimension value covered, and all three mandatory high-risk combinations
  run.
- Platform verification recorded for every claimed platform, or the claim
  reduced.
- Accessibility items either verified or recorded as inherited production
  requirements.
- All deferrals satisfy decision 9.

## Alternatives considered

**Expert-only sessions.** Cheaper to recruit and more articulate feedback.
Rejected: Guided mode and the first-launch flow exist specifically for people
who have never used apimock-rs, and an expert cannot report a newcomer's
confusion.

**Skip UX and rely on the reducer and view-build test suites.** Rejected. Those
prove construction, not comprehension. No test in this repository can tell you
that a label misleads.

**Defer M6 to production.** Rejected — it inverts the mockup's purpose. The
entire justification for building a prototype before production integration is
to learn this now, while changing it is cheap.

**Full Cartesian environment coverage.** Rejected as disproportionate at 192
configurations; decision 6's sampling rule with mandatory high-risk combinations
gives most of the signal at a fraction of the cost.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| Participants unavailable | The single unmitigable risk, and the reason recruitment should start before this RFC is accepted rather than after |
| Findings imply redesign rather than repair | Record and escalate; M6 measures, it does not redesign. A redesign finding is an owner decision, not an M6 fix |
| Two participants cannot distinguish idiosyncrasy from defect | Unreplicated single-participant findings cannot alone establish S1 |
| Facilitator grades rescues generously | Decision 5 defines rescue explicitly; ties resolve toward recording a rescue |
| macOS/Windows hardware unavailable | Decision 8 forces an explicit scope reduction rather than an untested claim |
| Session findings arrive after M5 restructuring | M5 is behaviour-neutral, so findings remain valid against it; re-test uses the recorded build SHA |

## Review questions

1. Is the two-persona floor sufficient, given that unreplicated findings cannot
   establish S1 — or should three participants be a hard requirement?
2. Is the rescue definition in decision 5 workable for a facilitator applying it
   live, or too strict to use in practice?
3. Are the three mandatory high-risk combinations the right ones, and is
   "every dimension value at least once" a sufficient sampling floor?
4. Does decision 8's forced choice — verify each platform or drop the claim —
   correctly reflect available hardware?
5. Should the preparation gate (decision 1) be a separate reviewed unit before
   this RFC's sessions are authorized, or is it fine as this RFC's first step?

Creation of this Proposed RFC does not authorize implementation.
