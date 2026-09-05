# RFC MK-056 — UX and accessibility acceptance protocol

**Status.** Proposed
**Tracks.** Stabilization roadmap M6 — UX acceptance evidence.
**Touches.** User-visible string localization, known no-op controls, a
cross-platform CI workflow, scripted GUI verification, the acceptance protocol
and its recorded evidence, and the accessibility requirements inherited by
production.

## Summary

Define how apimokka is validated for the purpose it exists to serve:
stakeholder review of workflows and interaction design.

Verification is **layered**, because the alternative — running every scenario on
every platform with human participants — is not achievable and pretending
otherwise would produce a dishonest result. Machine-checkable facts go to CI and
to scripted GUI automation; human sessions are reserved for the one thing
neither can measure, which is whether people understand what they are looking
at.

## Problem and motivation

The mockup exists to validate screen flows before production integration cost is
committed. That validation has never happened. Every milestone so far improved
correctness, governance, or verifiable contracts; none put the interface in
front of a person.

R1 recorded this as the programme's largest unretired question, and it is not a
technical one. A GO at R2 without UX evidence would certify a specification
nobody has exercised.

Two constraints shape the design:

- **Three platforms are required** — Linux, macOS, and Windows — and only Linux
  has ever been built or run.
- **Participants are the scarce resource.** Everything a machine can check must
  be checked by a machine, so session time is spent on comprehension. Decision 3
  records that the machine-checkable set on this host turned out smaller than
  assumed, which makes this constraint tighter rather than looser.

## Goals

1. Produce recorded, reproducible evidence that the principal workflows are
   completable by their intended audiences.
2. Obtain cross-platform evidence without requiring human sessions on every
   platform.
3. Make accessibility and layout checks repeatable, so a fix can be re-verified
   without booking a person.
4. Distinguish blocking workflow failure from confusion from polish by rule.
5. Convert what cannot be validated here into explicit production requirements.

## Non-goals

- Redesigning the interface. M6 measures; a finding implying redesign is
  recorded and escalated, not fixed in place.
- Fixing accessibility gaps iced 0.14 does not expose (decision 6).
- Production integration, file I/O, or any deferred feature.
- Module splitting; that is M5 / MK-057.

## Decision

### 1. Three verification layers

| Layer | Answers | Where it runs |
|---|---|---|
| **L1 — Cross-platform CI** | Does it build, and does the logic hold, on every supported platform? | GitHub Actions: Linux, macOS, Windows |
| **L2 — Scripted GUI verification** | Does it launch and stay responsive at each window size, and what does each size actually look like? (Narrowed 2026-08-04 — see decision 3) | Local Linux, `niri msg action` |
| **L3 — Human sessions** | Do people understand what they are seeing, and can they complete real tasks? | Local, Linux-primary |

Each layer answers a question the layer above cannot. A platform is "supported"
when L1 passes on it, not when a person has used it there — which is what makes
three-platform support achievable at all.

### 2. L1 — cross-platform CI

Add a GitHub Actions workflow running, on `ubuntu-latest`, `macos-latest`, and
`windows-latest`, for both stable and Rust 1.91:

```sh
cargo build --workspace --locked
cargo test --workspace --locked
```

