//! Canned workspace snapshots (RFC MK-047, MK-048, MK-053).

use crate::ids::{NodeId, RuleSetId};
use crate::node::{ConfigFileKind, ConfigFileView, FileNodeKind, FileNodeView};
use crate::respond::{RespondMode, RespondPayload};
use crate::rule::{
    BodyConditionPayload, BodyOp, HeaderConditionPayload, HeaderOp, RulePayload, UrlPathOp,
};
use crate::settings::{RootSettings, Strategy};
use crate::snapshot::{RuleSetView, RuleView, WorkspaceMeta, WorkspaceSnapshot};
use crate::validation::{Diagnostic, NodeValidation, Severity, ValidationIssue};

/// MK-047: A blank workspace with no rules, created from wizard input.
/// All settings default to safe values; the user fills in content after creation.
pub fn blank_workspace(name: &str, host: &str, port: u16, tls: bool) -> WorkspaceSnapshot {
    let settings = RootSettings {
        listener_ip: host.to_string(),
        listener_port: port,
        tls_enabled: tls,
        ..Default::default()
    };

    WorkspaceSnapshot {
        meta: WorkspaceMeta {
            name: name.to_string(),
            path: format!("~/{name}/apimock.toml"),
        },
        root_settings: settings,
        rule_sets: vec![],
        fallback_files: vec![],
        middleware_scripts: vec![],
        diagnostics: vec![],
    }
}

