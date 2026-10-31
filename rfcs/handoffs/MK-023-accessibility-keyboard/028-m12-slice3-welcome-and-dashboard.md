# Dev-team task 028 — M12 slice 3: Welcome and Dashboard

**Issued:** 2026-10-31
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** [MK-023](../../proposed/MK-023-accessibility-keyboard.md) § *External design — the keyboard interaction model*
**Cycle:** C2, implementation slice 3 of 6

---

## 1. Purpose

Close **D-12's headline symptom**: a keyboard-only user is stopped at Welcome,
the application's second screen. After this slice they reach a workspace.

**This is also where the model stops being easy.** Slices 1 and 2 declared four
edge cases and found all four inapplicable, because both regions were fixed-length
and neither scrolled. The Dashboard's workspace list is **filtered live by a
search field**, can be **emptied**, and sits **inside a `scrollable`** — so every
one of the four becomes real here, for the first time, at once.

## 2. The regions

**Welcome** — one region, two items, in visual order:

| # | Item | Message |
|---:|---|---|
| 0 | Open workspace | `GoDashboard` |
| 1 | Create new workspace | `GoWizard` |

**Dashboard** — three regions, in **visual** order, which is the Tab order:

| # | Region | Shape |
|---:|---|---|
| 0 | Create workspace (in the header) | one item, `GoWizard` |
| 1 | Search | one item, a **native `text_input`** |
| 2 | Workspaces | the **filtered** list; each item `OpenWorkspace(name)` |

Tab order follows the rendered column (`header`, `search`, `scrollable(content)`),
because a Tab order that disagrees with what the eye reads is its own confusion.

## 3. Three decisions the design settles here

External design, so mine to answer rather than yours to invent.

### 3.1 Leaving a text-field region must release its focus

Entering the Search region focuses the field, per the accepted design, so typing
works with no separate mode. **Moving the cursor away must unfocus it.**

Without this, a user who Tabs to the Workspaces region and types sends their
keystrokes into the still-focused search box — invisible, and it silently
re-filters the list under them. Buttons cannot take focus in iced, so nothing else
would claim it.

`iced_core::widget::operation::focusable::unfocus()` exists and is reachable
through the `advanced` feature we enable — verified. Pair it with the
`text_input::focus` M9 already wires.

### 3.2 The Workspaces cursor stores identity, not an index

This is the first region the design's identity rule applies to. The list is
rebuilt on every keystroke in the search field, so an index means the cursor
silently points at a different workspace as the user types.

**Store the workspace's identity** and derive its position when rendering and
activating. When the cursor's workspace is filtered away, fall back to **the
first surviving item**, or to **no cursor** when the list is empty.

### 3.3 The cursor must stay visible in the scrollable

The list is inside `scrollable`. A cursor that moves out of view is a cursor the
user has lost.

**M10's D-5 is the precedent, including its honesty:** iced 0.14 exposes no
scroll-into-view for a specific child, so that fix mapped the index to a relative
offset and said so plainly rather than claiming it scrolls. Do the same here, and
**if it cannot be made exact, say which cases are imprecise** instead of
describing it as solved.

## 4. Non-change scope

- **No other surface.** The sidebar tree is slice 4; the segmented regions and the
  trace list are slice 5; the palette moves and deleting `left_rail.rs` (D-16) are
  slice 6.
- **No change to slice 1's dialog defaults or slice 2's model** beyond what §3
  requires. If the model needs a genuine extension — identity cursors are one —
  that is expected; a *special case for one screen* is not, and is a design
  finding to report.
- **No `.on_submit`** on the search field. The tree-walking guard will catch it,
  and D-6 is why.
- `update` **stays a pure reducer.** Focus, unfocus and scroll are all `Task`s and
  belong in `update_and_dispatch`, exactly as M9's focus and M10's `snap_to` do.

## 5. Required tests

**All four edge cases are live here. None may be answered "N/A".**

1. **Clamp on shortening** — typing a query that filters the cursor's workspace
   away leaves a valid cursor, per §3.2.
2. **Empty region** — a query matching nothing leaves **no cursor**, and `Enter`
   is a **no-op** (not a panic, not opening a stale workspace).
3. **Entering resets** — per §3.1 of task 027: on the active item if one exists,
   otherwise the first.
4. **Scroll visibility** — tested as far as the mechanism allows, with the limits
   stated per §3.3.

Plus: Tab order matches §2; leaving Search unfocuses it (§3.1); `Enter` on a
workspace row opens that workspace and not another after filtering.

## 6. Required evidence — and this is the slice that proves the point

**A keyboard-only run, on the Wayland backend, with `wtype`, containing no
pointer click at all**, from launch to an open workspace:

launch → mode picker → `Enter` → Welcome → `Enter` on Open workspace → Dashboard →
type a filter → arrow to a workspace → `Enter` → workspace open.

**This retires the mixed-backend tension** recorded since task 026. Every earlier
run needed X11 and `xdotool` because reaching anything past Welcome required a
click. After this slice it does not, and the evidence should demonstrate exactly
that.

Record backend, renderer, build SHA and window size. Nested-compositor recipe with
`hotkey-overlay { skip-at-startup; }`. **No owner clicks** — and this time, none
available to fall back on, which is the proof.

## 7. Acceptance criteria

1. Welcome and Dashboard navigable per §2; a workspace opens by keyboard alone.
2. §3.1, §3.2 and §3.3 implemented as specified.
3. All four edge cases tested as live cases.
4. `update` still pure; gate green on both toolchains.
5. The §6 run captured, Wayland, `wtype`, **zero pointer events**.

## 8. Known risks

- **The identity cursor is new machinery.** Slice 2's index cursor cannot express
  it. Extending the model is expected; bolting a second cursor concept beside it
  is not — one model, as §7.5 of task 027 required.
- **Stray typing into the search field** is the §3.1 failure mode and it is silent.
  Test it, do not eyeball it.
- **Scroll precision.** M10 found iced gives no exact mechanism. An honest partial
  answer is acceptable; an overclaim is not.

## 9. Required review-request content

Per workflow §9.2: summary; how identity cursors were added to the model; the
§3.1 unfocus mechanism; the §3.3 scroll answer with its limits; the four edge
cases; Tab order; gate status; the keyboard-only run with its configuration and a
statement that it contained no pointer event; review focus.

Commit, and **do not push without authorization**. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
