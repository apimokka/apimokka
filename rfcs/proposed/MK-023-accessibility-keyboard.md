# RFC MK-023 — Accessibility and keyboard contract

**Status.** Proposed
**Tracks.** ABDD policy, keyboard shortcut catalogue, focus order, screen-reader landmarks.
**Touches.** Every interactive surface.
**Supersedes.** MK-015 (accessibility, keyboard, command palette).

> ## Status correction, 2026-08-15
>
> **Returned from `done/`.** Recorded as `Implemented (v0.6.0)`; the keyboard
> contract below was never built. Corrected by project-owner decision.
>
> **What is not implemented.** There is no `Named::Tab` handler, no
> `focus_next`/`focus_previous`, and no `widget::operate` anywhere in
> `crates/app/src`. iced does not traverse focus on its own — the application
> must wire it, and this one does not. Keyboard handling is a **global**
> subscription (`app.rs:3418`): `Escape` plus a six-entry accelerator table
> (Undo, Redo ×2, Save, ReloadConfig, ToggleCommandPalette). Nothing else.
>
> **Consequence.** A keyboard-only user cannot pass the application's **first
> screen**. `Message::ChooseAudienceMode` fires only from a button `.on_press`
> (`screens/mode_picker.rs:32`), it is not in the accelerator table, and
> `view()` short-circuits to the mode picker while `audience_mode.is_none()`.
> This is more severe than the shipped `high_contrast_dark` modal-dim defect
> M8 exists to fix, which at least degraded a usable application.
>
> **Line 93 of this RFC — `- [ ] Main workflows are keyboard reachable` — is
> unticked in the shipped document.** The gap was visible in the record and
> read past.
>
> **How it was found.** Attempting to drive the app with `wtype` for M8's
> capture. The tool works; the application does not accept the input. Evidence:
> `.git-exclude/reviewed/2026-08-15-wtype-probe-result-and-keyboard-contract-gap.md`.
>
> **The satisfiable path is this RFC's own wording.** Line 17 requires every
> control reachable *"via Tab **or the command palette**"*. A fully
> keyboard-operable palette satisfies the contract, and that is MK-033's
> unimplemented requirement rather than new design. See MK-033's matching
> correction.
>
> **Correction, 2026-08-18 — do not record Tab traversal as iced-blocked.** An
> earlier revision of this block said full Tab traversal "may not be achievable
> on iced 0.14". That was inherited from snora's documentation, which said a
> focus ring "cannot be rendered" on iced 0.14 and instructed reviewers not to
> file it. **snora has withdrawn that as over-scoped**, having probed iced
> directly:
>
> - `operation::focus_next()` and `focus_previous()` **are reachable** and
>   return a `Task`. **Moving focus works today.**
> - `focusable::find_focused()` — *querying* where focus is — requires iced's
>   `advanced` feature. **Knowing where focus is does not.**
> - A focus **ring** is renderable now for focus the *application* owns: a
>   `container` style closure is an arbitrary `Fn(&Theme) -> Style`, so a
>   focused boolean the application already holds can drive border colour and
>   width. The real constraint is only that iced cannot tell a style closure
>   that a widget **iced** owns is focused.
>
> So Tab traversal and visible focus rings are a **scope choice**, not a
> framework limitation. Choosing the palette route stays correct on cost
> grounds; it must not be justified on impossibility grounds, and the production
> project should not inherit that wrong constraint.
> ### Correction, 2026-09-02 — focus *querying* is available to us
>
> The bullet above says `find_focused()` needs iced's `advanced` feature and
> therefore modal focus **trapping** is unavailable. **That is wrong for this
> application.** It was inherited verbatim from snora, whose statement is
> *"requires iced's `advanced` feature, which snora does not enable"* — true of
> them, and carried across as though it bound us.
>
> **We enable it.** `Cargo.toml:26`: `iced = { version = "0.14", features =
> ["tokio", "advanced"] }`. And `iced::advanced::widget` re-exports
> `core::widget::*`, which carries
> `operation::focusable::find_focused()` (`iced_core-0.14.0/src/widget/
> operation/focusable.rs:201`, `pub`), alongside `advanced::widget::operate` for
> running an `Operation` as a `Task`.
>
> **Stated with its limit:** this establishes that querying focus is *reachable*.
> It does not establish that a working focus trap can be built from those pieces
> — that needs the modal's content ids, a boundary comparison and a redirect,
> and none of that has been designed. **Focus containment is therefore
> unimplemented, not impossible**, and must not be recorded as a framework limit.
>
> This is the fourth inherited snora constraint found not to bind us, and the
> second of that kind I propagated after having already been caught by it.

