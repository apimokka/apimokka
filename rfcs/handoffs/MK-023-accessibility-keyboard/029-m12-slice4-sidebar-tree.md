# Dev-team task 029 — M12 slice 4: the routes sidebar tree

**Issued:** 2026-10-31
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** [MK-023](../../done/MK-023-accessibility-keyboard.md) § *External design — the keyboard interaction model*
**Cycle:** C2, implementation slice 4 of 6 — **the structurally hardest**

---

## 1. Purpose

Make the routes sidebar navigable by keyboard: rule sets, their rules, the
fallback-file and middleware sections, and the add actions. This is scenario 1's
second half and the entry point for scenarios 2 and 3.

## 2. Why this slice is different

Slices 2 and 3 added flat regions. **This one is a tree whose expansion
semantics are not uniform**, and the audit flagged it as having no precedent in
the codebase. Verified at source:

| Section | Expansion state | Semantics |
|---|---|---|
| Rule sets | `app.rule_set_open: Option<RuleSetId>` | **Accordion** — opening one closes the others |
| Fallback files | `app.fallback_section_open: bool` | Independent toggle |
| Middleware scripts | `app.middleware_section_open: bool` | Independent toggle |

So one sidebar carries **three expansion behaviours**. The keyboard model must
accommodate them without inventing a fourth, and the accordion in particular
means expanding one row **changes the visible list above and below the cursor**.

The sidebar is inside a `scrollable` (`sidebar.rs:208`), so the scroll edge case
is live, as in slice 3.

## 3. The flattened visible sequence

Per the design: **do not build a tree walker.** Derive the **visible** rows in
display order — collapsed children simply absent — and move along that list.

Row order follows `left_sidebar`'s own rendering:

1. each rule set header, and when open: its rules, then its `+ Add rule`
2. `+ Add rule set`
3. the Fallback files header, and when open its files
4. the Middleware scripts header, and when open its scripts

**One source, as in slices 2 and 3.** The function that yields the visible rows
must be the one `sidebar.rs` renders from, not a parallel reimplementation — the
D-9 lesson, and here the consequence of drift would be a cursor pointing at a row
that is not on screen.

**The cursor stores identity**, not a position: the list changes whenever a
section expands or collapses. Rows are identified by what already identifies them
(`RuleSetId`, `NodeId`, a file path, a script path) plus their kind.

## 4. Left and Right — and the one place the data model cannot express the design

The design specifies: `ArrowRight` expands, or moves to the first child when
already expanded; `ArrowLeft` collapses, or moves to the parent when already
collapsed.

**Three of the four work directly. Collapsing a rule set does not exist.**
`rule_set_open` is a single `Option` set by `SelectRuleSet`, and there is **no
message that closes a rule set** — clicking another opens it and closes this one;
clicking the open one re-selects it and leaves it open. The two bool sections have
`ToggleFallbackSection`/`ToggleMiddlewareSection` and collapse fine.

**Add a keyboard-only collapse path** — set `rule_set_open = None` without
changing selection — so `ArrowLeft` behaves uniformly across all three section
kinds. **Do not change pointer behaviour**: clicking a rule-set header keeps
doing exactly what it does today. A keyboard affordance the mouse lacks is the
smaller inconsistency; a `ArrowLeft` that collapses two section kinds and not the
third is the one a user would actually feel.

**Record, do not fix:** that the pointer cannot collapse a rule set at all is a
product question predating this work. Report it; it is not this slice's to settle.

**`ArrowRight` on a collapsed rule set activates it** (`SelectRuleSet`, which
expands *and* selects, because the data model fuses them). On an expanded one it
moves to the first child. State plainly in the submission that `Right` and `Enter`
therefore coincide on a collapsed rule-set header, and that this follows from the
existing message rather than a choice you made.

## 5. A generalisation this slice needs — and slice 3 did not

Slice 3 unfocuses the search field by comparing `focused_region ==
Some(DashboardSearch)` before and after the reducer. **That is specific to one
region, and the workspace is full of text fields.**

`ArrowLeft`/`ArrowRight` **are captured** by a focused `text_input` (unlike
`ArrowUp`/`ArrowDown`/`Tab`), and slice 1's gate correctly drops captured events.
So if any rule-editor field holds focus — reachable today by clicking it, which is
**D-18** — the tree's expand and collapse keys silently do nothing.

**Generalise the mechanism:** entering a region that contains no text field must
release any focused field. Derive it from the region's own properties rather than
naming regions, so later slices inherit it. If that is not expressible with the
operations iced gives us, say so plainly rather than special-casing a list of
regions.

## 6. Non-change scope

- **No other surface.** Segmented controls and the trace list are slice 5; palette
  moves and deleting `left_rail.rs` (D-16) are slice 6.
- **No change to pointer behaviour anywhere**, including §4's collapse.
- **D-18 is not this slice's to fix** — slice 6 — though §5's generalisation
  reduces its blast radius.
- **No `.on_submit`.**
- `update` **stays a pure reducer**; focus, unfocus and scroll are `Task`s and
  belong in `update_and_dispatch`.
- **No change to the accordion's own semantics** beyond adding the collapse path.

## 7. Required tests

All four edge cases, live again:

1. **Clamp** — the cursor's row disappearing because a section collapsed leaves a
   valid cursor. **Include the accordion case**: expanding rule set B while the
   cursor sits on a rule inside rule set A.
2. **Empty** — a workspace with no rule sets leaves no cursor and `Enter` is a
   no-op.
3. **Entering resets** per task 027 §3.1.
4. **Scroll visibility**, with the same honesty slice 3 applied.

Plus: the visible sequence matches the rendered order for every combination of
expansion states; `Left`/`Right` per §4, including the rule-set collapse; §5's
unfocus generalisation tested at the reducer level.

## 8. Required evidence

A keyboard-only run on **Wayland with `wtype`, zero pointer events**, as slice 3
established: open a workspace, Tab to the sidebar, expand a rule set, move onto a
rule, collapse it again, and activate something.

**Include one frame proving §5**: a rule-editor field focused, then `ArrowRight`
on the sidebar expanding a rule set. Without the generalisation that keystroke is
swallowed, so the frame is the discriminating evidence.

Record backend, renderer, build SHA and window size; nested-compositor recipe with
`hotkey-overlay { skip-at-startup; }`.

## 9. Acceptance criteria

1. The sidebar is navigable per §3, including all three expansion behaviours.
2. §4's Left/Right implemented, with the rule-set collapse added and the pointer
   path untouched.
3. §5's generalisation implemented, or its impossibility reported.
4. All four edge cases tested as live cases, including the accordion clamp.
5. `update` still pure; gate green on both toolchains.
6. The §8 run captured, zero pointer events, with the §5 frame.

## 10. Known risks

- **The accordion changes rows above the cursor.** Identity cursors handle it;
  the tests in §7.1 are what prove it.
- **A parallel visible-row function** would be the D-9 defect in its worst form —
  a cursor on a row that is not rendered.
- **§5 may be awkward.** It is the right shape; if iced resists, report rather
  than naming regions one by one.
- **Scope pull toward D-18.** It is slice 6's.

## 11. Required review-request content

Per workflow §9.2: summary; the visible-row function and where it is shared from;
how identity is keyed across four row kinds; §4's Left/Right including the
collapse addition and the `Right`/`Enter` coincidence; §5's mechanism; the four
edge cases including the accordion clamp; the pointer-collapse observation; gate
status; the run with its configuration and the §5 frame; review focus.

Commit, and **do not push without authorization**. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
