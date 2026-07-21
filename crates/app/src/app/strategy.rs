use super::*;
use crate::message::Message;
use apimokka_model::settings::Strategy;

fn expert() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::OpenWorkspace("test".into()));
    a
}
fn guided() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Guided,
    ));
    a.update(Message::OpenWorkspace("test".into()));
    a
}

// ── Strategy update ──────────────────────────────────────────────────

#[test]
fn strategy_change_updates_snapshot() {
    let mut a = expert();
    let before = a.snapshot.as_ref().unwrap().root_settings.strategy;
    // Switch to something different
    let new_strategy = if before == Strategy::FirstMatch {
        Strategy::WeightedRandom
    } else {
        Strategy::FirstMatch
    };
    a.update(Message::RuleSetSetStrategy(new_strategy));
    let after = a.snapshot.as_ref().unwrap().root_settings.strategy;
    assert_eq!(after, new_strategy, "strategy should update the snapshot");
}

#[test]
fn weight_changed_updates_rule_payload() {
    let mut a = expert();
    a.update(Message::RuleSetSetStrategy(Strategy::WeightedRandom));
    let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(rule_id));
    a.update(Message::RuleWeightChanged("7".into()));
    let prototype = &a.snapshot.as_ref().unwrap().prototype.rule_extras[&rule_id];
    assert_eq!(prototype.weight, Some(7));
    assert_eq!(
        a.snapshot
            .as_ref()
            .unwrap()
            .find_rule(rule_id)
            .unwrap()
            .1
            .payload
            .weight,
        None
    );
}

#[test]
fn priority_changed_updates_rule_payload() {
    let mut a = expert();
    a.update(Message::RuleSetSetStrategy(Strategy::Priority));
    let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(rule_id));
    a.update(Message::RulePriorityChanged("-5".into()));
    let prototype = &a.snapshot.as_ref().unwrap().prototype.rule_extras[&rule_id];
    assert_eq!(prototype.priority, Some(-5));
    assert_eq!(
        a.snapshot
            .as_ref()
            .unwrap()
            .find_rule(rule_id)
            .unwrap()
            .1
            .payload
            .priority,
        None
    );
}

#[test]
fn invalid_weight_input_leaves_none() {
    let mut a = expert();
    a.update(Message::RuleSetSetStrategy(Strategy::WeightedRandom));
    let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(rule_id));
    a.update(Message::RuleWeightChanged("not-a-number".into()));
    let prototype = &a.snapshot.as_ref().unwrap().prototype.rule_extras[&rule_id];
    assert_eq!(
        prototype.weight, None,
        "non-numeric input should leave weight as None"
    );
}

// ── Layout density (Guided mode) ────────────────────────────────────

#[test]
fn rule_set_config_more_resets_on_guided() {
    let mut a = expert();
    a.rule_set_config_more = true;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Guided,
    ));
    assert!(
        !a.rule_set_config_more,
        "switching to Guided resets rule_set_config_more"
    );
}

#[test]
fn toggle_rule_set_config_more_flips() {
    let mut a = guided();
    assert!(!a.rule_set_config_more);
    a.update(Message::ToggleRuleSetConfigMore);
    assert!(a.rule_set_config_more);
    a.update(Message::ToggleRuleSetConfigMore);
    assert!(!a.rule_set_config_more);
}

// ── View smoke tests ─────────────────────────────────────────────────

#[test]
fn rule_set_config_builds_in_both_modes_and_all_strategies() {
    for mode in apimokka_model::AudienceMode::all() {
        for strategy in Strategy::all() {
            let mut a = App::new().0;
            a.update(Message::ChooseAudienceMode(mode));
            a.update(Message::OpenWorkspace("test".into()));
            a.update(Message::RuleSetSetStrategy(strategy));
            let rs_id = a.snapshot.as_ref().unwrap().rule_sets[0].id;
            a.update(Message::SelectRuleSet(rs_id));
            let _ = crate::screens::routes::view(&a);
        }
    }
}

#[test]
fn rule_editor_builds_with_weight_and_priority_fields() {
    for strategy in [Strategy::WeightedRandom, Strategy::Priority] {
        let mut a = expert();
        a.update(Message::RuleSetSetStrategy(strategy));
        let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        a.update(Message::SelectRule(rule_id));
        let _ = crate::screens::routes::view(&a); // should not panic
    }
}

#[test]
fn validation_strip_builds_for_rule_with_issues() {
    // The mock data has a rule (error-scenarios.toml rules[0]) with a
    // WeightedRandom validation warning — verify the view builds.
    let mut a = expert();
    let snap = a.snapshot.as_ref().unwrap();
    let rule_with_issues = snap
        .rule_sets
        .iter()
        .flat_map(|rs| rs.rules.iter())
        .find(|r| !r.validation.issues.is_empty());
    if let Some(rule) = rule_with_issues {
        let id = rule.id;
        a.update(Message::SelectRule(id));
        let _ = crate::screens::routes::view(&a);
    }
}
