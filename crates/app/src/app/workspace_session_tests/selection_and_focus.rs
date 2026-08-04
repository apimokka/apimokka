//! Selection and condition-focus tracking through edits, deletes, and
//! creation/restore identity changes.

use super::*;
use crate::message::Message;

#[test]
fn condition_focus_binds_pending_identity_and_clears_on_route_change() {
    let mut app = expert();
    let rule = app.selection.rule.unwrap();
    app.update(Message::HeaderAdd);
    let pending = app
        .snapshot
        .as_ref()
        .unwrap()
        .condition_focus
        .clone()
        .unwrap();
    assert_eq!(pending.rule_id, rule);
    assert_eq!(pending.family, ConditionFamily::Header);
    assert!(matches!(pending.binding, DraftBinding::Pending(_)));

    let index = app
        .snapshot
        .as_ref()
        .unwrap()
        .rule_draft(rule)
        .unwrap()
        .header_bindings
        .len()
        - 1;
    app.update(Message::HeaderSetName {
        index,
        value: "x-focus".into(),
    });
    assert!(matches!(
        app.snapshot
            .as_ref()
            .unwrap()
            .condition_focus
            .as_ref()
            .map(|focus| &focus.binding),
        Some(DraftBinding::Existing(_))
    ));

    let other_set = app.snapshot.as_ref().unwrap().rule_sets[1].id;
    app.update(Message::SelectRuleSet(other_set));
    assert!(app.snapshot.as_ref().unwrap().condition_focus.is_none());
}

#[test]
fn selected_removal_falls_back_only_to_the_captured_parent() {
    let mut app = expert();
    let rule = app.selection.rule.unwrap();
    let parent = app.selection.rule_set.unwrap();
    app.update(Message::DeleteRule(rule));
    assert_eq!(app.selection.rule, None);
    assert_eq!(app.selection.rule_set, Some(parent));

    app.update(Message::DeleteRuleSet(parent));
    app.update(Message::ConfirmProceed);
    assert_eq!(app.selection, RouteSelection::default());
}

#[test]
fn created_and_restored_nodes_select_the_receipt_or_verified_rebind() {
    let mut app = expert();

    app.update(Message::AddRuleSet);
    let created_set = app.selection.rule_set.unwrap();
    assert_eq!(app.selection.rule, None);
    assert!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule_set(created_set)
            .is_some()
    );

    let parent = app.snapshot.as_ref().unwrap().rule_sets[0].id;
    app.update(Message::SelectRuleSet(parent));
    app.update(Message::AddRule(parent));
    let created_rule = app.selection.rule.unwrap();
    assert_eq!(app.selection.rule_set, Some(parent));
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule(created_rule)
            .unwrap()
            .0
            .id,
        parent
    );

    app.update(Message::DuplicateRule(created_rule));
    let duplicate = app.selection.rule.unwrap();
    assert_ne!(duplicate, created_rule);
    assert_eq!(app.selection.rule_set, Some(parent));
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule(duplicate)
            .unwrap()
            .0
            .id,
        parent
    );

    app.update(Message::DeleteRule(duplicate));
    assert_eq!(app.selection.rule, None);
    app.update(Message::Undo);
    let rebound = app.selection.rule.unwrap();
    assert_ne!(rebound, duplicate);
    assert_eq!(app.selection.rule_set, Some(parent));
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule(rebound)
            .unwrap()
            .0
            .id,
        parent
    );
}
