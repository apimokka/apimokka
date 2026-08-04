//! Boundary decision: single-responsibility — unit tests for the complete
//! workspace-port value-type and mapping-function vocabulary, organized in
//! the same sub-domain order the production types are defined in
//! `workspace_port.rs` and `mapping.rs`. One test file for one contract,
//! mirroring the single-responsibility read given to `workspace_port.rs`
//! itself.

use serde_json::json;

use super::*;
use crate::rule::{BodyOp, HeaderOp, UrlPathOp};

#[test]
fn collection_edit_preserves_clears_and_replaces() {
    assert_eq!(CollectionEdit::<u8>::Preserve.into_reference_option(), None);
    assert_eq!(
        CollectionEdit::<u8>::Clear.into_reference_option(),
        Some(vec![])
    );
    assert_eq!(
        CollectionEdit::Replace(vec![1, 2]).into_reference_option(),
        Some(vec![1, 2])
    );
}

#[test]
fn workspace_paths_accept_canonical_relative_keys() {
    for value in [
        "routes.toml",
        "api/v1.toml",
        "responses/body.json",
        "A/file",
    ] {
        assert_eq!(
            parse_workspace_relative_path("path", value)
                .unwrap()
                .as_str(),
            value
        );
    }
    assert_eq!(
        parse_rule_set_path("routes.toml").unwrap().to_string(),
        "routes.toml"
    );
    assert_eq!(
        parse_rule_set_path("api/v1.toml").unwrap().to_string(),
        "api/v1.toml"
    );
}

#[test]
fn workspace_paths_reject_escape_and_host_prefix_forms() {
    let cases = [
        "",
        "/routes.toml",
        "C:/routes.toml",
        "C:routes.toml",
        "c:/x",
        "//server/share",
        "//?/C:/routes.toml",
        r"\\server\share",
        r"\\?\C:\routes.toml",
        "a\\b",
        "a//b",
        "./a",
        "a/../b",
        "a/./b",
        "a/",
        "a\0b",
    ];
    for value in cases {
        assert!(
            parse_workspace_relative_path("path", value).is_err(),
            "accepted {value:?}"
        );
    }
    assert!(parse_rule_set_path("routes.TOML").is_err());
    assert!(parse_rule_set_path("routes.json").is_err());
}

#[test]
fn rule_match_maps_all_url_operators_and_method_domain() {
    let operators = [
        (UrlPathOp::Equal, "Equal"),
        (UrlPathOp::StartsWith, "StartsWith"),
        (UrlPathOp::Contains, "Contains"),
        (UrlPathOp::EndsWith, "EndsWith"),
        (UrlPathOp::WildCard, "WildCard"),
        (UrlPathOp::NotEqual, "NotEqual"),
    ];
    for (op, label) in operators {
        let mapped = map_rule_match("/orders", Some(op), "GET").unwrap();
        assert_eq!(mapped.url_path(), Some("/orders"));
        assert_eq!(mapped.url_path_op(), Some(op));
        assert_eq!(mapped.url_path_op().unwrap().label(), label);
        assert_eq!(mapped.method(), Some("GET"));
    }
    assert_eq!(map_rule_match("", None, "").unwrap().method(), None);
    for method in ["GET", "POST", "PUT", "DELETE"] {
        assert!(map_rule_match("", None, method).is_ok());
    }
    assert!(map_rule_match("", Some(UrlPathOp::Equal), "").is_err());
    assert!(map_rule_match("/x", None, "").is_err());
    assert!(map_rule_match("", None, "PATCH").is_err());
}

#[test]
fn header_mapping_is_exhaustive_and_canonical() {
    let operators = [
        (HeaderOp::Equal, "Equal", true),
        (HeaderOp::Contains, "Contains", true),
        (HeaderOp::StartsWith, "StartsWith", true),
        (HeaderOp::EndsWith, "EndsWith", true),
        (HeaderOp::Regex, "Regex", true),
        (HeaderOp::Exists, "Exists", false),
        (HeaderOp::Absent, "Absent", false),
        (HeaderOp::NotEqual, "NotEqual", true),
        (HeaderOp::WildCard, "WildCard", true),
    ];
    for (op, label, value_bearing) in operators {
        let expected = if value_bearing { "Value" } else { "" };
        let mapped = map_header_condition("X-Request-ID", op, expected).unwrap();
        assert_eq!(mapped.name().as_str(), "x-request-id");
        assert_eq!(mapped.op().label(), label);
        assert_eq!(mapped.expected(), value_bearing.then_some("Value"));
    }
    assert!(map_header_condition("bad header", HeaderOp::Equal, "x").is_err());
    assert!(map_header_condition("x-a", HeaderOp::Exists, "unexpected").is_err());
}

