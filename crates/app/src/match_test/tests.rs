mod aggregation;
mod body;
mod input;
mod matrix;
mod screen;

use apimokka_model::RulePayload;

use super::{TestRequest, TestRuleResult, evaluate};

fn evaluate_rule<'a>(
    rule: &'a RulePayload,
    method: &'a str,
    path: &'a str,
    headers: &'a str,
    body: &'a str,
) -> TestRuleResult {
    evaluate(
        Some(rule),
        TestRequest {
            method,
            url_path: path,
            headers,
            body,
        },
    )
}
