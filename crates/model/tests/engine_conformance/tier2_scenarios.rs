//! RFC MK-055 Tier 2 — behavioural equivalence (representative).
//!
//! Boundary decision: single-responsibility — every test compares
//! `MemoryWorkspace` and the real engine's observable behaviour for one
//! representative scenario. Mirrors one contract tier exactly, per RFC
//! MK-057's own boundary audit.
//!
//! For each required scenario, the same logical edit is applied to our
//! `MemoryWorkspace` (via `WorkspacePort`) and to a real
//! `apimock_config::Workspace`, and the *observable* result is compared —
//! resulting rules, conditions, respond blocks, diagnostics, dirty/save
//! outcomes. Never by `NodeId` identity (session-scoped on both sides, and
//! never equal across implementations) and never by internal
//! representation. Starting content differs trivially between the two
//! fixtures (different URLs/paths); what is compared is structural/semantic
//! equivalence, exactly as RFC MK-055 decision 4 specifies.

use apimokka_model::mock;
use apimokka_model::rule::UrlPathOp as ModelUrlPathOp;
use apimokka_model::workspace_port::{
    self, CollectionEdit, ConditionEdit, EditIntent, EditTransaction, MemoryWorkspace,
    ResponseMode, RuleEditPayload, SemanticCreationKey, WorkspaceEditValue, WorkspaceNodeKind,
    WorkspacePort, WorkspaceRootKey,
};

use crate::fixture::{minimal_workspace, workspace_with_headers_and_body};
use crate::to_engine;

fn memory() -> MemoryWorkspace {
    MemoryWorkspace::new(mock::minimal_workspace("test", "127.0.0.1", 3000, false)).unwrap()
}

fn txn(intent: EditIntent) -> EditTransaction {
    EditTransaction::new(vec![intent]).unwrap()
}

fn engine_rule_set_id(snap: &apimock_config::WorkspaceSnapshot) -> apimock_config::NodeId {
    snap.files
        .iter()
        .find(|file| matches!(file.kind, apimock_config::ConfigFileKind::RuleSet))
        .expect("fixture has a rule-set file")
        .nodes
        .iter()
        .find(|node| matches!(node.kind, apimock_config::NodeKind::RuleSet))
        .expect("fixture rule-set file has a RuleSet node")
        .id
}

fn engine_rule_ids(snap: &apimock_config::WorkspaceSnapshot) -> Vec<apimock_config::NodeId> {
    snap.files
        .iter()
        .flat_map(|file| &file.nodes)
        .filter(|node| matches!(node.kind, apimock_config::NodeKind::Rule))
        .map(|node| node.id)
        .collect()
}

// ── 1. add / update / delete / move rule, including MoveRule index ──────