#[test]
fn body_mapping_covers_all_eighteen_operators() {
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
        assert!(
            map_body_condition("items.0.id", op, draft).is_ok(),
            "failed {op:?}"
        );
    }
}

#[test]
fn body_mapping_enforces_value_grammars() {
    assert_eq!(
        map_body_condition("a", BodyOp::Equal, "")
            .unwrap()
            .expected(),
        Some(&json!(""))
    );
    assert_eq!(
        map_body_condition("a", BodyOp::EqualTyped, " null ")
            .unwrap()
            .expected(),
        Some(&Value::Null)
    );
    for bad in ["+1", "01", "1.0", "1e2", " 1", "1 ", "-0"] {
        assert!(
            map_body_condition("a", BodyOp::EqualInteger, bad).is_err(),
            "accepted {bad}"
        );
    }
    for bad in ["-1", "+1", "01", "1.0", "1e2", " 1"] {
        assert!(
            map_body_condition("a", BodyOp::ArrayLengthEqual, bad).is_err(),
            "accepted {bad}"
        );
    }
    for bad in ["", ".a", "a.", "a..b", "$", "$.a", "a\0b"] {
        assert!(
            map_body_condition(bad, BodyOp::Equal, "x").is_err(),
            "accepted {bad:?}"
        );
    }
    assert!(map_body_condition("a", BodyOp::Exists, "x").is_err());
    assert!(map_body_condition("a", BodyOp::EqualNumber, "\"1\"").is_err());
    for bad in ["9223372036854775808", "-9223372036854775809"] {
        assert!(map_body_condition("a", BodyOp::EqualInteger, bad).is_err());
    }
    assert!(map_body_condition("a", BodyOp::ArrayLengthEqual, &u128::MAX.to_string()).is_err());
    for bad in ["1 trailing", "{\"a\":1} trailing"] {
        assert!(map_body_condition("a", BodyOp::EqualTyped, bad).is_err());
    }
    for bad in ["1e400", "NaN", "Infinity"] {
        assert!(map_body_condition("a", BodyOp::EqualNumber, bad).is_err());
    }
}

#[test]
fn response_mapping_distinguishes_mode_status_and_delay() {
    let inline = map_response(ResponseMode::Inline, "", "ignored", "200 OK", "0").unwrap();
    assert_eq!(inline.text(), Some(""));
    assert_eq!(inline.file_path(), None);
    assert_eq!(inline.delay_milliseconds(), Some(0));

    let file = map_response(ResponseMode::File, "ignored", "responses/a.json", "", "").unwrap();
    assert_eq!(file.text(), None);
    assert_eq!(file.file_path().unwrap().as_str(), "responses/a.json");
    assert_eq!(file.status(), None);
    assert_eq!(file.delay_milliseconds(), None);

    for bad in ["99", "600", "200 ", "20A", "200  OK"] {
        assert!(
            map_response(ResponseMode::Inline, "x", "", bad, "").is_err(),
            "accepted {bad}"
        );
    }
    assert!(map_response(ResponseMode::File, "", "", "200", "").is_err());
    assert!(map_response(ResponseMode::Inline, "", "", "200", "-1").is_err());
}

/// RFC MK-055: `apimock-config` 5.10.0's `RespondPayload.delay_milliseconds`
/// is `Option<u32>`, not `Option<u64>` as the never-published 5.10.1 prose
/// reference stated. A value above `u32::MAX` is our defect, not an accepted
/// divergence: nothing needs it, and the engine could not represent it.
#[test]
fn response_mapping_rejects_delay_beyond_engine_u32_range() {
    assert_eq!(
        map_response(ResponseMode::Inline, "x", "", "", &u32::MAX.to_string())
            .unwrap()
            .delay_milliseconds(),
        Some(u64::from(u32::MAX))
    );
    let just_over = u64::from(u32::MAX) + 1;
    assert!(map_response(ResponseMode::Inline, "x", "", "", &just_over.to_string()).is_err());
    assert!(map_response(ResponseMode::Inline, "x", "", "", "18446744073709551615").is_err());
}

