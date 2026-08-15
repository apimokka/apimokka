# RFC MK-058 — snora 0.28 adoption

**Status.** Proposed
**Tracks.** Stabilization roadmap M8 — snora 0.28 adoption.
**Touches.** The `snora` dependency, `main.rs`'s theme wiring,
`shell/view.rs`'s six render calls, and the appearance M6 will validate.

## Summary

Adopt snora 0.28.0, upgrading from 0.25.2. The jump is **non-breaking** across
every hop — 0.25.2 → 0.25.3 → 0.26.0 → 0.27.0 → 0.27.1 → 0.28.0 — verified by
the snora architect against published sources by set comparison of public names
and signatures.

Three of the migration guide's four adoption phases apply; the fourth has zero
call sites here. The change remains small: one manifest line, one theme wiring
point, and six render calls in a single file.

It is proposed ahead of M6's live runs because it fixes a real accessibility
defect in a preset we ship and advertise, and because validating an appearance
we intend to replace would waste the milestone whose entire purpose is
validating appearance.

**This RFC was originally written against 0.27.0.** 0.28.0 superseded it the
same day, carrying two capabilities that came from our own field feedback. The
adoption plan is unchanged — target `"0.28"` instead of `"0.27"` — but 0.28
unlocks work this RFC deliberately does not do (§7).

## Problem and motivation

### A shipped accessibility defect

In the **`high_contrast_dark`** preset, snora's modal dim has been **completely
invisible**: the dim was a hardcoded 40% black, and 40% black over that preset's
pure-black background composites to pure black. Modals have had no visual
modality signal at all — in the preset whose entire purpose is maximum
legibility.

We ship that preset. `README.md` advertises four themes including it as an
accessibility feature; MK-050 made high-contrast theming a headline outcome; and
**MK-056's mandatory combination #2 is "high-contrast theme with keyboard-only
input"**, so M6 is about to test the exact surface that is broken.

The M6 preparation gate established the principle: known defects are fixed
before sessions, so participant time finds unknown ones. This is now a known
defect, and its fix is six call sites in one file. The snora architect
independently advises prioritising this phase over the others for exactly this
reason.

### Timing is close to free

M6's L3 sessions are blocked on participant recruitment. M6's L2 live run has
not happened either — its infrastructure is built and reviewed but unrun. Doing
this now costs nothing on the critical path, and it means **both** L2's
screenshots and L3's sessions record the appearance that will actually ship.

Doing it after L2 would invalidate L2's screenshot evidence. Doing it after L3
would devalue the sessions.

## Sizing — measured, not estimated

| Phase | Guide's framing | Our actual surface |
|---|---|---|
| 1 — version bump | one line | **one line** in `Cargo.toml` |
| 2 — theme emission | "highest value per line changed" | **one wiring point** — `main.rs:16` `.theme(App::theme)`; `App::theme()` (`app.rs:545`) already derives `Tokens` per `ThemeChoice` |
| 3 — dialog card and modal dim | optional | **six `snora::render` calls, all in `crates/app/src/shell/view.rs`** |
| 4 — chrome geometry | "most call sites to touch" | **zero** — `snora::widget::*` is used nowhere |

Phase 4 is inapplicable because this application builds its own chrome —
`shell/top_bar.rs`, `left_rail.rs`, `tab_bar.rs` — and passes it into
`AppLayout`. The snora architect has recorded this engine-only adoption pattern
as their only downstream data point on it.

**MSRV:** rustc ≥ 1.88 required from 0.25.3 onward, unchanged at 0.28. This
workspace declares 1.91.

## Goals

1. Be current on snora, on a non-breaking upgrade.
2. Fix the invisible `high_contrast_dark` modal dim.
3. Bring stock iced widgets onto the same token palette as snora's primitives.
4. Ensure M6 validates the appearance that will ship.
5. Change no behaviour — this is appearance and dependency only.

## Non-goals

- Redesigning any screen, layout, or interaction.
- Adopting `snora::design::widget::*` (Phase 4) — no call sites.
- Implementing responsive breakpoints on `responsive_render` (§7).
- Rewriting L2's scripted verification around the new identifiers (§7).
- Changing theme choices, token values, or the four presets we ship.
- Behaviour, tests, or the `WorkspacePort` boundary.

## Decision