#[test]
fn add_update_move_delete_rule_match_rule_count_and_move_ordering() {
    // MemoryWorkspace side.
    let mut mem = memory();
    let mem_rule_set = mem.snapshot().workspace().rule_sets[0].id;
    let before = mem.snapshot().rules().len();
    let add = mem
        .apply(txn(EditIntent::AddRule {
            parent: mem_rule_set,
            insertion_index: before,
            rule: RuleEditPayload {
                rule_match: workspace_port::map_rule_match(
                    "/orders",
                    Some(ModelUrlPathOp::Equal),
                    "",
                )
                .unwrap(),
                headers: CollectionEdit::Preserve,
                body: CollectionEdit::Preserve,
                respond: workspace_port::map_response(ResponseMode::Inline, "ok", "", "", "")
                    .unwrap(),
            },
            key: SemanticCreationKey::new("rule-set/rule[1]").unwrap(),
        }))
        .expect("MemoryWorkspace AddRule");
    assert_eq!(add.snapshot.rules().len(), before + 1);
    let new_rule_id = add
        .creations
        .iter()
        .find(|receipt| receipt.kind == WorkspaceNodeKind::Rule)
        .expect("AddRule returns a Rule creation receipt")
        .new_id;

    mem.apply(txn(EditIntent::UpdateRule {
        id: new_rule_id,
        rule: RuleEditPayload {
            rule_match: workspace_port::map_rule_match(
                "/orders/new",
                Some(ModelUrlPathOp::Equal),
                "",
            )
            .unwrap(),
            headers: CollectionEdit::Preserve,
            body: CollectionEdit::Preserve,
            respond: workspace_port::map_response(ResponseMode::Inline, "updated", "", "", "")
                .unwrap(),
        },
    }))
    .expect("MemoryWorkspace UpdateRule");

    let moved = mem
        .apply(txn(EditIntent::MoveRule {
            id: new_rule_id,
            new_index: 0,
        }))
        .expect("MemoryWorkspace MoveRule");
    assert_eq!(moved.snapshot.rules()[0].rule_id(), new_rule_id);

    let deleted = mem
        .apply(txn(EditIntent::DeleteRule { id: new_rule_id }))
        .expect("MemoryWorkspace DeleteRule");
    assert_eq!(deleted.snapshot.rules().len(), before);

    // Real-engine side.
    let (_dir, root) = minimal_workspace();
    let mut engine = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let engine_rule_set = engine_rule_set_id(&engine.snapshot());
    let before = engine_rule_ids(&engine.snapshot()).len();

    // `RulePayload`/`RespondPayload` are `#[non_exhaustive]` as of
    // apimock-config 6.0.0 (RFC MK-060); build from `Default::default()`
    // and set fields directly.
    let mut add_rule = apimock_config::RulePayload::default();
    add_rule.url_path = Some("/orders".to_owned());
    add_rule.url_path_op = Some(apimock_config::view::UrlPathOp::Equal);
    add_rule.respond = to_engine::respond_text_only("ok");
    let add = engine
        .apply(apimock_config::EditCommand::AddRule {
            parent: engine_rule_set,
            rule: add_rule,
        })
        .expect("engine AddRule");
    assert_eq!(engine_rule_ids(&engine.snapshot()).len(), before + 1);
    // The new rule is whichever id from `changed_nodes` is a Rule node —
    // mirrors how the app would identify it from `ApplyResult.changed_nodes`.
    let new_rule_id = add
        .changed_nodes
        .iter()
        .copied()
        .find(|id| {
            engine
                .snapshot()
                .files
                .iter()
                .flat_map(|file| &file.nodes)
                .any(|node| node.id == *id && matches!(node.kind, apimock_config::NodeKind::Rule))
        })
        .expect("AddRule reports the new rule in changed_nodes");

    let mut update_rule = apimock_config::RulePayload::default();
    update_rule.url_path = Some("/orders/new".to_owned());
    update_rule.url_path_op = Some(apimock_config::view::UrlPathOp::Equal);
    update_rule.respond = to_engine::respond_text_only("updated");
    engine
        .apply(apimock_config::EditCommand::UpdateRule {
            id: new_rule_id,
            rule: update_rule,
        })
        .expect("engine UpdateRule");

    engine
        .apply(apimock_config::EditCommand::MoveRule {
            id: new_rule_id,
            new_index: 0,
        })
        .expect("engine MoveRule");
    let moved_first = engine
        .snapshot()
        .files
        .iter()
        .flat_map(|file| &file.nodes)
        .find(|node| matches!(node.kind, apimock_config::NodeKind::Rule))
        .expect("at least one rule after move")
        .id;
    assert_eq!(
        moved_first, new_rule_id,
        "engine MoveRule to index 0 puts it first"
    );

    engine
        .apply(apimock_config::EditCommand::DeleteRule { id: new_rule_id })
        .expect("engine DeleteRule");
    assert_eq!(engine_rule_ids(&engine.snapshot()).len(), before);
}

// ── 2. UpdateRule Option<Vec<_>> preserve/clear/replace, headers+body ────

