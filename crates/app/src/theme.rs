//! MK-022 — Design system: tokens, colour semantics, style helpers.
//!
//! Every screen imports from here. No magic numbers anywhere else.
//!
//! Boundary decision: single-responsibility — this is apimokka's entire
//! design-system surface: spacing, typography (sizes and, since RFC MK-059,
//! line-height), radii, padding, colour/token bridging, and every shared
//! container/button style. MK-022 established it as the one place every
//! screen imports design constants from; task 017 grew it past the
//! 500-line signal threshold by adding the typography scale's missing tier
//! and its line-height accessors, not by taking on a second responsibility,
//! so it stays one file rather than splitting.

use iced::{Background, Border, Color, Shadow, Theme, Vector, widget::container};

// ── MK-050: snora Design token bridge ─────────────────────────────────────────
// These helpers derive colors from contrast-tested snora Design tokens. The
// `*_tok` variants take `&Tokens` directly; the legacy `&Theme` variants are
// kept for call sites that only have a Theme in scope (they continue to work).

use snora::design::Tokens;
use snora::design::style::color::to_iced_color;

/// Whether `t`'s base palette exactly matches the given tokens' background
/// and text — the two base-Palette slots `snora::design::theme` carries
/// through unmodified from the source token (base tiers equal their
/// source token role exactly, never passed through a correcting
/// heuristic). Background alone is not sufficient: `light` and
/// `high_contrast_light` share an identical pure-white background,
/// differing only in text and border strength.
fn matches_tokens(t: &Theme, tokens: Tokens) -> bool {
    let p = t.palette();
    p.background == to_iced_color(tokens.palette.background)
        && p.text == to_iced_color(tokens.palette.text_primary)
}

/// MK-050: whether the given theme is one of the high-contrast presets.
/// High-contrast modes get visible borders on cards/panels (shadows alone are
/// insufficient for low-vision users — a WCAG non-text-contrast consideration).
///
/// RFC MK-058 phase 2: all four presets are now built by
/// `snora::design::theme`, which names every emitted theme "Snora Design
/// (dark)" or "Snora Design (light)" — light-vs-dark only, not
/// standard-vs-high-contrast — so the theme's name string can no longer
/// distinguish a preset. Matched against the token-derived (background,
/// text) pair instead.
pub fn is_high_contrast(t: &Theme) -> bool {
    matches_tokens(t, Tokens::high_contrast_light())
        || matches_tokens(t, Tokens::high_contrast_dark())
}

/// Resolve the snora Design token set matching `t`. Shared by every helper
/// that needs a `tokens.palette.*` role beyond the `background`/`text` pair
/// `Theme` already exposes — `muted`, `border_token`, and `dialog_style`'s
/// border (task 017, D-1/D-2).
fn tokens_for(t: &Theme) -> Tokens {
    if matches_tokens(t, Tokens::high_contrast_dark()) {
        Tokens::high_contrast_dark()
    } else if matches_tokens(t, Tokens::high_contrast_light()) {
        Tokens::high_contrast_light()
    } else if t.extended_palette().background.base.color.r < 0.5 {
        Tokens::dark()
    } else {
        Tokens::light()
    }
}

/// The snora `palette.border` token for the current preset.
///
/// Despite the former name (`hc_border`), this is not high-contrast only: it is
/// read for every preset by panel, chip, card and dialog borders (D-13, D-1).
pub fn border_token(t: &Theme) -> Color {
    to_iced_color(tokens_for(t).palette.border)
}

// ── Spacing scale ─────────────────────────────────────────────────────────────

pub mod space {
    pub const S1: f32 = 4.0; // inline icon/text gap
    pub const S2: f32 = 8.0; // compact field grouping
    pub const S3: f32 = 12.0; // form row gap
    pub const S4: f32 = 16.0; // card internal padding
    pub const S5: f32 = 20.0; // section spacing
    pub const S6: f32 = 24.0; // major panel padding
}

