# RFC MK-059 — Typography and readability

**Status.** Proposed
**Tracks.** Text scale, line-height, readability floor.
**Touches.** Every text-bearing surface — 294 sizing call sites.
**Amends.** MK-022 (visual design system), which owns `theme::size`.
**Requested by.** Project owner, 2026-08-15 — "introduce snora's typography
support to improve app readability".

## Summary

apimokka sizes text from seven hardcoded constants in `theme.rs` and **never
sets line-height anywhere**. snora has carried a six-role text scale — each role
holding a size *and* a line-height multiplier — since v0.20, reachable from the
version we already ship. This RFC adopts it.

The headline is not the scale. It is that **more than half of this
application's text renders at the minimum legible size**, and that the cause is
a gap in our scale rather than a series of individual choices.

## Problem

### 1. 152 of 294 text sizes are at the readability floor

| Constant | Value | Call sites |
|---|---|---|
| `CAPTION` | 12.0 | **152** |
| `BODY` | 16.0 | 98 |
| `SECTION` | 18.0 | 19 |
| `BODY_STRONG` | 16.0 | 10 |
| `TITLE` | 24.0 | 10 |
| `MONO` | 13.0 | 3 |
| `DISPLAY` | 36.0 | 2 |

`CAPTION` is commented "hints, metadata". At 152 sites it is not hints — it is
the default for anything that is not body text. snora's readability guide sets
**12 logical pixels as the floor** for text a user must read. We are at the
floor, not below it, for the majority of the interface.

### 2. The cause is a missing tier, not 152 bad decisions

Our scale jumps **16.0 → 12.0** with nothing between. Anything that should read
as secondary has one place to go, and it is the floor.

snora supplies exactly what is missing, twice:

| snora role | Size | Line height | For |
|---|---|---|---|
| `body` | 16.0 | 1.4 | ordinary explanatory text |
| `body_small` | **14.0** | 1.35 | secondary metadata, compact help |
| `label` | **14.0** | 1.2 | button, field and chip labels |
| `title` | 18.0 | 1.3 | card / dialog / notice title |
| `heading` | 24.0 | 1.25 | page or section heading |
| `display` | 32.0 | 1.2 | rare major page title |

Our existing values already agree with snora's on three of six — `BODY` 16.0 =
`body`, `SECTION` 18.0 = `title`, `TITLE` 24.0 = `heading`. This is not a
redesign. It is filling a hole.

### 3. Line-height is set nowhere

Zero `line_height` or `LineHeight` call sites in `crates/app/src`. Every
wrapping paragraph in this application renders at iced's default.

snora's own documentation said in four places that `TextRole.line_height` was
vocabulary-only because iced 0.14 did not expose line-height. **It does**, and
they have corrected all four:

```rust,ignore
iced::widget::text(body)
    .size(tokens.typography.body.size)
    .line_height(LineHeight::Relative(tokens.typography.body.line_height))
```

One of those four places was an item in their contributor accessibility
checklist instructing reviewers to skip line-height — a control telling people
not to look, which is why it survived several releases. Worth recording as a
pattern, not as their embarrassment: **our own MK-023 line 93 was an unticked
checklist item that everyone read past.** Same failure, different project.

**Superseded by helpers, 2026-08-19.** snora 0.38.0 (their RFC-068) adds six
`<role>_line_height` helpers beside the existing size helpers, returning
`LineHeight::Relative` directly. They withdrew the sentence in `typography.md`
that told consumers to read the multiplier off the field and wrap it themselves.

We are unaffected by the withdrawal — M10 is unimplemented, so no wrapper of
ours exists to replace — but **the implementation should use the helpers rather
than hand-wrapping**, and therefore needs a snora version that has them.

**Verify the exact import path against the shipped crate before writing the
task.** The helpers live in `snora-style::text`; we reach that layer through
`snora::design::style::*` and do not depend on `snora-style` directly. This
records snora's claim, not our compilation of it — the same caution MK-024
carries about `design::responsive_render`.

