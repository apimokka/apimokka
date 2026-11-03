# Dev-team task 036 — D-21, D-25 and (separably) D-27

**Issued:** 2026-11-04
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** [MK-022](../../done/MK-022-visual-design-system.md) § colour roles;
the remedy shape is **D-13**'s and **D-15**'s, applied a third and fourth time.
**Cycle:** before C3 — this is the appearance work M6's sessions must not
rediscover.

---

## 1. Purpose, and why it comes before the sessions

Three visible defects remain on surfaces a participant meets constantly, and
all three were found by eye or by capture rather than by a test. MK-056
requires findings to be re-tested against a recorded build identity, so
anything that changes what a participant sees must land **before** the
sessions, not between them.

**Parts A and B are defects and are the task. Part C is separable** — take it
only if the project owner authorized it.

## 2. Part A — D-21: the feedback banner's text fails AA in every preset

`theme::banner_style` paints `ep.warning.weak.color` — an iced-derived slot —
and sets `text_color` to `ep.background.base.text`, a colour chosen for a
different surface. Measured 2026-11-03:

| Preset | Banner text | Muted on banner | Floor |
|---|---:|---:|---|
| Light | 2.54 | 1.42 | 4.5 |
| Dark | 1.31 | 2.36 | 4.5 |
| HC Light | 1.94 | 1.05 | 4.5 |
| HC Dark | 1.25 | 1.46 | 4.5 |

**This is the surface that announces what the application just did and carries
the only visible Undo affordance.** In both dark presets the undo offer is
effectively invisible.

**Remedy — the same shape as D-13 and D-15, which is why this is mechanical:**

1. Take the banner's fill **and** its text from snora tokens that carry a
   guarantee *for that pairing* — not from an iced-derived slot, and not from
   a token documented for another surface.
2. **Measure the pairing. Never inherit the token's own claim.** D-13's own
   record notes a snora pairing measuring 4.53 against its documented floor;
   that is why this rule exists.
3. Assert **all four presets** in `crates/app/src/theme/tests.rs`. The harness
   is already there (`presets()`, `ratio()`, `muted_surfaces()`); the banner
   needs to join whatever list the muted-surface tests walk, so that the next
   colour change cannot skip it — our contrast tests assert our roles on our
   surfaces, and **none of them looked here**, which is how this survived.
4. Both text roles on the banner: the message text and any muted text on it.

**Do not change the muted role itself.** Whether muted text should move from
AA to AAA is a *preference*, not a defect, and it belongs to M6's readability
probe — see §5.

## 3. Part B — D-25: two surfaces name the wrong action

1. **The undo banner labels a strategy edit "Moved rule."**
   `HistoryEntry::banner_key` (`app.rs`) maps `RootSetting { .. }` — what a
   `RuleSetSetStrategy` edit records — to `Key::UndoRuleMoved`, a fallback
   several unrelated edit kinds share.
2. **The Create-workspace confirmation is titled "Switch workspaces?"**
   `ConfirmAction::CreateWorkspace` maps to `Key::ConfirmSwitchWorkspace`, so
   pressing *Create workspace* with unsaved edits is asked about *switching*.
   Right in substance — the open workspace is being left — wrong in the action
   it names.

Both are one-line mappings with an i18n key each. **Add the keys rather than
reusing a near-miss**, in both locales, and check the rest of each mapping for
other fallbacks that land on a label describing a different action: these two
were found by eye, so assume they are not alone and say what you found.

## 4. Part C — D-27, separable: the drawer's action row is clipped

`shell::view` gives the save-diff drawer `SheetSize::Ratio(0.32)` — 230
logical px — and its content is taller, so with the cursor on `Discard` the
button is cut by the window edge at the required 1280×720 (architect capture
`d1-02`). Reachable, half visible.

Two honest remedies, and **they are not equal**:

- **A taller sheet** (or a layout that keeps the action row out of the
  scrollable, as the drawer header already keeps `Close`). Small, local,
  testable at 1280×720.
