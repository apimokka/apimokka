# Dev-team task 026 — M12 slice 1: read capture status, and give dialogs a default action

**Issued:** 2026-10-31
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** `rfcs/done/MK-023-accessibility-keyboard.md` § *External design — the keyboard interaction model*, **accepted 2026-10-31**
**Closes:** D-6, and the `Enter`-in-dialog defect task 024's audit found
**Cycle:** C2, implementation slice 1 of 6

---

## 1. Purpose

Fix a live defect and remove a latent hazard, as the first slice of the accepted
keyboard design.

- **Live:** pressing `Enter` in the Test Rule dialog or the confirm dialog does
  nothing. **This affects pointer users today**, not only keyboard users.
- **Latent:** the subscription ignores the capture status it is handed, so the
  application sees key events a focused widget already consumed. That is **D-6**.

## 2. Background

`app.rs`'s subscription uses `iced::event::listen_with`, whose closure receives an
`event::Status`. It is bound as `_status` and discarded. M9 adopted `listen_with`
precisely *because* it reports capture status, then did not read it — correct at
the time, because the only overlap with `text_input`'s captured set was `Escape`.

`EnterPressed`'s reducer arm consults exactly two things: `command_palette.open`,
and `audience_mode.is_none()` with `mode_picker_selected`. It never consults
`test_rule.open` or `confirm_dialog`, so neither dialog has a default action.

## 3. Scope — behaviour only; no region machinery yet

**Do not build the region-and-cursor model in this slice.** That is slice 2. This
slice delivers the *behaviour* the design specifies for dialogs, by extending the
existing `EnterPressed` arm, and the capture-status fix. Slice 2 generalises the
plumbing and will subsume what you write here.

Keeping them apart is deliberate: this slice fixes a bug and should be reviewable
without reading a new architecture.

## 4. Required implementation

### 4.1 Read the capture status

In the subscription closure, bind the status and **ignore any key event already
captured by a focused widget — except `Escape`.**

`Escape` is the deliberate exception and the reason is recorded in M9's review:
`text_input` captures `Escape` unconditionally when focused, so without the
override the palette would again need two presses to close.

### 4.2 Give each dialog a default action — **on the safe side**

Extend `EnterPressed` to consult the open dialog. **`Enter` activates the dialog's
default item, and for a destructive dialog that is not the destructive button.**

| Dialog | Default on `Enter` | Why |
|---|---|---|
| Test Rule (`test_rule.open`) | **`Message::TestRuleRun`** (`test_rule.rs:134`) | A local dry run with no side effect. |
| Confirm (`confirm_dialog`) | **`Message::ConfirmCancel`** — **never `ConfirmProceed`** | `ConfirmProceed` deletes a rule set, or leaves/switches a workspace discarding unsaved work. One keystroke must not do that. |

**This is the point of the slice, so do not "improve" it.** Making `Enter` confirm
would be the conventional-looking choice and it is wrong here: the dialog exists
to make a destructive act deliberate, and a default that performs it defeats the
dialog. Proceed becomes reachable in slice 5, by moving a cursor first.

**Ordering matters.** A confirm dialog can be open over other state; resolve
precedence explicitly and say what you chose. `EscapePressed`'s existing priority
order (`confirm_dialog` → `workspace_menu_open` → `command_palette.open` →
`test_rule.open` → …) is the obvious precedent to follow.

### 4.3 Assert that no `text_input` sets `on_submit`

The design forbids it: with `on_submit` unset, `Enter` is never captured, so
activation reaches the application and D-6's double-fire cannot occur. **None
exists today** — the only two matches in `crates/app/src` are comments. Add a test
that fails if one appears.

## 5. Non-change scope

- **No region or cursor model.** Slice 2.
- **No new navigable surface.** Welcome, Dashboard, the sidebar and the segmented
  controls are later slices.
- **No `.on_submit`,** anywhere, ever — see §4.3.
- **No change to `EscapePressed`'s behaviour or priority order.**
- No palette change; no `theme.rs` change; no snora change.
- **Do not delete `left_rail.rs`** — that is slice 6.

## 6. Required tests

Reducer tests, in the established per-module `tests.rs` layout:

- `Enter` with the Test Rule dialog open dispatches `TestRuleRun`.
- **`Enter` with the confirm dialog open dispatches `ConfirmCancel` and leaves the
  workspace unchanged** — assert the *absence* of the destructive effect, not only
  the message, because this is the safety property the slice exists to establish.
- `Enter` with no dialog open behaves exactly as before for the palette and the
  mode picker.
- Precedence: with two surfaces open, the one you chose wins.
- A captured key event does not reach the application; `Escape` still does.
- No `text_input` in the tree sets `on_submit`.

Each must fail before the change. Say so plainly if one cannot.

## 7. Required evidence

- `bash scripts/check-release-gates.sh` green on both toolchains.
- **A live check of both dialogs**, keyboard-driven, showing `Enter` runs the test
  and `Enter` on the confirm dialog cancels rather than proceeds. Use task 022's
  nested-compositor recipe with the coordinate conversion and
  `hotkey-overlay { skip-at-startup; }`; `wtype` on the Wayland backend is the
  right tool for a keyboard behaviour. **No owner clicks.**
- Record backend, renderer, build SHA and window size.

## 8. Acceptance criteria

1. Captured key events ignored except `Escape`; D-6 closed.
2. Both dialogs have a default action, the confirm dialog's being **Cancel**.
3. Precedence stated and tested.
4. No `on_submit`, asserted.
5. Reducer tests as §6, each discriminating.
6. Gate green; live check captured with configuration recorded.

## 9. Known risks

- **The tempting wrong default.** See §4.2.
- **Reading status may silence something that currently works by accident.** If a
  shortcut stops working because it was only ever arriving as a captured event,
  that is a finding — report it rather than re-admitting captured events.
- **Precedence is a judgement call**; it is yours to make and mine to review.

## 10. Required review-request content

Per workflow §9.2: summary; the precedence decision; changed files; tests and what
each discriminates; anything the capture-status change silenced; gate status;
evidence paths with backend, renderer and SHA; review focus.

Commit, and **do not push without authorization**. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry. Commit
messages: digest plus optional body within 400 characters.