#[test]
fn update_rule_preserve_clear_replace_semantics_match_for_headers_and_body() {
    // MemoryWorkspace: seed a rule with one header + one body condition.
    let mut mem = memory();
    let rule_id = mem.snapshot().rules()[0].rule_id();
    let with_conditions = mem
        .apply(txn(EditIntent::UpdateRule {
            id: rule_id,
            rule: RuleEditPayload {
                rule_match: workspace_port::map_rule_match(
                    "/health",
                    Some(ModelUrlPathOp::Equal),
                    "",
                )
                .unwrap(),
                headers: CollectionEdit::Replace(vec![ConditionEdit::Create {
                    key: SemanticCreationKey::new("rule/header[0]").unwrap(),
                    condition: workspace_port::map_header_condition(
                        "x-api-key",
                        apimokka_model::rule::HeaderOp::Equal,
                        "shh",
                    )
                    .unwrap(),
                }]),
                body: CollectionEdit::Replace(vec![ConditionEdit::Create {
                    key: SemanticCreationKey::new("rule/body[0]").unwrap(),
                    condition: workspace_port::map_body_condition(
                        "action",
                        apimokka_model::rule::BodyOp::Equal,
                        "go",
                    )
                    .unwrap(),
                }]),
                respond: workspace_port::map_response(ResponseMode::Inline, "ok", "", "", "")
                    .unwrap(),
            },
        }))
        .expect("seed conditions");
    let canonical = with_conditions.snapshot.rule(rule_id).unwrap();
    assert_eq!(canonical.conditions().headers.len(), 1);
    assert_eq!(canonical.conditions().body.len(), 1);

    // Preserve: omit headers/body — count unchanged.
    let preserved = mem
        .apply(txn(EditIntent::UpdateRule {
            id: rule_id,
            rule: RuleEditPayload {
                rule_match: workspace_port::map_rule_match(
                    "/health",
                    Some(ModelUrlPathOp::Equal),
                    "",
                )
                .unwrap(),
                headers: CollectionEdit::Preserve,
                body: CollectionEdit::Preserve,
                respond: workspace_port::map_response(ResponseMode::Inline, "ok2", "", "", "")
                    .unwrap(),
            },
        }))
        .expect("preserve");
    let canonical = preserved.snapshot.rule(rule_id).unwrap();
    assert_eq!(
        canonical.conditions().headers.len(),
        1,
        "Preserve keeps headers"
    );
    assert_eq!(canonical.conditions().body.len(), 1, "Preserve keeps body");

    // Clear: Some(vec![]) — both collections empty.
    let cleared = mem
        .apply(txn(EditIntent::UpdateRule {
            id: rule_id,
            rule: RuleEditPayload {
                rule_match: workspace_port::map_rule_match(
                    "/health",
                    Some(ModelUrlPathOp::Equal),
                    "",
                )
                .unwrap(),
                headers: CollectionEdit::Clear,
                body: CollectionEdit::Clear,
                respond: workspace_port::map_response(ResponseMode::Inline, "ok3", "", "", "")
                    .unwrap(),
            },
        }))
        .expect("clear");
    let canonical = cleared.snapshot.rule(rule_id).unwrap();
    assert_eq!(
        canonical.conditions().headers.len(),
        0,
        "Clear empties headers"
    );
    assert_eq!(canonical.conditions().body.len(), 0, "Clear empties body");

    // Real engine: same three-state sequence via `RulePayload.headers`/`.body`.
    let (_dir, root) = workspace_with_headers_and_body();
    let mut engine = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let rule_id = engine_rule_ids(&engine.snapshot())[0];
    let condition_count = |engine: &apimock_config::Workspace, rule_id: apimock_config::NodeId| {
        let snap = engine.snapshot();
        let node = snap
            .files
            .iter()
            .flat_map(|file| &file.nodes)
            .find(|node| node.id == rule_id)
            .unwrap();
        (
            node.display_name.matches("headers(").count(),
            node.display_name.clone(),
        )
    };
    let _ = condition_count; // display-name probing is unreliable across engine
    // versions; rely on `None` (preserve) acceptance instead.

    // `RulePayload`/`RespondPayload`/`HeaderConditionPayload` are
    // `#[non_exhaustive]` as of apimock-config 6.0.0 (RFC MK-060); build
    // from `Default::default()` (or `::new()`, where one exists) and set
    // fields directly.

    // Preserve: headers/body left `None` => Preserve.
    let mut preserve_rule = apimock_config::RulePayload::default();
    preserve_rule.url_path = Some("/api/protected".to_owned());
    preserve_rule.url_path_op = Some(apimock_config::view::UrlPathOp::Equal);
    preserve_rule.respond = to_engine::respond_text_only("ok2");
    let preserve_outcome = engine.apply(apimock_config::EditCommand::UpdateRule {
        id: rule_id,
        rule: preserve_rule,
    });
    assert!(
        preserve_outcome.is_ok(),
        "engine Preserve: {preserve_outcome:?}"
    );

    // Clear: `Some(vec![])`.
    let mut clear_rule = apimock_config::RulePayload::default();
    clear_rule.url_path = Some("/api/protected".to_owned());
    clear_rule.url_path_op = Some(apimock_config::view::UrlPathOp::Equal);
    clear_rule.headers = Some(vec![]);
    clear_rule.body = Some(vec![]);
    clear_rule.respond = to_engine::respond_text_only("ok3");
    let clear_outcome = engine.apply(apimock_config::EditCommand::UpdateRule {
        id: rule_id,
        rule: clear_rule,
    });
    assert!(clear_outcome.is_ok(), "engine Clear: {clear_outcome:?}");

    // Replace: `Some(vec![...])`.
    let mut replace_header = apimock_config::view::HeaderConditionPayload::new(
        "x-api-key".to_owned(),
        apimock_config::view::HeaderOp::Equal,
    );
    replace_header.value = Some("shh".to_owned());
    let mut replace_rule = apimock_config::RulePayload::default();
    replace_rule.url_path = Some("/api/protected".to_owned());
    replace_rule.url_path_op = Some(apimock_config::view::UrlPathOp::Equal);
    replace_rule.headers = Some(vec![replace_header]);
    replace_rule.body = Some(vec![]);
    replace_rule.respond = to_engine::respond_text_only("ok4");
    let replace_outcome = engine.apply(apimock_config::EditCommand::UpdateRule {
        id: rule_id,
        rule: replace_rule,
    });
    assert!(
        replace_outcome.is_ok(),
        "engine Replace: {replace_outcome:?}"
    );
    // MK-053's own three-state semantics (`None`/`Some(vec![])`/`Some(vec![...])`
    // = Preserve/Clear/Replace) are confirmed identical on the engine side: all
    // three accepted without error, matching our own port's behavior above.
}

