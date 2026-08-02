//! Canonical `WorkspacePort` type → real `apimock_config` engine type
//! conversions (RFC MK-055 Tier 1).
//!
//! These conversions have no production consumer: `MemoryWorkspace` remains
//! the application's workspace implementation (RFC MK-055 non-goals), and
//! `apimock-config` is a test-only dev-dependency of this crate. They live
//! here, under `tests/`, rather than in `crates/model/src/`, precisely so
//! they cannot become production code before a production adapter is
//! separately designed and reviewed.

use apimokka_model::rule::{BodyOp, HeaderOp, UrlPathOp};
use apimokka_model::workspace_port::{
    BodyCondition, HeaderCondition, RespondDefinition, RuleMatch, RuleSetPath, WorkspaceEditValue,
    WorkspaceRootKey,
};

pub fn url_path_op(op: UrlPathOp) -> apimock_config::view::UrlPathOp {
    match op {
        UrlPathOp::Equal => apimock_config::view::UrlPathOp::Equal,
        UrlPathOp::StartsWith => apimock_config::view::UrlPathOp::StartsWith,
        UrlPathOp::Contains => apimock_config::view::UrlPathOp::Contains,
        UrlPathOp::EndsWith => apimock_config::view::UrlPathOp::EndsWith,
        UrlPathOp::WildCard => apimock_config::view::UrlPathOp::WildCard,
        UrlPathOp::NotEqual => apimock_config::view::UrlPathOp::NotEqual,
    }
}

pub fn header_op(op: HeaderOp) -> apimock_config::view::HeaderOp {
    match op {
        HeaderOp::Equal => apimock_config::view::HeaderOp::Equal,
        HeaderOp::Contains => apimock_config::view::HeaderOp::Contains,
        HeaderOp::StartsWith => apimock_config::view::HeaderOp::StartsWith,
        HeaderOp::EndsWith => apimock_config::view::HeaderOp::EndsWith,
        HeaderOp::Regex => apimock_config::view::HeaderOp::Regex,
        HeaderOp::Exists => apimock_config::view::HeaderOp::Exists,
        HeaderOp::Absent => apimock_config::view::HeaderOp::Absent,
        HeaderOp::NotEqual => apimock_config::view::HeaderOp::NotEqual,
        HeaderOp::WildCard => apimock_config::view::HeaderOp::WildCard,
    }
}

pub fn body_op(op: BodyOp) -> apimock_config::view::BodyOp {
    match op {
        BodyOp::Equal => apimock_config::view::BodyOp::Equal,
        BodyOp::EqualString => apimock_config::view::BodyOp::EqualString,
        BodyOp::Contains => apimock_config::view::BodyOp::Contains,
        BodyOp::StartsWith => apimock_config::view::BodyOp::StartsWith,
        BodyOp::EndsWith => apimock_config::view::BodyOp::EndsWith,
        BodyOp::Regex => apimock_config::view::BodyOp::Regex,
        BodyOp::EqualTyped => apimock_config::view::BodyOp::EqualTyped,
        BodyOp::ArrayContains => apimock_config::view::BodyOp::ArrayContains,
        BodyOp::EqualNumber => apimock_config::view::BodyOp::EqualNumber,
        BodyOp::GreaterThan => apimock_config::view::BodyOp::GreaterThan,
        BodyOp::LessThan => apimock_config::view::BodyOp::LessThan,
        BodyOp::GreaterOrEqual => apimock_config::view::BodyOp::GreaterOrEqual,
        BodyOp::LessOrEqual => apimock_config::view::BodyOp::LessOrEqual,
        BodyOp::EqualInteger => apimock_config::view::BodyOp::EqualInteger,
        BodyOp::ArrayLengthEqual => apimock_config::view::BodyOp::ArrayLengthEqual,
        BodyOp::ArrayLengthAtLeast => apimock_config::view::BodyOp::ArrayLengthAtLeast,
        BodyOp::Exists => apimock_config::view::BodyOp::Exists,
        BodyOp::Absent => apimock_config::view::BodyOp::Absent,
    }
}

pub fn header_condition(value: &HeaderCondition) -> apimock_config::view::HeaderConditionPayload {
    apimock_config::view::HeaderConditionPayload {
        name: value.name().as_str().to_owned(),
        op: header_op(value.op()),
        value: value.expected().map(str::to_owned),
    }
}

/// `BodyCondition::expected()` is `None` for `Exists`/`Absent`, but the
/// engine's `BodyConditionPayload.value` is a mandatory `serde_json::Value`
/// — a divergence this suite discovered directly from source, since the
/// unpublished 5.10.1 prose reference never showed this type and MK-053
/// could not have known it. `None` maps to `Value::Null`; the engine's
/// `Exists`/`Absent` matchers are presence-only and do not read this field
/// (verified in `tier2_scenarios.rs`).
pub fn body_condition(value: &BodyCondition) -> apimock_config::view::BodyConditionPayload {
    apimock_config::view::BodyConditionPayload {
        kind: apimock_config::view::BodyConditionKind::Json,
        path: value.path().to_owned(),
        op: body_op(value.op()),
        value: value.expected().cloned().unwrap_or(serde_json::Value::Null),
    }
}

pub fn rule_match_payload(
    value: &RuleMatch,
) -> (
    Option<String>,
    Option<apimock_config::view::UrlPathOp>,
    Option<String>,
) {
    (
        value.url_path().map(str::to_owned),
        value.url_path_op().map(url_path_op),
        value.method().map(str::to_owned),
    )
}

