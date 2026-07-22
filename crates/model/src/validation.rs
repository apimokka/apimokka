//! Validation, diagnostics, severity.
//!
//! ABDD requires severity to be communicated through text and shape,
//! not colour alone. The UI layer renders an icon glyph plus a textual
//! label ("Error", "Warning", "Info") sourced from `Severity::label()`.

use crate::ids::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "Error",
            Severity::Warning => "Warning",
            Severity::Info => "Info",
        }
    }
    /// Non-colour glyph for the severity. Used together with the text
    /// label so a colour-blind user or one running monochrome still sees
    /// the distinction.
    pub fn glyph(self) -> &'static str {
        match self {
            Severity::Error => "!",
            Severity::Warning => "△",
            Severity::Info => "i",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    /// `None` = workspace-wide issue not anchored to a node.
    pub node_id: Option<NodeId>,
    pub severity: Severity,
    pub message: String,
    /// Optional dotted breadcrumb such as `rules[2].when.url_path`.
    pub location: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn count(&self, sev: Severity) -> usize {
        self.issues.iter().filter(|i| i.severity == sev).count()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub node_id: Option<NodeId>,
    pub severity: Severity,
    pub message: String,
}

/// Per-node validation summary attached inline (e.g. on each rule).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodeValidation {
    pub issues: Vec<ValidationIssue>,
}

impl NodeValidation {
    pub fn worst(&self) -> Option<Severity> {
        let mut worst: Option<Severity> = None;
        for issue in &self.issues {
            worst = Some(match (worst, issue.severity) {
                (Some(Severity::Error), _) | (_, Severity::Error) => Severity::Error,
                (Some(Severity::Warning), _) | (_, Severity::Warning) => Severity::Warning,
                _ => Severity::Info,
            });
        }
        worst
    }
}