## Goals

- A text scale with a usable secondary tier, so nothing lands at the floor by
  default.
- Line-height applied to prose, from the token vocabulary rather than by hand.
- One owner for the scale, sourced from `tokens.typography`.

## Non-goals

- **Mechanically rewriting all 294 call sites.** Scope is defined by surface,
  not by count — see decision 4.
- **Font family or weight changes.** Out of scope entirely.
- **Per-preset typography.** All four snora presets share
  `Typography::default_roles()`; there is no variation to plumb today.
- **Restyling snora's own widgets.** We have zero `snora::widget::*` call sites,
  so their known gap — the notice widget rendering its title at `label_size` —
  does not reach us.

## Decision

### 1. Source the scale from `tokens.typography`, not constants

`theme::size` becomes accessors over `tokens.typography` rather than free
constants. Today this changes no value except where decision 2 and 3 say so; it
means a future preset *can* vary typography, and that the scale has one
definition instead of two that can drift.

### 2. Introduce the missing 14.0 tier

Add `body_small` (14.0/1.35) and `label` (14.0/1.2). These are the tier whose
absence produced finding 1.

### 3. Reconcile the two values that disagree

- **`DISPLAY` 36.0 vs snora `display` 32.0.** Ours is larger. Used twice. Adopt
  32.0 unless the welcome hero demonstrably needs more, in which case record why.
- **`CAPTION` 12.0 and `MONO` 13.0 have no snora role.** Keep both, but `CAPTION`
  is retained for *genuine* captions only — the 152 sites are triaged under
  decision 4, not preserved wholesale.

### 4. Triage the 152 `CAPTION` sites by surface, and do not touch labels

snora's readability guide is explicit that line-height matters for **prose that
wraps** and not for labels, which are single-line and where extra leading only
adds noise.

So the work is bounded by what the text *is*:

| Surface | Action |
|---|---|
| Wrapping prose — empty states, help text, dialog bodies, mode-picker descriptions | `body` + line-height |
| Secondary metadata, compact help | `body_small` + line-height |
| Button, field, chip, tab labels | `label`, **no line-height** |
| Genuine captions — timestamps, counts, hints under a field | `CAPTION` retained |

**Estimating the split is part of the implementation task, not this RFC.** The
one thing this RFC fixes is that "smaller than body" stops meaning "12.0".

### 5. Line-height only where text wraps

Apply the role's line height to prose and secondary prose — via snora's
`<role>_line_height` helper (0.38.0+). Do not apply it to labels, chips, or
single-line values.

**Qualified 2026-08-19 against the measured baseline.** iced's silent default is
`LineHeight::Relative(1.3)` (`iced_core-0.14.0/src/text.rs:215-219`), which is
what every piece of text in this application renders at today. Against that
baseline the six roles are **not equally worth applying**:

| Role | Line height | vs iced's 1.3 default |
|---|---|---|
| `body` | 1.4 | **looser — the clear gain** |
| `title` | 1.3 | identical; no observable effect |
| `body_small` | 1.35 | slightly looser |
| `heading` | 1.25 | **tighter** |
| `label` | 1.2 | tighter (labels excluded anyway) |
| `display` | 1.2 | **tighter** |

**Only `body` is unambiguously a readability improvement.** Applying the scale
reflexively would make headings and display text *tighter* than they are now
while calling it a readability change. Prioritise `body`; apply the others
because they are correct for the role, not because they improve legibility, and
say which is which in the submission.

Measured by the dev team against snora's RFC-070 and confirmed against iced's
own source.

## Risks and mitigations

| Risk | Assessment |
|---|---|
| **Vertical rhythm shifts** — line-height 1.4 makes prose blocks taller than iced's default. Fixed-height cards, dense lists and the bottom drawer could clip or overflow. | **The main implementation risk.** Requires visual checking, not just compilation. Bounded by decision 4 limiting line-height to prose. |
| Raising 152 sites from 12.0 to 14.0 makes content taller and wider | Same class. Triage exists partly to keep it proportionate. |
| Appearance change invalidates prior visual evidence | Handled by sequencing below. |
| Regression in `high_contrast_*` presets | Typography is preset-invariant today, so low — but the four-preset check applies as it did for M8. |

