# Dev-team task 035 — M12 slice 10: overlays the view actually draws

**Issued:** 2026-11-03
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** [MK-023](../../proposed/MK-023-accessibility-keyboard.md) § *External design*
**Review that produced it:** `.git-exclude/reviewed/2026-11-03-task034-slice9-review.md`
**Cycle:** C2, implementation slice **10 of 10**

---

## 1. Purpose

Slice 9 gave the overlays one precedence source. The source answers *which
overlay state is set*; it does not answer *which overlay is drawn*, and
those differ, because `App::view` routes only `AppView::Workspace` through
`shell::view`. The result is the design's own central sentence failing:

> At any moment exactly one **region** holds the keyboard cursor.

**The brief for slice 9 scoped D-22 to overlay-against-overlay and said the
existing pairs were enough. That was my omission, not yours.** This slice
closes the dimension it missed, and the container hierarchy — region inside
overlay inside view, with `App::view` as the root — is then exhausted.

On acceptance **D-22 and D-26 close** and **MK-023 moves to `done/`**.

## 2. The three states, reproduced in review

**(a) The workspace menu, four keystrokes from a fresh workspace, keyboard
only.** `Tab` (top-bar switcher) → `Enter` → `Tab` ("Open another
workspace") → `Enter`:

```
view = Dashboard   menu_open = true
order on this view = [WorkspaceMenu(Recent), WorkspaceMenu(Open), WorkspaceMenu(Create)]
```

On the Dashboard none of its own controls can be reached — not Search, not
Create, not a workspace row. `Enter` on `Recent` opens a workspace never
shown; `Enter` on `Create` opens the Wizard unasked. `Escape` recovers, with
nothing on screen to suggest it. **S1**: scenario 1 runs on this screen.

**(b) The confirm dialog on the Dashboard — pointer users too, pre-existing.**
With unsaved edits, activating a Dashboard workspace row raises
`ConfirmRequest(SwitchWorkspace)` and **nothing draws it**:

```
view = Dashboard   confirm_open = true   order = [ConfirmDialog]
```

The click appears to do nothing, every time.

**(c) The same from the Wizard** — `WizardCreate` with unsaved edits. Create
appears dead. **(d) The drawer**, same cause, palette-reachable, lower
severity.

## 3. Scope

1. **`topmost_overlay()` reports only overlays the current view draws.**
   `TestRule`, `PathAssistant`, `WorkspaceMenu` and `Drawer` are
   workspace-only surfaces: outside `AppView::Workspace` they are not
   reported, whatever their stored flag says. (`CommandPalette` is already
   inert off the workspace — D-4 option 2; `ModePicker` is handled by
   `App::view`'s own early return.)
   **Derive; do not clear on transition.** The six `self.view = …` sites are
   exactly the per-transition enumeration the design warns against, and the
   next feature would add a seventh.
2. **The confirm dialog is drawn wherever it is raised.** It is the one
   overlay legitimately raised outside the workspace, and an undrawable
   confirmation is a correctness defect for pointer users on its own.
   `App::view` attaches it over whichever screen is showing — one place, not
   four screens each growing a dialog layer.
3. **D-26 — one source per HTTP-method table.** `screens/test_rule.rs:111`
   and `regions.rs:1876` each spell the five-method list, with `len`'s `5`
   beside them; `rule_editor.rs:533` and `regions.rs:1547`/`:1711` each spell
   the six-entry list. One shared constant per table, the editor's `Any`
   prefix derived rather than written again, and `len` read from it.
4. **Two small cleanups** named in the review: express
   `blocks_region_navigation` and the `ArrowUp`/`ArrowDown` arms through
   `topmost_overlay()` like the rest of the chain, and remove (or justify)
   `shell/view.rs`'s unreachable `Overlay::ModePicker` arm.

## 4. Non-change scope

- **No contrast or colour work.** D-21 and D-25 remain one separate task.
- **D-24's scope half** — what `Discard` actually discards — is the owner's.
- **No new region kind**, and no change to the slice-9 ranking itself: its
  order is confirmed correct, and one-overlay-per-frame stays.
- `update` **stays a pure reducer**.

## 5. Required tests

- **A test per workspace-only overlay, outside the workspace**: with the
  flag set and `view` on Dashboard / Wizard / Welcome, `topmost_overlay()` is
  `None` and `order_for` yields that screen's own regions. §2(a) is the
  named case: assert the Dashboard's own regions are reachable after the
  four-keystroke sequence, driven by messages, not by setting state directly.
- **A test that the confirm dialog survives the view it was raised from**:
  raised on the Dashboard and on the Wizard, it is the topmost overlay, it
  owns the cursor, `Enter` cancels and `Escape` cancels — and the screen
  underneath is the one the user was on.
- **Re-read slice 9's two ruled-out pairs** (`032`-style structural
  arguments in that request's §3) and confirm they still hold once
  `topmost_overlay()` is view-aware; one of them argues from `order_for`,
  which is the thing changing. Say which you re-checked.
- **The method tables**: one assertion that the view's list and the region
  model's `len`/activation agree, so a sixth method cannot be added to one
  alone.

## 6. Required evidence

Keyboard only, Wayland, `wtype`, zero pointer events, **at the measured
1280×720** — the geometry recipe slice 9 established, including the nested
`maximize-column` and `output winit scale`. Record the measured size.

1. **§2(a) end to end**: the four keystrokes, then the Dashboard's own
   search field or a workspace row reached by `Tab` and activated — the
   frame that shows the trap is gone.
2. **§2(b)**: with unsaved edits, a Dashboard workspace row activated and
   **the confirm dialog visible over the Dashboard**; cancel, and the
   Dashboard is still there with the cursor restored.
3. **§2(c)**: the same from the Wizard's Create.
4. **The drawer while the confirmation it raised is up** — `Discard`,
   `Enter`, the dialog alone on screen, cancel, the drawer back with the
   cursor on `Discard`. Slice 9 added that routing and no frame shows it.

## 7. Acceptance criteria

1. The four states in §2 gone, each with a test and (1–4) a frame.
2. The confirm dialog drawn on every view that can raise it.
3. One source per method table.
4. The two cleanups in §3.4.
5. Gate green, exit code read directly; `update` still pure.
6. Evidence at the measured required size.

## 8. Known risks

- **Deriving "is this overlay drawn" can be written as a second list that
  drifts from `App::view`** — which is the defect, one level along. Put the
  view-to-overlay-set relation in one place that both `App::view` and
  `topmost_overlay()` read, or state plainly why the shape you chose cannot
  drift.
- **Attaching the confirm dialog outside the shell may want `AppLayout`**
  where the three non-workspace screens use none. If a thin host is the
  honest answer, build it; if it forces a change bigger than this slice,
  stop and report rather than absorbing it.
- **The method-table constant has four call sites with three shapes**
  (plain, `Any`-prefixed, `""`-prefixed). Derive the variants; do not add a
  fourth literal to avoid a derivation.

## 9. Required review-request content

Per workflow §9.2: summary; the view-aware source and how it cannot drift
from `App::view`; where the confirm dialog is now attached; the method
tables; the two cleanups; the tests; the evidence with measured geometry;
gate status; review focus.

Commit, and **do not push without authorization**. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
