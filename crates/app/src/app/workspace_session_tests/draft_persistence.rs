//! Draft survival and history rebinding under interleaved operations:
//! unrelated drafts persisting across edits, transient problems dismissed
//! only by their own correction, semantic no-ops not polluting history, and
//! delete+undo rebinding identity correctly.

use super::*;
use crate::message::Message;
use apimokka_model::RuntimeEffect;

#[test]
fn invalid_rule_draft_survives_without_changing_canonical_state() {
    let mut app = expert();
    let rule_id = app.selection.rule.unwrap();
    let canonical_before = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule_id)
        .unwrap()
        .rule_match()
        .clone();

    app.update(Message::RuleSetMethod("PATCH".into()));

    let session = app.snapshot.as_ref().unwrap();
    assert_eq!(session.rule_draft(rule_id).unwrap().payload.method, "PATCH");
    assert_eq!(
        session.latest().rule(rule_id).unwrap().rule_match(),
        &canonical_before
    );
    assert!(app.last_problem.is_some());
}

#[test]
fn only_the_corrected_operation_dismisses_its_transient_problem() {
    let mut app = expert();

    app.update(Message::SettingsSetPort("abc".into()));
    assert_eq!(
        app.transient_problem_kind,
        Some(TransientProblemKind::Operation)
    );
    let rejected_title = app.last_problem.as_ref().unwrap().title.clone();

    app.update(Message::RuleSetMethod("POST".into()));
    assert_eq!(
        app.last_problem.as_ref().unwrap().title,
        rejected_title,
        "an unrelated successful edit must retain the field problem"
    );

    app.update(Message::SettingsSetPort("9000".into()));
    assert!(app.last_problem.is_none());
    assert_eq!(app.transient_problem_kind, None);
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.listener_port,
        9000
    );
}

#[test]
fn pending_condition_draft_survives_navigation() {
    let mut app = expert();
    let rule_id = app.selection.rule.unwrap();
    let other = app.snapshot.as_ref().unwrap().rule_sets[0].rules[1].id;
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
        value: "not a header".into(),
    });
    app.update(Message::SelectRule(other));
    app.update(Message::SelectRule(rule_id));

    let draft = app.snapshot.as_ref().unwrap().rule_draft(rule_id).unwrap();
    assert_eq!(draft.payload.headers[index].name, "not a header");
    assert!(matches!(
        draft.header_bindings[index],
        DraftBinding::Pending(_)
    ));
}

#[test]
fn semantic_noop_does_not_record_history_or_dirty_an_extra_file() {
    let mut app = expert();
    let method = app.selected_rule_payload().unwrap().method.clone();
    let history = app.undo_stack().len();
    let dirty = app.dirty_count;

    app.update(Message::RuleSetMethod(method));

    assert_eq!(app.undo_stack().len(), history);
    assert_eq!(app.dirty_count, dirty);
}

#[test]
fn absent_delay_remains_absent_when_an_unrelated_response_field_changes() {
    let mut app = expert();
    let rule_id = app.selection.rule.unwrap();
    app.update(Message::RespondSetDelay(String::new()));
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule_id)
            .unwrap()
            .respond()
            .delay_milliseconds(),
        None
    );
    app.snapshot.as_mut().unwrap().rule_drafts.remove(&rule_id);

    app.update(Message::RespondSetStatus("201 Created".into()));

    let session = app.snapshot.as_ref().unwrap();
    assert_eq!(
        session
            .latest()
            .rule(rule_id)
            .unwrap()
            .respond()
            .delay_milliseconds(),
        None
    );
    assert_eq!(session.rule_draft(rule_id).unwrap().response_delay, "");
}

#[test]
fn semantic_noop_projects_the_accepted_root_value_without_history_or_dirty_change() {
    let mut app = expert();
    let history = app.undo_stack().len();
    let dirty = app.dirty_count;
    let port = app.snapshot.as_ref().unwrap().root_settings.listener_port;
    let runtime_pending = app.snapshot.as_ref().unwrap().latest().runtime_pending();
    let unsaved_hint = app.snapshot.as_ref().unwrap().latest().unsaved_hint();

    app.update(Message::SettingsSetPort(format!("0{port}")));

    assert_eq!(
        app.snapshot.as_ref().unwrap().root_drafts.listener_port,
        port.to_string()
    );
    assert_eq!(app.undo_stack().len(), history);
    assert_eq!(app.dirty_count, dirty);
    assert_eq!(
        app.snapshot.as_ref().unwrap().latest().runtime_pending(),
        runtime_pending
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().latest().unsaved_hint(),
        unsaved_hint
    );
}

