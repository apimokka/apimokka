use apimokka_i18n::Locale;
use apimokka_model::AudienceMode;

use crate::app::App;
use crate::match_test::{
    ConditionIdentity, ConditionOutcome, EvaluationError, TestRuleOutcome, TestRuleResult,
    UnsupportedReason, unsupported_conditions,
};
use crate::message::Message;

fn expert_app() -> App {
    let mut app = App::new().0;
    app.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    app.update(Message::OpenWorkspace("test".into()));
    let first_rule = app.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    app.update(Message::SelectRule(first_rule));
    app.update(Message::TestRuleOpen);
    app
}

fn expert_app_with_payload(payload: apimokka_model::RulePayload) -> App {
    let mut app = App::new().0;
    app.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    let mut seed = apimokka_model::mock::shop_api_canonical_seed();
    seed.rule_sets[0].rules[0].payload = payload;
    assert!(app.install_workspace(seed));
    let first_rule = app.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    app.update(Message::SelectRule(first_rule));
    app.update(Message::TestRuleOpen);
    app
}

#[test]
fn dialog_renders_all_outcomes_in_both_locales() {
    for locale in [Locale::En, Locale::Ja] {
        for outcome in [
            TestRuleOutcome::Matched,
            TestRuleOutcome::NoMatch,
            TestRuleOutcome::Unsupported,
            TestRuleOutcome::Error,
        ] {
            let mut app = expert_app();
            app.update(Message::ChangeLocale(locale));
            app.test_rule.result = Some(TestRuleResult {
                outcome,
                diagnostics: Vec::new(),
                conditions: vec![crate::match_test::ConditionResult {
                    condition: ConditionIdentity::UrlPath,
                    outcome: match outcome {
                        TestRuleOutcome::Matched => ConditionOutcome::Passed,
                        TestRuleOutcome::NoMatch => ConditionOutcome::Failed,
                        TestRuleOutcome::Unsupported => ConditionOutcome::Unsupported {
                            reason: UnsupportedReason::UrlOperator(
                                apimokka_model::UrlPathOp::EndsWith,
                            ),
                        },
                        TestRuleOutcome::Error => ConditionOutcome::Error {
                            reason: EvaluationError::InvalidConfiguredJson,
                        },
                    },
                }],
            });
            let lines = crate::screens::test_rule::result_lines(
                &app,
                app.test_rule.result.as_ref().unwrap(),
            );
            assert_eq!(lines.len(), 1);
            let (expected_title, expected_detail) = match (locale, outcome) {
                (Locale::En, TestRuleOutcome::Matched) => ("✓ Matched", "passed"),
                (Locale::En, TestRuleOutcome::NoMatch) => ("◯ No match", "failed"),
                (Locale::En, TestRuleOutcome::Unsupported) => (
                    "? Unable to verify",
                    "unsupported — operator is unsupported: Path EndsWith",
                ),
                (Locale::En, TestRuleOutcome::Error) => {
                    ("! Error", "error — configured value is invalid")
                }
                (Locale::Ja, TestRuleOutcome::Matched) => ("✓ マッチ", "成功"),
                (Locale::Ja, TestRuleOutcome::NoMatch) => ("◯ マッチなし", "失敗"),
                (Locale::Ja, TestRuleOutcome::Unsupported) => (
                    "? 検証できません",
                    "未対応 — 演算子は未対応です: パス EndsWith",
                ),
                (Locale::Ja, TestRuleOutcome::Error) => ("! エラー", "エラー — 設定値が無効です"),
            };
            assert_eq!(
                crate::screens::test_rule::result_title(&app, outcome),
                expected_title
            );
            assert!(lines[0].contains(expected_detail), "{:?}", lines[0]);
            let _ = crate::screens::test_rule::view(&app);
        }
    }
}

#[test]
fn unsupported_operator_families_are_localized_in_both_locales() {
    for locale in [Locale::En, Locale::Ja] {
        let mut app = expert_app();
        app.update(Message::ChangeLocale(locale));
        let result = TestRuleResult {
            outcome: TestRuleOutcome::Unsupported,
            diagnostics: Vec::new(),
            conditions: vec![
                crate::match_test::ConditionResult {
                    condition: ConditionIdentity::UrlPath,
                    outcome: ConditionOutcome::Unsupported {
                        reason: UnsupportedReason::UrlOperator(apimokka_model::UrlPathOp::EndsWith),
                    },
                },
                crate::match_test::ConditionResult {
                    condition: ConditionIdentity::Header {
                        index: 0,
                        name: "x-test".into(),
                    },
                    outcome: ConditionOutcome::Unsupported {
                        reason: UnsupportedReason::HeaderOperator(apimokka_model::HeaderOp::Regex),
                    },
                },
                crate::match_test::ConditionResult {
                    condition: ConditionIdentity::Body {
                        index: 0,
                        path: "name".into(),
                    },
                    outcome: ConditionOutcome::Unsupported {
                        reason: UnsupportedReason::BodyOperator(apimokka_model::BodyOp::Regex),
                    },
                },
            ],
        };
        let lines = crate::screens::test_rule::result_lines(&app, &result);
        let expected = match locale {
            Locale::En => [
                "operator is unsupported: Path EndsWith",
                "operator is unsupported: Headers (name: value, one per line) Regex",
                "operator is unsupported: Body (JSON) Regex",
            ],
            Locale::Ja => [
                "演算子は未対応です: パス EndsWith",
                "演算子は未対応です: ヘッダー（name: value、1行1件） Regex",
                "演算子は未対応です: ボディ（JSON） Regex",
            ],
        };
        for (line, fragment) in lines.iter().zip(expected) {
            assert!(line.contains(fragment), "{line:?}");
        }
        if locale == Locale::Ja {
            assert!(lines.iter().all(|line| !line.contains(": header ")));
            assert!(lines.iter().all(|line| !line.contains(": body ")));
        }
    }
}

