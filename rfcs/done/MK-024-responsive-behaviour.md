# RFC MK-024 — Responsive and window behaviour

**Status.** Implemented (v0.6.0)
**Tracks.** Window sizing, breakpoints, column widths, overflow rules.
**Touches.** Routes (three-column), Trace (two-column), Settings (form), every list-bearing surface.

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
