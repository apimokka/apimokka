# Dev-team task 037 — M6's four mandatory L2 combinations

**Issued:** 2026-11-04
**Issued by:** programme architect
**Authorized by:** project owner; round 1 ran 2026-11-07 (`1634aed`) and was
reviewed — **the task stays open for §10**
**Design:** [MK-056](../../accepted/MK-056-ux-acceptance-protocol.md) decision 7,
and its **2026-11-04 resolution** of the acceptance criterion that named L2
**Companion:** `003-m6-session-pack.md` §0 (withdrawn 2026-09-17, **restored
2026-11-04**), `002-m6-participant-recruitment.md`
**Cycle:** C3 — the last prerequisite before sessions are booked

---

## 1. Purpose, and what it unblocks

MK-056's acceptance criterion requires *"L2 recorded pass for every mandatory
combination in decision 7"*. That bullet sat **under decision** from
2026-09-17, because the four combinations were unreachable: `wtype` could not
drive the application past Welcome. **MK-023 removed every wall that note
named, and D-12 closed 2026-11-03**, so the criterion stands as written and
all four combinations are L2 work.

**They have never been run.** Every capture this programme has taken — M8,
M10, tasks 022 through 036 — was at **1280×720**. Two of the four
combinations are specifically about the **smallest supported window
(880×700)** and one is about **200% scale**. Those geometries have never
been rendered, let alone recorded.

On acceptance, booking is unblocked and sessions carry comprehension only.

## 2. The four combinations (MK-056 decision 7)

| # | Combination | Why it is mandatory |
|---:|---|---|
| 1 | **Japanese at 880×700** | Text expansion and clipping |
| 2 | **High-contrast theme, keyboard-only input** | Focus indication against high-contrast borders |
| 3 | **Guided mode at 200% scale** | The mode carrying the most inline hints |
| 4 | **Expert mode at 880×700** | Expert shows every control at once; density fails here first |

Record per run, per decision 7 — **a finding without its configuration is not
evidence**: commit SHA, platform, theme, locale, audience mode, window size,
input method, **renderer**, entry state.

Renderer matters and has its own history: no apimokka evidence recorded it
until 2026-09-26, and snora's 0.51.0 letter reports a contrast defect that
existed **on the software renderer only**. Record it the way the recent
captures do — `/proc/<pid>/fd` showing the open `/dev/dri/renderD*`
descriptors for wgpu.

## 3. Bring `scripts/ux/` onto the recipe that actually works

**This half is as important as the runs**, because it is what makes them
repeatable by someone who is not you.

`scripts/ux/` still describes a world this programme left behind. Its README
says keyboard synthesis is *"attempted via `xdotool`"* and instructs the
reader, on a native-Wayland host, to **"not install `ydotool`/`wtype`"** —
while every capture since M8 has been driven by `wtype`, and
`run-configuration.sh`'s own key path still shells out to `xdotool`, which
cannot see a native-Wayland surface at all.

Bring them up to date:

- **`wtype` on the nested compositor's socket** is the input path. `xdotool`
  stays only if you can say what it is still for; if it is for nothing,
  delete it rather than leaving two paths and a stale warning.
- **The nested-niri rig** is the geometry mechanism — `default-column-width
  { proportion 1.0; }` and `animations { off }` in the nested config, and
  `niri msg output <name> scale <s>` for scale, which is also how
  combination 3's 200% is produced.
- **Every capture step must prove its own output landed**: delete the target,
  issue `niri msg action screenshot-window --path`, poll for the file and for
  its size to settle, then compare the hash with the previous frame. The
  command returns *before* the PNG is written; three capture attempts were
  lost to that in task 035, and the same discipline correctly caught input
  stalls in task 036. A frame that did not change must read as a dropped key,
  never as a frozen application.
- The full recipe, with the two causes it fixes, is in
  `.git-exclude/release-evidence/2026-11-03-task035-m12-slice10-architect-capture/RESULTS.md`
  §0. **That file is outside version control; this task is the chance to move
  what matters into `scripts/ux/` where it belongs.**

