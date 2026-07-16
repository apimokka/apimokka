use apimock_routing::rule_set::rule::when::request::rule_op::RuleOp as EngineRuleOp;
use apimokka_model::{HeaderConditionPayload, HeaderOp, RulePayload, UrlPathOp};

use super::super::{ConditionOutcome, TestRuleOutcome, UnsupportedReason};
use super::evaluate_rule;

#[test]
fn configured_method_request_method_matrix() {
    for configured in ["GET", "POST", "PUT", "DELETE"] {
        let rule = RulePayload {
            method: configured.into(),
            ..RulePayload::default()
        };
        assert_eq!(
            evaluate_rule(&rule, configured, "/", "", "").outcome,
            TestRuleOutcome::Matched
        );
        for request in ["PATCH", "PURGE"] {
            assert_eq!(
                evaluate_rule(&rule, request, "/", "", "").outcome,
                TestRuleOutcome::NoMatch
            );
        }
    }

    let any = RulePayload::default();
    for request in ["GET", "PATCH", "HEAD", "PURGE"] {
        assert_eq!(
            evaluate_rule(&any, request, "/", "", "").outcome,
            TestRuleOutcome::Matched
        );
    }

    let get = RulePayload {
        method: "GET".into(),
        ..RulePayload::default()
    };
    assert_eq!(
        evaluate_rule(&get, "get", "/", "", "").outcome,
        TestRuleOutcome::Matched
    );

    for configured in ["PATCH", "HEAD", "PURGE"] {
        let rule = RulePayload {
            method: configured.into(),
            ..RulePayload::default()
        };
        assert_eq!(
            evaluate_rule(&rule, "GET", "/", "", "").outcome,
            TestRuleOutcome::Unsupported
        );
    }

    let malformed = RulePayload {
        method: "bad method".into(),
        ..RulePayload::default()
    };
    assert_eq!(
        evaluate_rule(&malformed, "GET", "/", "", "").outcome,
        TestRuleOutcome::Error
    );
    let both_malformed = evaluate_rule(&malformed, "also bad", "/", "", "");
    assert_eq!(both_malformed.outcome, TestRuleOutcome::Error);
    assert_eq!(both_malformed.diagnostics.len(), 1);
    assert_eq!(both_malformed.conditions.len(), 1);
    assert!(matches!(
        both_malformed.conditions[0].outcome,
        ConditionOutcome::Error { .. }
    ));
}

#[test]
fn every_url_operator_has_positive_negative_or_unsupported_cases() {
    let cases = [
        (UrlPathOp::Equal, "/a", "/a", "/b", EngineRuleOp::Equal),
        (
            UrlPathOp::StartsWith,
            "/a",
            "/abc",
            "/b",
            EngineRuleOp::StartsWith,
        ),
        (
            UrlPathOp::Contains,
            "mid",
            "/mid/path",
            "/other",
            EngineRuleOp::Contains,
        ),
        (
            UrlPathOp::WildCard,
            "/a/*/?",
            "/a//x",
            "/a/x/xy",
            EngineRuleOp::WildCard,
        ),
        (
            UrlPathOp::NotEqual,
            "/a",
            "/b",
            "/a",
            EngineRuleOp::NotEqual,
        ),
    ];
    for (operator, configured, passing, failing, oracle) in cases {
        assert!(oracle.is_match(passing, configured));
        assert!(!oracle.is_match(failing, configured));
        let rule = RulePayload {
            url_path: configured.into(),
            url_path_op: Some(operator),
            ..RulePayload::default()
        };
        assert_eq!(
            evaluate_rule(&rule, "GET", passing, "", "").outcome,
            TestRuleOutcome::Matched,
            "{operator:?} positive"
        );
        assert_eq!(
            evaluate_rule(&rule, "GET", failing, "", "").outcome,
            TestRuleOutcome::NoMatch,
            "{operator:?} negative"
        );
    }
    let unsupported = RulePayload {
        url_path: "end".into(),
        url_path_op: Some(UrlPathOp::EndsWith),
        ..RulePayload::default()
    };
    assert_eq!(
        evaluate_rule(&unsupported, "GET", "end", "", "").outcome,
        TestRuleOutcome::Unsupported
    );
    assert_eq!(UrlPathOp::all().len(), 6);
}