>
> **This is the third RFC found in `done/` whose central mechanism was never
> built**, after MK-024 (breakpoints) and MK-033 (palette keyboard operation).
> The common factor is that nothing in this programme executed the UI until
> `scripts/ux/` existed; all three were verified against design intent.
>
> ### Zone navigation — recorded for the production project, 2026-08-19
>
> The half of this contract that M9 does **not** address is movement *between*
> regions: header to body, or between the Routes workbench's sidebar, editor and
> respond columns. That has no keyboard story today, documented or otherwise.
>
> snora ships a purpose-built answer, available since 0.35.0 and present in our
> current pin: `snora_core::focus::{FocusZone, Cycle, ZonePresence, next_zone}`
> with `snora::keyboard::cycle_zones`. `next_zone` is pure — it takes state and
> returns `Option<FocusZone>`, cycling `Header → SideBar → Body → Footer`,
> skipping absent slots, and returning `None` while a dialog or sheet is open so
> that a modal owns focus. snora deliberately **does not take Tab**; F6 /
> Shift+F6 is their recommendation and the binding is ours to choose.
>
> **Deferred, not overlooked.** `ZonePresence` describes `AppLayout`'s
> `header`/`side_bar`/`body`/`footer` slots, and we populate only two of them —
> our tab bar and per-screen sidebars are composed into `body` by hand. Adopting
> it therefore means first modelling our own logical regions, which is design
> work rather than adoption, and belongs in the production project alongside the
> rest of this RFC's remaining scope.
>
> Recorded here so it is inherited rather than rediscovered.
>
> **Not shipped by snora, but not closed to us:** snora ships navigation, not
> containment — nothing bounds Tab inside an open modal. They attribute that to
> `focusable::find_focused()` requiring iced's `advanced` feature, which snora
> does not enable. **We do** (`Cargo.toml:26`), so containment is available to
> design for and is unimplemented rather than impossible. See the 2026-09-02
> correction above.
>
> The design below is unchanged and remains the intended target.

## Summary

apimokka commits to **Accessibility By Default Design** (ABDD): the app is usable by keyboard, by screen reader, at 200% text scale, in light or dark mode, with no information conveyed by colour alone. This RFC defines the contract every other screen RFC must honour.

## ABDD requirements

| Requirement | Design rule |
|---|---|
| Status clarity | All status indicators use **icon + text label** |
| Keyboard reachability | Every control is reachable via Tab or the command palette |
| Focus visibility | Focus ring is high-contrast in both themes |
| Non-colour signalling | Match/miss/error cannot rely only on colour |
| Screen-reader naming | Icon buttons have accessible labels |
| Font scaling | Layout tolerates 200% text scaling without clipping |
| Locale expansion | JA labels fit without clipping (typically 10–30% wider than EN) |
| Motion sensitivity | Animation is never required for understanding |

## Keyboard shortcut catalogue