### 1. Phase 1 — upgrade to 0.28, and verify the no-change guarantee

```toml
snora = { version = "0.28", features = ["widgets", "design"] }
```

snora guarantees rendered output is unchanged with existing call sites
untouched. **Treat that as a claim to test, not a fact to assume.** Capture
screenshots of the principal screens in all four presets before and after the
bump alone, with no other phase applied. Any visual difference at this point is
a snora bug, and the handoff asks for it to be reported.

**One exception to read deliberately.** Through 0.27.1 everything added was
`design`-gated. 0.28's **stable surface identifiers are unconditional** — the
only thing in the whole jump that is not opt-in. An `Id` has no rendering
effect, so appearance is still guaranteed, but anything that walks or snapshots
the widget tree will now see labels that were not there before.

Verified for this codebase: we have no `iced_test`, no `widget::Id` usage, and
nothing that snapshots a widget tree — every `snapshot` reference in
`crates/app/src` is our own `WorkspaceSnapshot`. **The unconditional identifiers
are inert here.** Recorded so the exception is not mistaken for a risk we
overlooked.

### 2. Phase 2 — theme emission

**This corrects an existing inconsistency, not a cosmetic preference.**
`ThemeChoice::iced()` (`app.rs:77-99`) returns iced's *own* `Theme::Light` and
`Theme::Dark` for two of the four presets, and a snora-token-derived custom
palette only for the two high-contrast ones. So in Light and Dark — the presets
most users see — stock iced widgets draw from iced's built-in palette while
everything around them draws from snora tokens. **Two palettes in one window.**
Only the accessibility presets are internally consistent, which is backwards.

Wire `snora::design::theme(&Tokens)` into iced's `.theme()` hook so stock
`text_input`, `pick_list`, `scrollable`, and the window background follow the
same palette across all four presets.

The snora architect has confirmed this is precisely the condition
`design::theme` was built to remove, and that they had no downstream instance of
it before ours.

**Expect a visible change in Light and Dark**, and do not mistake it for a
Phase 1 guarantee violation — that is why Phase 1 is verified separately and
first.

**Verification:** switch to `high_contrast_dark` and confirm stock controls
change too. If only snora's own primitives change, the theme is not reaching
iced.

### 3. Phase 3 — dialog card and modal dim

Replace all six `snora::render(layout)` calls in `shell/view.rs` with
`snora::design::render(layout, &tokens)`. This is the phase that fixes the
invisible modal dim.

In the two light presets the card is distinguishable **by its border only** —
its fill is bitwise identical to the page background by the token data's own
design. Expected, not a defect to chase.

### 4. Phase 4 — explicitly out of scope

Zero call sites. Recorded so a future reader does not conclude it was
overlooked.

### 5. Verify every preset, not just the default

The handoff names this as the item most likely to be skipped and most likely to
surface a problem, and records that the defects found during snora's own
milestone were nearly all preset-specific and nearly all in high contrast.

All four presets are checked for each phase adopted. A pass recorded only
against the default is not a pass.

### 6. Feedback — sent, answered, and what remains owed

Our field feedback was sent and has been answered in full
(`reply-to-feedback-2026-08-04.md`, bundled with the 0.28 handoff). All four
requests received a disposition; three shipped:

| Request | Disposition | Shipped |
|---|---|---|
| Width-responsive layout | Width exposure **accepted**; breakpoint behaviour deliberately deferred pending evidence | `snora::responsive_render`, 0.28.0 |
| Assistive-technology position | **Accepted** — position stated with an explicit reconsideration trigger (iced exposes an accessibility API) | docs, 0.27.1 |
| Focus visibility | Already documented; **discoverability fixed** with a consumer-facing guide. Interim wrapper **declined** | 0.27.1 |
| GUI testing affordances | Stable identifiers **accepted**; a test harness **declined** as a firm non-goal | 9 identifiers + per-toast, 0.28.0 |

**A correction we owe.** Our feedback stated the focus limitation was
"discoverable only in a migration guide's 'not available' section." That was
wrong: it is documented in `contributing/semantic-accessibility.md`,
`contributing/accessibility-checklist.md`, and the API docs. We inferred a claim
about snora's documentation as a whole from the two migration guides we had been
sent. The correct statement would have been scoped to the materials in hand.

