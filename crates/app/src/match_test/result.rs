use apimokka_model::{BodyOp, HeaderOp, UrlPathOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestRuleOutcome {
    Matched,
    NoMatch,
    Unsupported,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestRuleResult {
    pub outcome: TestRuleOutcome,
    pub conditions: Vec<ConditionResult>,
    pub diagnostics: Vec<RequestDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionResult {
    pub condition: ConditionIdentity,
    pub outcome: ConditionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionIdentity {
    Method,
    UrlPath,
    Header { index: usize, name: String },
    Body { index: usize, path: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionOutcome {
    Passed,
    Failed,
    Unsupported { reason: UnsupportedReason },
    Error { reason: EvaluationError },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnsupportedReason {
    ConfiguredMethod(String),
    UrlOperator(UrlPathOp),
    HeaderOperator(HeaderOp),
    BodyOperator(BodyOp),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestDiagnostic {
    pub scope: DiagnosticScope,
    pub reason: EvaluationError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticScope {
    Selection,
    RequestMethod,
    RequestHeaderLine(usize),
    RequestBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationError {
    NoRuleSelected,
    InvalidRequestMethod(String),
    InvalidConfiguredMethod(String),
    MissingHeaderColon,
    InvalidHeaderName(String),
    InvalidHeaderValue,
    HeaderValueNotText,
    DuplicateHeader { name: String, first_line: usize },
    InvalidConfiguredHeaderName(String),
    InvalidRequestBody,
    InvalidConfiguredJson,
    InvalidConfiguredNumber,
    InvalidConfiguredInteger,
    InvalidConfiguredLength,
}

pub(super) fn aggregate(
    conditions: &[ConditionResult],
    diagnostics: &[RequestDiagnostic],
) -> TestRuleOutcome {
    if !diagnostics.is_empty()
        || conditions
            .iter()
            .any(|result| matches!(result.outcome, ConditionOutcome::Error { .. }))
    {
        TestRuleOutcome::Error
    } else if conditions
        .iter()
        .any(|result| matches!(result.outcome, ConditionOutcome::Unsupported { .. }))
    {
        TestRuleOutcome::Unsupported
    } else if conditions
        .iter()
        .any(|result| matches!(result.outcome, ConditionOutcome::Failed))
    {
        TestRuleOutcome::NoMatch
    } else {
        TestRuleOutcome::Matched
    }
}
