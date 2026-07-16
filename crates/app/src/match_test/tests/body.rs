use apimock_routing::rule_set::rule::when::request::body::body_operator::BodyOperator;
use apimokka_model::{BodyConditionPayload, BodyOp, RulePayload};

use super::super::TestRuleOutcome;
use super::evaluate_rule;

#[test]
fn every_body_operator_has_positive_negative_or_unsupported_cases() {
    let cases = [
        (BodyOp::Equal, "42", r#"{"v":42}"#, r#"{"v":41}"#),
        (BodyOp::EqualString, "42", r#"{"v":"42"}"#, r#"{"v":"41"}"#),
        (
            BodyOp::Contains,
            "mid",
            r#"{"v":"a-mid-z"}"#,
            r#"{"v":"other"}"#,
        ),
        (
            BodyOp::StartsWith,
            "pre",
            r#"{"v":"prefix"}"#,
            r#"{"v":"other"}"#,
        ),
        (
            BodyOp::EndsWith,
            "end",
            r#"{"v":"the-end"}"#,
            r#"{"v":"other"}"#,
        ),
        (BodyOp::EqualTyped, "42", r#"{"v":42}"#, r#"{"v":"42"}"#),
        (
            BodyOp::ArrayContains,
            "2",
            r#"{"v":[1,2]}"#,
            r#"{"v":[1,3]}"#,
        ),
        (BodyOp::EqualNumber, "42", r#"{"v":"42"}"#, r#"{"v":41}"#),
        (BodyOp::GreaterThan, "42", r#"{"v":43}"#, r#"{"v":42}"#),
        (BodyOp::LessThan, "42", r#"{"v":41}"#, r#"{"v":42}"#),
        (BodyOp::GreaterOrEqual, "42", r#"{"v":42}"#, r#"{"v":41}"#),
        (BodyOp::LessOrEqual, "42", r#"{"v":42}"#, r#"{"v":43}"#),
        (
            BodyOp::EqualInteger,
            "9007199254740993",
            r#"{"v":9007199254740993}"#,
            r#"{"v":9007199254740992}"#,
        ),
        (
            BodyOp::ArrayLengthEqual,
            "2",
            r#"{"v":[1,2]}"#,
            r#"{"v":[1]}"#,
        ),
        (
            BodyOp::ArrayLengthAtLeast,
            "2",
            r#"{"v":[1,2]}"#,
            r#"{"v":[1]}"#,
        ),
        (BodyOp::Exists, "", r#"{"v":null}"#, r#"{}"#),
        (BodyOp::Absent, "", r#"{}"#, r#"{"v":null}"#),
    ];
    for (operator, configured, passing, failing) in cases {
        assert_engine_oracle(operator, configured, passing, failing);
        let rule = body_rule(operator, configured);
        assert_eq!(
            evaluate_rule(&rule, "GET", "/", "", passing).outcome,
            TestRuleOutcome::Matched,
            "{operator:?} positive"
        );
        assert_eq!(
            evaluate_rule(&rule, "GET", "/", "", failing).outcome,
            TestRuleOutcome::NoMatch,
            "{operator:?} negative"
        );
    }
    let regex = body_rule(BodyOp::Regex, "^value$");
    assert_eq!(
        evaluate_rule(&regex, "GET", "/", "", r#"{"v":"value"}"#).outcome,
        TestRuleOutcome::Unsupported
    );
    assert_eq!(BodyOp::all().len(), 18);
}

#[test]
fn body_validation_boundaries_are_errors() {
    for (operator, configured) in [
        (BodyOp::EqualTyped, "not-json"),
        (BodyOp::ArrayContains, "not-json"),
        (BodyOp::EqualNumber, "NaN"),
        (BodyOp::GreaterThan, "inf"),
        (BodyOp::LessThan, "-inf"),
        (BodyOp::EqualInteger, "1.5"),
        (BodyOp::EqualInteger, "9223372036854775808"),
    ] {
        assert_eq!(
            evaluate_rule(
                &body_rule(operator, configured),
                "GET",
                "/",
                "",
                r#"{"v":1}"#
            )
            .outcome,
            TestRuleOutcome::Error,
            "{operator:?} {configured}"
        );
    }
    for configured in [i64::MIN.to_string(), i64::MAX.to_string()] {
        let body = format!(r#"{{"v":"{configured}"}}"#);
        assert_eq!(
            evaluate_rule(
                &body_rule(BodyOp::EqualInteger, &configured),
                "GET",
                "/",
                "",
                &body
            )
            .outcome,
            TestRuleOutcome::Matched
        );
    }

    assert_eq!(
        evaluate_rule(
            &body_rule(BodyOp::EqualInteger, "1"),
            "GET",
            "/",
            "",
            r#"{"v":1.5}"#
        )
        .outcome,
        TestRuleOutcome::NoMatch,
        "a fractional request value is not an i64"
    );
    assert_eq!(
        evaluate_rule(
            &body_rule(BodyOp::EqualInteger, "0"),
            "GET",
            "/",
            "",
            r#"{"v":9223372036854775808}"#
        )
        .outcome,
        TestRuleOutcome::NoMatch,
        "an out-of-range request value is not an i64"
    );
}

#[test]
fn numeric_operators_reject_non_numeric_resolved_values_as_verified_non_matches() {
    for operator in [
        BodyOp::EqualNumber,
        BodyOp::GreaterThan,
        BodyOp::LessThan,
        BodyOp::GreaterOrEqual,
        BodyOp::LessOrEqual,
        BodyOp::EqualInteger,
    ] {
        assert_eq!(
            evaluate_rule(
                &body_rule(operator, "42"),
                "GET",
                "/",
                "",
                r#"{"v":"not-a-number"}"#,
            )
            .outcome,
            TestRuleOutcome::NoMatch,
            "{operator:?} must not coerce non-numeric text"
        );
    }
}

#[test]
fn both_array_length_operators_cover_all_configured_boundaries() {
    let max_length = usize::MAX.to_string();
    let out_of_range = format!("{max_length}0");
    for operator in [BodyOp::ArrayLengthEqual, BodyOp::ArrayLengthAtLeast] {
        assert_eq!(
            evaluate_rule(&body_rule(operator, "0"), "GET", "/", "", r#"{"v":[]}"#).outcome,
            TestRuleOutcome::Matched,
            "{operator:?} accepts zero"
        );
        assert_eq!(
            evaluate_rule(
                &body_rule(operator, &max_length),
                "GET",
                "/",
                "",
                r#"{"v":[]}"#
            )
            .outcome,
            TestRuleOutcome::NoMatch,
            "{operator:?} accepts usize::MAX configuration"
        );
        for configured in ["not-a-length", "-1", "1.5", " 1", &out_of_range] {
            assert_eq!(
                evaluate_rule(
                    &body_rule(operator, configured),
                    "GET",
                    "/",
                    "",
                    r#"{"v":[]}"#
                )
                .outcome,
                TestRuleOutcome::Error,
                "{operator:?} rejects {configured:?}"
            );
        }
    }
}

#[test]
fn empty_required_body_is_a_request_error_without_derivative_condition_error() {
    let result = evaluate_rule(&body_rule(BodyOp::EqualString, "value"), "GET", "/", "", "");
    assert_eq!(result.outcome, TestRuleOutcome::Error);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].scope,
        super::super::DiagnosticScope::RequestBody
    );
    assert_eq!(
        result.conditions.len(),
        1,
        "only the independent Any-method result remains"
    );
}

#[test]
fn engine_dotted_path_handles_objects_and_arrays() {
    let rule = RulePayload {
        body: vec![BodyConditionPayload {
            path: "items.1.name".into(),
            op: BodyOp::EqualString,
            value: "second".into(),
        }],
        ..RulePayload::default()
    };
    assert_eq!(
        evaluate_rule(
            &rule,
            "GET",
            "/",
            "",
            r#"{"items":[{"name":"first"},{"name":"second"}]}"#
        )
        .outcome,
        TestRuleOutcome::Matched
    );
}

fn assert_engine_oracle(operator: BodyOp, configured: &str, passing: &str, failing: &str) {
    let passing_json: serde_json::Value = serde_json::from_str(passing).unwrap();
    let failing_json: serde_json::Value = serde_json::from_str(failing).unwrap();
    let passing_value = apimock_routing::util::json::json_value_by_jsonpath(&passing_json, "v");
    let failing_value = apimock_routing::util::json::json_value_by_jsonpath(&failing_json, "v");
    match operator {
        BodyOp::Absent => {
            assert!(passing_value.is_none());
            assert!(failing_value.is_some());
        }
        BodyOp::Exists => {
            assert!(passing_value.is_some());
            assert!(failing_value.is_none());
        }
        _ => {
            let oracle = oracle_body_operator(operator);
            assert!(oracle.is_match(passing_value.unwrap(), configured));
            assert!(!oracle.is_match(failing_value.unwrap(), configured));
        }
    }
}

fn body_rule(operator: BodyOp, value: &str) -> RulePayload {
    RulePayload {
        body: vec![BodyConditionPayload {
            path: "v".into(),
            op: operator,
            value: value.into(),
        }],
        ..RulePayload::default()
    }
}

fn oracle_body_operator(operator: BodyOp) -> BodyOperator {
    match operator {
        BodyOp::Equal => BodyOperator::Equal,
        BodyOp::EqualString => BodyOperator::EqualString,
        BodyOp::Contains => BodyOperator::Contains,
        BodyOp::StartsWith => BodyOperator::StartsWith,
        BodyOp::EndsWith => BodyOperator::EndsWith,
        BodyOp::Regex => BodyOperator::Regex,
        BodyOp::EqualTyped => BodyOperator::EqualTyped,
        BodyOp::ArrayContains => BodyOperator::ArrayContains,
        BodyOp::EqualNumber => BodyOperator::EqualNumber,
        BodyOp::GreaterThan => BodyOperator::GreaterThan,
        BodyOp::LessThan => BodyOperator::LessThan,
        BodyOp::GreaterOrEqual => BodyOperator::GreaterOrEqual,
        BodyOp::LessOrEqual => BodyOperator::LessOrEqual,
        BodyOp::EqualInteger => BodyOperator::EqualInteger,
        BodyOp::ArrayLengthEqual => BodyOperator::ArrayLengthEqual,
        BodyOp::ArrayLengthAtLeast => BodyOperator::ArrayLengthAtLeast,
        BodyOp::Exists => BodyOperator::Exists,
        BodyOp::Absent => BodyOperator::Absent,
    }
}
