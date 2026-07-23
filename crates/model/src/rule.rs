//! Rule editing payloads.
//!
//! Local render and editor-draft payloads. RFC MK-053 maps them explicitly to
//! canonical port values; they do not mirror an engine payload. Operator
//! categorisation is retained because the GUI's body-condition value input
//! changes shape per category (external design § 15.7).

use crate::respond::RespondPayload;

/// URL path matching operators (RFC 001).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UrlPathOp {
    Equal,
    StartsWith,
    Contains,
    EndsWith,
    WildCard,
    NotEqual,
}

impl UrlPathOp {
    pub fn all() -> [UrlPathOp; 6] {
        [
            UrlPathOp::Equal,
            UrlPathOp::StartsWith,
            UrlPathOp::Contains,
            UrlPathOp::EndsWith,
            UrlPathOp::WildCard,
            UrlPathOp::NotEqual,
        ]
    }
    pub fn label(self) -> &'static str {
        match self {
            UrlPathOp::Equal => "Equal",
            UrlPathOp::StartsWith => "StartsWith",
            UrlPathOp::Contains => "Contains",
            UrlPathOp::EndsWith => "EndsWith",
            UrlPathOp::WildCard => "WildCard",
            UrlPathOp::NotEqual => "NotEqual",
        }
    }
}

impl std::fmt::Display for UrlPathOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Header condition operators. `Exists`/`Absent` ignore the value field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeaderOp {
    Equal,
    Contains,
    StartsWith,
    EndsWith,
    Regex,
    Exists,
    Absent,
    NotEqual,
    WildCard,
}

impl HeaderOp {
    pub fn all() -> [HeaderOp; 9] {
        [
            HeaderOp::Equal,
            HeaderOp::Contains,
            HeaderOp::StartsWith,
            HeaderOp::EndsWith,
            HeaderOp::Regex,
            HeaderOp::Exists,
            HeaderOp::Absent,
            HeaderOp::NotEqual,
            HeaderOp::WildCard,
        ]
    }
    pub fn label(self) -> &'static str {
        match self {
            HeaderOp::Equal => "Equal",
            HeaderOp::Contains => "Contains",
            HeaderOp::StartsWith => "StartsWith",
            HeaderOp::EndsWith => "EndsWith",
            HeaderOp::Regex => "Regex",
            HeaderOp::Exists => "Exists",
            HeaderOp::Absent => "Absent",
            HeaderOp::NotEqual => "NotEqual",
            HeaderOp::WildCard => "WildCard",
        }
    }
    /// True for operators where the GUI must hide the value field.
    pub fn value_irrelevant(self) -> bool {
        matches!(self, HeaderOp::Exists | HeaderOp::Absent)
    }
}

impl std::fmt::Display for HeaderOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Body condition operators (RFC 008 + 010). The mockup keeps the full
/// 18-variant list so the operator-category → input-shape mapping in
/// external design § 15.7 has somewhere to dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyOp {
    // String-coerced
    Equal,
    EqualString,
    Contains,
    StartsWith,
    EndsWith,
    Regex,
    // Type-aware
    EqualTyped,
    ArrayContains,
    // Numeric (f64)
    EqualNumber,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    // Exact integer
    EqualInteger,
    ArrayLengthEqual,
    ArrayLengthAtLeast,
    // Presence (ignore value)
    Exists,
    Absent,
}

/// Operator categories used by the rule builder to pick a value-input
/// widget. See external design § 15.7.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyOpCategory {
    String,
    Json,
    Number,
    Integer,
    Presence,
}

impl BodyOp {
    pub fn all() -> [BodyOp; 18] {
        [
            BodyOp::Equal,
            BodyOp::EqualString,
            BodyOp::Contains,
            BodyOp::StartsWith,
            BodyOp::EndsWith,
            BodyOp::Regex,
            BodyOp::EqualTyped,
            BodyOp::ArrayContains,
            BodyOp::EqualNumber,
            BodyOp::GreaterThan,
            BodyOp::LessThan,
            BodyOp::GreaterOrEqual,
            BodyOp::LessOrEqual,
            BodyOp::EqualInteger,
            BodyOp::ArrayLengthEqual,
            BodyOp::ArrayLengthAtLeast,
            BodyOp::Exists,
            BodyOp::Absent,
        ]
    }
    pub fn label(self) -> &'static str {
        match self {
            BodyOp::Equal => "Equal",
            BodyOp::EqualString => "EqualString",
            BodyOp::Contains => "Contains",
            BodyOp::StartsWith => "StartsWith",
            BodyOp::EndsWith => "EndsWith",
            BodyOp::Regex => "Regex",
            BodyOp::EqualTyped => "EqualTyped",
            BodyOp::ArrayContains => "ArrayContains",
            BodyOp::EqualNumber => "EqualNumber",
            BodyOp::GreaterThan => "GreaterThan",
            BodyOp::LessThan => "LessThan",
            BodyOp::GreaterOrEqual => "GreaterOrEqual",
            BodyOp::LessOrEqual => "LessOrEqual",
            BodyOp::EqualInteger => "EqualInteger",
            BodyOp::ArrayLengthEqual => "ArrayLengthEqual",
            BodyOp::ArrayLengthAtLeast => "ArrayLengthAtLeast",
            BodyOp::Exists => "Exists",
            BodyOp::Absent => "Absent",
        }
    }
    pub fn category(self) -> BodyOpCategory {
        use BodyOp::*;
        match self {
            Equal | EqualString | Contains | StartsWith | EndsWith | Regex => {
                BodyOpCategory::String
            }
            EqualTyped | ArrayContains => BodyOpCategory::Json,
            EqualNumber | GreaterThan | LessThan | GreaterOrEqual | LessOrEqual => {
                BodyOpCategory::Number
            }
            EqualInteger | ArrayLengthEqual | ArrayLengthAtLeast => BodyOpCategory::Integer,
            Exists | Absent => BodyOpCategory::Presence,
        }
    }
}

impl std::fmt::Display for BodyOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone)]
pub struct HeaderConditionPayload {
    pub name: String,
    pub op: HeaderOp,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct BodyConditionPayload {
    pub path: String, // dotted path; NOT JSONPath
    pub op: BodyOp,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct RulePayload {
    pub url_path: String,
    /// `None` means the URL path is unused. The GUI must not send a value
    /// here while the path is empty (RFC 013 validation).
    pub url_path_op: Option<UrlPathOp>,
    /// Empty string = "Any" / method not constrained.
    pub method: String,
    pub headers: Vec<HeaderConditionPayload>,
    pub body: Vec<BodyConditionPayload>,
    pub respond: RespondPayload,
    /// Strategy-specific. Only visible when current Strategy is Weighted.
    pub weight: Option<u32>,
    /// Strategy-specific. Only visible when current Strategy is Priority.
    pub priority: Option<i32>,
}

impl Default for RulePayload {
    fn default() -> Self {
        Self {
            url_path: String::new(),
            url_path_op: None,
            method: String::new(),
            headers: Vec::new(),
            body: Vec::new(),
            respond: RespondPayload::default(),
            weight: None,
            priority: None,
        }
    }
}
