//! D-13 contract tests. These pin the §5 measurements (task 021) so a later
//! change cannot silently trade muted-text contrast for panel separation, or
//! the reverse.

use super::*;
use snora::design::{Tokens, theme as snora_theme};

/// WCAG 2.x relative luminance.
fn luminance(c: Color) -> f32 {
    let lin = |v: f32| {
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
}

/// `fg` composited over an opaque `bg`, in sRGB.
fn over(fg: Color, bg: Color) -> Color {
    let a = fg.a;
    Color {
        r: fg.r * a + bg.r * (1.0 - a),
        g: fg.g * a + bg.g * (1.0 - a),
        b: fg.b * a + bg.b * (1.0 - a),
        a: 1.0,
    }
}

/// WCAG contrast ratio of `fg` on `bg`. Both are composited over white first,
/// so a translucent colour is measured as it renders.
fn ratio(fg: Color, bg: Color) -> f32 {
    let bg = over(bg, Color::WHITE);
    let fg = over(fg, bg);
    let (a, b) = (luminance(fg), luminance(bg));
    let (hi, lo) = if a > b { (a, b) } else { (b, a) };
    (hi + 0.05) / (lo + 0.05)
}

struct Preset {
    name: &'static str,
    tokens: Tokens,
    high_contrast: bool,
}

fn presets() -> [Preset; 4] {
    [
        Preset {
            name: "light",
            tokens: Tokens::light(),
            high_contrast: false,
        },
        Preset {
            name: "dark",
            tokens: Tokens::dark(),
            high_contrast: false,
        },
        Preset {
            name: "hc_light",
            tokens: Tokens::high_contrast_light(),
            high_contrast: true,
        },
        Preset {
            name: "hc_dark",
            tokens: Tokens::high_contrast_dark(),
            high_contrast: true,
        },
    ]
}

/// The surfaces that carry muted text, as the §7.1 audit identified them.
///
/// Deliberately absent:
/// - `card_style` — §8.7 keeps it out of scope (D-2).
/// - `card_selected_style` — OPEN, escalated to the architect. Its primary
///   tint measures 4.22:1 (Light) and 4.15:1 (Dark) under the muted hint on
///   the selected file-route row (`screens/routes/sidebar.rs:109`). Fixing it
///   changes the selection look, which is a design decision, not a D-13
///   repair. Add it back here once that decision is taken.
fn muted_surfaces(t: &Theme) -> [(&'static str, container::Style); 3] {
    [
        ("panel_style", panel_style(t)),
        ("chip_style", chip_style(t)),
        ("card_parent_selected_style", card_parent_selected_style(t)),
    ]
}

fn fill(style: &container::Style) -> Color {
    match style.background {
        Some(Background::Color(c)) => c,
        other => panic!("expected a solid fill, got {other:?}"),
    }
}

#[test]
fn muted_text_meets_aa_on_every_muted_surface_in_all_presets() {
    let mut failures = Vec::new();
    for p in presets() {
        let t = snora_theme(&p.tokens);
        let page = t.extended_palette().background.base.color;
        let muted = muted(&t);
        for (name, style) in muted_surfaces(&t) {
            let r = ratio(muted, over(fill(&style), page));
            if r < 4.5 {
                failures.push(format!("{} {name} {r:.2}:1", p.name));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "muted text below AA 4.5:1: {failures:?}"
    );
}

#[test]
fn light_and_dark_panels_and_chips_fill_with_snora_surface() {
    for p in presets().into_iter().filter(|p| !p.high_contrast) {
        let t = snora_theme(&p.tokens);
        let surface = to_iced_color(p.tokens.palette.surface);
        assert_eq!(fill(&panel_style(&t)), surface, "{} panel", p.name);
        assert_eq!(fill(&chip_style(&t)), surface, "{} chip", p.name);
    }
}

#[test]
fn light_and_dark_surfaces_stay_bordered_with_a_token_border() {
    // Separation from the page is carried by the border, since `surface` is
    // only 1.08–1.11:1 from the page in Light and Dark (§5).
    for p in presets().into_iter().filter(|p| !p.high_contrast) {
        let t = snora_theme(&p.tokens);
        let page = t.extended_palette().background.base.color;
        let token_border = to_iced_color(p.tokens.palette.border);
        for (name, style) in [
            ("panel_style", panel_style(&t)),
            ("chip_style", chip_style(&t)),
            ("card_parent_selected_style", card_parent_selected_style(&t)),
        ] {
            assert_eq!(style.border.width, 1.0, "{} {name} border width", p.name);
            assert_eq!(
                style.border.color, token_border,
                "{} {name} border colour",
                p.name
            );
            let r = ratio(token_border, page);
            assert!(
                r >= 3.0,
                "{}: {name} border is {r:.2}:1 against the page, below 3:1",
                p.name
            );
        }
    }
}

#[test]
fn high_contrast_presets_keep_their_iced_slots_and_borders() {
    // §8.3: the high-contrast presets already pass and are border-defined.
    // Their appearance must not move.
    for p in presets().into_iter().filter(|p| p.high_contrast) {
        let t = snora_theme(&p.tokens);
        let ep = t.extended_palette();
        assert_eq!(
            fill(&panel_style(&t)),
            ep.background.weak.color,
            "{} panel",
            p.name
        );
        assert_eq!(
            fill(&chip_style(&t)),
            ep.background.strong.color,
            "{} chip",
            p.name
        );
        assert_eq!(panel_style(&t).border.width, 1.0, "{} panel border", p.name);
        assert_eq!(chip_style(&t).border.width, 0.0, "{} chip border", p.name);
    }
}