## 4. Expect findings, and report rather than fix

880×700 is **40 % narrower than anything this application has been rendered
at in evidence**, and Japanese is text this application has never been seen
carrying at any size. Clipping, overflow and wrapped-to-unreadable layouts
are the *expected* output of combinations 1 and 4 — that is why MK-056 made
them mandatory.

**Report them. Do not fix them in this task.** A layout change now would
change what a participant sees, and MK-056 forbids running sessions against a
build we then change — so the fixes are a separate, scheduled task, and the
owner decides which are worth making before booking. One recent precedent to
keep in mind: task 033 reported a footer as clipped at 1280×720 and the
finding was **withdrawn** when the geometry turned out to be wrong. Measure
the window before believing the frame.

## 5. Non-change scope

- **No application changes at all** unless a run cannot proceed without one —
  and then stop and report, rather than changing what a participant will see.
- **No new defect fixes.** D-24's scope half, D-28 and the two open
  readability *preferences* are all after M6 by decision.
- **No session may be booked off this task** — that is the owner's, with
  task 002.

## 6. Required evidence

One directory per combination under `.git-exclude/release-evidence/`, each
with a `RESULTS.md` carrying decision 7's nine recorded fields and the
**measured** geometry (the nested output's reported logical size and scale,
and the PNG's physical pixel size), not the geometry the commands requested.

For each combination, enough frames to show the application's primary
surfaces at that configuration — at minimum the Routes workbench, Settings,
and one dialog — plus any surface where something clips, wraps badly, or
overflows.

Combination 2 additionally: the keyboard cursor visible on at least one
control per screen it covers, since "focus against high-contrast borders" is
the thing it exists to check.

## 7. Acceptance criteria

1. Four runs, four directories, decision 7's fields recorded per run.
2. Geometry measured and stated, never assumed from the commands issued.
3. `scripts/ux/` describes and performs the recipe that works, with the
   capture-landed check in code rather than in prose.
4. Findings reported, not fixed, with the measurement that supports each.
5. Gate green, exit code read directly. No application behaviour changed.

## 8. Known risks

- **880×700 may be unusable rather than merely tight.** If a primary surface
  cannot be operated at the smallest supported size, that is an S1-shaped
  finding and the owner needs it before recruiting, not after. Say so plainly
  and early rather than completing all four runs first.
- **200% scale may exceed the host output.** 880×700 at 2.0 needs 1760×1400
  physical; the host is 2133×1200 logical at 1.2. Combination 3 does not
  specify the window size — pick one that fits, state it, and say why.
- **The nested rig drops keys** roughly one in forty. The hash check catches
  it; retry the key rather than reinterpreting the frame.

## 9. Round 2 — added 2026-11-08; **complete 2026-11-08**

**TASK COMPLETE, 2026-11-08.** §9.1 landed as `7439269` and was accepted: combination 2
records a pass on `wgpu`, so **all four of decision 7's mandatory combinations now
have a recorded L2 pass** and MK-056's L2 acceptance criterion is met. The one
finding, **D-36 (S2)** — text-field focus is colour-only in all four presets — is
**task 042** and did not hold the pass. Review:
`.git-exclude/reviewed/2026-11-08-task037-s9-1-review.md`.

**Status, 2026-11-08:** `43d3cad` delivered **§9.2** (the editor and Test Rule
at 880×700, which opened **D-31 S1** and **D-32 S2** — task 040, and through
its row walk **D-33 S1** and **D-34 S3** — task 041; all four now closed) and
**§9.3** (the frame manifest, plus task 039's directory cleaned); both
**accepted**, see
`.git-exclude/reviewed/2026-11-08-task037-round2-review.md`. **Items 2 and 3
below are therefore done.** **§9.1 — item 1, with item 1b — is the whole of
what is left**, and **the host was rebooted on 2026-11-08, so it is no longer
blocked.** It is the one mandatory combination that has never recorded a pass,
and MK-056's acceptance criterion needs it before sessions are booked.

Round 1 produced three passes, one fail and one combination that never
exercised its own purpose. **Round 2 (2026-11-08, `43d3cad`) delivered §9.2
and §9.3**; §9.2's own findings became D-31/D-32 (task 040) and D-33/D-34
(task 041), all four now closed. What is left:

1. **Combination 2, re-run.** Its fail was **D-29**, closed by task 038
   (`3ac0e20`): the keyboard cursor paints a visible ring in both
   high-contrast presets now, and **D-30**, closed by task 039 (`7f26ba1`),
   gave selection its own width-based cue. Re-run the combination and record
   what decision 7 asked for — focus indication against high-contrast borders
   — with the cursor, the selected state, and an unselected row in frame.
   **This run must use `wgpu`**, because decision 7 records the renderer among
   its nine fields and round 1 was `wgpu`; a software re-run is not
   comparable. **It is blocked until the host is rebooted** — the running
   kernel's module tree was replaced by an upgrade on 2026-11-08, which is why
   task 039's frames are `tiny-skia`. **Cleared 2026-11-08:** the host now
   runs `7.3.3-1-cachyos` with its module tree present, the probe rig passes,
   and task 041's nine runs were all `wgpu`. Nothing blocks this item.
1b. **A scroll step for the rig, added 2026-11-08 by task 041's review.** At
   880×700 the editor's Headers and Body cards sit below the fold and the rig
   cannot reach them, so task 041's condition evidence had to be taken at
   880×1150. Nothing is unreachable in the *application* — `App::scroll_snap_
   target`'s `Region::Editor` arm snaps the WHEN/RESPOND scrollable to the
   cursor — so use the application's own mechanism: move the region cursor
   into the WHEN column and let `snap_to` scroll, then capture. That makes a
   claim at the mandated 880×700 showable rather than arguable, and it is the
   same keyboard path a participant has.

2. ~~**The rule editor at 880×700**, in combinations 1 and 4.~~ **Done** by
   round 2 and extended by tasks 040 and 041, whose captures cover both
   locales at 880×700, 880×1150 and 1280×800. Round 1 captured
   Routes in its empty state in both narrow runs, so the editor — WHEN and
   RESPOND side by side, decision 7's *"Expert shows every control at once,
   and this is where that density fails"* — has never been rendered below
   1280. This needs no GPU and can be done first.
3. ~~**Evidence-directory integrity, from task 039's review §4.**~~ **Done**
   — `scripts/ux/manifest.sh` and its self-test, the latter now inside
   `scripts/check-release-gates.sh` (task 040). That capture
   left twelve undocumented files in its directory, nine of them duplicates of
   three images and four naming a theme their pixels contradict. Two things,
   both in `scripts/ux/`:
   - **a per-run manifest** recording every kept frame's name, MD5 and
     measured ground pixel;
   - **the run fails** if two kept frames share an MD5, or if a frame's ground
     pixel does not match the theme the run requested.

   This is the third capture-integrity trap in three tasks — the asynchronous
   screenshot write, geometry taken from the request rather than measured, and
   now stale files surviving in the output. Each time the rig proved a file
   existed and never that it was *this* run's frame of *this* state. The
   manifest is what closes the class, not the instance.
   **Also clean `2026-11-08-task039-d30-selection-capture/`**: delete or
   rename the twelve undocumented files so the directory and its `RESULTS.md`
   agree.

Acceptance for what remains: §7's criteria again, plus **combination 2 passing
on its own terms** on `wgpu`, with the cursor, the selected state and an
unselected row in frame, and item 1b's scroll step used wherever a surface sits
below the fold at the mandated size.

## 10. Required review-request content

Per workflow §9.2: summary; the four runs with their recorded configurations;
what `scripts/ux/` now does and what was deleted; every finding with its
measurement; gate status; review focus.

Commit. **The push rule was reconciled on 2026-11-08** (ROADMAP,
*Release and delivery policy*): you may push **work you authored that has been
reviewed and approved** — so commit, request the review, and push once it is
accepted; an unreviewed or rejected change does not reach the remote. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