// ── Typography scale ──────────────────────────────────────────────────────────
// All f32 for iced 0.14's Pixels. See MK-022 §4.3.

pub mod size {
    pub const CAPTION: f32 = 12.0; // hints, metadata (unchanged); no snora role
    pub const BODY: f32 = 16.0; // default UI text (was 14 — comfort, WCAG); = snora `body`
    pub const BODY_STRONG: f32 = 16.0; // semibold body (set bold via style); = snora `body`
    pub const BODY_SMALL: f32 = 14.0; // secondary metadata, compact help (RFC MK-059); = snora `body_small`
    pub const LABEL: f32 = 14.0; // button/field/chip/tab labels, no line-height (RFC MK-059); = snora `label`
    pub const SECTION: f32 = 18.0; // card headings (was 17); = snora `title`
    pub const TITLE: f32 = 24.0; // screen titles (was 22); = snora `heading`
    pub const DISPLAY: f32 = 36.0; // welcome hero (was 32); recorded divergence from snora `display` (32.0) — RFC MK-059 resolution 1
    #[allow(dead_code)]
    pub const MONO: f32 = 13.0; // code, paths, JSON; no snora role
}

/// RFC MK-059 decision 5: line-height for prose that wraps, sourced from
/// snora's `<role>_line_height` helpers rather than hand-wrapped
/// (`snora-style` 0.38.0+ / RFC-068; snora withdrew the earlier guidance to
/// read `tokens.typography.<role>.line_height` and construct
/// `LineHeight::Relative` by hand). Named after apimokka's own `size::*`
/// constants (`section`/`title`), not snora's role names (`title`/`heading`)
/// — `size::SECTION` = snora's `title` role and `size::TITLE` = snora's
/// `heading` role, and mirroring snora's names here would read as if this
/// module's `title()` matched `size::TITLE`, which it does not.
///
/// Typography is preset-invariant (RFC MK-059 non-goals: all four snora
/// presets share `Typography::default_roles()`), so a fixed token set is
/// the correct source here, not a per-`Theme` lookup like
/// `theme::muted`/`theme::border_token`.
///
/// No `label()`: labels never take a line-height override (decision 5) —
/// call sites at `size::LABEL` must not call anything in this module.
pub mod line_height {
    use iced::widget::text::LineHeight;
    use snora::design::Tokens;
    use snora::design::style::text;

    fn tokens() -> Tokens {
        Tokens::light()
    }

    /// vs iced's own default (`LineHeight::Relative(1.3)`): looser (1.4) —
    /// the one unambiguous readability gain (RFC MK-059 decision 5).
    pub fn body() -> LineHeight {
        text::body_line_height(&tokens())
    }

    /// vs iced's default: slightly looser (1.35).
    pub fn body_small() -> LineHeight {
        text::body_small_line_height(&tokens())
    }

    /// snora's `title` role (`size::SECTION`'s line-height). vs iced's
    /// default: identical (1.3) — a no-op, kept for symmetry with the other
    /// four, not because it changes anything.
    ///
    /// Genuinely unused today, checked rather than assumed: every
    /// `size::SECTION` call site in this crate renders a short heading or
    /// identifier (a screen title, a card heading, a file/workspace name) —
    /// none of them wrap, and decision 5 is explicit that line-height
    /// applies to prose that wraps, not to single-line values. Kept as a
    /// correct, ready accessor rather than deleted, for whichever
    /// `SECTION`-sized text is the first to genuinely need it.
    #[allow(dead_code)]
    pub fn section() -> LineHeight {
        text::title_line_height(&tokens())
    }

    /// snora's `heading` role (`size::TITLE`'s line-height). vs iced's
    /// default: tighter (1.25). Genuinely unused today, for the same reason
    /// as `section` above — every `size::TITLE` site is a short screen or
    /// dialog title, never wrapping prose.
    #[allow(dead_code)]
    pub fn title() -> LineHeight {
        text::heading_line_height(&tokens())
    }