/// Realises the example workspace from external design § 36
/// (`shop-api-mock`, two rule sets, six rules, weighted/priority
/// examples, fallback files, middleware scripts, validation samples).
pub fn shop_api_mock() -> WorkspaceSnapshot {
    let main_id = RuleSetId(NodeId::new());
    let errors_id = RuleSetId(NodeId::new());

    // ---- main.toml --------------------------------------------------
    let r_health = RuleView {
        id: NodeId::new(),
        payload: RulePayload {
            url_path: "/health".into(),
            url_path_op: Some(UrlPathOp::Equal),
            method: "GET".into(),
            respond: RespondPayload {
                mode: RespondMode::InlineText,
                text: r#"{"status":"ok"}"#.into(),
                status: "200 OK".into(),
                ..Default::default()
            },
            ..Default::default()
        },
        validation: NodeValidation::default(),
        matched_by_latest_trace: false,
    };

    let r_users = RuleView {
        id: NodeId::new(),
        payload: RulePayload {
            url_path: "/api/users".into(),
            url_path_op: Some(UrlPathOp::Equal),
            method: "GET".into(),
            respond: RespondPayload {
                mode: RespondMode::ServeFile,
                file_path: "responses/users.json".into(),
                status: "200 OK".into(),
                ..Default::default()
            },
            ..Default::default()
        },
        validation: NodeValidation::default(),
        matched_by_latest_trace: false,
    };

    let r_orders_main = RuleView {
        id: NodeId::new(),
        payload: RulePayload {
            url_path: "/api/orders".into(),
            url_path_op: Some(UrlPathOp::Equal),
            method: "POST".into(),
            headers: vec![HeaderConditionPayload {
                name: "content-type".into(),
                op: HeaderOp::Contains,
                value: "application/json".into(),
            }],
            body: vec![
                BodyConditionPayload {
                    path: "action".into(),
                    op: BodyOp::Equal,
                    value: "create".into(),
                },
                BodyConditionPayload {
                    path: "items.0.qty".into(),
                    op: BodyOp::GreaterThan,
                    value: "0".into(),
                },
            ],
            respond: RespondPayload {
                mode: RespondMode::ServeFile,
                file_path: "responses/order-created.json".into(),
                status: "201 Created".into(),
                delay_milliseconds: 120,
                ..Default::default()
            },
            weight: Some(3),
            ..Default::default()
        },
        validation: NodeValidation::default(),
        matched_by_latest_trace: true,
    };

    let main_set = RuleSetView {
        id: main_id,
        file: ConfigFileView {
            kind: ConfigFileKind::RuleSet,
            path: "rules/main.toml".into(),
            dirty: true, // demonstrates the dirty indicator
        },
        rules: vec![r_health, r_users, r_orders_main],
        validation: NodeValidation::default(),
    };

    // ---- error-scenarios.toml --------------------------------------
    let r_orders_error_payment = RuleView {
        id: NodeId::new(),
        payload: RulePayload {
            url_path: "/api/orders".into(),
            url_path_op: Some(UrlPathOp::Equal),
            method: "POST".into(),
            headers: vec![HeaderConditionPayload {
                name: "x-test-scenario".into(),
                op: HeaderOp::Equal,
                value: "payment-failed".into(),
            }],
            respond: RespondPayload {
                mode: RespondMode::InlineText,
                text: r#"{"error":"payment_required","detail":"funds insufficient"}"#.into(),
                status: "402 Payment Required".into(),
                ..Default::default()
            },
            priority: Some(10),
            ..Default::default()
        },
        validation: NodeValidation {
            issues: vec![ValidationIssue {
                node_id: None, // filled below at snapshot level too
                severity: Severity::Warning,
                message: "WeightedRandom is selected, but this rule has no weight set.".into(),
                location: Some("service.strategy".into()),
            }],
        },
        matched_by_latest_trace: false,
    };

    let r_users_404 = RuleView {
        id: NodeId::new(),
        payload: RulePayload {
            url_path: "/api/users/404".into(),
            url_path_op: Some(UrlPathOp::Equal),
            method: "GET".into(),
            respond: RespondPayload {
                mode: RespondMode::InlineText,
                text: r#"{"error":"not_found"}"#.into(),
                status: "404 Not Found".into(),
                ..Default::default()
            },
            ..Default::default()
        },
        validation: NodeValidation::default(),
        matched_by_latest_trace: false,
    };

    let r_legacy_bad_path = RuleView {
        id: NodeId::new(),
        payload: RulePayload {
            url_path: "/api/legacy".into(),
            url_path_op: Some(UrlPathOp::Equal),
            method: "GET".into(),
            body: vec![BodyConditionPayload {
                // Deliberately wrong syntax: demonstrates the validation error
                // listed in § 36.3 (`Use dotted path syntax, not JSONPath.`).
                path: "$.user.id".into(),
                op: BodyOp::Equal,
                value: "42".into(),
            }],
            respond: RespondPayload {
                mode: RespondMode::InlineText,
                text: r#"{"legacy":true}"#.into(),
                status: "200 OK".into(),
                ..Default::default()
            },
            ..Default::default()
        },
        validation: NodeValidation {
            issues: vec![ValidationIssue {
                node_id: None,
                severity: Severity::Error,
                message: "Use dotted path syntax, not JSONPath.".into(),
                location: Some("rules[2].when.body[0].path".into()),
            }],
        },
        matched_by_latest_trace: false,
    };

    let errors_set = RuleSetView {
        id: errors_id,
        file: ConfigFileView {
            kind: ConfigFileKind::RuleSet,
            path: "rules/error-scenarios.toml".into(),
            dirty: false,
        },
        rules: vec![r_orders_error_payment, r_users_404, r_legacy_bad_path],
        validation: NodeValidation::default(),
    };

    // ---- file fallback ---------------------------------------------
    let fallback_files = vec![
        FileNodeView {
            name: "health.json".into(),
            path: "responses/health.json".into(),
            kind: FileNodeKind::File,
            route_hint: Some("/health".into()),
        },
        FileNodeView {
            name: "users.json".into(),
            path: "responses/users.json".into(),
            kind: FileNodeKind::File,
            route_hint: Some("/users".into()),
        },
        FileNodeView {
            name: "order-created.json".into(),
            path: "responses/order-created.json".into(),
            kind: FileNodeKind::File,
            route_hint: Some("/order-created".into()),
        },
    ];

    // ---- middleware -------------------------------------------------
    let middleware_scripts = vec![
        ConfigFileView {
            kind: ConfigFileKind::Middleware,
            path: "middleware/auth.rhai".into(),
            dirty: false,
        },
        ConfigFileView {
            kind: ConfigFileKind::Middleware,
            path: "middleware/rewrite.rhai".into(),
            dirty: false,
        },
    ];

    // ---- workspace-wide diagnostics --------------------------------
    let diagnostics = vec![Diagnostic {
        node_id: None,
        severity: Severity::Info,
        message: "No include filter is set. All supported files are visible.".into(),
    }];

    let settings = RootSettings {
        strategy: Strategy::WeightedRandom,
        ..Default::default()
    };

    WorkspaceSnapshot {
        meta: WorkspaceMeta {
            name: "shop-api-mock".into(),
            path: "~/projects/shop-api-mock".into(),
        },
        root_settings: settings,
        rule_sets: vec![main_set, errors_set],
        fallback_files,
        middleware_scripts,
        diagnostics,
    }
}

