//! Legacy mockup command shapes retained for render/prototype compatibility.
//!
//! Configuration mutation now uses [`crate::workspace_port::EditIntent`] and
//! [`crate::workspace_port::WorkspacePort`]. These older index-addressed
//! shapes are not the MK-053 port contract and do not claim a direct mapping to
//! a future engine API.

use crate::ids::{NodeId, RuleSetId};
use crate::respond::RespondPayload;
use crate::rule::{BodyConditionPayload, HeaderConditionPayload, RulePayload};
use crate::settings::RootSettings;
use crate::validation::Diagnostic;

#[derive(Debug, Clone)]
pub enum EditCommand {
    // --- Rule sets -----------------------------------------------
    AddRuleSet {
        path: String,
    },
    RemoveRuleSet {
        id: RuleSetId,
    },

    // --- Rules ---------------------------------------------------
    AddRule {
        parent: RuleSetId,
        rule: RulePayload,
    },
    UpdateRule {
        id: NodeId,
        rule: RulePayload,
    },
    DeleteRule {
        id: NodeId,
    },
    MoveRule {
        id: NodeId,
        new_index: usize,
    },

    // --- Respond -------------------------------------------------
    UpdateRespond {
        id: NodeId,
        respond: RespondPayload,
    },

    // --- Root settings ------------------------------------------
    UpdateRootSetting {
        key: RootSettingKey,
        value: EditValue,
    },

    // --- Per-condition (RFC 016) --------------------------------
    AddHeaderCondition {
        rule_id: NodeId,
        condition: HeaderConditionPayload,
    },
    UpdateHeaderCondition {
        rule_id: NodeId,
        index: usize,
        condition: HeaderConditionPayload,
    },
    RemoveHeaderCondition {
        rule_id: NodeId,
        index: usize,
    },

    AddBodyCondition {
        rule_id: NodeId,
        condition: BodyConditionPayload,
    },
    UpdateBodyCondition {
        rule_id: NodeId,
        index: usize,
        condition: BodyConditionPayload,
    },
    RemoveBodyCondition {
        rule_id: NodeId,
        index: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootSettingKey {
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
    TraceEnabled,
    TraceTransport,
    TraceUdsPath,
    TraceTcpAddr,
    TraceQueueSize,
}

impl RootSettingKey {
    /// External design § 5.1: each setting communicates whether saving it
    /// will require a runtime reload or a full restart.
    pub fn requires_restart(self) -> bool {
        matches!(
            self,
            RootSettingKey::ListenerIpAddress
                | RootSettingKey::ListenerPort
                | RootSettingKey::TlsEnabled
                | RootSettingKey::TlsCertFile
                | RootSettingKey::TlsKeyFile
                | RootSettingKey::LogFile
        )
    }
}

#[derive(Debug, Clone)]
pub enum EditValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    StringList(Vec<String>),
    /// String label of an enum variant (e.g. a Strategy name).
    Enum(String),
}

#[derive(Debug, Clone, Default)]
pub struct ApplyResult {
    /// NodeIds whose render is no longer valid. Used by the UI to scope
    /// re-render in production; the mockup always re-renders.
    pub changed_nodes: Vec<NodeId>,
    /// Whether the running server needs a config reload to see this
    /// change. Restart is tracked separately at the settings level.
    pub requires_reload: bool,
    pub requires_restart: bool,
    pub diagnostics: Vec<Diagnostic>,
}

// Re-export so consumers don't need to remember which submodule owns these.
pub use crate::settings::Strategy;
#[allow(dead_code)]
fn _retain_payload_imports(_s: RootSettings) {}