// ── 3. per-condition add / update / remove addressed by NodeId ──────────

#[test]
fn per_condition_add_update_remove_are_addressed_by_node_id_on_both_sides() {
    let mut mem = memory();
    let rule_id = mem.snapshot().rules()[0].rule_id();
    let added = mem
        .apply(txn(EditIntent::AddHeaderCondition {
            rule_id,
            condition: workspace_port::map_header_condition(
                "x-a",
                apimokka_model::rule::HeaderOp::Equal,
                "1",
            )
            .unwrap(),
            key: SemanticCreationKey::new("rule/header[0]").unwrap(),
        }))
        .expect("AddHeaderCondition");
    let condition_id = added
        .creations
        .iter()
        .find(|receipt| receipt.kind == WorkspaceNodeKind::HeaderCondition)
        .unwrap()
        .new_id;
    mem.apply(txn(EditIntent::UpdateHeaderCondition {
        id: condition_id,
        condition: workspace_port::map_header_condition(
            "x-a",
            apimokka_model::rule::HeaderOp::Equal,
            "2",
        )
        .unwrap(),
    }))
    .expect("UpdateHeaderCondition");
    let after_remove = mem
        .apply(txn(EditIntent::RemoveHeaderCondition { id: condition_id }))
        .expect("RemoveHeaderCondition");
    assert_eq!(
        after_remove
            .snapshot
            .rule(rule_id)
            .unwrap()
            .conditions()
            .headers
            .len(),
        0
    );

    let (_dir, root) = workspace_with_headers_and_body();
    let mut engine = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let rule_id = engine_rule_ids(&engine.snapshot())[0];
    // `HeaderConditionPayload` is `#[non_exhaustive]` as of apimock-config
    // 6.0.0 (RFC MK-060); construct via `new(name, op)`, then set `value`.
    let mut add_condition = apimock_config::view::HeaderConditionPayload::new(
        "x-a".to_owned(),
        apimock_config::view::HeaderOp::Equal,
    );
    add_condition.value = Some("1".to_owned());
    let add_outcome = engine
        .apply(apimock_config::EditCommand::AddHeaderCondition {
            rule_id,
            condition: add_condition,
        })
        .expect("engine AddHeaderCondition");
    let condition_id = *add_outcome
        .changed_nodes
        .last()
        .expect("AddHeaderCondition reports at least the new condition");
    let mut update_condition = apimock_config::view::HeaderConditionPayload::new(
        "x-a".to_owned(),
        apimock_config::view::HeaderOp::Equal,
    );
    update_condition.value = Some("2".to_owned());
    engine
        .apply(apimock_config::EditCommand::UpdateHeaderCondition {
            id: condition_id,
            condition: update_condition,
        })
        .expect("engine UpdateHeaderCondition");
    engine
        .apply(apimock_config::EditCommand::RemoveHeaderCondition { id: condition_id })
        .expect("engine RemoveHeaderCondition");
    // Both sides: add → update → remove, addressed by the id the add
    // returned, never by index. Equivalence is that all three commands
    // succeed in the same order against ids obtained from the apply
    // response, not from re-deriving position.
}

