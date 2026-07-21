use std::collections::HashSet;

use super::mapping::{project_body_condition, project_header_condition};
use super::*;
use crate::mock::minimal_workspace;
use crate::rule::{BodyOp, HeaderOp, UrlPathOp};
use crate::validation::{Diagnostic, Severity};
use crate::{ConfigFileKind, NodeValidation, Strategy, ValidationIssue};

fn workspace() -> MemoryWorkspace {
    MemoryWorkspace::new(minimal_workspace("test", "127.0.0.1", 3000, false)).unwrap()
}

fn transaction(intent: EditIntent) -> EditTransaction {
    EditTransaction::new(vec![intent]).unwrap()
}

fn rule_payload(path: &str, method: &str) -> RuleEditPayload {
    RuleEditPayload {
        rule_match: map_rule_match(path, Some(UrlPathOp::Equal), method).unwrap(),
        headers: CollectionEdit::Preserve,
        body: CollectionEdit::Preserve,
        respond: map_response(ResponseMode::Inline, "ok", "", "200 OK", "").unwrap(),
    }
}

fn rule_set_archive(path: &str, insertion_index: usize) -> ArchivedSubtree {
    let old_id = NodeId::new();
    ArchivedSubtree::new(
        old_id,
        RestorePlacement::RuleSetRoot { insertion_index },
        vec![ArchivedNode {
            old_id,
            parent: None,
            key: SemanticCreationKey::new(format!("restore/{path}")).unwrap(),
            payload: ArchivedNodePayload::RuleSet {
                path: parse_rule_set_path(path).unwrap(),
            },
        }],
    )
    .unwrap()
}

