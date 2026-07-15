//! Stable identifiers anchored on engine `NodeId` semantics.
//!
//! Per the integration reference, `NodeId` is a `Uuid` wrapper that is
//! stable across snapshots/apply for one `Workspace` lifetime and not
//! persisted to disk. The mockup honours both properties: ids are
//! generated freshly at workspace load (here, mock load).

use uuid::Uuid;

/// A node identifier scoped to one workspace session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        NodeId(Uuid::new_v4())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // A short, stable form is plenty for UI hover/debug labels.
        let s = self.0.to_string();
        f.write_str(&s[..8])
    }
}

/// A logical id identifying which rule set a rule belongs to. Kept as a
/// thin newtype around `NodeId` so the GUI can distinguish "select a rule
/// set" from "select a rule" without a runtime kind check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuleSetId(pub NodeId);
