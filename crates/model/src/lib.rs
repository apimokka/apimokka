//! # apimokka-model
//!
//! Pure data types mirroring the `apimock-rs` GUI integration surface
//! (Workspace, Snapshot, EditCommand, RulePayload, RespondPayload,
//! ValidationReport, MatchTraceEvent, …).
//!
//! This crate has **no UI dependency**. It exists so the GUI layer
//! (`apimokka-app`) and any test fixtures can talk in the same vocabulary
//! the engine eventually consumes, while the mockup remains decoupled
//! from a live engine integration. See `apimock-rs-GUI-INTEGRATION-
//! REFERENCE-v5.10.1.md` for the engine surface this mirrors.
//!
//! All types are deliberately read-mostly. Mockup `apply()` calls in the
//! sibling `apimokka-app` crate update in-memory copies so the
//! snapshot-apply loop (RFC-MK-003) is demonstrable without a real
//! engine connection.

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
    CollectionEdit, ConditionWithId, CreationReceipt, EditIntent, EditOutcome, EditTransaction,
    FieldError, HeaderCondition, NodeRebind, RespondDefinition, ResponseMode, RestorePlacement,
    RootSettingEdit, RuleConditionsView, RuleEditPayload, RuleMatch, RuleSetPath, RuntimeEffect,
    SaveError, SaveErrorKind, SaveFailure, SaveOutcome, SemanticCreationKey, WorkspaceEditValue,
    WorkspaceNodeKind, WorkspacePort, WorkspaceRelativePath, WorkspaceRootKey,
};
