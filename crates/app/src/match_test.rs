//! Fail-closed Test Rule conformance adapter (RFC MK-052).

mod engine;
mod input;
mod result;

use apimokka_model::{BodyOp, RulePayload};
use engine::{body_operator, method_operator, rule_operator};
use input::{parse_headers, parse_request_method};
use result::aggregate;
pub use result::{
    ConditionIdentity, ConditionOutcome, ConditionResult, DiagnosticScope, EvaluationError,
    RequestDiagnostic, TestRuleOutcome, TestRuleResult, UnsupportedReason,
};

#[cfg(test)]
mod tests;

pub struct TestRequest<'a> {
    pub method: &'a str,
    pub url_path: &'a str,
    pub headers: &'a str,
    pub body: &'a str,
}

/// Evaluate one request against one selected rule without inventing unavailable
/// engine semantics.
pub fn evaluate(rule: Option<&RulePayload>, request: TestRequest<'_>) -> TestRuleResult {
    let Some(rule) = rule else {
        return TestRuleResult {
            outcome: TestRuleOutcome::Error,
            conditions: Vec::new(),
            diagnostics: vec![RequestDiagnostic {
                scope: DiagnosticScope::Selection,
                reason: EvaluationError::NoRuleSelected,
            }],
        };
    };

    let mut conditions = Vec::new();
    let mut diagnostics = Vec::new();

    let request_method = match parse_request_method(request.method) {
        Ok(method) => Some(method),
        Err(reason) => {
            diagnostics.push(RequestDiagnostic {
                scope: DiagnosticScope::RequestMethod,
                reason,
            });
            None
        }
    };
    if let Some(result) = evaluate_method(&rule.method, request_method.as_ref()) {
        conditions.push(result);
    }

    if let Some(op) = rule.url_path_op {
        let outcome = match rule_operator(op) {
            Ok(engine_op) => pass_fail(engine_op.is_match(request.url_path, &rule.url_path)),
            Err(reason) => ConditionOutcome::Unsupported { reason },
        };
        conditions.push(ConditionResult {
            condition: ConditionIdentity::UrlPath,
            outcome,
        });
    }

    let parsed_headers = if rule.headers.is_empty() {
        input::ParsedHeaderValues::default()
    } else {
        let parsed = parse_headers(request.headers);
        diagnostics.extend(parsed.diagnostics);
        parsed.values
    };

    for (index, condition) in rule.headers.iter().enumerate() {
        let identity = ConditionIdentity::Header {
            index,
            name: condition.name.clone(),
        };
        let outcome = evaluate_header(condition, &parsed_headers);
        conditions.push(ConditionResult {
            condition: identity,
            outcome,
        });
    }

    let body = if rule.body.is_empty() {
        None
    } else {
        match serde_json::from_str::<serde_json::Value>(request.body) {
            Ok(value) => Some(value),
            Err(_) => {
                diagnostics.push(RequestDiagnostic {
                    scope: DiagnosticScope::RequestBody,
                    reason: EvaluationError::InvalidRequestBody,
                });
                None
            }
        }
    };

    for (index, condition) in rule.body.iter().enumerate() {
        let identity = ConditionIdentity::Body {
            index,
            path: condition.path.clone(),
        };
        if let Some(outcome) = evaluate_body(condition, body.as_ref()) {
            conditions.push(ConditionResult {
                condition: identity,
                outcome,
            });
        }
    }

    let outcome = aggregate(&conditions, &diagnostics);
    TestRuleResult {
        outcome,
        conditions,
        diagnostics,
    }
}

pub fn unsupported_conditions(rule: Option<&RulePayload>) -> Vec<ConditionResult> {
    let Some(rule) = rule else {
        return Vec::new();
    };
    let mut results = Vec::new();
    if !rule.method.is_empty()
        && method_operator(&rule.method).is_ok_and(|operator| operator.is_none())
    {
        results.push(ConditionResult {
            condition: ConditionIdentity::Method,
            outcome: ConditionOutcome::Unsupported {
                reason: UnsupportedReason::ConfiguredMethod(rule.method.clone()),
            },
        });
    }
    if let Some(op) = rule.url_path_op
        && let Err(reason) = rule_operator(op)
    {
        results.push(ConditionResult {
            condition: ConditionIdentity::UrlPath,
            outcome: ConditionOutcome::Unsupported { reason },
        });
    }
    for (index, header) in rule.headers.iter().enumerate() {
        if let Err(reason) = rule_operator(header.op) {
            results.push(ConditionResult {
                condition: ConditionIdentity::Header {
                    index,
                    name: header.name.clone(),
                },
                outcome: ConditionOutcome::Unsupported { reason },
            });
        }
    }
    for (index, body) in rule.body.iter().enumerate() {
        if let Err(reason) = body_operator(body.op) {
            results.push(ConditionResult {
                condition: ConditionIdentity::Body {
                    index,
                    path: body.path.clone(),
                },
                outcome: ConditionOutcome::Unsupported { reason },
            });
        }
    }
    results
}

