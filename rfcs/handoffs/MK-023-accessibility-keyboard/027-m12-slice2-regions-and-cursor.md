# Dev-team task 027 — M12 slice 2: the region and cursor model

**Issued:** 2026-10-31
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** [MK-023](../../done/MK-023-accessibility-keyboard.md) § *External design — the keyboard interaction model*, accepted 2026-10-31
**Cycle:** C2, implementation slice 2 of 6 — **the foundation the remaining four build on**

---

## 1. Purpose

Build the region-and-cursor model the accepted design specifies, and prove it on
two surfaces: the **tab bar**, which has no keyboard route today, and the **mode
picker**, which already has one.

**The mode picker is in scope deliberately.** M9 built it, task 022 captured it,
and its behaviour is known-good. Re-expressing it through the general model is a
refactor with a target: if the model cannot reproduce M9's behaviour exactly,
that is a design problem surfaced in the cheapest possible place rather than on
the fifth surface. `mode_picker_selected` should not survive this slice as a
special case.

## 2. What the design fixes, and what is yours

**Fixed by MK-023 and not open for reinterpretation:** Tab and Shift+Tab move
between regions; ArrowUp/ArrowDown move within one; Enter activates the cursor's
item; Escape is unchanged; the cursor stores **identity** where a list can change
and an index where it is fixed and short; the indicator is the 2 px primary
border `card_selected_style` already draws.

**Yours:** the internal design — the types, where the state lives, how a region
declares its items and its activation message. The `accelerator.rs` and
`palette_commands.rs` tables are the codebase's own precedent for "one source
both `view` and `update` read", and D-9 is the lesson about what happens when a
table and its readers can disagree.

## 3. Two user-visible questions the design settles here

These are external design, so they are mine to answer rather than yours to
invent.

### 3.1 Entering a region puts the cursor on the active item, not the first

Tabbing into the tab bar places the cursor on the **currently active tab**, not
on Routes. Tabbing into any region whose items include a current selection starts
there. Starting at the first item would make Tab a destructive-feeling act — the
user would see the cursor jump away from where they are.

Where a region has no current item, the cursor starts at the first.

### 3.2 "Active" and "has the cursor" are different states and may both show

A tab is **active** because it is the tab being displayed. A tab **has the
cursor** because the keyboard is pointing at it. A user can move the cursor along
the tab bar without activating anything until they press Enter, so both states
must be visible at once and must not be confusable.

- The active tab keeps the styling it has today, unchanged.
- The cursor is the 2 px primary border, drawn **only while its region holds
  focus**.
- When the cursor is on the active tab, both apply to the same item. Make sure
  that reads as one coherent state rather than a doubled one — if it does not,
  say so and show it; that is a finding, not something to quietly tune.

**No other region shows a cursor** while it does not hold focus. Two visible
cursors at once would be exactly the confusion the philosophy forbids.

## 4. Scope

- The **region/cursor mechanism**, with Tab and Shift+Tab.
- Applied to the **tab bar** and the **mode picker**.
- `mode_picker_selected` removed in favour of the general model.
- The region order for these two surfaces only, declared through a mechanism that
  extends to later slices without rework.

## 5. Non-change scope

- **No other surface becomes navigable.** Welcome and Dashboard are slice 3; the
  sidebar tree is slice 4; the segmented regions and the trace list are slice 5.
  Declaring a region for them now, unused, is not "getting ahead" — it is
  shipping untested surface.
- **Do not move theme or audience mode to the palette** (slice 6).
- **Do not delete `left_rail.rs`** (slice 6, D-16).
- **No `.on_submit`,** ever. Slice 1's tree-walking guard will catch it.
- **No change to `EscapePressed`** or to slice 1's dialog defaults.
- `update` **stays a pure reducer**. M9 kept it so across 486 call sites with the
  one genuine `Task` isolated in `update_and_dispatch`; focus commands belong
  there, not in `update`.

## 6. Required tests

Reducer tests, per-module `tests.rs` layout. **The four edge cases, for every
region this slice creates:**

1. a list shortening under the cursor **clamps** rather than leaving it out of
   range;
2. an empty region has **no cursor** and `Enter` is a no-op;
3. entering a region sets the cursor per §3.1 — on the active item where one
   exists;
4. whether the cursor can move outside the visible scroll — **state it plainly
   if it can** rather than claiming it scrolls.

Plus:

- Tab and Shift+Tab cycle the declared order and wrap;
- **the mode picker behaves exactly as before** — the same keys produce the same
  messages. This is the refactor's safety net and should be written against M9's
  existing behaviour before the old field is removed;
- only the focused region reports a cursor (§3.2).

## 7. Required evidence

A keyboard-driven run showing: launch → cursor on the mode picker → Enter →
Welcome; then, in a workspace, Tab to the tab bar and arrow between tabs with
Enter switching.

**Expect a mixed backend and say so.** Reaching a workspace still needs one
pointer click until slice 3, so the tab-bar half runs on X11 with `xdotool`,
while `wtype` on Wayland covers what is reachable without a click. This tension
is recorded in task 026's review and **expires when slice 3 lands**.

Record backend, renderer, build SHA and window size. Use the nested-compositor
recipe with the coordinate conversion and `hotkey-overlay { skip-at-startup; }`.
**No owner clicks.**

## 8. Acceptance criteria

1. Tab/Shift+Tab move between regions; arrows within; Enter activates.
2. The tab bar is fully keyboard-operable; the mode picker is unchanged in
   behaviour and no longer a special case.
3. §3.1 and §3.2 implemented, with §3.2's doubled-state question answered from
   the capture.
4. The four edge cases tested per region.
5. `update` still pure; gate green on both toolchains.
6. Evidence captured with configuration recorded.

## 9. Known risks

- **The model is the deliverable, not the two surfaces.** Four slices depend on
  it. If something feels like it needs a special case already, that is a design
  finding — report it rather than adding the case.
- **Regressing the mode picker** would be invisible to anything but the test
  §6 asks for first.
- **Over-generalising.** The philosophy warns against *"vague definitions
  resulting from over-pursuing abstraction"* as much as against rigidity. Two
  regions now, four slices later — build for what the design names, not for
  every region a GUI could have.

## 10. Required review-request content

Per workflow §9.2: summary; the internal design and why; how a region declares
items and activation; the §3.2 answer with its frame; the four edge cases per
region; the mode-picker equivalence test; gate status; evidence paths with
backend, renderer and SHA; review focus.

Commit, and **do not push without authorization**. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
