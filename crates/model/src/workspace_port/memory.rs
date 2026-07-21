use std::collections::{HashMap, HashSet};

use super::mapping::{
    project_body_condition, project_header_condition, project_response, project_rule_match,
};
use super::*;
use crate::node::{ConfigFileKind, ConfigFileView};
use crate::respond::RespondMode;
use crate::rule::{BodyConditionPayload, HeaderConditionPayload, RulePayload};
use crate::settings::{RootSettings, Strategy};
use crate::snapshot::{RuleSetView, RuleView, WorkspaceMeta};
use crate::validation::{NodeValidation, Severity, ValidationIssue};

#[derive(Debug, Clone)]
pub struct MemoryWorkspace {
    state: MemoryState,
    saved_root: RootBaseline,
    saved_rule_sets: Vec<SavedRuleSet>,
    forced_dirty: HashSet<WorkspaceRelativePath>,
    runtime_pending: RuntimeEffect,
    injected_save_failure: Option<WorkspaceRelativePath>,
}

#[derive(Debug, Clone)]
struct MemoryState {
    meta: WorkspaceMeta,
    root_settings: RootSettings,
    rule_sets: Vec<MemoryRuleSet>,
    fallback_files: Vec<crate::node::FileNodeView>,
    middleware_scripts: Vec<ConfigFileView>,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
struct MemoryRuleSet {
    id: RuleSetId,
    path: RuleSetPath,
    rules: Vec<MemoryRule>,
    validation: NodeValidation,
}

#[derive(Debug, Clone)]
struct MemoryRule {
    id: NodeId,
    rule_match: RuleMatch,
    headers: Vec<ConditionWithId<HeaderCondition>>,
    body: Vec<ConditionWithId<BodyCondition>>,
    respond: RespondDefinition,
    validation: NodeValidation,
    matched_by_latest_trace: bool,
    // Migration-only legacy render preservation. These ReferenceGap values
    // have no port mutation and do not independently participate in effects.
    weight: Option<u32>,
    priority: Option<i32>,
}

type HeaderConditions = Vec<ConditionWithId<HeaderCondition>>;
type BodyConditions = Vec<ConditionWithId<BodyCondition>>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RootBaseline {
    listener_ip: String,
    listener_port: u16,
    tls_enabled: bool,
    tls_cert_file: String,
    tls_key_file: String,
    log_level: String,
    log_file: String,
    log_format: String,
    fallback_respond_dir: String,
    strategy: Strategy,
    file_tree_show_hidden: bool,
    file_tree_builtin_excludes: bool,
    file_tree_extra_excludes: Vec<String>,
    file_tree_include: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct SavedRuleSet {
    path: RuleSetPath,
    rules: Vec<SavedRule>,
}

#[derive(Debug, Clone, PartialEq)]
struct SavedRule {
    rule_match: RuleMatch,
    headers: Vec<HeaderCondition>,
    body: Vec<BodyCondition>,
    respond: RespondDefinition,
    weight: Option<u32>,
    priority: Option<i32>,
}

impl MemoryWorkspace {
    /// Imports a legacy render snapshot whose editable fields already satisfy
    /// the canonical MK-053 mapping contract. Invalid legacy draft values are
    /// rejected with a [`FieldError`] rather than admitted into canonical state.
    /// Legacy delay zero has no absent/present bit and is normalized once to
    /// canonical `Some(0)` during this import.
    pub fn new(workspace: WorkspaceSnapshot) -> Result<Self, FieldError> {
        validate_root_settings(&workspace.root_settings)?;
        let mut used_ids = HashSet::new();
        let mut paths = HashSet::new();
        let mut rule_sets = Vec::with_capacity(workspace.rule_sets.len());
        let mut forced_dirty = HashSet::new();
        for rule_set in workspace.rule_sets {
            if rule_set.file.kind != ConfigFileKind::RuleSet {
                return Err(FieldError::new(
                    "rule_set_kind",
                    FieldErrorKind::ValueTypeMismatch,
                ));
            }
            if !used_ids.insert(rule_set.id.0) {
                return Err(FieldError::new(
                    "workspace",
                    FieldErrorKind::DuplicateNodeId,
                ));
            }
            let path = parse_rule_set_path(&rule_set.file.path)?;
            if rule_set.file.dirty {
                forced_dirty.insert(path.as_relative().clone());
            }
            if !paths.insert(path.clone()) {
                return Err(FieldError::new(
                    "rule_set_path",
                    FieldErrorKind::DuplicateRuleSetPath,
                ));
            }
            let mut rules = Vec::with_capacity(rule_set.rules.len());
            for rule in rule_set.rules {
                if !used_ids.insert(rule.id) {
                    return Err(FieldError::new(
                        "workspace",
                        FieldErrorKind::DuplicateNodeId,
                    ));
                }
                rules.push(MemoryRule::from_legacy(rule, &mut used_ids)?);
            }
            rule_sets.push(MemoryRuleSet {
                id: rule_set.id,
                path,
                rules,
                validation: rule_set.validation,
            });
        }
        let state = MemoryState {
            meta: workspace.meta,
            root_settings: workspace.root_settings,
            rule_sets,
            fallback_files: workspace.fallback_files,
            middleware_scripts: workspace.middleware_scripts,
            diagnostics: workspace.diagnostics,
        };
        Ok(Self {
            saved_root: RootBaseline::from(&state.root_settings),
            saved_rule_sets: state.rule_sets.iter().map(SavedRuleSet::from).collect(),
            forced_dirty,
            state,
            runtime_pending: RuntimeEffect::None,
            injected_save_failure: None,
        })
    }

    /// Injects one failure at the named dirty file. This is a memory-adapter
    /// test control and deliberately is not part of [`WorkspacePort`].
    pub fn inject_save_failure(&mut self, path: WorkspaceRelativePath) -> Result<(), FieldError> {
        if !self.dirty_files().iter().any(|diff| diff.path == path) {
            return Err(FieldError::new(
                "save_failure_path",
                FieldErrorKind::SaveTargetNotDirty,
            ));
        }
        self.injected_save_failure = Some(path);
        Ok(())
    }

    fn dirty_files(&self) -> Vec<FileDiff> {
        let mut dirty = Vec::new();
        let root_effect = root_effect(&self.state.root_settings, &self.saved_root);
        if root_effect != RuntimeEffect::None {
            dirty.push(FileDiff {
                path: parse_workspace_relative_path("path", "apimock.toml")
                    .expect("static root path is valid"),
                effect: root_effect,
            });
        }

        let mut paths = self
            .state
            .rule_sets
            .iter()
            .map(|rule_set| rule_set.path.clone())
            .chain(
                self.saved_rule_sets
                    .iter()
                    .map(|rule_set| rule_set.path.clone()),
            )
            .collect::<Vec<_>>();
        paths.sort_by(|left, right| left.as_relative().cmp(right.as_relative()));
        paths.dedup();
        for path in paths {
            let current = self
                .state
                .rule_sets
                .iter()
                .find(|rule_set| rule_set.path == path)
                .map(SavedRuleSet::from);
            let saved = self
                .saved_rule_sets
                .iter()
                .find(|rule_set| rule_set.path == path);
            if current.as_ref() != saved {
                dirty.push(FileDiff {
                    path: path.as_relative().clone(),
                    effect: RuntimeEffect::Reload,
                });
            }
        }
        for path in &self.forced_dirty {
            if !dirty.iter().any(|diff| &diff.path == path) {
                dirty.push(FileDiff {
                    path: path.clone(),
                    effect: RuntimeEffect::Reload,
                });
            }
        }
        dirty.sort_by(|left, right| left.path.cmp(&right.path));
        dirty
    }

    fn snapshot_inner(&self) -> PortSnapshot {
        let dirty_files = self.dirty_files();
        let unsaved_hint = dirty_files
            .iter()
            .fold(RuntimeEffect::None, |effect, diff| {
                effect.combine(diff.effect)
            });
        let dirty_paths = dirty_files
            .iter()
            .map(|diff| diff.path.as_str())
            .collect::<HashSet<_>>();
        let mut port_rules = Vec::new();
        let rule_sets = self
            .state
            .rule_sets
            .iter()
            .map(|rule_set| {
                let rules = rule_set
                    .rules
                    .iter()
                    .map(|rule| {
                        port_rules.push(rule.port_view());
                        rule.legacy_view()
                    })
                    .collect();
                RuleSetView {
                    id: rule_set.id,
                    file: ConfigFileView {
                        kind: ConfigFileKind::RuleSet,
                        path: rule_set.path.to_string(),
                        dirty: dirty_paths.contains(rule_set.path.as_relative().as_str()),
                    },
                    rules,
                    validation: rule_set.validation.clone(),
                }
            })
            .collect();
        PortSnapshot::new(
            WorkspaceSnapshot {
                meta: self.state.meta.clone(),
                root_settings: self.state.root_settings.clone(),
                rule_sets,
                fallback_files: self.state.fallback_files.clone(),
                middleware_scripts: self.state.middleware_scripts.clone(),
                diagnostics: self.state.diagnostics.clone(),
            },
            port_rules,
            dirty_files,
            unsaved_hint,
            self.runtime_pending,
        )
    }

    fn apply_intent(
        candidate: &mut MemoryState,
        intent: EditIntent,
        changed: &mut Vec<NodeId>,
        creations: &mut Vec<CreationReceipt>,
        rebound: &mut Vec<NodeRebind>,
        used_ids: &mut HashSet<NodeId>,
    ) -> Result<(), ApplyFailure> {
        match intent {
            EditIntent::AddRuleSet { path, key } => {
                if candidate
                    .rule_sets
                    .iter()
                    .any(|rule_set| rule_set.path == path)
                {
                    return fail(None, "rule-set path already exists");
                }
                let id = RuleSetId(fresh_id(used_ids));
                candidate.rule_sets.push(MemoryRuleSet {
                    id,
                    path,
                    rules: Vec::new(),
                    validation: NodeValidation::default(),
                });
                changed.push(id.0);
                creations.push(receipt(key, WorkspaceNodeKind::RuleSet, None, id.0));
            }
            EditIntent::RemoveRuleSet { id } => {
                let index = candidate
                    .rule_sets
                    .iter()
                    .position(|rule_set| rule_set.id == id)
                    .ok_or_else(|| failure(Some(id.0), "rule set does not exist"))?;
                let removed = candidate.rule_sets.remove(index);
                changed.push(id.0);
                for rule in removed.rules {
                    changed.push(rule.id);
                    changed.extend(rule.headers.into_iter().map(|value| value.id));
                    changed.extend(rule.body.into_iter().map(|value| value.id));
                }
            }
            EditIntent::AddRule {
                parent,
                insertion_index,
                rule,
                key,
            } => {
                let rule_set_index = candidate
                    .rule_sets
                    .iter()
                    .position(|rule_set| rule_set.id == parent)
                    .ok_or_else(|| failure(Some(parent.0), "parent rule set does not exist"))?;
                if insertion_index > candidate.rule_sets[rule_set_index].rules.len() {
                    return fail(Some(parent.0), "rule insertion index is out of range");
                }
                let RuleEditPayload {
                    rule_match,
                    headers,
                    body,
                    respond,
                } = rule;
                let id = fresh_id(used_ids);
                creations.push(receipt(key, WorkspaceNodeKind::Rule, Some(parent.0), id));
                let headers = edit_condition_collection(
                    headers,
                    &[],
                    id,
                    WorkspaceNodeKind::HeaderCondition,
                    false,
                    used_ids,
                    creations,
                )?
                .unwrap_or_default();
                let body = edit_condition_collection(
                    body,
                    &[],
                    id,
                    WorkspaceNodeKind::BodyCondition,
                    false,
                    used_ids,
                    creations,
                )?
                .unwrap_or_default();
                let condition_ids = headers
                    .iter()
                    .map(|value| value.id)
                    .chain(body.iter().map(|value| value.id))
                    .collect::<Vec<_>>();
                let new_rule = MemoryRule::new(id, rule_match, headers, body, respond);
                candidate.rule_sets[rule_set_index]
                    .rules
                    .insert(insertion_index, new_rule);
                changed.extend([parent.0, id]);
                changed.extend(condition_ids);
            }
            EditIntent::UpdateRule { id, rule } => {
                let current = find_rule_mut(candidate, id)
                    .ok_or_else(|| failure(Some(id), "rule does not exist"))?;
                let current_headers = current.headers.clone();
                let current_body = current.body.clone();
                let RuleEditPayload {
                    rule_match,
                    headers,
                    body,
                    respond,
                } = rule;
                let changed_header_ids = changed_condition_ids(&headers, &current_headers);
                let changed_body_ids = changed_condition_ids(&body, &current_body);
                let headers = edit_condition_collection(
                    headers,
                    &current_headers,
                    id,
                    WorkspaceNodeKind::HeaderCondition,
                    true,
                    used_ids,
                    creations,
                )?;
                let body = edit_condition_collection(
                    body,
                    &current_body,
                    id,
                    WorkspaceNodeKind::BodyCondition,
                    true,
                    used_ids,
                    creations,
                )?;
                let target = find_rule_mut(candidate, id)
                    .expect("rule located before replacement remains present");
                target.rule_match = rule_match;
                target.respond = respond;
                if let Some(headers) = headers {
                    target.headers = headers;
                }
                if let Some(body) = body {
                    target.body = body;
                }
                changed.push(id);
                changed.extend(changed_header_ids);
                changed.extend(changed_body_ids);
                changed.extend(
                    target
                        .headers
                        .iter()
                        .filter(|value| !current_headers.iter().any(|old| old.id == value.id))
                        .map(|value| value.id),
                );
                changed.extend(
                    target
                        .body
                        .iter()
                        .filter(|value| !current_body.iter().any(|old| old.id == value.id))
                        .map(|value| value.id),
                );
            }
            EditIntent::DeleteRule { id } => {
                let (rule_set_index, rule_index) = find_rule_position(candidate, id)
                    .ok_or_else(|| failure(Some(id), "rule does not exist"))?;
                let parent = candidate.rule_sets[rule_set_index].id.0;
                let removed = candidate.rule_sets[rule_set_index].rules.remove(rule_index);
                changed.extend([parent, id]);
                changed.extend(removed.headers.into_iter().map(|value| value.id));
                changed.extend(removed.body.into_iter().map(|value| value.id));
            }
            EditIntent::MoveRule { id, new_index } => {
                let (rule_set_index, rule_index) = find_rule_position(candidate, id)
                    .ok_or_else(|| failure(Some(id), "rule does not exist"))?;
                let rules = &mut candidate.rule_sets[rule_set_index].rules;
                if new_index >= rules.len() {
                    return fail(Some(id), "rule insertion index is out of range");
                }
                let rule = rules.remove(rule_index);
                rules.insert(new_index, rule);
                changed.push(id);
            }
            EditIntent::UpdateRespond { id, respond } => {
                let rule = find_rule_mut(candidate, id)
                    .ok_or_else(|| failure(Some(id), "rule does not exist"))?;
                rule.respond = respond;
                changed.push(id);
            }
            EditIntent::UpdateRootSetting(edit) => {
                apply_root_setting(&mut candidate.root_settings, edit);
            }
            EditIntent::AddHeaderCondition {
                rule_id,
                condition,
                key,
            } => {
                let id = fresh_id(used_ids);
                let rule = find_rule_mut(candidate, rule_id)
                    .ok_or_else(|| failure(Some(rule_id), "rule does not exist"))?;
                rule.headers.push(ConditionWithId { id, condition });
                changed.extend([rule_id, id]);
                creations.push(receipt(
                    key,
                    WorkspaceNodeKind::HeaderCondition,
                    Some(rule_id),
                    id,
                ));
            }
            EditIntent::UpdateHeaderCondition { id, condition } => {
                let (rule_id, target) = find_header_mut(candidate, id)
                    .ok_or_else(|| failure(Some(id), "header condition does not exist"))?;
                target.condition = condition;
                changed.extend([rule_id, id]);
            }
            EditIntent::RemoveHeaderCondition { id } => {
                let (rule_id, index) = find_header_position(candidate, id)
                    .ok_or_else(|| failure(Some(id), "header condition does not exist"))?;
                find_rule_mut(candidate, rule_id)
                    .expect("located rule exists")
                    .headers
                    .remove(index);
                changed.extend([rule_id, id]);
            }
            EditIntent::AddBodyCondition {
                rule_id,
                condition,
                key,
            } => {
                let id = fresh_id(used_ids);
                let rule = find_rule_mut(candidate, rule_id)
                    .ok_or_else(|| failure(Some(rule_id), "rule does not exist"))?;
                rule.body.push(ConditionWithId { id, condition });
                changed.extend([rule_id, id]);
                creations.push(receipt(
                    key,
                    WorkspaceNodeKind::BodyCondition,
                    Some(rule_id),
                    id,
                ));
            }
            EditIntent::UpdateBodyCondition { id, condition } => {
                let (rule_id, target) = find_body_mut(candidate, id)
                    .ok_or_else(|| failure(Some(id), "body condition does not exist"))?;
                target.condition = condition;
                changed.extend([rule_id, id]);
            }
            EditIntent::RemoveBodyCondition { id } => {
                let (rule_id, index) = find_body_position(candidate, id)
                    .ok_or_else(|| failure(Some(id), "body condition does not exist"))?;
                find_rule_mut(candidate, rule_id)
                    .expect("located rule exists")
                    .body
                    .remove(index);
                changed.extend([rule_id, id]);
            }
            EditIntent::RestoreSubtree { archive } => {
                restore_subtree(candidate, archive, changed, creations, rebound, used_ids)?;
            }
        }
        Ok(())
    }

    fn mark_saved(&mut self, path: &WorkspaceRelativePath) {
        self.forced_dirty.remove(path);
        if path.as_str() == "apimock.toml" {
            self.saved_root = RootBaseline::from(&self.state.root_settings);
            return;
        }
        self.saved_rule_sets
            .retain(|rule_set| rule_set.path.as_relative() != path);
        if let Some(current) = self
            .state
            .rule_sets
            .iter()
            .find(|rule_set| rule_set.path.as_relative() == path)
        {
            self.saved_rule_sets.push(SavedRuleSet::from(current));
            self.saved_rule_sets
                .sort_by(|left, right| left.path.as_relative().cmp(right.path.as_relative()));
        }
    }
}

impl WorkspacePort for MemoryWorkspace {
    fn snapshot(&self) -> PortSnapshot {
        self.snapshot_inner()
    }

    fn apply(&mut self, transaction: EditTransaction) -> Result<EditOutcome, ApplyFailure> {
        let mut candidate = self.state.clone();
        let mut used_ids = all_ids(&candidate);
        let mut changed = Vec::new();
        let mut creations = Vec::new();
        let mut rebound = Vec::new();
        for intent in transaction.0 {
            Self::apply_intent(
                &mut candidate,
                intent,
                &mut changed,
                &mut creations,
                &mut rebound,
                &mut used_ids,
            )?;
        }
        refresh_validation(&mut candidate);
        let mut seen = HashSet::new();
        changed.retain(|id| seen.insert(*id));
        self.state = candidate;
        let snapshot = self.snapshot_inner();
        Ok(EditOutcome {
            unsaved_hint: snapshot.unsaved_hint(),
            snapshot,
            changed_nodes: changed,
            creations,
            rebound_nodes: rebound,
        })
    }

    fn validate(&self) -> ValidationReport {
        ValidationReport {
            issues: self
                .state
                .diagnostics
                .iter()
                .map(|diagnostic| ValidationIssue {
                    node_id: diagnostic.node_id,
                    severity: diagnostic.severity,
                    message: diagnostic.message.clone(),
                    location: None,
                })
                .collect(),
        }
    }

    fn save(&mut self) -> Result<SaveOutcome, SaveFailure> {
        let dirty = self.dirty_files();
        let mut written_files = Vec::new();
        let mut diffs = Vec::new();
        for diff in dirty {
            if self.injected_save_failure.as_ref() == Some(&diff.path) {
                self.injected_save_failure = None;
                let snapshot = self.snapshot_inner();
                let unsaved_hint = snapshot.unsaved_hint();
                return Err(SaveFailure {
                    snapshot: Box::new(snapshot),
                    written_files,
                    diffs,
                    failed_file: diff.path,
                    cause: SaveError::injected_failure("injected in-memory save failure"),
                    unsaved_hint,
                    runtime_pending: self.runtime_pending,
                });
            }
            self.mark_saved(&diff.path);
            self.runtime_pending = self.runtime_pending.combine(diff.effect);
            written_files.push(diff.path.clone());
            diffs.push(diff);
        }
        let snapshot = self.snapshot_inner();
        Ok(SaveOutcome {
            unsaved_hint: snapshot.unsaved_hint(),
            runtime_pending: self.runtime_pending,
            snapshot,
            written_files,
            diffs,
        })
    }

    fn acknowledge_reload(&mut self) -> PortSnapshot {
        if self.runtime_pending == RuntimeEffect::Reload {
            self.runtime_pending = RuntimeEffect::None;
        }
        self.snapshot_inner()
    }

    fn acknowledge_restart(&mut self) -> PortSnapshot {
        self.runtime_pending = RuntimeEffect::None;
        self.snapshot_inner()
    }
}

impl MemoryRule {
    fn from_legacy(rule: RuleView, used_ids: &mut HashSet<NodeId>) -> Result<Self, FieldError> {
        let rule_match = map_rule_match(
            &rule.payload.url_path,
            rule.payload.url_path_op,
            &rule.payload.method,
        )?;
        let mut headers = Vec::with_capacity(rule.payload.headers.len());
        for value in rule.payload.headers {
            headers.push(ConditionWithId {
                id: fresh_id(used_ids),
                condition: map_header_condition(&value.name, value.op, &value.value)?,
            });
        }
        let mut body = Vec::with_capacity(rule.payload.body.len());
        for value in rule.payload.body {
            body.push(ConditionWithId {
                id: fresh_id(used_ids),
                condition: map_body_condition(&value.path, value.op, &value.value)?,
            });
        }
        let mode = match rule.payload.respond.mode {
            RespondMode::InlineText => ResponseMode::Inline,
            RespondMode::ServeFile => ResponseMode::File,
        };
        let delay = rule.payload.respond.delay_milliseconds.to_string();
        let respond = map_response(
            mode,
            &rule.payload.respond.text,
            &rule.payload.respond.file_path,
            &rule.payload.respond.status,
            &delay,
        )?;
        Ok(Self {
            id: rule.id,
            rule_match,
            headers,
            body,
            respond,
            validation: rule.validation,
            matched_by_latest_trace: rule.matched_by_latest_trace,
            weight: rule.payload.weight,
            priority: rule.payload.priority,
        })
    }

    fn new(
        id: NodeId,
        rule_match: RuleMatch,
        headers: HeaderConditions,
        body: BodyConditions,
        respond: RespondDefinition,
    ) -> Self {
        Self {
            id,
            rule_match,
            headers,
            body,
            respond,
            validation: NodeValidation::default(),
            matched_by_latest_trace: false,
            weight: None,
            priority: None,
        }
    }

    fn port_view(&self) -> PortRuleView {
        PortRuleView {
            rule_id: self.id,
            rule_match: self.rule_match.clone(),
            conditions: RuleConditionsView {
                headers: self.headers.clone(),
                body: self.body.clone(),
            },
            respond: self.respond.clone(),
        }
    }

    fn legacy_view(&self) -> RuleView {
        let (url_path, url_path_op, method) = project_rule_match(&self.rule_match);
        RuleView {
            id: self.id,
            payload: RulePayload {
                url_path,
                url_path_op,
                method,
                headers: self
                    .headers
                    .iter()
                    .map(|value| project_header_condition(&value.condition))
                    .collect::<Vec<HeaderConditionPayload>>(),
                body: self
                    .body
                    .iter()
                    .map(|value| project_body_condition(&value.condition))
                    .collect::<Vec<BodyConditionPayload>>(),
                respond: project_response(&self.respond),
                weight: self.weight,
                priority: self.priority,
            },
            validation: self.validation.clone(),
            matched_by_latest_trace: self.matched_by_latest_trace,
        }
    }
}

impl From<&RootSettings> for RootBaseline {
    fn from(value: &RootSettings) -> Self {
        Self {
            listener_ip: value.listener_ip.clone(),
            listener_port: value.listener_port,
            tls_enabled: value.tls_enabled,
            tls_cert_file: value.tls_cert_file.clone(),
            tls_key_file: value.tls_key_file.clone(),
            log_level: value.log_level.clone(),
            log_file: value.log_file.clone(),
            log_format: value.log_format.clone(),
            fallback_respond_dir: value.fallback_respond_dir.clone(),
            strategy: value.strategy,
            file_tree_show_hidden: value.file_tree_show_hidden,
            file_tree_builtin_excludes: value.file_tree_builtin_excludes,
            file_tree_extra_excludes: value.file_tree_extra_excludes.clone(),
            file_tree_include: value.file_tree_include.clone(),
        }
    }
}

impl From<&MemoryRuleSet> for SavedRuleSet {
    fn from(value: &MemoryRuleSet) -> Self {
        Self {
            path: value.path.clone(),
            rules: value.rules.iter().map(SavedRule::from).collect(),
        }
    }
}

impl From<&MemoryRule> for SavedRule {
    fn from(value: &MemoryRule) -> Self {
        Self {
            rule_match: value.rule_match.clone(),
            headers: value
                .headers
                .iter()
                .map(|value| value.condition.clone())
                .collect(),
            body: value
                .body
                .iter()
                .map(|value| value.condition.clone())
                .collect(),
            respond: value.respond.clone(),
            weight: value.weight,
            priority: value.priority,
        }
    }
}

fn edit_condition_collection<T>(
    edit: CollectionEdit<ConditionEdit<T>>,
    current: &[ConditionWithId<T>],
    parent: NodeId,
    kind: WorkspaceNodeKind,
    allow_existing: bool,
    used_ids: &mut HashSet<NodeId>,
    creations: &mut Vec<CreationReceipt>,
) -> Result<Option<Vec<ConditionWithId<T>>>, ApplyFailure> {
    match edit {
        CollectionEdit::Preserve => Ok(None),
        CollectionEdit::Clear => Ok(Some(Vec::new())),
        CollectionEdit::Replace(values) => {
            let mut seen = HashSet::new();
            let mut replaced = Vec::with_capacity(values.len());
            for value in values {
                match value {
                    ConditionEdit::Existing { id, condition } => {
                        if !allow_existing {
                            return fail(
                                Some(id),
                                "new rules cannot reference an existing condition",
                            );
                        }
                        if !seen.insert(id) {
                            return fail(Some(id), "replacement condition ID is duplicated");
                        }
                        if !current.iter().any(|value| value.id == id) {
                            return fail(
                                Some(id),
                                "replacement condition does not belong to the rule collection",
                            );
                        }
                        replaced.push(ConditionWithId { id, condition });
                    }
                    ConditionEdit::Create { key, condition } => {
                        let id = fresh_id(used_ids);
                        seen.insert(id);
                        replaced.push(ConditionWithId { id, condition });
                        creations.push(receipt(key, kind, Some(parent), id));
                    }
                }
            }
            Ok(Some(replaced))
        }
    }
}

fn changed_condition_ids<T>(
    edit: &CollectionEdit<ConditionEdit<T>>,
    current: &[ConditionWithId<T>],
) -> Vec<NodeId> {
    match edit {
        CollectionEdit::Preserve => Vec::new(),
        CollectionEdit::Clear | CollectionEdit::Replace(_) => {
            current.iter().map(|value| value.id).collect()
        }
    }
}

fn all_ids(state: &MemoryState) -> HashSet<NodeId> {
    let mut ids = HashSet::new();
    for rule_set in &state.rule_sets {
        ids.insert(rule_set.id.0);
        for rule in &rule_set.rules {
            ids.insert(rule.id);
            ids.extend(rule.headers.iter().map(|value| value.id));
            ids.extend(rule.body.iter().map(|value| value.id));
        }
    }
    ids
}

fn fresh_id(used_ids: &mut HashSet<NodeId>) -> NodeId {
    loop {
        let id = NodeId::new();
        if used_ids.insert(id) {
            return id;
        }
    }
}

fn receipt(
    key: SemanticCreationKey,
    kind: WorkspaceNodeKind,
    parent: Option<NodeId>,
    new_id: NodeId,
) -> CreationReceipt {
    CreationReceipt {
        key,
        kind,
        parent,
        new_id,
    }
}

fn failure(node_id: Option<NodeId>, message: &str) -> ApplyFailure {
    ApplyFailure {
        diagnostic: Diagnostic {
            node_id,
            severity: Severity::Error,
            message: message.to_owned(),
        },
    }
}

fn fail<T>(node_id: Option<NodeId>, message: &str) -> Result<T, ApplyFailure> {
    Err(failure(node_id, message))
}

fn find_rule_position(state: &MemoryState, id: NodeId) -> Option<(usize, usize)> {
    state
        .rule_sets
        .iter()
        .enumerate()
        .find_map(|(set, rule_set)| {
            rule_set
                .rules
                .iter()
                .position(|rule| rule.id == id)
                .map(|rule| (set, rule))
        })
}

fn find_rule_mut(state: &mut MemoryState, id: NodeId) -> Option<&mut MemoryRule> {
    state
        .rule_sets
        .iter_mut()
        .find_map(|rule_set| rule_set.rules.iter_mut().find(|rule| rule.id == id))
}

fn find_header_position(state: &MemoryState, id: NodeId) -> Option<(NodeId, usize)> {
    state.rule_sets.iter().find_map(|rule_set| {
        rule_set.rules.iter().find_map(|rule| {
            rule.headers
                .iter()
                .position(|value| value.id == id)
                .map(|index| (rule.id, index))
        })
    })
}

fn find_header_mut(
    state: &mut MemoryState,
    id: NodeId,
) -> Option<(NodeId, &mut ConditionWithId<HeaderCondition>)> {
    state.rule_sets.iter_mut().find_map(|rule_set| {
        rule_set.rules.iter_mut().find_map(|rule| {
            rule.headers
                .iter_mut()
                .find(|value| value.id == id)
                .map(|value| (rule.id, value))
        })
    })
}

fn find_body_position(state: &MemoryState, id: NodeId) -> Option<(NodeId, usize)> {
    state.rule_sets.iter().find_map(|rule_set| {
        rule_set.rules.iter().find_map(|rule| {
            rule.body
                .iter()
                .position(|value| value.id == id)
                .map(|index| (rule.id, index))
        })
    })
}

fn find_body_mut(
    state: &mut MemoryState,
    id: NodeId,
) -> Option<(NodeId, &mut ConditionWithId<BodyCondition>)> {
    state.rule_sets.iter_mut().find_map(|rule_set| {
        rule_set.rules.iter_mut().find_map(|rule| {
            rule.body
                .iter_mut()
                .find(|value| value.id == id)
                .map(|value| (rule.id, value))
        })
    })
}

fn root_effect(current: &RootSettings, saved: &RootBaseline) -> RuntimeEffect {
    let current = RootBaseline::from(current);
    if current == *saved {
        return RuntimeEffect::None;
    }
    if current.listener_ip != saved.listener_ip
        || current.listener_port != saved.listener_port
        || current.tls_enabled != saved.tls_enabled
        || current.tls_cert_file != saved.tls_cert_file
        || current.tls_key_file != saved.tls_key_file
        || current.log_file != saved.log_file
    {
        RuntimeEffect::Restart
    } else {
        RuntimeEffect::Reload
    }
}

fn validate_root_settings(settings: &RootSettings) -> Result<(), FieldError> {
    use WorkspaceEditValue as V;
    use WorkspaceRootKey as K;
    for key in WorkspaceRootKey::ALL {
        let value = match key {
            K::ListenerIpAddress => V::String(settings.listener_ip.clone()),
            K::ListenerPort => V::Integer(i64::from(settings.listener_port)),
            K::ServiceFallbackRespondDir => V::String(settings.fallback_respond_dir.clone()),
            K::ServiceStrategy => V::Enum(settings.strategy.label().to_owned()),
            K::TlsEnabled => V::Boolean(settings.tls_enabled),
            K::TlsCertFile => V::String(settings.tls_cert_file.clone()),
            K::TlsKeyFile => V::String(settings.tls_key_file.clone()),
            K::LogLevel => V::Enum(settings.log_level.clone()),
            K::LogFile => V::String(settings.log_file.clone()),
            K::LogFormat => V::Enum(settings.log_format.clone()),
            K::FileTreeShowHidden => V::Boolean(settings.file_tree_show_hidden),
            K::FileTreeBuiltinExcludes => V::Boolean(settings.file_tree_builtin_excludes),
            K::FileTreeExtraExcludes => V::StringList(settings.file_tree_extra_excludes.clone()),
            K::FileTreeInclude => V::StringList(settings.file_tree_include.clone()),
        };
        map_root_setting(key, value)?;
    }
    Ok(())
}

fn apply_root_setting(settings: &mut RootSettings, edit: RootSettingEdit) {
    use WorkspaceEditValue as V;
    use WorkspaceRootKey as K;
    match (edit.key, edit.value) {
        (K::ListenerIpAddress, V::String(value)) => settings.listener_ip = value,
        (K::ListenerPort, V::Integer(value)) => settings.listener_port = value as u16,
        (K::ServiceFallbackRespondDir, V::String(value)) => settings.fallback_respond_dir = value,
        (K::ServiceStrategy, V::Enum(value)) => {
            settings.strategy = match value.as_str() {
                "FirstMatch" => Strategy::FirstMatch,
                "UniformRandom" => Strategy::UniformRandom,
                "WeightedRandom" => Strategy::WeightedRandom,
                "Priority" => Strategy::Priority,
                "RoundRobin" => Strategy::RoundRobin,
                _ => unreachable!("RootSettingEdit seals strategy values"),
            }
        }
        (K::TlsEnabled, V::Boolean(value)) => settings.tls_enabled = value,
        (K::TlsCertFile, V::String(value)) => settings.tls_cert_file = value,
        (K::TlsKeyFile, V::String(value)) => settings.tls_key_file = value,
        (K::LogLevel, V::Enum(value)) => settings.log_level = value,
        (K::LogFile, V::String(value)) => settings.log_file = value,
        (K::LogFormat, V::Enum(value)) => settings.log_format = value,
        (K::FileTreeShowHidden, V::Boolean(value)) => settings.file_tree_show_hidden = value,
        (K::FileTreeBuiltinExcludes, V::Boolean(value)) => {
            settings.file_tree_builtin_excludes = value
        }
        (K::FileTreeExtraExcludes, V::StringList(value)) => {
            settings.file_tree_extra_excludes = value
        }
        (K::FileTreeInclude, V::StringList(value)) => settings.file_tree_include = value,
        _ => unreachable!("RootSettingEdit seals key/value combinations"),
    }
}

fn refresh_validation(state: &mut MemoryState) {
    // The local adapter admits only values that have passed the canonical
    // mapping and structural checks. Imported diagnostics cannot safely be
    // correlated after an edit, so a successful candidate validation replaces
    // them with the local validator's empty result.
    state.diagnostics.clear();
    for rule_set in &mut state.rule_sets {
        rule_set.validation = NodeValidation::default();
        for rule in &mut rule_set.rules {
            rule.validation = NodeValidation::default();
        }
    }
}

fn restore_subtree(
    state: &mut MemoryState,
    archive: ArchivedSubtree,
    changed: &mut Vec<NodeId>,
    creations: &mut Vec<CreationReceipt>,
    rebound: &mut Vec<NodeRebind>,
    used_ids: &mut HashSet<NodeId>,
) -> Result<(), ApplyFailure> {
    for node in &archive.nodes {
        if used_ids.contains(&node.old_id) {
            return fail(
                Some(node.old_id),
                "archived old ID is still live in this workspace session",
            );
        }
    }

    match archive.placement {
        RestorePlacement::Rule {
            parent,
            insertion_index,
        } => {
            let rule_set = state
                .rule_sets
                .iter()
                .find(|rule_set| rule_set.id == parent)
                .ok_or_else(|| failure(Some(parent.0), "restore parent rule set does not exist"))?;
            if insertion_index > rule_set.rules.len() {
                return fail(Some(parent.0), "restore insertion index is out of range");
            }
            changed.push(parent.0);
        }
        RestorePlacement::RuleSetRoot { insertion_index } => {
            if insertion_index > state.rule_sets.len() {
                return fail(None, "rule-set restore insertion index is out of range");
            }
            let root = archive
                .nodes
                .iter()
                .find(|node| node.old_id == archive.former_root)
                .expect("archive constructor validates root");
            let ArchivedNodePayload::RuleSet { path } = &root.payload else {
                unreachable!("archive constructor validates root kind");
            };
            if state
                .rule_sets
                .iter()
                .any(|rule_set| rule_set.path == *path)
            {
                return fail(
                    Some(archive.former_root),
                    "restored rule-set path already exists",
                );
            }
        }
    }

    let mut ids = HashMap::new();
    for node in &archive.nodes {
        ids.insert(node.old_id, fresh_id(used_ids));
    }
    let root_new = ids[&archive.former_root];

    if let RestorePlacement::RuleSetRoot { insertion_index } = archive.placement {
        let root = archive
            .nodes
            .iter()
            .find(|node| node.old_id == archive.former_root)
            .expect("archive constructor validates root");
        let ArchivedNodePayload::RuleSet { path } = &root.payload else {
            unreachable!("archive constructor validates root kind");
        };
        state.rule_sets.insert(
            insertion_index,
            MemoryRuleSet {
                id: RuleSetId(root_new),
                path: path.clone(),
                rules: Vec::new(),
                validation: NodeValidation::default(),
            },
        );
    }

    for node in &archive.nodes {
        if let ArchivedNodePayload::Rule(payload) = &node.payload {
            if !matches!(&payload.headers, CollectionEdit::Preserve)
                || !matches!(&payload.body, CollectionEdit::Preserve)
            {
                return fail(
                    Some(node.old_id),
                    "archived rule conditions must be represented as archived child nodes",
                );
            }
            let rule = MemoryRule::new(
                ids[&node.old_id],
                payload.rule_match.clone(),
                Vec::new(),
                Vec::new(),
                payload.respond.clone(),
            );
            let (parent, insertion_index) = if node.old_id == archive.former_root {
                let RestorePlacement::Rule {
                    parent,
                    insertion_index,
                } = archive.placement
                else {
                    unreachable!("rule-set archive rule has internal parent");
                };
                (parent, Some(insertion_index))
            } else {
                let parent = RuleSetId(ids[&node.parent.expect("archive validates parent")]);
                (parent, None)
            };
            let rule_set = state
                .rule_sets
                .iter_mut()
                .find(|rule_set| rule_set.id == parent)
                .ok_or_else(|| failure(Some(parent.0), "restored rule parent is unavailable"))?;
            if let Some(index) = insertion_index {
                rule_set.rules.insert(index, rule);
            } else {
                rule_set.rules.push(rule);
            }
        }
    }

    for node in &archive.nodes {
        let Some(old_parent) = node.parent else {
            continue;
        };
        let new_parent = ids[&old_parent];
        match &node.payload {
            ArchivedNodePayload::HeaderCondition(condition) => {
                find_rule_mut(state, new_parent)
                    .expect("archive validates condition parent kind")
                    .headers
                    .push(ConditionWithId {
                        id: ids[&node.old_id],
                        condition: condition.clone(),
                    });
            }
            ArchivedNodePayload::BodyCondition(condition) => {
                find_rule_mut(state, new_parent)
                    .expect("archive validates condition parent kind")
                    .body
                    .push(ConditionWithId {
                        id: ids[&node.old_id],
                        condition: condition.clone(),
                    });
            }
            _ => {}
        }
    }

    for node in archive.nodes {
        let new_id = ids[&node.old_id];
        let parent = if node.old_id == archive.former_root {
            match archive.placement {
                RestorePlacement::RuleSetRoot { .. } => None,
                RestorePlacement::Rule { parent, .. } => Some(parent.0),
            }
        } else {
            Some(ids[&node.parent.expect("archive validates parent")])
        };
        let kind = node.payload.kind();
        creations.push(receipt(node.key, kind, parent, new_id));
        rebound.push(NodeRebind {
            old_id: node.old_id,
            kind,
            new_id,
        });
        changed.push(new_id);
    }
    Ok(())
}
