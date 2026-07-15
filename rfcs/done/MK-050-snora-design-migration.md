# RFC MK-050 — Migrate to snora 0.25 Snora Design system

**Status.** Implemented (v0.10.0)
**Tracks.** Dependency upgrade snora 0.18.1 → 0.25.0; adoption of the new
Snora Design token system for the visual layer.
**Touches.** Root `Cargo.toml`, `theme.rs`, `app.rs` (ThemeChoice, Tokens
state), `screens/settings.rs` (theme picker), i18n.
**Follows.** MK-022 (visual design system), MK-037 (iced/snora handoff).

## Context

snora 0.25.0 introduces **Snora Design** — an optional, iced-free design token
layer (`snora-design` crate) plus an iced style bridge
(`snora::design::style`). It provides:

- A `Tokens` bundle: `palette` (semantic color roles), `spacing`,
  `typography`, `radius`, `focus`, `density`.
- Four built-in presets with **automated WCAG-AA contrast tests**:
  `light`, `dark`, `high_contrast_light`, `high_contrast_dark`.
- A semantic `Palette` with named roles: `background`, `surface`,
  `surface_raised`, `text_primary`, `text_secondary`, `text_muted`, `border`,
  `accent`, `accent_text`, `success`/`success_text`, `warning`/`warning_text`,
  `danger`/`danger_text`.

This directly serves apimokka's stated UI/UX and accessibility goals: the
project already maintains a hand-rolled `theme.rs` token layer (MK-022), but
its colors are derived ad-hoc from iced's `extended_palette()` with hardcoded
greys and no contrast verification. Adopting Snora Design gives us
contrast-tested tokens and, critically, **two high-contrast presets** that we
can expose as an accessibility option.

## Decision

Adopt snora 0.25 with the `design` feature. Store a `snora::design::Tokens`
value in `App` state, selected by the user's theme choice. Rewrite the color
derivation inside `theme.rs` to read from `Tokens` rather than iced's extended
palette — **without changing the public signatures** of the `theme::*` style
helpers, so the ~200 call sites across the screens are untouched.

### Why keep the `theme.rs` facade

The pilot button/card helpers in `snora::design::{button, card}` are useful but
shallow (primary/secondary/ghost/danger; surface/raised/selected). apimokka has
a richer set of surfaces (panel, chip, dialog, banner, accent strip, rail item,
segmented control, naked wrapper, parent-selected card). Rewriting every call
site to the snora helpers would be a large, risky change that loses
apimokka-specific styling. Instead, `theme.rs` becomes a thin adapter: its
helpers now pull colors/spacing/radius from the stored `Tokens` (via the style
bridge's `to_iced_color`) while keeping their existing names and signatures.

### ThemeChoice extension

```rust
pub enum ThemeChoice {
    Light,            // Tokens::light()
    Dark,             // Tokens::dark()
    HighContrastLight,// Tokens::high_contrast_light()  (NEW)
    HighContrastDark, // Tokens::high_contrast_dark()   (NEW)
}
```

The Settings → Appearance theme control gains the two high-contrast options.
`App` stores `tokens: snora::design::Tokens`, recomputed whenever the theme
choice changes.

## Vendoring

snora 0.25.0 is not yet on crates.io. The four crates (`snora`, `snora-core`,
`snora-design`, `snora-widgets`) are vendored under `vendor/` and added as
workspace members. When 0.25 publishes, the dependency reverts to a registry
version and `vendor/` is removed.

## Acceptance criteria

- Workspace builds with snora 0.25 + `design` feature; zero warnings.
- Existing snora layout usage (`AppLayout`, `render`, `Dialog`, `Sheet`)
  unchanged — no API breakage (verified: 0.18 → 0.25 layout API is compatible).
- `theme.rs` helpers derive color/spacing/radius from `Tokens`; call sites
  unchanged.
- Settings exposes Light / Dark / High Contrast Light / High Contrast Dark.
- High-contrast presets visibly increase contrast.
- All existing tests pass; new tests cover token selection per theme choice.

## Out of scope

- Migrating individual call sites to `snora::design::button`/`card` helpers.
- Wiring `FocusTokens` (iced 0.14 exposes no button focus status — BLOCKED
  upstream; tracked by snora).
- OS-level contrast / reduced-motion detection (snora does not provide it).
