use std::net::IpAddr;
use std::str::FromStr;

use serde_json::{Number, Value};

use super::{
    BodyCondition, FieldError, FieldErrorKind, HeaderCondition, PathError, RespondDefinition,
    ResponseMode, RootSettingEdit, RuleMatch, RuleSetPath, RuntimeEffect, WorkspaceEditValue,
    WorkspaceRelativePath, WorkspaceRootKey,
};
use crate::respond::{RespondMode, RespondPayload};
use crate::rule::{BodyConditionPayload, BodyOp, HeaderConditionPayload, HeaderOp, UrlPathOp};

pub fn parse_workspace_relative_path(
    field: &'static str,
    value: &str,
) -> Result<WorkspaceRelativePath, FieldError> {
    let fail = |kind| FieldError::new(field, FieldErrorKind::InvalidPath(kind));
    if value.is_empty() {
        return Err(fail(PathError::Empty));
    }
    if value.contains('\0') {
        return Err(fail(PathError::Nul));
    }
    if value.contains('\\') {
        return Err(fail(PathError::Backslash));
    }
    if value.starts_with('/') {
        return Err(fail(PathError::Absolute));
    }
    let first = value.split('/').next().unwrap_or_default().as_bytes();
    if first.len() >= 2 && first[0].is_ascii_alphabetic() && first[1] == b':' {
        return Err(fail(PathError::WindowsPrefix));
    }
    for component in value.split('/') {
        if component.is_empty() {
            return Err(fail(PathError::EmptyComponent));
        }
        if matches!(component, "." | "..") {
            return Err(fail(PathError::DotComponent));
        }
    }
    Ok(WorkspaceRelativePath(value.to_owned()))
}

pub fn parse_rule_set_path(value: &str) -> Result<RuleSetPath, FieldError> {
    let path = parse_workspace_relative_path("rule_set_path", value)?;
    if !value.ends_with(".toml") {
        return Err(FieldError::new(
            "rule_set_path",
            FieldErrorKind::InvalidPath(PathError::WrongExtension),
        ));
    }
    Ok(RuleSetPath(path))
}

pub fn map_rule_match(
    url_path: &str,
    url_path_op: Option<UrlPathOp>,
    method: &str,
) -> Result<RuleMatch, FieldError> {
    let (url_path, url_path_op) = if url_path.is_empty() {
        if url_path_op.is_some() {
            return Err(FieldError::new(
                "url_path_op",
                FieldErrorKind::UnexpectedUrlOperator,
            ));
        }
        (None, None)
    } else {
        let op = url_path_op
            .ok_or_else(|| FieldError::new("url_path_op", FieldErrorKind::MissingUrlOperator))?;
        // This identity conversion is intentionally exhaustive: adding an
        // operator must break this boundary until its mapping is reviewed.
        #[allow(clippy::needless_match)]
        let op = match op {
            UrlPathOp::Equal => UrlPathOp::Equal,
            UrlPathOp::StartsWith => UrlPathOp::StartsWith,
            UrlPathOp::Contains => UrlPathOp::Contains,
            UrlPathOp::EndsWith => UrlPathOp::EndsWith,
            UrlPathOp::WildCard => UrlPathOp::WildCard,
            UrlPathOp::NotEqual => UrlPathOp::NotEqual,
        };
        (Some(url_path.to_owned()), Some(op))
    };

    let method = match method {
        "" => None,
        "GET" | "POST" | "PUT" | "DELETE" => Some(method.to_owned()),
        _ => return Err(FieldError::new("method", FieldErrorKind::InvalidMethod)),
    };
    Ok(RuleMatch {
        url_path,
        url_path_op,
        method,
    })
}

pub fn map_header_condition(
    name: &str,
    op: HeaderOp,
    expected: &str,
) -> Result<HeaderCondition, FieldError> {
    let name = http::HeaderName::from_bytes(name.as_bytes())
        .map_err(|_| FieldError::new("header_name", FieldErrorKind::InvalidHeaderName))?;
    let expected = match op {
        HeaderOp::Equal
        | HeaderOp::Contains
        | HeaderOp::StartsWith
        | HeaderOp::EndsWith
        | HeaderOp::Regex
        | HeaderOp::NotEqual
        | HeaderOp::WildCard => Some(expected.to_owned()),
        HeaderOp::Exists | HeaderOp::Absent => {
            if !expected.is_empty() {
                return Err(FieldError::new(
                    "header_expected",
                    FieldErrorKind::UnexpectedValue,
                ));
            }
            None
        }
    };
    Ok(HeaderCondition { name, op, expected })
}

