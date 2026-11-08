# Dev-team task 041 — D-33: the condition rows collapse at 880×700

**Issued:** 2026-11-08
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** [MK-028](../../done/MK-028-routes-workbench.md) § the rule editor;
[MK-041](../../done/MK-041-layout-density.md) § density
**Found by:** task 040's row walk — the dev team reported rows 9 and 10 and
asked for the design call rather than choosing silently, which is what task
040 §3 told them to do
**Review:** `.git-exclude/reviewed/2026-11-08-task040-d31-d32-review.md` §3–§6

---

## 1. The defect

`headers_card` (`crates/app/src/screens/routes/rule_editor.rs:589`) builds
`row![name Fixed(110), op Fixed(110), value Fill, ✕]`. At 880×700 the card's
inner width is ~273 logical px, the fixed children and spacings consume
essentially all of it, and **the value field resolves to zero width**. The
frame `walk-rule3-880x1150-expert-en-headers-body.png` shows `content-type` ·
`Contains` · ✕ with nothing between the operator and the remove button — while
the rule's actual value is **`"application/json"`**
(`crates/model/src/mock/workspace_fixtures.rs:89`).

`body_card` (`:672`) has two `Fill` children, path and value, and both
collapse to ~28–31 logical px: the same frame shows `ac`, `ite`, `cr`, `0.(`.

**S1**, by the clause that graded D-31: *"an unreachable control is observed
fact, not opinion — one sighting is sufficient, and S1 applies"* (MK-056 §5).
A pointer user cannot click or read the header value at a **mandatory**
configuration; the keyboard can focus and type into it blind. Primary scenario
3 — *"edit a rule's conditions and verify the change with Test Rule"* — is
exactly this data. **It blocks M6 session booking** (recruitment is
unaffected), and §8 forbids deferring it.

## 2. The fix is decided: option (a), two lines at every width

Task 040 asked for the call and this is it. **Each condition row becomes two
lines, at every width:**

- **headers:** line 1 `[name Fixed(110)][op Fixed(110)]`, line 2
  `[value Fill][✕]`;
- **body:** line 1 `[path Fill][…][op Fixed(120)]`, line 2
  `[value Fill][✕]`.

Why this and not the alternatives, so the reasoning is not re-litigated:

1. **the value keeps `Fill`** — ~230 logical px at 880 and still growing at
   1920. A fixed ~140 px (option (b)) is worse at both ends for a field
   holding JSON paths and header values;
2. **the shape is the same at every width.** Option (b) would change the row's
   structure at an unstated width — implicit responsive behaviour in the row
   that carries typed text. MK-024 is the RFC that decides breakpoints and it
   is unbuilt. (A wrapping group of fixed-width buttons, as in D-31/D-32, is a
   different case: nothing is hidden either way.);
3. **cursor order is preserved** — name, op, value, remove — so `regions.rs`,
   `app/region_cursor.rs` and the control census stay untouched;
4. the cost is vertical space in a column that already scrolls, and it is
   uniform rather than conditional.

`Wrapping` is **not** the mechanism here: it lays children out under
`limits.loose()` (`iced_widget 0.14.2`, `row.rs:412`), so a `Fill` child takes
the whole line and pushes `✕` onto a third. Use two explicit rows in a column.

**If building it reveals something that contradicts the above, stop and say
so** with what you measured, as you did for rows 9 and 10.

## 3. Also in this visit

1. **`test_rule.rs:153`'s method row takes `.wrap()`.** Five methods in a
   non-wrapping row; it fits at 880 only because the dialog is wide, so it is
   the same mechanism one window size away. Preventive, one call, with a
   capture showing no visible difference at 880×700.
2. **D-34 (S3), optional and only if it measures clean:** in Japanese at
   880×700 「インラインテキスト」 wraps inside its own tab — the label plus
   `mode_tab`'s `S4` horizontal padding needs ~160 logical px against ~155
   available. If reducing that padding fixes it without crowding the other
   tabs or disturbing the D-31/D-32 wrap, do it and show the measurement;
   otherwise leave it and say why. **Do not shorten the Japanese string** —
   microcopy is MK-036's.
3. **A bounded row audit.** Every `row` with fixed-width children on the
   surfaces M6's mandatory combinations exercise — `test_rule.rs`,
   `settings.rs`, `wizard.rs`, `shell/top_bar.rs`, `shell/tab_bar.rs`,
   `shell/bottom_drawer.rs` — walked at 880×700 against live frames.
   **Report-only, except where a control is unreachable or a value is
   invisible, which is fixed in this visit.** Bounded deliberately: an
   app-wide sweep would re-open work M6 is waiting on, and these are the
   surfaces a participant actually touches. Say which rows you checked and the
   verdict for each, as you did for the editor's thirteen.

## 4. Non-change scope

- **No region, cursor or `activation_message` change.** The two-line shape
  keeps the existing order; if it cannot, stop and report rather than
  reordering.
- **No theme change**, and no change to the D-31/D-32 groups beyond what the
  new structure requires.
- **No breakpoint**, no `pick_list` substitution, no horizontal scrollable.
- `update` stays a pure reducer.

## 5. Guards

1. **Structural**, beside task 040's two: a header condition row builds its
   four controls and a body condition row its five, for both `show_val`
   states — the `value_irrelevant()` operators (`Exists`, and whatever else
   `HeaderOp`/`BodyOp` mark) replace the value with a `Space`, and that path
   must stay covered.
2. **Captures** (§6) are the layout evidence, as before; no layout engine
   exists in the suite and inventing one is not this task.

## 6. Required evidence

**On `wgpu` — the host was rebooted on 2026-11-08 and the module tree is
present again, so the reason for software frames is gone.** Record the
renderer as measured, as the rig already does.

1. Combination 1 (Japanese, Guided) and combination 4 (English, Expert) at
   **880×700**, with **rule 3** selected so the header and body conditions are
   on screen — the header value legible, both body fields legible;
2. the same editor at **1280×800**, so the wide case is on the record rather
   than assumed;
3. the Test Rule dialog at 880×700 (§3.1);
4. manifests passing for every run.

## 7. Acceptance criteria

1. The header value and both body fields are visible, legible and operable at
   880×700 in both locales, with the mock's own values readable
   (`application/json`, `action`/`create`, `items`/`0.0`).
2. Cursor order unchanged; the control census test still passes untouched.
3. §3's three items done or reported with their measurements.
4. §5's structural guards exist; §6's captures exist with passing manifests.
5. Gate green, exit code read directly; `update` still pure.

**On acceptance D-33 closes and, with it, the last defect blocking M6 session
booking from this repository's side.**

## 8. Known risks

- **Two-line conditions make Expert's WHEN column taller**, pushing more
  below the fold at 700 px. That is a density observation for M6, not a reason
  to avoid the fix — capture it and say so.
- **The `…` path-assistant button and the operator `pick_list` keep their
  natural widths**; if line 1 of a body condition still crowds in Japanese,
  report the measurement rather than shrinking the path field below legibility.

## 9. Required review-request content

Per workflow §9.2: summary; the two-line structure as built; the bounded
audit's verdict per row; what §3's three items produced; the captures and
their manifests; gate status; review focus.

Commit. Under the reconciled push rule (ROADMAP, *Release and delivery
policy*, amended 2026-11-08) you may push work you authored once it is
reviewed and approved — commit, request the review, push on acceptance. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
