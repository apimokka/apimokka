//! MK-053 app-owned workspace session and editor-draft state.

use std::collections::HashMap;
use std::ops::Deref;

use super::HistoryEntry;

use apimokka_model::workspace_port::{
    map_body_condition, map_header_condition, map_response, map_rule_match,
};
use apimokka_model::{
    ApplyFailure, BodyConditionPayload, CollectionEdit, ConditionEdit, EditIntent, EditOutcome,
    EditTransaction, HeaderConditionPayload, MemoryWorkspace, NodeId, PortSnapshot, RulePayload,
    SemanticCreationKey, WorkspaceNodeKind, WorkspacePort, WorkspaceSnapshot,
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
    pub faulted: bool,
    pub contract_fault: Option<String>,
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

impl WorkspaceSession {
    pub fn new(seed: WorkspaceSnapshot) -> Result<Self, apimokka_model::FieldError> {
        let prototype = prototype_from_seed(&seed);
        let port = MemoryWorkspace::new(seed)?;
        Ok(Self::from_port(Box::new(port), prototype))
    }

    pub(crate) fn from_port(port: Box<dyn WorkspacePort>, prototype: PrototypeState) -> Self {
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
            faulted: false,
            contract_fault: None,
            next_creation_key: 0,
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
        SessionApplyResult::Validated(Box::new(outcome))
    }

    pub(super) fn save(&mut self) -> SessionSaveResult {
        if self.faulted {
            return SessionSaveResult::ContractFault;
        }
        match self.port.save() {
            Ok(outcome) => {
                if let Err(problem) = self.adopt_snapshot(outcome.snapshot.clone()) {
                    self.enter_contract_fault(problem);
                    SessionSaveResult::ContractFault
                } else {
                    SessionSaveResult::Saved
                }
            }
            Err(failure) => {
                if let Err(problem) = self.adopt_snapshot((*failure.snapshot).clone()) {
                    self.enter_contract_fault(problem);
                    SessionSaveResult::ContractFault
                } else {
                    SessionSaveResult::SaveFailure(failure)
                }
            }
        }
    }

    pub fn acknowledge_reload(&mut self) {
        if self.faulted {
            return;
        }
        let snapshot = self.port.acknowledge_reload();
        if let Err(problem) = self.adopt_snapshot(snapshot) {
            self.enter_contract_fault(problem);
        }
    }

    pub fn acknowledge_restart(&mut self) {
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
        self.rule_drafts.clear();
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
    }
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