pub fn map_body_condition(
    path: &str,
    op: BodyOp,
    expected: &str,
) -> Result<BodyCondition, FieldError> {
    validate_body_path(path)?;
    use BodyOp::*;
    let expected = match op {
        Equal | EqualString | Contains | StartsWith | EndsWith | Regex => {
            Some(Value::String(expected.to_owned()))
        }
        EqualTyped | ArrayContains => Some(
            serde_json::from_str(expected)
                .map_err(|_| FieldError::new("body_expected", FieldErrorKind::InvalidJson))?,
        ),
        EqualNumber | GreaterThan | LessThan | GreaterOrEqual | LessOrEqual => {
            let parsed: Value = serde_json::from_str(expected)
                .map_err(|_| FieldError::new("body_expected", FieldErrorKind::InvalidNumber))?;
            let number = parsed
                .as_f64()
                .filter(|value| value.is_finite())
                .and_then(Number::from_f64)
                .ok_or_else(|| FieldError::new("body_expected", FieldErrorKind::InvalidNumber))?;
            Some(Value::Number(number))
        }
        EqualInteger => {
            if !valid_signed_integer(expected) {
                return Err(FieldError::new(
                    "body_expected",
                    FieldErrorKind::InvalidInteger,
                ));
            }
            let value = expected
                .parse::<i64>()
                .map_err(|_| FieldError::new("body_expected", FieldErrorKind::InvalidInteger))?;
            Some(Value::Number(Number::from(value)))
        }
        ArrayLengthEqual | ArrayLengthAtLeast => {
            if !valid_unsigned_integer(expected) {
                return Err(FieldError::new(
                    "body_expected",
                    FieldErrorKind::InvalidInteger,
                ));
            }
            let value = expected
                .parse::<usize>()
                .map_err(|_| FieldError::new("body_expected", FieldErrorKind::InvalidInteger))?;
            Some(Value::Number(Number::from(value as u64)))
        }
        Exists | Absent => {
            if !expected.is_empty() {
                return Err(FieldError::new(
                    "body_expected",
                    FieldErrorKind::UnexpectedValue,
                ));
            }
            None
        }
    };
    Ok(BodyCondition {
        path: path.to_owned(),
        op,
        expected,
    })
}

fn validate_body_path(path: &str) -> Result<(), FieldError> {
    if path.is_empty()
        || path.contains('\0')
        || path.split('.').next() == Some("$")
        || path.split('.').any(str::is_empty)
    {
        return Err(FieldError::new(
            "body_path",
            FieldErrorKind::InvalidBodyPath,
        ));
    }
    Ok(())
}

fn valid_unsigned_integer(value: &str) -> bool {
    value == "0"
        || value
            .strip_prefix(['1', '2', '3', '4', '5', '6', '7', '8', '9'])
            .is_some_and(|rest| rest.bytes().all(|byte| byte.is_ascii_digit()))
}

fn valid_signed_integer(value: &str) -> bool {
    valid_unsigned_integer(value)
        || value
            .strip_prefix('-')
            .is_some_and(|rest| rest != "0" && valid_unsigned_integer(rest))
}

