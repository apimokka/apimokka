# Dev-team task 032 — M12 slice 7: the final slice

**Issued:** 2026-11-03
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** [MK-023](../../proposed/MK-023-accessibility-keyboard.md) § *External design*
**Cycle:** C2, implementation slice **7 of 7**

---

## 1. Purpose

Close the four defects the slices surfaced, add the palette routes, and produce
**M12's own acceptance evidence**: all six primary scenarios driven by keyboard
alone.

On acceptance, **D-12 closes** and **MK-023 moves to `done/` with its mechanism
executed** — which is the point of the whole cycle, after three RFCs were
returned from `done/` for the opposite.

## 2. D-16 — delete the left rail

`shell/left_rail.rs` implements MK-027's left rail and **nothing renders it**;
the tab bar does that job. Delete the file and its `pub mod left_rail;` line in
`shell/mod.rs`.

**It orphans one thing, and clippy will tell you:** `theme::rail_selected_style`
(`theme.rs:471`) has exactly one caller, `left_rail.rs:30`. **Delete it too.**
Do not re-add an `#[allow(dead_code)]` — a stale one on that very function was
removed in task 021 for good reason.

MK-027 already carries the dated correction explaining why the rail is
unrendered; this slice makes the tree match it.

## 3. D-18 — a click should enter its region, and the scope is narrower than it looks

The divergence is between **iced's native focus** and **the region model's
cursor**. Only `text_input` and `text_editor` can take native focus — a `button`
cannot — so **a click can only cause the divergence on a text field.** Wrapping
every region would be invasive and most of it would be wrapping controls that
cannot diverge.

**So: wrap the text-field regions only.** `mouse_area(...).on_press(…)`
(`iced_widget::mouse_area`) dispatches a message on click; a new message that
sets `focused_region` to that field's region, and the cursor with it, is enough.

**If extending it to every region is cheap, say so and say what it would buy** —
the cursor following whatever the user last touched is arguably better, but it is
not what D-18 is, and it is not required here.

## 4. D-19 — `Enter` must do something, and the handle must stop lying

Two halves; **either alone is a partial fix**, which the defect entry already
records.

**`Enter` cycles forward** on a value-cycler region, so the natural key for an
apparently-openable control is not dead. `activation_message` currently returns
`None` for all four cyclers (three operators plus Settings' locale).

**The handle stops promising a popup while the cursor is on it.**
`pick_list::handle(Handle::Static(Icon { font, code_point, size, line_height,
shaping }))` is the mechanism — verified. A glyph suggesting up/down cycling
rather than opening is what is wanted; something like `⇅` needs
`text::Shaping::Advanced` since it is not ASCII. **Pick the glyph and justify
it in one line**; if the font cannot render your first choice, say which you
fell back to rather than leaving a tofu box in the UI.

Applies to **all four** cyclers, not only the editor's three.

## 5. D-20 — scroll-into-view for six scrollables

Only three are wired — the palette results, the Dashboard list and the sidebar —
and `app.rs` holds exactly three `snap_to` sites. Wire the rest:

| Scrollable | Note |
|---|---|
| `rule_editor.rs:295`, `:363` | two separate scrollables; **a region belongs to one of them**, so the dispatch must pick the right one |
| `trace.rs:151`, `:256` | the event list is the sharpest case — it grows while the server runs |
| `settings.rs:255` | |
| `wizard.rs:270` | |

Use the same relative-offset approximation already used three times. **D-5's
recorded limits still apply and should be restated, not quietly dropped**: iced
0.14 exposes no scroll-into-view for a specific child, so this maps a position to
an offset and is approximate. Say so again rather than letting the fourth
implementation imply it became exact.

## 6. The palette commands

Add rows for **theme**, **audience mode** and **locale** — an *additional* route,
not a replacement: all three remain regions in Settings, per the 2026-11-03
correction to the design.

`Key::PaletteCmdToggleTheme` and `Key::PaletteCmdLocale` already exist, and
`PaletteCmdLocale` has been **orphaned since D-10**; this finally uses it.
Audience mode needs a new key in both locales.

Mind **D-9's lesson**: a palette row's label must track state where the action is
a toggle. "Toggle theme" is state-independent and fine; decide deliberately
whether the audience-mode and locale rows should name the target
("Switch to Expert") or the action, and say which and why.

## 7. Non-change scope

- **No new region kinds, no new navigation mechanism.** Everything here is a fix
  or a table entry.
- **No change to pointer behaviour**, except that §3 makes a click additionally
  move the region cursor.
- **No change to MK-027's or MK-023's Status fields** — mine when the evidence is
  accepted.
- `update` **stays a pure reducer**.

## 8. Required tests

- `Enter` on each of the four cyclers advances the value and dispatches the
  dropdown's message.
- Clicking a text field enters its region — reducer-level, via the new message.
- Each newly wired scrollable dispatches a snap for cursor movement within it,
  and the editor's two columns dispatch to the **correct** one.
- The three palette rows dispatch what they claim; labels behave per §6.
- `left_rail` and `rail_selected_style` are gone and nothing references them.

## 9. Required evidence — M12's acceptance, not just this slice's

**All six primary scenarios, keyboard only, Wayland, `wtype`, zero pointer
events**, in as few sessions as the entry states allow:

1. Open a workspace and find the rule serving a given path.
2. Create a workspace from the wizard and add a rule matching a stated request.
3. Edit a rule's conditions and verify with Test Rule.
4. Inspect a trace event and determine why a request did not match.
5. Save and then revert a fallback-file draft.
6. Change theme and locale, and switch audience mode.

**Scenario 5 has not been exercised by any slice's capture.** The fallback-file
editor is a `text_editor`, which is natively focusable, so it should already
work — **but it has never been driven by keyboard end to end, so do not assume
it does.** If it does not, that is a finding for this slice, not a surprise for
M6.

Also capture: a cycler showing the new handle, and one frame proving the cursor
now follows into view on a previously unwired scrollable.

Record backend, renderer, build SHA and window size throughout.

## 10. Acceptance criteria

1. D-16, D-18, D-19 and D-20 closed as specified.
2. The three palette rows present and correct.
3. **All six scenarios captured keyboard-only with zero pointer events**,
   scenario 5 included.
4. Gate green on both toolchains; `update` still pure.
5. Nothing references the deleted rail.

## 11. Known risks

- **Scenario 5 is unproven.** §9.
- **The handle glyph may not render** in the chosen font. Check it in the capture
  rather than trusting the code point.
- **The editor's two scrollables** are the one place D-20's fix can be wired to
  the wrong target and still look plausible.
- **Four defects in one slice** is a wide diff. If any one of them turns out to
  need design rather than application, **stop and report it** — the precedent is
  task 024, and it has paid twice.

## 12. Required review-request content

Per workflow §9.2: summary; each defect and how it was closed; the §3 scope
decision and whether extending is cheap; the §4 glyph and its justification; the
§5 column-targeting; the §6 label decisions; scenario 5's result; gate status;
the six-scenario evidence with configuration; review focus.

Commit, and **do not push without authorization**. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
