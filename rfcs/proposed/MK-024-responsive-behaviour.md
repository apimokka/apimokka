# RFC MK-024 — Responsive and window behaviour

**Status.** Proposed
**Tracks.** Window sizing, breakpoints, column widths, overflow rules.
**Touches.** Routes (three-column), Trace (two-column), Settings (form), every list-bearing surface.

> ## Status correction, 2026-08-12
>
> **Returned from `done/`.** This RFC was recorded as `Implemented (v0.6.0)`
> from the beginning. That was never true of the code.
>
> **This RFC's breakpoints were never implemented.** The status field said
> `Implemented (v0.6.0)` from the beginning; the code has never contained the
> behaviour it describes. Corrected by project-owner decision after the gap was
> found during M6 preparation.
>
> **It has been returned to `proposed/`**, not archived. Archiving would mean
> the work will not happen; this is an accepted design still intended, now
> newly implementable. `scripts/check-rfcs.sh` refused the status edit while the
> file sat in `done/` — the folder is the source of truth for state under
> MK-000, and a not-implemented RFC cannot live there. The checker MK-051 added
> is what caught it.
>
> **How it was found.** Writing the L2 scripted-verification task, the intent
> was to check layout against the four breakpoints below. Verification of the
> code found:
>
> - `crates/app/src/main.rs` declares no window settings — no default size, no
>   minimum;
> - the application observes window size nowhere — no `iced::window`, no
>   `Event::Window`, no `iced::widget::responsive`, no width-based branching
>   anywhere in `crates/app/src`;
> - snora 0.25.2 provided no responsive layer either, so nothing supplied the
>   behaviour from below.
>
> **Confirmed empirically on 2026-08-04.** The mode picker was captured at
> 880×700, 1024×720, 1280×800 and 1920×1080. The card measured ~645 physical
> pixels wide at *every* size; only the surrounding whitespace changed. Evidence:
> `.git-exclude/release-evidence/2026-08-04-mk056-l2-mode-picker-sizes/`.
>
> **Why the record said otherwise.** M1's repository-truth repair reconciled RFC
> *indexes* against files on disk. It did not reconcile RFC *claims* against
> code, so a status field asserting a feature that was never built survived it
> intact. That is the same class as R1's finding B3, and this is its last known
> instance.
>
> **What is now possible.** snora 0.28's `snora::responsive_render` supplies the
> layout's available width, which is the capability this RFC needed and never
> had. Implementing the breakpoints is deliberately **not** scheduled: it is a
> behaviour change, M6 is about to validate behaviour, and snora has asked for
> our thresholds as the evidence deciding whether they ship breakpoint behaviour
> themselves — thresholds we can only choose honestly after sessions show what
> users do at small sizes. See MK-058 §7 and its resolution 4.
>
> ### Correction, 2026-08-15 — `responsive_render` is not usable on our path
>
> The paragraph above overstates what 0.28 unlocked, and this is a blocking
> constraint rather than a caveat.
>
> `snora::responsive_render` hardcodes the **engine** renderer
> (`src/responsive.rs`: `Responsive::new(|size| { … crate::render::render(layout) })`).
> It takes no `Tokens`, and there is no `design::responsive_render` — `grep
> responsive src/design*` returns nothing at 0.29.0.
>
> apimokka renders through `snora::design::render(layout, &tokens)`, which MK-058
> Phase 3 adopted specifically to fix the `high_contrast_dark` modal dim — a
> shipped accessibility defect. **So adopting `responsive_render` as written
> would regress M8's accessibility fix.** Responsive layout and design chrome are
> mutually exclusive in snora as shipped.
>
> **Consequence for scheduling:** the blocker on this RFC is no longer only
> "thresholds need session evidence." Even with thresholds in hand, implementing
> it requires one of:
>
> 1. snora shipping `design::responsive_render` taking `&Tokens` — requested
>    2026-08-15; or
> 2. hand-rolling `iced::widget::responsive(|size| design::render(build(size.width), &tokens))`.
>    This appears viable — `design::render` is the only public entry point, as
>    `render_with_style` is private — but it is undocumented and unverified.
>
> Option 2 is the fallback if snora declines. Neither is scheduled, and this
> constraint should be settled **before** any implementation task is written, not
> discovered inside one.
>
> Related: snora's `design` feature requires `widgets`
> (`design = ["widgets", …]`), so a design-path consumer cannot build engine-only.
> We have zero `snora::widget::*` call sites and compile the crate regardless.
>
> The design below is unchanged and remains the intended target. It is a
> specification awaiting implementation, not a record of work done.

