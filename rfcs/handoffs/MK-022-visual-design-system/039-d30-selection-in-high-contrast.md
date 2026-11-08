# Dev-team task 039 — D-30: selection is the invisible state in high contrast

**Issued:** 2026-11-08
**Issued by:** programme architect
**Authorized by:** project owner, 2026-11-08
**Design:** [MK-022](../../done/MK-022-visual-design-system.md) § colour roles;
[MK-028](../../done/MK-028-routes-workbench.md) § the non-colour rule
**Found by:** the task 038 review, by its own measurement — the dev team
reported the qualitative half (*"in HC Dark the selection tint is subtle
against black"*), the review measured it
**Review:** `.git-exclude/reviewed/2026-11-08-task038-d29-cursor-ring-review.md` §4–§5

---

## 1. The defect

Task 038 fixed the cursor. In the two high-contrast presets the **selected**
state is now the one that is almost not drawn — and unlike D-29 this is not an
oversight in a borrowed value, it is what the styles say.

| Preset | unselected `card_style` | selected `card_selected_style` | selected vs unselected fill | border width |
|---|---|---|---:|---|
| light | `surface` + 1.0px token | `surface` + **2.0px primary** | 1.00:1 | 1.0 → **2.0** |
| dark | `surface` + 1.0px token | `surface` + **2.0px primary** | 1.00:1 | 1.0 → **2.0** |
| hc_light | white + **1.5px token** | 10% primary wash, **no border** | **1.60:1** | 1.5 → **0.0** |
| hc_dark | near-black + **1.5px token** | 18% primary wash, **no border** | **1.35:1** | 1.5 → **0.0** |

`card_parent_selected_style` has the same shape: **1.47:1** (hc_light) and
**1.28:1** (hc_dark), border **1.5 → 0.0**.

**In the rendered frames, not only at source.** Scanning task 038's own
`hcdark-4-editor-header.png` down x=300: the selected row is a flat
**(30, 37, 48)** from y240 to y274; unselected rows are **(3, 3, 3)**; the
unselected row below opens with a **(193,193,193) → (255,255,255)** edge at
y289–290 — a white border at ~21:1. The selected row has **no edge at all**.
`hclight-4` gives **(200,205,218)** selected against **(255,255,255)**
unselected. There is **no left accent strip** on the row in either preset
(x=0…60 scanned).

So in high contrast **the selected row is the row whose border is missing**:
a 1.35–1.60:1 fill difference, below WCAG 1.4.11's 3:1 for identifying a
component's state, plus the *removal* of the strongest edge on screen — the
inverse of emphasis. In Light and Dark the same information is carried
properly: the border doubles in width *and* takes the primary colour at
6.19–6.70:1, which is MK-028's non-colour rule satisfied.

**S2**, per MK-056 §5: a *degrading* accessibility failure. Completion still
survives by two channels — the cursor is visible after task 038, and the rule
editor shows the selected rule's contents — so it is not S1, which that
section reserves for a failure that **prevents** completion. **It escalates to
S1 if an M6 participant on a high-contrast preset cannot identify the selected
row**, and it does not block M6 booking.

## 2. The fix is a decision, and one option is forbidden

**Do not give the high-contrast selected card the 2px primary border that
Light and Dark use.** It is the obvious move and it is the wrong one: the
cursor is a 2px primary border, so selection would become pixel-identical to
the cursor. Two states that look alike is the defect task 038 was told to
avoid creating, and it would undo that task's reasoning.

Three states must stay distinguishable on the same row — **unselected**,
**selected**, **selected and cursored** — and the cursor already owns primary.

**Option A — width in the token colour (recommended).** Selected keeps its
tint and takes a **3px border in the same token colour** `card_style` already
uses at 1.5px. Both read at ~21:1 against their own fills, so the cue is
**width**, which is MK-028's non-colour rule, and the colour channel stays
free for the cursor. Measure the token border against both fills in both HC
presets before shipping.

**Option B — a left accent strip.** `card_selected_style`'s own doc comment
says the caller draws one. Cheaper visually, but it must be added at every
selected-row call site rather than in one style function, and if it is drawn
in primary it reintroduces the collision Option A avoids.

**Option C — raise the wash until the fill difference clears 3:1.** Measure
it, and expect it to fail: the wash carries muted text (the chevron and the
count), and D-13 fixed those numbers at AA by *reducing* exactly this kind of
tint. If the alpha needed for 3:1 against the page drops muted text below
4.5:1, report that and take Option A.

**If you prefer a different option, propose it with the measurement.** What is
not negotiable: three distinguishable states, the cursor keeps primary, and
muted text on any new surface stays ≥ 4.5:1 in all four presets.

## 3. The guards

**3.1 — a selection-state guard, for all four presets.** Encode the rule, not
a magic number: for `card_selected_style` and `card_parent_selected_style`
against `card_style`, assert that **either** the composited fills differ by
≥ 3:1 **or** the border is wider by ≥ 1.0px with its colour ≥ 3:1 against its
own fill. That is what makes Light and Dark pass today and both HC presets
fail, so write it before the fix and watch it fail.

**3.2 — the cursor guard gains its fourth ground, and this one is mine.**
`both_cursor_styles_paint_a_visible_opaque_ring_in_all_presets` measures
`cursor_style` against page, `surface` and `surface_raised` — the three
grounds **task 038's handoff named**. The application draws it on a fourth:
**`chip_style`'s fill**, in the undo and notice banners
(`shell/view.rs:298`, `:321`, `:352`). That is the ground-selection error D-21
was made of. **Already measured, and it passes: 6.19 / 6.24 / 6.48 / 7.63.**
Add the ground so the guard covers it; nothing else about that test changes.

## 4. Non-change scope

- **The cursor is not yours this time.** `cursor_style` and
  `banner_cursor_style` keep their appearance exactly as task 038 left it; the
  only edit to the cursor is the extra ground in §3.2's test.
- **Light and Dark do not change.** Their selection already satisfies the
  rule; a change there would invalidate cycle C2's acceptance captures.
- **No region, cursor, or `activation_message` changes**; `regions.rs` and
  `app/region_cursor.rs` stay untouched. `update` stays a pure reducer.
- **Both selected styles are in scope** — `card_selected_style` and
  `card_parent_selected_style`. They have the same defect and share a surface;
  fixing one and leaving the other is how D-25 came back a second time.

## 5. Required evidence

Frames at the geometry `scripts/ux/`'s README prescribes, in **both**
high-contrast presets, each showing **three rows in one frame** so the states
are compared where a user compares them:

1. an unselected row, a selected row, and a selected **and cursored** row,
   together in the sidebar;
2. the same for a rule-set header row (`card_parent_selected_style`);
3. one Light frame of the same screen, for comparison.

Say in `RESULTS.md` which row is which, by position — a reader should not have
to infer it.

## 6. Acceptance criteria

1. In all four presets, a selected card is distinguishable from an unselected
   one by the §3.1 rule, measured.
2. Selection and the keyboard cursor remain distinguishable from each other,
   and from an unselected row, in both high-contrast presets — shown in one
   frame, not argued.
3. Muted text on any new or altered surface stays ≥ 4.5:1 in all four presets
   (the existing D-13 guard already asserts this — do not weaken it).
4. §3.1 and §3.2's guards exist, and §3.1 fails on the pre-fix styles.
5. Gate green, exit code read directly; `update` still pure; Light and Dark
   unchanged.

**On acceptance D-30 closes.**

## 7. Known risks

- **A 3px border on a tight row can cost a pixel of layout** (`snap: true`).
  Task 037's 880×700 re-run is where that shows; if you see it at 1280 too,
  report it rather than trimming the border to fit.
- **The tint plus a token border may read as two competing edges** in
  `hc_light`, where the page is white and the wash is only 10%. If it does,
  say so with a frame.

## 8. Required review-request content

Per workflow §9.2: summary; the option taken and why, with its measurement
against all four presets; the before/after for both selected styles; the two
guards and what makes each fail; the frames; gate status; review focus.

Commit. **The push rule was reconciled on 2026-11-08** (ROADMAP,
*Release and delivery policy*): you may push **work you authored that has been
reviewed and approved** — so commit, request the review, and push once it is
accepted; an unreviewed or rejected change does not reach the remote. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
