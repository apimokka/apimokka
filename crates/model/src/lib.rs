//! # apimokka-model
//!
//! UI-facing workspace, rendering, validation, and trace data types.
//!
//! This crate has **no UI dependency**. It exists so the GUI layer
//! (`apimokka`) and test fixtures can share a typed local contract while the
//! mockup remains decoupled from a live engine integration.
//!
//! [`workspace_port`] is the authoritative application boundary adopted by
//! RFC MK-053. It explicitly maps UI concepts to the documented apimock-rs
//! 5.10.1 semantics; it is not source- or binary-isomorphic with an engine
//! crate. Older render and prototype types remain for the mock UI and must not
//! be treated as engine commands or reconstructed canonical state.

pub mod audience;
pub mod edit_command;
pub mod friendly_error;
pub mod ids;
pub mod mock;
pub mod node;
pub mod respond;
pub mod rule;
pub mod save;
pub mod settings;
pub mod snapshot;
pub mod trace;
pub mod validation;
pub mod workspace_port;

pub use audience::AudienceMode;
pub use edit_command::{ApplyResult, EditCommand, EditValue, RootSettingKey};
pub use friendly_error::FriendlyProblem;
pub use ids::{NodeId, RuleSetId};
pub use node::{
    ConfigFileKind, ConfigFileView, ConfigNodeView, FileNodeKind, FileNodeView, NodeKind,
};
pub use respond::RespondPayload;
pub use rule::{
    BodyConditionPayload, BodyOp, HeaderConditionPayload, HeaderOp, RulePayload, UrlPathOp,
};
pub use save::{DiffItem, DiffKind, ReloadHint, SaveResult};
pub use settings::{RootSettings, Strategy};
pub use snapshot::{RuleSetView, RuleView, WorkspaceMeta, WorkspaceSnapshot};
pub use trace::{MatchTraceEvent, RequestSummary, TraceOutcome};
pub use validation::{Diagnostic, NodeValidation, Severity, ValidationIssue, ValidationReport};
pub use workspace_port::{
    ApplyFailure, ArchivedNode, ArchivedNodePayload, ArchivedSubtree, BodyCondition,
    CollectionEdit, ConditionEdit, ConditionWithId, CreationReceipt, EditIntent, EditOutcome,
    EditTransaction, FieldError, FileDiff, HeaderCondition, MemoryWorkspace, NodeRebind,
    PortRuleView, PortSnapshot, RespondDefinition, ResponseMode, RestorePlacement, RootSettingEdit,
    RuleConditionsView, RuleEditPayload, RuleMatch, RuleSetPath, RuntimeEffect, SaveError,
    SaveErrorKind, SaveFailure, SaveOutcome, SemanticCreationKey, WorkspaceEditValue,
    WorkspaceNodeKind, WorkspacePort, WorkspaceRelativePath, WorkspaceRootKey,
};