| Shortcut | Action | Available in |
|---|---|---|
| `Esc` | Close topmost overlay (dialog → drawer) | Any |
| `Ctrl/Cmd + K` | Toggle command palette | Any workspace view |
| `Ctrl/Cmd + S` | Save workspace | Workspace |
| `Ctrl/Cmd + R` | Reload config | Workspace when reload available |
| `Ctrl/Cmd + Enter` | Run Test Rule | Test Rule dialog focused / rule editor |
| `Tab` / `Shift + Tab` | Move focus forward / backward | Any |
| Arrow keys | Navigate lists, trees, segmented controls | Tree / segmented contexts |
| `Enter` | Activate focused **non-danger** primary action | Any |
| `Space` | Toggle checkbox / button where appropriate | Any |

Notes:
- `Enter` is intentionally restricted to non-danger primary actions to prevent accidental destructive activation. Danger actions require explicit focus on the danger button.
- The command palette is the canonical fallback for any action that lacks a dedicated shortcut. Adding a shortcut is preferred when it serves a frequent workflow; otherwise the palette suffices.

## Control inventory — measured 2026-10-31, task 024's audit

Promoted here from `.git-exclude/review-request/2026-10-31-task024-audit-halt.md`
because it is **specification, and that directory is not under version control.**
Whatever the project owner decides about D-12's scope, this inventory is what a
production implementation needs and what this repository exists to hand over.

### What the application dispatches globally today

`Message::ArrowUp` / `ArrowDown` / `EnterPressed` are dispatched unconditionally
from `iced::event::listen_with` and touch **exactly two fields in the whole
application**: `command_palette.selected` and `mode_picker_selected`.
`EscapePressed` is the only other global route and is **dismiss-only** — it closes
overlays in priority order and activates nothing.

### Per-scenario gaps

| Scenario | Controls with no keyboard route | Shape of the work |
|---|---|---|
| 1. Open a workspace, find a rule | Welcome's 2 buttons; Dashboard's workspace list; the sidebar's rule-set and rule rows | **A two-level tree.** Selection lives in `selection::RouteSelection` as opaque IDs, not a navigable index, so arrow navigation needs index↔ID mapping that does not exist even latently — and collapse/expand interacting with arrow position has no precedent here. |
| 2. Wizard and add a rule | Wizard: 3 section toggles, 3 starter radios, TLS checkbox, Create; then the sidebar add actions, `rule_editor`'s 3 `pick_list`s, 6 method buttons, 2 respond-mode tabs | The wizard's two text fields are natively focusable; everything else is not. |
| 3. Edit conditions, Test Rule | `rule_editor` as above; header/body add-remove (4 buttons); the Test Rule dialog's 5 method buttons, Run and Close | **Run is unreachable by `Enter`** — see below. |
| 4. Inspect a trace event | Pause, Clear, the trace event list (by-ID selection again), 3 outcome-specific actions, Replay | Same missing index concept as the sidebar, without the tree. |
| 5. Save and revert a fallback draft | `text_editor` (natively focusable); Format, Save, Revert; the confirm dialog's Cancel and Proceed | **Proceed is unreachable by `Enter`.** |
| 6. Theme, locale, audience mode | 4 theme buttons, 2 audience-mode buttons, the locale `pick_list` | A `pick_list` is unreachable by any focus mechanism in iced 0.14. |

### Two findings that are defects rather than gaps

1. **`EnterPressed` never consults `test_rule.open` or `confirm_dialog`.** Both are
   modal surfaces with an obvious default action, and Enter does nothing in them —
   **for pointer users too**. This is a correctness defect, not a missing feature.
2. **It cannot be fixed by adding `.on_submit`** without waking **D-6**: Test
   Rule's dialog holds three `text_input`s, and `listen_with` ignores capture
   status, so `Enter` would be consumed by the field *and* dispatched globally,
   firing twice. A mechanism decision is required before code.

### Why this is more than "extend what M9 built"

Nine-plus new navigable state fields would be needed, each requiring M9's four edge
cases (clamp on shortening, empty-list no-op, reset on entry, scroll visibility);
five of them are segmented controls repeating one idiom; one is the tree above. The
architect's 2026-10-06 recommendation called this *"extension, not invention"* —
**the audit shows it is invention**, and that assessment was made without measuring.