#[test]
fn wildcard_uses_engine_zero_many_one_and_unicode_semantics() {
    for (pattern, actual, expected) in [
        ("a*b", "ab", TestRuleOutcome::Matched),
        ("a**b", "axyzb", TestRuleOutcome::Matched),
        ("?", "界", TestRuleOutcome::Matched),
        ("?", "界a", TestRuleOutcome::NoMatch),
    ] {
        let rule = RulePayload {
            url_path: pattern.into(),
            url_path_op: Some(UrlPathOp::WildCard),
            ..RulePayload::default()
        };
        assert_eq!(
            evaluate_rule(&rule, "GET", actual, "", "").outcome,
            expected
        );
    }
}

#[test]
fn every_header_operator_has_positive_negative_or_unsupported_cases() {
    let supported = [
        (
            HeaderOp::Equal,
            "token",
            "token",
            "other",
            EngineRuleOp::Equal,
        ),
        (
            HeaderOp::Contains,
            "mid",
            "a-mid-z",
            "other",
            EngineRuleOp::Contains,
        ),
        (
            HeaderOp::StartsWith,
            "pre",
            "prefix",
            "other",
            EngineRuleOp::StartsWith,
        ),
        (
            HeaderOp::NotEqual,
            "token",
            "other",
            "token",
            EngineRuleOp::NotEqual,
        ),
        (
            HeaderOp::WildCard,
            "a*?",
            "abc",
            "a",
            EngineRuleOp::WildCard,
        ),
    ];
    for (operator, configured, passing, failing, oracle) in supported {
        assert!(oracle.is_match(passing, configured));
        assert!(!oracle.is_match(failing, configured));
        let rule = header_rule(operator, configured);
        assert_eq!(
            evaluate_rule(&rule, "GET", "/", &format!("X-Test: {passing}"), "").outcome,
            TestRuleOutcome::Matched
        );
        assert_eq!(
            evaluate_rule(&rule, "GET", "/", &format!("x-test: {failing}"), "").outcome,
            TestRuleOutcome::NoMatch
        );
        assert_eq!(
            evaluate_rule(&rule, "GET", "/", "", "").outcome,
            TestRuleOutcome::NoMatch,
            "missing header must fail even for NotEqual"
        );
    }
    let case_sensitive = header_rule(HeaderOp::Equal, "Token");
    assert!(!EngineRuleOp::Equal.is_match("token", "Token"));
    assert_eq!(
        evaluate_rule(&case_sensitive, "GET", "/", "x-test: token", "").outcome,
        TestRuleOutcome::NoMatch,
        "header values remain case-sensitive"
    );
    for operator in [
        HeaderOp::EndsWith,
        HeaderOp::Regex,
        HeaderOp::Exists,
        HeaderOp::Absent,
    ] {
        let result = evaluate_rule(&header_rule(operator, "x"), "GET", "/", "x-test: x", "");
        assert_eq!(result.outcome, TestRuleOutcome::Unsupported);
        assert!(matches!(
            result.conditions[1].outcome,
            ConditionOutcome::Unsupported {
                reason: UnsupportedReason::HeaderOperator(found)
            } if found == operator
        ));
    }
    assert_eq!(HeaderOp::all().len(), 9);
}

fn header_rule(operator: HeaderOp, value: &str) -> RulePayload {
    RulePayload {
        headers: vec![HeaderConditionPayload {
            name: "x-test".into(),
            op: operator,
            value: value.into(),
        }],
        ..RulePayload::default()
    }
}