## Summary

apimokka is a desktop app, but laptop screens vary widely and users often dock it alongside another window (their app under test). The layout must remain usable from `1024 × 720` up to ultrawide; the Routes screen is the most affected because it has the most columns.

## Target window sizes

| Size | Status |
|---|---|
| 1280 × 800 | Comfortable default — the design's reference frame |
| 1024 × 720 | Minimum acceptable — right column may collapse |
| 1920 × 1080 | Common — center editor gains extra width |
| ≥ 2560 px wide | Rare — center editor caps at a comfortable max width; extra space becomes padding |

## Breakpoints

| Width | Behaviour |
|---:|---|
| ≥ 1280 px | Full three-column Routes (rule-set tree + editor + trace strip / inspector) |
| 1100–1279 px | Right column narrower (260 px); editor remains comfortable |
| 900–1099 px | Right column hidden behind toggle (`∿`); editor takes the full right side |
| < 900 px | Single body column; left rail switches to icon-only; trace detail opens as a drawer |

The breakpoints are pixel widths, not iced-`Length` ratios. Implementation reads the window width and switches the layout variant — there's no responsive grid library to lean on.

## Column widths (Routes screen)

| Region | Recommended | Min | Max |
|---|---:|---:|---:|
| Left rail | 72–96 px (icon-only) or 120 px (with labels) | 72 | 160 |
| Routes sidebar (rule-set tree) | 260–300 px | 240 | 360 |
| Center editor | flexible | 520 | 900 |
| Right trace strip / inspector | 260–320 px | 240 | 360 |
| Bottom drawer | 30–40% of window height | 200 px | 60% |

When the center editor would exceed its max, the extra horizontal space becomes left-edge padding rather than allowing each input to stretch absurdly wide.

## Overflow rules

- **Long file paths** truncate in the middle (`…`) with a tooltip showing the full path and a copy action.
- **Long URLs in lists** truncate in the middle; the full value is shown in the detail panel.
- **JSON bodies** use a scrollable monospace area with both horizontal and vertical scroll.
- **Tables** must not force the whole app to scroll horizontally — they scroll inside their own container.
- **Tree views** scroll vertically only.

## Component-level behaviour

### Top bar
- The workspace identity truncates its workspace name with `…` if the bar would otherwise wrap.
- The action group (Save / Reload / Restart / Start/Stop) collapses into a single overflow menu below 900 px window width.
- The view controls (trace toggle, theme toggle, command palette) and locale picker are always visible.

### Routes sidebar
- File names truncate mid-string with full-value tooltip on hover/focus.
- The "+ Rule set" button stays pinned to the bottom of the sidebar.

### Rule editor
- The WHEN and RESPOND columns share width 50/50 at the reference frame.
- Below 1100 px window width the WHEN/RESPOND split can switch to stacked (WHEN on top, RESPOND below) — but only when the right column is also hidden, so the editor takes the full body width.

### Bottom drawer
- Drawer minimum height is 200 px; below that it should not open.
- Drawer maximum height is 60% of window height.

## Out of scope

- Mobile / tablet layouts — apimokka is desktop-only.
- iced `Length::Fill` / `Length::FillPortion` arithmetic — implementation detail.

## Acceptance criteria

- The Routes screen is usable at exactly 1024 × 720, 1280 × 800, and 1920 × 1080.
- The trace strip toggle hides the strip cleanly; the rule editor reflows to take the new width.
- Long paths and URLs never push the layout horizontally.
- The top bar's action group survives a window-width sweep from 800 px to 2560 px without overlapping or wrapping awkwardly.