### Scope status

Open. D-12 is S1 and MK-056 §8 forbids deferring an S1, so the choice is between
building it and the project owner narrowing what the mockup claims. The options,
with a recommendation, are in
`.git-exclude/reviewed/2026-10-31-task024-audit-review-and-decision-request.md` §3
and its 2026-10-31 revision. **This RFC's Status does not change until that is
decided.**

**What the inventory above is not.** It is a gap inventory, not a design. The
architect's first recommendation was to implement the two cheapest routes and
inherit the rest; that was withdrawn the same day against the owner's
fundamental philosophy — a model present on two surfaces and absent on the rest
is the kind of UI that confuses rather than one *"for users not to be confused or
misunderstand"*, and shipping it would have handed the production project a
keyboard contract never executed, which is R1 finding B3's defect created
deliberately.

**The recommendation is therefore an external design first**, amending this RFC,
settling tree and list navigation, the segmented-control idiom, the dialog
default action (resolving D-6), the palette's role, D-16's left rail — and,
under *"Less is more"*, whether all nine surfaces should keep their current shape
at all, since five segmented controls repeating one idiom is itself a signal that
the answer may be fewer controls rather than more navigation. Implementation
follows acceptance, in reviewable slices.

## External design — the keyboard interaction model

**Status: accepted by the project owner 2026-10-31** as the external design that
D-12 requires. Internal design, program design and implementation follow from it,
in slices. This section is the contract; the control inventory above is the
problem statement it answers.

### Why a conventional focus model is unavailable

In `iced_widget` 0.14.2 only `text_input` and `text_editor` implement `Focusable`.
A `button`, `pick_list`, `radio` or `checkbox` cannot take focus, so there is no
focus ring to move and no traversal to wire. **The application must own the
selection itself.** That is not a workaround; it is the only model the toolkit
permits, and it should be designed deliberately rather than approximated per
screen.

### The key budget, measured rather than assumed

`text_input` captures exactly: `ArrowLeft`, `ArrowRight`, `Backspace`, `Delete`,
`End`, `Escape`, `Home`, and `Enter` only when `on_submit` is set — verified in
`iced_widget-0.14.2/src/text_input.rs`.

| Key | Captured by a focused field? | Therefore usable for |
|---|---|---|
| **Tab / Shift+Tab** | **No** | Moving between regions — always available, even while typing |
| **ArrowUp / ArrowDown** | **No** | Moving within a region — always available |
| ArrowLeft / ArrowRight | **Yes** | Within-region movement **only where the region holds no text field** |
| Enter | Only with `on_submit` | Activation — and `on_submit` is therefore forbidden, see D-6 below |
| Escape | Always | Dismissal, already handled |

This budget is the reason the model below works: the two movements a user performs
constantly are on keys nothing competes for.

### One model: regions and a cursor

**At any moment exactly one *region* holds the keyboard cursor, and the cursor
names one item within it.**

- **Tab / Shift+Tab** move between regions, in a fixed order per screen.
- **ArrowUp / ArrowDown** move the cursor within the region.
- **ArrowLeft / ArrowRight** move within regions that are horizontal, and
  collapse/expand in the tree — available because those regions contain no text
  field.
- **Enter** activates the cursor's item.
- **Escape** is unchanged: dismissal only, in the existing priority order.

Entering a region whose item is a text field focuses that field, so typing works
without a separate mode. This reuses `text_input::focus`, which M9 already wires.

**One model, every screen.** Task 024's audit found nine surfaces each needing
their own selection concept; this is one concept applied nine times instead of
nine idioms. A user who learns it on the first screen knows it everywhere, which
is what *"for users not to be confused or misunderstand"* requires.

### A cursor stores identity where the list can change

