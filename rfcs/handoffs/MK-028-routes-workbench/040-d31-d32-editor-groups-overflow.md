# Dev-team task 040 — D-31 and D-32: the editor's groups overflow at 880×700

**Issued:** 2026-11-08
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Design:** [MK-028](../../done/MK-028-routes-workbench.md) § the rule editor;
[MK-041](../../done/MK-041-layout-density.md) § density
**Found by:** task 037 round 2 — combinations 1 and 4 at 880×700, the
combinations decision 7 added because *"Expert shows every control at once,
and this is where that density fails"*
**Review:** `.git-exclude/reviewed/2026-11-08-task037-round2-review.md` §2–§3

---

## 1. The defects

Both are one mechanism: a `row` of fixed-width items inside a card narrower
than their total width. iced clips the overflow; nothing indicates it.

**D-31 — the Method group loses PATCH and DELETE. S1.**
`method_card` (`crates/app/src/screens/routes/rule_editor.rs:540`) builds
`row(btns)` over `regions::editor_method_group`'s **six** entries with no wrap
and no horizontal scroll. At 880×700 the card is 308 logical px and renders
**Any / GET / POST / PUT** only — confirmed in both combinations' own frames
(`combo1-04-editor.png`, `combo4-04-editor.png`), and at 1280 all six appear.

- **A pointer user cannot select PATCH or DELETE at this width**, and nothing
  says the list is truncated: no ellipsis, no scrollbar. The four visible
  choices read as the whole set.
- **A keyboard user can still set them, blind.** Each button is wrapped by
  `cursor_wrap_group(app, EditorField::Method, i, …)`, so cursor indices 4 and
  5 exist and `Enter` dispatches `RuleSetMethod("PATCH")` onto a control that
  is not drawn — the ring vanishes into the clip.

**S1** by MK-056 §5: *"an unreachable control is observed fact, not opinion —
one sighting is sufficient, and S1 applies."* 880×700 is one of decision 7's
**mandatory** combinations, the application sets no minimum window size, and
primary scenario 3 is *"edit a rule's conditions and verify the change with
Test Rule"* — the method is one of those conditions. §8 forbids deferring it,
so **this blocks M6 session booking** (recruitment is unaffected).

**D-32 — the RESPOND mode group collapses. S2.**
`respond_card` (`:766`) builds `row![inline, serve_file]` of two `mode_tab`
buttons. In Japanese at 880×700 the second collapses to a **~29 logical-px
column** with 「ファイル提供」 stacked vertically at the panel edge, beside
「インラインテキスト」 which itself wraps; English Expert wraps it to two
lines ("Serve / file"). Still clickable, so it degrades rather than blocks —
**S2**.

## 2. The fix, and iced already has it

**`iced_widget 0.14`'s `Row::wrap()`** (`row.rs:158`, returning `Wrapping`)
lays a row onto further lines when it cannot fit. That is the whole change for
both groups: every choice stays visible and reachable at any width, with **no
breakpoint, no new abstraction, and nothing from MK-024's unbuilt responsive
machinery**.

Verify before you commit to it: that `Wrapping` keeps `spacing`, that it
converts into an `Element` where the current `row(...)` does, and that the
wrapped group still measures sensibly inside `card(...)`. If any of that does
not hold, say so with what you found and propose the next option rather than
reaching for a horizontal scrollable — a scrollbar that only appears on hover
hides the same choices from the same user.

**Do not add a breakpoint** and do not swap the Method group for a `pick_list`
at narrow widths: that is a second control with a second interaction model for
the same data, and MK-024 is the RFC that would have to decide it.

## 3. Walk the rest of the editor, do not guess it

The editor's doc comment names *"its two multi-item groups (`Method`,
`RespondMode`)"*, and those are the two that failed. **There are thirteen
`row` constructions in that file.** Walk them all at 880×700 and report which
overflow — fix only those, and say in the review request which ones you
checked and found sound. The pattern this programme keeps paying for is a
repair applied where the defect was reported rather than everywhere it lives
(D-25 came back a second time that way).

**The Test Rule dialog is the in-repo counter-example**: its own method row
shows all five methods at 880×700 (task 037 round 2 §4). Read how it differs
before changing the editor's.

## 4. Non-change scope

- **No region, cursor, or `activation_message` changes.** The cursor already
  reaches all six method items; the defect is that two of them are not drawn.
  `regions.rs` and `app/region_cursor.rs` stay untouched.
- **No theme change.** D-29 and D-30 settled the appearance layer; this is
  layout.
- **No new widget abstraction** unless `wrap()` genuinely cannot serve, and
  then it is proposed, not built.
- `update` stays a pure reducer; no behaviour changes, only layout.

## 5. The guard, and why it is a capture this time

There is no layout engine in the test suite, so no unit test can assert "all
six are drawn at 880 logical px". Do not fake one. Instead:

1. **A structural test** that the Method group is built from
   `editor_method_group`'s full six and the RESPOND group from both modes —
   cheap, and it catches a future edit that drops an entry;
2. **captures at 880×700** in both combinations showing all six methods and
   both respond modes legible, with the manifest passing;
3. **state the width at which the old layout began clipping**, measured, in
   the review request. It is the number that tells us whether anything else in
   the application is near the same edge.

## 6. Required evidence

Re-capture combinations 1 and 4 at 880×700 — Japanese Guided and English
Expert — with `*-04-editor.png` showing the Method group complete and the
RESPOND group legible in both languages. The frame manifest must pass. If the
host has been rebooted by then, run these on **wgpu** and say so; if not,
software frames are acceptable for a layout claim, as they were for round 2.

## 7. Also in this visit — wire the integrity check into the gate

`scripts/ux/manifest-selftest.sh` needs no display and runs in under a second,
but nothing runs it. Add it to `scripts/check-release-gates.sh` alongside the
four existing *self-test then checker* pairs. **`check-release-gates-self-test.sh`
records the expected step list**, so both files change together, and the gate's
own self-test must still pass.

The guard against false evidence should itself be guarded. Leaving it unrun is
the same shape as the defect it was built to catch.

## 8. Acceptance criteria

1. All six method choices and both respond modes are drawn, legible and
   operable at 880×700 in both locales.
2. The rest of the editor's rows walked, with the finding stated for each.
3. §5's structural test exists; §6's captures exist and their manifests pass.
4. The manifest self-test runs inside the release gate, and the gate's own
   self-test passes.
5. Gate green, exit code read directly; `update` still pure; no theme or
   region change.

**On acceptance D-31 and D-32 close, and M6's session booking is unblocked**
as far as this repository is concerned — task 037 §9.1 and recruitment are the
remaining items, and neither is yours.

## 9. Known risks

- **`wrap()` may change the card's height**, pushing the Headers/Weight cards
  further below the fold at 700 px. That is a density observation for M6, not
  a reason to avoid the fix; capture it and say so.
- **Japanese labels are the long ones.** If a wrapped group still looks
  cramped in Japanese, report it with the measurement rather than shortening a
  translation — microcopy is MK-036's, not this task's.

## 10. Required review-request content

Per workflow §9.2: summary; what changed and why `wrap()` (or what replaced
it); the thirteen rows walked, with the verdict for each; the measured
clipping width of the old layout; the captures and their manifests; the gate
change; gate status; review focus.

Commit. **The push rule was reconciled on 2026-11-08** (ROADMAP,
*Release and delivery policy*): you may push **work you authored that has been
reviewed and approved** — so commit, request the review, and push once it is
accepted. Check `git log origin/mockup-v0-01..HEAD` first and say what a push
would carry. Commit messages: digest plus optional body within 400 characters.
