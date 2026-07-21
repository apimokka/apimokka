//! Canned mock data for the v1 mockup.
//!
//! Realises the example workspace from external design § 36
//! (`shop-api-mock`, two rule sets, six rules, weighted/priority
//! examples, fallback files, middleware scripts, four trace outcomes,
//! validation samples).

use crate::ids::{NodeId, RuleSetId};
use crate::node::{ConfigFileKind, ConfigFileView, FileNodeKind, FileNodeView};
use crate::respond::{RespondMode, RespondPayload};
use crate::rule::{
    BodyConditionPayload, BodyOp, HeaderConditionPayload, HeaderOp, RulePayload, UrlPathOp,
};
use crate::settings::{RootSettings, Strategy};
use crate::snapshot::{RuleSetView, RuleView, WorkspaceMeta, WorkspaceSnapshot};
use crate::trace::{MatchTraceEvent, RequestSummary, TraceOutcome};
use crate::validation::{Diagnostic, NodeValidation, Severity, ValidationIssue};

/// MK-047: A blank workspace with no rules, created from wizard input.
/// All settings default to safe values; the user fills in content after creation.
pub fn blank_workspace(name: &str, host: &str, port: u16, tls: bool) -> WorkspaceSnapshot {
    let mut settings = RootSettings::default();
    settings.listener_ip = host.to_string();
    settings.listener_port = port;
    settings.tls_enabled = tls;

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

    let mut settings = RootSettings::default();
    settings.strategy = Strategy::WeightedRandom;

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

/// Four canned trace events covering matched / fallback / miss / error
/// (external design § 36.2). Ordered newest first because the trace
/// panel auto-scrolls to the top.
pub fn sample_trace_events() -> Vec<MatchTraceEvent> {
    vec![
        MatchTraceEvent {
            event_id: 4,
            time: "12:04:18.331".into(),
            duration_ms: 6,
            request: RequestSummary {
                method: "POST".into(),
                url_path: "/api/orders".into(),
                headers: vec![("content-type".into(), "application/json".into())],
                body_preview: Some(r#"{ "action": "create" }"#.into()),
            },
            outcome: TraceOutcome::Error {
                kind: "RespondFile".into(),
                message: "responses/order-created.json: permission denied".into(),
            },
            dropped_count: 0,
        },
        MatchTraceEvent {
            event_id: 3,
            time: "12:04:16.771".into(),
            duration_ms: 2,
            request: RequestSummary {
                method: "GET".into(),
                url_path: "/api/unknown".into(),
                headers: vec![("accept".into(), "*/*".into())],
                body_preview: None,
            },
            outcome: TraceOutcome::Miss {
                status: "404 Not Found".into(),
            },
            dropped_count: 0,
        },
        MatchTraceEvent {
            event_id: 2,
            time: "12:04:14.008".into(),
            duration_ms: 1,
            request: RequestSummary {
                method: "GET".into(),
                url_path: "/health".into(),
                headers: vec![("user-agent".into(), "curl/8.5".into())],
                body_preview: None,
            },
            outcome: TraceOutcome::Fallback {
                file_path: "responses/health.json".into(),
                status: "200 OK".into(),
            },
            dropped_count: 3,
        },
        MatchTraceEvent {
            event_id: 1,
            time: "12:04:13.152".into(),
            duration_ms: 4,
            request: RequestSummary {
                method: "POST".into(),
                url_path: "/api/orders".into(),
                headers: vec![
                    ("content-type".into(), "application/json".into()),
                    ("x-trace-id".into(), "abc-123".into()),
                ],
                body_preview: Some(r#"{ "action": "create", "items": [{ "qty": 2 }] }"#.into()),
            },
            outcome: TraceOutcome::Matched {
                rule_set_index: 0,
                rule_index: 2,
            },
            dropped_count: 0,
        },
    ]
}

/// Recent workspace cards shown on Welcome / Dashboard.
pub fn recent_workspaces() -> Vec<RecentWorkspace> {
    vec![
        RecentWorkspace {
            name: "shop-api-mock".into(),
            path: "~/projects/shop-api-mock".into(),
            last_opened: "today".into(),
            pinned: true,
        },
        RecentWorkspace {
            name: "billing-sandbox".into(),
            path: "~/work/billing-sandbox".into(),
            last_opened: "yesterday".into(),
            pinned: false,
        },
        RecentWorkspace {
            name: "qa-edge-cases".into(),
            path: "~/qa/edge-cases".into(),
            last_opened: "3 days ago".into(),
            pinned: false,
        },
    ]
}

#[derive(Debug, Clone)]
pub struct RecentWorkspace {
    pub name: String,
    pub path: String,
    pub last_opened: String,
    pub pinned: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_workspace_is_well_formed() {
        let snap = shop_api_mock();
        // Every workspace should have at least one rule set, each with rules.
        assert!(!snap.rule_sets.is_empty(), "mock must have rule sets");
        for rs in &snap.rule_sets {
            assert!(
                !rs.rules.is_empty(),
                "rule set {:?} has no rules",
                rs.file.path
            );
            // Rule summaries are user-facing labels — never empty.
            for r in &rs.rules {
                assert!(
                    !r.summary().trim().is_empty(),
                    "rule summary must be non-empty"
                );
            }
        }
    }

    #[test]
    fn mock_has_fallback_files_with_routes() {
        let snap = shop_api_mock();
        assert!(
            !snap.fallback_files.is_empty(),
            "mock must have fallback files"
        );
        // Each fallback file should advertise the route it serves.
        assert!(
            snap.fallback_files.iter().any(|f| f.route_hint.is_some()),
            "at least one fallback file should have a route hint"
        );
    }

    #[test]
    fn sample_trace_has_distinct_event_ids() {
        let events = sample_trace_events();
        assert!(!events.is_empty());
        let mut ids: Vec<u64> = events.iter().map(|e| e.event_id).collect();
        let before = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(before, ids.len(), "trace event ids must be unique");
    }
}

#[cfg(test)]
mod tests_blank {
    use super::*;

    #[test]
    fn blank_workspace_uses_wizard_inputs() {
        let ws = blank_workspace("payments-mock", "0.0.0.0", 9090, true);
        assert_eq!(ws.meta.name, "payments-mock");
        assert_eq!(ws.root_settings.listener_ip, "0.0.0.0");
        assert_eq!(ws.root_settings.listener_port, 9090);
        assert!(ws.root_settings.tls_enabled);
        assert!(
            ws.rule_sets.is_empty(),
            "blank workspace starts with no rules"
        );
        assert!(ws.fallback_files.is_empty());
        assert!(ws.diagnostics.is_empty());
    }
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

    let mut settings = RootSettings::default();
    settings.listener_ip = host.to_string();
    settings.listener_port = port;
    settings.tls_enabled = tls;

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
