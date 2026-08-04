# RFC MK-056 layer L2 — scripted GUI verification

Scripts here drive the live `apimokka` GUI to produce repeatable evidence
for the mechanical parts of M6. They require an actual display and cannot
run in the canonical gate — kept separate from `scripts/check-*.sh` and
never wired into `scripts/check-release-gates.sh`.

**Compositor:** written and tested against **niri** (a scrolling-tiled
Wayland compositor), driven via `niri msg action` for window placement,
resize, and screenshot capture. Keyboard input synthesis (where used) is
attempted via `xdotool`, which can only reach a window if it is visible
through XWayland — a native-Wayland surface is invisible to X11 clients by
Wayland's security model, and that is expected, not a defect. Run
`probe.sh` first on any new host to find out which case applies before
trusting any keyboard-reachability result.

## Order of use

1. **`bash scripts/ux/probe.sh`** — launches apimokka once, reports whether
   xdotool can see and target its window on this host, then exits. Read
   its output before running anything else. If it reports
   `PROBE_RESULT=incapable` (likely, on native Wayland), that is a
   **finding to report, not a blocker to work around**: do not install
   `ydotool`/`wtype`, do not switch the app to XWayland, and do not
   otherwise change the environment to make input driving work. Report
   which configurations could be keyboard-driven and which could not, and
   why, in the evidence — an honest gap, not a silent skip.

2. **`bash scripts/ux/discover-tab-order.sh <max-tabs> <output-dir>`** —
   only if step 1 reports `capable`. No screen's Tab order is documented
   anywhere in this codebase (confirmed: no custom focus manager exists;
   traversal is iced/winit's default widget order). This sends one Tab at
   a time from first launch and screenshots after each, so the resulting
   images can be read to find which Tab index reaches a given control. The
   discovered sequence becomes a `--keys-file` (one `xdotool key` name per
   line) for `run-configuration.sh`. If the resulting order is unusable or
   non-deterministic (e.g. it changes between otherwise-identical runs),
   that is itself a finding — it means keyboard navigability is not
   reliably testable, which bears directly on M6's keyboard-reachability
   criterion — and belongs in the evidence, not in a retry loop.

3. **`bash scripts/ux/run-configuration.sh --name … --width … --height …
   --out-dir … [--keys-file … --expect-title-contains …]
   [--input-method pointer|keyboard]`** — runs one configuration: launch,
   resize (via `move-window-to-floating` + `set-window-width`/
   `set-window-height`, since niri only gives an arbitrary pixel size to a
   floated window), optional keyboard drive, screenshot at each step.
   Prints a `KEY=value` result block. The only machine-asserted outcome is
   `KEYBOARD_REACHABILITY=pass`, which requires `--expect-title-contains`
   and checks the window title afterward — `App::title()` is `"apimokka"`
   with no workspace open and `"{name} — apimokka"` once one is
   (`crates/app/src/app.rs:538-543`), the one state change this app
   exposes without an accessibility API. Every other claim (layout intact,
   focus visible, contrast, Japanese text expansion) is a screenshot for a
   human to judge, not something this script decides.

4. **`bash scripts/ux/run-all.sh <evidence-dir>`** — runs the five designed
   configurations (documented in the script's own header) and writes a
   `results.txt` alongside the screenshots.

## Re-running a single configuration after a fix

```sh
cargo build -p apimokka --locked
bash scripts/ux/run-configuration.sh \
  --name row-a-guided-ja-smallest --width 880 --height 700 \
  --out-dir .git-exclude/release-evidence/<date>-mk056-l2 \
  --input-method pointer
```

Screenshot filenames are derived from `--name` and the step
(`<name>-00-launch.png`, `<name>-01-resized.png`,
`<name>-02-after-keys.png`), so a re-run after a fix can be diffed
image-by-image against the original run without renaming anything by hand.

## Known limitations, stated rather than hidden

- **Window size is entirely externally driven.** The app has no window
  settings and never observes its own size (confirmed in the governing
  task doc). `run-configuration.sh` sets the OS-level window size; nothing
  in the app is aware this happened. A resize that doesn't crash the
  process is the only thing that can be asserted — layout correctness at
  that size is a screenshot for a human.
- **Text scale (200%) is deliberately not exercised.** The only way to do
  so is a niri *output* scale change, which rescales every window on that
  output while set, not just apimokka's, disrupting the operator's live
  desktop for the run's duration. Per review
  (`.git-exclude/reviewed/2026-08-04-mk056-scripted-gui-verification-review.md`
  §3.2), that disruption is not worth it: with no scale-aware rendering
  anywhere in the app, a 2.0x output scale at a normal window is close to
  1.0x scale at half the logical space — substantially the same layout
  regime the sub-900px row already exercises. Record this as a gap in the
  evidence, with the near-equivalence argument, and record a true
  200%-scale check as an inherited production requirement alongside
  MK-056 decision 6's other deferred accessibility items — not as
  something these scripts silently skip.
- **Pointer-method configurations are not click-driven by these scripts.**
  The RFC's machine-assertable list names keyboard reachability
  specifically, not pointer; pointer-designated configurations here supply
  screenshot evidence for human review of pointer-context rendering (hover
  states, click targets) without asserting click behaviour, since discovering
  reliable click coordinates has the same live-only-discoverable problem as
  Tab order, times the number of controls.
- **The application is externally unobservable except for its window
  title.** Confirmed per review (§3.3): there is no signal beyond
  `App::title()` that a script — or any assistive technology — can read.
  Which control has focus, whether a panel is clipped, whether contrast is
  adequate, are all pixels only. This is the same fact as "iced 0.14
  exposes no accessibility tree" seen from the scripting side, and belongs
  in the L2 evidence as an inherited production requirement, not treated
  as a quirk of this harness.
- **Do not modify the application to make a check pass.** If a
  configuration cannot be driven or asserted as designed, that is a
  finding to report, not a reason to change `crates/app/src`.