- **A precise scroll-into-view** replacing D-5's relative-offset
  approximation. That is the larger change: it would retire D-5's recorded
  limits everywhere rather than here, and it is **out of scope for this
  task** — if you believe it is the only correct answer, say so and stop.

Take the first. If it cannot be made to hold at 1280×720 without moving
something else, report rather than cascade.

## 5. Non-change scope

- **Do not raise the muted role to AAA**, and **do not touch MK-059's 50
  deliberately-retained 12.0px sites.** Both are preferences about
  readability that **M6's own probe exists to answer on evidence** (MK-059
  resolution 3, carried into MK-056). Changing either now would destroy the
  subject of that probe and answer it on our judgement instead — which is the
  one thing MK-059 declined to do. *(This supersedes items 2 and 3 of the
  architect's 2026-11-03 recommendation, withdrawn 2026-11-04.)*
- No keyboard-model changes. MK-023 is `done/` and this task must not reopen
  it: `regions.rs` and `app/region_cursor.rs` should not need editing at all.
  If Part B's label work touches a region's `activation_message`, stop and say
  why.
- `update` stays a pure reducer.

## 6. Required tests

- **All four presets**, both banner text roles, asserted at the AA floor —
  joining the existing muted-surface walk rather than a new one-off test.
- A test that the banner's fill and text both come from **snora tokens**, in
  the shape `light_and_dark_muted_surfaces_fill_with_their_snora_tokens`
  already uses, so a future edit cannot quietly reintroduce an iced-derived
  slot.
- One test per corrected label mapping, asserting the key, not the string.
- Part C: a test that the drawer's action row is inside the sheet's own
  height at 1280×720 — or, if that is not expressible, say so plainly and let
  the frame carry it.

## 7. Required evidence

Keyboard-only is **not** required here; this is an appearance task. What is
required is **all four presets, captured**, at a measured 1280×720, showing:

1. the error banner, the undo banner and the success notice, each with its
   text legible — the undo banner in **both dark presets** is the frame that
   matters most;
2. the corrected labels: a strategy edit's undo banner, and the
   Create-workspace confirmation;
3. Part C, if taken: the drawer with the cursor on `Discard`, whole.

**Use the capture recipe that works** — it is recorded in
`.git-exclude/release-evidence/2026-11-03-task035-m12-slice10-architect-capture/RESULTS.md`
§0 and it is the reason this cycle's last capture succeeded:
`default-column-width { proportion 1.0; }` and `animations { off }` in the
nested compositor's own config, the nested output's scale set to the host's,
and — the one that cost three attempts — **a capture step must prove its own
output landed**: `niri msg action screenshot-window` returns *before* the PNG
is written, so delete the target first, then poll for the file and for its
size to settle, then compare its hash with the previous frame. A frame that
did not change is then visibly a dropped key rather than a frozen
application.

Record backend, renderer, build SHA, and the **measured** window size.

## 8. Acceptance criteria

1. Banner text meets AA in all four presets, measured, with both roles
   asserted in `theme/tests.rs`.
2. The two labels name their own actions, with new keys in both locales, and
   a report of anything else the mapping check found.
3. Part C, if authorized: the action row whole at 1280×720.
4. Gate green, exit code read directly; `update` still pure; MK-023's
   keyboard model untouched.
5. Evidence per §7, four presets, measured geometry.

## 9. Known risks

- **A token that passes on one surface and fails on another.** Measure each
  pairing you actually render; do not generalise from one preset to four.
- **The banner's fill is semantic** — warning-coloured for a reason. If the
  only token pairing that meets AA loses that semantics, report the tension
  rather than silently making every banner neutral.
- **Part B's new keys must exist in both locales.** A missing key is the
  defect D-17 already was once.

## 10. Required review-request content

Per workflow §9.2: summary; the measured before/after for all four presets and
both roles; the two label mappings and anything else the check found; Part C
if taken; the tests; the evidence with measured geometry; gate status; review
focus.

Commit, and **do not push without authorization**. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