pub fn map_response(
    mode: ResponseMode,
    text: &str,
    file_path: &str,
    status: &str,
    delay_milliseconds: &str,
) -> Result<RespondDefinition, FieldError> {
    let (text, file_path) = match mode {
        ResponseMode::Inline => (Some(text.to_owned()), None),
        ResponseMode::File => (
            None,
            Some(parse_workspace_relative_path("response_file", file_path)?),
        ),
    };
    let status = if status.is_empty() {
        None
    } else {
        validate_status(status)?;
        Some(status.to_owned())
    };
    let delay_milliseconds = if delay_milliseconds.is_empty() {
        None
    } else if delay_milliseconds.bytes().all(|byte| byte.is_ascii_digit()) {
        let value = delay_milliseconds
            .parse::<u64>()
            .map_err(|_| FieldError::new("response_delay", FieldErrorKind::InvalidDelay))?;
        // RFC MK-055: the engine's `RespondPayload.delay_milliseconds` is
        // `Option<u32>`, not `Option<u64>` as the unpublished 5.10.1 prose
        // reference stated. Reject values the engine cannot represent rather
        // than silently truncating or letting a later engine call fail.
        if value > u64::from(u32::MAX) {
            return Err(FieldError::new(
                "response_delay",
                FieldErrorKind::InvalidDelay,
            ));
        }
        Some(value)
    } else {
        return Err(FieldError::new(
            "response_delay",
            FieldErrorKind::InvalidDelay,
        ));
    };
    Ok(RespondDefinition {
        text,
        file_path,
        status,
        delay_milliseconds,
    })
}

fn validate_status(status: &str) -> Result<(), FieldError> {
    let bytes = status.as_bytes();
    let valid_shape = bytes.len() >= 3
        && bytes[..3].iter().all(u8::is_ascii_digit)
        && (bytes.len() == 3 || (bytes[3] == b' ' && bytes.len() > 4 && bytes[4] != b' '));
    let code = status.get(..3).and_then(|value| value.parse::<u16>().ok());
    if !valid_shape || !code.is_some_and(|code| (100..=599).contains(&code)) {
        return Err(FieldError::new(
            "response_status",
            FieldErrorKind::InvalidStatus,
        ));
    }
    Ok(())
}

pub fn map_root_setting(
    key: WorkspaceRootKey,
    value: WorkspaceEditValue,
) -> Result<RootSettingEdit, FieldError> {
    use WorkspaceEditValue as V;
    use WorkspaceRootKey as K;

    let effect = match key {
        K::ListenerIpAddress => {
            let V::String(value) = &value else {
                return type_mismatch();
            };
            IpAddr::from_str(value)
                .map_err(|_| FieldError::new("root_value", FieldErrorKind::ValueOutOfRange))?;
            RuntimeEffect::Restart
        }
        K::ListenerPort => {
            let V::Integer(value) = value else {
                return type_mismatch();
            };
            if !(1..=65535).contains(&value) {
                return Err(FieldError::new(
                    "root_value",
                    FieldErrorKind::ValueOutOfRange,
                ));
            }
            return Ok(RootSettingEdit {
                key,
                value: V::Integer(value),
                effect: RuntimeEffect::Restart,
            });
        }
        K::ServiceFallbackRespondDir => {
            validate_optional_path(&value, "root_value")?;
            RuntimeEffect::Reload
        }
        K::ServiceStrategy => {
            validate_enum(
                &value,
                &[
                    "FirstMatch",
                    "UniformRandom",
                    "WeightedRandom",
                    "Priority",
                    "RoundRobin",
                ],
            )?;
            RuntimeEffect::Reload
        }
        K::TlsEnabled => {
            require_boolean(&value)?;
            RuntimeEffect::Restart
        }
        K::TlsCertFile | K::TlsKeyFile | K::LogFile => {
            validate_optional_path(&value, "root_value")?;
            RuntimeEffect::Restart
        }
        K::LogLevel => {
            validate_enum(&value, &["error", "warn", "info", "debug", "trace"])?;
            RuntimeEffect::Reload
        }
        K::LogFormat => {
            validate_enum(&value, &["plain", "json"])?;
            RuntimeEffect::Reload
        }
        K::FileTreeShowHidden | K::FileTreeBuiltinExcludes => {
            require_boolean(&value)?;
            RuntimeEffect::Reload
        }
        K::FileTreeExtraExcludes | K::FileTreeInclude => {
            let V::StringList(values) = &value else {
                return type_mismatch();
            };
            if values.iter().any(String::is_empty) {
                return Err(FieldError::new("root_value", FieldErrorKind::Empty));
            }
            RuntimeEffect::Reload
        }
    };
    Ok(RootSettingEdit { key, value, effect })
}

fn validate_optional_path(
    value: &WorkspaceEditValue,
    field: &'static str,
) -> Result<(), FieldError> {
    let WorkspaceEditValue::String(value) = value else {
        return type_mismatch();
    };
    if !value.is_empty() {
        parse_workspace_relative_path(field, value)?;
    }
    Ok(())
}

