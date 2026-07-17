use super::*;
use crate::message::Message;

fn expert_at_wizard() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::GoWizard);
    a
}

// ── AddRuleSet ───────────────────────────────────────────────────────

#[test]
fn add_rule_set_creates_real_rule_set() {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::WizardSetStarter(WizardStarter::Empty));
    a.update(Message::WizardCreate);
    assert!(a.snapshot.as_ref().unwrap().rule_sets.is_empty());

    a.update(Message::AddRuleSet);

    let snap = a.snapshot.as_ref().unwrap();
    assert_eq!(
        snap.rule_sets.len(),
        1,
        "AddRuleSet creates a real rule set"
    );
    assert!(
        snap.rule_sets[0].file.path.contains("rule-set-1"),
        "generated filename includes the sequence number"
    );
    assert!(snap.rule_sets[0].file.dirty, "new rule set starts dirty");
    assert_eq!(
        a.selection.rule_set,
        Some(snap.rule_sets[0].id),
        "new rule set is selected"
    );
}

#[test]
fn add_rule_set_increments_filename_number() {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::WizardSetStarter(WizardStarter::Empty));
    a.update(Message::WizardCreate);

    a.update(Message::AddRuleSet);
    a.update(Message::AddRuleSet);

    let snap = a.snapshot.as_ref().unwrap();
    assert_eq!(snap.rule_sets.len(), 2);
    assert!(
        snap.rule_sets[1].file.path.contains("rule-set-2"),
        "second rule set is numbered 2"
    );
}

// ── Wizard starter ───────────────────────────────────────────────────

#[test]
fn wizard_starter_minimal_creates_health_rule() {
    let mut a = expert_at_wizard();
    a.update(Message::WizardSetStarter(WizardStarter::Minimal));
    a.update(Message::WizardCreate);

    let snap = a.snapshot.as_ref().unwrap();
    assert_eq!(snap.rule_sets.len(), 1);
    assert_eq!(snap.rule_sets[0].rules.len(), 1);
    assert_eq!(snap.rule_sets[0].rules[0].payload.url_path, "/health");
    assert_eq!(snap.rule_sets[0].rules[0].payload.method, "GET");
}

#[test]
fn wizard_starter_shop_api_loads_full_mock() {
    let mut a = expert_at_wizard();
    a.update(Message::WizardSetStarter(WizardStarter::ShopApi));
    a.update(Message::WizardCreate);

    let snap = a.snapshot.as_ref().unwrap();
    assert!(
        snap.rule_sets.len() >= 2,
        "ShopApi starter loads the full mock with multiple rule sets"
    );
    assert!(
        !snap.fallback_files.is_empty(),
        "ShopApi starter includes fallback files"
    );
}

#[test]
fn wizard_starter_empty_produces_blank() {
    let mut a = expert_at_wizard();
    a.update(Message::WizardSetStarter(WizardStarter::Empty));
    a.update(Message::WizardCreate);

    let snap = a.snapshot.as_ref().unwrap();
    assert!(snap.rule_sets.is_empty(), "Empty starter = no rule sets");
}

#[test]
fn wizard_starter_default_is_minimal() {
    let a = expert_at_wizard();
    assert_eq!(a.wizard.starter, WizardStarter::Minimal);
}

#[test]
fn wizard_set_starter_message_updates_state() {
    let mut a = expert_at_wizard();
    a.update(Message::WizardSetStarter(WizardStarter::ShopApi));
    assert_eq!(a.wizard.starter, WizardStarter::ShopApi);
    a.update(Message::WizardSetStarter(WizardStarter::Empty));
    assert_eq!(a.wizard.starter, WizardStarter::Empty);
}

// ── Minimal workspace model ───────────────────────────────────────────

#[test]
fn minimal_workspace_has_health_check_rule() {
    let ws = apimokka_model::mock::minimal_workspace("svc", "127.0.0.1", 8080, false);
    assert_eq!(ws.meta.name, "svc");
    assert_eq!(ws.rule_sets.len(), 1);
    let rule = &ws.rule_sets[0].rules[0];
    assert_eq!(rule.payload.url_path, "/health");
    assert_eq!(rule.payload.method, "GET");
    assert_eq!(rule.payload.respond.status, "200 OK");
}