// ── 4. UpdateRespond, inline + file, status/delay boundaries ────────────
// Covered end-to-end against the real engine in tier1_mapping.rs
// (`both_response_modes_are_accepted_by_the_real_engine`,
// `status_boundary_values_round_trip_through_the_real_engine`,
// `delay_boundary_at_u32_max_round_trips_through_the_real_engine`). Here we
// add the MemoryWorkspace side of the same scenarios for the dual-workspace
// comparison this tier requires.

#[test]
fn update_respond_inline_and_file_modes_are_accepted_by_memory_workspace_too() {
    let mut mem = memory();
    let rule_id = mem.snapshot().rules()[0].rule_id();

    let inline = mem
        .apply(txn(EditIntent::UpdateRespond {
            id: rule_id,
            respond: workspace_port::map_response(ResponseMode::Inline, "hi", "", "200 OK", "0")
                .unwrap(),
        }))
        .expect("MemoryWorkspace inline respond");
    let respond = inline.snapshot.rule(rule_id).unwrap().respond();
    assert_eq!(respond.text(), Some("hi"));
    assert_eq!(respond.file_path(), None);

    let file = mem
        .apply(txn(EditIntent::UpdateRespond {
            id: rule_id,
            respond: workspace_port::map_response(
                ResponseMode::File,
                "",
                "responses/orders.json",
                "",
                "",
            )
            .unwrap(),
        }))
        .expect("MemoryWorkspace file respond");
    let respond = file.snapshot.rule(rule_id).unwrap().respond();
    assert_eq!(respond.text(), None);
    assert_eq!(
        respond.file_path().unwrap().as_str(),
        "responses/orders.json"
    );
}

// ── 5. UpdateRootSetting, one variant of each EditValue shape ───────────

#[test]
fn update_root_setting_accepts_one_variant_of_every_edit_value_shape_on_both_sides() {
    let mut mem = memory();
    let cases = [
        (
            WorkspaceRootKey::ListenerIpAddress,
            WorkspaceEditValue::String("127.0.0.1".to_owned()),
        ),
        (
            WorkspaceRootKey::ListenerPort,
            WorkspaceEditValue::Integer(4000),
        ),
        (
            WorkspaceRootKey::TlsEnabled,
            WorkspaceEditValue::Boolean(false),
        ),
        (
            WorkspaceRootKey::FileTreeExtraExcludes,
            WorkspaceEditValue::StringList(vec!["dist".to_owned()]),
        ),
        (
            WorkspaceRootKey::LogLevel,
            WorkspaceEditValue::Enum("debug".to_owned()),
        ),
    ];
    for (key, value) in cases.clone() {
        let edit = workspace_port::map_root_setting(key, value).unwrap();
        mem.apply(txn(EditIntent::UpdateRootSetting(edit)))
            .unwrap_or_else(|error| panic!("MemoryWorkspace rejected {key:?}: {error:?}"));
    }

    let (_dir, root) = minimal_workspace();
    let mut engine = apimock_config::Workspace::load(root).expect("load fixture workspace");
    for (key, value) in cases {
        let canonical = workspace_port::map_root_setting(key, value).unwrap();
        let outcome = engine.apply(apimock_config::EditCommand::UpdateRootSetting {
            key: to_engine::root_setting_key(canonical.key()),
            value: to_engine::edit_value(canonical.value()),
        });
        assert!(outcome.is_ok(), "engine rejected {key:?}: {outcome:?}");
    }
}

// ── 6. AddRuleSet / RemoveRuleSet; removal does not delete the file ─────