fn validate_enum(value: &WorkspaceEditValue, allowed: &[&str]) -> Result<(), FieldError> {
    let WorkspaceEditValue::Enum(value) = value else {
        return type_mismatch();
    };
    if !allowed.contains(&value.as_str()) {
        return Err(FieldError::new(
            "root_value",
            FieldErrorKind::UnknownEnumValue,
        ));
    }
    Ok(())
}

fn require_boolean(value: &WorkspaceEditValue) -> Result<(), FieldError> {
    if matches!(value, WorkspaceEditValue::Boolean(_)) {
        Ok(())
    } else {
        type_mismatch()
    }
}

fn type_mismatch<T>() -> Result<T, FieldError> {
    Err(FieldError::new(
        "root_value",
        FieldErrorKind::ValueTypeMismatch,
    ))
}

pub(super) fn project_header_condition(value: &HeaderCondition) -> HeaderConditionPayload {
    let expected = match value.op {
        HeaderOp::Equal
        | HeaderOp::Contains
        | HeaderOp::StartsWith
        | HeaderOp::EndsWith
        | HeaderOp::Regex
        | HeaderOp::NotEqual
        | HeaderOp::WildCard => value
            .expected
            .as_deref()
            .expect("value-bearing canonical header condition has an expected value")
            .to_owned(),
        HeaderOp::Exists | HeaderOp::Absent => String::new(),
    };
    HeaderConditionPayload {
        name: value.name.as_str().to_owned(),
        op: value.op,
        value: expected,
    }
}

pub(super) fn project_body_condition(value: &BodyCondition) -> BodyConditionPayload {
    use BodyOp::*;
    let expected = match value.op {
        Equal | EqualString | Contains | StartsWith | EndsWith | Regex => value
            .expected
            .as_ref()
            .and_then(Value::as_str)
            .expect("canonical string condition contains a JSON string")
            .to_owned(),
        EqualTyped | ArrayContains => value
            .expected
            .as_ref()
            .map(canonical_json)
            .expect("canonical typed condition has an expected value"),
        EqualNumber | GreaterThan | LessThan | GreaterOrEqual | LessOrEqual | EqualInteger
        | ArrayLengthEqual | ArrayLengthAtLeast => value
            .expected
            .as_ref()
            .and_then(Value::as_number)
            .map(ToString::to_string)
            .expect("canonical numeric condition contains a JSON number"),
        Exists | Absent => String::new(),
    };
    BodyConditionPayload {
        path: value.path.clone(),
        op: value.op,
        value: expected,
    }
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => {
            serde_json::to_string(value).expect("serializing an in-memory JSON string cannot fail")
        }
        Value::Array(values) => {
            let values = values.iter().map(canonical_json).collect::<Vec<_>>();
            format!("[{}]", values.join(","))
        }
        Value::Object(values) => {
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_by(|(left, _), (right, _)| left.as_bytes().cmp(right.as_bytes()));
            let entries = entries
                .into_iter()
                .map(|(key, value)| {
                    let key = serde_json::to_string(key)
                        .expect("serializing an in-memory JSON object key cannot fail");
                    format!("{key}:{}", canonical_json(value))
                })
                .collect::<Vec<_>>();
            format!("{{{}}}", entries.join(","))
        }
    }
}

pub(super) fn project_rule_match(value: &RuleMatch) -> (String, Option<UrlPathOp>, String) {
    (
        value.url_path.clone().unwrap_or_default(),
        value.url_path_op,
        value.method.clone().unwrap_or_default(),
    )
}

pub(super) fn project_response(value: &RespondDefinition) -> RespondPayload {
    let (mode, text, file_path) = match (&value.text, &value.file_path) {
        (Some(text), None) => (RespondMode::InlineText, text.clone(), String::new()),
        (None, Some(file_path)) => (
            RespondMode::ServeFile,
            String::new(),
            file_path.as_str().to_owned(),
        ),
        _ => unreachable!("RespondDefinition construction seals response mode"),
    };
    RespondPayload {
        mode,
        text,
        file_path,
        status: value.status.clone().unwrap_or_default(),
        delay_milliseconds: value.delay_milliseconds.unwrap_or(0),
    }
}
