//! Tree node descriptions for the route sidebar.
//!
//! Local render-oriented types informed by the integration reference. They
//! contain only fields used by the mockup and are not an engine-isomorphic
//! schema.

use crate::ids::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFileKind {
    /// `apimock.toml`
    Root,
    /// A rule-set TOML file
    RuleSet,
    /// A `.rhai` middleware script
    Middleware,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Root,
    Listener,
    Log,
    Service,
    FallbackRespondDir,
    RuleSet,
    Rule,
    Respond,
    Middleware,
    FileTreeView,
}

#[derive(Debug, Clone)]
pub struct ConfigFileView {
    pub kind: ConfigFileKind,
    /// Workspace-relative path string, used directly as the display label.
    pub path: String,
    pub dirty: bool,
}

#[derive(Debug, Clone)]
pub struct ConfigNodeView {
    pub id: NodeId,
    pub kind: NodeKind,
    pub display_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileNodeKind {
    File,
    Directory,
}

/// One entry in the fallback file browser (under `responses/`, etc.).
#[derive(Debug, Clone)]
pub struct FileNodeView {
    pub name: String,
    pub path: String,
    pub kind: FileNodeKind,
    /// E.g. `/users` for `users.json`. Shown alongside the file name so
    /// users see that file fallback is a real routing layer.
    pub route_hint: Option<String>,
}
