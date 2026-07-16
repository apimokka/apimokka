use apimokka_model::{
    BodyConditionPayload, BodyOp, HeaderConditionPayload, HeaderOp, RulePayload, UrlPathOp,
};

use super::super::{
    ConditionOutcome, DiagnosticScope, EvaluationError, TestRequest, TestRuleOutcome, evaluate,
};
use super::evaluate_rule;

#[test]
fn selection_error_has_no_condition_results() {
    let result = evaluate(
        None,
        TestRequest {
            method: "GET",
            url_path: "/",
            headers: "bad",
            body: "bad",
        },
    );
    assert_eq!(result.outcome, TestRuleOutcome::Error);
    assert!(result.conditions.is_empty());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].scope, DiagnosticScope::Selection);
    assert_eq!(
        result.diagnostics[0].reason,
        EvaluationError::NoRuleSelected
    );
}

#[test]
fn precedence_is_error_then_unsupported_then_failed_then_passed() {
    let mut rule = RulePayload {
        method: "PATCH".into(),
        url_path: "/expected".into(),
        url_path_op: Some(UrlPathOp::Equal),
        ..RulePayload::default()
    };
    let unsupported = evaluate_rule(&rule, "GET", "/wrong", "", "");
    assert_eq!(unsupported.outcome, TestRuleOutcome::Unsupported);

    let error = evaluate_rule(&rule, "bad method", "/wrong", "", "");
    assert_eq!(error.outcome, TestRuleOutcome::Error);

    rule.method = "GET".into();
    let failed = evaluate_rule(&rule, "GET", "/wrong", "", "");
    assert_eq!(failed.outcome, TestRuleOutcome::NoMatch);

    let passed = evaluate_rule(&rule, "GET", "/expected", "", "");
    assert_eq!(passed.outcome, TestRuleOutcome::Matched);
}

#[test]
fn malformed_body_outranks_unsupported_body_regex() {
    let rule = RulePayload {
        body: vec![BodyConditionPayload {
            path: "name".into(),
            op: BodyOp::Regex,
            value: "^a".into(),
        }],
        ..RulePayload::default()
    };
    let result = evaluate_rule(&rule, "GET", "/", "", "not-json");
    assert_eq!(result.outcome, TestRuleOutcome::Error);
    assert!(matches!(
        result.conditions[1].outcome,
        ConditionOutcome::Unsupported { .. }
    ));
}

#[test]
fn unused_header_and_body_input_is_ignored() {
    let rule = RulePayload::default();
    let result = evaluate_rule(&rule, "GET", "/", "missing colon", "not-json");
    assert_eq!(result.outcome, TestRuleOutcome::Matched);
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.conditions.len(), 1);
}

#[test]
fn failed_sibling_cannot_hide_any_unsupported_operator_family() {
    let url = RulePayload {
        method: "GET".into(),
        url_path: "end".into(),
        url_path_op: Some(UrlPathOp::EndsWith),
        ..RulePayload::default()
    };
    assert_eq!(
        evaluate_rule(&url, "POST", "not-end", "", "").outcome,
        TestRuleOutcome::Unsupported
    );

    for operator in [
        HeaderOp::EndsWith,
        HeaderOp::Regex,
        HeaderOp::Exists,
        HeaderOp::Absent,
    ] {
        let rule = RulePayload {
            url_path: "/expected".into(),
            url_path_op: Some(UrlPathOp::Equal),
            headers: vec![HeaderConditionPayload {
                name: "x-test".into(),
                op: operator,
                value: "x".into(),
            }],
            ..RulePayload::default()
        };
        assert_eq!(
            evaluate_rule(&rule, "GET", "/failed", "x-test: x", "").outcome,
            TestRuleOutcome::Unsupported,
            "{operator:?} must outrank a failed URL sibling"
        );
    }

    let body = RulePayload {
        url_path: "/expected".into(),
        url_path_op: Some(UrlPathOp::Equal),
        body: vec![BodyConditionPayload {
            path: "name".into(),
            op: BodyOp::Regex,
            value: "^a".into(),
        }],
        ..RulePayload::default()
    };
    assert_eq!(
        evaluate_rule(&body, "GET", "/failed", "", r#"{"name":"a"}"#).outcome,
        TestRuleOutcome::Unsupported
    );
}

#[test]
fn global_diagnostics_and_condition_results_have_stable_order() {
    let rule = RulePayload {
        method: "bad method".into(),
        url_path: "/expected".into(),
        url_path_op: Some(UrlPathOp::Equal),
        headers: vec![HeaderConditionPayload {
            name: "x-test".into(),
            op: HeaderOp::Equal,
            value: "expected".into(),
        }],
        body: vec![BodyConditionPayload {
            path: "name".into(),
            op: BodyOp::Regex,
            value: "^a".into(),
        }],
        ..RulePayload::default()
    };
    let result = evaluate_rule(
        &rule,
        "also bad",
        "/failed",
        "missing colon\nalso missing",
        "not-json",
    );
    assert_eq!(result.outcome, TestRuleOutcome::Error);
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.scope.clone())
            .collect::<Vec<_>>(),
        vec![
            DiagnosticScope::RequestMethod,
            DiagnosticScope::RequestHeaderLine(1),
            DiagnosticScope::RequestHeaderLine(2),
            DiagnosticScope::RequestBody,
        ]
    );
    assert!(matches!(
        result.conditions.as_slice(),
        [
            super::super::ConditionResult {
                condition: super::super::ConditionIdentity::Method,
                ..
            },
            super::super::ConditionResult {
                condition: super::super::ConditionIdentity::UrlPath,
                ..
            },
            super::super::ConditionResult {
                condition: super::super::ConditionIdentity::Header { .. },
                ..
            },
            super::super::ConditionResult {
                condition: super::super::ConditionIdentity::Body { .. },
                ..
            }
        ]
    ));
    assert!(matches!(
        result.conditions[0].outcome,
        ConditionOutcome::Error {
            reason: EvaluationError::InvalidConfiguredMethod(_)
        }
    ));
    assert!(matches!(
        result.conditions[3].outcome,
        ConditionOutcome::Unsupported { .. }
    ));
}