The underlying observation still landed — snora treated our not finding it as a
navigation finding and shipped a consumer-facing accessibility guide — but the
overreach was ours and is recorded here rather than left in their correspondence
uncorrected.

**Still owed to snora**, after adoption:

- whether the Phase 1 no-visual-change guarantee held across all four presets;
- which parts of `AppLayout` we use and which we ignore;
- which breakpoint thresholds we pick, if we implement them, and whether width
  alone sufficed or we wanted height — this is the evidence that decides whether
  snora ever ships breakpoint behaviour;
- whether the identifier set is the right set, and specifically anything our
  scripted verification reaches for that has no identifier.

## 7. What 0.28 unlocks that this RFC deliberately does not do

Both new capabilities answer problems this programme found. Neither belongs in
MK-058, whose scope is the dependency and the appearance. Recorded so they are
not lost.

**`responsive_render` makes MK-024 implementable.** We found this week that
MK-024's four breakpoints were never implemented, and that neither the app nor
snora could observe window width. snora now supplies the width. Implementing
MK-024 is a behaviour change requiring its own decision — including the prior
question of whether MK-024's `Implemented (v0.6.0)` status should be corrected.
Note that snora prescribes no thresholds deliberately, and has asked for ours as
the evidence deciding whether they ever ship breakpoint behaviour.

**Stable identifiers materially change L2's approach.** Task 008's central
difficulty was that the application is externally unobservable except for its
window title. `snora-modal-dim`, `snora-dialog-card`, `snora-sheet-panel`,
`snora-header`, `snora-sidebar`, `snora-body`, `snora-footer`,
`snora-menu-backdrop`, `snora-toast-stack`, and per-toast identifiers are now
addressable, and are a compatibility surface — renaming one is a minor bump,
recorded in their versioning policy.

**Correction, 2026-08-04.** The claim above that identifiers make L2's dialog
and drawer verification tractable is **wrong**, and it was checked after M8
landed rather than before it was written.

`iced::widget::Id` lives inside iced's widget tree. It is not surfaced to the
compositor, to X11, or to any accessibility API — so a process driving the
application from outside with `xdotool` or `niri msg` cannot see one. L2 is
external scripted verification; the identifiers are internal. They do not help
it.

What they do help is **in-process** testing, and snora's own testing guide is
explicit about the shape it expects: assert against application state, keep
`update` pure, and accept that "what you cannot test with this approach is the
rendered pixel output." That is the pattern this codebase already follows in its
203 reducer tests.

So the identifiers change nothing for L2, and the useful follow-up is narrower
than stated: they would matter only if this project revisited an in-process
harness such as `iced_test`, which MK-053's DEC-006 judged immature and which
snora has suggested — without pressing — may be worth re-examining.

**Task 008 is still revisited after M8**, but for a different reason: M8 changed
the appearance, so L2's captures must record the post-M8 look, and M8's own
four-preset capture can share the same live session.

### snora 0.29.0 — adopted before M8's capture (Phase 5)

This RFC keeps its `0-28` filename. It is the snora-adoption RFC, and 0.28.0 is
what Phases 1–3 shipped, so the name stays historically accurate; 0.29.0 is a
one-line follow-on folded in here rather than given an RFC of its own, which
would be overhead disproportionate to a version bump with nothing to migrate.

snora sent an unprompted heads-up
(`.git-exclude/tmp/note-2026-08-15-upgrade-before-identifier-work.md`) after our
post-adoption report: **do identifier work on 0.29.0, not 0.28.0.**

**What changed.** On 0.28.0, `snora-dialog-card` is attached to the dialog's
centring wrapper — a container filling the window — so it resolves to
window-sized bounds and never to a card. 0.29.0 splits it: `snora-dialog` is the
centring container, `snora-dialog-card` becomes the actual card on the `design`
path we adopted in Phase 3.

**Why the warning matters.** These are plain strings. An assertion written
against 0.28.0's `snora-dialog-card` would, after upgrading, silently resolve a
much smaller element with no compile error and no failing test. snora shipped
the rename on the explicit premise that no consumer had adopted 0.28.0
identifiers — a premise our own report confirmed.

**Sequencing decision, superseded — see below.** The position first recorded here
was *upgrade after M8 closes, not before*, on the reasoning that a new minor
version underneath a nearly-closed milestone re-opens it for no benefit. That
reasoning rested on one unknown: whether 0.29.0 changes rendered appearance. The
migration guide had been referenced rather than bundled, so the answer was
snora's claim rather than something we had read.

