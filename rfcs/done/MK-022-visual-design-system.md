# RFC MK-022 — Visual design system

**Status.** Implemented (v0.6.0)
**Tracks.** Tokens, typography, colour semantics, icon vocabulary, component rules.
**Touches.** Every visible surface; every other RFC in the redesign series uses these tokens by name.
**Supersedes.** MK-019 (visual polish and design tokens).

## Summary

Define a single token vocabulary — spacing, type, radius, colour semantics, icon meanings, component variants — that every screen draws from. The vocabulary is what keeps the redesign coherent; without it each screen invents its own values and the result feels busy and inconsistent (the failure mode of the v0.1–v0.5 mockup).

## Visual direction

The product expresses **"developer control without IDE heaviness."**

**Do:**
- Quiet background
- Distinct cards and panels, but fewer borders
- Clear typographic hierarchy
- Small icons used consistently
- Status chips grouped and simplified
- Dense data lists only where users benefit from scanning

**Don't:**
- Excessive dividers
- Multiple competing accent colours
- Status chips without text
- Large decorative effects unsupported by iced (backdrop blur, complex gradients)
- Tiny form controls

## Layout tokens

| Token | Value | Usage |
|---|---:|---|
| `space.1` | 4 px | Inline icon/text gap |
| `space.2` | 8 px | Compact field grouping |
| `space.3` | 12 px | Form row gap |
| `space.4` | 16 px | Card internal padding |
| `space.5` | 20 px | Section spacing |
| `space.6` | 24 px | Major panel padding |
| `radius.sm` | 6 px | Inputs, small chips |
| `radius.md` | 10 px | Buttons, compact cards |
| `radius.lg` | 14 px | Primary cards and dialogs |
| `radius.xl` | 18 px | Welcome hero and major panels |
| `border.thin` | 1 px | Panel and input outlines (used sparingly) |
| `shadow.soft` | subtle | Dialogs, top-level cards |

## Typography scale

Restrained scale; iced's text rendering blurs small size differences.

| Token | Size | Weight | Usage |
|---|---:|---|---|
| `display` | 30–34 px | Semibold | Welcome hero only |
| `title` | 22–24 px | Semibold | Screen title |
| `section` | 17–18 px | Semibold | Card heading |
| `body` | 14–15 px | Regular | Default UI text |
| `bodyStrong` | 14–15 px | Semibold | Row labels, selected items |
| `caption` | 12–13 px | Regular | Hints, metadata |
| `mono` | 13–14 px | Regular | Code, paths, JSON, route strings |

A font family will be chosen by the visual designer; system defaults are acceptable for the v1 implementation. The mono family must be a true monospace (e.g. JetBrains Mono, Berkeley Mono, system mono).

## Colour semantics

The visual designer picks exact hex values. The semantic mapping below is contract-stable.

| Semantic | Meaning | Light | Dark |
|---|---|---|---|
| `surface.base` | Main app background | low-contrast neutral | near-black neutral |
| `surface.panel` | Panels and cards | white/off-white | elevated dark |
| `surface.raised` | Dialogs, menus | highest surface | highest surface |
| `border.muted` | Boundaries | subtle grey | subtle dark grey |
| `text.primary` | Main text | high contrast | high contrast |
| `text.secondary` | Metadata, hints | medium contrast | medium contrast |
| `accent.primary` | Main action, active nav | brand accent | brighter brand accent |
| `semantic.success` | Matched, saved, running | green family | accessible green |
| `semantic.warning` | Reload pending, validation warning | amber family | accessible amber |
| `semantic.danger` | Error, destructive action | red family | accessible red |
| `semantic.info` | Info, fallback | blue/cyan family | accessible blue/cyan |

Both light and dark mode ship in v1. All semantic colours must remain legible on both backgrounds.

## Icon and status vocabulary

Every icon must be paired with a label or screen-reader-accessible name. Colour is never the only signal (ABDD — see MK-023).

| Concept | Text label | Suggested icon meaning |
|---|---|---|
| Server running | Running | filled circle / play |
| Server stopped | Stopped | square / stop |
| Reload pending | Reload pending | refresh |
| Restart required | Restart required | power |
| Error | Error | alert |
| Saved | Saved | check |
| Unsaved | Unsaved | dot / pencil |
| Match | Matched | check |
| Fallback | Fallback | return arrow |
| Miss | Miss | hollow circle |
| Warning | Warning | alert triangle |
| Info | Info | information |

## Component rules

### Buttons

| Variant | Use |
|---|---|
| Primary | Create workspace, Add rule, Save all, Run test |
| Secondary | Open workspace, Reload, Restart, Duplicate |
| Ghost | Cancel, low-risk inline actions |
| Danger | Delete rule, discard changes |
| Icon button | Replay, pause, clear, move up/down — must include tooltip and accessibility label |

The implementation should expose these as helper functions (`primary_btn`, `secondary_btn`, `ghost_btn`, `danger_btn`, `icon_btn`). Each helper takes a label or icon plus an `on_press` message. Custom hover/pressed styling is not required for v1; iced's built-in `button::{primary,secondary,danger,text}` styles are acceptable defaults provided the theme palette is tuned.

### Chips

Short, grouped, glyph + text.

Examples: `Running`, `Unsaved`, `Reload pending`, `3 issues`, `Trace paused`.

If more than three global chips would appear in the top bar, collapse lower-priority detail into a status menu (see MK-027).

### Cards

Use cards for meaningful conceptual units:
- WHEN URL path
- WHEN headers
- WHEN body
- RESPOND
- Settings section
- Trace match detail
- Validation group

Avoid nesting cards more than one level deep.

Card visual treatment: subtle elevation (soft shadow) on a neutral surface; **no hard border** by default. A border is only acceptable when the card sits directly on `surface.base` with no other visual separation.

### Inputs

- Labels appear **outside or above** fields, not placeholder-only
- Every input shows: label, value, optional hint, validation message when applicable
- Disabled / read-only states are visually distinct without colour reliance
- The field's pixel height is consistent across inputs in a row

### Empty states

Every empty surface (no rule selected, no events yet, no scripts, no validation issues) has a paired message + optional CTA. See MK-036 for the microcopy table.

## Acceptance criteria

- Every screen file imports tokens from a single module (`crate::theme` or equivalent); no magic numbers for spacing, sizing, or radius anywhere in screen code.
- Both `Theme::Light` and `Theme::Dark` render correctly with no hardcoded theme references in helper functions.
- The full button variant set (primary / secondary / ghost / danger / icon) is exposed and used consistently — Create actions use primary; destructive use danger; Cancel uses ghost.
- Every icon button has an accessible label.
- Cards have a subtle shadow, not a 1px grey border, on the default surface.

## Out of scope

- Per-screen detail (handled in MK-025..MK-032)
- Animation and transitions (deferred; iced 0.14 supports them but the team will not build them in v1)
- Branding (logo, splash) — designer's choice