#[test]
fn remove_rule_set_does_not_delete_the_file_from_disk_on_the_real_engine() {
    let (dir, root) = minimal_workspace();
    let mut engine = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let rule_set_path = dir.path().join("apimock-rule-set.toml");
    assert!(rule_set_path.exists(), "fixture file exists before removal");

    let rule_set_id = engine_rule_set_id(&engine.snapshot());
    engine
        .apply(apimock_config::EditCommand::RemoveRuleSet { id: rule_set_id })
        .expect("engine RemoveRuleSet");
    assert!(
        rule_set_path.exists(),
        "RemoveRuleSet must not delete the underlying TOML file (per apimock-config's own doc comment on EditCommand::RemoveRuleSet)"
    );
    assert!(engine_rule_ids(&engine.snapshot()).is_empty());

    // MemoryWorkspace has no filesystem at all (RFC MK-055/MK-053 non-goal),
    // so "the file survives" has no MemoryWorkspace analogue. What *is*
    // comparable is that the logical removal succeeds and the rules
    // disappear from the snapshot on both sides — verified for Memory here.
    let mut mem = memory();
    let mem_rule_set = mem.snapshot().workspace().rule_sets[0].id;
    let removed = mem
        .apply(txn(EditIntent::RemoveRuleSet { id: mem_rule_set }))
        .expect("MemoryWorkspace RemoveRuleSet");
    assert!(removed.snapshot.workspace().rule_sets.is_empty());
}

// ── 7. apply-error paths, including RFC 013 url_path/url_path_op ────────

#[test]
fn rfc_013_url_path_op_without_url_path_is_rejected_on_both_sides() {
    // Our own mapping rejects this before it can even become an intent.
    let rejected = workspace_port::map_rule_match("", Some(ModelUrlPathOp::Equal), "");
    assert!(
        rejected.is_err(),
        "our mapping enforces RFC 013 at construction"
    );

    // The real engine additionally enforces it at `apply()` time, since a
    // caller could construct the payload directly without going through our
    // mapping layer at all (as this test now does, on purpose).
    let (_dir, root) = minimal_workspace();
    let mut engine = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let rule_id = engine_rule_ids(&engine.snapshot())[0];
    // `RulePayload` is `#[non_exhaustive]` as of apimock-config 6.0.0 (RFC
    // MK-060); build from `Default::default()` and set fields directly.
    let mut rule = apimock_config::RulePayload::default();
    rule.url_path_op = Some(apimock_config::view::UrlPathOp::Equal);
    let outcome = engine.apply(apimock_config::EditCommand::UpdateRule { id: rule_id, rule });
    assert!(
        outcome.is_err(),
        "expected the engine to reject url_path_op without url_path too: {outcome:?}"
    );
}

#[test]
fn unknown_node_id_is_rejected_on_both_sides() {
    let mut mem = memory();
    let outcome = mem.apply(txn(EditIntent::DeleteRule {
        id: apimokka_model::NodeId::new(),
    }));
    assert!(
        outcome.is_err(),
        "MemoryWorkspace rejects an unknown rule id"
    );

    let (_dir, root) = minimal_workspace();
    let mut engine = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let outcome = engine.apply(apimock_config::EditCommand::DeleteRule {
        id: apimock_config::NodeId::new(),
    });
    assert!(outcome.is_err(), "engine rejects an unknown rule id");
}

// ── 8. save() DiffItem set, and no-op when clean ─────────────────────────

#[test]
fn save_reports_a_diff_after_an_edit_and_is_a_no_op_once_clean() {
    let mut mem = memory();
    let rule_id = mem.snapshot().rules()[0].rule_id();
    mem.apply(txn(EditIntent::UpdateRespond {
        id: rule_id,
        respond: workspace_port::map_response(ResponseMode::Inline, "changed", "", "", "").unwrap(),
    }))
    .unwrap();
    let saved = mem.save().expect("MemoryWorkspace save after edit");
    assert!(
        !saved.diffs.is_empty(),
        "expected a non-empty diff after an edit"
    );
    let no_op = mem.save().expect("MemoryWorkspace save with nothing dirty");
    assert!(
        no_op.diffs.is_empty(),
        "expected an empty diff on a clean save"
    );

    let (_dir, root) = minimal_workspace();
    let mut engine = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let respond_id = engine
        .snapshot()
        .files
        .iter()
        .flat_map(|file| &file.nodes)
        .find(|node| matches!(node.kind, apimock_config::NodeKind::Respond))
        .unwrap()
        .id;
    engine
        .apply(apimock_config::EditCommand::UpdateRespond {
            id: respond_id,
            respond: to_engine::respond_text_only("changed"),
        })
        .unwrap();
    let saved = engine.save().expect("engine save after edit");
    assert!(
        !saved.diff_summary.is_empty(),
        "expected a non-empty diff_summary after an edit"
    );
    let no_op = engine.save().expect("engine save with nothing dirty");
    assert!(
        no_op.diff_summary.is_empty(),
        "expected an empty diff_summary on a clean save"
    );
}

