# Owner task 003 — M6 L3 session pack

**Issued:** 2026-09-16
**Issued by:** programme architect
**Governing design:** `rfcs/proposed/MK-056-ux-acceptance-protocol.md`, decisions 4–8
**Companion:** `rfcs/handoffs/MK-056-ux-acceptance-protocol/002-m6-participant-recruitment.md`
**Build at issue:** `bf16c48`

This is the document you run a session from. Read §0 before booking anything.

> ## ⚠ §0 is withdrawn — 2026-09-17. Do not book sessions yet.
>
> §0.2 says M9's run was "entirely by injected keys" and §0.3 says `wtype` can
> drive the app into any mode, theme and locale. **Both are false.** M9's own
> evidence shows Welcome has no keyboard path; the owner supplied two clicks.
> `wtype` reaches nothing past the mode picker, so the L2 plan cannot work.
>
> Worse for sessions: a keyboard-only participant is stopped at Welcome on every
> scenario — **D-12, S1**, in `ROADMAP.md`. Running sessions now would spend
> participants rediscovering it. Recruitment (task 002) can continue; booking
> waits for the D-12 decision. §1 onward stands for the pointer path.

---

## §0 — Two things to settle first, and they are the same fix

### §0.1 MK-056's acceptance criteria currently cannot be satisfied

Its acceptance criteria require:

> **L2 recorded pass for every mandatory combination in decision 7.**

Decision 7's four mandatory combinations are Japanese at the smallest window;
high-contrast with keyboard-only input; Guided at 200% text scale; and Expert at
the smallest window.

**L2 can produce none of them.** Decision 3's 2026-08-04 amendment migrated
every check requiring navigation to L3, because the application "has no CLI,
environment, or persistence hooks, so every launch begins at the mode picker in
`Light` with no workspace". Choosing a mode, switching a theme, and switching a
locale are all input. So all four mandatory combinations moved to L3 — **and the
acceptance criterion that names L2 was never amended to match.**

As written, M6 cannot be signed off. This must be resolved before sessions, not
discovered at the acceptance review.

### §0.2 The reason L2 gave up is no longer true

Decision 3's amendment says:

> `ydotool` and `wtype` were declined rather than installing kernel-level input
> injection on a live desktop.

**`wtype` is not kernel-level input injection.** It speaks
`zwp_virtual_keyboard_manager_v1`, a Wayland protocol the compositor implements;
`ydotool` is the one that needs `/dev/uinput`. The amendment treated them as one
thing and declined both on the stronger one's grounds.

We have since installed `wtype` and **the programme now depends on it**: M8's
capture was driven with it, M9's acceptance run went launch → mode selected →
palette open → row selected → command executed entirely by injected keys, and
dev-team task 014 makes a `wtype` run a standing verification requirement. The
one caveat is already known and recorded — niri ignores its own keybindings from
a virtual keyboard, so it is a false-negative oracle for delivery; test against
the application, not against a compositor bind.

### §0.3 The decision

**Recommended: reclaim L2, then run two sessions instead of three.**

With `wtype`, `scripts/ux/run-configuration.sh` can drive the application into
any mode, theme and locale, which restores to L2 everything the amendment
migrated except 200% text scale:

| Mandatory combination | With wtype |
|---|---|
| Japanese at the smallest window | **L2** — scriptable, screenshot-verifiable |
| High-contrast + keyboard-only | **L2** — this *is* a keyboard-driven run |
| Expert at the smallest window | **L2** — scriptable |
| Guided at 200% text scale | **L3** — see §2.4; needs an output-scale change, not input |

**What you get for it:** the acceptance criterion becomes satisfiable, and
participants stop carrying verification load MK-056 itself flagged as "not a
free reassignment". Sessions then measure comprehension, which is what only
people can do.

**What it costs:** one dev-team task, a day or two, before the first session.
**It does not delay recruitment** — start that now either way.

