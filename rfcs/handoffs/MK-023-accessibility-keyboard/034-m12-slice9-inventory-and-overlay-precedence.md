# Dev-team task 034 — M12 slice 9: the inventory, and one overlay precedence

**Issued:** 2026-11-03
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** [MK-023](../../done/MK-023-accessibility-keyboard.md) § *External design*, § *Acceptance*
**Review that produced it:** `.git-exclude/reviewed/2026-11-03-task033-slice8-review.md`
**Cycle:** C2, implementation slice **9 of 9**

---

## 1. Purpose

Slice 8's census did what it was commissioned to do: it found what eight
slices of coverage had not. **Three of its findings are named in MK-023's own
acceptance clause**, which is the checklist this RFC is judged against:

> The audit's inventory is the checklist: **every control it lists must be
> reachable**, or the design must say why it no longer exists.

This slice covers the inventory, and fixes the one defect the census made
visible: the same overlay precedence is written out by hand in three places,
and they have drifted.

On acceptance **D-22 and D-23 close**, **D-24's confirmation half closes**,
and **MK-023 moves to `done/`**.

## 2. First, the one defect — D-22

`shell/view.rs`'s z-order, `regions::order_for`'s override chain, and
`app.rs`'s `EscapePressed` chain each state the overlay precedence
separately. Three reachable disagreements today:

1. **`order_for` puts the drawer above the path assistant**; the view puts
   the sheet below every dialog. With both open, `Tab`/arrows/`Enter` drive
   the hidden drawer — reproduced in review:
   `ORDER WITH BOTH OPEN = [Drawer(Close), Drawer(JumpToDiagnostic(0))]`.
   In SaveDiff mode that list contains `Discard`, which is destructive,
   unconfirmed and not undoable. **S1.**
2. **The workspace menu has no override at all** — while it is open the
   keyboard acts on the screen behind it.
3. **`Escape` dismisses the workspace menu while the palette is above it**
   (`ToggleWorkspaceMenu` closes the palette; `ToggleCommandPalette` does
   not close the menu, so the state is reachable).

**Do not fix this by reordering two `if`s.** Build **one** source:

- `App::topmost_overlay() -> Option<Overlay>` — the overlay actually on top,
  in one list, with the mode picker's unconditional first place included.
- `shell/view.rs` chooses its dialog from it.
- `regions::order_for` takes its override from it.
- `EscapePressed` dismisses from it.
- The two `confirm_dialog.is_none()` gates slice 8 added to `EnterPressed`
  are rewritten in terms of it (their shape was right; their source should
  be this one).

**A test per adjacent pair**, asserting the three readers agree: which one is
topmost, which regions `order_for` yields, and what `Escape` closes. The
pairs that exist today are enough; no need to manufacture combinations the
application cannot reach — but say which you ruled out and why.

## 3. The inventory — D-23

Every surface below holds visible controls no key reaches. Numbers are
controls, not regions; group what the design already groups.

| Surface | Controls | Notes |
|---|---|---|
| `screens/test_rule.rs` | 5 method buttons (a group), `✕`, Path, Headers, Body, Close, **Run** | Named in the RFC's scenario-3 inventory row. Mechanism: the shape slice 8 built for the confirm dialog. `test_rule.open` leaves `blocks_region_navigation`, as `confirm_dialog` did. **Task 026 §4.2's safe default must survive**: `Enter` still runs the dry run from the entry position and from any field whose own activation would otherwise be `None`. State that as a decision in the code, since "a text-field region has no activation" is the rule everywhere else and this dialog is the deliberate exception. |
| `shell/top_bar.rs` | workspace switcher, Save all, Reload, Restart, Start/Stop | The RFC's Routes focus order, entries **one and two**. The palette rows stay; they are a second route to the action, not a route to the control. The RFC already withdrew the palette-instead-of-reach argument once (task 031 §3) — do not re-run it. |
| `screens/workspace_menu.rs` | recent-workspace rows, Open, Create | Only once the menu is open, as an override (§2). Without it the switcher above opens a surface the keyboard cannot use. |
| `screens/dotted_path.rs` | `✕`, JSON input, per-leaf Use rows, Cancel, Insert path | Reachable today from `EditorField::BodyPathAssistant`; a keyboard user can open it and can only `Escape` out. Same dialog shape as the confirm dialog and Test Rule. |
| `screens/routes/trace_activity.rs` | "View all in Trace", up to 3 "Jump to trace event" | The Routes focus order's **last** entry. |
| `shell/view.rs` `feedback_banner` | error: details toggle / action / dismiss; undo: Undo / dismiss; notice: dismiss | `Ctrl+Z` already reaches Undo; the rest have no route. Mutually exclusive, so one region with a field list derived from which banner renders. **Touch no colour** — D-21 is a separate task. |
| `screens/routes.rs` empty state | "Add a rule" / "Add rule set" | The census missed it; it is the state the Routes tab opens in. |