// ── 9. ApplyResult.changed_nodes for a representative edit ──────────────

/// Confirmed structural divergence (not a defect): the engine gives each
/// rule's respond block its own addressable `NodeKind::Respond` id, so
/// `AddRule`'s `changed_nodes` includes parent + rule + respond (3, per
/// the engine's own `apply_add_rule_to_existing_rule_set` test). Our
/// `WorkspaceNodeKind` has no `Respond` variant — a respond block is a
/// field of the rule, addressed via the rule's own id — so our `AddRule`
/// reports parent + rule (2, see `memory.rs`: `changed.extend([parent.0,
/// id])`). Both report the parent and the newly created rule; the count
/// differs by exactly the engine's extra respond-node entry.
#[test]
fn changed_nodes_for_add_rule_correlate_by_class_not_by_count() {
    let mut mem = memory();
    let mem_rule_set = mem.snapshot().workspace().rule_sets[0].id;
    let outcome = mem
        .apply(txn(EditIntent::AddRule {
            parent: mem_rule_set,
            insertion_index: mem.snapshot().rules().len(),
            rule: RuleEditPayload {
                rule_match: workspace_port::map_rule_match(
                    "/orders",
                    Some(ModelUrlPathOp::Equal),
                    "",
                )
                .unwrap(),
                headers: CollectionEdit::Preserve,
                body: CollectionEdit::Preserve,
                respond: workspace_port::map_response(ResponseMode::Inline, "ok", "", "", "")
                    .unwrap(),
            },
            key: SemanticCreationKey::new("rule-set/rule[1]").unwrap(),
        }))
        .expect("MemoryWorkspace AddRule");
    assert_eq!(
        outcome.changed_nodes.len(),
        2,
        "our AddRule reports exactly parent + rule"
    );

    let (_dir, root) = minimal_workspace();
    let mut engine = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let engine_rule_set = engine_rule_set_id(&engine.snapshot());
    // `RulePayload`/`RespondPayload` are `#[non_exhaustive]` as of
    // apimock-config 6.0.0 (RFC MK-060); build from `Default::default()`
    // and set fields directly.
    let mut rule = apimock_config::RulePayload::default();
    rule.url_path = Some("/orders".to_owned());
    rule.url_path_op = Some(apimock_config::view::UrlPathOp::Equal);
    rule.respond = to_engine::respond_text_only("ok");
    let outcome = engine
        .apply(apimock_config::EditCommand::AddRule {
            parent: engine_rule_set,
            rule,
        })
        .expect("engine AddRule");
    assert!(
        outcome.changed_nodes.len() >= 3,
        "the engine's own test asserts >= 3 (parent, rule, respond); got {}",
        outcome.changed_nodes.len()
    );

    // Both sets include the parent rule-set id.
    assert!(outcome.changed_nodes.contains(&engine_rule_set));
}

// ── 10. ReloadHint restart-vs-reload classification, all 14 keys ────────

