# Dev-team task 043 — the headless UI level

**Issued:** 2026-11-11
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Governing rules:** `.git-exclude/rules/project-instructions-gui-common.md`
§ *Test Optimization Rules* (updated by the owner 2026-11-11)
**Design:** [MK-056](../../accepted/MK-056-ux-acceptance-protocol.md) decision 3,
the layered verification model — amended 2026-11-11 to name this level
**Audit:** `.git-exclude/reviewed/2026-11-11-gui-rules-update-compliance-audit.md`

---

## Evidence plan, stated before the work

| Question | Level | Evidence |
|---|---|---|
| Can iced's own headless harness lay out *our* views, at a chosen size, and tell a clipped control from a drawn one? | spike | one test asserting D-31's and D-33's own geometry, red before the fix commits, green after |
| Does layout measure identically on Linux, macOS and Windows? | spike | the same assertions run on all six CI legs, reported as a fact, not assumed |
| Do our layout invariants hold across size × theme × locale? | **headless (new)** | the matrix test, on every CI run |
| Does the real window still look right? | real-window capture | worst case plus one baseline, not six frames |

## 1. Why this exists

The suite has **logic-and-state tests** and **real-window captures**, and
nothing between. The owner's updated rules name the missing level — the
framework's own headless harness, for interaction and **layout properties**,
across every text size, language, theme and window size, on every CI run.

**iced ships it and we have never used it.** `iced_test 0.14.0` matches our
pinned iced, takes our exact element type
(`Element<Message, Theme, iced_renderer::Renderer>` — no signature changes),
and offers `simulator(view).with_size(…)`, `find`/`click` over targets with
**`visible_bounds()`** (`click` fails with `TargetNotVisible`), `tap_key`,
`typewrite`, and `snapshot(&theme)` with `matches_image` / `matches_hash`.

**Two defects this programme spent a fortnight on are unit-testable there:**
**D-31** (PATCH and DELETE clipped out of the Method row at 880×700) and
**D-33** (the header value field at zero width while the rule matched on
`application/json`). Both cost a capture run, a review and a task. Neither
would survive a headless `click("DELETE")` at 880×700.

## 2. Step 1 — the spike, and it may come back "no"

**Before building anything, answer three questions and report them.**

1. **Does it see what we need?** Write one test: build the rule editor's view
   for the mock's rule 3, simulate **880×700**, and assert that every method
   choice and the header value have non-zero `visible_bounds()`. **Run it
   against `c046485~1`** (before D-31's fix) and against `HEAD`: it must fail
   on the first and pass on the second. If it passes on both, the harness
   cannot see clipping and this whole task changes shape — say so and stop.
2. **Is it deterministic across platforms?** Layout depends on font metrics
   and `snapshot` on rasterisation. Run the spike test on **all six CI legs**
   and report what is identical and what is not — in particular whether any
   assertion depends on a *system* font (we use `Font::MONOSPACE` in places).
   **Report the answer; do not paper over a difference with a tolerance.**
3. **What does it cost?** Build time and test time for `iced_test` as a
   dev-dependency, and whether it drags a GPU backend into `cargo test`.

**Stop after step 1 and request review.** If the answers are good the rest is
mechanical; if they are not, the design decision returns to me rather than
being absorbed into an implementation.

## 3. Step 2 — the matrix, if the spike holds

Layout invariants, asserted headlessly across **size × theme × locale**:

- **nothing a user must read or click is clipped**: for each of the surfaces
  M6's mandatory combinations exercise, every label and control the view
  declares has visible bounds with non-zero width and height;
- **the editor's groups survive**: all six method choices, both respond modes,
  both condition rows' four and five controls — the D-31/D-32/D-33 family, now
  at the cheapest level that can hold them;
- **sizes**: 880×700 and 1280×800 at least; **themes**: all four presets;
  **locales**: EN and JA, because Japanese is where labels are longest.

Keep the matrix declarative — one table of cases, one assertion body — so a
new size or locale is a row, not a copy.

**It runs on every CI run.** Wire it as ordinary `cargo test`; nothing new in
the gate script unless the spike says the matrix needs its own leg.

## 4. Step 3 — the small gaps in the rig, from the same audit

All five are in `scripts/ux/` and none is more than a few lines:

1. **The focus check must precede every keystroke**, not run once per launch.
   Today `ux_assert_focus` is called at `run-configuration.sh:166` and in two
   probes, while `ux_wtype_key`/`_text`/`_ctrl_key` send keys unchecked. Put
   the assertion inside those wrappers. The nested compositor holds exactly
   one client, so the practical risk is low — which is why this is a few lines
   and not a redesign, and why it should still be done: the rule exists
   because "only one window can have focus here" is an assumption that rots.
2. **Isolate the directories.** Set throwaway `XDG_CONFIG_HOME`,
   `XDG_STATE_HOME`, `XDG_CACHE_HOME` and `XDG_DATA_HOME` under the run's own
   scratch directory. The application reads none of them today — **assert that
   in the run log rather than leaving it implied**, so the day production
   integration adds file I/O the rig is already honest.
3. **Record the backend** as a tenth field in `RESULTS.md`: *Wayland (nested
   niri)*, beside the renderer that is already measured.
4. **Choose the nested window by pid**, as the host window already is.
5. **Encode the repeat limit**: if the same capture set has to be run a second
   time, say so in the review request; a third needs the architect first.

## 5. Non-change scope

- **No application behaviour, no appearance, no pixel.** This task must not
  change `crates/app`'s views, theme or regions. If a layout invariant fails
  for a real reason, **report it as a finding** — do not fix it here.
- **No new capture runs** beyond what the spike needs. The matrix is the
  point; captures are what it replaces.
- `update` stays a pure reducer. `Cargo.lock` gains a dev-dependency only.

## 6. Acceptance criteria

1. Step 1's three answers reported, with the red-then-green evidence for the
   known defect.
2. If the spike holds: the matrix exists, runs in `cargo test`, and covers
   size × theme × locale for the surfaces named in §3.
3. §4's five rig items done, or reported with a reason.
4. Gate green, exit code read directly; no `crates/app` behaviour change.
5. The review request opens with the evidence plan, as this handoff does.

## 7. Known risks

- **The harness may not model clipping at all.** `visible_bounds()` may report
  layout bounds rather than visible ones. That is what step 1 exists to find,
  and a "no" is a perfectly good result — report it with the test that shows
  it.
- **Fonts.** If layout depends on a system font, the matrix may be
  Linux-only. Say so and we will run it where it is deterministic rather than
  weakening the assertions until they pass everywhere.
- **Scope pull.** A headless harness invites rewriting the whole suite. It
  does not: this task adds a level, it does not move what already works.

## 8. Required review-request content

The evidence plan; step 1's three answers; what the matrix covers and what it
deliberately does not; §4's five items; gate status; review focus.

Commit. Under the reconciled push rule (ROADMAP, *Release and delivery
policy*, amended 2026-11-08) you may push work you authored once it is
reviewed and approved. Check `git log origin/mockup-v0-01..HEAD` first and say
what a push would carry. Commit messages: digest plus optional body within 400
characters.
