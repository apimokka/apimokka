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
fn standard_themes_use_native_iced() {
    assert!(matches!(ThemeChoice::Light.iced(), iced::Theme::Light));
    assert!(matches!(ThemeChoice::Dark.iced(), iced::Theme::Dark));
}

#[test]
fn high_contrast_themes_use_custom_palette() {
    assert!(matches!(
        ThemeChoice::HighContrastLight.iced(),
        iced::Theme::Custom(_)
    ));
    assert!(matches!(
        ThemeChoice::HighContrastDark.iced(),
        iced::Theme::Custom(_)
    ));
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