    /// vs iced's default: tighter (1.2). Genuinely unused today, for the
    /// same reason as `section` above — both `size::DISPLAY` sites are the
    /// single-word app name.
    #[allow(dead_code)]
    pub fn display() -> LineHeight {
        text::display_line_height(&tokens())
    }
}

// ── Border radius ─────────────────────────────────────────────────────────────

pub mod radius {
    #[allow(dead_code)]
    pub const SM: f32 = 6.0; // inputs, small chips
    pub const MD: f32 = 10.0; // buttons, compact cards
    pub const LG: f32 = 14.0; // primary cards and dialogs
    pub const XL: f32 = 18.0; // welcome hero, major panels
    pub const PILL: f32 = 999.0; // status chips
}

// ── Padding presets [vertical, horizontal] ────────────────────────────────────

pub mod pad {
    pub const BUTTON: [f32; 2] = [6.0, 14.0];
    pub const BUTTON_PRIMARY: [f32; 2] = [10.0, 22.0];
    pub const CARD: [f32; 2] = [16.0, 18.0];
    #[allow(dead_code)]
    pub const CHIP: [f32; 2] = [4.0, 10.0];
    #[allow(dead_code)]
    pub const RAIL_ITEM: [f32; 2] = [10.0, 16.0];
}

/// Minimum interactive target sizes (MK-039). 44 px is the WCAG / platform
/// floor; 52 px is the comfortable size for primary actions.
pub mod touch {
    #[allow(dead_code)]
    pub const MIN: f32 = 44.0;
    pub const COMFORTABLE: f32 = 52.0;
}

// ── Colour helpers ─────────────────────────────────────────────────────────────

/// Secondary text — mid-grey legible on both light and dark surfaces.
pub fn muted(t: &Theme) -> Color {
    // MK-050: derive the muted text color from the matching snora Design
    // preset so high-contrast themes get a properly-contrasted muted grey
    // instead of a fixed value. RFC MK-058 phase 2: every preset is now
    // named "Snora Design (dark)"/"(light)" by snora::design::theme
    // (light-vs-dark only), so the high-contrast pair is identified by its
    // token-derived (background, text) pair via `matches_tokens`, same as
    // `is_high_contrast`, rather than by theme name.
    to_iced_color(tokens_for(t).palette.text_muted)
}

#[allow(dead_code)]
pub fn severity_color(t: &Theme, sev: apimokka_model::Severity) -> Color {
    let ep = t.extended_palette();
    match sev {
        apimokka_model::Severity::Error => ep.danger.base.color,
        apimokka_model::Severity::Warning => ep.warning.base.color,
        apimokka_model::Severity::Info => ep.primary.base.color,
    }
}

// ── Container style helpers ───────────────────────────────────────────────────

/// Fill for surfaces that carry muted text (panels, chips, selected parent
/// rows). D-13: Light and Dark take snora's `surface` token, the only token
/// under which `text_muted` keeps its AA guarantee. The iced-derived
/// `weak`/`strong` slots measure 3.1–4.2:1 there and are not used.
///
/// High-contrast presets keep their iced-derived slot: they already pass
/// (muted on `weak`/`strong` is 7.2–11.3:1, see the D-13 measurements) and
/// they are border-defined, so their appearance does not change.
fn text_safe_fill(t: &Theme, high_contrast_slot: Color) -> Color {
    if is_high_contrast(t) {
        high_contrast_slot
    } else {
        to_iced_color(tokens_for(t).palette.surface)
    }
}

/// D-13: the 1 px token border that separates a `surface`-filled panel or
/// chip from the page in Light and Dark. Snora's `palette.border` is
/// contrast-tested as a non-text edge, and reading it from the token keeps it
/// on the same source as `dialog_style` (D-1).
fn surface_border(t: &Theme, width: f32, radius: iced::border::Radius) -> Border {
    Border {
        radius,
        width,
        color: border_token(t),
    }
}

