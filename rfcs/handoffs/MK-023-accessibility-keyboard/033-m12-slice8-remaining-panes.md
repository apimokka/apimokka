# Dev-team task 033 — M12 slice 8: the remaining panes, and the census

**Issued:** 2026-11-03
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** [MK-023](../../proposed/MK-023-accessibility-keyboard.md) § *External design*
**Cycle:** C2, implementation slice **8 of 8**

---

## 1. Purpose

Close scenario 5, cover the panes still unreachable, fold the scroll dispatches
into one pattern, and **produce the census that lets MK-023 claim keyboard
operability honestly** rather than by accumulation across eight slices.

On acceptance **D-12 closes** and **MK-023 moves to `done/`**.

## 2. Three surfaces are uncovered, not one — I enumerated this time

Slice 7 found the fallback editor. I then enumerated the rest, which I should
have done before slice 6 and did not:

| Surface | Interactive controls | State |
|---|---:|---|
| `routes/fallback_editor.rs` | `text_editor`, a status `text_input`, **Format / Save / Revert** | **uncovered** — scenario 5 |
| `routes/rule_set_config.rs` | a rule link, a strategy `pick_list`, a More toggle | **uncovered** |
| `shell/bottom_drawer.rs` | Close, and `JumpToDiagnostic` per row | **uncovered** |
| `routes/script_viewer.rs` | **none** — read-only | nothing needed |

The Routes tab's centre pane shows one of four things by sidebar selection, and
`order_for`'s Routes arm only ever adds `editor_fields`, which is empty unless a
**rule** is selected. So a keyboard user can select a rule set or a fallback file
from a navigable sidebar and land on a pane they cannot reach into.

The drawer is reachable from the palette's own rows. `Escape` closes it, so it is
not a trap, but its diagnostic list cannot be used.

## 3. Scope

- **Regions for the three uncovered surfaces.** Everything needed exists: text
  fields are one-item regions, the strategy `pick_list` is a value cycler
  (D-19's mechanism, including `Enter` cycling and the `⇅` handle), buttons are
  one-item regions, and the drawer's diagnostic list is an identity-cursor region
  if its rows have stable identity — **check, and if they do not, say so and use
  position with the reason stated**, as slice 5 did for header and body rows.
- **`script_viewer.rs` gets nothing**, deliberately. Record it as a null result so
  the census shows it was examined rather than missed.
- **Fold the three pre-existing scroll dispatches into slice 7's
  collect-and-batch pattern**, per that slice's review. They were never broken,
  but two patterns side by side means the next person picks one by coin flip —
  and the bug slice 7 hit was invisible until it ran live.

## 4. The census — the deliverable that ends this cycle

**Produce a table of every screen and pane in the application, with the
interactive controls it holds and the regions that cover them.** Derive it by
walking `crates/app/src/screens/` and `crates/app/src/shell/`, not from memory
and not from this handoff's table — **mine is a sample, and the point is to find
what it missed.**

Any surface with interactive controls and no region is a finding. **Report it
rather than building it**, unless it is small enough that leaving it would be
sillier than the scope breach — in which case say that is what you did.

This is what lets MK-023 move to `done/` on evidence rather than on the accretion
of eight slices each covering what it was told to. **The slice plan has now
under-scoped twice** — slice 5's editor and slice 6's two screens — both because
surfaces were taken from a list rather than counted.

## 5. Non-change scope

- **No change to D-21** or any contrast work. That is a separate task awaiting the
  project owner's decision on its scope.
- **No new region kind.** Everything here is an existing shape.
- **No change to pointer behaviour.**
- `update` **stays a pure reducer**.

## 6. Required tests

- The four edge cases per region kind, live where live, N/A with a reason where
  not — **and this time include *scroll visibility* explicitly**, which slices 5
  and 6 both substituted away and which became D-20.
- The drawer's list behaves when it empties and when its contents change.
- The strategy `pick_list` cycles and dispatches the dropdown's message.
- The three refactored scroll dispatches still fire — the refactor must not
  silently lose them, which is precisely how D-20 reached live.

## 7. Required evidence

**Scenario 5 end to end, keyboard only, Wayland, `wtype`, zero pointer events:**
select a fallback file, edit it, save, then revert — including the confirm
dialog, whose default action slice 1 already set to Cancel, so reverting needs a
deliberate move to Proceed.

Also: the rule-set config pane reached and operated, and the drawer opened from
the palette and used.

Record backend, renderer, build SHA, window size.

## 8. Acceptance criteria

1. The three surfaces navigable; `script_viewer` recorded as needing nothing.
2. **The §4 census delivered**, with any surface it finds that this handoff
   missed.
3. Scroll dispatches unified; the three pre-existing ones still fire, tested.
4. **Scenario 5 captured end to end**, zero pointer events.
5. Edge cases per §6, scroll visibility included.
6. Gate green; `update` still pure.

## 9. Known risks

- **The census may find a fourth surface.** That is its purpose. Report it;
  whether it lands here or in a ninth slice is mine to decide, not yours to
  absorb silently.
- **The refactor can silently drop a dispatch** — §6's last test exists for that.
- **The drawer's rows may have no stable identity**, as header and body rows did
  not. The precedent for that answer is already set.

## 10. Required review-request content

Per workflow §9.2: summary; the census; each surface's decomposition; the drawer
identity decision; the refactor and its test; the four edge cases including
scroll; scenario 5's capture; gate status; review focus.

Commit, and **do not push without authorization**. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