#[test]
fn every_root_key_has_a_typed_mapping_and_effect() {
    use WorkspaceEditValue as V;
    use WorkspaceRootKey as K;
    let cases = [
        (
            K::ListenerIpAddress,
            V::String("127.0.0.1".into()),
            RuntimeEffect::Restart,
        ),
        (K::ListenerPort, V::Integer(8080), RuntimeEffect::Restart),
        (
            K::ServiceFallbackRespondDir,
            V::String("responses".into()),
            RuntimeEffect::Reload,
        ),
        (
            K::ServiceStrategy,
            V::Enum("FirstMatch".into()),
            RuntimeEffect::Reload,
        ),
        (K::TlsEnabled, V::Boolean(true), RuntimeEffect::Restart),
        (
            K::TlsCertFile,
            V::String("cert.pem".into()),
            RuntimeEffect::Restart,
        ),
        (
            K::TlsKeyFile,
            V::String("key.pem".into()),
            RuntimeEffect::Restart,
        ),
        (K::LogLevel, V::Enum("info".into()), RuntimeEffect::Reload),
        (
            K::LogFile,
            V::String("logs/app.log".into()),
            RuntimeEffect::Restart,
        ),
        (K::LogFormat, V::Enum("plain".into()), RuntimeEffect::Reload),
        (
            K::FileTreeShowHidden,
            V::Boolean(true),
            RuntimeEffect::Reload,
        ),
        (
            K::FileTreeBuiltinExcludes,
            V::Boolean(true),
            RuntimeEffect::Reload,
        ),
        (
            K::FileTreeExtraExcludes,
            V::StringList(vec!["target".into()]),
            RuntimeEffect::Reload,
        ),
        (
            K::FileTreeInclude,
            V::StringList(vec!["rules".into()]),
            RuntimeEffect::Reload,
        ),
    ];
    assert_eq!(cases.len(), WorkspaceRootKey::ALL.len());
    for (key, value, effect) in cases {
        assert_eq!(map_root_setting(key, value).unwrap().effect(), effect);
    }
}

#[test]
fn root_mapping_accepts_complete_enum_domains_and_empty_optional_paths() {
    use WorkspaceEditValue as V;
    use WorkspaceRootKey as K;
    for strategy in [
        "FirstMatch",
        "UniformRandom",
        "WeightedRandom",
        "Priority",
        "RoundRobin",
    ] {
        assert!(map_root_setting(K::ServiceStrategy, V::Enum(strategy.into())).is_ok());
    }
    for level in ["error", "warn", "info", "debug", "trace"] {
        assert!(map_root_setting(K::LogLevel, V::Enum(level.into())).is_ok());
    }
    for format in ["plain", "json"] {
        assert!(map_root_setting(K::LogFormat, V::Enum(format.into())).is_ok());
    }
    for key in [
        K::ServiceFallbackRespondDir,
        K::TlsCertFile,
        K::TlsKeyFile,
        K::LogFile,
    ] {
        assert!(map_root_setting(key, V::String(String::new())).is_ok());
    }
}

#[test]
fn root_mapping_rejects_type_range_and_enum_errors() {
    use WorkspaceEditValue as V;
    use WorkspaceRootKey as K;
    assert!(map_root_setting(K::ListenerPort, V::Integer(0)).is_err());
    assert!(map_root_setting(K::ListenerPort, V::Integer(65536)).is_err());
    assert!(map_root_setting(K::ListenerPort, V::String("8080".into())).is_err());
    assert!(map_root_setting(K::ListenerIpAddress, V::String("localhost".into())).is_err());
    assert!(map_root_setting(K::ServiceStrategy, V::Enum("BestMatch".into())).is_err());
    assert!(map_root_setting(K::LogFormat, V::Enum("pretty".into())).is_err());
    assert!(map_root_setting(K::TlsCertFile, V::String("../cert.pem".into())).is_err());
    assert!(map_root_setting(K::FileTreeInclude, V::StringList(vec![String::new()])).is_err());
}

#[test]
fn transactions_are_nonempty_and_creation_keys_are_semantic() {
    assert!(EditTransaction::new(vec![]).is_err());
    assert!(SemanticCreationKey::new("").is_err());
    let key = SemanticCreationKey::new("rule-set/rule[1]/body[2]").unwrap();
    let transaction =
        EditTransaction::new(vec![EditIntent::RemoveBodyCondition { id: NodeId::new() }]).unwrap();
    assert_eq!(key.as_str(), "rule-set/rule[1]/body[2]");
    assert_eq!(transaction.intents().len(), 1);

    let duplicate = SemanticCreationKey::new("body[0]").unwrap();
    let rule_id = NodeId::new();
    let condition = map_body_condition("a", BodyOp::Equal, "same").unwrap();
    assert!(
        EditTransaction::new(vec![
            EditIntent::AddBodyCondition {
                rule_id,
                condition: condition.clone(),
                key: duplicate.clone(),
            },
            EditIntent::AddBodyCondition {
                rule_id,
                condition,
                key: duplicate,
            },
        ])
        .is_err()
    );
}

#[test]
fn runtime_effect_precedence_is_total() {
    assert_eq!(
        RuntimeEffect::None.combine(RuntimeEffect::Reload),
        RuntimeEffect::Reload
    );
    assert_eq!(
        RuntimeEffect::Reload.combine(RuntimeEffect::Restart),
        RuntimeEffect::Restart
    );
    assert_eq!(
        RuntimeEffect::Restart.combine(RuntimeEffect::None),
        RuntimeEffect::Restart
    );
}