**Scope is deliberately narrow.** The canonical gate's shell checkers, `cargo
audit`, `cargo doc`, and Clippy remain a local Linux concern; portability of
Bash checkers to Windows runners is a distraction from the question CI is being
added to answer. CI proves the workspace compiles and its logic holds on each
platform, which is exactly the evidence currently missing.

This reverses MK-054's deferral of CI, and the reversal is deliberate. That
deferral was sound for a single-platform mockup. With three platforms required,
CI is the only practical source of multi-platform evidence, and no amount of
local discipline substitutes.

`README.md:40` currently states a Linux prerequisite and attributes it to iced
0.14. That attribution is false — iced supports all three — and the line is
corrected as part of this work.

### 3. L2 — scripted GUI verification

**Amended 2026-08-04, after the capability probe.** This decision as originally
written assumed `xdotool` could drive the application. It cannot on this host,
and the amendment below records what L2 actually is rather than leaving an
aspiration in an accepted design.

**The probe result.** `scripts/ux/probe.sh` confirmed that apimokka's window is
a native-Wayland surface and therefore invisible to X11 clients by Wayland's
security model. `xdotool` is installed and functional; it simply cannot see or
target the window. No workaround was adopted: `ydotool` and `wtype` were
declined rather than installing kernel-level input injection on a live desktop,
for a short-lived repository, to automate checks a human performs anyway in L3.

**What L2 is.** Compositor IPC (`niri msg action`) still provides launch,
resize, and screenshot. Those work, are repeatable, and are genuinely useful:

- the application launches and stays responsive at each supported window size;
- resize is honoured rather than clamped, verified against physical pixels;
- screenshots per configuration, for human review.

**What L2 is not, and where those checks went.** Everything requiring
navigation needs input synthesis, which is unavailable. The application has no
CLI, environment, or persistence hooks, so every launch begins at the mode
picker in `Light` with no workspace — meaning no other screen, preset, or locale
is reachable without input. These therefore **migrate to L3**:

- keyboard reachability of primary-scenario controls;
- focus visibility;
- per-preset and per-locale layout, including Japanese expansion;
- 200% text scale, which decision 7 already records as not exercised here.

**The consequence for L3, stated plainly:** participants now carry verification
load this design assigned to a script. That raises what the sessions must cover
and should be reflected when scenarios are finalised — it is not a free
reassignment.

The layered model itself stands. Separating machine-checkable facts from human
judgement was correct; this host simply has a smaller machine-checkable set than
the design assumed.

### 4. L3 — human sessions

#### Personas

- **Guided newcomer.** Has not used apimock-rs; understands HTTP and JSON; has
  never seen this interface. Exercises Guided mode, first-launch, and the wizard.
- **Expert apimock-rs user.** Has hand-written apimock-rs TOML. Exercises Expert
  mode, the rule builder, and Test Rule — and judges whether the model the GUI
  presents matches the model the engine actually has.

Participants must not have contributed to this repository.

#### Entry state is a first-class dimension

The same person gets different results depending on the state the application
was in. This matters more than participant count, and it has a consequence that
constrains scheduling:

**First-boot is one-shot per participant.** Once someone has seen the mode
picker and the first-launch flow, they cannot un-see it. That observation cannot
be repeated with the same person at any price, so first-boot scenarios are
scheduled **first**, before anything else that would spend the participant's
naivety.

Declared entry states, one named per scenario:

| State | Meaning |
|---|---|
| **First boot** | No audience mode chosen; mode picker shown. One-shot per participant. |
| **Returning** | Audience mode already set; opens to Welcome. |
| **Empty workspace** | Workspace open, no rules. |
| **Populated workspace** | Workspace open with seeded rules. |
| **Dirty** | Unsaved edits and non-empty undo history present. |

Coverage is judged across states, not across headcount. A scenario verified only
from a populated, clean workspace is not verified for the empty or dirty case.

#### Primary scenarios

Release-blocking, each scripted as a goal and never as instructions, each
declaring its entry state:

1. Open an existing workspace and find the rule serving a given path.
2. Create a workspace from the wizard and add a rule matching a stated request.
3. Edit a rule's conditions and verify the change with Test Rule.
4. Inspect a trace event and determine why a request did not match.
5. Save and then revert a fallback-file draft.
6. Change theme and locale, and switch audience mode.

**Carried in from L2's first captures (2026-08-04).** The mode-picker card is
fixed-width and centred — measured at ~645 physical pixels across window widths
from 1056 to 2304, with only the surrounding whitespace changing. At the
smallest tested size it fills roughly 61% of the width; at 1920×1080 it occupies
under 10% of the window's area.

Nothing clips, so this is not a defect. But "the layout does not respond to
window size" reads as benign from small-window evidence and as an under-filled
screen at large ones, and that is the kind of thing participants remark on
unprompted. **Watch for it as an observation across window sizes**, and note it
also sharpens the breakpoint-threshold evidence snora has asked us for: the
problem is not only content cramping at small sizes, it is content failing to
use space at large ones.

### 5. Findings, severity, and the facilitator

#### Severity

| Severity | Definition |
|---|---|
| **S1 — Blocking** | A primary scenario cannot be completed without facilitator rescue; or state is lost, corrupted, or silently discarded; or an accessibility failure prevents completion on a supported configuration. |
| **S2 — Serious** | Completed, but with a wrong mental model, significant hesitation or backtracking, or a degrading accessibility failure. |
| **S3 — Polish** | Cosmetic or preference-level. |

**Objective findings establish severity on a single observation.** A participant
failing to complete a task without help, a lost edit, or an unreachable control
is observed fact, not opinion — one sighting is sufficient, and S1 applies.

**Interpretive findings** — confusion, a misleading label, a wrong mental model
— are graded S2 on a single observation and escalate to S1 only if seen again,
whether by another participant or by the same participant in a different entry
state. This is the axis that matters: the same wording can read clearly from a
populated workspace and mislead from an empty one.

S1 blocks R2. S2 is fixed or explicitly deferred under decision 8. S3 may be
deferred freely.

#### Facilitator rules

A rescue is any facilitator utterance or gesture conveying task-relevant
information the participant had not derived — naming a control, pointing at one,
confirming a choice, or correcting a wrong path.

**The facilitator does not classify during the session.** They log every
deviation from the script verbatim; classification happens in analysis. Judging
live imposes cognitive load at the worst moment and invites the person running
the session to grade their own performance. Where analysis is unsure, it records
a rescue.

Not rescues: reading or repeating the script unchanged, encouraging think-aloud,
answering questions about the session itself, and answering domain questions
unrelated to the interface — domain ignorance is a recruiting variable, not an
interface defect.

### 6. Accessibility scope

iced 0.14 exposes no button focus status and no assistive-technology API. These
are upstream gaps; failing the mockup for them would record an upstream
limitation as a project defect.

**Verified here**, largely at L2: non-colour status communication, high-contrast
theme contrast in practice, 200% text scale, keyboard reachability of every
primary scenario, and visible focus wherever iced provides it.

**Inherited by production**, recorded in `architecture.md`'s production-adapter
inheritance list: screen-reader and assistive-technology support, and custom
focus-ring rendering.

> ### Amendment, 2026-09-05 — the readability probe
>
> **Added under MK-059 resolution 3**, which made M10's typography scope
> conditional on this existing. It is owed before L3 sessions run.
>
> **What it exists to settle.** MK-059 found that **152 of 294 text-sizing call
> sites rendered at 12.0px** — snora's stated floor for text a user must read —
> because our scale jumped 16.0 to 12.0 with nothing between. M10 introduced the
> missing 14.0 tier and triaged 102 of those sites upward. **Fifty were left at
> 12.0 on purpose**, and that decision was explicitly deferred to session
> evidence rather than settled by judgement: timestamps, counts, file paths and
> positional metadata under a field.
>
> **So there is a live question this protocol must answer, and nothing else can.**
> Not "is 12.0 above the floor" — it is, by one pixel — but *does a real person
> reading a real screen find that text comfortable, and if not, which of it.*
>
> **Method — observation first, question second.** The order matters, because
> asking about text size teaches a participant to look for a problem they had not
> noticed.
>
> 1. **Unprompted, throughout every scenario:** record any instance of a
>    participant leaning in, squinting, enlarging the window, or asking what
>    something says. Log it against the surface, not as a general impression. An
>    unprompted instance is worth more than any answer to question 2.
> 2. **At the end of the session only**, on a screen carrying both tiers — the
>    Routes workbench and Settings both do — ask: *"Is there anything here you
>    find hard to read?"* Open, not leading. Do **not** name text size, point at a
>    specific element, or offer a scale.
> 3. **Record which tier** any named text belongs to (`CAPTION` 12.0,
>    `BODY_SMALL` 14.0, or `BODY` 16.0). Without that the finding cannot be acted
>    on, because the fix differs: a 12.0 complaint means the triage was too
>    conservative; a 14.0 complaint means the tier itself is wrong.
>
> **Environment.** The probe rides on scenarios already being run — it adds no
> session. It must be observed in **at least one high-contrast preset**, which
> §7's sampling already requires, since low-vision legibility is the case the
> preset exists for.
>
> **Severity, per §5.** Text a participant cannot read on a supported
> configuration is **S2** — it does not block completion, but it degrades a
> primary workflow. It reaches **S1** only if it prevents completion.
>
> **What a null result means, and it is a real result.** If no participant
> remarks on text at any tier, unprompted or asked, that is evidence the
> conservative triage was correct — and it is the evidence MK-059's decision 2
> was deferred *to*. Record it as a finding, not as an absence of findings.
>
> **What this probe does not do.** It does not measure contrast, which is L2's
> job and already asserted; it does not evaluate line-height separately from
> size, which no participant can decompose; and it does not revisit `DISPLAY`
> staying at 36.0, which is a recorded divergence rather than an open question.

### 7. Environment sampling

Full coverage across audience mode (2), locale (2), theme (4), window size (2),
input method (2), and entry state (5) is not required. Instead:

- **every value of every dimension appears at least once** across L2 and L3
  combined;
- these combinations are **mandatory**, being where defects cluster:
  - Japanese at the smallest supported window — text expansion and clipping;
  - high-contrast theme with keyboard-only input — focus against high-contrast
    borders;
  - Guided mode at 200% text scale — the mode carrying the most inline hints;
  - **Expert mode at the smallest supported window** — Expert shows every
    control at once, and this is where that density fails;
- build identity (commit SHA), platform, theme, locale, mode, window size, input
  method, and entry state are recorded per session and per L2 run. A finding
  without its configuration is not reproducible and is not evidence.

### 8. Deferral rule

A finding may be deferred only with a named owner, rationale, user impact, and
target step. **S1 may never be deferred.** Fixes are re-tested with the same
scenario and a recorded build identity — at L2 by re-running the script, at L3
by re-running the scenario. An untested fix does not close a finding.

## Preparation gate — a separate reviewed unit

Sessions do not begin until the following is delivered **and independently
reviewed as its own unit**, not folded into the acceptance review. Participant
time is the scarcest resource in the programme; spending it rediscovering known
defects produces noise, and burying the confirmation inside a later review means
nobody checks before the booking is made.

**a. Enabled no-op controls.** `crates/app/src/screens/routes.rs:132` and `:191`
dispatch `Message::Noop` behind enabled buttons. An enabled control that does
nothing is the most reliable way to destroy a participant's model of what the
application can do. Given the no-I/O boundary, **disabled-with-visible-reason is
the expected resolution**.

**b. Localization inventory.** A recorded, repeatable sweep of user-visible
literals in `screens/`, `shell/`, and `widgets/`. At the time of writing it
returns **14 candidates across three files** — a bounded task. Each is migrated
to an `apimokka_i18n::Key` or given a recorded exemption, following the
accelerator-notation precedent: platform or protocol notation is exempt, prose
is not.

**c. `README.md:40`** corrected per decision 2.

**Exit:** no enabled no-op controls; no unexplained user-visible English; the
canonical gate green.

## Acceptance criteria

- Preparation gate delivered and separately accepted.
- L1 green on all three platforms, both toolchains.
- L2 recorded pass for every mandatory combination in decision 7.
- Every primary scenario completed by a participant of the intended persona,
  across the declared entry states, on both pointer and keyboard paths.
- No unresolved S1 finding.
- Accessibility items verified or recorded as inherited production requirements.
- All deferrals satisfy decision 8.

## Alternatives considered

**Human sessions on all three platforms.** Rejected as unachievable and
unnecessary; L1 and L2 supply platform and mechanical evidence, leaving sessions
to measure comprehension, which does not vary materially by platform.

**Skip CI, verify platforms by hand.** Rejected. Manual multi-platform runs decay
precisely because they are tedious, and they cannot be re-run cheaply after a
fix.

**Expert-only sessions.** Rejected: Guided mode and first-launch exist for people
who have never used apimock-rs, and an expert cannot report a newcomer's
confusion.

**Rely on the reducer and view-build suites.** Rejected. They prove
construction, not comprehension. No test here can tell you a label misleads.

**Defer M6 to production.** Rejected — it inverts the mockup's purpose, which is
to learn this while changing it is still cheap.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| Participants unavailable | The one unmitigable risk, and why recruitment should begin before this RFC is accepted |
| First-boot naivety spent accidentally | First-boot scenarios scheduled first, by rule |
| Windows/macOS CI runners diverge from real desktop behaviour | L1 claims only build and logic; interaction claims come from L2/L3 and are scoped to what was actually run |
| Facilitator grades generously | Classification moves out of the session; ties record a rescue |
| Findings imply redesign | Recorded and escalated; M6 measures, it does not redesign |
| Session findings arrive after M5 restructuring | M5 is behaviour-neutral, so scenarios stay valid; re-tests cite build SHAs |

## Resolved review questions

Raised at design review and settled by the project owner on 2026-08-02:

1. **Participant count versus situation.** The material axis is **entry state**,
   not headcount — the same person gets different results on first boot than on
   later boots. Replication-by-headcount is replaced by state coverage plus the
   objective/interpretive split in decision 5, which also removes a
   contradiction in the original draft, where a two-participant floor combined
   with a replication requirement made S1 unreachable.
2. **Facilitator classification** moves out of the live session into analysis.
3. **Expert mode at the smallest window** added as a fourth mandatory
   combination; the original three all stressed Guided.
4. **All three platforms are required**, satisfied through the layered model
   rather than a verify-or-drop-the-claim choice.
5. **The preparation gate is a separate reviewed unit.**

## Status

Design accepted by the project owner on 2026-08-02, including the five
resolutions above. Under the four-folder lifecycle this RFC remains `Proposed`
until its implementation ships, at which point it moves to `done/`; design
acceptance is not a folder transition.

Acceptance of this design is recorded separately from authorization to
implement. The project owner authorized implementation on 2026-08-02, assigned to the dev
team. Authorization covers the scope defined here and nothing beyond it; the
non-goals remain binding. The preparation gate is delivered and reviewed as its
own unit before any human session is scheduled.