**Sequencing decision as settled, 2026-08-15: upgrade to 0.29.0 first, then
capture.** We asked for the guide; snora supplied it
(`.git-exclude/tmp/app-team-snora-0.28.0-to-0.29.0.tar.gz`) with the appearance
question answered directly and by evidence, not assertion — a semantic diff of
the two tags with comments and formatting normalised away. Exactly two files
change behaviour: `overlay/dialog.rs`, where `.id()` moves to a different
element, and `identifiers.rs`. Everything else is doc comments, `rustfmt`, or
tests. `render_semantics`, the suite backing the no-visual-change guarantee, is
semantically unmodified across the span.

That inverts the decision. The capture's entire value is that it records the
version we ship. Capturing at 0.28.0 and bumping immediately after would produce
evidence for a version we are about to leave, and no pixel evidence for the one
we keep. The cost that argued for waiting — re-opening a closed milestone — is
one line in `Cargo.toml` with nothing to migrate, verified below.

**Nothing to migrate, verified rather than assumed.** The guide names one risk
the upgrade cannot catch mechanically: a test asserting on `snora-dialog-card`
does not fail, it silently starts resolving the styled card instead of the
window-sized wrapper. `grep -rn 'snora-dialog'` across the tree returns prose in
two RFCs and no code. There are no identifier assertions to re-read. MSRV is
unchanged at 1.88, and the public API is 157 items at both tags, compared as
sets.

**One caveat snora stated unprompted, and it argues for the capture rather than
against the upgrade.** The no-visual-change guarantee is *test-backed, not
pixel-verified*: `render_semantics` asserts composition — layer order, which
surfaces materialise, dismissal, RTL mirroring — and nothing in their CI compares
pixels. No downstream team has ever checked it visually. That makes the 0.28→0.29
comparison the one snora most wants and the one nobody has run, which is why
owner task 001's optional pass has been retargeted onto it. 0.28.1 sits between
and is documentation only.

**Our §2 correction changed their documentation.** snora had recorded the premise
behind the rename as "confirmed, then expired", the expiry being an identifier
task they inferred we had scheduled. `contributing/versioning-policy.md` now says
the premise holds, with the reason we supplied: `Id`s serve in-process harnesses
only. They also recorded the process lesson — ask the adopter, do not infer from
their last reported version — and fixed the twice-repeated bundling gap at its
root, with a script that walks every link to closure and refuses to produce the
tarball if one does not resolve inside it.

**Useful detail recorded now so it is not rediscovered:** three of the ten
identifiers will never render for this application — `snora-sidebar`,
`snora-footer`, and `snora-toast-stack` with its per-toast scheme — because we
populate none of those `AppLayout` slots. The seven that will:
`snora-menu-backdrop`, `snora-modal-dim`, `snora-dialog`, `snora-dialog-card`,
`snora-sheet-panel`, `snora-header`, `snora-body`.

## Sequencing

**M8 runs before M6's L2 live run and before L3 sessions.** Both are unrun,
which is what makes this cheap. Task 008's live run waits for M8, and should be
revisited in light of §7 before it happens.

M8 does not block participant recruitment.

## Acceptance evidence

All visual evidence is captured **at 0.29.0**, after Phase 5, per the sequencing
settled in §7.

Required:

- After Phase 2: stock controls following the palette, demonstrated by switching
  to `high_contrast_dark`.
- After Phase 3: a dialog distinguishable in all four presets, and specifically
  the `high_contrast_dark` modal dim now visible.
- `bash scripts/check-release-gates.sh` green.
- CI green on all three platforms — this changes a rendering dependency.
- Resolved `snora` family versions and checksums from `Cargo.lock`.
- The post-adoption report owed to snora under decision 6.

Optional, and **deliberately reduced to one comparison**, because each costs a
separate build and a second pass by a human whose time is this milestone's
binding constraint:

- **Phase 5, 0.28.0 → 0.29.0.** The comparison snora asked for and that no
  downstream team has ever run; their guarantee here is test-backed, not
  pixel-verified. This is the one to do.
