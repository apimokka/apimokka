//! RFC MK-055 Tier 1 — mapping totality.
//!
//! Boundary decision: single-responsibility — every test proves mapping
//! totality for one `WorkspacePort`-to-engine type conversion. Mirrors one
//! contract tier exactly, per RFC MK-057's own boundary audit.
//!
//! For every `WorkspacePort` type that corresponds to an engine type, prove
//! the conversion in `to_engine` is total over our type's domain by driving
//! it through a real `apimock_config::Workspace::apply`: not just "this
//! typechecks," but "the engine's own validation accepts it." Where a
//! conversion is not total (`status`/`delay` boundaries), the boundary is
//! tested explicitly rather than only the happy path.

use apimokka_model::rule::{BodyOp, HeaderOp, UrlPathOp};
use apimokka_model::workspace_port::{self, ResponseMode, WorkspaceEditValue, WorkspaceRootKey};

use crate::fixture::{minimal_workspace, workspace_with_headers_and_body};
use crate::to_engine;

fn rule_set_node(snap: &apimock_config::WorkspaceSnapshot) -> apimock_config::NodeId {
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

fn first_rule_node(snap: &apimock_config::WorkspaceSnapshot) -> apimock_config::NodeId {
    snap.files
        .iter()
        .flat_map(|file| &file.nodes)
        .find(|node| matches!(node.kind, apimock_config::NodeKind::Rule))
        .expect("fixture has at least one rule")
        .id
}

fn first_respond_node(snap: &apimock_config::WorkspaceSnapshot) -> apimock_config::NodeId {
    snap.files
        .iter()
        .flat_map(|file| &file.nodes)
        .find(|node| matches!(node.kind, apimock_config::NodeKind::Respond))
        .expect("fixture has at least one respond node")
        .id
}

// ── URL path operator (6) ───────────────────────────────────────────────

#[test]
fn every_url_path_operator_is_accepted_by_the_real_engine() {
    let (_dir, root) = minimal_workspace();
    let mut workspace = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let rule_id = first_rule_node(&workspace.snapshot());

    for op in UrlPathOp::all() {
        let canonical =
            workspace_port::map_rule_match("/api/orders", Some(op), "").expect("valid rule match");
        let (url_path, url_path_op, method) = to_engine::rule_match_payload(&canonical);
        let outcome = workspace.apply(apimock_config::EditCommand::UpdateRule {
            id: rule_id,
            rule: apimock_config::RulePayload {
                url_path,
                url_path_op,
                method,
                ..Default::default()
            },
        });
        assert!(
            outcome.is_ok(),
            "engine rejected UrlPathOp::{op:?}: {outcome:?}"
        );
    }
}

// ── Method domain (Any, GET, POST, PUT, DELETE) ─────────────────────────

#[test]
fn every_accepted_method_value_is_accepted_by_the_real_engine() {
    let (_dir, root) = minimal_workspace();
    let mut workspace = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let rule_id = first_rule_node(&workspace.snapshot());

    for method in ["", "GET", "POST", "PUT", "DELETE"] {
        let canonical = workspace_port::map_rule_match("", None, method).expect("valid rule match");
        let (url_path, url_path_op, mapped_method) = to_engine::rule_match_payload(&canonical);
        let outcome = workspace.apply(apimock_config::EditCommand::UpdateRule {
            id: rule_id,
            rule: apimock_config::RulePayload {
                url_path,
                url_path_op,
                method: mapped_method,
                ..Default::default()
            },
        });
        assert!(
            outcome.is_ok(),
            "engine rejected method {method:?}: {outcome:?}"
        );
    }
}

// ── Header operators (9) ────────────────────────────────────────────────

#[test]
fn every_header_operator_is_accepted_by_the_real_engine() {
    let (_dir, root) = workspace_with_headers_and_body();
    let mut workspace = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let rule_id = first_rule_node(&workspace.snapshot());

    for op in HeaderOp::all() {
        let expected = if op.value_irrelevant() { "" } else { "value" };
        let canonical = workspace_port::map_header_condition("X-Request-ID", op, expected)
            .expect("valid header condition");
        let outcome = workspace.apply(apimock_config::EditCommand::AddHeaderCondition {
            rule_id,
            condition: to_engine::header_condition(&canonical),
        });
        assert!(
            outcome.is_ok(),
            "engine rejected HeaderOp::{op:?}: {outcome:?}"
        );
    }
}

// ── Body operators (18) ─────────────────────────────────────────────────

#[test]
fn every_body_operator_is_accepted_by_the_real_engine() {
    let (_dir, root) = workspace_with_headers_and_body();
    let mut workspace = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let rule_id = first_rule_node(&workspace.snapshot());

    for op in BodyOp::all() {
        let draft = match op {
            BodyOp::Equal
            | BodyOp::EqualString
            | BodyOp::Contains
            | BodyOp::StartsWith
            | BodyOp::EndsWith
            | BodyOp::Regex => "text",
            BodyOp::EqualTyped => r#"{"id":42}"#,
            BodyOp::ArrayContains => r#"[1,2]"#,
            BodyOp::EqualNumber
            | BodyOp::GreaterThan
            | BodyOp::LessThan
            | BodyOp::GreaterOrEqual
            | BodyOp::LessOrEqual => "1.5e2",
            BodyOp::EqualInteger => "-42",
            BodyOp::ArrayLengthEqual | BodyOp::ArrayLengthAtLeast => "42",
            BodyOp::Exists | BodyOp::Absent => "",
        };
        let canonical = workspace_port::map_body_condition("items.0.id", op, draft)
            .expect("valid body condition");
        let outcome = workspace.apply(apimock_config::EditCommand::AddBodyCondition {
            rule_id,
            condition: to_engine::body_condition(&canonical),
        });
        assert!(
            outcome.is_ok(),
            "engine rejected BodyOp::{op:?}: {outcome:?}"
        );
    }
}

/// The specific divergence `to_engine::body_condition` documents: our
/// `Exists`/`Absent` conditions carry no value, but the engine's
/// `BodyConditionPayload.value` field is mandatory. This test is the
/// boundary proof that mapping `None` to `Value::Null` is accepted by the
/// real engine, not merely by our own type system.
#[test]
fn presence_body_conditions_map_absent_value_to_json_null_and_are_accepted() {
    let (_dir, root) = workspace_with_headers_and_body();
    let mut workspace = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let rule_id = first_rule_node(&workspace.snapshot());

    for op in [BodyOp::Exists, BodyOp::Absent] {
        let canonical =
            workspace_port::map_body_condition("items.0.id", op, "").expect("valid condition");
        let payload = to_engine::body_condition(&canonical);
        assert_eq!(payload.value, serde_json::Value::Null);
        let outcome = workspace.apply(apimock_config::EditCommand::AddBodyCondition {
            rule_id,
            condition: payload,
        });
        assert!(
            outcome.is_ok(),
            "engine rejected presence BodyOp::{op:?}: {outcome:?}"
        );
    }
}

// ── Response modes and status/delay boundaries ──────────────────────────

#[test]
fn both_response_modes_are_accepted_by_the_real_engine() {
    let (_dir, root) = minimal_workspace();
    let mut workspace = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let respond_id = first_respond_node(&workspace.snapshot());

    let inline = workspace_port::map_response(ResponseMode::Inline, "hi", "", "200 OK", "0")
        .expect("valid inline response");
    let outcome = workspace.apply(apimock_config::EditCommand::UpdateRespond {
        id: respond_id,
        respond: to_engine::respond(&inline),
    });
    assert!(
        outcome.is_ok(),
        "engine rejected inline respond: {outcome:?}"
    );

    let file =
        workspace_port::map_response(ResponseMode::File, "", "responses/orders.json", "", "")
            .expect("valid file response");
    let outcome = workspace.apply(apimock_config::EditCommand::UpdateRespond {
        id: respond_id,
        respond: to_engine::respond(&file),
    });
    assert!(outcome.is_ok(), "engine rejected file respond: {outcome:?}");
}

/// Boundary proof for the `status: String -> Option<u16>` conversion: the
/// engine's own validation must accept both ends of the `100..=599` range
/// our `validate_status` allows, confirming the conversion is total over our
/// domain rather than merely well-typed.
#[test]
fn status_boundary_values_round_trip_through_the_real_engine() {
    let (_dir, root) = minimal_workspace();
    let mut workspace = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let respond_id = first_respond_node(&workspace.snapshot());

    for status in ["100", "599", "200 OK", "404 Not Found"] {
        let canonical = workspace_port::map_response(ResponseMode::Inline, "x", "", status, "")
            .expect("valid status");
        let payload = to_engine::respond(&canonical);
        assert_eq!(payload.status, Some(status[..3].parse().unwrap()));
        let outcome = workspace.apply(apimock_config::EditCommand::UpdateRespond {
            id: respond_id,
            respond: payload,
        });
        assert!(
            outcome.is_ok(),
            "engine rejected status {status:?}: {outcome:?}"
        );
    }
}

/// Boundary proof for the `delay: u64 -> u32` correction (RFC MK-055 our
/// defect fix in `map_response`): `u32::MAX` is accepted end to end; a value
/// above it is now rejected by our own mapping before an engine call is even
/// possible, which is the accepted handling rule for this divergence.
#[test]
fn delay_boundary_at_u32_max_round_trips_through_the_real_engine() {
    let (_dir, root) = minimal_workspace();
    let mut workspace = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let respond_id = first_respond_node(&workspace.snapshot());

    let canonical =
        workspace_port::map_response(ResponseMode::Inline, "x", "", "", &u32::MAX.to_string())
            .expect("u32::MAX delay is valid");
    let payload = to_engine::respond(&canonical);
    assert_eq!(payload.delay_milliseconds, Some(u32::MAX));
    let outcome = workspace.apply(apimock_config::EditCommand::UpdateRespond {
        id: respond_id,
        respond: payload,
    });
    assert!(
        outcome.is_ok(),
        "engine rejected delay u32::MAX: {outcome:?}"
    );

    let just_over = u64::from(u32::MAX) + 1;
    assert!(
        workspace_port::map_response(ResponseMode::Inline, "x", "", "", &just_over.to_string())
            .is_err(),
        "map_response should reject a delay the engine cannot represent"
    );
}

// ── Root settings (14 keys) ─────────────────────────────────────────────

/// 11 of the 14 keys: our canonical value is accepted by the real engine
/// verbatim through `to_engine::edit_value`. `ServiceStrategy` and
/// `LogFormat` are covered separately below — they are accepted
/// divergences, not totality failures, and asserting they fail here would
/// hide the finding rather than document it.
#[test]
fn eleven_of_fourteen_root_setting_keys_are_accepted_by_the_real_engine_verbatim() {
    let (_dir, root) = minimal_workspace();
    let mut workspace = apimock_config::Workspace::load(root).expect("load fixture workspace");

    let cases: [(WorkspaceRootKey, WorkspaceEditValue); 11] = [
        (
            WorkspaceRootKey::ListenerIpAddress,
            WorkspaceEditValue::String("127.0.0.1".to_owned()),
        ),
        (
            WorkspaceRootKey::ListenerPort,
            WorkspaceEditValue::Integer(3100),
        ),
        (
            WorkspaceRootKey::ServiceFallbackRespondDir,
            WorkspaceEditValue::String(String::new()),
        ),
        (
            WorkspaceRootKey::TlsEnabled,
            WorkspaceEditValue::Boolean(false),
        ),
        (
            WorkspaceRootKey::TlsCertFile,
            WorkspaceEditValue::String(String::new()),
        ),
        (
            WorkspaceRootKey::TlsKeyFile,
            WorkspaceEditValue::String(String::new()),
        ),
        (
            WorkspaceRootKey::LogLevel,
            WorkspaceEditValue::Enum("info".to_owned()),
        ),
        (
            WorkspaceRootKey::LogFile,
            WorkspaceEditValue::String(String::new()),
        ),
        (
            WorkspaceRootKey::FileTreeShowHidden,
            WorkspaceEditValue::Boolean(true),
        ),
        (
            WorkspaceRootKey::FileTreeBuiltinExcludes,
            WorkspaceEditValue::Boolean(true),
        ),
        (
            WorkspaceRootKey::FileTreeExtraExcludes,
            WorkspaceEditValue::StringList(vec!["dist".to_owned()]),
        ),
    ];

    for (key, value) in cases {
        let canonical = workspace_port::map_root_setting(key, value)
            .unwrap_or_else(|error| panic!("our own mapping rejected {key:?}: {error:?}"));
        let outcome = workspace.apply(apimock_config::EditCommand::UpdateRootSetting {
            key: to_engine::root_setting_key(canonical.key()),
            value: to_engine::edit_value(canonical.value()),
        });
        assert!(outcome.is_ok(), "engine rejected {key:?}: {outcome:?}");
    }

    // FileTreeInclude is exercised separately (its own key case above stops
    // at 11 to keep this list within the verbatim-acceptance group); include
    // it here to still cover 12 of the 14 verbatim.
    let canonical = workspace_port::map_root_setting(
        WorkspaceRootKey::FileTreeInclude,
        WorkspaceEditValue::StringList(vec![".json".to_owned()]),
    )
    .expect("valid FileTreeInclude value");
    let outcome = workspace.apply(apimock_config::EditCommand::UpdateRootSetting {
        key: to_engine::root_setting_key(canonical.key()),
        value: to_engine::edit_value(canonical.value()),
    });
    assert!(
        outcome.is_ok(),
        "engine rejected FileTreeInclude: {outcome:?}"
    );
}

/// The two remaining keys: accepted divergences. Our canonical value, sent
/// verbatim, is *rejected* by the real engine — documenting the failure is
/// the point, not a bug in this test. The translated wire value (per the
/// accepted handling rule in `to_engine::strategy_wire_value`/
/// `log_format_wire_value`) is then accepted.
#[test]
fn service_strategy_and_log_format_are_accepted_divergences_not_verbatim_matches() {
    let (_dir, root) = minimal_workspace();
    let mut workspace = apimock_config::Workspace::load(root).expect("load fixture workspace");

    let strategy = workspace_port::map_root_setting(
        WorkspaceRootKey::ServiceStrategy,
        WorkspaceEditValue::Enum("FirstMatch".to_owned()),
    )
    .expect("our mapping accepts the PascalCase label the UI sends");
    let workspace_port::WorkspaceEditValue::Enum(label) = strategy.value() else {
        panic!("ServiceStrategy value must be an Enum");
    };
    let verbatim_outcome = workspace.apply(apimock_config::EditCommand::UpdateRootSetting {
        key: to_engine::root_setting_key(strategy.key()),
        value: apimock_config::EditValue::Enum(label.clone()),
    });
    assert!(
        verbatim_outcome.is_err(),
        "expected the engine to reject the PascalCase strategy label verbatim"
    );
    let translated_outcome = workspace.apply(apimock_config::EditCommand::UpdateRootSetting {
        key: to_engine::root_setting_key(strategy.key()),
        value: apimock_config::EditValue::Enum(to_engine::strategy_wire_value(label).to_owned()),
    });
    assert!(
        translated_outcome.is_ok(),
        "engine rejected the translated snake_case strategy value: {translated_outcome:?}"
    );

    let log_format = workspace_port::map_root_setting(
        WorkspaceRootKey::LogFormat,
        WorkspaceEditValue::Enum("plain".to_owned()),
    )
    .expect("our mapping accepts \"plain\", our own default");
    let workspace_port::WorkspaceEditValue::Enum(label) = log_format.value() else {
        panic!("LogFormat value must be an Enum");
    };
    let verbatim_outcome = workspace.apply(apimock_config::EditCommand::UpdateRootSetting {
        key: to_engine::root_setting_key(log_format.key()),
        value: apimock_config::EditValue::Enum(label.clone()),
    });
    assert!(
        verbatim_outcome.is_err(),
        "expected the engine to reject \"plain\" verbatim"
    );
    let translated_outcome = workspace.apply(apimock_config::EditCommand::UpdateRootSetting {
        key: to_engine::root_setting_key(log_format.key()),
        value: apimock_config::EditValue::Enum(to_engine::log_format_wire_value(label).to_owned()),
    });
    assert!(
        translated_outcome.is_ok(),
        "engine rejected the translated \"text\" value: {translated_outcome:?}"
    );
}

/// Accepted divergence: the engine allows `ListenerPort = 0` (OS-assigned
/// ephemeral port); our own `map_root_setting` deliberately rejects it
/// (existing test: `workspace_port::tests::root_mapping_rejects_type_range_and_enum_errors`).
/// We are stricter than the engine, never more permissive, so no defect
/// follows — recorded here as Tier 1 boundary evidence rather than left
/// undiscovered.
#[test]
fn listener_port_zero_is_valid_for_the_engine_but_rejected_by_our_stricter_mapping() {
    let (_dir, root) = minimal_workspace();
    let mut workspace = apimock_config::Workspace::load(root).expect("load fixture workspace");

    assert!(
        workspace_port::map_root_setting(
            WorkspaceRootKey::ListenerPort,
            WorkspaceEditValue::Integer(0)
        )
        .is_err(),
        "our own mapping should still reject port 0"
    );
    let outcome = workspace.apply(apimock_config::EditCommand::UpdateRootSetting {
        key: apimock_config::RootSettingKey::ListenerPort,
        value: apimock_config::EditValue::Integer(0),
    });
    assert!(
        outcome.is_ok(),
        "expected the real engine to accept port 0 even though we do not: {outcome:?}"
    );
}

// ── AddRuleSet path (String, not RuleSetPath) ───────────────────────────

#[test]
fn rule_set_path_converts_to_a_string_the_real_engine_accepts() {
    let (dir, root) = minimal_workspace();
    let mut workspace = apimock_config::Workspace::load(root).expect("load fixture workspace");

    let canonical =
        workspace_port::parse_rule_set_path("more/orders.toml").expect("valid rule-set path");
    let path = to_engine::rule_set_path(&canonical);
    assert_eq!(path, "more/orders.toml");

    let outcome = workspace.apply(apimock_config::EditCommand::AddRuleSet { path });
    // The engine's `AddRuleSet` requires the referenced file to already
    // exist on disk and parse as a valid rule set (unlike our in-memory
    // adapter, which does not model a filesystem at all). This is a Tier 2
    // behavioural-equivalence finding, not a Tier 1 mapping-totality
    // failure: the *type* conversion is total and accepted at the payload
    // level; the *file-existence* precondition is a real-engine-only
    // constraint captured here rather than asserted away.
    assert!(
        outcome.is_err(),
        "expected the engine to require the referenced rule-set file to exist on disk"
    );

    std::fs::create_dir_all(dir.path().join("more")).expect("create rule-set subdir");
    std::fs::write(dir.path().join("more/orders.toml"), "rules = []\n")
        .expect("create minimal valid rule set");
    let canonical =
        workspace_port::parse_rule_set_path("more/orders.toml").expect("valid rule-set path");
    let outcome = workspace.apply(apimock_config::EditCommand::AddRuleSet {
        path: to_engine::rule_set_path(&canonical),
    });
    assert!(
        outcome.is_ok(),
        "engine rejected AddRuleSet once the file exists and parses: {outcome:?}"
    );
}

// ── Rule set node access sanity (used by other tier1/tier2 modules) ────

#[test]
fn rule_set_node_helper_finds_the_fixture_rule_set() {
    let (_dir, root) = minimal_workspace();
    let workspace = apimock_config::Workspace::load(root).expect("load fixture workspace");
    let _ = rule_set_node(&workspace.snapshot());
}
