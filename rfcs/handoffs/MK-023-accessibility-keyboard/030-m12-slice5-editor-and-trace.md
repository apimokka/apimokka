# Dev-team task 030 — M12 slice 5: the rule editor and the trace screen

**Issued:** 2026-11-01
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** [MK-023](../../proposed/MK-023-accessibility-keyboard.md) § *External design — the keyboard interaction model*
**Cycle:** C2, implementation slice 5 of 6 — **the largest remaining**

---

## 1. Purpose and a scope correction

Make the rule editor and the trace screen navigable, completing scenarios 2, 3
and 4.

**The design called this slice "the segmented regions and the trace list". That
under-scoped it, and the correction is mine.** Counted at source,
`rule_editor.rs` alone holds **11 `text_input`, 3 `pick_list` and 10 buttons**.
Leaving the fields and dropdowns out would mean a keyboard user could reach the
method buttons but not the URL path they apply to, and **scenario 3 — "edit a
rule's conditions" — would not be completable**, which is M12's own acceptance
criterion.

So this slice covers the editor and the trace screen whole.

## 2. How a form is navigated — the decision, and why

Three mechanisms were available. The model already expresses the right one.

**Single controls are one-item regions; groups are multi-item regions.**

- A lone `text_input`, button or dropdown is a **region of one item** — in other
  words a plain Tab stop, which is how every form a user has ever met behaves.
  Arrows do nothing there, and that is correct.
- A segmented group — the **6 method buttons**, the **2 respond-mode tabs**, the
  Test Rule dialog's **5 method buttons** — is a **multi-item region**: Tab to the
  group, arrows within, `Enter` to activate. Also conventional.

**Tab order follows reading order**, as slice 3 established for Dashboard.

**The alternative I rejected:** a two-level scheme where Tab moves between *zones*
(sidebar, editor, drawer) and a second key moves within. It would make the
workspace's Tab chain shorter, and it would give Tab two meanings depending on
depth — the confusion the philosophy forbids, and close to snora's `next_zone`,
which this programme already assessed and declined. One Tab semantics, a longer
chain, and the command palette for long jumps.

## 3. `pick_list` — the one control the toolkit cannot give us

**A `pick_list` cannot be opened or focused from the keyboard in iced 0.14.**
Verified at source: it implements no `Focusable`, `state.is_open` is private and
set only by pointer interaction, and `on_open` merely *reports* an opening. There
is no API to open one.

**So a `pick_list` is a one-item region whose arrows change the value in place.**
`ArrowUp`/`ArrowDown` move through the option list and dispatch the same message
the dropdown's selection would — no popup appears. This is a standard accessible
select pattern and it is the only behaviour expressible here.

The three are `RuleSetUrlPathOp` (`rule_editor.rs:406`), the header operator
(`:475`) and the body operator (`:545`). **These are the only way to set a
matching operator**, so without this, scenario 3 fails.

**State the consequence plainly in the submission:** a pointer user opens a
dropdown and a keyboard user cycles a value. Different interaction, same outcome,
and the difference is the toolkit's, not a choice. If that reads badly in the
capture, say so — it is a finding, not something to work around.

## 4. The trace screen

- **Pause** and **Clear** — one-item regions.
- **The event list** — an identity-cursor region, keyed by event id, exactly the
  machinery slices 3 and 4 already share. The list grows while the server runs, so
  identity is required, not optional.
- **The match-detail actions** — `JumpToRule`, `JumpToFile`, `AddRuleForPath` —
  appear conditionally on the selected event's outcome. One-item regions that
  exist only when rendered; `order_for` must derive them, never assume them.
- The filter field is a one-item text-field region.

## 5. Explicitly excluded — do not make these regions

**Theme (4 buttons) and audience mode (2 buttons) in Settings.** The design moves
both to the command palette in **slice 6**. Making them segmented regions now
would be work slice 6 deletes. The audit counted them among the nine surfaces;
they are two of the five that leave the keyboard model by leaving the screen.

Also excluded: the palette moves and deleting `left_rail.rs` (slice 6), and
**D-18** (slice 6), though slice 4's focus release already bounds it.

## 6. Non-change scope

- **No change to pointer behaviour anywhere**, including the `pick_list`s.
- **No new widget, no replacing a `pick_list` with buttons.** §3's answer is the
  one that needs no product change; proposing one is a finding to report, not a
  thing to take.
- **No `.on_submit`** on any of the 11 fields. Slice 1's tree-walking guard will
  catch it, and D-6 is why.
- `update` **stays a pure reducer**; focus, unfocus and scroll are `Task`s for
  `update_and_dispatch`.

## 7. If the pattern does not fit, stop and say so

The counts above are mine, from `grep`. **If the editor's real inventory does not
decompose cleanly into one-item and group regions** — if some control needs a
third kind, or the Tab chain turns out unusable in the capture rather than merely
long — **report before building around it.** Task 024's halt is the precedent and
it saved a wasted slice; the same gate applies here, narrower.

## 8. Required tests

The four edge cases for every region kind this slice adds, **live where they are
live and N/A with a reason where they are not** — the discipline held in both
directions across slices 2–4.

- The trace list's identity cursor survives events arriving while it is focused.
- Conditional match-detail regions appear and disappear from `order_for` with the
  selected event's outcome, and the cursor does not survive into a region that no
  longer exists.
- A `pick_list` region cycles its value and dispatches the same message the
  dropdown does — assert the message, not the widget.
- Tab order matches reading order on both screens.

## 9. Required evidence

A keyboard-only run, **Wayland, `wtype`, zero pointer events**, completing
**scenario 3 end to end**: open a workspace, select a rule, change its URL-path
operator via the `pick_list` region, change the method, and run Test Rule.

Plus a trace-screen run: select an event, read its match detail, and use one
outcome action.

**Include one frame of a `pick_list` region mid-cycle**, since §3's behaviour is
the one a reviewer cannot infer from code.

Record backend, renderer, build SHA, window size; nested-compositor recipe with
`hotkey-overlay { skip-at-startup; }`.

## 10. Acceptance criteria

1. The rule editor and trace screen navigable per §2–§4.
2. Scenario 3 completable by keyboard alone, captured.
3. `pick_list` regions cycle in place, with the consequence stated.
4. Theme and audience mode untouched (§5).
5. Edge cases per §8; `update` still pure; gate green.
6. The §9 runs captured, zero pointer events.

## 11. Known risks

- **The Tab chain is long.** That is the accepted cost of one Tab semantics. If
  the capture shows it is unusable rather than merely long, that is a §7 report.
- **The `pick_list` compromise is visible to users.** Expected; record how it
  reads.
- **Conditional regions** are the first regions that come and go with state.
  `order_for` must derive them; a stale cursor into a vanished region is the
  failure mode, and `valid_focused_region` should already prevent it — confirm it
  does rather than assuming.

## 12. Required review-request content

Per workflow §9.2: summary; the region decomposition and any control that did not
fit; §3's `pick_list` behaviour and how it reads; the conditional-region handling;
the four edge cases; Tab order; gate status; both runs with configuration and the
`pick_list` frame; review focus.

Commit, and **do not push without authorization**. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