- ~~Phase 1 alone, 0.25.2 → 0.28.0~~ — **deferred, not abandoned.** Still a valid
  isolated test of the same guarantee across the span Phase 1 crossed, but its
  value has decayed: that span is shipped, reviewed, and CI-green on three
  platforms, and snora's own `render_semantics` covers it. If it had broken
  appearance materially, Passes A and B would be reporting a problem. Recorded
  here so the omission is a decision rather than a gap.
- Confirmation that the unconditional identifiers changed nothing here — folded
  into the above; identifiers have no rendering effect, which is now established
  by semantic diff rather than inference.

## Alternatives considered

**Target 0.27.0 as originally drafted.** Rejected. 0.28.0 is current, the plan
is identical, and 0.27.0 lacks both capabilities that answer this programme's
own findings.

**Defer everything past R2.** Rejected. It leaves a known accessibility defect
in an advertised feature, and M6 would validate an appearance we intend to
replace.

**Phase 1 only.** Rejected. The modal-dim fix arrives specifically through
`snora::design::render`, which is Phase 3. Bumping without adopting would leave
the defect in place while implying currency.

**Adopt after M6, before R2.** Rejected. It invalidates L2's screenshots and
L3's findings, requiring a re-run of the work that is hardest to repeat.

**Fold §7's work into M8.** Rejected. Responsive behaviour is a behaviour
change gated on a separate governance question; L2 rework belongs to M6. Both
are unlocked by M8, neither is M8.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| Unconditional identifiers affect something unforeseen | Verified inert here — no widget-tree walking exists; Phase 1's before/after would catch a surprise |
| We are the only downstream data point on the token mapping | The snora architect states this plainly; treat unexpected geometry as a finding to report |
| Phase 2 changes appearance broadly | Adopted as its own step with per-preset verification before Phase 3 |
| The no-change guarantee does not hold | Phase 1 verified separately and first, so a violation is attributable |
| A rendering dependency behaves differently per platform | CI covers all three |
| §7's unlocked work quietly expands M8 | Named as non-goals and deferred explicitly |

## Review questions

1. Phase 2 is recommended for adoption now — not as visual unification, but
   because `ThemeChoice::iced()` currently renders Light and Dark with a split
   palette (decision 2). Accept, or limit M8 to Phase 1 + Phase 3?
2. Is M8 the right home, or should this fold into M6's preparation?
3. Is the Phase 1 before/after screenshot comparison proportionate?
4. §7 records two unlocked workstreams. Should either be scheduled now — in
   particular, does MK-024's `Implemented` status need correcting regardless of
   whether the breakpoints get built?

### Resolutions

Recommended by the programme architect and accepted with the design on
2026-08-04:

1. **Adopt Phase 2.** Not as visual unification but as a defect fix — Light and
   Dark currently render stock controls from a different palette than everything
   around them.
2. **M8, not folded into M6.** MK-056's non-goals exclude redesign from M6; this
   is a dependency and appearance change rather than UX measurement; and it
   gates M6's live runs, which reads more clearly as a prerequisite step than as
   a sub-item of the thing it gates.
3. **Keep the Phase 1 comparison**, and not only as courtesy to snora. Without
   Phase 1 baselines, an intended Phase 2 change in Light/Dark and an
   unintended Phase 1 regression are **indistinguishable**. The screenshots are
   what make attribution possible, and L2 will capture them anyway.
4. Split:
   - **MK-024's status is corrected regardless.** An RFC marked
     `Implemented (v0.6.0)` whose feature exists nowhere is a governance defect
     of exactly the class R1's finding B3 covered, and it is independent of
     whether breakpoints are ever built.
   - **Breakpoints are not scheduled now.** Implementing them is a behaviour
     change, and M6 is about to validate behaviour. snora has also asked for our
     thresholds as the evidence deciding whether they ship breakpoint behaviour
     — and we can only choose thresholds honestly after sessions show what users
     actually do at small window sizes. Sequence M6 first, then decide.
   - **Task 008 is revisited after M8**, before its live run, per §7.

## Status

Design accepted by the project owner on 2026-08-04, including the four
resolutions above. Under the four-folder lifecycle this RFC remains `Proposed`
until its implementation ships.

Acceptance of this design is recorded separately from authorization to
implement. The project owner authorized implementation on 2026-08-04, assigned
to the dev team. Authorization covers the scope defined here and nothing beyond
it; §7's unlocked workstreams remain out of scope and the non-goals stand.
