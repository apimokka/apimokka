//! Exhaustive undo/redo round-trip coverage for every edit family: header
//! and body conditions, root settings, rules and rule sets, duplication,
//! response fields, and history-depth capping.

use super::*;
use crate::message::Message;

#[test]
fn header_and_body_update_remove_and_clear_round_trip_both_directions() {
    let mut header_app = expert();
    let rule = header_app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rules()
        .iter()
        .find(|rule| !rule.conditions().headers.is_empty())
        .unwrap()
        .rule_id();
    header_app.update(Message::SelectRule(rule));
    assert!(
        !header_app
            .selected_rule_payload()
            .unwrap()
            .headers
            .is_empty()
    );
    let original_name = header_app.selected_rule_payload().unwrap().headers[0]
        .name
        .clone();
    header_app.update(Message::HeaderSetName {
        index: 0,
        value: "X-Round-Trip".into(),
    });
    header_app.update(Message::Undo);
    assert_eq!(
        header_app.selected_rule_payload().unwrap().headers[0].name,
        original_name
    );
    header_app.update(Message::Redo);
    assert_eq!(
        header_app.selected_rule_payload().unwrap().headers[0].name,
        "x-round-trip"
    );
    let header_count = header_app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule)
        .unwrap()
        .conditions()
        .headers
        .len();
    header_app.update(Message::HeaderRemove(0));
    header_app.update(Message::Undo);
    assert_eq!(
        header_app
            .snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule)
            .unwrap()
            .conditions()
            .headers
            .len(),
        header_count
    );
    header_app.update(Message::Redo);
    assert_eq!(
        header_app
            .snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule)
            .unwrap()
            .conditions()
            .headers
            .len(),
        header_count - 1
    );

    let mut body_app = expert();
    let rule = body_app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rules()
        .iter()
        .find(|rule| !rule.conditions().body.is_empty())
        .unwrap()
        .rule_id();
    body_app.update(Message::SelectRule(rule));
    assert!(!body_app.selected_rule_payload().unwrap().body.is_empty());
    let original_path = body_app.selected_rule_payload().unwrap().body[0]
        .path
        .clone();
    body_app.update(Message::BodySetPath {
        index: 0,
        value: "user.name".into(),
    });
    body_app.update(Message::Undo);
    assert_eq!(
        body_app.selected_rule_payload().unwrap().body[0].path,
        original_path
    );
    body_app.update(Message::Redo);
    assert_eq!(
        body_app.selected_rule_payload().unwrap().body[0].path,
        "user.name"
    );
    let body_count = body_app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule)
        .unwrap()
        .conditions()
        .body
        .len();
    body_app.update(Message::BodyRemove(0));
    body_app.update(Message::Undo);
    assert_eq!(
        body_app
            .snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule)
            .unwrap()
            .conditions()
            .body
            .len(),
        body_count
    );
    body_app.update(Message::Redo);
    assert_eq!(
        body_app
            .snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule)
            .unwrap()
            .conditions()
            .body
            .len(),
        body_count - 1
    );
    body_app.update(Message::Undo);
    body_app.update(Message::BodyClearAll);
    body_app.update(Message::Undo);
    assert_eq!(
        body_app
            .snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule)
            .unwrap()
            .conditions()
            .body
            .len(),
        body_count
    );
    body_app.update(Message::Redo);
    assert!(
        body_app
            .snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule)
            .unwrap()
            .conditions()
            .body
            .is_empty()
    );
}

#[test]
fn every_supported_root_key_and_response_redo_round_trip() {
    let mut app = expert();
    let initial = app.snapshot.as_ref().unwrap().root_settings.clone();
    let strategy = if initial.strategy == apimokka_model::settings::Strategy::FirstMatch {
        apimokka_model::settings::Strategy::RoundRobin
    } else {
        apimokka_model::settings::Strategy::FirstMatch
    };
    let edits = [
        Message::SettingsSetHost("127.0.0.2".into()),
        Message::SettingsSetPort("4567".into()),
        Message::SettingsSetTls(!initial.tls_enabled),
        Message::SettingsSetLogLevel("debug".into()),
        Message::SettingsSetStrategy(strategy),
    ];
    for edit in edits {
        app.update(edit);
        app.update(Message::Undo);
        app.update(Message::Redo);
    }
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.listener_ip,
        "127.0.0.2"
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.listener_port,
        4567
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.tls_enabled,
        !initial.tls_enabled
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.log_level,
        "debug"
    );

    let rule = app.selection.rule.unwrap();
    app.update(Message::RespondSetStatus("202 Accepted".into()));
    app.update(Message::Undo);
    app.update(Message::Redo);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule)
            .unwrap()
            .respond()
            .status(),
        Some("202 Accepted")
    );
}

#[test]
fn rule_and_rule_set_add_remove_redo_with_new_identity() {
    let mut app = expert();
    let set = app.selection.rule_set.unwrap();
    let rule_count = app
        .snapshot
        .as_ref()
        .unwrap()
        .find_rule_set(set)
        .unwrap()
        .rules
        .len();
    app.update(Message::AddRule(set));
    let first_added = app.selection.rule.unwrap();
    app.update(Message::Undo);
    app.update(Message::Redo);
    let second_added = app.selection.rule.unwrap();
    assert_ne!(first_added, second_added);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule_set(set)
            .unwrap()
            .rules
            .len(),
        rule_count + 1
    );

    app.update(Message::DeleteRule(second_added));
    app.update(Message::Undo);
    let restored = app.selection.rule.unwrap();
    app.update(Message::Redo);
    assert!(app.snapshot.as_ref().unwrap().find_rule(restored).is_none());

    let set_count = app.snapshot.as_ref().unwrap().rule_sets.len();
    app.update(Message::AddRuleSet);
    let added_set = app.selection.rule_set.unwrap();
    app.update(Message::Undo);
    app.update(Message::Redo);
    let rebound_set = app.selection.rule_set.unwrap();
    assert_ne!(added_set, rebound_set);
    assert_eq!(
        app.snapshot.as_ref().unwrap().rule_sets.len(),
        set_count + 1
    );
    app.update(Message::DeleteRuleSet(rebound_set));
    app.update(Message::ConfirmProceed);
    app.update(Message::Undo);
    let restored_set = app.selection.rule_set.unwrap();
    app.update(Message::Redo);
    assert!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule_set(restored_set)
            .is_none()
    );
}