#[test]
fn undo_and_redo_preserve_unrelated_invalid_rule_and_root_drafts() {
    let mut app = expert();
    let rule_id = app.selection.rule.unwrap();
    app.update(Message::RuleSetMethod("PATCH".into()));
    app.update(Message::SettingsSetHost("invalid-ip".into()));
    app.update(Message::RespondSetStatus("201 Created".into()));
    app.update(Message::RespondSetDelay("-1".into()));

    app.update(Message::Undo);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .rule_draft(rule_id)
            .unwrap()
            .payload
            .method,
        "PATCH"
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_drafts.listener_ip,
        "invalid-ip"
    );
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .rule_draft(rule_id)
            .unwrap()
            .response_delay,
        "-1"
    );

    app.update(Message::Redo);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .rule_draft(rule_id)
            .unwrap()
            .payload
            .method,
        "PATCH"
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_drafts.listener_ip,
        "invalid-ip"
    );
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .rule_draft(rule_id)
            .unwrap()
            .response_delay,
        "-1"
    );

    let mut rule_app = expert();
    let rule_id = rule_app.selection.rule.unwrap();
    rule_app.update(Message::RuleSetUrlPath("/field-scoped".into()));
    rule_app.update(Message::RuleSetMethod("PATCH".into()));
    rule_app.update(Message::Undo);
    assert_eq!(
        rule_app
            .snapshot
            .as_ref()
            .unwrap()
            .rule_draft(rule_id)
            .unwrap()
            .payload
            .method,
        "PATCH"
    );
}

#[test]
fn refresh_acknowledgement_preserves_unrelated_invalid_drafts() {
    let mut app = expert();
    app.update(Message::SettingsSetPort("9000".into()));
    app.update(Message::Save);
    let rule = app.selection.rule.unwrap();
    app.update(Message::RuleSetMethod("PATCH".into()));
    app.update(Message::SettingsSetHost("invalid-ip".into()));
    match app.runtime_phase() {
        RuntimeEffect::Reload => app.update(Message::ReloadConfig),
        RuntimeEffect::Restart => app.update(Message::RestartServer),
        RuntimeEffect::None => panic!("saved runtime-affecting edit must expose an action"),
    }

    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .rule_draft(rule)
            .unwrap()
            .payload
            .method,
        "PATCH"
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_drafts.listener_ip,
        "invalid-ip"
    );
}

#[test]
fn delete_restore_rebinds_older_rule_history() {
    let mut app = expert();
    let old_id = app.selection.rule.unwrap();
    let original = app.selected_rule_payload().unwrap().url_path.clone();
    app.update(Message::RuleSetUrlPath("/history-rebind".into()));
    app.update(Message::DeleteRule(old_id));
    app.update(Message::Undo);
    let rebound = app.selection.rule.unwrap();
    assert_ne!(rebound, old_id);

    app.update(Message::Undo);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule(rebound)
            .unwrap()
            .1
            .payload
            .url_path,
        original
    );
}

#[test]
fn repeated_restore_rebinds_history_from_the_previous_live_generation() {
    let mut app = expert();
    let original = app.selected_rule_payload().unwrap().url_path.clone();
    let first = app.selection.rule.unwrap();
    app.update(Message::RuleSetUrlPath("/generation".into()));
    app.update(Message::DeleteRule(first));
    app.update(Message::Undo);
    app.update(Message::Undo);
    app.update(Message::Redo);
    app.update(Message::Redo);
    app.update(Message::Undo);
    let third = app.selection.rule.unwrap();
    app.update(Message::Undo);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule(third)
            .unwrap()
            .1
            .payload
            .url_path,
        original
    );
}

#[test]
fn remove_rule_set_undo_restores_exact_root_position() {
    let mut app = expert();
    app.update(Message::AddRuleSet);
    let ids = app
        .snapshot
        .as_ref()
        .unwrap()
        .rule_sets
        .iter()
        .map(|set| set.id)
        .collect::<Vec<_>>();
    let removed = ids[0];
    app.update(Message::DeleteRuleSet(removed));
    app.update(Message::ConfirmProceed);
    app.update(Message::Undo);

    assert_eq!(
        app.snapshot.as_ref().unwrap().rule_sets[0].file.path,
        "rules/main.toml"
    );
}

#[test]
fn prototype_edit_is_history_backed_but_not_workspace_dirty() {
    let mut app = expert();
    app.update(Message::Save);
    let rule_id = app.selection.rule.unwrap();
    let dirty_before = app.dirty_count;
    let before = app.snapshot.as_ref().unwrap().prototype.rule_extras[&rule_id].clone();

    app.update(Message::RuleWeightChanged("17".into()));
    assert_eq!(app.dirty_count, dirty_before);
    assert_eq!(
        app.snapshot.as_ref().unwrap().prototype.rule_extras[&rule_id].weight,
        Some(17)
    );
    app.update(Message::Undo);
    assert_eq!(
        app.snapshot.as_ref().unwrap().prototype.rule_extras[&rule_id],
        before
    );
}

#[test]
fn condition_add_undo_redo_uses_new_identity() {
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
        value: "X-New".into(),
    });
    let first_id = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule_id)
        .unwrap()
        .conditions()
        .headers
        .last()
        .unwrap()
        .id;
    app.update(Message::Undo);
    app.update(Message::Redo);
    let second_id = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule_id)
        .unwrap()
        .conditions()
        .headers
        .last()
        .unwrap()
        .id;
    assert_ne!(first_id, second_id);
}