/// Canonical rich workspace seed for the MK-053 in-memory adapter.
///
/// [`shop_api_mock`] deliberately contains one invalid JSONPath-style body
/// path to demonstrate legacy validation UI. A fail-closed workspace port must
/// not admit that draft, so app session construction uses this explicit seed
/// with the equivalent dotted path and without the obsolete inline issue.
pub fn shop_api_canonical_seed() -> WorkspaceSnapshot {
    let mut workspace = shop_api_mock();
    let legacy_rule = workspace
        .rule_sets
        .iter_mut()
        .flat_map(|rule_set| &mut rule_set.rules)
        .find(|rule| rule.payload.url_path == "/api/legacy")
        .expect("shop API fixture retains its legacy example rule");
    let condition = legacy_rule
        .payload
        .body
        .first_mut()
        .expect("legacy example rule retains its body condition");
    condition.path = "user.id".into();
    legacy_rule.validation = NodeValidation::default();
    workspace
}

/// MK-048: A minimal workspace — one rule set, one health-check rule.
/// The idiomatic "first rule" for any new mock service.
pub fn minimal_workspace(name: &str, host: &str, port: u16, tls: bool) -> WorkspaceSnapshot {
    let rs_id = RuleSetId(NodeId::new());
    let rule_id = NodeId::new();

    let rule = RuleView {
        id: rule_id,
        payload: RulePayload {
            url_path: "/health".into(),
            url_path_op: Some(UrlPathOp::Equal),
            method: "GET".into(),
            headers: vec![],
            body: vec![],
            respond: RespondPayload {
                mode: RespondMode::InlineText,
                text: "{\"status\":\"ok\"}".into(),
                file_path: String::new(),
                status: "200 OK".into(),
                delay_milliseconds: 0,
            },
            weight: None,
            priority: None,
        },
        validation: NodeValidation::default(),
        matched_by_latest_trace: false,
    };

    let rs = RuleSetView {
        id: rs_id,
        file: ConfigFileView {
            kind: ConfigFileKind::RuleSet,
            path: "rules/main.toml".into(),
            dirty: false,
        },
        rules: vec![rule],
        validation: NodeValidation::default(),
    };

    let settings = RootSettings {
        listener_ip: host.to_string(),
        listener_port: port,
        tls_enabled: tls,
        ..Default::default()
    };

    WorkspaceSnapshot {
        meta: WorkspaceMeta {
            name: name.to_string(),
            path: format!("~/{name}/apimock.toml"),
        },
        root_settings: settings,
        rule_sets: vec![rs],
        fallback_files: vec![],
        middleware_scripts: vec![],
        diagnostics: vec![],
    }
}

#[cfg(test)]
#[path = "workspace_fixtures/tests.rs"]
mod tests;