#[test]
fn escape_closes_test_rule_dialog() {
    let mut app = expert_app();
    assert!(app.test_rule.open);
    app.update(Message::EscapePressed);
    assert!(!app.test_rule.open);
}

#[test]
fn every_request_input_edit_invalidates_a_stored_result() {
    let edits = [
        Message::TestRuleSetMethod("POST".into()),
        Message::TestRuleSetPath("/changed".into()),
        Message::TestRuleSetHeaders("x-test: changed".into()),
        Message::TestRuleSetBody(r#"{"changed":true}"#.into()),
    ];
    for edit in edits {
        let mut app = expert_app();
        app.update(Message::TestRuleRun);
        assert!(app.test_rule.result.is_some(), "Run must store a result");
        app.update(edit);
        assert!(
            app.test_rule.result.is_none(),
            "editing visible request input must invalidate its result"
        );
        let _ = crate::screens::test_rule::view(&app);
    }
}

#[test]
fn pre_run_limitation_and_reducer_result_are_disclosed() {
    let mut app = expert_app_with_payload(apimokka_model::RulePayload {
        url_path: "end".into(),
        url_path_op: Some(apimokka_model::UrlPathOp::EndsWith),
        ..apimokka_model::RulePayload::default()
    });
    app.test_rule.method = "GET".into();
    app.test_rule.url_path = "the-end".into();
    app.test_rule.result = None;

    let limitations = unsupported_conditions(app.selected_rule().map(|rule| &rule.payload));
    assert_eq!(limitations.len(), 1);
    assert!(app.test_rule.result.is_none());
    let _ = crate::screens::test_rule::view(&app);

    app.update(Message::TestRuleRun);
    let result = app.test_rule.result.as_ref().unwrap();
    assert_eq!(result.outcome, TestRuleOutcome::Unsupported);
    let lines = crate::screens::test_rule::result_lines(&app, result);
    assert_eq!(lines.len(), 2, "method then unavailable URL condition");
    assert!(lines[1].contains("Path"));
    assert!(lines[1].contains("unsupported"));
}

#[test]
fn reducer_renders_multiple_issues_in_diagnostic_then_condition_order() {
    let mut app = expert_app_with_payload(apimokka_model::RulePayload {
        method: "GET".into(),
        url_path: "/expected".into(),
        url_path_op: Some(apimokka_model::UrlPathOp::Equal),
        headers: vec![apimokka_model::HeaderConditionPayload {
            name: "x-test".into(),
            op: apimokka_model::HeaderOp::Equal,
            value: "expected".into(),
        }],
        body: vec![apimokka_model::BodyConditionPayload {
            path: "name".into(),
            op: apimokka_model::BodyOp::Regex,
            value: "^a".into(),
        }],
        ..apimokka_model::RulePayload::default()
    });
    app.update(Message::TestRuleSetMethod("also bad".into()));
    app.update(Message::TestRuleSetPath("/failed".into()));
    app.update(Message::TestRuleSetHeaders(
        "missing colon\nalso missing".into(),
    ));
    app.update(Message::TestRuleSetBody("not-json".into()));
    app.update(Message::TestRuleRun);

    let result = app.test_rule.result.as_ref().unwrap();
    assert_eq!(result.outcome, TestRuleOutcome::Error);
    let lines = crate::screens::test_rule::result_lines(&app, result);
    assert_eq!(lines.len(), 7);
    for (line, expected) in lines.iter().zip([
        "Request method",
        "Header line 1",
        "Header line 2",
        "Request body",
        "Path",
        "Headers",
        "Body",
    ]) {
        assert!(
            line.starts_with(expected),
            "{line:?} must start with {expected:?}"
        );
    }
    assert!(lines[4].contains("failed"));
    assert!(lines[6].contains("unsupported"));
    let _ = crate::screens::test_rule::view(&app);
}