/// `RespondDefinition::status()` is a validated `"<3-digit code>[ reason]"`
/// string (see `map_response`/`validate_status`); `RespondPayload.status` is
/// a bare `u16`. Total given that validation invariant — the engine has no
/// representation for a reason phrase, so it is dropped, not rejected.
pub fn respond_status(status: &str) -> u16 {
    status[..3]
        .parse()
        .expect("RespondDefinition::status is validated to start with a 3-digit code")
}

/// `RespondDefinition::delay_milliseconds()` is `Option<u64>`;
/// `RespondPayload.delay_milliseconds` is `Option<u32>`. Total only because
/// `map_response` now rejects values above `u32::MAX` at construction time
/// (RFC MK-055 correction — the engine has no representation above that,
/// where the never-published 5.10.1 reference claimed `Option<u64>`).
pub fn respond(value: &RespondDefinition) -> apimock_config::RespondPayload {
    apimock_config::RespondPayload {
        file_path: value.file_path().map(|path| path.as_str().to_owned()),
        text: value.text().map(str::to_owned),
        status: value.status().map(respond_status),
        delay_milliseconds: value.delay_milliseconds().map(|millis| {
            u32::try_from(millis).expect(
                "map_response rejects delay_milliseconds above u32::MAX before construction",
            )
        }),
    }
}

pub fn rule_set_path(value: &RuleSetPath) -> String {
    value.as_relative().as_str().to_owned()
}

/// Accepted divergence, discovered by execution, not by reading: our
/// canonical `ServiceStrategy` value is whatever
/// `apimokka_model::settings::Strategy::label()` produced (`"FirstMatch"`,
/// PascalCase, matching the Settings screen's dropdown and already sent
/// verbatim by `app.rs`), but `apimock-config` 5.10.0's
/// `cmd_update_root_setting` only recognizes lowercase snake_case
/// (`workspace/edit.rs:557-573`: `"first_match"`, `"uniform_random"`,
/// `"weighted_random"`, `"priority"`, `"round_robin"`; anything else is
/// `ApplyError::InvalidPayload`).
///
/// This is not corrected in `map_root_setting` because doing so would
/// require changing what `app.rs` actually sends (`Strategy::label()`),
/// which is an application-behavior change this RFC's summary rules out
/// ("changes no production code path") — the same boundary that keeps the
/// status/delay corrections to the mapping layer alone. A future production
/// adapter needs exactly this translation at its boundary; this function
/// documents the accepted handling rule and lets Tier 1/2 tests exercise it
/// against the real engine.
pub fn strategy_wire_value(label: &str) -> &'static str {
    match label {
        "FirstMatch" => "first_match",
        "UniformRandom" => "uniform_random",
        "WeightedRandom" => "weighted_random",
        "Priority" => "priority",
        "RoundRobin" => "round_robin",
        other => panic!("unknown Strategy label: {other:?}"),
    }
}

/// Accepted divergence, discovered by execution: our canonical
/// `LogFormat` default and only currently-used value is `"plain"`
/// (`apimokka_model::settings::Settings::default`), but
/// `apimock-config` 5.10.0 only recognizes `"text"`/`"json"`
/// (`workspace/edit.rs:638-649`). Not corrected in `map_root_setting` for
/// the same application-behavior boundary reason as `strategy_wire_value`.
pub fn log_format_wire_value(label: &str) -> &'static str {
    match label {
        "plain" => "text",
        "json" => "json",
        other => panic!("unknown LogFormat label: {other:?}"),
    }
}

pub fn root_setting_key(value: WorkspaceRootKey) -> apimock_config::RootSettingKey {
    match value {
        WorkspaceRootKey::ListenerIpAddress => apimock_config::RootSettingKey::ListenerIpAddress,
        WorkspaceRootKey::ListenerPort => apimock_config::RootSettingKey::ListenerPort,
        WorkspaceRootKey::ServiceFallbackRespondDir => {
            apimock_config::RootSettingKey::ServiceFallbackRespondDir
        }
        WorkspaceRootKey::ServiceStrategy => apimock_config::RootSettingKey::ServiceStrategy,
        WorkspaceRootKey::TlsEnabled => apimock_config::RootSettingKey::TlsEnabled,
        WorkspaceRootKey::TlsCertFile => apimock_config::RootSettingKey::TlsCertFile,
        WorkspaceRootKey::TlsKeyFile => apimock_config::RootSettingKey::TlsKeyFile,
        WorkspaceRootKey::LogLevel => apimock_config::RootSettingKey::LogLevel,
        WorkspaceRootKey::LogFile => apimock_config::RootSettingKey::LogFile,
        WorkspaceRootKey::LogFormat => apimock_config::RootSettingKey::LogFormat,
        WorkspaceRootKey::FileTreeShowHidden => apimock_config::RootSettingKey::FileTreeShowHidden,
        WorkspaceRootKey::FileTreeBuiltinExcludes => {
            apimock_config::RootSettingKey::FileTreeBuiltinExcludes
        }
        WorkspaceRootKey::FileTreeExtraExcludes => {
            apimock_config::RootSettingKey::FileTreeExtraExcludes
        }
        WorkspaceRootKey::FileTreeInclude => apimock_config::RootSettingKey::FileTreeInclude,
    }
}

pub fn edit_value(value: &WorkspaceEditValue) -> apimock_config::EditValue {
    match value {
        WorkspaceEditValue::String(v) => apimock_config::EditValue::String(v.clone()),
        WorkspaceEditValue::Integer(v) => apimock_config::EditValue::Integer(*v),
        WorkspaceEditValue::Boolean(v) => apimock_config::EditValue::Boolean(*v),
        WorkspaceEditValue::StringList(v) => apimock_config::EditValue::StringList(v.clone()),
        WorkspaceEditValue::Enum(v) => apimock_config::EditValue::Enum(v.clone()),
    }
}
