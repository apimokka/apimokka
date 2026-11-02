# Dev-team task 031 — M12 slice 6: Settings and the Wizard

**Issued:** 2026-11-03
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** [MK-023](../../proposed/MK-023-accessibility-keyboard.md) § *External design*, and its **2026-11-03 correction**
**Cycle:** C2, implementation slice 6 of **7** — the count changed; see §1

---

## 1. Why this slice exists at all — an error in the design, not in your work

**Two screens were never assigned to a slice.** Verified at source:
`Region::order_for` has `AppView::Wizard => vec![]` and
`WorkspaceTab::Settings => {}`.

So **two of MK-056's six primary scenarios are not keyboard-completable**, and
M12's acceptance criterion cannot be met as the slice list stood:

- **Scenario 2** — *"create a workspace from the wizard and add a rule"* — the
  wizard's 8 controls are unreachable.
- **Scenario 6** — *"change theme and locale, and switch audience mode"* —
  Settings' 11 controls are unreachable.

The six slices were drawn from the audit's list of **surfaces** and never mapped
back against the six **scenarios**. The audit named both screens' controls; the
slice list dropped them. Mine to own, and the reason slice 6 is now two slices.

## 2. Scope — mechanical, against a pattern already established

Nothing here needs a new mechanism. Slices 2–5 built every shape this needs.

### Settings

| Control | Region shape |
|---|---|
| Theme: 4 buttons | **multi-item group** (arrows within), like method and respond-mode |
| Guidance: Guided / Expert | **multi-item group** |
| Locale `pick_list` | **value cycler**, per slice 5's `is_value_cycler`/`cycle_value` |
| Host, Port, Queue size, Log file | one-item regions; **Log file is a disabled input** — reachable or skipped, your call, but say which and why |
| Enable TLS, Enable trace (checkboxes) | one-item regions; `Enter` toggles |
| More/Fewer settings disclosure | one-item region |

### The Wizard

Name, Folder, Host, Port, Queue size (text inputs), the TLS and trace
checkboxes, and the section toggles — one-item regions in reading order; the
starter-choice radios are a **multi-item group**.

**Tab order follows reading order** on both screens, as every slice since 3.

## 3. The decision this slice reverses, and why

The design said theme and audience mode would **move to the command palette**
instead of becoming regions. **That was wrong and is withdrawn.**

They do not leave the screen — the buttons stay for pointer users — so the effect
would be **visible controls a keyboard user cannot reach, on a screen whose other
controls they can**. That is a worse inconsistency than the one it avoided. The
*"Less is more"* argument was for fewer **controls**, and this never removed any.

**So: make them regions, like everything else.** Palette commands for theme,
audience mode and locale are added in **slice 7**, as an *additional* route, not
a substitute.

## 4. Non-change scope

- **Not this slice:** **D-16** (deleting `left_rail.rs`), **D-18** (a click
  entering its region), **D-19** (`Enter` on a cycler, and the handle affordance),
  and the palette commands. All are **slice 7**, where the remaining design
  judgement lives. Keeping them out keeps this slice reviewable as pattern
  application.
- **D-19 is open while you build this.** Settings' locale picker will have the
  same inert `Enter` and misleading chevron as the editor's three. **Do not fix it
  here** — one fix in one slice, applied to all four.
- **No change to pointer behaviour.** No widget replaced.
- **No `.on_submit`** on any field; slice 1's guard will catch it.
- `update` **stays a pure reducer**.

## 5. Required tests

The four edge cases per region kind, **live where live and N/A with a reason
where not**, as every slice since 2.

Specifically:
- the theme and guidance groups **resume on the current value**, not the first
  item (task 027 §3.1);
- conditional regions behave: Settings' advanced block appears only when expanded,
  the wizard's section contents only when their section is open — derived in
  `order_for`, never stored, and a cursor must not survive into a region that has
  gone;
- the locale cycler dispatches `Message::ChangeLocale` — assert the message;
- Tab order matches reading order on both screens.

## 6. Required evidence

Keyboard-only, **Wayland, `wtype`, zero pointer events**:

- **Scenario 6 end to end**: change the theme, change the locale, switch audience
  mode — all three by keyboard, which is the scenario's own wording.
- **Scenario 2's wizard half**: reach the wizard from Welcome, fill the name and
  folder, and create the workspace.

Record backend, renderer, build SHA, window size; nested-compositor recipe with
`hotkey-overlay { skip-at-startup; }`.

**One frame worth capturing deliberately:** Settings with the cursor on the theme
group, showing a keyboard user can now reach controls that were visible and
unreachable before this slice. That is the defect being closed.

## 7. Acceptance criteria

1. Settings and the Wizard navigable per §2; theme and audience mode are
   **regions** per §3.
2. **Scenarios 2 and 6 keyboard-completable**, captured.
3. Edge cases per §5, including the conditional regions.
4. `update` still pure; gate green on both toolchains.
5. Zero pointer events in both runs.

## 8. Known risks

- **The Tab chain on Settings is long**, as it is in the editor. Accepted cost of
  one Tab semantics, unless the capture shows it unusable rather than merely long
   — which is a report, not a workaround.
- **The disabled Log file input.** A disabled control that is a Tab stop is
  debatable either way; pick one, state the reasoning, and be consistent with the
  wizard if it has a parallel case.
- **Scope pull toward D-19**, which you will see on the locale picker. §4.

## 9. Required review-request content

Per workflow §9.2: summary; the region decomposition for both screens; the
disabled-input decision; conditional-region handling; the four edge cases; Tab
order; gate status; both runs with configuration; review focus.

Commit, and **do not push without authorization**. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