The audit found the sidebar's selection lives as opaque IDs rather than an index,
and read that as an obstacle. **It is the better shape and the design adopts it.**

- Where the list can change under the cursor — the sidebar tree, the trace list —
  **store the item's identity and derive its position on demand** from a function
  over the current snapshot. A stored index goes stale when the list shortens,
  which is the first of M9's four edge cases and the one most easily got wrong.
- Where the list is fixed and short — the tab bar, a segmented control, a
  dialog's actions — an index is sufficient and simpler.

This is the `accelerator.rs` and `palette_commands.rs` precedent again: derive from
one source rather than storing a second copy that can disagree.

### The tree is a flattened visible sequence

The sidebar is two-level. The design does **not** invent a tree-walking scheme:

- a function yields the **visible** rows in display order — collapsed children
  simply absent;
- `ArrowUp`/`ArrowDown` move along that sequence;
- `ArrowLeft` collapses an expanded row, or moves to the parent when already
  collapsed; `ArrowRight` expands, or moves to the first child when already
  expanded.

This is the conventional tree-as-list behaviour users already know from file
trees, and it removes the "index into a tree" problem entirely: there is no tree
index, only a position in a derived list.

### Activation, dialogs, and the resolution of D-6

**D-6 is resolved by reading the capture status we already receive.** The
subscription uses `iced::event::listen_with`, whose closure is handed an
`event::Status`; today it is bound as `_status` and discarded. The design:

- **ignore any key event already captured by a focused widget**, except `Escape`,
  which is deliberately overridden for the reason M9 recorded;
- **never set `on_submit`** on any `text_input`. With `on_submit` unset, `Enter`
  is not captured, so activation reaches the application cleanly and the
  double-fire D-6 warns about cannot occur.

Together these close D-6 rather than stepping around it.

**A dialog is a region whose cursor starts on its safe action.** `Enter`
activates **the cursor's item**, not "the primary button" — a distinction that
matters and that an earlier draft of this section got wrong.

- **Test Rule**: the cursor starts on **Run** (`Message::TestRuleRun`). The action
  is a local dry run with no side effect, so `Enter` may activate it immediately.
- **The confirm dialog**: the cursor starts on **Cancel**, never on Proceed.
  `Message::ConfirmProceed` deletes a rule set, or leaves or switches a workspace
  discarding unsaved work. **`Enter` defaulting to Proceed would let one keystroke
  destroy a rule set**, which fails *"finally clean, safe and secure"* outright.
  Proceed is reached by moving the cursor first — deliberately, which is the point
  of a confirmation.

Escape continues to dismiss, so a keyboard user has two safe exits and one
deliberate path to the destructive one.

This also fixes the live defect the audit found, where `Enter` does nothing in
either dialog **for pointer users too**.

While a dialog is open it is the only region Tab cycles within, which is the focus
containment MK-023 has wanted since its 2026-09-02 correction and which is
achievable here because the application owns the cursor.

### The visible indicator

The cursor uses the **2 px primary border** that `card_selected_style` already
draws. Task 022's capture judged it reads clearly as selection in both presets, on
exactly these list-like surfaces, so this reuses a validated visual rather than
inventing one. Border width and colour together are a non-colour cue, satisfying
MK-022 and the non-colour requirement.

### What the model removes — *"Less is more"*

The audit counted nine navigable surfaces. **The design does not make all nine
navigable**, because five of them repeating one idiom is a signal about the
product, not only about the keyboard:

- **Theme and audience mode move to the command palette.** Both are global,
  infrequent, and already belong to the palette's "operate on current context"
  pattern; theme has a row there today. They stop being segmented controls to
  navigate and become commands to invoke.
- **Method, respond-mode and test-rule method stay** as horizontal regions. They
  are contextual and frequent — part of editing a rule, not a setting.

Seven regions remain, and two controls leave the keyboard model by leaving the
screen rather than by being excluded from it.

