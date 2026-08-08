use super::*;
use crate::message::Message;

#[test]
fn theme_choice_has_four_variants() {
    assert_eq!(ThemeChoice::all().len(), 4);
}

#[test]
fn theme_toggle_cycles_through_all_four() {
    let mut c = ThemeChoice::Light;
    c = c.toggle();
    assert_eq!(c, ThemeChoice::Dark);
    c = c.toggle();
    assert_eq!(c, ThemeChoice::HighContrastLight);
    c = c.toggle();
    assert_eq!(c, ThemeChoice::HighContrastDark);
    c = c.toggle();
    assert_eq!(c, ThemeChoice::Light);
}

#[test]
fn each_theme_choice_yields_distinct_tokens() {
    // Tokens differ across presets — verify text_muted is not identical
    // between light and high-contrast light (HC is darker/stronger).
    let light = ThemeChoice::Light.tokens();
    let hc = ThemeChoice::HighContrastLight.tokens();
    let l = light.palette.text_muted;
    let h = hc.palette.text_muted;
    assert!(
        (l.r - h.r).abs() > f32::EPSILON
            || (l.g - h.g).abs() > f32::EPSILON
            || (l.b - h.b).abs() > f32::EPSILON,
        "high-contrast muted text should differ from standard light"
    );
}

#[test]
fn all_four_presets_are_token_derived_custom_themes() {
    // RFC MK-058 phase 2: every preset, not only the high-contrast ones,
    // is now built by snora::design::theme from the same Tokens source
    // that already backs `tokens()` — so stock iced widgets follow the
    // same palette as snora's own primitives in every preset.
    for choice in ThemeChoice::all() {
        assert!(matches!(choice.iced(), iced::Theme::Custom(_)));
    }
}

#[test]
fn stock_widget_palette_follows_tokens_in_every_preset() {
    // RFC MK-058 phase 2's actual claim: stock iced widgets (which read
    // Theme::extended_palette(), not the token bundle directly) now draw
    // from the same background as snora's own primitives, in every
    // preset — not only the two high-contrast ones.
    use snora::design::style::color::to_iced_color;
    for choice in ThemeChoice::all() {
        let expected = to_iced_color(choice.tokens().palette.background);
        let actual = choice.iced().extended_palette().background.base.color;
        assert_eq!(
            actual, expected,
            "{choice:?}: stock widget background must match its token background"
        );
    }
}

#[test]
fn high_contrast_themes_are_detected() {
    assert!(crate::theme::is_high_contrast(
        &ThemeChoice::HighContrastLight.iced()
    ));
    assert!(crate::theme::is_high_contrast(
        &ThemeChoice::HighContrastDark.iced()
    ));
    assert!(!crate::theme::is_high_contrast(&ThemeChoice::Light.iced()));
    assert!(!crate::theme::is_high_contrast(&ThemeChoice::Dark.iced()));
}

#[test]
fn muted_text_picks_each_presets_own_token_not_a_neighbor() {
    // Regression coverage: an earlier draft of this fix left `muted()`
    // detecting high-contrast presets by parsing the theme's Debug/Display
    // name for "hc-dark"/"hc-light" substrings — a string
    // `snora::design::theme` (RFC MK-058 phase 2) no longer produces, since
    // it names every theme "Snora Design (dark)"/"(light)" regardless of
    // contrast. That left every preset's muted color silently falling
    // through to the wrong (non-high-contrast) branch. Assert each of the
    // four presets' emitted muted color matches its own token, not one of
    // the other three.
    let all: Vec<(ThemeChoice, iced::Color)> = ThemeChoice::all()
        .into_iter()
        .map(|choice| {
            use snora::design::style::color::to_iced_color;
            (choice, to_iced_color(choice.tokens().palette.text_muted))
        })
        .collect();
    for (choice, expected_muted) in &all {
        let actual = crate::theme::muted(&choice.iced());
        assert_eq!(actual, *expected_muted, "{choice:?}: wrong muted color");
        for (other, other_muted) in &all {
            if other != choice {
                assert_ne!(
                    actual, *other_muted,
                    "{choice:?}: muted color coincidentally matches {other:?}'s"
                );
            }
        }
    }
}

#[test]
fn set_theme_message_updates_choice() {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::SetTheme(ThemeChoice::HighContrastDark));
    assert_eq!(a.theme_choice, ThemeChoice::HighContrastDark);
}

#[test]
fn is_dark_classification() {
    assert!(!ThemeChoice::Light.is_dark());
    assert!(ThemeChoice::Dark.is_dark());
    assert!(!ThemeChoice::HighContrastLight.is_dark());
    assert!(ThemeChoice::HighContrastDark.is_dark());
}

#[test]
fn card_style_builds_for_all_themes() {
    // The high-contrast branch adds a border; verify no panic for any theme.
    for choice in ThemeChoice::all() {
        let th = choice.iced();
        let _ = crate::theme::card_style(&th);
        let _ = crate::theme::panel_style(&th);
        let _ = crate::theme::muted(&th);
    }
}

#[test]
fn settings_view_builds_with_high_contrast_theme() {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::OpenWorkspace("test".into()));
    a.update(Message::SetTheme(ThemeChoice::HighContrastLight));
    a.tab = crate::selection::WorkspaceTab::Settings;
    let _ = crate::screens::settings::view(&a);
}