fn evaluate_method(configured: &str, request: Option<&http::Method>) -> Option<ConditionResult> {
    let outcome = match method_operator(configured) {
        Err(reason) => Some(ConditionOutcome::Error { reason }),
        Ok(None) if configured.is_empty() => Some(ConditionOutcome::Passed),
        Ok(None) => Some(ConditionOutcome::Unsupported {
            reason: UnsupportedReason::ConfiguredMethod(configured.to_owned()),
        }),
        Ok(Some(operator)) => request.map(|request| pass_fail(operator.is_match(request))),
    };
    outcome.map(|outcome| ConditionResult {
        condition: ConditionIdentity::Method,
        outcome,
    })
}

fn evaluate_header(
    condition: &apimokka_model::HeaderConditionPayload,
    headers: &input::ParsedHeaderValues,
) -> ConditionOutcome {
    let name = match http::HeaderName::from_bytes(condition.name.as_bytes()) {
        Ok(name) => name,
        Err(_) => {
            return ConditionOutcome::Error {
                reason: EvaluationError::InvalidConfiguredHeaderName(condition.name.clone()),
            };
        }
    };
    let operator = match rule_operator(condition.op) {
        Ok(operator) => operator,
        Err(reason) => return ConditionOutcome::Unsupported { reason },
    };
    let Some(actual) = headers.get(&name) else {
        return ConditionOutcome::Failed;
    };
    pass_fail(operator.is_match(actual, &condition.value))
}

fn evaluate_body(
    condition: &apimokka_model::BodyConditionPayload,
    body: Option<&serde_json::Value>,
) -> Option<ConditionOutcome> {
    if let Err(reason) = validate_body_value(condition) {
        return Some(ConditionOutcome::Error { reason });
    }
    let operator = match body_operator(condition.op) {
        Ok(operator) => operator,
        Err(reason) => return Some(ConditionOutcome::Unsupported { reason }),
    };
    let body = body?;
    let resolved = apimock_routing::util::json::json_value_by_jsonpath(body, &condition.path);
    Some(match (condition.op, resolved) {
        (BodyOp::Absent, None) => ConditionOutcome::Passed,
        (BodyOp::Exists, None) => ConditionOutcome::Failed,
        (_, None) => ConditionOutcome::Failed,
        (_, Some(value)) => pass_fail(operator.is_match(value, &condition.value)),
    })
}

fn validate_body_value(
    condition: &apimokka_model::BodyConditionPayload,
) -> Result<(), EvaluationError> {
    match condition.op {
        BodyOp::EqualTyped | BodyOp::ArrayContains => {
            serde_json::from_str::<serde_json::Value>(&condition.value)
                .map(|_| ())
                .map_err(|_| EvaluationError::InvalidConfiguredJson)
        }
        BodyOp::EqualNumber
        | BodyOp::GreaterThan
        | BodyOp::LessThan
        | BodyOp::GreaterOrEqual
        | BodyOp::LessOrEqual => condition
            .value
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .map(|_| ())
            .ok_or(EvaluationError::InvalidConfiguredNumber),
        BodyOp::EqualInteger => condition
            .value
            .parse::<i64>()
            .map(|_| ())
            .map_err(|_| EvaluationError::InvalidConfiguredInteger),
        BodyOp::ArrayLengthEqual | BodyOp::ArrayLengthAtLeast => condition
            .value
            .parse::<usize>()
            .map(|_| ())
            .map_err(|_| EvaluationError::InvalidConfiguredLength),
        _ => Ok(()),
    }
}

fn pass_fail(matched: bool) -> ConditionOutcome {
    if matched {
        ConditionOutcome::Passed
    } else {
        ConditionOutcome::Failed
    }
}