**If you decline**, that is defensible and I will not re-raise it: say so, and I
will amend MK-056's acceptance criterion to name L3 where L2 cannot deliver, and
the session plan below runs as three sessions with the heavier §2.3.

Everything from §1 onward assumes the **three-session** plan, which is the safe
one. If §0.3 is taken, session 3 shrinks to the 200% segment only.

---

## §1 — Coverage: what must be true when the sessions are over

MK-056 decision 7 requires **every value of every dimension at least once across
L2 and L3 combined** — not full coverage.

| Dimension | Values | Where covered |
|---|---|---|
| Audience mode | Guided, Expert | S1 Guided, S2 Expert |
| Locale | EN, JA | S1 EN, S2 JA |
| Theme | Light, Dark, HC Light, HC Dark | S1 Light→Dark (scenario 6), S3 HC Light + HC Dark |
| Window size | smallest (880×700), large | S1 large, S2 smallest |
| Input method | pointer, keyboard | S1/S2 pointer, S3 keyboard |
| Entry state | First boot, Returning, Empty, Populated, Dirty | See §2's per-scenario column — all five occur in S1 and S2 |

**Record the configuration for every finding.** MK-056: *"A finding without its
configuration is not reproducible and is not evidence."* The log template in §7
has a field per dimension for exactly this reason.

---

## §2 — Session plan

### §2.1 Session 1 — Guided newcomer, pointer, EN, large window

