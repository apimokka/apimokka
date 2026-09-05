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
/// `Theme` already exposes — `muted`, `hc_border`, and `dialog_style`'s
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

/// MK-050: the border color for high-contrast surfaces, from snora tokens.
pub fn hc_border(t: &Theme) -> Color {
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
/// `theme::muted`/`theme::hc_border`.
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

/// Default panel surface (sidebars, top bar, bottom drawer).
/// Slightly off-base; no border. Subtle bottom shadow separates from body.
pub fn panel_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    container::Style {
        background: Some(Background::Color(ep.background.weak.color)),
        text_color: Some(ep.background.base.text),
        border: if is_high_contrast(t) {
            Border {
                width: 1.0,
                color: hc_border(t),
                ..Default::default()
            }
        } else {
            Border::default()
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.04),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        },
        snap: true,
    }
}

/// Elevated card — the primary unit for rules, trace events, settings sections.
/// Subtle shadow; no border; radius::LG.
pub fn card_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    let bg = if ep.background.base.color.r < 0.5 {
        // dark mode: step up from base
        ep.background.weak.color
    } else {
        ep.background.base.color // light mode: white
    };
    container::Style {
        background: Some(Background::Color(bg)),
        text_color: Some(ep.background.base.text),
        border: if is_high_contrast(t) {
            Border {
                radius: radius::LG.into(),
                width: 1.5,
                color: hc_border(t),
            }
        } else {
            Border {
                radius: radius::LG.into(),
                ..Default::default()
            }
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.06),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        snap: true,
    }
}

/// Selected variant of card_style — primary tint + stronger shadow.
/// The left accent strip is drawn by the caller (a 3px-wide container).
pub fn card_selected_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    let p = ep.primary.base.color;
    let alpha = if ep.background.base.color.r < 0.5 {
        0.18
    } else {
        0.10
    };
    container::Style {
        background: Some(Background::Color(Color {
            r: p.r,
            g: p.g,
            b: p.b,
            a: alpha,
        })),
        text_color: Some(ep.background.base.text),
        border: Border {
            radius: radius::LG.into(),
            ..Default::default()
        },
        shadow: Shadow {
            color: Color::from_rgba(p.r, p.g, p.b, 0.22),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        snap: true,
    }
}

/// Chip / badge — pill shape, muted background.
pub fn chip_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    container::Style {
        background: Some(Background::Color(ep.background.strong.color)),
        text_color: Some(ep.background.base.text),
        border: Border {
            radius: radius::PILL.into(),
            ..Default::default()
        },
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
/// all four presets — the same source `hc_border` already reads for the
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

/// Semantic banner: reload/restart pending.
pub fn banner_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    container::Style {
        background: Some(Background::Color(ep.warning.weak.color)),
        text_color: Some(ep.background.base.text),
        border: Border {
            radius: radius::MD.into(),
            ..Default::default()
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
#[allow(dead_code)]
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

/// Left-rail selected destination background.
#[allow(dead_code)]
pub fn rail_selected_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    let p = ep.primary.base.color;
    let alpha = if ep.background.base.color.r < 0.5 {
        0.20
    } else {
        0.08
    };
    container::Style {
        background: Some(Background::Color(Color {
            r: p.r,
            g: p.g,
            b: p.b,
            a: alpha,
        })),
        text_color: Some(ep.background.base.text),
        border: Border {
            radius: radius::MD.into(),
            ..Default::default()
        },
        shadow: Shadow::default(),
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

pub fn card_parent_selected_style(t: &Theme) -> container::Style {
    let ep = t.extended_palette();
    let base = ep.background.strong.color;
    let alpha = if ep.background.base.color.r < 0.5 {
        0.70
    } else {
        0.55
    };
    container::Style {
        background: Some(Background::Color(Color {
            r: base.r,
            g: base.g,
            b: base.b,
            a: alpha,
        })),
        text_color: Some(ep.background.base.text),
        border: Border {
            radius: radius::LG.into(),
            ..Default::default()
        },
        shadow: Shadow::default(),
        snap: true,
    }
}