#[test]
fn port_snapshot_aligns_legacy_and_canonical_rules_with_unique_ids() {
    let port = workspace();
    let snapshot = port.snapshot();
    assert_eq!(snapshot.rules().len(), 1);
    let legacy = &snapshot.workspace().rule_sets[0].rules[0];
    let canonical = snapshot.rule(legacy.id).unwrap();
    assert_eq!(canonical.rule_id(), legacy.id);
    assert_eq!(canonical.rule_match().url_path(), Some("/health"));
    assert_eq!(canonical.respond().text(), Some(r#"{"status":"ok"}"#));

    let mut ids = HashSet::new();
    for rule_set in &snapshot.workspace().rule_sets {
        assert!(ids.insert(rule_set.id.0));
        for rule in &rule_set.rules {
            assert!(ids.insert(rule.id));
            let canonical = snapshot.rule(rule.id).unwrap();
            for condition in &canonical.conditions().headers {
                assert!(ids.insert(condition.id));
            }
            for condition in &canonical.conditions().body {
                assert!(ids.insert(condition.id));
            }
        }
    }
}

#[test]
fn legacy_import_validates_root_settings_and_rule_set_file_kind() {
    let mut accepted = minimal_workspace("test", "127.0.0.1", 3000, false);
    accepted.root_settings.listener_ip = "::1".into();
    accepted.root_settings.listener_port = 65535;
    accepted.root_settings.fallback_respond_dir = "responses/alt".into();
    accepted.root_settings.strategy = Strategy::RoundRobin;
    accepted.root_settings.tls_enabled = true;
    accepted.root_settings.tls_cert_file = "certs/cert.pem".into();
    accepted.root_settings.tls_key_file = "certs/key.pem".into();
    accepted.root_settings.log_level = "trace".into();
    accepted.root_settings.log_file = "logs/app.log".into();
    accepted.root_settings.log_format = "json".into();
    accepted.root_settings.file_tree_show_hidden = true;
    accepted.root_settings.file_tree_builtin_excludes = false;
    accepted.root_settings.file_tree_extra_excludes = vec!["target".into()];
    accepted.root_settings.file_tree_include = vec!["rules".into()];
    assert!(MemoryWorkspace::new(accepted).is_ok());

    let mut invalid = minimal_workspace("test", "127.0.0.1", 3000, false);
    invalid.root_settings.listener_ip = "localhost".into();
    assert!(MemoryWorkspace::new(invalid).is_err());

    let mut invalid = minimal_workspace("test", "127.0.0.1", 3000, false);
    invalid.root_settings.listener_port = 0;
    assert!(MemoryWorkspace::new(invalid).is_err());

    let mut invalid = minimal_workspace("test", "127.0.0.1", 3000, false);
    invalid.root_settings.tls_cert_file = "../cert.pem".into();
    assert!(MemoryWorkspace::new(invalid).is_err());

    let mut invalid = minimal_workspace("test", "127.0.0.1", 3000, false);
    invalid.root_settings.log_level = "verbose".into();
    assert!(MemoryWorkspace::new(invalid).is_err());

    let mut invalid = minimal_workspace("test", "127.0.0.1", 3000, false);
    invalid.root_settings.file_tree_include = vec![String::new()];
    assert!(MemoryWorkspace::new(invalid).is_err());

    let mut invalid = minimal_workspace("test", "127.0.0.1", 3000, false);
    invalid.rule_sets[0].file.kind = ConfigFileKind::Root;
    assert!(MemoryWorkspace::new(invalid).is_err());
}

#[test]
fn every_header_projection_round_trips_to_the_same_canonical_value() {
    let operators = [
        HeaderOp::Equal,
        HeaderOp::Contains,
        HeaderOp::StartsWith,
        HeaderOp::EndsWith,
        HeaderOp::Regex,
        HeaderOp::Exists,
        HeaderOp::Absent,
        HeaderOp::NotEqual,
        HeaderOp::WildCard,
    ];
    for op in operators {
        let draft = "";
        let canonical = map_header_condition("X-Request-ID", op, draft).unwrap();
        let legacy = project_header_condition(&canonical);
        assert_eq!(legacy.name, "x-request-id");
        assert_eq!(
            map_header_condition(&legacy.name, legacy.op, &legacy.value).unwrap(),
            canonical
        );
    }
    let canonical = map_header_condition("X-A", HeaderOp::Equal, "Value").unwrap();
    assert_eq!(project_header_condition(&canonical).value, "Value");
}

#[test]
fn every_body_projection_round_trips_and_json_is_recursively_canonical() {
    let cases = [
        (BodyOp::Equal, ""),
        (BodyOp::EqualString, "text"),
        (BodyOp::Contains, "text"),
        (BodyOp::StartsWith, "text"),
        (BodyOp::EndsWith, "text"),
        (BodyOp::Regex, "text"),
        (
            BodyOp::EqualTyped,
            r#"{ "z": {"b": 1, "a": 2}, "a": [3, {"y": 1, "x": 2}] }"#,
        ),
        (BodyOp::ArrayContains, r#"[{"b":2,"a":1}]"#),
        (BodyOp::EqualNumber, "1.5e2"),
        (BodyOp::GreaterThan, "2.5"),
        (BodyOp::LessThan, "-2.5"),
        (BodyOp::GreaterOrEqual, "3"),
        (BodyOp::LessOrEqual, "4"),
        (BodyOp::EqualInteger, "-42"),
        (BodyOp::ArrayLengthEqual, "0"),
        (BodyOp::ArrayLengthAtLeast, "42"),
        (BodyOp::Exists, ""),
        (BodyOp::Absent, ""),
    ];
    for (op, draft) in cases {
        let canonical = map_body_condition("items.0", op, draft).unwrap();
        let legacy = project_body_condition(&canonical);
        assert_eq!(
            map_body_condition(&legacy.path, legacy.op, &legacy.value).unwrap(),
            canonical,
            "failed {op:?}"
        );
        if op == BodyOp::EqualTyped {
            assert_eq!(legacy.value, r#"{"a":[3,{"x":2,"y":1}],"z":{"a":2,"b":1}}"#);
        }
    }
    let escaped = map_body_condition(
        "a",
        BodyOp::EqualTyped,
        r#"{"s":"line\n\"quote","a":[2,1]}"#,
    )
    .unwrap();
    assert_eq!(
        project_body_condition(&escaped).value,
        r#"{"a":[2,1],"s":"line\n\"quote"}"#
    );
}

#[test]
fn absent_and_zero_delay_remain_distinct_in_the_canonical_view() {
    let mut port = workspace();
    let rule_id = port.snapshot().rules()[0].rule_id();
    port.apply(transaction(EditIntent::UpdateRespond {
        id: rule_id,
        respond: map_response(ResponseMode::Inline, "ok", "", "", "").unwrap(),
    }))
    .unwrap();
    let absent = port.snapshot();
    assert_eq!(
        absent.rule(rule_id).unwrap().respond().delay_milliseconds(),
        None
    );
    assert_eq!(
        absent
            .workspace()
            .find_rule(rule_id)
            .unwrap()
            .1
            .payload
            .respond
            .delay_milliseconds,
        0
    );

    port.apply(transaction(EditIntent::UpdateRespond {
        id: rule_id,
        respond: map_response(ResponseMode::Inline, "ok", "", "", "0").unwrap(),
    }))
    .unwrap();
    assert_eq!(
        port.snapshot()
            .rule(rule_id)
            .unwrap()
            .respond()
            .delay_milliseconds(),
        Some(0)
    );
}

#[test]
fn failed_compound_apply_is_atomic() {
    let mut port = workspace();
    let before = port.snapshot();
    let edit = map_root_setting(
        WorkspaceRootKey::ListenerPort,
        WorkspaceEditValue::Integer(4000),
    )
    .unwrap();
    let transaction = EditTransaction::new(vec![
        EditIntent::UpdateRootSetting(edit),
        EditIntent::DeleteRule { id: NodeId::new() },
    ])
    .unwrap();
    assert!(port.apply(transaction).is_err());
    let after = port.snapshot();
    assert_eq!(
        after.workspace().root_settings.listener_port,
        before.workspace().root_settings.listener_port
    );
    assert!(after.dirty_files().is_empty());
}

#[test]
fn transaction_order_is_preserved_and_typed_failures_leave_state_unchanged() {
    let mut port = workspace();
    let first = map_root_setting(
        WorkspaceRootKey::LogLevel,
        WorkspaceEditValue::Enum("debug".into()),
    )
    .unwrap();
    let second = map_root_setting(
        WorkspaceRootKey::LogLevel,
        WorkspaceEditValue::Enum("trace".into()),
    )
    .unwrap();
    port.apply(
        EditTransaction::new(vec![
            EditIntent::UpdateRootSetting(first),
            EditIntent::UpdateRootSetting(second),
        ])
        .unwrap(),
    )
    .unwrap();
    assert_eq!(port.snapshot().workspace().root_settings.log_level, "trace");

    let before_count = port.snapshot().workspace().rule_sets.len();
    let existing_path = parse_rule_set_path("rules/main.toml").unwrap();
    assert!(
        port.apply(transaction(EditIntent::AddRuleSet {
            path: existing_path,
            key: SemanticCreationKey::new("duplicate").unwrap(),
        }))
        .is_err()
    );
    assert_eq!(port.snapshot().workspace().rule_sets.len(), before_count);
}

#[test]
fn adds_return_typed_receipts_for_duplicate_equal_conditions() {
    let mut port = workspace();
    let rule_id = port.snapshot().rules()[0].rule_id();
    let first = map_body_condition("a", BodyOp::Equal, "same").unwrap();
    let second = first.clone();
    let outcome = port
        .apply(
            EditTransaction::new(vec![
                EditIntent::AddBodyCondition {
                    rule_id,
                    condition: first,
                    key: SemanticCreationKey::new("body[0]").unwrap(),
                },
                EditIntent::AddBodyCondition {
                    rule_id,
                    condition: second,
                    key: SemanticCreationKey::new("body[1]").unwrap(),
                },
            ])
            .unwrap(),
        )
        .unwrap();
    assert_eq!(outcome.creations.len(), 2);
    assert_ne!(outcome.creations[0].new_id, outcome.creations[1].new_id);
    assert_eq!(
        outcome
            .snapshot
            .rule(rule_id)
            .unwrap()
            .conditions()
            .body
            .len(),
        2
    );
}

#[test]
fn add_rule_correlates_duplicate_equal_nested_conditions_by_key() {
    let mut port = workspace();
    let parent = port.snapshot().workspace().rule_sets[0].id;
    let header = map_header_condition("x-a", HeaderOp::Equal, "same").unwrap();
    let body = map_body_condition("a", BodyOp::Equal, "same").unwrap();
    let payload = RuleEditPayload {
        rule_match: map_rule_match("/nested", Some(UrlPathOp::Equal), "POST").unwrap(),
        headers: CollectionEdit::Replace(vec![
            ConditionEdit::Create {
                key: SemanticCreationKey::new("rule/header[0]").unwrap(),
                condition: header.clone(),
            },
            ConditionEdit::Create {
                key: SemanticCreationKey::new("rule/header[1]").unwrap(),
                condition: header,
            },
        ]),
        body: CollectionEdit::Replace(vec![
            ConditionEdit::Create {
                key: SemanticCreationKey::new("rule/body[0]").unwrap(),
                condition: body.clone(),
            },
            ConditionEdit::Create {
                key: SemanticCreationKey::new("rule/body[1]").unwrap(),
                condition: body,
            },
        ]),
        respond: map_response(ResponseMode::Inline, "ok", "", "", "").unwrap(),
    };
    let outcome = port
        .apply(transaction(EditIntent::AddRule {
            parent,
            insertion_index: 1,
            rule: payload,
            key: SemanticCreationKey::new("rule").unwrap(),
        }))
        .unwrap();
    assert_eq!(outcome.creations.len(), 5);
    assert_eq!(outcome.creations[0].key.as_str(), "rule");
    let ids = outcome
        .creations
        .iter()
        .map(|receipt| receipt.new_id)
        .collect::<HashSet<_>>();
    assert_eq!(ids.len(), 5);
    let rule_id = outcome.creations[0].new_id;
    let rule = outcome.snapshot.rule(rule_id).unwrap();
    assert_eq!(rule.conditions().headers.len(), 2);
    assert_eq!(rule.conditions().body.len(), 2);
}

#[test]
fn nested_condition_creation_keys_are_transaction_unique_with_parent_keys() {
    let port = workspace();
    let parent = port.snapshot().workspace().rule_sets[0].id;
    let duplicate = SemanticCreationKey::new("duplicate").unwrap();
    let payload = RuleEditPayload {
        rule_match: map_rule_match("/nested", Some(UrlPathOp::Equal), "POST").unwrap(),
        headers: CollectionEdit::Replace(vec![ConditionEdit::Create {
            key: duplicate.clone(),
            condition: map_header_condition("x-a", HeaderOp::Equal, "v").unwrap(),
        }]),
        body: CollectionEdit::Preserve,
        respond: map_response(ResponseMode::Inline, "ok", "", "", "").unwrap(),
    };
    assert!(
        EditTransaction::new(vec![EditIntent::AddRule {
            parent,
            insertion_index: 1,
            rule: payload,
            key: duplicate,
        }])
        .is_err()
    );
}

#[test]
fn update_rule_replacement_preserves_existing_ids_and_receipts_new_ids() {
    let mut port = workspace();
    let rule_id = port.snapshot().rules()[0].rule_id();
    let equal = map_body_condition("a", BodyOp::Equal, "same").unwrap();
    let created = port
        .apply(
            EditTransaction::new(vec![
                EditIntent::AddBodyCondition {
                    rule_id,
                    condition: equal.clone(),
                    key: SemanticCreationKey::new("old[0]").unwrap(),
                },
                EditIntent::AddBodyCondition {
                    rule_id,
                    condition: equal,
                    key: SemanticCreationKey::new("old[1]").unwrap(),
                },
            ])
            .unwrap(),
        )
        .unwrap();
    let first = created.creations[0].new_id;
    let second = created.creations[1].new_id;
    let current = created.snapshot.rule(rule_id).unwrap();
    let payload = RuleEditPayload {
        rule_match: current.rule_match().clone(),
        headers: CollectionEdit::Preserve,
        body: CollectionEdit::Replace(vec![
            ConditionEdit::Existing {
                id: second,
                condition: map_body_condition("a", BodyOp::Equal, "updated").unwrap(),
            },
            ConditionEdit::Create {
                key: SemanticCreationKey::new("replacement[1]").unwrap(),
                condition: map_body_condition("b", BodyOp::Exists, "").unwrap(),
            },
        ]),
        respond: current.respond().clone(),
    };
    let outcome = port
        .apply(transaction(EditIntent::UpdateRule {
            id: rule_id,
            rule: payload,
        }))
        .unwrap();
    assert_eq!(outcome.creations.len(), 1);
    assert_eq!(outcome.creations[0].key.as_str(), "replacement[1]");
    let replacement = outcome.creations[0].new_id;
    let body = &outcome.snapshot.rule(rule_id).unwrap().conditions().body;
    assert_eq!(body[0].id, second);
    assert_eq!(body[1].id, replacement);
    assert!(!body.iter().any(|condition| condition.id == first));
    assert_eq!(
        outcome.changed_nodes,
        vec![rule_id, first, second, replacement]
    );
}

#[test]
fn condition_replacement_rejects_cross_rule_family_and_new_rule_existing_ids() {
    let mut port = workspace();
    let parent = port.snapshot().workspace().rule_sets[0].id;
    let first_rule = port.snapshot().rules()[0].rule_id();
    let header_id = port
        .apply(transaction(EditIntent::AddHeaderCondition {
            rule_id: first_rule,
            condition: map_header_condition("x-a", HeaderOp::Equal, "v").unwrap(),
            key: SemanticCreationKey::new("header").unwrap(),
        }))
        .unwrap()
        .creations[0]
        .new_id;
    let second_rule = port
        .apply(transaction(EditIntent::AddRule {
            parent,
            insertion_index: 1,
            rule: rule_payload("/second", "GET"),
            key: SemanticCreationKey::new("second-rule").unwrap(),
        }))
        .unwrap()
        .creations[0]
        .new_id;
    let second_body = port
        .apply(transaction(EditIntent::AddBodyCondition {
            rule_id: second_rule,
            condition: map_body_condition("a", BodyOp::Equal, "v").unwrap(),
            key: SemanticCreationKey::new("second-body").unwrap(),
        }))
        .unwrap()
        .creations[0]
        .new_id;

    let current = port.snapshot();
    let first = current.rule(first_rule).unwrap();
    let wrong_family = RuleEditPayload {
        rule_match: first.rule_match().clone(),
        headers: CollectionEdit::Preserve,
        body: CollectionEdit::Replace(vec![ConditionEdit::Existing {
            id: header_id,
            condition: map_body_condition("a", BodyOp::Equal, "v").unwrap(),
        }]),
        respond: first.respond().clone(),
    };
    assert!(
        port.apply(transaction(EditIntent::UpdateRule {
            id: first_rule,
            rule: wrong_family,
        }))
        .is_err()
    );

    let current = port.snapshot();
    let first = current.rule(first_rule).unwrap();
    let wrong_rule = RuleEditPayload {
        rule_match: first.rule_match().clone(),
        headers: CollectionEdit::Preserve,
        body: CollectionEdit::Replace(vec![ConditionEdit::Existing {
            id: second_body,
            condition: map_body_condition("a", BodyOp::Equal, "v").unwrap(),
        }]),
        respond: first.respond().clone(),
    };
    assert!(
        port.apply(transaction(EditIntent::UpdateRule {
            id: first_rule,
            rule: wrong_rule,
        }))
        .is_err()
    );

    let existing_on_new_rule = RuleEditPayload {
        rule_match: map_rule_match("/invalid", Some(UrlPathOp::Equal), "GET").unwrap(),
        headers: CollectionEdit::Replace(vec![ConditionEdit::Existing {
            id: header_id,
            condition: map_header_condition("x-a", HeaderOp::Equal, "v").unwrap(),
        }]),
        body: CollectionEdit::Preserve,
        respond: map_response(ResponseMode::Inline, "", "", "", "").unwrap(),
    };
    assert!(
        port.apply(transaction(EditIntent::AddRule {
            parent,
            insertion_index: 2,
            rule: existing_on_new_rule,
            key: SemanticCreationKey::new("invalid-rule").unwrap(),
        }))
        .is_err()
    );
}

#[test]
fn update_rule_changed_nodes_distinguish_preserve_clear_and_replace() {
    let mut port = workspace();
    let rule_id = port.snapshot().rules()[0].rule_id();
    let header_id = port
        .apply(transaction(EditIntent::AddHeaderCondition {
            rule_id,
            condition: map_header_condition("x-a", HeaderOp::Equal, "v").unwrap(),
            key: SemanticCreationKey::new("header").unwrap(),
        }))
        .unwrap()
        .creations[0]
        .new_id;
    let snapshot = port.snapshot();
    let current = snapshot.rule(rule_id).unwrap();
    let preserve = RuleEditPayload {
        rule_match: current.rule_match().clone(),
        headers: CollectionEdit::Preserve,
        body: CollectionEdit::Preserve,
        respond: current.respond().clone(),
    };
    assert_eq!(
        port.apply(transaction(EditIntent::UpdateRule {
            id: rule_id,
            rule: preserve,
        }))
        .unwrap()
        .changed_nodes,
        vec![rule_id]
    );

    let current = port.snapshot();
    let current = current.rule(rule_id).unwrap();
    let clear = RuleEditPayload {
        rule_match: current.rule_match().clone(),
        headers: CollectionEdit::Clear,
        body: CollectionEdit::Preserve,
        respond: current.respond().clone(),
    };
    assert_eq!(
        port.apply(transaction(EditIntent::UpdateRule {
            id: rule_id,
            rule: clear,
        }))
        .unwrap()
        .changed_nodes,
        vec![rule_id, header_id]
    );
}

#[test]
fn condition_rule_and_rule_set_mutation_families_are_supported() {
    let mut port = workspace();
    let parent = port.snapshot().workspace().rule_sets[0].id;
    let rule_id = port.snapshot().rules()[0].rule_id();

    let header_id = port
        .apply(transaction(EditIntent::AddHeaderCondition {
            rule_id,
            condition: map_header_condition("x-a", HeaderOp::Equal, "one").unwrap(),
            key: SemanticCreationKey::new("header").unwrap(),
        }))
        .unwrap()
        .creations[0]
        .new_id;
    port.apply(transaction(EditIntent::UpdateHeaderCondition {
        id: header_id,
        condition: map_header_condition("x-a", HeaderOp::Contains, "two").unwrap(),
    }))
    .unwrap();
    assert_eq!(
        port.snapshot().rule(rule_id).unwrap().conditions().headers[0]
            .condition
            .expected(),
        Some("two")
    );
    port.apply(transaction(EditIntent::RemoveHeaderCondition {
        id: header_id,
    }))
    .unwrap();

    let body_id = port
        .apply(transaction(EditIntent::AddBodyCondition {
            rule_id,
            condition: map_body_condition("a", BodyOp::Equal, "one").unwrap(),
            key: SemanticCreationKey::new("body").unwrap(),
        }))
        .unwrap()
        .creations[0]
        .new_id;
    port.apply(transaction(EditIntent::UpdateBodyCondition {
        id: body_id,
        condition: map_body_condition("a", BodyOp::EqualInteger, "2").unwrap(),
    }))
    .unwrap();
    assert_eq!(
        port.snapshot().rule(rule_id).unwrap().conditions().body[0]
            .condition
            .op(),
        BodyOp::EqualInteger
    );
    port.apply(transaction(EditIntent::RemoveBodyCondition { id: body_id }))
        .unwrap();

    let replacement = RuleEditPayload {
        rule_match: map_rule_match("/updated", Some(UrlPathOp::StartsWith), "POST").unwrap(),
        headers: CollectionEdit::Replace(vec![ConditionEdit::Create {
            key: SemanticCreationKey::new("replacement-header").unwrap(),
            condition: map_header_condition("x-b", HeaderOp::Exists, "").unwrap(),
        }]),
        body: CollectionEdit::Clear,
        respond: map_response(ResponseMode::File, "", "responses/a.json", "", "").unwrap(),
    };
    port.apply(transaction(EditIntent::UpdateRule {
        id: rule_id,
        rule: replacement,
    }))
    .unwrap();
    assert_eq!(
        port.snapshot()
            .rule(rule_id)
            .unwrap()
            .rule_match()
            .url_path(),
        Some("/updated")
    );
    assert_eq!(
        port.snapshot()
            .rule(rule_id)
            .unwrap()
            .conditions()
            .headers
            .len(),
        1
    );

    let added_rule = port
        .apply(transaction(EditIntent::AddRule {
            parent,
            insertion_index: 1,
            rule: rule_payload("/delete", "DELETE"),
            key: SemanticCreationKey::new("rule-delete").unwrap(),
        }))
        .unwrap()
        .creations[0]
        .new_id;
    port.apply(transaction(EditIntent::DeleteRule { id: added_rule }))
        .unwrap();
    assert!(port.snapshot().rule(added_rule).is_none());

    let added_set = port
        .apply(transaction(EditIntent::AddRuleSet {
            path: parse_rule_set_path("temporary.toml").unwrap(),
            key: SemanticCreationKey::new("temporary-set").unwrap(),
        }))
        .unwrap()
        .creations[0]
        .new_id;
    port.apply(transaction(EditIntent::RemoveRuleSet {
        id: RuleSetId(added_set),
    }))
    .unwrap();
    assert!(
        port.snapshot()
            .workspace()
            .find_rule_set(RuleSetId(added_set))
            .is_none()
    );
}

#[test]
fn stable_ids_survive_update_move_snapshot_and_save() {
    let mut port = workspace();
    let parent = port.snapshot().workspace().rule_sets[0].id;
    let original = port.snapshot().rules()[0].rule_id();
    let added = port
        .apply(transaction(EditIntent::AddRule {
            parent,
            insertion_index: 1,
            rule: rule_payload("/second", "POST"),
            key: SemanticCreationKey::new("rule[1]").unwrap(),
        }))
        .unwrap()
        .creations[0]
        .new_id;
    port.apply(transaction(EditIntent::MoveRule {
        id: added,
        new_index: 0,
    }))
    .unwrap();
    port.apply(transaction(EditIntent::UpdateRespond {
        id: original,
        respond: map_response(ResponseMode::Inline, "changed", "", "", "").unwrap(),
    }))
    .unwrap();
    port.save().unwrap();
    let ids = port.snapshot().workspace().rule_sets[0]
        .rules
        .iter()
        .map(|rule| rule.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![added, original]);
}

#[test]
fn add_rule_inserts_at_zero_middle_and_len_with_duplicate_placement() {
    let mut port = workspace();
    let parent = port.snapshot().workspace().rule_sets[0].id;
    let original = port.snapshot().workspace().rule_sets[0].rules[0].id;

    let zero = port
        .apply(transaction(EditIntent::AddRule {
            parent,
            insertion_index: 0,
            rule: rule_payload("/zero", "GET"),
            key: SemanticCreationKey::new("zero").unwrap(),
        }))
        .unwrap();
    assert_eq!(zero.creations.len(), 1);
    assert!(zero.rebound_nodes.is_empty());
    assert_eq!(zero.changed_nodes.len(), 2);
    assert_eq!(zero.unsaved_hint, RuntimeEffect::Reload);
    let zero_id = zero.creations[0].new_id;

    let end_index = port.snapshot().workspace().rule_sets[0].rules.len();
    let end = port
        .apply(transaction(EditIntent::AddRule {
            parent,
            insertion_index: end_index,
            rule: rule_payload("/end", "GET"),
            key: SemanticCreationKey::new("end").unwrap(),
        }))
        .unwrap();
    assert_eq!(end.creations.len(), 1);
    assert!(end.rebound_nodes.is_empty());
    let end_id = end.creations[0].new_id;

    let original_index = port.snapshot().workspace().rule_sets[0]
        .rules
        .iter()
        .position(|rule| rule.id == original)
        .unwrap();
    let duplicate = port
        .apply(transaction(EditIntent::AddRule {
            parent,
            insertion_index: original_index + 1,
            rule: rule_payload("/health", "GET"),
            key: SemanticCreationKey::new("duplicate").unwrap(),
        }))
        .unwrap();
    assert_eq!(duplicate.creations.len(), 1);
    assert!(duplicate.rebound_nodes.is_empty());
    let duplicate_id = duplicate.creations[0].new_id;

    let ids = port.snapshot().workspace().rule_sets[0]
        .rules
        .iter()
        .map(|rule| rule.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec![zero_id, original, duplicate_id, end_id]);
    assert_eq!(port.snapshot().dirty_files().len(), 1);
    assert_eq!(port.snapshot().unsaved_hint(), RuntimeEffect::Reload);
    assert_eq!(port.snapshot().runtime_pending(), RuntimeEffect::None);
}

#[test]
fn add_rule_out_of_range_is_atomic() {
    let mut port = workspace();
    let before = port.snapshot();
    let parent = before.workspace().rule_sets[0].id;
    let before_ids = before.workspace().rule_sets[0]
        .rules
        .iter()
        .map(|rule| rule.id)
        .collect::<Vec<_>>();

    assert!(
        port.apply(transaction(EditIntent::AddRule {
            parent,
            insertion_index: before_ids.len() + 1,
            rule: rule_payload("/invalid", "GET"),
            key: SemanticCreationKey::new("invalid").unwrap(),
        }))
        .is_err()
    );

    let after = port.snapshot();
    assert_eq!(
        after.workspace().rule_sets[0]
            .rules
            .iter()
            .map(|rule| rule.id)
            .collect::<Vec<_>>(),
        before_ids
    );
    assert_eq!(after.dirty_files(), before.dirty_files());
    assert_eq!(after.unsaved_hint(), before.unsaved_hint());
    assert_eq!(after.runtime_pending(), before.runtime_pending());

    let valid = port
        .apply(transaction(EditIntent::AddRule {
            parent,
            insertion_index: 1,
            rule: rule_payload("/valid", "GET"),
            key: SemanticCreationKey::new("valid").unwrap(),
        }))
        .unwrap();
    assert_eq!(valid.creations.len(), 1);
    assert!(valid.rebound_nodes.is_empty());
}

#[test]
fn compensation_back_to_baseline_clears_dirty_state() {
    let mut port = workspace();
    let rule_id = port.snapshot().rules()[0].rule_id();
    let before = port.snapshot().rule(rule_id).unwrap().respond().clone();
    port.apply(transaction(EditIntent::UpdateRespond {
        id: rule_id,
        respond: map_response(ResponseMode::Inline, "changed", "", "200 OK", "0").unwrap(),
    }))
    .unwrap();
    assert_eq!(port.snapshot().unsaved_hint(), RuntimeEffect::Reload);
    assert!(port.snapshot().workspace().rule_sets[0].file.dirty);
    port.apply(transaction(EditIntent::UpdateRespond {
        id: rule_id,
        respond: before,
    }))
    .unwrap();
    assert!(port.snapshot().dirty_files().is_empty());
}

#[test]
fn case_distinct_rule_set_paths_are_accepted_and_initial_dirty_is_preserved() {
    let mut initial = minimal_workspace("test", "127.0.0.1", 3000, false);
    initial.rule_sets[0].file.dirty = true;
    let mut port = MemoryWorkspace::new(initial).unwrap();
    assert_eq!(
        port.snapshot().dirty_files()[0].path.as_str(),
        "rules/main.toml"
    );
    port.apply(transaction(EditIntent::AddRuleSet {
        path: parse_rule_set_path("rules/Main.toml").unwrap(),
        key: SemanticCreationKey::new("case-distinct").unwrap(),
    }))
    .unwrap();
    assert_eq!(port.snapshot().workspace().rule_sets.len(), 2);
    port.save().unwrap();
    assert!(port.snapshot().dirty_files().is_empty());
}

#[test]
fn deleting_a_saved_rule_set_remains_dirty_until_its_file_is_saved() {
    let mut port = workspace();
    let rule_set = port.snapshot().workspace().rule_sets[0].id;
    port.apply(transaction(EditIntent::RemoveRuleSet { id: rule_set }))
        .unwrap();
    assert_eq!(
        port.snapshot().dirty_files()[0].path.as_str(),
        "rules/main.toml"
    );
    let saved = port.save().unwrap();
    assert_eq!(saved.written_files[0].as_str(), "rules/main.toml");
    assert!(saved.snapshot.workspace().rule_sets.is_empty());
    assert!(saved.snapshot.dirty_files().is_empty());
}

#[test]
fn partial_save_is_path_ordered_and_retry_accumulates_effects() {
    let mut port = workspace();
    let transaction = EditTransaction::new(vec![
        EditIntent::AddRuleSet {
            path: parse_rule_set_path("00.toml").unwrap(),
            key: SemanticCreationKey::new("set[0]").unwrap(),
        },
        EditIntent::UpdateRootSetting(
            map_root_setting(
                WorkspaceRootKey::ListenerPort,
                WorkspaceEditValue::Integer(4000),
            )
            .unwrap(),
        ),
        EditIntent::AddRuleSet {
            path: parse_rule_set_path("zz.toml").unwrap(),
            key: SemanticCreationKey::new("set[1]").unwrap(),
        },
    ])
    .unwrap();
    let applied = port.apply(transaction).unwrap();
    assert_eq!(applied.unsaved_hint, applied.snapshot.unsaved_hint());
    let root = parse_workspace_relative_path("path", "apimock.toml").unwrap();
    port.inject_save_failure(root.clone()).unwrap();
    let failure = port.save().unwrap_err();
    assert_eq!(failure.unsaved_hint, failure.snapshot.unsaved_hint());
    assert_eq!(failure.runtime_pending, failure.snapshot.runtime_pending());
    assert_eq!(
        failure
            .written_files
            .iter()
            .map(WorkspaceRelativePath::as_str)
            .collect::<Vec<_>>(),
        vec!["00.toml"]
    );
    assert_eq!(failure.failed_file, root);
    assert_eq!(failure.runtime_pending, RuntimeEffect::Reload);
    assert_eq!(
        failure
            .snapshot
            .dirty_files()
            .iter()
            .map(|diff| diff.path.as_str())
            .collect::<Vec<_>>(),
        vec!["apimock.toml", "zz.toml"]
    );

    let retry = port.save().unwrap();
    assert_eq!(retry.unsaved_hint, retry.snapshot.unsaved_hint());
    assert_eq!(retry.runtime_pending, retry.snapshot.runtime_pending());
    assert_eq!(
        retry
            .written_files
            .iter()
            .map(WorkspaceRelativePath::as_str)
            .collect::<Vec<_>>(),
        vec!["apimock.toml", "zz.toml"]
    );
    assert_eq!(retry.runtime_pending, RuntimeEffect::Restart);
    assert!(retry.snapshot.dirty_files().is_empty());
}

#[test]
fn save_failure_at_first_and_last_file_preserves_correct_prefix() {
    for (failed, expected_prefix) in [("00.toml", vec![]), ("zz.toml", vec!["00.toml"])] {
        let mut port = workspace();
        port.apply(
            EditTransaction::new(vec![
                EditIntent::AddRuleSet {
                    path: parse_rule_set_path("00.toml").unwrap(),
                    key: SemanticCreationKey::new("set[0]").unwrap(),
                },
                EditIntent::AddRuleSet {
                    path: parse_rule_set_path("zz.toml").unwrap(),
                    key: SemanticCreationKey::new("set[1]").unwrap(),
                },
            ])
            .unwrap(),
        )
        .unwrap();
        port.inject_save_failure(parse_workspace_relative_path("path", failed).unwrap())
            .unwrap();
        let failure = port.save().unwrap_err();
        assert_eq!(
            failure
                .written_files
                .iter()
                .map(WorkspaceRelativePath::as_str)
                .collect::<Vec<_>>(),
            expected_prefix
        );
    }
}

#[test]
fn no_op_save_and_runtime_acknowledgement_obey_phase_rules() {
    let mut port = workspace();
    assert!(
        port.inject_save_failure(parse_workspace_relative_path("path", "apimock.toml").unwrap())
            .is_err()
    );
    let no_op = port.save().unwrap();
    assert!(no_op.written_files.is_empty());
    assert_eq!(no_op.runtime_pending, RuntimeEffect::None);

    port.apply(transaction(EditIntent::UpdateRootSetting(
        map_root_setting(
            WorkspaceRootKey::ListenerPort,
            WorkspaceEditValue::Integer(4000),
        )
        .unwrap(),
    )))
    .unwrap();
    port.save().unwrap();
    assert_eq!(port.snapshot().runtime_pending(), RuntimeEffect::Restart);
    assert_eq!(
        port.acknowledge_reload().runtime_pending(),
        RuntimeEffect::Restart
    );
    assert_eq!(
        port.acknowledge_restart().runtime_pending(),
        RuntimeEffect::None
    );

    port.apply(transaction(EditIntent::UpdateRootSetting(
        map_root_setting(
            WorkspaceRootKey::LogLevel,
            WorkspaceEditValue::Enum("debug".into()),
        )
        .unwrap(),
    )))
    .unwrap();
    port.save().unwrap();
    assert_eq!(port.snapshot().runtime_pending(), RuntimeEffect::Reload);
    assert_eq!(
        port.acknowledge_reload().runtime_pending(),
        RuntimeEffect::None
    );
}

#[test]
fn restore_returns_complete_bijective_rebinds() {
    let mut port = workspace();
    let parent = port.snapshot().workspace().rule_sets[0].id;
    let old_rule = NodeId::new();
    let old_header = NodeId::new();
    let archive = ArchivedSubtree::new(
        old_rule,
        RestorePlacement::Rule {
            parent,
            insertion_index: 1,
        },
        vec![
            ArchivedNode {
                old_id: old_rule,
                parent: None,
                key: SemanticCreationKey::new("rule").unwrap(),
                payload: ArchivedNodePayload::Rule(rule_payload("/restored", "GET")),
            },
            ArchivedNode {
                old_id: old_header,
                parent: Some(old_rule),
                key: SemanticCreationKey::new("rule/header[0]").unwrap(),
                payload: ArchivedNodePayload::HeaderCondition(
                    map_header_condition("x-a", HeaderOp::Equal, "v").unwrap(),
                ),
            },
        ],
    )
    .unwrap();
    let outcome = port
        .apply(transaction(EditIntent::RestoreSubtree { archive }))
        .unwrap();
    assert_eq!(outcome.rebound_nodes.len(), 2);
    assert_eq!(outcome.creations.len(), 2);
    let rebound_rule = outcome
        .rebound_nodes
        .iter()
        .find(|binding| binding.old_id == old_rule)
        .unwrap()
        .new_id;
    let rebound_header = outcome
        .rebound_nodes
        .iter()
        .find(|binding| binding.old_id == old_header)
        .unwrap()
        .new_id;
    assert_ne!(rebound_rule, old_rule);
    assert_ne!(rebound_header, old_header);
    assert_eq!(
        outcome
            .snapshot
            .rule(rebound_rule)
            .unwrap()
            .conditions()
            .headers[0]
            .id,
        rebound_header
    );
}

#[test]
fn rule_set_root_restore_inserts_at_zero_middle_and_end() {
    let mut port = workspace();
    let outcomes = [("zero.toml", 0), ("middle.toml", 1), ("end.toml", 3)]
        .into_iter()
        .map(|(path, insertion_index)| {
            port.apply(transaction(EditIntent::RestoreSubtree {
                archive: rule_set_archive(path, insertion_index),
            }))
            .unwrap()
        })
        .collect::<Vec<_>>();

    for outcome in &outcomes {
        assert_eq!(outcome.creations.len(), 1);
        assert_eq!(outcome.rebound_nodes.len(), 1);
        assert_eq!(outcome.changed_nodes.len(), 1);
        assert_eq!(outcome.unsaved_hint, RuntimeEffect::Reload);
    }
    let paths = port
        .snapshot()
        .workspace()
        .rule_sets
        .iter()
        .map(|rule_set| rule_set.file.path.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        vec!["zero.toml", "middle.toml", "rules/main.toml", "end.toml"]
    );
    assert_eq!(port.snapshot().dirty_files().len(), 3);
    assert_eq!(port.snapshot().runtime_pending(), RuntimeEffect::None);
}

#[test]
fn rule_set_root_restore_out_of_range_is_atomic() {
    let mut port = workspace();
    let before = port.snapshot();
    let before_paths = before
        .workspace()
        .rule_sets
        .iter()
        .map(|rule_set| rule_set.file.path.clone())
        .collect::<Vec<_>>();

    assert!(
        port.apply(transaction(EditIntent::RestoreSubtree {
            archive: rule_set_archive("invalid.toml", before_paths.len() + 1),
        }))
        .is_err()
    );

    let after = port.snapshot();
    assert_eq!(
        after
            .workspace()
            .rule_sets
            .iter()
            .map(|rule_set| rule_set.file.path.clone())
            .collect::<Vec<_>>(),
        before_paths
    );
    assert_eq!(after.dirty_files(), before.dirty_files());
    assert_eq!(after.unsaved_hint(), before.unsaved_hint());
    assert_eq!(after.runtime_pending(), before.runtime_pending());
}

#[test]
fn restore_rejects_live_rule_set_rule_and_condition_old_ids_atomically() {
    let mut port = workspace();
    let initial = port.snapshot();
    let live_set = initial.workspace().rule_sets[0].id;
    let live_rule = initial.rules()[0].rule_id();

    let set_archive = ArchivedSubtree::new(
        live_set.0,
        RestorePlacement::RuleSetRoot { insertion_index: 0 },
        vec![ArchivedNode {
            old_id: live_set.0,
            parent: None,
            key: SemanticCreationKey::new("set").unwrap(),
            payload: ArchivedNodePayload::RuleSet {
                path: parse_rule_set_path("collision-set.toml").unwrap(),
            },
        }],
    )
    .unwrap();
    assert!(
        port.apply(transaction(EditIntent::RestoreSubtree {
            archive: set_archive,
        }))
        .is_err()
    );

    let rule_archive = ArchivedSubtree::new(
        live_rule,
        RestorePlacement::Rule {
            parent: live_set,
            insertion_index: 1,
        },
        vec![ArchivedNode {
            old_id: live_rule,
            parent: None,
            key: SemanticCreationKey::new("rule").unwrap(),
            payload: ArchivedNodePayload::Rule(rule_payload("/collision", "GET")),
        }],
    )
    .unwrap();
    assert!(
        port.apply(transaction(EditIntent::RestoreSubtree {
            archive: rule_archive,
        }))
        .is_err()
    );

    let live_header = port
        .apply(transaction(EditIntent::AddHeaderCondition {
            rule_id: live_rule,
            condition: map_header_condition("x-live", HeaderOp::Equal, "v").unwrap(),
            key: SemanticCreationKey::new("live-header").unwrap(),
        }))
        .unwrap()
        .creations[0]
        .new_id;
    let before = port.snapshot();
    let old_rule = NodeId::new();
    let condition_archive = ArchivedSubtree::new(
        old_rule,
        RestorePlacement::Rule {
            parent: live_set,
            insertion_index: 1,
        },
        vec![
            ArchivedNode {
                old_id: old_rule,
                parent: None,
                key: SemanticCreationKey::new("restored-rule").unwrap(),
                payload: ArchivedNodePayload::Rule(rule_payload("/restored", "GET")),
            },
            ArchivedNode {
                old_id: live_header,
                parent: Some(old_rule),
                key: SemanticCreationKey::new("restored-rule/header").unwrap(),
                payload: ArchivedNodePayload::HeaderCondition(
                    map_header_condition("x-a", HeaderOp::Equal, "v").unwrap(),
                ),
            },
        ],
    )
    .unwrap();
    assert!(
        port.apply(transaction(EditIntent::RestoreSubtree {
            archive: condition_archive,
        }))
        .is_err()
    );
    let after = port.snapshot();
    assert_eq!(after.rules().len(), before.rules().len());
    assert_eq!(
        after.rule(live_rule).unwrap().conditions().headers.len(),
        before.rule(live_rule).unwrap().conditions().headers.len()
    );
    assert_eq!(after.dirty_files(), before.dirty_files());
}

#[test]
fn nonempty_rule_set_restore_rebinds_rules_and_equal_conditions() {
    let mut port = workspace();
    let old_set = NodeId::new();
    let old_rule = NodeId::new();
    let old_first = NodeId::new();
    let old_second = NodeId::new();
    let equal = map_body_condition("a", BodyOp::Equal, "same").unwrap();
    let archive = ArchivedSubtree::new(
        old_set,
        RestorePlacement::RuleSetRoot { insertion_index: 1 },
        vec![
            ArchivedNode {
                old_id: old_set,
                parent: None,
                key: SemanticCreationKey::new("set").unwrap(),
                payload: ArchivedNodePayload::RuleSet {
                    path: parse_rule_set_path("restored.toml").unwrap(),
                },
            },
            ArchivedNode {
                old_id: old_rule,
                parent: Some(old_set),
                key: SemanticCreationKey::new("set/rule").unwrap(),
                payload: ArchivedNodePayload::Rule(rule_payload("/restored", "POST")),
            },
            ArchivedNode {
                old_id: old_first,
                parent: Some(old_rule),
                key: SemanticCreationKey::new("set/rule/body[0]").unwrap(),
                payload: ArchivedNodePayload::BodyCondition(equal.clone()),
            },
            ArchivedNode {
                old_id: old_second,
                parent: Some(old_rule),
                key: SemanticCreationKey::new("set/rule/body[1]").unwrap(),
                payload: ArchivedNodePayload::BodyCondition(equal),
            },
        ],
    )
    .unwrap();
    let outcome = port
        .apply(transaction(EditIntent::RestoreSubtree { archive }))
        .unwrap();
    assert_eq!(outcome.rebound_nodes.len(), 4);
    let new_rule = outcome
        .rebound_nodes
        .iter()
        .find(|binding| binding.old_id == old_rule)
        .unwrap()
        .new_id;
    let body = &outcome.snapshot.rule(new_rule).unwrap().conditions().body;
    assert_eq!(body.len(), 2);
    assert_ne!(body[0].id, body[1].id);
    assert_eq!(body[0].condition, body[1].condition);
}

#[test]
fn validation_matches_snapshot_diagnostics() {
    let mut initial = minimal_workspace("test", "127.0.0.1", 3000, false);
    initial.diagnostics.push(Diagnostic {
        node_id: None,
        severity: Severity::Warning,
        message: "test diagnostic".into(),
    });
    let port = MemoryWorkspace::new(initial).unwrap();
    assert_eq!(
        port.validate().issues.len(),
        port.snapshot().workspace().diagnostics.len()
    );
    assert!(
        port.snapshot()
            .rule(port.snapshot().rules()[0].rule_id())
            .is_some()
    );
}

#[test]
fn successful_update_and_delete_refresh_diagnostics_and_inline_validation() {
    fn invalid_initial() -> (crate::WorkspaceSnapshot, NodeId) {
        let mut initial = minimal_workspace("test", "127.0.0.1", 3000, false);
        let rule_id = initial.rule_sets[0].rules[0].id;
        let issue = ValidationIssue {
            node_id: Some(rule_id),
            severity: Severity::Warning,
            message: "stale inline issue".into(),
            location: Some("rules[0]".into()),
        };
        initial.rule_sets[0].validation = NodeValidation {
            issues: vec![issue.clone()],
        };
        initial.rule_sets[0].rules[0].validation = NodeValidation {
            issues: vec![issue],
        };
        initial.diagnostics.push(Diagnostic {
            node_id: Some(rule_id),
            severity: Severity::Warning,
            message: "stale workspace diagnostic".into(),
        });
        (initial, rule_id)
    }

    let (initial, rule_id) = invalid_initial();
    let mut port = MemoryWorkspace::new(initial).unwrap();
    port.apply(transaction(EditIntent::UpdateRespond {
        id: rule_id,
        respond: map_response(ResponseMode::Inline, "updated", "", "", "").unwrap(),
    }))
    .unwrap();
    let updated = port.snapshot();
    assert!(updated.workspace().diagnostics.is_empty());
    assert!(
        updated.workspace().rule_sets[0]
            .validation
            .issues
            .is_empty()
    );
    assert!(
        updated
            .workspace()
            .find_rule(rule_id)
            .unwrap()
            .1
            .validation
            .issues
            .is_empty()
    );
    assert!(port.validate().issues.is_empty());

    let (initial, rule_id) = invalid_initial();
    let mut port = MemoryWorkspace::new(initial).unwrap();
    port.apply(transaction(EditIntent::DeleteRule { id: rule_id }))
        .unwrap();
    assert!(port.snapshot().workspace().diagnostics.is_empty());
    assert!(port.snapshot().workspace().find_rule(rule_id).is_none());
}