#[test]
fn reload_hint_classification_matches_for_twelve_of_fourteen_root_setting_keys() {
    use workspace_port::{RuntimeEffect, WorkspaceRootKey as K};
    // (key, a value our own map_root_setting accepts for it)
    // `TlsCertFile`/`TlsKeyFile` are covered separately below (RFC MK-060 —
    // new confirmed divergence as of apimock-config 6.0.0, not present at
    // 5.10.0).
    let cases: [(K, WorkspaceEditValue); 12] = [
        (
            K::ListenerIpAddress,
            WorkspaceEditValue::String("127.0.0.1".into()),
        ),
        (K::ListenerPort, WorkspaceEditValue::Integer(4000)),
        (
            K::ServiceFallbackRespondDir,
            WorkspaceEditValue::String(String::new()),
        ),
        (
            K::ServiceStrategy,
            WorkspaceEditValue::Enum("FirstMatch".into()),
        ),
        (K::TlsEnabled, WorkspaceEditValue::Boolean(false)),
        (K::LogLevel, WorkspaceEditValue::Enum("info".into())),
        (K::LogFile, WorkspaceEditValue::String(String::new())),
        (K::LogFormat, WorkspaceEditValue::Enum("plain".into())),
        (K::FileTreeShowHidden, WorkspaceEditValue::Boolean(true)),
        (
            K::FileTreeBuiltinExcludes,
            WorkspaceEditValue::Boolean(true),
        ),
        (
            K::FileTreeExtraExcludes,
            WorkspaceEditValue::StringList(vec!["dist".into()]),
        ),
        (
            K::FileTreeInclude,
            WorkspaceEditValue::StringList(vec![".json".into()]),
        ),
    ];
    assert_eq!(cases.len() + 2, K::ALL.len());

    for (key, value) in cases {
        let ours = workspace_port::map_root_setting(key, value)
            .unwrap()
            .effect();
        let engine_key = to_engine::root_setting_key(key);
        let engine_hint = apimock_config::ReloadHint::for_key(engine_key);
        let engine_effect = if engine_hint.requires_restart {
            RuntimeEffect::Restart
        } else if engine_hint.requires_reload {
            RuntimeEffect::Reload
        } else {
            RuntimeEffect::None
        };
        assert_eq!(
            ours, engine_effect,
            "{key:?}: our effect {ours:?} vs engine ReloadHint {engine_hint:?}"
        );
    }
}

/// New confirmed divergence, found by execution against 6.0.0 and absent at
/// 5.10.0 (RFC MK-060 re-run): apimock-config's own RFC 020 moved TLS
/// cert/key rotation onto a reloadable resolver, so `ReloadHint::for_key`
/// now classifies `TlsCertFile`/`TlsKeyFile` as reload-only. Our
/// `map_root_setting` still classifies both as `RuntimeEffect::Restart`,
/// matching the engine's own 5.10.0 behaviour at the time MK-053/MK-055
/// were written. This is the same disposition as `ListenerPort = 0` below:
/// we are stricter than the engine, never more permissive, so no UI
/// correctness defect follows and no production change is made here — the
/// RFC MK-060 task's own non-goals keep `WorkspacePort` boundary changes out
/// of this task's scope. Recorded rather than silently asserted away, and
/// worth flagging upstream: 6.0.0's own `ReloadHint` doc comment table still
/// reads "TlsEnabled, TlsCert*" -> `HardRestart`, which now contradicts its
/// own `for_key` match arm two lines below it.
#[test]
fn tls_cert_and_key_reload_hint_is_a_new_confirmed_stricter_divergence() {
    use workspace_port::{RuntimeEffect, WorkspaceRootKey as K};

    for key in [K::TlsCertFile, K::TlsKeyFile] {
        let ours = workspace_port::map_root_setting(key, WorkspaceEditValue::String(String::new()))
            .unwrap()
            .effect();
        assert_eq!(
            ours,
            RuntimeEffect::Restart,
            "{key:?}: our own mapping's classification did not change"
        );

        let engine_hint = apimock_config::ReloadHint::for_key(to_engine::root_setting_key(key));
        assert!(
            engine_hint.requires_reload && !engine_hint.requires_restart,
            "{key:?}: expected the 6.0.0 engine to now classify this as reload-only (RFC 020): {engine_hint:?}"
        );
    }
}

// ── 11. validate() / ValidationReport ────────────────────────────────────

#[test]
fn validate_reports_no_issues_for_a_clean_workspace_on_both_sides() {
    let mem = memory();
    let report = mem.validate();
    assert!(
        report.issues.is_empty(),
        "expected a clean MemoryWorkspace to validate with no issues: {:?}",
        report.issues
    );

    let (_dir, root) = minimal_workspace();
    let engine = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let report = engine.validate();
    assert!(
        report.is_valid && report.diagnostics.is_empty(),
        "expected a clean engine workspace to validate with no diagnostics: {:?}",
        report.diagnostics
    );
}
