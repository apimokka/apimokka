//! MK-053 application-facing workspace boundary.
//!
//! These types intentionally describe the local UI contract. They do not claim
//! to be source- or binary-isomorphic with an unavailable engine crate.
//!
//! Boundary decision: single-responsibility — the complete value-type
//! vocabulary the `WorkspacePort` contract is expressed in, plus the trait
//! itself: errors and paths, conditions and edits, rule/respond views, root
//! settings, the edit-intent/transaction vocabulary, archived-subtree/undo
//! types, and outcomes. This is the contract's grammar, not independent
//! concerns — implementations are small accessors and constructors, not
//! logic. Splitting would scatter one contract's vocabulary across several
//! files a reader must open together anyway.

mod mapping;
mod memory;

#[cfg(test)]
mod memory_tests;

#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::fmt;

use serde_json::Value;

use crate::ids::{NodeId, RuleSetId};
use crate::rule::{BodyOp, HeaderOp, UrlPathOp};
use crate::snapshot::WorkspaceSnapshot;
use crate::validation::{Diagnostic, ValidationReport};

pub use mapping::{
    map_body_condition, map_header_condition, map_response, map_root_setting, map_rule_match,
    parse_rule_set_path, parse_workspace_relative_path,
};
pub use memory::MemoryWorkspace;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError {
    pub field: &'static str,
    pub kind: FieldErrorKind,
}

impl FieldError {
    pub const fn new(field: &'static str, kind: FieldErrorKind) -> Self {
        Self { field, kind }
    }
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.kind)
    }
}

impl std::error::Error for FieldError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldErrorKind {
    Empty,
    InvalidPath(PathError),
    InvalidHeaderName,
    InvalidBodyPath,
    InvalidJson,
    WrongJsonType,
    InvalidInteger,
    InvalidNumber,
    InvalidStatus,
    InvalidDelay,
    InvalidMethod,
    MissingUrlOperator,
    UnexpectedUrlOperator,
    UnexpectedValue,
    ValueTypeMismatch,
    ValueOutOfRange,
    UnknownEnumValue,
    EmptyTransaction,
    DuplicateCreationKey,
    MissingArchiveRoot,
    DuplicateArchivedNode,
    InvalidRestorePlacement,
    InvalidArchiveTopology,
    DuplicateNodeId,
    DuplicateRuleSetPath,
    SaveTargetNotDirty,
}

impl fmt::Display for FieldErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    Empty,
    Absolute,
    WindowsPrefix,
    Backslash,
    EmptyComponent,
    DotComponent,
    Nul,
    WrongExtension,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkspaceRelativePath(String);

impl WorkspaceRelativePath {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }
}

impl fmt::Display for WorkspaceRelativePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuleSetPath(WorkspaceRelativePath);

impl RuleSetPath {
    pub fn as_relative(&self) -> &WorkspaceRelativePath {
        &self.0
    }
}

