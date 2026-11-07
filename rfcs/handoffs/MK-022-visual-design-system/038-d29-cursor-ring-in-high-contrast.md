# Dev-team task 038 — D-29: the keyboard cursor paints nothing in high contrast

**Issued:** 2026-11-07
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** [MK-022](../../done/MK-022-visual-design-system.md) § colour roles;
[MK-023](../../done/MK-023-accessibility-keyboard.md) § *What the tests must
validate* — *"every region's indicator is rendered when the cursor is in it"*
**Found by:** task 037, combination 2 — the mandatory combination that exists
to check focus indication against high-contrast borders
**Review:** `.git-exclude/reviewed/2026-11-07-task037-l2-mandatory-combinations-review.md`

---

## 1. The defect

`theme::cursor_style` is the keyboard cursor's appearance for every region
in the application that is not a text field. It borrows **only the border**
from `card_selected_style`:

```rust
pub fn cursor_style(t: &Theme) -> container::Style {
    container::Style {
        background: None,                         // discards HC's alpha wash
        text_color: None,
        border: card_selected_style(t).border,    // HC branch: radius only
        shadow: Shadow::default(),                // discards HC's shadow
        snap: true,
    }
}
```

`card_selected_style`'s high-contrast branch carries **selection** by a
10–18% alpha tint and a shadow, and deliberately leaves its border at
`..Default::default()`. `cursor_style` borrows the one part that carries
nothing and throws away the two that do. Measured:

| Preset | Ring width | Ring colour alpha | Background | Shadow |
|---|---:|---:|---|---|
| Light | 2.0 | 1.00 | — | — |
| Dark | 2.0 | 1.00 | — | — |
| **HC Light** | **0.0** | **0.00** | none | none |
| **HC Dark** | **0.0** | **0.00** | none | none |

**Nothing is painted.** In both high-contrast presets a keyboard user has no
indication of where the cursor is, on the top bar, the tab bar, the sidebar,
the rule editor, rule-set config, the dashboard, welcome, settings, Test
Rule, the path assistant, the drawer and the notice banners — every
`cursor_style` caller, which is all of them. The only survivors are text
fields' native focus rings and the error banner's three controls, which
escape because `banner_cursor_style` (D-21, task 036 round 2) sets its own
explicit border.

**S1**, and it blocks M6's sessions: MK-056 decision 7 makes high contrast
with keyboard-only input a mandatory combination, §5 grades an accessibility
failure on a supported configuration S1, and §8 forbids deferring one.

## 2. The fix, and the measurement is already done

**`cursor_style` defines its own border instead of borrowing one.** The
cursor and a selection are different states and should not share an
appearance decision — borrowing is what produced this.

You do not need to find a colour: **the primary colour already clears the
3:1 non-text floor on every surface the ring is drawn on**, with more margin
in the high-contrast presets than in the two where the ring already works:

| Preset | primary vs page | vs `surface` | vs `surface_raised` |
|---|---:|---:|---:|
| Light | 6.70 | 6.19 | 6.70 |
| Dark | 6.78 | 6.24 | 5.63 |
| **HC Light** | **10.25** | **10.25** | **10.25** |
| **HC Dark** | **11.75** | **11.75** | **11.08** |

So a 2px primary border in all four presets is both the smallest change and
the measured one. **Measure it yourself before shipping it** rather than
taking this table — that rule has paid for itself three times now (D-13's
4.53, D-21's 4.10 dimming, and this).

If you find a reason the high-contrast presets want something other than the
same 2px primary ring — a thicker ring, say, which is a legitimate
high-contrast convention — propose it with the measurement. Do not reach for
a tint: a tint is what `card_selected_style` uses for *selection*, and two
states that look alike is a different defect.

## 3. The test that was missing

MK-023's design requires *"every region's indicator is rendered when the
cursor is in it"*. A test asserts that **in the model**. Nothing asserted it
is **visible**, and every acceptance capture in cycle C2 was taken in Light
or Dark — which is how ten reviewed slices and their reviews all missed
this.

Add, in `theme/tests.rs`, for **all four presets**:

- `cursor_style`'s border width is non-zero and its colour is fully opaque;
- its colour clears **3:1** against `page`, `surface` and `surface_raised`
  — the three grounds a cursor-wrapped region sits on;
- the same for `banner_cursor_style`, which already passes, so that the two
  cursor treatments are guarded by one rule rather than one each.

Prefer a single test that walks both cursor styles over two near-copies.

## 4. Non-change scope

- **`card_selected_style` is not yours to change.** Its high-contrast branch
  is a deliberate D-13-era decision about *selection*; this task is about the
  *cursor*. If you believe selection has the same problem, measure it and
  report it — do not fold it in.
- **No region, cursor or `activation_message` changes.** MK-023 is `done/`
  and this is a styling fix; `regions.rs` and `app/region_cursor.rs` should
  not be touched at all.
- `update` stays a pure reducer.

## 5. Required evidence

The frames task 037's combination 2 could not produce, at a measured
1280×800 (its own geometry) in **both** high-contrast presets:

1. The keyboard cursor visible on a **top-bar** control;
2. visible on a **sidebar** row;
3. visible on a control inside the **rule editor**;
4. and one non-HC frame of the same control for comparison, so the two read
   side by side.

Capture discipline and geometry recording per `scripts/ux/`'s own README —
which, as of task 037, is the authority rather than an out-of-tree
`RESULTS.md`.

## 6. Acceptance criteria

1. `cursor_style` paints a visible, opaque indicator in all four presets,
   measured against all three grounds.
2. The test in §3 exists and fails if either cursor style loses its
   visibility.
3. Evidence per §5.
4. Gate green, exit code read directly; `update` still pure; MK-023's model
   untouched.

**On acceptance D-29 closes, and task 037's combination 2 can be re-run** —
that re-run is task 037's, not this task's.

## 7. Known risks

- **The ring may now collide with `card_selected_style`'s own tint** where a
  cursored row is also the selected row — a real state in the sidebar. Look
  at it; if the two read as one, say so with a frame rather than adjusting
  one of them silently.
- **`snap: true` and a 2px border on a tight row** can cost a pixel of
  layout. The existing non-HC presets already carry it, so this is a
  high-contrast-only risk at most; check the sidebar at 880×700, where task
  037 will be capturing anyway.

## 8. Required review-request content

Per workflow §9.2: summary; the measured before/after for both cursor styles
on all three grounds in all four presets; the test and what makes it fail;
the frames; gate status; review focus.

Commit, and **do not push without authorization**. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
