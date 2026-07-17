use super::*;
use crate::message::Message;

fn guided() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Guided,
    ));
    a.update(Message::OpenWorkspace("test".into()));
    a
}
fn expert() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::OpenWorkspace("test".into()));
    a
}

#[test]
fn guided_when_starts_collapsed_and_resets_on_mode_switch() {
    let mut a = expert();
    a.update(Message::ToggleRuleWhenMore);
    assert!(a.rule_when_more);
    // Switching to Guided resets density toggles.
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Guided,
    ));
    assert!(
        !a.rule_when_more,
        "switching to Guided resets advanced layout"
    );
    assert!(!a.settings_advanced_more);
}

#[test]
fn rule_when_more_persists_across_rule_navigation() {
    let mut a = guided();
    a.update(Message::ToggleRuleWhenMore);
    assert!(a.rule_when_more);
    // Navigate to a different rule — expanded state must persist.
    let snap = a.snapshot.as_ref().unwrap();
    if snap.rule_sets[0].rules.len() > 1 {
        let other_id = snap.rule_sets[0].rules[1].id;
        a.update(Message::SelectRule(other_id));
    }
    assert!(
        a.rule_when_more,
        "expanded state persists across rule navigation"
    );
}

#[test]
fn settings_advanced_toggle_works() {
    let mut a = guided();
    assert!(!a.settings_advanced_more);
    a.update(Message::ToggleSettingsAdvancedMore);
    assert!(a.settings_advanced_more);
    a.update(Message::ToggleSettingsAdvancedMore);
    assert!(!a.settings_advanced_more);
}

#[test]
fn routes_view_builds_in_guided_collapsed_and_expanded() {
    let rule_id = expert().snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    for expanded in [false, true] {
        let mut a = guided();
        a.rule_when_more = expanded;
        a.update(Message::SelectRule(rule_id));
        let _ = crate::screens::routes::view(&a);
    }
}

#[test]
fn settings_view_builds_in_guided_collapsed_and_expanded() {
    for expanded in [false, true] {
        let mut a = guided();
        a.settings_advanced_more = expanded;
        a.tab = crate::selection::WorkspaceTab::Settings;
        let _ = crate::screens::settings::view(&a);
    }
}