**D-4 stands.** With Welcome and Dashboard becoming regions, the palette is no
longer needed for navigation outside a workspace, so making it render there is
unnecessary and the "inert outside `AppView::Workspace`" resolution is kept.

### D-16's left rail is deleted, not wired

`shell/left_rail.rs` implements a navigation surface nothing renders, duplicating
what the tab bar does. Adding it to the keyboard model would mean designing
navigation for a surface users never see. **Delete it, and amend MK-027
accordingly** — an unrendered implementation in the tree is a specification that
misleads, which is the defect D-16 records rather than a thing to preserve.

### Implementation slices

Independently reviewable, in order. The first is a live bug fix and is deliberately
first.

1. **Capture status and the dialog default action.** Read `Status`; add the
   dialog region with `Enter` activating Test Rule's Run and the confirm dialog's
   Proceed. Closes D-6 and the audit's correctness defect.
2. **Regions and the cursor**, with Tab/Shift+Tab, applied to the tab bar and the
   mode picker — replacing `mode_picker_selected` with the general model and
   proving it against a surface that already works.
3. **Welcome and Dashboard** — the flat-list case, which unblocks scenarios 1 and 2
   at their first step.
4. **The sidebar tree**, including collapse and expand.
5. **The segmented regions** and the trace list.
6. **Theme and audience mode to the palette**; delete the left rail.

### What the tests must validate

Per the owner's testing rule, tests validate **this design**, not the code that
happens to be written:

- the four edge cases, for every region: clamp on shortening, empty region has no
  cursor and `Enter` is a no-op, entering a region resets the cursor, and whether
  the cursor can leave the visible scroll — **stated plainly where it can**;
- identity-stored cursors survive a list changing under them;
- a captured key event does not reach the application, and `Escape` still does;
- no `text_input` sets `on_submit`;
- every region's indicator is rendered when the cursor is in it.

### Acceptance

**A keyboard-only run of all six primary scenarios, driven by injected keys, with
no pointer click anywhere in it.** The audit's inventory is the checklist: every
control it lists must be reachable, or the design must say why it no longer
exists. On acceptance this RFC moves to `done/` with its mechanism executed —
which is the whole point, after three RFCs were returned from `done/` for the
opposite.

## Focus order — Routes screen (the most-used view)

```
Top bar identity (workspace switcher)
→ Server status / global actions
→ Left rail
→ Rule-set tree
→ Add rule set
→ Rule editor: URL operator
→ Rule editor: URL path
→ Method controls
→ Header rows (each row: name → op → value → row actions)
→ Body rows (each row: path → op → value → row actions)
→ Respond controls (mode tabs → status → delay → body)
→ Test rule
→ Right column (trace strip or rule inspector)
→ Bottom drawer trigger
```

Focus order on Trace, Settings, Scripts and overlays follows the same top-to-bottom, left-to-right reading order with no surprises.

## Screen-reader landmarks

| Region | Landmark label |
|---|---|
| Top bar | Workspace actions |
| Left rail | Main navigation |
| Rule-set tree | Rule sets and files |
| Rule editor | Rule editor |
| Live trace strip | Recent requests |
| Bottom drawer | Validation and save details |
| Dialog | The dialog's title |

Each landmark must be addressable by an iced accessibility role hint or, where iced lacks the affordance, a descriptive `aria`-equivalent annotation provided to snora.

## Discoverability of shortcuts

The command palette header displays the active shortcuts (`Ctrl/Cmd + K toggle`, `Esc close`) as chip annotations. Individual rows in the palette may also show their shortcut where one exists. New users who never read a tutorial will discover the keyboard surface via the palette.

The palette is opened from a visible button in the top bar (with the shortcut visible in its tooltip).

## Empty-state and error voice

Both error and empty-state copy must avoid blame and unhelpful generic language. The microcopy contract (MK-036) details the exact patterns; this RFC requires only that they be applied.

## Accessibility checklist (for every PR)