/// Default panel surface (sidebars, top bar, bottom drawer).
/// Light and Dark: snora `surface` fill with a 1 px token border (D-13).
/// High-contrast: unchanged — `weak` fill with the same border.
pub fn panel_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    container::Style {
        background: Some(Background::Color(text_safe_fill(
            t,
            ep.background.weak.color,
        ))),
        text_color: Some(ep.background.base.text),
        border: surface_border(t, 1.0, Default::default()),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.04),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        },
        snap: true,
    }
}

/// Elevated card — the primary unit for rules, trace events, settings sections.
/// Subtle shadow; radius::LG.
///
/// D-13 (task 021 correction F1): Light and Dark take the `surface` fill and a
/// 1 px token border, the same treatment as panels, because muted text on the
/// previous Dark `weak` fill measured 4.22:1. The border keeps a card distinct
/// from the panel it sits on, which is `surface` too. High-contrast is
/// unchanged: its `weak`/`base` fill and 1.5 px border already pass.
pub fn card_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    let hc = is_high_contrast(t);
    let bg = if ep.background.base.color.r < 0.5 {
        // dark mode: step up from base
        ep.background.weak.color
    } else {
        ep.background.base.color // light mode: white
    };
    container::Style {
        background: Some(Background::Color(if hc {
            bg
        } else {
            to_iced_color(tokens_for(t).palette.surface)
        })),
        text_color: Some(ep.background.base.text),
        border: if hc {
            Border {
                radius: radius::LG.into(),
                width: 1.5,
                color: border_token(t),
            }
        } else {
            surface_border(t, 1.0, radius::LG.into())
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.06),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        snap: true,
    }
}

/// D-30: the high-contrast selected-card border — the token colour
/// `card_style` already uses at 1.5 px, at twice the width. Selection is
/// carried by *width* (MK-028's non-colour rule), not colour, so the colour
/// channel stays free: the keyboard cursor (`cursor_style`) owns primary, and
/// a selected-and-cursored row must read as three distinct states —
/// unselected (1.5 px), selected (3 px token), selected + cursor (3 px token
/// inside a 2 px primary ring). The tint is kept as a secondary cue.
fn hc_selected_border(t: &Theme) -> Border {
    Border {
        radius: radius::LG.into(),
        width: 3.0,
        color: border_token(t),
    }
}

/// Selected variant of card_style — primary-coloured border + stronger shadow.
/// The left accent strip is drawn by the caller (a 3px-wide container).
///
/// D-13 (task 021 correction F2): the primary tint measured 4.22:1 (Light) and
/// 4.15:1 (Dark) under muted text, a fork from the token, so Light and Dark
/// take the `surface` fill instead. Selection is carried by a 2 px primary
/// border — twice the 1 px card border, so it is distinct by width as well as
/// colour (MK-028 non-colour rule). High-contrast keeps its tint and, since D-30,
/// adds a 3 px token-colour border (`hc_selected_border`).
pub fn card_selected_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    let p = ep.primary.base.color;
    if is_high_contrast(t) {
        let alpha = if ep.background.base.color.r < 0.5 {
            0.18
        } else {
            0.10
        };
        return container::Style {
            background: Some(Background::Color(Color {
                r: p.r,
                g: p.g,
                b: p.b,
                a: alpha,
            })),
            text_color: Some(ep.background.base.text),
            border: hc_selected_border(t),
            shadow: Shadow {
                color: Color::from_rgba(p.r, p.g, p.b, 0.22),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            },
            snap: true,
        };
    }
    container::Style {
        background: Some(Background::Color(to_iced_color(
            tokens_for(t).palette.surface,
        ))),
        text_color: Some(ep.background.base.text),
        border: Border {
            radius: radius::LG.into(),
            width: 2.0,
            color: p,
        },
        shadow: Shadow {
            color: Color::from_rgba(p.r, p.g, p.b, 0.22),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        snap: true,
    }
}

