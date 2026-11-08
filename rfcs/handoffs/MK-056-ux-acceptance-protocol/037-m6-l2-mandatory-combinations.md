# Dev-team task 037 — M6's four mandatory L2 combinations

**Issued:** 2026-11-04
**Issued by:** programme architect
**Authorized by:** project owner — pending
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

## 9. Required review-request content

Per workflow §9.2: summary; the four runs with their recorded configurations;
what `scripts/ux/` now does and what was deleted; every finding with its
measurement; gate status; review focus.

Commit. **The push rule was reconciled on 2026-11-08** (ROADMAP,
*Release and delivery policy*): you may push **work you authored that has been
reviewed and approved** — so commit, request the review, and push once it is
accepted; an unreviewed or rejected change does not reach the remote. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry.
Commit messages: digest plus optional body within 400 characters.
