# Dev-team task 042 — D-36: text-field focus is carried by colour alone

**Issued:** 2026-11-08
**Issued by:** programme architect
**Authorized by:** project owner, 2026-11-09
**Design:** [MK-022](../../done/MK-022-visual-design-system.md) § colour roles;
[MK-028](../../done/MK-028-routes-workbench.md) § the non-colour rule;
[MK-023](../../done/MK-023-accessibility-keyboard.md) § visible focus
**Found by:** task 037 §9.1 — combination 2, the mandatory combination whose
stated purpose is *"focus indication against high-contrast borders"*. The dev
team measured the two high-contrast presets; the review extended it to all four
**Review:** `.git-exclude/reviewed/2026-11-08-task037-s9-1-review.md` §2 and §4

---

## 1. The defect

Text inputs are the one widget iced 0.14 gives native focus to, so they do not
take the region cursor — their focus indication is the field's own border.
`theme::text_input_style`'s `Focused` arm sets **only the placeholder colour**,
leaving iced's default focused border, while the `Active` arm overrides the
resting border to `border_token`. The two were never compared with each other:

| Preset | resting border | focused border | focused vs resting | width change |
|---|---|---|---:|---:|
| light | (137, 140, 143) | (4, 0, 180) | **3.74** | 0.0 |
| dark | (105, 113, 125) | (101, 199, 255) | **2.63** | 0.0 |
| hc_light | (0, 0, 0) | (13, 0, 140) | **1.40** | 0.0 |
| hc_dark | (255, 255, 255) | (188, 245, 255) | **1.19** | 0.0 |

**Focus is a hue change and nothing else, in every preset, and it clears WCAG
1.4.11's 3:1 for state identification in exactly one of the four.** Visible in
combination 2's own frames: `c2x-hclight-07-header-value.png` and
`c2x-hcdark-08-body-value.png`.

**S2** by MK-056 §5 — a *degrading* accessibility failure. The caret and the
typed text still answer "where am I", so completion is not prevented, which is
that clause's test for S1. It does **not** block M6 booking.

**It should land before the first session.** One match arm, and the sessions
put high contrast in front of participants — the preparation gate exists so
that *"no participant's time is spent rediscovering a defect we already
knew"*. Recruitment runs beside this, not behind it.

## 2. The fix

**Give focus a width cue and keep the colour.** In `text_input_style`'s
`Focused` arm set `border.width = 2.0` against the resting `1.0`, in every
preset. That is the move that closed D-30: the state is carried by width —
MK-028's non-colour rule — so the hue difference becomes a secondary cue
rather than the only one, and no new colour has to be measured against four
backgrounds.

- **Do not change the `Disabled` arm.** Its border colour carries a recorded
  measurement (`text_secondary`, 6.53 / 7.10 / 12.91 / 16.50 against `weak`)
  and a reason; leave both.
- **Do not reach for `primary`.** The region cursor owns it, and a text field
  that takes the same 2 px primary ring would read as the region cursor on a
  widget that does not have one.
- If a width cue turns out to be impossible for a reason you find in iced's
  `text_input`, **stop and report with the measurement** rather than
  substituting a colour.

## 3. Also in this visit

**Delete `ux_app_show_region_cursor`'s `soft` branch**
(`scripts/ux/app-nav.sh`). No caller passes `yes` since §9.1 made the check
strict, and its comment still states in the present tense that *"under an HC
theme a Tab can genuinely move focus without producing any visible frame
difference at all"* — that was **D-29**, and D-29 is closed. Remove the branch
and the parameter; the history is in the register and in task 037's reviews.

## 4. Non-change scope

- **No region, cursor or layout change.** `regions.rs`,
  `app/region_cursor.rs`, `rule_editor.rs` and the condition rows stay as task
  041 left them.
- **No other `text_input` role changes** — placeholder, value and selection
  colours are measured where they are.
- `update` stays a pure reducer.

## 5. The guard

Extend `theme/tests.rs` beside the existing cursor guards: for **all four
presets**, assert that the focused text input differs from the resting one by
**either** a border colour clearing 3:1 against the resting border **or** a
border at least 1 px wider. Encode the rule, not the number — it is the same
shape as `selected_cards_are_distinguishable_from_unselected_in_all_presets`,
and like that one it must be **written first and watched fail** on the current
styles (expect three failures: dark 2.63, hc_light 1.40, hc_dark 1.19).

Task 023's D-15 guard measures the focus ring against the *page* and still
holds; leave it, and say in the new test's comment why both exist — a ring can
clear the page and still vanish against the border beside it.

## 6. Required evidence

At the window size combination 2 used, on **wgpu**:

1. a focused text field beside unfocused siblings in **HC Light** and **HC
   Dark** — the header value and one body value, as §9.1's `07`/`08` frames
   did, reached by the same keyboard walk;
2. the same in **Dark**, which this review found also fails the floor and
   which no capture has looked at for focus;
3. **880×700** with the condition rows on screen (§9.1's scroll step), to show
   the 2 px border has not disturbed task 041's two-line layout.

Manifests must pass, as always.

## 7. Acceptance criteria

1. Focused and resting text inputs are distinguishable by the §5 rule in all
   four presets, measured.
2. §5's guard exists and fails on the pre-fix styles.
3. §3's dead branch is gone.
4. Evidence per §6; no layout shift at 880×700.
5. Gate green, exit code read directly; `update` still pure; no region or
   layout change.

**On acceptance D-36 closes.**

## 8. Known risks

- **A 2 px border on a 110 px input may shift the condition rows** at 880×700.
  D-33's two-line shape is fresh; if the border costs a pixel, capture it and
  report rather than trimming either.
- **iced may draw the focused border inside the widget bounds**, in which case
  the extra pixel eats the text's left padding. Look at it in the frame before
  calling it done.

## 9. Required review-request content

Per workflow §9.2: summary; the measurement for all four presets before and
after; the guard and what makes it fail; the frames; what §3 removed; gate
status; review focus.

Commit. Under the reconciled push rule (ROADMAP, *Release and delivery
policy*, amended 2026-11-08) you may push work you authored once it is
reviewed and approved. Check `git log origin/mockup-v0-01..HEAD` first and say
what a push would carry. Commit messages: digest plus optional body within 400
characters.