/// Chip / badge — pill shape. Light and Dark: `surface` fill with a 1 px token
/// border (D-13). High-contrast: unchanged — `strong` fill, no border.
pub fn chip_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    let width = if is_high_contrast(t) { 0.0 } else { 1.0 };
    container::Style {
        background: Some(Background::Color(text_safe_fill(
            t,
            ep.background.strong.color,
        ))),
        text_color: Some(ep.background.base.text),
        border: surface_border(t, width, radius::PILL.into()),
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Modal dialog surface — highest elevation.
///
/// Task 017 D-1/D-2: this used `..Default::default()` for `border` — width
/// zero — which overpainted snora's own dialog-card border (their 0.34.0
/// `palette.border` repair) with nothing, inside the exact card that repair
/// targets. Over a modal dim the fill alone still separated the card, which
/// is why every dimmed case passed; on a full-screen surface with no dim
/// behind it (the wizard, `high_contrast_dark`), fill and shadow both
/// composite to nothing and the card had no discernible edge at all
/// (M8 capture case B2-1). `tokens.palette.border` is contrast-tested for
/// all four presets — the same source `border_token` already reads for the
/// high-contrast pair — so this now inherits snora's repair instead of
/// overpainting it, uniformly, not only where high contrast already forced
/// a border elsewhere (`panel_style`, `card_style`).
pub fn dialog_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    container::Style {
        background: Some(Background::Color(ep.background.base.color)),
        text_color: Some(ep.background.base.text),
        border: Border {
            radius: radius::XL.into(),
            width: 1.0,
            color: to_iced_color(tokens_for(t).palette.border),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.18),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        snap: true,
    }
}

/// D-21 (task 036): the pairing `banner_style`'s text takes, for every text
/// role the friendly-error banner carries (the message text and any muted
/// text on it, per the handoff's item 4). Snora guarantees exactly one
/// foreground for its `warning` fill — `warning_text` — so both roles share
/// it; there is no second, dimmer role carrying the same guarantee against
/// this fill the way `text_muted` carries one against the three neutral
/// surfaces (see `text_safe_fill`). Measured 2026-11-04 (`ratio`, WCAG):
/// 5.42:1 Light, 10.64:1 Dark, 8.41:1 HC Light, 14.86:1 HC Dark — all above
/// the 4.5 floor, none inherited from the token's own doc claim.
pub fn banner_text(t: &Theme) -> Color {
    to_iced_color(tokens_for(t).palette.warning_text)
}

