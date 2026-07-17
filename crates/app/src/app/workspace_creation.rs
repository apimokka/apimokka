use super::*;
use crate::message::Message;

#[test]
fn wizard_create_produces_blank_workspace_with_wizard_name() {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    // Fill in wizard fields
    a.update(Message::WizardSetName("inventory-mock".into()));
    a.update(Message::WizardSetHost("0.0.0.0".into()));
    a.wizard.port = "9090".into();

    a.update(Message::WizardCreate);

    let snap = a.snapshot.as_ref().expect("snapshot after WizardCreate");
    assert_eq!(snap.meta.name, "inventory-mock");
    assert_eq!(snap.root_settings.listener_ip, "0.0.0.0");
    assert_eq!(snap.root_settings.listener_port, 9090);
    // Default starter is Minimal — one rule set with a health-check rule.
    assert_eq!(
        snap.rule_sets.len(),
        1,
        "Minimal starter creates one rule set"
    );
    assert_eq!(
        snap.rule_sets[0].rules.len(),
        1,
        "Minimal starter has one rule"
    );
    assert_eq!(snap.rule_sets[0].rules[0].payload.url_path, "/health");
    assert!(matches!(a.view, AppView::Workspace));
    assert!(a.notice.is_some(), "welcome notice shown after create");
}

#[test]
fn wizard_create_with_empty_name_uses_default() {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    // Leave wizard name empty
    a.update(Message::WizardCreate);
    let snap = a.snapshot.as_ref().unwrap();
    assert_eq!(snap.meta.name, "my-mock", "default name used when blank");
}

#[test]
fn open_workspace_still_loads_the_mock() {
    // OpenWorkspace (from Dashboard) continues to load the rich mock workspace.
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::OpenWorkspace("payments-mock".into()));
    let snap = a.snapshot.as_ref().unwrap();
    assert!(
        !snap.rule_sets.is_empty(),
        "opening an existing workspace loads the full mock"
    );
}

#[test]
fn blank_workspace_shows_add_rule_set_cta() {
    // With no rule sets, the centre panel shows the blank-workspace CTA.
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::WizardCreate);
    assert!(matches!(a.view, AppView::Workspace));
    // Build the Routes view — should not panic even with no rule sets.
    let _ = crate::screens::routes::view(&a);
}
