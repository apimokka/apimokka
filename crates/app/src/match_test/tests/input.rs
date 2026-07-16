use apimokka_model::{HeaderConditionPayload, HeaderOp, RulePayload};

use super::super::{DiagnosticScope, EvaluationError, TestRuleOutcome};
use super::evaluate_rule;

fn header_rule() -> RulePayload {
    RulePayload {
        headers: vec![HeaderConditionPayload {
            name: "x-test".into(),
            op: HeaderOp::Equal,
            value: "a:b".into(),
        }],
        ..RulePayload::default()
    }
}

#[test]
fn headers_ignore_blank_lines_trim_ows_and_split_first_colon() {
    let result = evaluate_rule(&header_rule(), "GET", "/", "\n X-Test \t: a:b \t\n", "");
    assert_eq!(result.outcome, TestRuleOutcome::Matched);
}

#[test]
fn header_errors_keep_original_line_numbers_and_order() {
    let result = evaluate_rule(
        &header_rule(),
        "GET",
        "/",
        "\nmissing-colon\nx-test: a:b\nX-TEST: second",
        "",
    );
    assert_eq!(result.outcome, TestRuleOutcome::Error);
    assert_eq!(result.diagnostics.len(), 2);
    assert_eq!(
        result.diagnostics[0].scope,
        DiagnosticScope::RequestHeaderLine(2)
    );
    assert_eq!(
        result.diagnostics[0].reason,
        EvaluationError::MissingHeaderColon
    );
    assert_eq!(
        result.diagnostics[1].scope,
        DiagnosticScope::RequestHeaderLine(4)
    );
    assert_eq!(
        result.diagnostics[1].reason,
        EvaluationError::DuplicateHeader {
            name: "x-test".into(),
            first_line: 3,
        }
    );
}

#[test]
fn empty_header_value_is_valid() {
    let mut rule = header_rule();
    rule.headers[0].value.clear();
    let result = evaluate_rule(&rule, "GET", "/", "x-test:", "");
    assert_eq!(result.outcome, TestRuleOutcome::Matched);
}

#[test]
fn invalid_header_name_and_value_are_errors() {
    let name = evaluate_rule(&header_rule(), "GET", "/", "bad name: value", "");
    assert!(matches!(
        name.diagnostics[0].reason,
        EvaluationError::InvalidHeaderName(_)
    ));

    let value = evaluate_rule(&header_rule(), "GET", "/", "x-test: bad\u{7f}", "");
    assert_eq!(
        value.diagnostics[0].reason,
        EvaluationError::InvalidHeaderValue
    );
}

#[test]
fn accepted_non_text_header_value_and_invalid_configured_name_are_errors() {
    let value = http::HeaderValue::from_bytes("é".as_bytes())
        .expect("http accepts obs-text bytes in a header value");
    assert!(value.to_str().is_err());

    let request = evaluate_rule(&header_rule(), "GET", "/", "x-test: é", "");
    assert_eq!(request.outcome, TestRuleOutcome::Error);
    assert_eq!(
        request.diagnostics[0].reason,
        EvaluationError::HeaderValueNotText
    );

    let mut rule = header_rule();
    rule.headers[0].name = "bad name".into();
    let configured = evaluate_rule(&rule, "GET", "/", "x-test: a:b", "");
    assert_eq!(configured.outcome, TestRuleOutcome::Error);
    assert!(matches!(
        configured.conditions[1].outcome,
        super::super::ConditionOutcome::Error {
            reason: EvaluationError::InvalidConfiguredHeaderName(ref name)
        } if name == "bad name"
    ));
}

#[test]
fn malformed_request_method_is_a_global_error() {
    let result = evaluate_rule(&RulePayload::default(), "bad method", "/", "", "");
    assert_eq!(result.outcome, TestRuleOutcome::Error);
    assert_eq!(result.diagnostics[0].scope, DiagnosticScope::RequestMethod);
    assert!(matches!(
        result.diagnostics[0].reason,
        EvaluationError::InvalidRequestMethod(_)
    ));
}

#[test]
fn malformed_request_method_omits_supported_dependent_method_condition() {
    let rule = RulePayload {
        method: "GET".into(),
        ..RulePayload::default()
    };
    let result = evaluate_rule(&rule, "bad method", "/", "", "");
    assert_eq!(result.outcome, TestRuleOutcome::Error);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].scope, DiagnosticScope::RequestMethod);
    assert!(result.conditions.is_empty());
}

#[test]
fn unsupported_header_still_demands_and_validates_complete_request_input() {
    let rule = RulePayload {
        headers: vec![HeaderConditionPayload {
            name: "x-test".into(),
            op: HeaderOp::Regex,
            value: "^value$".into(),
        }],
        ..RulePayload::default()
    };
    for headers in ["missing colon", "x-test: first\nX-TEST: duplicate"] {
        let result = evaluate_rule(&rule, "GET", "/", headers, "");
        assert_eq!(result.outcome, TestRuleOutcome::Error);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(matches!(
            result.conditions[1].outcome,
            super::super::ConditionOutcome::Unsupported { .. }
        ));
    }
}