/// Semantic banner: the friendly-error surface (`shell::view`'s highest-
/// priority `feedback_banner` branch) — reload/restart pending, or any
/// other friendly error, carrying the message text and the only visible
/// Undo affordance's sibling surface.
///
/// D-21 (task 036): the fill and text were `ep.warning.weak.color` and
/// `ep.background.base.text` — one iced-derived slot and one token picked
/// for a different surface (`background`, not `warning`) — and measured
/// 1.25–2.54:1 against the 4.5 floor in every preset (§5 of the task
/// handoff). Both roles now come from snora's `warning`/`warning_text`
/// pair, the only snora tokens that carry a guarantee for this specific
/// fill+text pairing (same shape as D-13's `text_safe_fill` and D-15's
/// `text_input_style`, a third and fourth time). This keeps the banner's
/// warning semantic — the fill stays the status colour, not a neutral one —
/// which the known-risk in the handoff flagged as the alternative to avoid.
pub fn banner_style(t: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(to_iced_color(
            tokens_for(t).palette.warning,
        ))),
        text_color: Some(banner_text(t)),
        border: Border {
            radius: radius::MD.into(),
            ..Default::default()
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

/// D-21 round 2 (task 036, F1a): the "Show/Hide details" and "✕" dismiss
/// buttons on the friendly-error banner used `iced::widget::button::text`,
/// which paints its label with `palette.background.base.text` — the exact
/// iced-derived slot round 1 moved everything else on this surface off of.
/// A container's `text_color` does not reach a button's own label, so
/// `banner_style`'s fix never reached these two.
///
/// Deliberately **not** `button::text`'s own shape, which dims the label on
/// Hover/Disabled via `scale_alpha` — measured first, not assumed safe:
/// scaling `banner_text`'s alpha to 0.8 and compositing over `warning`
/// drops Light to ~4.09:1, *below* the 4.5 floor this role exists to clear.
/// This surface has exactly one guaranteed foreground and no second,
/// dimmer one to fall back to (see `banner_text`'s own doc comment), so
/// every status keeps the same full-strength colour — the hierarchy these
/// two controls need comes from size/weight/position, not opacity.
pub fn banner_text_button(
    t: &Theme,
    _status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    iced::widget::button::Style {
        text_color: banner_text(t),
        ..iced::widget::button::Style::default()
    }
}

/// D-21 round 2 (task 036, F2): the region-cursor ring on the friendly-error
/// banner's own controls. `cursor_style`'s primary-coloured border measures
/// 1.13–2.33:1 against this surface's `warning` fill — below the WCAG
/// 1.4.11 non-text floor (3:1) in every preset — because it is shared with
/// every other surface's cursor, none of them this colour. A per-surface
/// cursor treatment (the task handoff's own fallback, proposed here rather
/// than assumed): `banner_text` already clears 4.5:1 for *text* on this
/// fill (see `banner_text`'s own doc comment), so a 2 px ring in that colour
/// clears the lower 3:1 non-text floor with room in every preset, uniformly
/// — no high-contrast/non-high-contrast split, unlike `card_selected_style`'s
/// border (which this otherwise mirrors).
pub fn banner_cursor_style(t: &Theme) -> container::Style {
    container::Style {
        background: None,
        text_color: None,
        border: Border {
            width: 2.0,
            color: banner_text(t),
            radius: radius::MD.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Hairline divider (1 px high container).
pub fn hairline_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    container::Style {
        background: Some(Background::Color(Color {
            a: 0.30,
            ..ep.background.strong.color
        })),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: None,
        snap: true,
    }
}

/// Left accent strip for the selected rail item or sidebar row.
pub fn accent_strip_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    container::Style {
        background: Some(Background::Color(ep.primary.base.color)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: None,
        snap: true,
    }
}

// ── Segmented-control button styles ──────────────────────────────────────────
//
// These replace the container-wrapping pattern that caused double-radius
// artefacts. The button itself carries the full selection visual; no wrapper
// container is needed.

/// Active item in a segmented control (method picker, mode tabs, etc.).
/// Renders a primary-tinted background with radius::MD — no border.
pub fn seg_active(
    theme: &Theme,
    _status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let ep = theme.extended_palette();
    let p = ep.primary.base.color;
    let alpha = if ep.background.base.color.r < 0.5 {
        0.20
    } else {
        0.12
    };
    iced::widget::button::Style {
        background: Some(Background::Color(Color {
            r: p.r,
            g: p.g,
            b: p.b,
            a: alpha,
        })),
        text_color: ep.background.base.text,
        border: Border {
            radius: radius::MD.into(),
            ..Default::default()
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

/// Inactive item in a segmented control.
/// Renders as plain muted text with no background, no border, no radius.
pub fn seg_inactive(
    theme: &Theme,
    _status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: None,
        text_color: muted(theme),
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    }
}

// ── Naked wrapper button ──────────────────────────────────────────────────────
//
// Used for buttons that wrap a styled container (card_style / card_selected_style).
// The container provides the full visual; the button wrapper must be invisible.
// Without this, iced renders the default theme button background and radius
// outside the container's own radius, creating a double-radius artefact.

pub fn naked(theme: &Theme, _status: iced::widget::button::Status) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: None,
        text_color: theme.extended_palette().background.base.text,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    }
}

// ── Parent-row selected style (rule set headers) ──────────────────────────────
//
// Visually distinct from card_selected_style (which uses the primary tint).
// Uses a neutral background.strong tint so a selected rule-set header never
// merges visually with a selected child rule below it.

///
/// D-13: the tint is a translucent `strong`, which composites to 3.9:1 (Light)
/// and 4.1:1 (Dark) under the muted chevron and count it carries.
///
/// Light and Dark take two independent cues against the unselected card beside
/// it (task 021 correction F8): the snora `surface_raised` fill (the token for
/// elevated cards; muted 4.93 / 4.53:1), and the 2 px token border. The card's
/// shadow is also applied, so the selected row reads as raised, not sunk. A
/// selected parent therefore differs from an unselected card by fill, width and
/// shadow parity, not width alone. High-contrast keeps its tint and takes the D-30 3 px token border (`hc_selected_border`).
pub fn card_parent_selected_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    let base = ep.background.strong.color;
    let alpha = if ep.background.base.color.r < 0.5 {
        0.70
    } else {
        0.55
    };
    if is_high_contrast(t) {
        return container::Style {
            background: Some(Background::Color(Color {
                r: base.r,
                g: base.g,
                b: base.b,
                a: alpha,
            })),
            text_color: Some(ep.background.base.text),
            border: hc_selected_border(t),
            shadow: Shadow::default(),
            snap: true,
        };
    }
    container::Style {
        background: Some(Background::Color(to_iced_color(
            tokens_for(t).palette.surface_raised,
        ))),
        text_color: Some(ep.background.base.text),
        border: surface_border(t, 2.0, radius::LG.into()),
        shadow: card_style(t).shadow,
        snap: true,
    }
}

// ── Keyboard cursor indicator ──────────────────────────────────────────────────
//
// MK-023 slice 2 (task 027): the region/cursor keyboard model's visible
// indicator — "the 2 px primary border `card_selected_style` already draws",
// reused rather than invented. Unlike `card_selected_style`, this carries no
// fill or shadow: it wraps an existing widget (a tab bar cell, a list row)
// purely to draw a border on top of whatever that widget already renders, so
// it must add nothing but the border itself.

/// The keyboard cursor: a 2 px primary border, drawn only around the item
/// the cursor currently names, in the region that currently holds focus
/// (task 027 §3.2 — no other region shows a cursor while it does not hold
/// focus). Transparent otherwise, so it never competes with the wrapped
/// widget's own fill (a tab's `seg_active`/`seg_inactive`, a card's
/// `card_style`/`card_selected_style`).
///
/// D-29: the border is this function's own, not borrowed from
/// `card_selected_style` — that style's high-contrast branch carries
/// *selection* by tint and shadow and leaves its border at default, so
/// borrowing it painted nothing at all in both HC presets. A cursor and a
/// selection are different states and must not share an appearance decision.
pub fn cursor_style(t: &Theme) -> container::Style {
    container::Style {
        background: None,
        text_color: None,
        border: Border {
            width: 2.0,
            color: t.extended_palette().primary.base.color,
            radius: radius::LG.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

/// D-19 (task 032 §4): a value-cycling `pick_list`'s handle must stop
/// promising a popup it cannot give — the editor's three operator pickers
/// and Settings' locale, the only four `pick_list`s this app has
/// (`Region::is_value_cycler`). `⇅` (U+21C5, "up arrow over down arrow")
/// reads as "arrows change this" rather than "click to open", unlike the
/// default `▼` handle every other `pick_list` in the wild uses. Not ASCII,
/// so `text::Shaping::Advanced` (`Basic` only covers Latin-1) — verified to
/// render in this app's own font stack in the task 032 live capture rather
/// than trusted from the code point alone, per the handoff's own warning
/// that the font might not have it.
pub fn cycler_handle() -> iced::widget::pick_list::Handle<iced::Font> {
    iced::widget::pick_list::Handle::Static(iced::widget::pick_list::Icon {
        font: iced::Font::DEFAULT,
        code_point: '⇅',
        size: None,
        line_height: iced::widget::text::LineHeight::default(),
        shaping: iced::widget::text::Shaping::Advanced,
    })
}

// ── Text input style ──────────────────────────────────────────────────────────

/// D-15: `iced::widget::text_input::default` derives four colour roles from
/// slots `snora::design::theme` does not guarantee for text — `secondary.base`
/// (placeholder, and disabled value by inheritance), `background.strongest`
/// (disabled placeholder), and `background.strong` (border). Same defect
/// class as D-13, across a whole widget rather than one container: we styled
/// no `text_input` at all, so iced's own default ran unexamined.
///
/// Starts from `iced::widget::text_input::default` and overrides only the
/// four failing roles (task 023 §4.1) — M8's Pass B accepted that stock
/// controls follow the palette (B1–B4), and that evidence stays valid only if
/// the fields it validated keep deriving from the same place.
///
/// §4.3 decision: a placeholder and a disabled field's *value* cannot share
/// one colour. `text_muted` is right for a placeholder (guaranteed ≥4.5:1 on
/// `background.base`, the active fill) but fails on `background.weak` (the
/// disabled fill) — the exact D-13 failure, one token over. `text_secondary`
/// is the token that clears 4.5:1 on `weak` in all four presets, so disabled
/// `value` takes it. Disabled `placeholder` (an empty disabled field, e.g.
/// `settings.rs`'s log-file path before one is set) takes the same token:
/// no third guaranteed tier sits between muted and secondary, and a disabled
/// field's content — present or absent — is equally something the user is
/// meant to read, not decoration to dim further.
///
/// The border (`background.strong`, 1.54–1.58:1 against the fill in every
/// preset) becomes `border_token` for Active — D-13's non-text edge, ≥3:1
/// there in every preset. It is **not** reused for Disabled: `border_token`
/// is contrast-tested against `base` and `surface`, not `weak`, and measures
/// only 2.51 / 2.95 against it in Light / Dark — short of the 3:1 floor.
/// Disabled's border takes `text_secondary` instead, the same token already
/// carrying that status's text, which clears 3:1 against `weak` everywhere
/// (6.53 / 7.10 / 12.91 / 16.50). Hover's border (`background.base.text`) is
/// untouched. The focus ring keeps its colour (`primary.strong`, ≥3:1 against
/// the page and pinned by a test) but, since D-36, is 2 px against the resting
/// 1 px: against the resting border beside it the colour alone cleared 3:1 in
/// Light only (3.74 / 2.63 / 1.40 / 1.19), so width now carries the state.
pub fn text_input_style(
    theme: &Theme,
    status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    use iced::widget::text_input::Status;

    let mut style = iced::widget::text_input::default(theme, status);
    match status {
        Status::Active => {
            style.border.color = border_token(theme);
            style.placeholder = muted(theme);
        }
        Status::Hovered => {
            style.placeholder = muted(theme);
        }
        Status::Focused { .. } => {
            style.placeholder = muted(theme);
            // D-36: focus was iced's hue change at the resting 1 px width and
            // nothing else — 3.74 / 2.63 / 1.40 / 1.19:1 against the resting
            // border in Light / Dark / HC Light / HC Dark, clearing 3:1 in one
            // preset. Text inputs take no region cursor, so this border is the
            // whole indication. Width carries the state (MK-028's non-colour
            // rule, as D-30 did for selection) and the stock colour stays, so
            // no new colour is measured and `primary` is left to the region
            // cursor.
            style.border.width = 2.0;
        }
        Status::Disabled => {
            // `border_token` only clears 3:1 against this status's `weak`
            // fill in the high-contrast pair (15.58 / 20.59); Light and Dark
            // measure 2.51 / 2.95, short of the non-text floor, because the
            // token is contrast-tested against `base` and `surface`, not
            // `weak`. `text_secondary` is already the role carrying this
            // status's text and clears 3:1 against `weak` in every preset
            // (6.53 / 7.10 / 12.91 / 16.50) — reused for the border too,
            // rather than reaching for a token not tested for this pairing.
            let secondary = to_iced_color(tokens_for(theme).palette.text_secondary);
            style.border.color = secondary;
            style.value = secondary;
            style.placeholder = secondary;
        }
    }
    style
}

#[cfg(test)]
#[path = "theme/tests.rs"]
mod tests;
