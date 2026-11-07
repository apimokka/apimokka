# RFC MK-056 layer L2 — scripted GUI verification

Scripts here drive the live `apimokka` GUI to produce repeatable evidence
for the mechanical parts of M6. They require an actual display and cannot
run in the canonical gate — kept separate from `scripts/check-*.sh` and
never wired into `scripts/check-release-gates.sh`.

**Compositor:** niri. **Geometry:** a *nested* niri instance (the winit
backend), sized and scaled **at launch, inside its own config** —
`default-column-width { proportion 1.0; }` and `animations { off }`, plus
an explicit output `scale` — never by floating or resizing the
*application's own* window after the fact
(`.git-exclude/rules/project-instructions-gui-common.md`'s own rule: doing
that to a running window on a shared desktop rearranges the layout and the
capture then exposes other sessions). What *is* floated and resized on the
host, by `lib.sh`'s own helpers, is the nested compositor's disposable
container window — never the application under test.

**Input:** `wtype`, speaking `zwp_virtual_keyboard_manager_v1` against the
nested compositor's own socket — the only path that reaches apimokka's
native-Wayland surface at all. `xdotool` is not used anywhere in this
directory: it is structurally unable to see a native-Wayland window
(verified, recorded in `rfcs/accepted/MK-056-ux-acceptance-protocol.md`'s
own decision 3), and every one of MK-056 decision 7's four mandatory
combinations is a keyboard-only L2 check, so there is nothing here for a
pointer tool to do. `xdotool` remains this project's pointer-input route
*elsewhere* — the X11/XWayland backend, per the GUI evidence rules file
above — which is a different host configuration from the one these
scripts run against.

**Capture:** `niri msg action screenshot-window` returns *before* the PNG
is written. Every capture in this directory goes through
`lib.sh`'s `ux_capture`/`ux_capture_step`, which delete the target first,
poll for the file to exist and for two consecutive size reads to agree,
then — for a capture expected to differ from the one before it — compare
hashes and retry the drive step (not reinterpret the frame) on a match,
up to twice, before stopping the run rather than looping a third time.
Three capture attempts were lost to the unguarded version of this in task
035; the same discipline correctly told an input stall from a frozen
application in task 036.

## Layout

- **`lib.sh`** — generic rig primitives: nested-compositor lifecycle,
  `wtype` wrappers, the capture-with-proof discipline, measured-geometry
  readers, and a focus assertion (`ux_assert_focus`, run before every
  drive step — the GUI evidence rules' own requirement that `wtype` types
  into whatever holds focus, not into a window you merely chose).
- **`app-nav.sh`** — apimokka-specific navigation built on `lib.sh`: select
  an audience mode at the picker, open the mock's default workspace, run
  a command-palette row, set the theme or locale, open Settings or the
  save-diff drawer, show the region cursor. Every state-changing function
  captures a checkpoint frame after acting and retries on a dropped key
  before returning — "assert, then continue" (GUI evidence rules).
- **`run-configuration.sh`** — runs one configuration end to end: starts
  the nested rig at the requested size/scale, launches apimokka, drives it
  to the requested mode/locale/theme, captures the Routes workbench,
  Settings, and the save-diff drawer, tears down, and writes a
  `<name>-RESULTS.md` with the **measured** geometry (nested output's own
  report, cross-checked against the PNG's own physical size) and the
  renderer in use (`/proc/<pid>/fd`'s open `/dev/dri/render*` descriptors
  for `wgpu`; their absence means iced's `tiny-skia` software fallback).
- **`run-mandatory-combinations.sh`** — runs MK-056 decision 7's four
  mandatory combinations with their specific parameters, including
  combination 3's temporary host-output-mode switch (see the script's own
  header for why 200% scale needs it, and that the switch is restored and
  the restore verified, not merely assumed).
- **`probe.sh`** — a rig smoke test: start a small nested compositor,
  launch the app, send one Tab, assert the two captures around it differ.
  Run this once on a new host before trusting a real run.
- **`discover-tab-order.sh`** — for a screen `app-nav.sh` does not yet
  cover: sends one Tab at a time from first launch and captures after
  each, so a human can read the images and hand-write the next navigation
  helper from what Tab index reaches what control. Produces evidence for a
  decision; it does not assert pass or fail itself.

## Usage

```sh
cargo build -p apimokka --locked

# once, on a new host:
bash scripts/ux/probe.sh

# the four mandatory combinations:
bash scripts/ux/run-mandatory-combinations.sh \
  .git-exclude/release-evidence/<date>-task037-m6-l2-mandatory

# one configuration, re-run after a fix:
bash scripts/ux/run-configuration.sh \
  --name combo1 --width 880 --height 700 --mode guided --locale ja \
  --theme light \
  --out-dir .git-exclude/release-evidence/<date>-task037-m6-l2-mandatory/combo1-japanese-880x700
```

Screenshot filenames are derived from `--name` and the step
(`<name>-00-launch.png`, `<name>-01-routes.png`, …), so a re-run after a
fix can be diffed image-by-image against the original run without
renaming anything by hand. Checkpoint frames used only to verify a drive
step landed go to `_checkpoints/` under the same `--out-dir` and are not
themselves evidence.

## Known limitations, stated rather than hidden

- **The application is externally unobservable except for its window
  title.** There is no signal beyond `App::title()` that a script — or
  any assistive technology — can read. Which control has focus, whether a
  panel is clipped, whether contrast is adequate, are all pixels only.
  This is "iced 0.14 exposes no accessibility tree" seen from the
  scripting side, and belongs in the evidence as an inherited production
  requirement, not a quirk of this harness.
- **Pointer behaviour is not exercised here.** MK-056 decision 7's four
  mandatory combinations are all keyboard paths; this directory has no
  pointer-input client and does not need one for them.
- **`app-nav.sh`'s sequences assume a fresh launch, opening the mock's
  default workspace, with locale/theme at their own defaults (En, Light)
  beforehand.** Every driver here launches a new app instance per
  configuration for exactly this reason, rather than reusing one across
  configurations — state carried over from a previous run would make a
  navigation helper's own assumptions (what step N lands on) silently
  wrong.
- **Do not modify the application to make a check pass.** If a
  configuration cannot be driven or asserted as designed, that is a
  finding to report, not a reason to change `crates/app/src`.
- **No `XDG_*` isolation.** Confirmed at source (`crates/app/src`,
  `crates/model/src`): apimokka reads no `XDG_CONFIG_HOME`/
  `XDG_STATE_HOME` or any other environment-derived directory — every
  launch starts from the same hardcoded mock state regardless of the
  host's own configuration, so there is nothing to isolate.
- **`--show-cursor` under a High Contrast theme may capture no visible
  ring at all.** `theme::card_selected_style`'s own `is_high_contrast`
  branch (`crates/app/src/theme.rs`) sets only a low-alpha (10-18%)
  background wash and no border, unlike the solid 2px coloured border the
  non-HC branch draws — found live running task 037's combination 2.
  `ux_app_show_region_cursor`'s `soft=yes` mode (used automatically by
  `run-configuration.sh` for `hclight`/`hcdark`) still sends the Tab and
  still captures the frame, but logs a NOTE instead of retrying the
  capture to death — the identical frame is the finding, not a dropped
  key. See the function's own comment in `app-nav.sh`.