#[test]
fn duplicate_rule_preserves_subtree_prototype_and_trace_history() {
    let mut app = expert();
    let source = app.selection.rule.unwrap();
    let source_conditions = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(source)
        .unwrap()
        .conditions()
        .clone();
    let source_prototype = app.snapshot.as_ref().unwrap().prototype.rule_extras[&source].clone();
    app.update(Message::DuplicateRule(source));
    let duplicate = app.selection.rule.unwrap();
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(duplicate)
            .unwrap()
            .conditions()
            .headers
            .len(),
        source_conditions.headers.len()
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().prototype.rule_extras[&duplicate],
        source_prototype
    );
    app.update(Message::Undo);
    app.update(Message::Redo);
    let rebound = app.selection.rule.unwrap();
    assert_eq!(
        app.snapshot.as_ref().unwrap().prototype.rule_extras[&rebound],
        source_prototype
    );

    let trace_before = app
        .snapshot
        .as_ref()
        .unwrap()
        .prototype
        .trace
        .clone()
        .unwrap();
    app.update(Message::SettingsSetTraceEnabled(!trace_before.enabled));
    app.update(Message::Undo);
    assert_eq!(
        app.snapshot.as_ref().unwrap().prototype.trace.as_ref(),
        Some(&trace_before)
    );
    app.update(Message::Redo);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .prototype
            .trace
            .as_ref()
            .unwrap()
            .enabled,
        !trace_before.enabled
    );
}

#[test]
fn root_draft_rejection_and_history_keep_identity_consistent() {
    let mut app = expert();
    let original = app
        .snapshot
        .as_ref()
        .unwrap()
        .root_settings
        .listener_ip
        .clone();
    app.update(Message::SettingsSetHost("invalid-ip".into()));
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_drafts.listener_ip,
        "invalid-ip"
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.listener_ip,
        original
    );

    app.update(Message::SettingsSetHost("127.0.0.2".into()));
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.listener_ip,
        "127.0.0.2"
    );
    app.update(Message::Undo);
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.listener_ip,
        original
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_drafts.listener_ip,
        original
    );
}

#[test]
fn response_invalid_delay_stays_draft_and_valid_edit_round_trips() {
    let mut app = expert();
    let rule_id = app.selection.rule.unwrap();
    let before = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule_id)
        .unwrap()
        .respond()
        .clone();
    app.update(Message::RespondSetDelay("-1".into()));
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .rule_draft(rule_id)
            .unwrap()
            .response_delay,
        "-1"
    );
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule_id)
            .unwrap()
            .respond(),
        &before
    );

    app.update(Message::RespondSetDelay("25".into()));
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule_id)
            .unwrap()
            .respond()
            .delay_milliseconds(),
        Some(25)
    );
    app.update(Message::Undo);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule_id)
            .unwrap()
            .respond(),
        &before
    );
}

#[test]
fn clear_conditions_is_one_exact_undo_step() {
    let mut app = expert();
    let rule_id = app.selection.rule.unwrap();
    app.update(Message::HeaderAdd);
    let index = app
        .snapshot
        .as_ref()
        .unwrap()
        .rule_draft(rule_id)
        .unwrap()
        .payload
        .headers
        .len()
        - 1;
    app.update(Message::HeaderSetName {
        index,
        value: "x-clear-test".into(),
    });
    let before = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule_id)
        .unwrap()
        .conditions()
        .headers
        .iter()
        .map(|condition| condition.condition.clone())
        .collect::<Vec<_>>();
    assert!(!before.is_empty());
    let history = app.undo_stack().len();
    app.update(Message::HeaderClearAll);
    assert!(
        app.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule_id)
            .unwrap()
            .conditions()
            .headers
            .is_empty()
    );
    assert_eq!(app.undo_stack().len(), history + 1);
    app.update(Message::Undo);
    let after = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule_id)
        .unwrap()
        .conditions()
        .headers
        .iter()
        .map(|condition| condition.condition.clone())
        .collect::<Vec<_>>();
    assert_eq!(after, before);
}

#[test]
fn semantic_history_is_capped_at_fifty_entries() {
    let mut app = expert();
    for value in 0..55 {
        app.update(Message::RuleWeightChanged(value.to_string()));
    }
    assert_eq!(app.undo_stack().len(), 50);
}

#[test]
fn move_up_undo_redo_uses_the_recorded_after_index() {
    let mut app = expert();
    let moved = app.snapshot.as_ref().unwrap().rule_sets[0].rules[1].id;
    app.update(Message::MoveRuleUp(moved));
    assert_eq!(
        app.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id,
        moved
    );
    app.update(Message::Undo);
    assert_eq!(
        app.snapshot.as_ref().unwrap().rule_sets[0].rules[1].id,
        moved
    );
    app.update(Message::Redo);
    assert_eq!(
        app.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id,
        moved
    );
}
