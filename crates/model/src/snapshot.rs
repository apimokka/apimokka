//! The render-ready workspace snapshot.
//!
//! Exposed as the render projection inside
//! [`crate::workspace_port::PortSnapshot`] and consumed by the UI to render
//! the route tree, rule list, and rule editor. Canonical rule and condition
//! state lives beside it in the port snapshot; this projection is lossy and
//! must not be used to reconstruct canonical history values. Selection state
//! lives in the app and is anchored to session-scoped `NodeId`s.

use crate::ids::{NodeId, RuleSetId};
use crate::node::{ConfigFileView, FileNodeView};
use crate::respond::RespondPayload;
use crate::rule::{BodyConditionPayload, HeaderConditionPayload, RulePayload, UrlPathOp};
use crate::settings::RootSettings;
use crate::validation::{Diagnostic, NodeValidation};

/// Top-level workspace identity, immutable for one admitted session.
#[derive(Debug, Clone)]
pub struct WorkspaceMeta {
    pub name: String,
    pub path: String,
}

/// One rule within a rule set, in render-ready form. The `payload` is a
/// full editable copy; the snapshot is read-only from the GUI's view.
#[derive(Debug, Clone)]
pub struct RuleView {
    pub id: NodeId,
    pub payload: RulePayload,
    pub validation: NodeValidation,
    /// Set when the user has selected a trace event matched by this rule.
    pub matched_by_latest_trace: bool,
}

impl RuleView {
    /// Equivalent of the engine's `WhenView::summary()`. Example:
    /// `POST /api/orders +headers(2) +body(1)`.
    pub fn summary(&self) -> String {
        let method = if self.payload.method.is_empty() {
            "ANY".to_string()
        } else {
            self.payload.method.clone()
        };
        let path = if self.payload.url_path.is_empty() {
            "(any path)".to_string()
        } else {
            self.payload.url_path.clone()
        };
        let h = self.payload.headers.len();
        let b = self.payload.body.len();

        let mut s = format!("{method} {path}");
        if h > 0 {
            s.push_str(&format!(" +headers({h})"));
        }
        if b > 0 {
            s.push_str(&format!(" +body({b})"));
        }
        s
    }
}

#[derive(Debug, Clone)]
pub struct RuleSetView {
    pub id: RuleSetId,
    /// File-relative display name (e.g. `rules/main.toml`).
    pub file: ConfigFileView,
    pub rules: Vec<RuleView>,
    pub validation: NodeValidation,
}

#[derive(Debug, Clone)]
pub struct WorkspaceSnapshot {
    pub meta: WorkspaceMeta,
    pub root_settings: RootSettings,
    pub rule_sets: Vec<RuleSetView>,
    /// Files in the fallback responses directory. Each carries a route
    /// hint when applicable (`users.json` → `/users`).
    pub fallback_files: Vec<FileNodeView>,
    /// `.rhai` middleware scripts found under the middleware directory.
    pub middleware_scripts: Vec<ConfigFileView>,
    pub diagnostics: Vec<Diagnostic>,
}

impl WorkspaceSnapshot {
    /// Locate a rule by id across all rule sets.
    pub fn find_rule(&self, id: NodeId) -> Option<(&RuleSetView, &RuleView)> {
        for rs in &self.rule_sets {
            if let Some(r) = rs.rules.iter().find(|r| r.id == id) {
                return Some((rs, r));
            }
        }
        None
    }

    pub fn find_rule_mut(&mut self, id: NodeId) -> Option<&mut RuleView> {
        for rs in &mut self.rule_sets {
            if let Some(r) = rs.rules.iter_mut().find(|r| r.id == id) {
                return Some(r);
            }
        }
        None
    }

    pub fn find_rule_set(&self, id: RuleSetId) -> Option<&RuleSetView> {
        self.rule_sets.iter().find(|rs| rs.id == id)
    }

    pub fn find_rule_set_mut(&mut self, id: RuleSetId) -> Option<&mut RuleSetView> {
        self.rule_sets.iter_mut().find(|rs| rs.id == id)
    }

    /// Sum of dirty markers across rule-set files.
    pub fn dirty_file_count(&self) -> usize {
        self.rule_sets.iter().filter(|rs| rs.file.dirty).count()
    }
}

// Re-export so module consumers can build payloads without an extra import.
pub use crate::respond::RespondMode;

// Suppress unused-import warnings: re-export is part of the module surface.
#[allow(dead_code)]
fn _retain_payload_imports(
    _r: RespondPayload,
    _h: HeaderConditionPayload,
    _b: BodyConditionPayload,
    _u: UrlPathOp,
) {
}