#[test]
fn archived_subtree_requires_and_types_its_former_root() {
    let old_id = NodeId::new();
    let node = ArchivedNode {
        old_id,
        parent: None,
        key: SemanticCreationKey::new("rule-set").unwrap(),
        payload: ArchivedNodePayload::RuleSet {
            path: parse_rule_set_path("routes.toml").unwrap(),
        },
    };
    let archive = ArchivedSubtree::new(
        old_id,
        RestorePlacement::RuleSetRoot { insertion_index: 0 },
        vec![node],
    )
    .unwrap();
    assert_eq!(archive.former_root(), old_id);
    assert_eq!(
        archive.placement(),
        RestorePlacement::RuleSetRoot { insertion_index: 0 }
    );
    assert_eq!(
        archive.nodes()[0].payload.kind(),
        WorkspaceNodeKind::RuleSet
    );
    assert!(
        ArchivedSubtree::new(
            NodeId::new(),
            RestorePlacement::RuleSetRoot { insertion_index: 0 },
            archive.nodes().to_vec()
        )
        .is_err()
    );
}

fn archived_rule(old_id: NodeId, parent: Option<NodeId>, key: &str) -> ArchivedNode {
    ArchivedNode {
        old_id,
        parent,
        key: SemanticCreationKey::new(key).unwrap(),
        payload: ArchivedNodePayload::Rule(RuleEditPayload {
            rule_match: map_rule_match("", None, "").unwrap(),
            headers: CollectionEdit::Preserve,
            body: CollectionEdit::Preserve,
            respond: map_response(ResponseMode::Inline, "", "", "", "").unwrap(),
        }),
    }
}

#[test]
fn rule_restore_keeps_typed_external_parent_and_insertion_position() {
    let old_id = NodeId::new();
    let expected_parent = RuleSetId(NodeId::new());
    let other_parent = RuleSetId(NodeId::new());
    let placement = RestorePlacement::Rule {
        parent: expected_parent,
        insertion_index: 3,
    };
    let archive =
        ArchivedSubtree::new(old_id, placement, vec![archived_rule(old_id, None, "rule")]).unwrap();
    assert_eq!(archive.placement(), placement);
    assert_ne!(
        archive.placement(),
        RestorePlacement::Rule {
            parent: other_parent,
            insertion_index: 3,
        }
    );
}

#[test]
fn archive_rejects_duplicate_ids_invalid_placement_and_invalid_topology() {
    let old_id = NodeId::new();
    let node = archived_rule(old_id, None, "rule");
    let parent = RuleSetId(NodeId::new());
    assert!(
        ArchivedSubtree::new(
            old_id,
            RestorePlacement::Rule {
                parent,
                insertion_index: 0,
            },
            vec![node.clone(), node.clone()]
        )
        .is_err()
    );
    assert!(
        ArchivedSubtree::new(
            old_id,
            RestorePlacement::RuleSetRoot { insertion_index: 0 },
            vec![node.clone()]
        )
        .is_err()
    );
    let rule_set_id = NodeId::new();
    let rule_set = ArchivedNode {
        old_id: rule_set_id,
        parent: None,
        key: SemanticCreationKey::new("rule-set").unwrap(),
        payload: ArchivedNodePayload::RuleSet {
            path: parse_rule_set_path("routes.toml").unwrap(),
        },
    };
    assert!(
        ArchivedSubtree::new(
            rule_set_id,
            RestorePlacement::Rule {
                parent,
                insertion_index: 0,
            },
            vec![rule_set]
        )
        .is_err()
    );

    let orphan = ArchivedNode {
        old_id: NodeId::new(),
        parent: Some(NodeId::new()),
        key: SemanticCreationKey::new("header").unwrap(),
        payload: ArchivedNodePayload::HeaderCondition(
            map_header_condition("x-a", HeaderOp::Equal, "v").unwrap(),
        ),
    };
    assert!(
        ArchivedSubtree::new(
            old_id,
            RestorePlacement::Rule {
                parent,
                insertion_index: 0,
            },
            vec![node, orphan]
        )
        .is_err()
    );
}

#[test]
fn save_errors_preserve_typed_category_and_detail() {
    let errors = [
        (
            SaveError::validation("invalid workspace"),
            SaveErrorKind::Validation,
        ),
        (
            SaveError::injected_failure("rules.toml"),
            SaveErrorKind::InjectedFailure,
        ),
        (SaveError::io("permission denied"), SaveErrorKind::Io),
    ];
    for (error, expected_kind) in errors {
        assert_eq!(error.kind(), expected_kind);
        assert!(!error.detail().is_empty());
    }
}