- [ ] Every icon-only button has an accessible label.
- [ ] Every status has text + icon.
- [ ] Focus is visible in light and dark mode.
- [ ] Main workflows are keyboard reachable.
- [ ] `Esc` closes the topmost overlay.
- [ ] `Ctrl/Cmd + K` opens the command palette.
- [ ] Colour is not the only severity/outcome signal.
- [ ] JA strings do not clip when the locale is switched.
- [ ] Text can scale to 200% without clipping.

## Acceptance criteria

- The shortcut catalogue is implemented in a single keyboard subscription module that maps key events to messages.
- A test (manual or automated) confirms every catalogue entry actually works.
- The palette is reachable from every workspace view.
- A locale-switch smoke test confirms JA strings do not clip any of the listed surfaces.

## Out of scope

- Custom focus-ring styling — handled in MK-022 component rules (the theme picks the ring colour; this RFC only requires it be visible).
- Specific shortcut conflicts on JA / non-US keyboards — flagged as a v2 follow-up.

## Status

**Status.** Proposed — the folder and the metadata field are correct, and the
paragraph below explains why an accepted design does not yet move this file.

**Returned from `done/` 2026-08-15** because its central mechanism had never been
built; the correction block at the head of this RFC records that in full.

**First-screen half shipped 2026-09-05** as milestone M9 (`74e7f75`): the mode
picker became keyboard-operable, which was MK-023's first-screen gap. **The
second screen then stopped a keyboard user**, recorded as D-12 (S1).

**External design accepted by the project owner 2026-10-31** — the § *External
design — the keyboard interaction model* section above. What was accepted is the
**design**, not an implementation: one model of regions and a cursor, Tab between
and arrows within, on the keys `text_input` never captures; cursors that store
identity where a list can change; the tree as a flattened visible sequence;
dialogs as regions whose cursor starts on the **safe** action; D-6 closed by
reading the capture status already supplied; theme and audience mode moved to the
palette; the left rail deleted.

**Implementation follows in six reviewable slices**, listed in that section.

**Slice 1 complete 2026-10-31** (`dbf947c`, CI run `41193214523` green on six
legs). It closed **D-6** by reading the capture status the subscription was
always handed, and gave both dialogs a default action on the safe side: Test
Rule's `Enter` runs the test, the confirm dialog's **cancels**. That also fixed
the live defect task 024's audit found, where `Enter` did nothing in either
dialog **for pointer users too** — recorded in the inventory above and now
closed. Evidence shows the confirm dialog raised over genuinely unsaved work
(*"Switch workspaces? Unsaved edits will be lost"*, status chip reading
**Unsaved (1)**) and the edit surviving the keystroke.

Slices 2–6 remain: regions and the cursor; Welcome and Dashboard; the sidebar
tree; the segmented regions and the trace list; the palette moves and deleting
the left rail (**D-16**).

**This RFC moves to `done/` when its mechanism has been executed**, not when the
design was agreed. That distinction is the whole reason it was returned from
`done/` in the first place, and the reason three other RFCs were returned with it.

> **A lifecycle note, and it is now the second instance.** Under
> `.git-exclude/rules/000-rfc-lifecycle-policy.md` the project has adopted the
> **5-folder variant**, in which `accepted/` means *"review complete; implementer
> may start"*. That is exactly this RFC's state as of 2026-10-31, and exactly
> MK-056's since 2026-08-02. **`rfcs/accepted/` does not exist**, and
> `scripts/check-rfcs.sh` requires only `proposed/`, `done/` and `archive/`, so
> the gate enforces the variant the project did not adopt. Recorded as finding B1
> of `.git-exclude/reviewed/2026-10-06-roles-and-rules-compliance-audit.md`, where
> the migration is described and still awaits authorization. Until then these two
> RFCs sit in `proposed/`, whose policy meaning — *"the implementer should not yet
> start work"* — is the opposite of their real state.
