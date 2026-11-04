# Dev-team task 036, round 2 — every colour on the banner

**Issued:** 2026-11-04
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** [MK-022](../../done/MK-022-visual-design-system.md) § colour roles
**Follows:** `036-d21-d25-banner-contrast-and-labels.md`, round 1 — Parts B
and C accepted (**D-25**, **D-27** closed); **D-21 stays open**
**Review:** `.git-exclude/reviewed/2026-11-04-task036-d21-d25-d27-review.md`

---

## 1. Purpose

Round 1 fixed the banner's fill and its message text, and the pairing it
chose is right: `warning`/`warning_text` measures 5.43 / 10.63 / 8.42 /
14.85, confirmed independently. **D-21 does not close**, because
`banner_style` is a shared style with five users and a container's
`text_color` does not reach a button. Three more colours on those surfaces
are below AA, and one of them is worse than before round 1.

This round finishes the defect. It is the same remedy, applied to everything
the surface actually draws.

## 2. What still fails, measured

**(a) The banner's own two buttons** — "Show/Hide details" and the "✕"
dismiss, both `button(...).style(iced::widget::button::text)`, which paints
with `ep.background.base.text`: the exact token D-21 named.

| Preset | Message (fixed) | Both buttons | Floor |
|---|---:|---:|---|
| Light | 5.43 ✓ | **3.28** ✗ | 4.5 |
| Dark | 10.63 ✓ | **1.49** ✗ | 4.5 |
| HC Light | 8.42 ✓ | **2.50** ✗ | 4.5 |
| HC Dark | 14.85 ✓ | **1.41** ✗ | 4.5 |

Sampled from round 1's own frames as well as from the tokens:
`dark-error-banner.png` draws both in `rgb(241,243,246)` on `rgb(251,191,74)`.

**(b) `screens/routes/rule_editor.rs:137`'s validation warning**, which puts
`theme::muted` on a `banner_style` container — the pairing round 1 moved the
banner's own detail text *off*, for exactly this reason:

| Preset | Before round 1 | After round 1 |
|---|---:|---:|
| Light | 1.42 | **1.10** |
| Dark | 2.36 | **2.08** |
| HC Light | 1.05 | **1.35** |
| HC Dark | 1.46 | **1.29** |

**Three of four are worse than they were.** Changing a shared style's fill
without re-measuring its other users is how.

**(c) The region cursor ring on this surface.** `theme::cursor_style` draws
in the primary colour: **2.33 / 1.13 / 1.79 / 1.19** against the new fill.
The banner's three controls became regions in MK-023 slice 9, so the ring is
the only indication of keyboard position there, and in the dark presets it
cannot be seen. The floor here is **3:1** (non-text), not 4.5.

## 3. Scope

1. **Every colour drawn on a `banner_style` container takes a pairing
   guaranteed against that fill.** The five users are `shell/view.rs:270`,
   `screens/routes/rule_editor.rs:137`, `screens/test_rule.rs:79` and `:106`,
   `screens/trace.rs:255`. Walk them; the last three inherit the container's
   `text_color` and already pass — confirm that rather than assume it.
2. **The two buttons.** Give them a style whose label colour is
   `theme::banner_text`, not iced's `button::text`. Keep them visually
   secondary by means other than colour if you need to — this surface has
   exactly one guaranteed foreground.
3. **The rule editor's validation header.** Same treatment, or drop the
   explicit colour so it inherits the container's. Either is fine; say which
   and why.
4. **The cursor ring.** Either `cursor_style` takes a colour measured against
   the surface it is drawn on, or the banner's controls carry their own
   cursor treatment. **Do not** solve it by removing the indicator.
5. **D-28 is not this round's.** The eleven mislabelled history variants are
   recorded for a future task — see §5.

## 4. Non-change scope

- **No change to the fill or the message text.** Round 1's pairing is
  accepted; this round adds to it, it does not revisit it.
- **No muted-to-AAA work, no MK-059 font sizes.** Still M6's probe to answer.
- **No keyboard-model changes.** MK-023 is `done/`.
- `update` stays a pure reducer.

## 5. Recorded, not yours this round — D-28

Eleven `HistoryEntry` variants announce the wrong action when undone: ten
share `UndoRuleMoved` (`RulePrototype`, `TracePrototype`, `HeaderAdd`,
`HeaderUpdate`, `HeaderRemove`, `HeadersClear`, `BodyAdd`, `BodyUpdate`,
`BodyRemove`, `BodiesClear`) and `Respond` announces `UndoUrlPathEdited` —
*"URL path changed"* for a response edit, which round 1's audit missed
because it was looking under the other key. Counted from source: round 1's
request said nine, its evidence said eight, and both lists held ten.

## 6. Required tests

**One test, derived from the style's users, not from the roles we touched.**
Round 1's test passed while the surface failed, which is the same shape as
the defect itself. The new assertion must:

- enumerate **every** `banner_style` site, and for each, **every** colour
  drawn on it — including button label colours, which a container's
  `text_color` does not reach;
- assert AA (4.5) for text and **3:1 for the cursor ring**, in **all four**
  presets;
- fail if a new `banner_style` user appears without being added to the list —
  the same shape as `app/control_census.rs`'s own staleness alarm, which
  exists for this exact class of omission.

If a literal enumeration of "every colour drawn" is not expressible against
this codebase's widgets, say so and propose the narrowest guard you can
defend — do not ship a test that passes by being vague.

## 7. Required evidence

All four presets at a measured 1280×720, using round 1's own capture
discipline (which worked — the hash check caught the input stalls instead of
mistaking them for a freeze):

1. The error banner with **both buttons legible**, all four presets. The two
   dark presets are the frames that matter.
2. The rule editor's validation warning, legible, in at least Light and Dark.
3. **The keyboard cursor on one of the banner's controls, in Dark** — the
   frame no capture has taken yet, and the one that proves F2 is fixed.
4. **HC Dark's success notice**, which round 1 reported as its one gap.
   You are re-capturing anyway; close it rather than carrying it.

Record backend, renderer, build SHA, and the **measured** window size.

## 8. Acceptance criteria

1. Every colour on every `banner_style` surface meets AA, measured, all four
   presets — buttons included.
2. The cursor ring meets 3:1 on that surface, or the controls carry their own
   treatment.
3. The test is derived from the style's users and goes stale loudly.
4. Evidence per §7, including the two frames round 1 never took.
5. Gate green, exit code read directly; `update` still pure; MK-023 untouched.

**On acceptance D-21 closes.**

## 9. Known risks

- **"One guaranteed foreground" may flatten the visual hierarchy.** Size,
  weight and position are the tools left. If a button genuinely needs to read
  as secondary and cannot, report the tension rather than inventing an
  unguaranteed colour.
- **The ring's 3:1 may not be reachable with the primary colour on amber.**
  That is a real possibility; a per-surface cursor treatment is the fallback,
  and it is a design change — propose it, do not assume it.

## 10. Required review-request content

Per workflow §9.2: summary; the measured after-numbers for every colour on
every `banner_style` site, all four presets; the cursor-ring treatment; the
test and what makes it go stale; the evidence with measured geometry; gate
status; review focus.

Commit, and **do not push without authorization**. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