Persona A. **Starts at first boot.** Large window (1920×1080 logical or the
display's native), Light theme, English.

Run scenarios 1–6 in the order given in §4. Scenario 6 changes theme and locale
by design, so it goes last.

### §2.2 Session 2 — Expert user, pointer, JA, smallest window

Persona B. **Starts at first boot** — this is the second and final first-boot
observation in the programme.

Set the window to the smallest supported size (**880×700**) before launching,
and the locale to Japanese as part of scenario 6 — or, if you want the Japanese
expansion finding earlier, switch it at the start and note that scenario 6 then
covers theme and mode only.

This session carries two mandatory combinations at once: **Japanese at the
smallest window** and **Expert at the smallest window**. Expert shows every
control simultaneously, which is where density fails.

### §2.3 Session 3 — keyboard-only, high contrast

Either participant, **not at first boot** (it is spent). Re-run scenarios 1–5 in
both high-contrast presets, using the keyboard only.

**Under §0.3 this session largely disappears**, replaced by a scripted L2 run.
What cannot be scripted and stays here is the human judgement: *can you tell
where you are?* Focus visibility against high-contrast borders is a perception
question, not a reachability one.

### §2.4 The 200% text-scale segment

A segment, not a session — attach it to the end of S1 (Guided, as the mandatory
combination requires).

**Setup, and one trap.** The display is currently `2560x1440` at scale `1.2`
(logical `2133x1200`). Doubling to `2.4` at that mode gives a logical
`1066x600`, which is **shorter than the smallest supported window height** —
the application would not fit, and you would record an S1 that is an artefact of
your setup. Switch the mode first:

```sh
niri msg output DP-3 mode 3840x2160@59.997
niri msg output DP-3 scale 2.4          # 200% of the session's 1.2 baseline
# ... run the segment ...
niri msg output DP-3 mode 2560x1440@59.951
niri msg output DP-3 scale 1.2
```

`niri msg output` changes are **temporary and not written to the config**, so a
restart restores everything regardless.

"200%" means **double the baseline the session ran at**, which is 1.2 here, not
1.0. Record both numbers in the log.

---

## §3 — Facilitator rules

**You are not a classifier during the session.** MK-056: log every deviation
from the script verbatim; classification happens in analysis. Judging live
imposes cognitive load at the worst moment and invites you to grade your own
facilitation. **Where analysis is unsure, it records a rescue.**

### A rescue is

Any utterance or gesture conveying task-relevant information the participant had
not derived: **naming a control, pointing at one, confirming a choice, or
correcting a wrong path.**

"It's under Settings" is a rescue. So is nodding when they hover the right
button. So is "are you sure?" when they are about to go wrong.

### Not a rescue

- Reading or repeating the script **unchanged**.
- Encouraging think-aloud — *"what are you thinking?"*, *"say that out loud"*.
- Answering questions **about the session** — how long, can I stop, is this
  being recorded.
- Answering **domain** questions unrelated to the interface — what a 404 is,
  what JSONPath means. Domain ignorance is a recruiting variable, not an
  interface defect.

### The three things that most often go wrong

1. **Rescuing out of kindness during a silence.** Silence is data. Count to ten.
2. **Naming a feature while setting up.** See task 002 — the first screen is
   under test.
3. **Asking about text size before the end.** See §5. It invalidates the probe.

---

## §4 — The six primary scenarios

Each is stated as a **goal, never as instructions**. Read it once, verbatim.
Repeat it verbatim if asked. Do not rephrase — a rephrasing is a hint.

| # | Entry state | Say this |
|---|---|---|
| 1 | Returning | "This workspace already has some rules. Find the one that answers requests for `/api/orders`." |
| 2 | First boot | "Set up a new workspace, and add a rule so that a `POST` to `/api/login` with a JSON body containing `user` gets a response." |
| 3 | Populated | "Change that rule so it only matches when the header `x-env` is `staging`, then check that it does." |
| 4 | Populated | "A request came in and didn't match what you expected. Work out why." |
| 5 | Dirty | "Edit the fallback file, save it, then undo that — put it back the way it was." |
| 6 | Returning | "Make the application dark, switch it to Japanese, and then change how much detail it shows you." |

**Scenario 2 runs first in S1 and S2**, because it is the first-boot one. The
table is in reading order, not running order — running order is 2, 1, 3, 4, 5,
6.

Scenario 6's phrasing deliberately avoids "Guided", "Expert", "theme" and
"locale". If the participant cannot find the mode switch from *"how much detail
it shows you"*, that is the finding.

### Watch for this one unprompted, in every session

**The mode-picker card is fixed-width and centred** — measured at ~645 physical
pixels from 1056 to 2304 px of window width, with only the surrounding
whitespace changing. At the smallest tested size it fills ~61% of the width; at
1920×1080 it occupies under 10% of the window's area.

Nothing clips, so it is not a defect. But "the layout does not respond to window
size" reads as benign from small-window evidence and as an under-filled screen
at large ones, and participants remark on that unprompted. **Log it if they do,
against the window size.** It also sharpens the breakpoint evidence snora asked
us for: the problem is not only cramping at small sizes, it is failing to use
space at large ones.

---

## §5 — The readability probe

Owed by MK-059 resolution 3. **Fifty text sites were deliberately left at
12.0px** — one pixel above snora's floor — and that decision was explicitly
deferred to this evidence rather than settled by judgement. This probe is the
only thing that can answer it.

**Order matters. Observation first, question second.**

1. **Throughout every scenario, unprompted:** log any instance of leaning in,
   squinting, enlarging the window, or asking what something says. **Log it
   against the surface**, not as a general impression. One unprompted instance
   is worth more than any answer to step 2.
2. **At the very end of the session only**, on a screen carrying both tiers —
   the Routes workbench and Settings both do — ask exactly:

   > **"Is there anything here you find hard to read?"**

   Do **not** name text size, point at an element, or offer a scale.
3. **Record which tier** any named text belongs to — `CAPTION` 12.0,
   `BODY_SMALL` 14.0, or `BODY` 16.0 — **and which role**: ordinary body text,
   or the **muted** grey used for hints, counts, timestamps and metadata.
   Without the tier the finding cannot be acted on: a 12.0 complaint means the
   triage was too conservative; a 14.0 complaint means the tier itself is
   wrong. **Without the role it can be filed against the wrong half of the
   problem entirely** — muted text at `BODY` 16.0 that is hard to read is a
   *contrast* finding, not a size one, and the muted role passes AA (4.55
   Light / 5.01 Dark) while failing the 7:1 AAA bar that is the age-relevant
   one. If you cannot tell either, screenshot it and I will identify both from
   source.

   **Do not ask about colour**, any more than about size. The question in
   step 2 is the only question; the role is something *you* record about what
   the participant names, not something you put to them. (MK-056 amendment,
   2026-11-04.)

**Must be observed in at least one high-contrast preset** — S3, or the
high-contrast part of scenario 6 — since low-vision legibility is the case the
preset exists for.

**A null result is a real finding.** If nobody remarks on text at any tier,
unprompted or asked, that is evidence the conservative triage was correct, and
it is the evidence MK-059's decision 2 was deferred *to*. **Record it as a
finding, not as an absence of findings.**

---

## §6 — Severity, assigned in analysis and not in the room

| Severity | Definition |
|---|---|
| **S1 — Blocking** | A primary scenario cannot be completed without rescue; or state is lost, corrupted, or silently discarded; or an accessibility failure prevents completion on a supported configuration. |
| **S2 — Serious** | Completed, but with a wrong mental model, significant hesitation or backtracking, or a degrading accessibility failure. |
| **S3 — Polish** | Cosmetic or preference-level. |

**Objective findings take severity on a single observation.** A participant
failing without help, a lost edit, an unreachable control — observed fact, one
sighting, S1 applies.

**Interpretive findings** — confusion, a misleading label, a wrong mental model
— are **S2 on one observation** and escalate to **S1 only if seen again**, by
another participant *or* by the same participant in a different entry state.
That second axis is the one that matters: the same wording can read clearly from
a populated workspace and mislead from an empty one.

**S1 blocks R2 and may never be deferred.** S2 is fixed or deferred with a named
owner, rationale, user impact, and target step. S3 may be deferred freely. A fix
is re-tested with the same scenario and a recorded build SHA; an untested fix
does not close a finding.

---

## §7 — Per-session log

Copy this per session into
`.git-exclude/release-evidence/<date>-m6-l3-sessions/session-N.md`.

```markdown
# M6 L3 — session N

| Field | Value |
|---|---|
| Date |  |
| Persona | A (Guided newcomer) / B (Expert apimock-rs user) |
| Facilitator |  |
| Build SHA |  |
| Platform | Linux, niri <version> |
| Audience mode |  |
| Locale |  |
| Theme(s) |  |
| Window size |  |
| Output scale | baseline / 200% segment |
| Input method | pointer / keyboard |
| Renderer | wgpu / tiny-skia — check the app's startup log; iced falls back to tiny-skia on VMs, remote desktops and missing drivers |

## Scenario log

For each scenario: entry state, start time, whether it completed, and **every
deviation from the script verbatim**. Do not classify here.

### Scenario 2 — new workspace + rule (entry state: first boot)
- Completed: yes / no
- Rescues (verbatim):
- Observations (verbatim):

### Scenario 1 — find the rule for /api/orders (entry state: returning)
...

## Unprompted readability observations
Surface, what happened, window size, theme.

## End-of-session readability question
Asked: "Is there anything here you find hard to read?"
Answer (verbatim):
Named text and its tier:
(If nothing was named, write: "Null result — nothing named, unprompted or
asked." That is the finding.)

## Anything the participant asked to strike
```

---

## §8 — After both sessions

Hand me the session logs. I will:

1. classify every logged deviation into S1/S2/S3 per §6, applying the
   single-vs-second-sighting rule across sessions rather than within one;
2. reconcile the readability probe against MK-059's fifty retained sites and
   record the outcome in MK-059, including a null result — **and, where a
   finding names the muted role rather than a tier, measure that pairing in
   `theme/tests.rs` the way D-13, D-15 and D-21 were, rather than acting on
   the report alone** (MK-056 amendment, 2026-11-04);
3. produce the M6 exit evidence against MK-056's acceptance criteria and state
   plainly which are met and which are not;
4. raise any S1 as a blocking defect before R2 is considered.

**Do not fix anything you observe during a session, and do not let a participant
see a fix.** MK-056: M6 measures, it does not redesign.
