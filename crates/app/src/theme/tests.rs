//! D-13 contract tests (task 021 and its correction round). They pin the §5
//! measurements so a later change cannot silently trade muted-text contrast
//! for surface separation, or the reverse.

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

/// WCAG contrast ratio of `fg` on `bg`. `bg` must be opaque: a translucent
/// fill is composited over the page it sits on first (see `over`), so the
/// figure is what renders. Passing a translucent `bg` is a bug in the caller.
fn ratio(fg: Color, bg: Color) -> f32 {
    assert!(
        bg.a >= 1.0,
        "ratio needs an opaque background; composite over the page first"
    );
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

/// Every style that places muted text, as the §7.1 audit and its correction
/// round identified them. No exclusions.
fn muted_surfaces(t: &Theme) -> [(&'static str, container::Style); 5] {
    [
        ("panel_style", panel_style(t)),
        ("chip_style", chip_style(t)),
        ("card_style", card_style(t)),
        ("card_selected_style", card_selected_style(t)),
        ("card_parent_selected_style", card_parent_selected_style(t)),
    ]
}

fn fill(style: &container::Style) -> Color {
    match style.background {
        Some(Background::Color(c)) => c,
        other => panic!("expected a solid fill, got {other:?}"),
    }
}

/// The page a surface sits on in these tests: the preset's base colour.
fn page_of(t: &Theme) -> Color {
    t.extended_palette().background.base.color
}

#[test]
fn muted_text_meets_aa_on_every_muted_surface_in_all_presets() {
    let mut failures = Vec::new();
    for p in presets() {
        let t = snora_theme(&p.tokens);
        let page = page_of(&t);
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
fn light_and_dark_muted_surfaces_fill_with_their_snora_tokens() {
    for p in presets().into_iter().filter(|p| !p.high_contrast) {
        let t = snora_theme(&p.tokens);
        let surface = to_iced_color(p.tokens.palette.surface);
        let raised = to_iced_color(p.tokens.palette.surface_raised);
        for (name, style) in muted_surfaces(&t) {
            // The selected parent is the one deliberate exception: F8 gives it
            // the snora `surface_raised` token as its state fill.
            let expected = if name == "card_parent_selected_style" {
                raised
            } else {
                surface
            };
            assert_eq!(fill(&style), expected, "{} {name} fill", p.name);
        }
    }
}

#[test]
fn light_and_dark_surfaces_carry_token_borders_with_visible_contrast() {
    // `surface` is 1.08–1.11:1 from the page in Light and Dark (§5), so the
    // border is the edge. Each border must be a snora token and clear the
    // WCAG 1.4.11 non-text threshold of 3:1 against the surface it outlines.
    for p in presets().into_iter().filter(|p| !p.high_contrast) {
        let t = snora_theme(&p.tokens);
        let surface = to_iced_color(p.tokens.palette.surface);
        let token_border = to_iced_color(p.tokens.palette.border);
        let primary = t.extended_palette().primary.base.color;
        let expected = [
            ("panel_style", token_border, 1.0),
            ("chip_style", token_border, 1.0),
            ("card_style", token_border, 1.0),
            ("card_selected_style", primary, 2.0),
            ("card_parent_selected_style", token_border, 2.0),
        ];
        for ((name, style), (expected_name, colour, width)) in
            muted_surfaces(&t).into_iter().zip(expected)
        {
            assert_eq!(name, expected_name);
            assert_eq!(style.border.width, width, "{} {name} border width", p.name);
            assert_eq!(
                style.border.color, colour,
                "{} {name} border colour",
                p.name
            );
            let r = ratio(colour, surface);
            assert!(
                r >= 3.0,
                "{}: {name} border is {r:.2}:1 against the surface, below 3:1",
                p.name
            );
        }
    }
}

#[test]
fn muted_surfaces_stay_pairwise_distinct() {
    // Pairwise distinctions the design relies on. Some pairs share a fill and
    // are told apart by border; the selected parent differs by fill and shadow.
    for p in presets().into_iter().filter(|p| !p.high_contrast) {
        let t = snora_theme(&p.tokens);
        let panel = panel_style(&t).border;
        let chip = chip_style(&t).border;
        let card = card_style(&t).border;
        let selected = card_selected_style(&t).border;
        let parent = card_parent_selected_style(&t).border;

        // Card on panel: same fill, so the card's border must exist.
        assert!(
            card.width > 0.0 && card.width == panel.width,
            "{} card vs panel",
            p.name
        );
        // Chip on panel: same fill, same border token; the pill shape carries it.
        assert_eq!(chip.color, panel.color, "{} chip vs panel colour", p.name);
        // Selected card vs unselected card: width and colour both differ.
        assert!(
            selected.width > card.width,
            "{} selected vs card width",
            p.name
        );
        assert_ne!(
            selected.color, card.color,
            "{} selected vs card colour",
            p.name
        );
        // Selected parent vs unselected card: width differs.
        assert!(parent.width > card.width, "{} parent vs card width", p.name);
        // Selected parent vs selected card: colour differs.
        assert_ne!(
            parent.color, selected.color,
            "{} parent vs selected colour",
            p.name
        );
        // F8: selected parent vs unselected card must have two independent
        // cues, at least one not a border-width delta. Fill (surface_raised vs
        // surface) and shadow parity (the raised shadow, same as the card's).
        let parent_fill = fill(&card_parent_selected_style(&t));
        assert_eq!(
            parent_fill,
            to_iced_color(p.tokens.palette.surface_raised),
            "{} parent fill is the raised token",
            p.name
        );
        assert_ne!(
            parent_fill,
            to_iced_color(p.tokens.palette.surface),
            "{} parent fill differs from the card fill",
            p.name
        );
        assert_eq!(
            card_parent_selected_style(&t).shadow,
            card_style(&t).shadow,
            "{} parent carries the card shadow",
            p.name
        );
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
        assert_eq!(panel_style(&t).border.width, 1.0, "{} panel border", p.name);
        assert_eq!(
            fill(&chip_style(&t)),
            ep.background.strong.color,
            "{} chip",
            p.name
        );
        assert_eq!(chip_style(&t).border.width, 0.0, "{} chip border", p.name);
        assert_eq!(card_style(&t).border.width, 1.5, "{} card border", p.name);
        assert_eq!(
            card_selected_style(&t).border.width,
            0.0,
            "{} selected card border",
            p.name
        );
        assert_eq!(
            card_parent_selected_style(&t).border.width,
            0.0,
            "{} parent border",
            p.name
        );
    }
}

// ── D-15 contract tests (task 023). text_input's colour roles, pinned so a
// later change cannot silently reintroduce an unguaranteed colour. ──────────

use iced::widget::text_input::{self, Status};

#[test]
fn text_input_placeholder_meets_aa_in_all_presets() {
    // Active, Hovered and Focused all use the same placeholder colour.
    let mut failures = Vec::new();
    for p in presets() {
        let t = snora_theme(&p.tokens);
        let base = page_of(&t);
        for status in [
            Status::Active,
            Status::Hovered,
            Status::Focused { is_hovered: false },
        ] {
            let r = ratio(text_input_style(&t, status).placeholder, base);
            if r < 4.5 {
                failures.push(format!("{} {status:?} {r:.2}:1", p.name));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "placeholder below AA 4.5:1: {failures:?}"
    );
    // Discriminates against the pre-fix value: stock `secondary.base` measures
    // 1.00–1.09 against `background.base` in every preset (task 023 §3).
}

#[test]
fn text_input_disabled_value_and_placeholder_meet_aa_in_all_presets() {
    let mut failures = Vec::new();
    for p in presets() {
        let t = snora_theme(&p.tokens);
        let weak = t.extended_palette().background.weak.color;
        let style = text_input_style(&t, Status::Disabled);
        for (role, colour) in [("value", style.value), ("placeholder", style.placeholder)] {
            let r = ratio(colour, weak);
            if r < 4.5 {
                failures.push(format!("{} disabled {role} {r:.2}:1", p.name));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "disabled text below AA 4.5:1: {failures:?}"
    );
    // Discriminates: stock disabled `value` (= active placeholder) measures
    // 1.02–1.35 against `background.weak`; stock disabled `placeholder`
    // (`background.strongest`) measures 1.14–1.46 (task 023 §3).
}

#[test]
fn text_input_borders_meet_non_text_contrast_in_all_presets() {
    // Active's border sits on `background.base`; Disabled's sits on
    // `background.weak` and is not the same token (see `text_input_style`'s
    // doc comment — `border_token` alone does not clear 3:1 there).
    let mut failures = Vec::new();
    for p in presets() {
        let t = snora_theme(&p.tokens);
        let base = page_of(&t);
        let weak = t.extended_palette().background.weak.color;
        let active = ratio(text_input_style(&t, Status::Active).border.color, base);
        let disabled = ratio(text_input_style(&t, Status::Disabled).border.color, weak);
        if active < 3.0 {
            failures.push(format!("{} active border {active:.2}:1", p.name));
        }
        if disabled < 3.0 {
            failures.push(format!("{} disabled border {disabled:.2}:1", p.name));
        }
    }
    assert!(failures.is_empty(), "border below 3:1: {failures:?}");
    // Discriminates: stock border (`background.strong`) measures 1.54–1.58
    // against `background.base` in every preset (task 023 §3).
}

#[test]
fn text_input_focus_ring_stays_above_non_text_contrast() {
    // Pin: already passes today (10.00–17.70); not touched by the fix. Assert
    // it so a future change cannot quietly take it away.
    for p in presets() {
        let t = snora_theme(&p.tokens);
        let base = page_of(&t);
        let r = ratio(
            text_input_style(&t, Status::Focused { is_hovered: false })
                .border
                .color,
            base,
        );
        assert!(r >= 3.0, "{}: focus ring {r:.2}:1, below 3:1", p.name);
    }
}

#[test]
fn text_input_style_starts_from_iced_default() {
    // Task 023 §4.1 / acceptance criterion 2: every untouched role (icon,
    // selection, hover/focus border, active/hovered/focused background) must
    // still equal what `text_input::default` produces, so M8's Pass B
    // evidence (stock controls follow the palette) stays valid.
    for p in presets() {
        let t = snora_theme(&p.tokens);
        for status in [
            Status::Active,
            Status::Hovered,
            Status::Focused { is_hovered: false },
            Status::Disabled,
        ] {
            let base = text_input::default(&t, status);
            let ours = text_input_style(&t, status);
            assert_eq!(
                ours.background, base.background,
                "{} {status:?} background",
                p.name
            );
            assert_eq!(ours.icon, base.icon, "{} {status:?} icon", p.name);
            assert_eq!(
                ours.selection, base.selection,
                "{} {status:?} selection",
                p.name
            );
        }
        // Hover and Focus borders are untouched (not in the failing set).
        let hover_base = text_input::default(&t, Status::Hovered).border;
        let hover_ours = text_input_style(&t, Status::Hovered).border;
        assert_eq!(hover_ours, hover_base, "{} hover border", p.name);
        let focus_base = text_input::default(&t, Status::Focused { is_hovered: false }).border;
        let focus_ours = text_input_style(&t, Status::Focused { is_hovered: false }).border;
        assert_eq!(focus_ours, focus_base, "{} focus border", p.name);
    }
}