## Sequencing

**After M9, and last before M6's sessions.** Three constraints fix this:

- **Not before owner task 001's capture.** It would invalidate M8's evidence.
- **Not concurrently with M9 or task 015.** Three appearance-or-behaviour
  changes at once cannot be attributed when something looks wrong — the lesson
  from `f7356c6`.
- **Before M6, and as the last change before it**, so participants evaluate the
  typography we intend to hand over, and so nothing moves underneath it
  afterwards.

## Acceptance evidence

- Screenshots of the four presets before and after, on the prose-heaviest
  surfaces — at minimum the mode picker, an empty state, and a dialog body.
- Confirmation that no fixed-height surface clips at the new line-heights.
- A count of remaining `CAPTION` sites, with the triage rationale.
- `bash scripts/check-release-gates.sh` green; CI green on three platforms.

## Review questions

1. **`DISPLAY` 36.0 → 32.0?** Adopting snora's value costs a slightly smaller
   welcome hero. Keeping 36.0 means one deliberate divergence, recorded.
2. **How far to triage the 152 `CAPTION` sites?** All of them, or only the
   prose-bearing surfaces, leaving genuine metadata alone? Fewer changes mean
   less visual churn before M6.
3. **Is readability a topic M6 should probe explicitly?** MK-056's scenarios do
   not currently ask about text comfort. If this lands before the sessions, they
   are the natural place to find out whether it worked.

### Resolutions

Accepted by the project owner 2026-08-15 without separate answers to the three
questions above. Resolved by the architect as below; each is reversible, and
**resolution 2 is the one that materially changes scope** — say so if you would
rather it went the other way.

**1. Keep `DISPLAY` at 36.0. Record the divergence.**

Reversed from the draft's suggestion, on evidence in the code. `theme.rs:73`
reads `DISPLAY: f32 = 36.0; // welcome hero (was 32)`, and its neighbours carry
the same marks — `BODY` "was 14 — comfort, WCAG", `SECTION` "was 17", `TITLE`
"was 22". **Our scale has already been through a deliberate readability uplift**,
and 36.0 is its outcome, not an accident of drift.

Adopting snora's 32.0 would undo a readability decision inside a readability
RFC, on two call sites, for consistency alone. The divergence is recorded here
instead: apimokka's `display` is 36.0 because a prior pass raised it for comfort.
Line-height still comes from `tokens.typography.display.line_height`.

**2. Triage prose-bearing surfaces only. Leave genuine metadata at `CAPTION`.**

Rewriting all 152 sites before M6 maximises visual churn immediately before the
sessions that are supposed to evaluate the result, and would be done on
judgement rather than evidence — the same guessing this programme keeps finding
in its own record.

Scope for the implementation task is decision 4's first three rows: wrapping
prose, secondary metadata that wraps, and labels. Timestamps, counts and
single-line hints stay at `CAPTION` **for now**, deliberately, with the question
left open rather than closed by assumption.

**3. Yes — and resolution 2 is what makes it worth doing.**

Because some text will still sit at the 12.0 floor after this RFC lands, M6 can
answer whether that is acceptable, on evidence, from people who are not us.
Without resolution 2 there would be nothing left to ask about.

Requires an MK-056 amendment adding a readability probe to the session
scenarios. **That amendment is owed before M6's sessions run** and is tracked as
part of M10 rather than left implicit here.

## Status

**Accepted** 2026-08-15 by the project owner. Not yet implemented.

Scheduled as **M10**, after M9 and task 015, and last before M6's sessions. The
implementation task will be issued when that window opens, not now: M9 changes
the command palette, and this RFC's triage should be performed against the code
as it will then be, rather than against code about to change.
