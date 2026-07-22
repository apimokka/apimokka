//! UI selection state (MK-021, MK-035).

use apimokka_model::{NodeId, RuleSetId, WorkspaceSnapshot};

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
pub enum DrawerMode {
    Validation,
    SaveDiff,
}

/// Which items in the Routes sidebar are currently selected.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RouteSelection {
    pub rule_set: Option<RuleSetId>,
    pub rule: Option<NodeId>,
    pub file_route: Option<String>,
    pub script: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RouteTarget {
    Rule {
        id: NodeId,
        captured_parent: Option<RuleSetId>,
    },
    RuleSet(RuleSetId),
    Fallback(String),
    Script(String),
    None,
}

impl RouteSelection {
    pub(crate) fn capture(&self, snapshot: &WorkspaceSnapshot) -> RouteTarget {
        if let Some(id) = self.rule {
            return RouteTarget::Rule {
                id,
                captured_parent: snapshot.find_rule(id).map(|(set, _)| set.id),
            };
        }
        if let Some(id) = self.rule_set {
            return RouteTarget::RuleSet(id);
        }
        if let Some(path) = &self.file_route {
            return RouteTarget::Fallback(path.clone());
        }
        if let Some(path) = &self.script {
            return RouteTarget::Script(path.clone());
        }
        RouteTarget::None
    }

    pub(crate) fn reconcile(&mut self, snapshot: &WorkspaceSnapshot, target: RouteTarget) {
        *self = match target {
            RouteTarget::Rule {
                id,
                captured_parent,
            } => {
                if let Some((parent, _)) = snapshot.find_rule(id) {
                    Self {
                        rule_set: Some(parent.id),
                        rule: Some(id),
                        file_route: None,
                        script: None,
                    }
                } else if let Some(parent) =
                    captured_parent.filter(|parent| snapshot.find_rule_set(*parent).is_some())
                {
                    Self {
                        rule_set: Some(parent),
                        ..Self::default()
                    }
                } else {
                    Self::default()
                }
            }
            RouteTarget::RuleSet(id) if snapshot.find_rule_set(id).is_some() => Self {
                rule_set: Some(id),
                ..Self::default()
            },
            RouteTarget::Fallback(path)
                if snapshot.fallback_files.iter().any(|file| file.path == path) =>
            {
                Self {
                    file_route: Some(path),
                    ..Self::default()
                }
            }
            RouteTarget::Script(path)
                if snapshot
                    .middleware_scripts
                    .iter()
                    .any(|script| script.path == path) =>
            {
                Self {
                    script: Some(path),
                    ..Self::default()
                }
            }
            _ => Self::default(),
        };
    }

    pub(crate) fn select_rule_set(&mut self, id: RuleSetId) {
        *self = Self {
            rule_set: Some(id),
            ..Self::default()
        };
    }

    pub(crate) fn select_rule(&mut self, id: NodeId, parent: RuleSetId) {
        *self = Self {
            rule_set: Some(parent),
            rule: Some(id),
            ..Self::default()
        };
    }

    pub(crate) fn select_fallback(&mut self, path: String) {
        *self = Self {
            file_route: Some(path),
            ..Self::default()
        };
    }

    pub(crate) fn select_script(&mut self, path: String) {
        *self = Self {
            script: Some(path),
            ..Self::default()
        };
    }
}

#[cfg(test)]
#[path = "selection/tests.rs"]
mod tests;
