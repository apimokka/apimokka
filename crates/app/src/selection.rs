//! UI selection state (MK-021, MK-035).

use apimokka_model::{NodeId, RuleSetId};

/// The 3 workspace navigation destinations (MK-021; Scripts deferred to future).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkspaceTab {
    #[default]
    Routes,
    Trace,
    Settings,
}

/// Bottom drawer modes (MK-032).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerMode { Validation, SaveDiff }

/// Which items in the Routes sidebar are currently selected.
#[derive(Debug, Clone, Default)]
pub struct RouteSelection {
    pub rule_set:   Option<RuleSetId>,
    pub rule:       Option<NodeId>,
    pub file_route: Option<String>,
    pub script:     Option<String>,
}
