//! Save-flow result types.
//!
//! Legacy Save Diff presentation types retained by the mockup.
//!
//! The authoritative MK-053 save boundary is
//! [`crate::workspace_port::WorkspacePort::save`], with `SaveOutcome`,
//! `SaveFailure`, `FileDiff`, and `RuntimeEffect`. The app owns the typed
//! Global Save report that combines workspace and fallback attempts.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    Created,
    Modified,
    Removed,
}

impl DiffKind {
    pub fn label(self) -> &'static str {
        match self {
            DiffKind::Created => "Created",
            DiffKind::Modified => "Modified",
            DiffKind::Removed => "Removed",
        }
    }
    pub fn glyph(self) -> &'static str {
        match self {
            DiffKind::Created => "+",
            DiffKind::Modified => "~",
            DiffKind::Removed => "-",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiffItem {
    pub kind: DiffKind,
    pub path: String,
}

/// Whether a saved change implies the engine needs a reload or a full
/// restart. Restart wins over reload (external design § 5.1: "If both
/// reload and restart are required, restart wins").
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReloadHint {
    pub requires_reload: bool,
    pub requires_restart: bool,
}

impl ReloadHint {
    pub fn label(self) -> &'static str {
        match (self.requires_restart, self.requires_reload) {
            (true, _) => "Restart required",
            (false, true) => "Reload pending",
            _ => "No runtime action",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SaveResult {
    pub written_files: Vec<String>,
    pub reload_hint: ReloadHint,
    pub diffs: Vec<DiffItem>,
}