**`screens/command_palette.rs`'s `✕`** stays uncovered: `Escape` closes it,
the same equivalence every other dialog's `✕` has. Recorded, not built.

**Delete `screens/scripts.rs`.** The census found it reachable from nothing
— a D-16-shaped orphan. This repository's deliverable is an executable
specification; it should not describe a screen the application does not
have. One commit-sized removal, with its `mod` declaration.

## 4. Two corrections from the review

1. **The confirm dialog restores the cursor on close.** Keep the prior
   `(region, cursor)` in `ConfirmDialogState`; restore on both
   `ConfirmCancel` and `ConfirmProceed`. Safe as it stands:
   `valid_focused_region()` re-derives and `reclamp_identity_cursor` runs
   after every message, so a restore that `Proceed` invalidated simply does
   not survive the next keystroke. A dialog is not a screen transition —
   the screen underneath never went anywhere.
2. **D-24, the confirmation half only.** The drawer's `Discard` dispatches
   `Message::DiscardChanges` directly, while `ConfirmAction::DiscardChanges`
   and its two i18n keys exist and are dispatched by nothing. Route it
   through `ConfirmRequest`. **Do not touch the second half** — that
   `discard_all_changes` reverts fallback drafts only, leaving the dirty
   rule sets the same drawer lists — that is a product question with the
   owner.

## 5. Non-change scope

- **No contrast or colour work.** D-21 and D-25 are one separate task on the
  banner surface, awaiting the owner's scope decision.
- **No new region kind.** Dialog-as-region, group, one-item, value cycler
  and the scroll-snap mechanism are all built.
- **No change to pointer behaviour.**
- **D-24's scope half** — what `Discard` discards — is not yours.
- `update` **stays a pure reducer**.

## 6. Required tests

- The four edge cases per new region kind, **scroll visibility included**,
  live where live and N/A with a reason where not.
- **One test per adjacent overlay pair** (§2), asserting all three readers
  agree.
- **The confirm dialog returns the cursor** where it was, after both Cancel
  and Proceed, including the case where Proceed destroyed what it named.
- **Test Rule's `Enter` still runs the dry run** from the entry position and
  from each text field, and reaches each button by `Tab`.
- **Make the census executable.** A test that walks `crates/app/src/screens`
  and `crates/app/src/shell` counting `.on_press(` / `.on_input(` /
  `.on_toggle(` / `.on_action(` plus the `widgets::*_btn` helpers per file,
  and asserts the counts against a recorded table — the same tree-walk shape
  that guards `on_submit` (D-6). A new control then forces whoever adds it to
  update the table, and the census stops being a document that was true once.
  Put the table where the next reader will meet it, with a comment saying
  what to do when it fails.

## 7. Required evidence

**At the real 1280×720.** Slice 8's capture ran at half the window width —
the app occupied 50.3 % of the nested output in every required frame, against
100 % in slices 3–7 — and `RESULTS.md` nonetheless recorded 1280×720.
**Measure the window and record the measured number**, not the number the rig
was told to use.

Keyboard only, Wayland, `wtype`, zero pointer events. Capture:

1. `Tab` reaching the **top bar** and operating one of its actions.
2. The **Test Rule dialog**: a method button reached and changed, `Run` still
   firing on a bare `Enter`.
3. The **path assistant** used end to end: opened from the editor, JSON typed,
   a path chosen, `Insert path` activated, the value landing in the field.
4. The **confirm dialog restoring the cursor**: a frame before opening and a
   frame after cancelling, showing the cursor back where it was.
5. The **feedback banner's** Undo reached by `Tab` (not only `Ctrl+Z`).

Also **re-capture slice 8's three required sequences** at the corrected size
and **correct `RESULTS.md` in place** — including its test split, which says
31/6 where the real counts are 26/11.

Record backend, renderer, build SHA, and the **measured** window size.

## 8. Acceptance criteria

1. One overlay precedence source; the three disagreements gone; a test per
   pair.
2. Every control in §3 reachable, or recorded with the reason it is not.
3. `screens/scripts.rs` gone.
4. The two §4 corrections.
5. Tests per §6, including the executable census.
6. Evidence per §7, at the measured required size.
7. Gate green, exit code read directly; `update` still pure.

## 9. Known risks

- **The executable census may be noisy** against helper macros or multi-line
  calls. If a clean count is not achievable, say so and propose the narrower
  guard you can defend — do not ship a test that passes by being vague.
- **Test Rule's `Enter` default is easy to lose** while making its fields
  regions. It is the oldest behaviour in this cycle (task 026) and has a test;
  do not loosen that test to fit.
- **The banner is mutually exclusive by construction.** Derive its field list
  from the same function `feedback_banner` branches on, not from a second
  reading of the same conditions.

## 10. Required review-request content

Per workflow §9.2: summary; each surface decomposed; the overlay-precedence
source and its pair tests; the two corrections; the executable census and what
it guards; the four edge cases including scroll; the re-captured evidence with
measured geometry; gate status; review focus.

Commit, and **do not push without authorization**. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