impl fmt::Display for RuleSetPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionEdit<T> {
    Preserve,
    Clear,
    Replace(Vec<T>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionEdit<T> {
    Existing {
        id: NodeId,
        condition: T,
    },
    Create {
        key: SemanticCreationKey,
        condition: T,
    },
}

impl<T> CollectionEdit<T> {
    pub fn into_reference_option(self) -> Option<Vec<T>> {
        match self {
            Self::Preserve => None,
            Self::Clear => Some(Vec::new()),
            Self::Replace(values) => Some(values),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionWithId<T> {
    pub id: NodeId,
    pub condition: T,
}

/// Canonical header condition. Construction is restricted to
/// [`map_header_condition`], so a presence operator cannot carry a value.
///
/// ```compile_fail
/// use apimokka_model::{HeaderOp, workspace_port::HeaderCondition};
/// let _ = HeaderCondition {
///     name: "x-a".parse().unwrap(),
///     op: HeaderOp::Exists,
///     expected: Some("forged".into()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderCondition {
    name: http::HeaderName,
    op: HeaderOp,
    expected: Option<String>,
}

impl HeaderCondition {
    pub fn name(&self) -> &http::HeaderName {
        &self.name
    }
    pub fn op(&self) -> HeaderOp {
        self.op
    }
    pub fn expected(&self) -> Option<&str> {
        self.expected.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BodyCondition {
    path: String,
    op: BodyOp,
    expected: Option<Value>,
}

impl BodyCondition {
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn op(&self) -> BodyOp {
        self.op
    }
    pub fn expected(&self) -> Option<&Value> {
        self.expected.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleConditionsView {
    pub headers: Vec<ConditionWithId<HeaderCondition>>,
    pub body: Vec<ConditionWithId<BodyCondition>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortRuleView {
    rule_id: NodeId,
    rule_match: RuleMatch,
    conditions: RuleConditionsView,
    respond: RespondDefinition,
}

impl PortRuleView {
    pub fn rule_id(&self) -> NodeId {
        self.rule_id
    }

    pub fn rule_match(&self) -> &RuleMatch {
        &self.rule_match
    }

    pub fn conditions(&self) -> &RuleConditionsView {
        &self.conditions
    }

    pub fn respond(&self) -> &RespondDefinition {
        &self.respond
    }
}

/// Canonical rule match. Construction is restricted to [`map_rule_match`].
///
/// ```compile_fail
/// use apimokka_model::workspace_port::RuleMatch;
/// let _ = RuleMatch {
///     url_path: Some("/orders".into()),
///     url_path_op: None,
///     method: Some("PATCH".into()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleMatch {
    url_path: Option<String>,
    url_path_op: Option<UrlPathOp>,
    method: Option<String>,
}

impl RuleMatch {
    pub fn url_path(&self) -> Option<&str> {
        self.url_path.as_deref()
    }
    pub fn url_path_op(&self) -> Option<UrlPathOp> {
        self.url_path_op
    }
    pub fn method(&self) -> Option<&str> {
        self.method.as_deref()
    }
}

/// Canonical response value. Fields are private so callers must use
/// [`map_response`] and cannot construct both inline and file modes.
///
/// ```compile_fail
/// use apimokka_model::workspace_port::RespondDefinition;
/// let _ = RespondDefinition {
///     text: Some("x".into()),
///     file_path: None,
///     status: None,
///     delay_milliseconds: None,
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RespondDefinition {
    text: Option<String>,
    file_path: Option<WorkspaceRelativePath>,
    status: Option<String>,
    delay_milliseconds: Option<u64>,
}

impl RespondDefinition {
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }
    pub fn file_path(&self) -> Option<&WorkspaceRelativePath> {
        self.file_path.as_ref()
    }
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }
    pub fn delay_milliseconds(&self) -> Option<u64> {
        self.delay_milliseconds
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseMode {
    Inline,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkspaceRootKey {
    ListenerIpAddress,
    ListenerPort,
    ServiceFallbackRespondDir,
    ServiceStrategy,
    TlsEnabled,
    TlsCertFile,
    TlsKeyFile,
    LogLevel,
    LogFile,
    LogFormat,
    FileTreeShowHidden,
    FileTreeBuiltinExcludes,
    FileTreeExtraExcludes,
    FileTreeInclude,
}

impl WorkspaceRootKey {
    pub const ALL: [Self; 14] = [
        Self::ListenerIpAddress,
        Self::ListenerPort,
        Self::ServiceFallbackRespondDir,
        Self::ServiceStrategy,
        Self::TlsEnabled,
        Self::TlsCertFile,
        Self::TlsKeyFile,
        Self::LogLevel,
        Self::LogFile,
        Self::LogFormat,
        Self::FileTreeShowHidden,
        Self::FileTreeBuiltinExcludes,
        Self::FileTreeExtraExcludes,
        Self::FileTreeInclude,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceEditValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    StringList(Vec<String>),
    Enum(String),
}

/// Canonical root-setting edit. The runtime effect is derived by
/// [`map_root_setting`] and cannot be supplied by a caller.
///
/// ```compile_fail
/// use apimokka_model::workspace_port::{
///     RootSettingEdit, RuntimeEffect, WorkspaceEditValue, WorkspaceRootKey,
/// };
/// let _ = RootSettingEdit {
///     key: WorkspaceRootKey::ListenerPort,
///     value: WorkspaceEditValue::Integer(8080),
///     effect: RuntimeEffect::None,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootSettingEdit {
    key: WorkspaceRootKey,
    value: WorkspaceEditValue,
    effect: RuntimeEffect,
}

impl RootSettingEdit {
    pub fn key(&self) -> WorkspaceRootKey {
        self.key
    }
    pub fn value(&self) -> &WorkspaceEditValue {
        &self.value
    }
    pub fn effect(&self) -> RuntimeEffect {
        self.effect
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimeEffect {
    #[default]
    None,
    Reload,
    Restart,
}

impl RuntimeEffect {
    pub fn combine(self, other: Self) -> Self {
        self.max(other)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleEditPayload {
    pub rule_match: RuleMatch,
    pub headers: CollectionEdit<ConditionEdit<HeaderCondition>>,
    pub body: CollectionEdit<ConditionEdit<BodyCondition>>,
    pub respond: RespondDefinition,
}

impl RuleEditPayload {
    fn creation_keys(&self) -> impl Iterator<Item = &SemanticCreationKey> {
        self.headers
            .creation_keys()
            .chain(self.body.creation_keys())
    }
}

impl<T> CollectionEdit<ConditionEdit<T>> {
    fn creation_keys(&self) -> impl Iterator<Item = &SemanticCreationKey> {
        match self {
            Self::Replace(values) => values.iter(),
            Self::Preserve | Self::Clear => [].iter(),
        }
        .filter_map(|value| match value {
            ConditionEdit::Create { key, .. } => Some(key),
            ConditionEdit::Existing { .. } => None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemanticCreationKey(String);

impl SemanticCreationKey {
    pub fn new(value: impl Into<String>) -> Result<Self, FieldError> {
        let value = value.into();
        if value.is_empty() {
            return Err(FieldError::new("creation_key", FieldErrorKind::Empty));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkspaceNodeKind {
    RuleSet,
    Rule,
    HeaderCondition,
    BodyCondition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreationReceipt {
    pub key: SemanticCreationKey,
    pub kind: WorkspaceNodeKind,
    pub parent: Option<NodeId>,
    pub new_id: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeRebind {
    pub old_id: NodeId,
    pub kind: WorkspaceNodeKind,
    pub new_id: NodeId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditIntent {
    AddRuleSet {
        path: RuleSetPath,
        key: SemanticCreationKey,
    },
    RemoveRuleSet {
        id: RuleSetId,
    },
    AddRule {
        parent: RuleSetId,
        insertion_index: usize,
        rule: RuleEditPayload,
        key: SemanticCreationKey,
    },
    UpdateRule {
        id: NodeId,
        rule: RuleEditPayload,
    },
    DeleteRule {
        id: NodeId,
    },
    MoveRule {
        id: NodeId,
        new_index: usize,
    },
    UpdateRespond {
        id: NodeId,
        respond: RespondDefinition,
    },
    UpdateRootSetting(RootSettingEdit),
    AddHeaderCondition {
        rule_id: NodeId,
        condition: HeaderCondition,
        key: SemanticCreationKey,
    },
    UpdateHeaderCondition {
        id: NodeId,
        condition: HeaderCondition,
    },
    RemoveHeaderCondition {
        id: NodeId,
    },
    AddBodyCondition {
        rule_id: NodeId,
        condition: BodyCondition,
        key: SemanticCreationKey,
    },
    UpdateBodyCondition {
        id: NodeId,
        condition: BodyCondition,
    },
    RemoveBodyCondition {
        id: NodeId,
    },
    RestoreSubtree {
        archive: ArchivedSubtree,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArchivedSubtree {
    former_root: NodeId,
    placement: RestorePlacement,
    nodes: Vec<ArchivedNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestorePlacement {
    RuleSetRoot {
        insertion_index: usize,
    },
    Rule {
        parent: RuleSetId,
        insertion_index: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArchivedNode {
    pub old_id: NodeId,
    /// Parent within this archive. The archive root has no internal parent;
    /// its surviving external placement is carried by [`RestorePlacement`].
    pub parent: Option<NodeId>,
    pub key: SemanticCreationKey,
    pub payload: ArchivedNodePayload,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArchivedNodePayload {
    RuleSet { path: RuleSetPath },
    Rule(RuleEditPayload),
    HeaderCondition(HeaderCondition),
    BodyCondition(BodyCondition),
}

impl ArchivedNodePayload {
    pub fn kind(&self) -> WorkspaceNodeKind {
        match self {
            Self::RuleSet { .. } => WorkspaceNodeKind::RuleSet,
            Self::Rule(_) => WorkspaceNodeKind::Rule,
            Self::HeaderCondition(_) => WorkspaceNodeKind::HeaderCondition,
            Self::BodyCondition(_) => WorkspaceNodeKind::BodyCondition,
        }
    }
}

impl ArchivedSubtree {
    pub fn new(
        former_root: NodeId,
        placement: RestorePlacement,
        nodes: Vec<ArchivedNode>,
    ) -> Result<Self, FieldError> {
        let mut old_ids = HashSet::new();
        if nodes.iter().any(|node| !old_ids.insert(node.old_id)) {
            return Err(FieldError::new(
                "archive",
                FieldErrorKind::DuplicateArchivedNode,
            ));
        }
        let Some(root) = nodes.iter().find(|node| node.old_id == former_root) else {
            return Err(FieldError::new(
                "archive",
                FieldErrorKind::MissingArchiveRoot,
            ));
        };
        let valid_placement = matches!(
            (root.payload.kind(), placement),
            (
                WorkspaceNodeKind::RuleSet,
                RestorePlacement::RuleSetRoot { .. }
            ) | (WorkspaceNodeKind::Rule, RestorePlacement::Rule { .. })
        );
        if !valid_placement {
            return Err(FieldError::new(
                "archive",
                FieldErrorKind::InvalidRestorePlacement,
            ));
        }
        for node in &nodes {
            if node.old_id == former_root {
                if node.parent.is_some() {
                    return Err(FieldError::new(
                        "archive",
                        FieldErrorKind::InvalidArchiveTopology,
                    ));
                }
                continue;
            }
            let Some(parent_id) = node.parent else {
                return Err(FieldError::new(
                    "archive",
                    FieldErrorKind::InvalidArchiveTopology,
                ));
            };
            let Some(parent) = nodes.iter().find(|candidate| candidate.old_id == parent_id) else {
                return Err(FieldError::new(
                    "archive",
                    FieldErrorKind::InvalidArchiveTopology,
                ));
            };
            let valid_parent = matches!(
                (parent.payload.kind(), node.payload.kind()),
                (WorkspaceNodeKind::RuleSet, WorkspaceNodeKind::Rule)
                    | (WorkspaceNodeKind::Rule, WorkspaceNodeKind::HeaderCondition)
                    | (WorkspaceNodeKind::Rule, WorkspaceNodeKind::BodyCondition)
            );
            if !valid_parent {
                return Err(FieldError::new(
                    "archive",
                    FieldErrorKind::InvalidArchiveTopology,
                ));
            }
        }
        Ok(Self {
            former_root,
            placement,
            nodes,
        })
    }

    pub fn former_root(&self) -> NodeId {
        self.former_root
    }

    pub fn nodes(&self) -> &[ArchivedNode] {
        &self.nodes
    }

    pub fn placement(&self) -> RestorePlacement {
        self.placement
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditTransaction(Vec<EditIntent>);

impl EditTransaction {
    pub fn new(intents: Vec<EditIntent>) -> Result<Self, FieldError> {
        if intents.is_empty() {
            return Err(FieldError::new(
                "transaction",
                FieldErrorKind::EmptyTransaction,
            ));
        }
        let mut keys = HashSet::new();
        for intent in &intents {
            for key in intent.creation_keys() {
                if !keys.insert(key.as_str()) {
                    return Err(FieldError::new(
                        "creation_key",
                        FieldErrorKind::DuplicateCreationKey,
                    ));
                }
            }
        }
        Ok(Self(intents))
    }

    pub fn intents(&self) -> &[EditIntent] {
        &self.0
    }
}

impl EditIntent {
    fn creation_keys(&self) -> Vec<&SemanticCreationKey> {
        match self {
            Self::AddRuleSet { key, .. }
            | Self::AddHeaderCondition { key, .. }
            | Self::AddBodyCondition { key, .. } => vec![key],
            Self::AddRule { rule, key, .. } => {
                let mut keys = vec![key];
                keys.extend(rule.creation_keys());
                keys
            }
            Self::UpdateRule { rule, .. } => rule.creation_keys().collect(),
            Self::RestoreSubtree { archive } => {
                archive.nodes.iter().map(|node| &node.key).collect()
            }
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EditOutcome {
    pub snapshot: PortSnapshot,
    pub changed_nodes: Vec<NodeId>,
    pub creations: Vec<CreationReceipt>,
    pub rebound_nodes: Vec<NodeRebind>,
    pub unsaved_hint: RuntimeEffect,
}

#[derive(Debug, Clone)]
pub struct ApplyFailure {
    pub diagnostic: Diagnostic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiff {
    pub path: WorkspaceRelativePath,
    pub effect: RuntimeEffect,
}

#[derive(Debug, Clone)]
pub struct PortSnapshot {
    workspace: WorkspaceSnapshot,
    rules: Vec<PortRuleView>,
    dirty_files: Vec<FileDiff>,
    unsaved_hint: RuntimeEffect,
    runtime_pending: RuntimeEffect,
}

impl PortSnapshot {
    fn new(
        workspace: WorkspaceSnapshot,
        rules: Vec<PortRuleView>,
        dirty_files: Vec<FileDiff>,
        unsaved_hint: RuntimeEffect,
        runtime_pending: RuntimeEffect,
    ) -> Self {
        Self {
            workspace,
            rules,
            dirty_files,
            unsaved_hint,
            runtime_pending,
        }
    }

    pub fn workspace(&self) -> &WorkspaceSnapshot {
        &self.workspace
    }

    pub fn rules(&self) -> &[PortRuleView] {
        &self.rules
    }

    pub fn rule(&self, rule_id: NodeId) -> Option<&PortRuleView> {
        self.rules.iter().find(|rule| rule.rule_id == rule_id)
    }

    pub fn dirty_files(&self) -> &[FileDiff] {
        &self.dirty_files
    }

    pub fn unsaved_hint(&self) -> RuntimeEffect {
        self.unsaved_hint
    }

    pub fn runtime_pending(&self) -> RuntimeEffect {
        self.runtime_pending
    }

    /// Exposes the render projection only for cross-crate adapter contract tests.
    #[cfg(feature = "contract-test-support")]
    #[doc(hidden)]
    pub fn contract_test_workspace_mut(&mut self) -> &mut WorkspaceSnapshot {
        &mut self.workspace
    }

    /// Discards canonical rule state and condition identity. Migration-only.
    pub fn into_legacy_workspace(self) -> WorkspaceSnapshot {
        self.workspace
    }
}

#[derive(Debug, Clone)]
pub struct SaveOutcome {
    pub snapshot: PortSnapshot,
    pub written_files: Vec<WorkspaceRelativePath>,
    pub diffs: Vec<FileDiff>,
    pub unsaved_hint: RuntimeEffect,
    pub runtime_pending: RuntimeEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveErrorKind {
    Validation,
    InjectedFailure,
    Io,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveError {
    kind: SaveErrorKind,
    detail: String,
}

impl SaveError {
    pub fn validation(detail: impl Into<String>) -> Self {
        Self::new(SaveErrorKind::Validation, detail)
    }

    pub fn injected_failure(detail: impl Into<String>) -> Self {
        Self::new(SaveErrorKind::InjectedFailure, detail)
    }

    pub fn io(detail: impl Into<String>) -> Self {
        Self::new(SaveErrorKind::Io, detail)
    }

    fn new(kind: SaveErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> SaveErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

#[derive(Debug, Clone)]
pub struct SaveFailure {
    /// Owned post-attempt snapshot, boxed to keep the port's `Result` error
    /// representation small without weakening the mandatory snapshot contract.
    pub snapshot: Box<PortSnapshot>,
    pub written_files: Vec<WorkspaceRelativePath>,
    pub diffs: Vec<FileDiff>,
    pub failed_file: WorkspaceRelativePath,
    pub cause: SaveError,
    pub unsaved_hint: RuntimeEffect,
    pub runtime_pending: RuntimeEffect,
}

pub trait WorkspacePort {
    fn snapshot(&self) -> PortSnapshot;
    fn apply(&mut self, transaction: EditTransaction) -> Result<EditOutcome, ApplyFailure>;
    fn validate(&self) -> ValidationReport;
    fn save(&mut self) -> Result<SaveOutcome, SaveFailure>;
    fn acknowledge_reload(&mut self) -> PortSnapshot;
    fn acknowledge_restart(&mut self) -> PortSnapshot;
}
