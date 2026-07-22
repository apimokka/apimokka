//! MK-053 app-owned workspace session and editor-draft state.

use std::collections::HashMap;
use std::ops::Deref;

use super::HistoryEntry;
use super::runtime::{
    RuntimeAction, RuntimeRequestId, RuntimeRequestToken, SavedConfigRevision, SessionGeneration,
};

use apimokka_model::workspace_port::{
    map_body_condition, map_header_condition, map_response, map_rule_match,
};
use apimokka_model::{
    ApplyFailure, BodyConditionPayload, CollectionEdit, ConditionEdit, EditIntent, EditOutcome,
    EditTransaction, HeaderConditionPayload, MemoryWorkspace, NodeId, PortSnapshot, RulePayload,
    SemanticCreationKey, ValidationIssue, ValidationReport, WorkspaceNodeKind, WorkspacePort,
    WorkspaceSnapshot,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceIdentity {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftBinding {
    Existing(NodeId),
    Pending(SemanticCreationKey),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionFamily {
    Header,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionFocus {
    pub rule_id: NodeId,
    pub family: ConditionFamily,
    pub binding: DraftBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractFaultAdoption {
    PostCommit,
    NonAdoptingRead,
}

#[derive(Debug, Clone)]
pub struct RuleEditorDraft {
    pub payload: RulePayload,
    pub response_delay: String,
    pub header_bindings: Vec<DraftBinding>,
    pub body_bindings: Vec<DraftBinding>,
}

impl RuleEditorDraft {
    fn from_snapshot(snapshot: &PortSnapshot, rule_id: NodeId) -> Option<Self> {
        let legacy = snapshot.workspace().find_rule(rule_id)?.1;
        let canonical = snapshot.rule(rule_id)?;
        if legacy.payload.headers.len() != canonical.conditions().headers.len()
            || legacy.payload.body.len() != canonical.conditions().body.len()
        {
            return None;
        }
        Some(Self {
            payload: legacy.payload.clone(),
            response_delay: canonical
                .respond()
                .delay_milliseconds()
                .map(|delay| delay.to_string())
                .unwrap_or_default(),
            header_bindings: canonical
                .conditions()
                .headers
                .iter()
                .map(|condition| DraftBinding::Existing(condition.id))
                .collect(),
            body_bindings: canonical
                .conditions()
                .body
                .iter()
                .map(|condition| DraftBinding::Existing(condition.id))
                .collect(),
        })
    }

    pub fn push_header(&mut self, key: SemanticCreationKey) {
        self.payload.headers.push(HeaderConditionPayload {
            name: String::new(),
            op: apimokka_model::HeaderOp::Equal,
            value: String::new(),
        });
        self.header_bindings.push(DraftBinding::Pending(key));
    }

    pub fn push_body(&mut self, key: SemanticCreationKey) {
        self.payload.body.push(BodyConditionPayload {
            path: String::new(),
            op: apimokka_model::BodyOp::Equal,
            value: String::new(),
        });
        self.body_bindings.push(DraftBinding::Pending(key));
    }
}

#[derive(Debug, Clone)]
pub struct RootSettingDrafts {
    pub listener_ip: String,
    pub listener_port: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RulePrototype {
    pub weight: Option<u32>,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracePrototypeSettings {
    pub enabled: bool,
    pub transport: apimokka_model::settings::TraceTransport,
    pub uds_path: String,
    pub tcp_addr: String,
    pub queue_size: u32,
}

#[derive(Debug, Clone, Default)]
pub struct PrototypeState {
    pub rule_extras: HashMap<NodeId, RulePrototype>,
    pub trace: Option<TracePrototypeSettings>,
}

pub struct WorkspaceSession {
    port: Box<dyn WorkspacePort>,
    latest: PortSnapshot,
    pub identity: WorkspaceIdentity,
    pub rule_drafts: HashMap<NodeId, RuleEditorDraft>,
    pub root_drafts: RootSettingDrafts,
    pub prototype: PrototypeState,
    pub undo_stack: Vec<HistoryEntry>,
    pub redo_stack: Vec<HistoryEntry>,
    pub condition_focus: Option<ConditionFocus>,
    pub faulted: bool,
    pub contract_fault: Option<String>,
    pub contract_fault_adoption: Option<ContractFaultAdoption>,
    generation: SessionGeneration,
    saved_config_revision: SavedConfigRevision,
    next_runtime_request_id: u64,
    next_creation_key: u64,
}

pub(super) enum SessionApplyResult {
    Validated(Box<EditOutcome>),
    ApplyFailure(ApplyFailure),
    ContractFault,
}

pub(super) enum SessionSaveResult {
    Saved,
    SaveFailure(apimokka_model::SaveFailure),
    ContractFault,
}

pub(super) enum SessionValidationResult {
    Equal,
    ContractFault,
}

impl WorkspaceSession {
    #[cfg(test)]
    pub fn new(seed: WorkspaceSnapshot) -> Result<Self, apimokka_model::FieldError> {
        Self::new_with_generation(seed, SessionGeneration(0))
    }

    pub(crate) fn new_with_generation(
        seed: WorkspaceSnapshot,
        generation: SessionGeneration,
    ) -> Result<Self, apimokka_model::FieldError> {
        let prototype = prototype_from_seed(&seed);
        let port = MemoryWorkspace::new(seed)?;
        Ok(Self::from_port_with_generation(
            Box::new(port),
            prototype,
            generation,
        ))
    }

    #[cfg(test)]
    pub(crate) fn from_port(port: Box<dyn WorkspacePort>, prototype: PrototypeState) -> Self {
        Self::from_port_with_generation(port, prototype, SessionGeneration(0))
    }

    pub(crate) fn from_port_with_generation(
        port: Box<dyn WorkspacePort>,
        prototype: PrototypeState,
        generation: SessionGeneration,
    ) -> Self {
        let latest = port.snapshot();
        let identity = WorkspaceIdentity {
            name: latest.workspace().meta.name.clone(),
            path: latest.workspace().meta.path.clone(),
        };
        let root_drafts = RootSettingDrafts {
            listener_ip: latest.workspace().root_settings.listener_ip.clone(),
            listener_port: latest.workspace().root_settings.listener_port.to_string(),
        };
        Self {
            port,
            latest,
            identity,
            rule_drafts: HashMap::new(),
            root_drafts,
            prototype,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            condition_focus: None,
            faulted: false,
            contract_fault: None,
            contract_fault_adoption: None,
            generation,
            saved_config_revision: SavedConfigRevision(0),
            next_runtime_request_id: 1,
            next_creation_key: 0,
        }
    }

    #[cfg(test)]
    pub fn generation(&self) -> SessionGeneration {
        self.generation
    }

    pub fn saved_config_revision(&self) -> SavedConfigRevision {
        self.saved_config_revision
    }

    pub(super) fn issue_runtime_request(&mut self, action: RuntimeAction) -> RuntimeRequestToken {
        let request_id = RuntimeRequestId(self.next_runtime_request_id);
        self.next_runtime_request_id = self
            .next_runtime_request_id
            .checked_add(1)
            .expect("runtime request ID overflow");
        RuntimeRequestToken {
            generation: self.generation,
            request_id,
            action,
            consumed_revision: (action != RuntimeAction::Stop)
                .then_some(self.saved_config_revision),
        }
    }

    pub fn latest(&self) -> &PortSnapshot {
        &self.latest
    }

    pub fn ensure_rule_draft(&mut self, rule_id: NodeId) -> Option<&mut RuleEditorDraft> {
        if !self.rule_drafts.contains_key(&rule_id) {
            let draft = RuleEditorDraft::from_snapshot(&self.latest, rule_id)?;
            self.rule_drafts.insert(rule_id, draft);
        }
        self.rule_drafts.get_mut(&rule_id)
    }

    pub fn rule_draft(&self, rule_id: NodeId) -> Option<&RuleEditorDraft> {
        self.rule_drafts.get(&rule_id)
    }

    pub fn focus_condition(
        &mut self,
        rule_id: NodeId,
        family: ConditionFamily,
        binding: DraftBinding,
    ) {
        self.condition_focus = Some(ConditionFocus {
            rule_id,
            family,
            binding,
        });
    }

    pub fn clear_condition_focus_unless_rule(&mut self, rule_id: Option<NodeId>) {
        if self
            .condition_focus
            .as_ref()
            .is_some_and(|focus| Some(focus.rule_id) != rule_id)
        {
            self.condition_focus = None;
        }
    }

    pub fn clear_condition_focus_family(&mut self, rule_id: NodeId, family: ConditionFamily) {
        if self
            .condition_focus
            .as_ref()
            .is_some_and(|focus| focus.rule_id == rule_id && focus.family == family)
        {
            self.condition_focus = None;
        }
    }

    pub fn has_pending_drafts(&self) -> bool {
        if self.root_drafts.listener_ip != self.root_settings.listener_ip
            || self.root_drafts.listener_port != self.root_settings.listener_port.to_string()
        {
            return true;
        }
        self.rule_drafts
            .iter()
            .any(|(rule_id, draft)| self.rule_draft_is_pending(*rule_id, draft))
    }

    fn rule_draft_is_pending(&self, rule_id: NodeId, draft: &RuleEditorDraft) -> bool {
        let Some(canonical) = self.latest.rule(rule_id) else {
            return true;
        };
        let Ok(rule_match) = map_rule_match(
            &draft.payload.url_path,
            draft.payload.url_path_op,
            &draft.payload.method,
        ) else {
            return true;
        };
        if &rule_match != canonical.rule_match() {
            return true;
        }
        let mode = match draft.payload.respond.mode {
            apimokka_model::snapshot::RespondMode::InlineText => {
                apimokka_model::ResponseMode::Inline
            }
            apimokka_model::snapshot::RespondMode::ServeFile => apimokka_model::ResponseMode::File,
        };
        let Ok(respond) = map_response(
            mode,
            &draft.payload.respond.text,
            &draft.payload.respond.file_path,
            &draft.payload.respond.status,
            &draft.response_delay,
        ) else {
            return true;
        };
        if &respond != canonical.respond()
            || draft.payload.headers.len() != draft.header_bindings.len()
            || draft.payload.body.len() != draft.body_bindings.len()
            || draft.header_bindings.len() != canonical.conditions().headers.len()
            || draft.body_bindings.len() != canonical.conditions().body.len()
        {
            return true;
        }
        for (payload, binding) in draft.payload.headers.iter().zip(&draft.header_bindings) {
            let DraftBinding::Existing(id) = binding else {
                return true;
            };
            let Ok(mapped) = map_header_condition(&payload.name, payload.op, &payload.value) else {
                return true;
            };
            if canonical
                .conditions()
                .headers
                .iter()
                .find(|condition| condition.id == *id)
                .is_none_or(|condition| condition.condition != mapped)
            {
                return true;
            }
        }
        for (payload, binding) in draft.payload.body.iter().zip(&draft.body_bindings) {
            let DraftBinding::Existing(id) = binding else {
                return true;
            };
            let Ok(mapped) = map_body_condition(&payload.path, payload.op, &payload.value) else {
                return true;
            };
            if canonical
                .conditions()
                .body
                .iter()
                .find(|condition| condition.id == *id)
                .is_none_or(|condition| condition.condition != mapped)
            {
                return true;
            }
        }
        false
    }

    pub fn creation_key(&mut self, family: &str) -> SemanticCreationKey {
        let value = format!("app/{family}/{}", self.next_creation_key);
        self.next_creation_key += 1;
        SemanticCreationKey::new(value).expect("generated creation keys are nonempty")
    }

    pub(super) fn apply(&mut self, transaction: EditTransaction) -> SessionApplyResult {
        if self.faulted {
            return SessionApplyResult::ContractFault;
        }
        let before = self.latest.clone();
        let expectations = creation_expectations(&transaction);
        let rebound_expectations = rebind_expectations(&transaction);
        let outcome = match self.port.apply(transaction) {
            Ok(outcome) => outcome,
            Err(failure) => return SessionApplyResult::ApplyFailure(failure),
        };
        let correlation_problem =
            validate_correlations(&before, &expectations, &rebound_expectations, &outcome).err();
        let identity_problem = self.adopt_snapshot(outcome.snapshot.clone()).err();
        if let Some(problem) = correlation_problem.or(identity_problem) {
            self.enter_contract_fault(problem);
            return SessionApplyResult::ContractFault;
        }
        self.reconcile_condition_focus(&outcome);
        SessionApplyResult::Validated(Box::new(outcome))
    }

    pub(super) fn validate(&mut self) -> SessionValidationResult {
        if self.faulted {
            return SessionValidationResult::ContractFault;
        }
        let expected = match canonical_validation_projection(&self.latest) {
            Ok(expected) => expected,
            Err(problem) => {
                self.enter_read_contract_fault(problem);
                return SessionValidationResult::ContractFault;
            }
        };
        let actual = self.port.validate();
        if actual == expected {
            SessionValidationResult::Equal
        } else {
            self.enter_read_contract_fault(validation_mismatch_detail(&expected, &actual));
            SessionValidationResult::ContractFault
        }
    }

    pub(super) fn save(&mut self) -> SessionSaveResult {
        if self.faulted {
            return SessionSaveResult::ContractFault;
        }
        let before = self.latest.clone();
        match self.port.save() {
            Ok(outcome) => {
                let progress_problem = validate_save_outcome_progress(&before, &outcome).err();
                let phase_problem = validate_save_phases(
                    &before,
                    &outcome.snapshot,
                    &outcome.diffs,
                    &[],
                    outcome.unsaved_hint,
                    outcome.runtime_pending,
                )
                .err();
                let identity_problem = self.adopt_snapshot(outcome.snapshot.clone()).err();
                if progress_problem.is_none() {
                    self.advance_saved_config_revision(&outcome.diffs);
                }
                if let Some(problem) = progress_problem.or(phase_problem).or(identity_problem) {
                    self.enter_contract_fault(problem);
                    SessionSaveResult::ContractFault
                } else {
                    SessionSaveResult::Saved
                }
            }
            Err(failure) => {
                let prefix_len = failure.written_files.len().min(before.dirty_files().len());
                let progress_problem = validate_save_failure_progress(&before, &failure).err();
                let phase_problem = validate_save_phases(
                    &before,
                    &failure.snapshot,
                    &failure.diffs,
                    &before.dirty_files()[prefix_len..],
                    failure.unsaved_hint,
                    failure.runtime_pending,
                )
                .err();
                let identity_problem = self.adopt_snapshot((*failure.snapshot).clone()).err();
                if progress_problem.is_none() {
                    self.advance_saved_config_revision(&failure.diffs);
                }
                if let Some(problem) = progress_problem.or(phase_problem).or(identity_problem) {
                    self.enter_contract_fault(problem);
                    SessionSaveResult::ContractFault
                } else {
                    SessionSaveResult::SaveFailure(failure)
                }
            }
        }
    }

    fn advance_saved_config_revision(&mut self, diffs: &[apimokka_model::FileDiff]) {
        if diffs
            .iter()
            .any(|diff| diff.effect != apimokka_model::RuntimeEffect::None)
        {
            self.saved_config_revision.0 = self
                .saved_config_revision
                .0
                .checked_add(1)
                .expect("saved configuration revision overflow");
        }
    }

    pub(super) fn acknowledge_reload(&mut self) {
        if self.faulted {
            return;
        }
        let snapshot = self.port.acknowledge_reload();
        if let Err(problem) = self.adopt_snapshot(snapshot) {
            self.enter_contract_fault(problem);
        }
    }

    pub(super) fn acknowledge_restart(&mut self) {
        if self.faulted {
            return;
        }
        let snapshot = self.port.acknowledge_restart();
        if let Err(problem) = self.adopt_snapshot(snapshot) {
            self.enter_contract_fault(problem);
        }
    }

    fn adopt_snapshot(&mut self, snapshot: PortSnapshot) -> Result<(), String> {
        let identity_drift = (snapshot.workspace().meta.name != self.identity.name
            || snapshot.workspace().meta.path != self.identity.path)
            .then(|| {
                format!(
                    "workspace identity drifted from admitted name/path {:?}/{:?} to {:?}/{:?}",
                    self.identity.name,
                    self.identity.path,
                    snapshot.workspace().meta.name,
                    snapshot.workspace().meta.path
                )
            });
        self.latest = snapshot;
        self.retain_live_state();
        identity_drift.map_or(Ok(()), Err)
    }

    fn enter_contract_fault(&mut self, problem: String) {
        self.enter_fault(problem, ContractFaultAdoption::PostCommit);
    }

    fn enter_read_contract_fault(&mut self, problem: String) {
        self.enter_fault(problem, ContractFaultAdoption::NonAdoptingRead);
    }

    #[cfg(test)]
    pub(super) fn enter_future_fault_for_test(
        &mut self,
        problem: String,
        adoption: ContractFaultAdoption,
    ) {
        self.enter_fault(problem, adoption);
    }

    fn enter_fault(&mut self, problem: String, adoption: ContractFaultAdoption) {
        self.rule_drafts.clear();
        self.condition_focus = None;
        self.root_drafts = RootSettingDrafts {
            listener_ip: self.latest.workspace().root_settings.listener_ip.clone(),
            listener_port: self
                .latest
                .workspace()
                .root_settings
                .listener_port
                .to_string(),
        };
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.faulted = true;
        self.contract_fault = Some(problem);
        self.contract_fault_adoption = Some(adoption);
    }

    fn retain_live_state(&mut self) {
        let live_rules = self
            .latest
            .rules()
            .iter()
            .map(|rule| rule.rule_id())
            .collect::<std::collections::HashSet<_>>();
        self.prototype
            .rule_extras
            .retain(|id, _| live_rules.contains(id));
        self.rule_drafts.retain(|id, _| live_rules.contains(id));
        if self
            .condition_focus
            .as_ref()
            .is_some_and(|focus| !live_rules.contains(&focus.rule_id))
        {
            self.condition_focus = None;
        }
    }

    fn reconcile_condition_focus(&mut self, outcome: &EditOutcome) {
        let Some(mut focus) = self.condition_focus.take() else {
            return;
        };
        if let DraftBinding::Pending(key) = &focus.binding {
            let expected_kind = match focus.family {
                ConditionFamily::Header => WorkspaceNodeKind::HeaderCondition,
                ConditionFamily::Body => WorkspaceNodeKind::BodyCondition,
            };
            if let Some(receipt) = outcome.creations.iter().find(|receipt| {
                receipt.key == *key
                    && receipt.kind == expected_kind
                    && receipt.parent == Some(focus.rule_id)
            }) {
                focus.binding = DraftBinding::Existing(receipt.new_id);
            }
        }
        if let DraftBinding::Existing(id) = focus.binding
            && let Some(rebind) = outcome
                .rebound_nodes
                .iter()
                .find(|rebind| rebind.old_id == id)
        {
            focus.binding = DraftBinding::Existing(rebind.new_id);
        }
        if condition_focus_is_live(&self.latest, &focus) {
            self.condition_focus = Some(focus);
        }
    }
}

fn condition_focus_is_live(snapshot: &PortSnapshot, focus: &ConditionFocus) -> bool {
    let Some(rule) = snapshot.rule(focus.rule_id) else {
        return false;
    };
    match (&focus.family, &focus.binding) {
        (_, DraftBinding::Pending(_)) => true,
        (ConditionFamily::Header, DraftBinding::Existing(id)) => rule
            .conditions()
            .headers
            .iter()
            .any(|condition| condition.id == *id),
        (ConditionFamily::Body, DraftBinding::Existing(id)) => rule
            .conditions()
            .body
            .iter()
            .any(|condition| condition.id == *id),
    }
}

fn validate_save_outcome_progress(
    before: &PortSnapshot,
    outcome: &apimokka_model::SaveOutcome,
) -> Result<(), String> {
    let dirty = before.dirty_files();
    let written_paths = outcome
        .diffs
        .iter()
        .map(|diff| &diff.path)
        .collect::<Vec<_>>();
    if outcome.diffs != dirty
        || outcome.written_files.iter().collect::<Vec<_>>() != written_paths
        || !outcome.snapshot.dirty_files().is_empty()
    {
        return Err("save outcome did not report the exact captured dirty workspace".into());
    }
    Ok(())
}

fn validate_save_failure_progress(
    before: &PortSnapshot,
    failure: &apimokka_model::SaveFailure,
) -> Result<(), String> {
    let dirty = before.dirty_files();
    let prefix_len = failure.written_files.len();
    if prefix_len >= dirty.len()
        || failure.diffs != dirty[..prefix_len]
        || failure
            .written_files
            .iter()
            .zip(&failure.diffs)
            .any(|(path, diff)| path != &diff.path)
        || failure.failed_file != dirty[prefix_len].path
        || failure.snapshot.dirty_files() != &dirty[prefix_len..]
    {
        return Err("save failure did not report an exact captured dirty prefix and suffix".into());
    }
    Ok(())
}

fn validate_save_phases(
    before: &PortSnapshot,
    after: &PortSnapshot,
    written: &[apimokka_model::FileDiff],
    remaining: &[apimokka_model::FileDiff],
    reported_unsaved: apimokka_model::RuntimeEffect,
    reported_pending: apimokka_model::RuntimeEffect,
) -> Result<(), String> {
    let expected_pending = written
        .iter()
        .fold(before.runtime_pending(), |pending, diff| {
            pending.combine(diff.effect)
        });
    let expected_unsaved = remaining
        .iter()
        .fold(apimokka_model::RuntimeEffect::None, |pending, diff| {
            pending.combine(diff.effect)
        });
    if reported_pending != expected_pending
        || after.runtime_pending() != expected_pending
        || reported_unsaved != expected_unsaved
        || after.unsaved_hint() != expected_unsaved
    {
        return Err("save result phase fields differed from verified workspace progress".into());
    }
    Ok(())
}

fn canonical_validation_projection(snapshot: &PortSnapshot) -> Result<ValidationReport, String> {
    let live = editable_node_index(snapshot)?;
    let mut issues = Vec::new();
    for diagnostic in &snapshot.workspace().diagnostics {
        if let Some(id) = diagnostic.node_id
            && !live.contains_key(&id)
        {
            return Err(format!(
                "workspace diagnostic targets unknown editable node {id:?}"
            ));
        }
        issues.push(ValidationIssue {
            node_id: diagnostic.node_id,
            severity: diagnostic.severity,
            message: diagnostic.message.clone(),
            location: None,
        });
    }
    for rule_set in &snapshot.workspace().rule_sets {
        append_owned_validation_issues(&mut issues, &rule_set.validation.issues, rule_set.id.0)?;
        for rule in &rule_set.rules {
            append_owned_validation_issues(&mut issues, &rule.validation.issues, rule.id)?;
        }
    }
    Ok(ValidationReport { issues })
}

fn validation_mismatch_detail(expected: &ValidationReport, actual: &ValidationReport) -> String {
    let first_difference = expected
        .issues
        .iter()
        .map(Some)
        .chain(std::iter::repeat(None))
        .zip(
            actual
                .issues
                .iter()
                .map(Some)
                .chain(std::iter::repeat(None)),
        )
        .enumerate()
        .find(|(_, (expected, actual))| expected != actual)
        .expect("unequal validation reports have a first differing issue");
    format!(
        "workspace validation report differed from the cached canonical projection: expected {} issue(s), received {}; first difference at index {}: expected {:?}, received {:?}",
        expected.issues.len(),
        actual.issues.len(),
        first_difference.0,
        first_difference.1.0,
        first_difference.1.1,
    )
}

fn append_owned_validation_issues(
    projected: &mut Vec<ValidationIssue>,
    issues: &[ValidationIssue],
    owner: NodeId,
) -> Result<(), String> {
    for issue in issues {
        if let Some(explicit) = issue.node_id
            && explicit != owner
        {
            return Err(format!(
                "node validation issue owner mismatch: expected {owner:?}, received {explicit:?}"
            ));
        }
        let mut issue = issue.clone();
        issue.node_id = Some(owner);
        projected.push(issue);
    }
    Ok(())
}

#[derive(Debug)]
struct CreationExpectation {
    key: SemanticCreationKey,
    kind: WorkspaceNodeKind,
    parent: ExpectedParent,
}

#[derive(Debug)]
enum ExpectedParent {
    None,
    Existing(NodeId),
    Created(SemanticCreationKey),
}

fn creation_expectations(transaction: &EditTransaction) -> Vec<CreationExpectation> {
    let mut expected = Vec::new();
    for intent in transaction.intents() {
        match intent {
            EditIntent::AddRuleSet { key, .. } => expected.push(CreationExpectation {
                key: key.clone(),
                kind: WorkspaceNodeKind::RuleSet,
                parent: ExpectedParent::None,
            }),
            EditIntent::AddRule {
                parent, rule, key, ..
            } => {
                expected.push(CreationExpectation {
                    key: key.clone(),
                    kind: WorkspaceNodeKind::Rule,
                    parent: ExpectedParent::Existing(parent.0),
                });
                collect_condition_expectations(
                    &mut expected,
                    rule,
                    ExpectedParent::Created(key.clone()),
                );
            }
            EditIntent::UpdateRule { id, rule } => {
                collect_condition_expectations(&mut expected, rule, ExpectedParent::Existing(*id));
            }
            EditIntent::AddHeaderCondition { rule_id, key, .. } => {
                expected.push(CreationExpectation {
                    key: key.clone(),
                    kind: WorkspaceNodeKind::HeaderCondition,
                    parent: ExpectedParent::Existing(*rule_id),
                })
            }
            EditIntent::AddBodyCondition { rule_id, key, .. } => {
                expected.push(CreationExpectation {
                    key: key.clone(),
                    kind: WorkspaceNodeKind::BodyCondition,
                    parent: ExpectedParent::Existing(*rule_id),
                })
            }
            EditIntent::RestoreSubtree { archive } => {
                let keys = archive
                    .nodes()
                    .iter()
                    .map(|node| (node.old_id, node.key.clone()))
                    .collect::<HashMap<_, _>>();
                for node in archive.nodes() {
                    let parent = if let Some(parent) = node.parent {
                        ExpectedParent::Created(keys[&parent].clone())
                    } else {
                        match archive.placement() {
                            apimokka_model::RestorePlacement::RuleSetRoot { .. } => {
                                ExpectedParent::None
                            }
                            apimokka_model::RestorePlacement::Rule { parent, .. } => {
                                ExpectedParent::Existing(parent.0)
                            }
                        }
                    };
                    expected.push(CreationExpectation {
                        key: node.key.clone(),
                        kind: node.payload.kind(),
                        parent,
                    });
                }
            }
            _ => {}
        }
    }
    expected
}

fn collect_condition_expectations(
    expected: &mut Vec<CreationExpectation>,
    rule: &apimokka_model::RuleEditPayload,
    parent: ExpectedParent,
) {
    let created_rule_key = match &parent {
        ExpectedParent::Created(key) => Some(key.clone()),
        _ => None,
    };
    let existing_parent = match parent {
        ExpectedParent::Existing(id) => Some(id),
        _ => None,
    };
    if let CollectionEdit::Replace(headers) = &rule.headers {
        for condition in headers {
            if let ConditionEdit::Create { key, .. } = condition {
                expected.push(CreationExpectation {
                    key: key.clone(),
                    kind: WorkspaceNodeKind::HeaderCondition,
                    parent: existing_parent
                        .map(ExpectedParent::Existing)
                        .unwrap_or_else(|| {
                            ExpectedParent::Created(created_rule_key.clone().unwrap())
                        }),
                });
            }
        }
    }
    if let CollectionEdit::Replace(body) = &rule.body {
        for condition in body {
            if let ConditionEdit::Create { key, .. } = condition {
                expected.push(CreationExpectation {
                    key: key.clone(),
                    kind: WorkspaceNodeKind::BodyCondition,
                    parent: existing_parent
                        .map(ExpectedParent::Existing)
                        .unwrap_or_else(|| {
                            ExpectedParent::Created(created_rule_key.clone().unwrap())
                        }),
                });
            }
        }
    }
}

fn rebind_expectations(
    transaction: &EditTransaction,
) -> Vec<(NodeId, WorkspaceNodeKind, SemanticCreationKey)> {
    transaction
        .intents()
        .iter()
        .flat_map(|intent| match intent {
            EditIntent::RestoreSubtree { archive } => archive
                .nodes()
                .iter()
                .map(|node| (node.old_id, node.payload.kind(), node.key.clone()))
                .collect(),
            _ => Vec::new(),
        })
        .collect()
}

fn validate_correlations(
    before: &PortSnapshot,
    expected: &[CreationExpectation],
    expected_rebinds: &[(NodeId, WorkspaceNodeKind, SemanticCreationKey)],
    outcome: &EditOutcome,
) -> Result<(), String> {
    let before_ids = editable_node_index(before)?;
    let after_ids = editable_node_index(&outcome.snapshot)?;
    if expected.len() != outcome.creations.len() {
        return Err(format!(
            "creation receipt count mismatch: expected {}, received {}",
            expected.len(),
            outcome.creations.len()
        ));
    }
    let mut seen_keys = std::collections::HashSet::new();
    let mut seen_ids = std::collections::HashSet::new();
    for expectation in expected {
        let matches = outcome
            .creations
            .iter()
            .filter(|receipt| receipt.key == expectation.key)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(format!(
                "creation key {} was not reported exactly once",
                expectation.key.as_str()
            ));
        }
        let receipt = matches[0];
        if receipt.kind != expectation.kind {
            return Err(format!(
                "creation key {} reported the wrong node kind",
                expectation.key.as_str()
            ));
        }
        if !seen_keys.insert(receipt.key.as_str()) || !seen_ids.insert(receipt.new_id) {
            return Err("creation receipts were not bijective".into());
        }
        if before_ids.contains_key(&receipt.new_id) {
            return Err(format!(
                "creation key {} reused a preexisting node identity",
                expectation.key.as_str()
            ));
        }
        let expected_parent = match &expectation.parent {
            ExpectedParent::None => None,
            ExpectedParent::Existing(id) => Some(*id),
            ExpectedParent::Created(key) => outcome
                .creations
                .iter()
                .find(|candidate| candidate.key == *key)
                .map(|candidate| candidate.new_id),
        };
        if receipt.parent != expected_parent {
            return Err(format!(
                "creation key {} reported the wrong parent",
                expectation.key.as_str()
            ));
        }
        let Some((actual_kind, actual_parent)) = after_ids.get(&receipt.new_id) else {
            return Err(format!(
                "creation key {} reported a node absent from the adopted snapshot",
                expectation.key.as_str()
            ));
        };
        if *actual_kind != receipt.kind || *actual_parent != receipt.parent {
            return Err(format!(
                "creation key {} disagrees with the adopted snapshot kind or parent",
                expectation.key.as_str()
            ));
        }
    }
    if expected_rebinds.len() != outcome.rebound_nodes.len() {
        return Err(format!(
            "rebind count mismatch: expected {}, received {}",
            expected_rebinds.len(),
            outcome.rebound_nodes.len()
        ));
    }
    let mut old_ids = std::collections::HashSet::new();
    let mut new_ids = std::collections::HashSet::new();
    for (old_id, kind, key) in expected_rebinds {
        let matches = outcome
            .rebound_nodes
            .iter()
            .filter(|rebind| rebind.old_id == *old_id && rebind.kind == *kind)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(format!(
                "node {old_id:?} was not rebound exactly once with its expected kind"
            ));
        }
        if !old_ids.insert(*old_id) || !new_ids.insert(matches[0].new_id) {
            return Err("rebinds were not bijective".into());
        }
        let Some(creation) = outcome
            .creations
            .iter()
            .find(|creation| creation.key == *key)
        else {
            return Err(format!(
                "rebound node {old_id:?} has no matching creation receipt"
            ));
        };
        if creation.new_id != matches[0].new_id {
            return Err(format!(
                "rebound node {old_id:?} disagrees with its creation receipt"
            ));
        }
    }
    Ok(())
}

fn editable_node_index(
    snapshot: &PortSnapshot,
) -> Result<HashMap<NodeId, (WorkspaceNodeKind, Option<NodeId>)>, String> {
    let mut index = HashMap::new();
    for rule_set in &snapshot.workspace().rule_sets {
        insert_editable_node(
            &mut index,
            rule_set.id.0,
            (WorkspaceNodeKind::RuleSet, None),
        )?;
        for rule in &rule_set.rules {
            insert_editable_node(
                &mut index,
                rule.id,
                (WorkspaceNodeKind::Rule, Some(rule_set.id.0)),
            )?;
            let Some(canonical) = snapshot.rule(rule.id) else {
                return Err(format!(
                    "adopted snapshot has no canonical rule view for {:?}",
                    rule.id
                ));
            };
            for condition in &canonical.conditions().headers {
                insert_editable_node(
                    &mut index,
                    condition.id,
                    (WorkspaceNodeKind::HeaderCondition, Some(rule.id)),
                )?;
            }
            for condition in &canonical.conditions().body {
                insert_editable_node(
                    &mut index,
                    condition.id,
                    (WorkspaceNodeKind::BodyCondition, Some(rule.id)),
                )?;
            }
        }
    }
    Ok(index)
}

fn insert_editable_node(
    index: &mut HashMap<NodeId, (WorkspaceNodeKind, Option<NodeId>)>,
    id: NodeId,
    value: (WorkspaceNodeKind, Option<NodeId>),
) -> Result<(), String> {
    if index.insert(id, value).is_some() {
        Err(format!(
            "adopted snapshot contains duplicate node identity {id:?}"
        ))
    } else {
        Ok(())
    }
}

impl Deref for WorkspaceSession {
    type Target = WorkspaceSnapshot;

    fn deref(&self) -> &Self::Target {
        self.latest.workspace()
    }
}

fn prototype_from_seed(seed: &WorkspaceSnapshot) -> PrototypeState {
    let rule_extras = seed
        .rule_sets
        .iter()
        .flat_map(|rule_set| &rule_set.rules)
        .map(|rule| {
            (
                rule.id,
                RulePrototype {
                    weight: rule.payload.weight,
                    priority: rule.payload.priority,
                },
            )
        })
        .collect();
    let root = &seed.root_settings;
    PrototypeState {
        rule_extras,
        trace: Some(TracePrototypeSettings {
            enabled: root.trace_enabled,
            transport: root.trace_transport,
            uds_path: root.trace_uds_path.clone(),
            tcp_addr: root.trace_tcp_addr.clone(),
            queue_size: root.trace_queue_size,
        }),
    }
}

#[cfg(test)]
mod correlation_index_tests {
    use super::*;

    #[test]
    fn duplicate_snapshot_identity_is_rejected() {
        let id = NodeId::new();
        let mut index = HashMap::new();
        insert_editable_node(&mut index, id, (WorkspaceNodeKind::RuleSet, None)).unwrap();
        let error = insert_editable_node(
            &mut index,
            id,
            (WorkspaceNodeKind::Rule, Some(NodeId::new())),
        )
        .unwrap_err();
        assert!(error.contains("duplicate node identity"));
    }
}
