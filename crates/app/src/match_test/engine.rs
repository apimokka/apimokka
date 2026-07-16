use apimock_routing::rule_set::rule::when::request::body::body_operator::BodyOperator;
use apimock_routing::rule_set::rule::when::request::http_method::HttpMethod;
use apimock_routing::rule_set::rule::when::request::rule_op::RuleOp;
use apimokka_model::{BodyOp, HeaderOp, UrlPathOp};

use super::{EvaluationError, UnsupportedReason};

pub(super) fn method_operator(configured: &str) -> Result<Option<HttpMethod>, EvaluationError> {
    if configured.is_empty() {
        return Ok(None);
    }
    http::Method::from_bytes(configured.as_bytes())
        .map_err(|_| EvaluationError::InvalidConfiguredMethod(configured.to_owned()))?;
    let method = if configured.eq_ignore_ascii_case("GET") {
        Some(HttpMethod::Get)
    } else if configured.eq_ignore_ascii_case("POST") {
        Some(HttpMethod::Post)
    } else if configured.eq_ignore_ascii_case("PUT") {
        Some(HttpMethod::Put)
    } else if configured.eq_ignore_ascii_case("DELETE") {
        Some(HttpMethod::Delete)
    } else {
        None
    };
    Ok(method)
}

pub(super) trait IntoRuleOperator {
    fn into_rule_operator(self) -> Result<RuleOp, UnsupportedReason>;
}

impl IntoRuleOperator for UrlPathOp {
    fn into_rule_operator(self) -> Result<RuleOp, UnsupportedReason> {
        match self {
            Self::Equal => Ok(RuleOp::Equal),
            Self::StartsWith => Ok(RuleOp::StartsWith),
            Self::Contains => Ok(RuleOp::Contains),
            Self::WildCard => Ok(RuleOp::WildCard),
            Self::NotEqual => Ok(RuleOp::NotEqual),
            Self::EndsWith => Err(UnsupportedReason::UrlOperator(self)),
        }
    }
}

impl IntoRuleOperator for HeaderOp {
    fn into_rule_operator(self) -> Result<RuleOp, UnsupportedReason> {
        match self {
            Self::Equal => Ok(RuleOp::Equal),
            Self::Contains => Ok(RuleOp::Contains),
            Self::StartsWith => Ok(RuleOp::StartsWith),
            Self::NotEqual => Ok(RuleOp::NotEqual),
            Self::WildCard => Ok(RuleOp::WildCard),
            Self::EndsWith | Self::Regex | Self::Exists | Self::Absent => {
                Err(UnsupportedReason::HeaderOperator(self))
            }
        }
    }
}

pub(super) fn rule_operator<T: IntoRuleOperator>(value: T) -> Result<RuleOp, UnsupportedReason> {
    value.into_rule_operator()
}

pub(super) fn body_operator(value: BodyOp) -> Result<BodyOperator, UnsupportedReason> {
    match value {
        BodyOp::Equal => Ok(BodyOperator::Equal),
        BodyOp::EqualString => Ok(BodyOperator::EqualString),
        BodyOp::Contains => Ok(BodyOperator::Contains),
        BodyOp::StartsWith => Ok(BodyOperator::StartsWith),
        BodyOp::EndsWith => Ok(BodyOperator::EndsWith),
        BodyOp::Regex => Err(UnsupportedReason::BodyOperator(value)),
        BodyOp::EqualTyped => Ok(BodyOperator::EqualTyped),
        BodyOp::ArrayContains => Ok(BodyOperator::ArrayContains),
        BodyOp::EqualNumber => Ok(BodyOperator::EqualNumber),
        BodyOp::GreaterThan => Ok(BodyOperator::GreaterThan),
        BodyOp::LessThan => Ok(BodyOperator::LessThan),
        BodyOp::GreaterOrEqual => Ok(BodyOperator::GreaterOrEqual),
        BodyOp::LessOrEqual => Ok(BodyOperator::LessOrEqual),
        BodyOp::EqualInteger => Ok(BodyOperator::EqualInteger),
        BodyOp::ArrayLengthEqual => Ok(BodyOperator::ArrayLengthEqual),
        BodyOp::ArrayLengthAtLeast => Ok(BodyOperator::ArrayLengthAtLeast),
        BodyOp::Exists => Ok(BodyOperator::Exists),
        BodyOp::Absent => Ok(BodyOperator::Absent),
    }
}
