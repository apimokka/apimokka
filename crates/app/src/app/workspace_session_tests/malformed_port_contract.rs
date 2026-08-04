//! Fail-closed behaviour when a port's apply outcome violates its contract:
//! missing/unexpected/reused receipts, kind or parent mismatches, dangling
//! rebinds, and identity drift. Each must fault the session closed without
//! corrupting undo/redo ownership or admitted UI identity reads.

use super::*;
use crate::message::Message;
use apimokka_model::{
    ApplyFailure, CreationReceipt, Diagnostic, EditOutcome, EditTransaction, MemoryWorkspace,
    NodeId, PortSnapshot, SaveFailure, SaveOutcome, SemanticCreationKey, Severity,
    ValidationReport, WorkspaceNodeKind, WorkspacePort,
};
use std::cell::RefCell;
use std::rc::Rc;

struct MissingReceiptPort {
    inner: MemoryWorkspace,
    calls: Rc<RefCell<PortCallCounts>>,
}

#[derive(Default)]
struct PortCallCounts {
    apply: usize,
    save: usize,
}

impl WorkspacePort for MissingReceiptPort {
    fn snapshot(&self) -> PortSnapshot {
        self.inner.snapshot()
    }

    fn apply(&mut self, transaction: EditTransaction) -> Result<EditOutcome, ApplyFailure> {
        self.calls.borrow_mut().apply += 1;
        let mut outcome = self.inner.apply(transaction)?;
        outcome.creations.clear();
        Ok(outcome)
    }

    fn validate(&self) -> ValidationReport {
        self.inner.validate()
    }
    fn save(&mut self) -> Result<SaveOutcome, SaveFailure> {
        self.calls.borrow_mut().save += 1;
        self.inner.save()
    }
    fn acknowledge_reload(&mut self) -> PortSnapshot {
        self.inner.acknowledge_reload()
    }
    fn acknowledge_restart(&mut self) -> PortSnapshot {
        self.inner.acknowledge_restart()
    }
}

#[derive(Clone, Copy)]
enum ApplyBehavior {
    Fail,
    UnexpectedReceipt,
    NonexistentReceipt,
    ReusedReceipt,
    ActualKindMismatch,
    ActualParentMismatch,
    RebindNonexistent,
    IdentityDrift,
}

struct ControlledPort {
    inner: MemoryWorkspace,
    calls: usize,
    behavior_at: usize,
    behavior: ApplyBehavior,
}

impl WorkspacePort for ControlledPort {
    fn snapshot(&self) -> PortSnapshot {
        self.inner.snapshot()
    }

    fn apply(&mut self, transaction: EditTransaction) -> Result<EditOutcome, ApplyFailure> {
        self.calls += 1;
        if self.calls == self.behavior_at && matches!(self.behavior, ApplyBehavior::Fail) {
            return Err(ApplyFailure {
                diagnostic: Diagnostic {
                    node_id: None,
                    severity: Severity::Error,
                    message: "injected pre-commit failure".into(),
                },
            });
        }
        let mut outcome = self.inner.apply(transaction)?;
        if self.calls == self.behavior_at {
            match self.behavior {
                ApplyBehavior::UnexpectedReceipt => {
                    outcome.creations.push(CreationReceipt {
                        key: SemanticCreationKey::new("fake/unexpected").unwrap(),
                        kind: WorkspaceNodeKind::RuleSet,
                        parent: None,
                        new_id: NodeId::new(),
                    });
                }
                ApplyBehavior::NonexistentReceipt => {
                    outcome.creations[0].new_id = NodeId::new();
                }
                ApplyBehavior::ReusedReceipt => {
                    outcome.creations[0].new_id = outcome.snapshot.workspace().rule_sets[0].id.0;
                }
                ApplyBehavior::ActualKindMismatch => {
                    let receipt_id = outcome.creations[0].new_id;
                    let mut workspace = outcome.snapshot.clone().into_legacy_workspace();
                    let created_set = workspace
                        .rule_sets
                        .iter_mut()
                        .find(|set| set.id.0 == receipt_id)
                        .unwrap();
                    created_set.id = apimokka_model::RuleSetId(NodeId::new());
                    workspace.rule_sets[0].rules[0].id = receipt_id;
                    outcome.snapshot = MemoryWorkspace::new(workspace).unwrap().snapshot();
                }
                ApplyBehavior::ActualParentMismatch => {
                    let receipt = outcome
                        .creations
                        .iter()
                        .find(|receipt| receipt.kind == WorkspaceNodeKind::Rule)
                        .unwrap();
                    let mut workspace = outcome.snapshot.clone().into_legacy_workspace();
                    let source_index = workspace
                        .rule_sets
                        .iter()
                        .position(|set| set.id.0 == receipt.parent.unwrap())
                        .unwrap();
                    let rule_index = workspace.rule_sets[source_index]
                        .rules
                        .iter()
                        .position(|rule| rule.id == receipt.new_id)
                        .unwrap();
                    let rule = workspace.rule_sets[source_index].rules.remove(rule_index);
                    let target_index = (0..workspace.rule_sets.len())
                        .find(|index| *index != source_index)
                        .unwrap();
                    workspace.rule_sets[target_index].rules.push(rule);
                    outcome.snapshot = MemoryWorkspace::new(workspace).unwrap().snapshot();
                }
                ApplyBehavior::RebindNonexistent => {
                    let nonexistent = NodeId::new();
                    outcome.creations[0].new_id = nonexistent;
                    outcome.rebound_nodes[0].new_id = nonexistent;
                }
                ApplyBehavior::IdentityDrift => {
                    let mut workspace = outcome.snapshot.clone().into_legacy_workspace();
                    workspace.meta.name = "untrusted-drift".into();
                    outcome.snapshot = MemoryWorkspace::new(workspace).unwrap().snapshot();
                }
                ApplyBehavior::Fail => {}
            }
        }
        Ok(outcome)
    }

    fn validate(&self) -> ValidationReport {
        self.inner.validate()
    }

    fn save(&mut self) -> Result<SaveOutcome, SaveFailure> {
        self.inner.save()
    }

    fn acknowledge_reload(&mut self) -> PortSnapshot {
        self.inner.acknowledge_reload()
    }

    fn acknowledge_restart(&mut self) -> PortSnapshot {
        self.inner.acknowledge_restart()
    }
}

fn controlled_app(behavior_at: usize, behavior: ApplyBehavior) -> App {
    let mut app = expert();
    let seed = apimokka_model::mock::shop_api_canonical_seed();
    let port = ControlledPort {
        inner: MemoryWorkspace::new(seed).unwrap(),
        calls: 0,
        behavior_at,
        behavior,
    };
    let session =
        WorkspaceSession::from_port(Box::new(port), workspace_session::PrototypeState::default());
    let rule_set = session.rule_sets[0].id;
    let rule = session.rule_sets[0].rules[0].id;
    app.snapshot = Some(session);
    app.selection.rule_set = Some(rule_set);
    app.selection.rule = Some(rule);
    app.recompute_dirty();
    app
}

#[test]
fn malformed_success_is_adopted_then_faults_session_closed() {
    let mut app = expert();
    let seed = apimokka_model::mock::shop_api_canonical_seed();
    let calls = Rc::new(RefCell::new(PortCallCounts::default()));
    let port = MissingReceiptPort {
        inner: MemoryWorkspace::new(seed).unwrap(),
        calls: calls.clone(),
    };
    app.snapshot = Some(WorkspaceSession::from_port(
        Box::new(port),
        workspace_session::PrototypeState::default(),
    ));

    app.update(Message::AddRuleSet);
    let count_after_fault = app.snapshot.as_ref().unwrap().rule_sets.len();
    assert!(app.snapshot.as_ref().unwrap().faulted);
    assert!(app.last_problem.is_some());
    assert!(app.undo_stack().is_empty() && app.redo_stack().is_empty());

    app.update(Message::AddRuleSet);
    assert_eq!(
        app.snapshot.as_ref().unwrap().rule_sets.len(),
        count_after_fault
    );
    app.update(Message::RuleSetMethod("POST".into()));
    app.update(Message::Save);
    app.update(Message::Undo);
    app.update(Message::Redo);
    assert!(app.snapshot.as_ref().unwrap().rule_drafts.is_empty());
    assert_eq!(calls.borrow().apply, 1);
    assert_eq!(calls.borrow().save, 0);
}

#[test]
fn malformed_success_clears_root_drafts_and_prepopulated_history() {
    let mut app = controlled_app(1, ApplyBehavior::UnexpectedReceipt);
    app.update(Message::SettingsSetHost("invalid-ip".into()));
    app.update(Message::RuleWeightChanged("17".into()));
    assert!(!app.undo_stack().is_empty());

    app.update(Message::AddRuleSet);

    let session = app.snapshot.as_ref().unwrap();
    assert!(session.faulted);
    assert!(session.undo_stack.is_empty() && session.redo_stack.is_empty());
    assert!(session.rule_drafts.is_empty());
    assert_eq!(
        session.root_drafts.listener_ip,
        session.root_settings.listener_ip
    );
}

#[test]
fn malformed_ordinary_edit_adopts_snapshot_and_reconciles_removed_selection() {
    let mut app = controlled_app(1, ApplyBehavior::UnexpectedReceipt);
    let removed = app.selection.rule.unwrap();
    app.update(Message::DeleteRule(removed));

    let session = app.snapshot.as_ref().unwrap();
    assert!(session.faulted);
    assert!(session.find_rule(removed).is_none());
    assert_eq!(app.selection.rule, None);
    assert!(session.undo_stack.is_empty() && session.redo_stack.is_empty());
}

#[test]
fn malformed_undo_and_redo_do_not_repopulate_the_cleared_stacks() {
    let mut undo_app = controlled_app(2, ApplyBehavior::UnexpectedReceipt);
    undo_app.update(Message::RuleSetUrlPath("/undo-fault".into()));
    undo_app.update(Message::SettingsSetHost("invalid-ip".into()));
    undo_app.update(Message::Undo);
    let session = undo_app.snapshot.as_ref().unwrap();
    assert!(session.faulted);
    assert!(session.undo_stack.is_empty() && session.redo_stack.is_empty());
    assert_eq!(
        session.root_drafts.listener_ip,
        session.root_settings.listener_ip
    );

    let mut redo_app = controlled_app(3, ApplyBehavior::UnexpectedReceipt);
    redo_app.update(Message::RuleSetUrlPath("/redo-fault".into()));
    redo_app.update(Message::Undo);
    redo_app.update(Message::SettingsSetHost("invalid-ip".into()));
    redo_app.update(Message::Redo);
    let session = redo_app.snapshot.as_ref().unwrap();
    assert!(session.faulted);
    assert!(session.undo_stack.is_empty() && session.redo_stack.is_empty());
    assert_eq!(
        session.root_drafts.listener_ip,
        session.root_settings.listener_ip
    );

    let mut selected_undo = controlled_app(2, ApplyBehavior::UnexpectedReceipt);
    let parent = selected_undo.selection.rule_set.unwrap();
    selected_undo.update(Message::AddRule(parent));
    let added = selected_undo.selection.rule.unwrap();
    selected_undo.update(Message::Undo);
    assert!(selected_undo.snapshot.as_ref().unwrap().faulted);
    assert!(
        selected_undo
            .snapshot
            .as_ref()
            .unwrap()
            .find_rule(added)
            .is_none()
    );
    assert_eq!(selected_undo.selection.rule, None);

    let mut selected_redo = controlled_app(3, ApplyBehavior::UnexpectedReceipt);
    let deleted = selected_redo.selection.rule.unwrap();
    selected_redo.update(Message::DeleteRule(deleted));
    selected_redo.update(Message::Undo);
    assert!(selected_redo.selection.rule.is_some());
    selected_redo.update(Message::Redo);
    assert!(selected_redo.snapshot.as_ref().unwrap().faulted);
    assert_eq!(selected_redo.selection.rule, None);
}

#[test]
fn precommit_delete_and_undo_failures_preserve_selection_and_stack_ownership() {
    let mut delete_app = controlled_app(1, ApplyBehavior::Fail);
    let selected = delete_app.selection.clone();
    let rule = selected.rule.unwrap();
    delete_app.update(Message::DeleteRule(rule));
    assert_eq!(delete_app.selection.rule, selected.rule);
    assert_eq!(delete_app.selection.rule_set, selected.rule_set);
    assert!(delete_app.undo_stack().is_empty());

    let mut set_delete_app = controlled_app(1, ApplyBehavior::Fail);
    let selected_set = set_delete_app.selection.rule_set.unwrap();
    let selected_rule = set_delete_app.selection.rule;
    set_delete_app.update(Message::DeleteRuleSet(selected_set));
    set_delete_app.update(Message::ConfirmProceed);
    assert_eq!(set_delete_app.selection.rule_set, Some(selected_set));
    assert_eq!(set_delete_app.selection.rule, selected_rule);
    assert!(set_delete_app.undo_stack().is_empty());

    let mut undo_app = controlled_app(2, ApplyBehavior::Fail);
    let undo_rule = undo_app.selection.rule.unwrap();
    undo_app.update(Message::RuleSetUrlPath("/precommit".into()));
    let canonical = undo_app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(undo_rule)
        .unwrap()
        .rule_match()
        .clone();
    undo_app.update(Message::Undo);
    let session = undo_app.snapshot.as_ref().unwrap();
    assert_eq!(
        session.latest().rule(undo_rule).unwrap().rule_match(),
        &canonical
    );
    assert_eq!(session.undo_stack.len(), 1);
    assert!(session.redo_stack.is_empty());

    let mut redo_app = controlled_app(3, ApplyBehavior::Fail);
    let redo_rule = redo_app.selection.rule.unwrap();
    let original = redo_app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(redo_rule)
        .unwrap()
        .rule_match()
        .clone();
    redo_app.update(Message::RuleSetUrlPath("/redo-precommit".into()));
    redo_app.update(Message::Undo);
    redo_app.update(Message::Redo);
    let session = redo_app.snapshot.as_ref().unwrap();
    assert_eq!(
        session.latest().rule(redo_rule).unwrap().rule_match(),
        &original
    );
    assert!(session.undo_stack.is_empty());
    assert_eq!(session.redo_stack.len(), 1);
}

#[test]
fn receipts_must_reference_fresh_nodes_in_the_adopted_snapshot() {
    for behavior in [
        ApplyBehavior::NonexistentReceipt,
        ApplyBehavior::ReusedReceipt,
    ] {
        let mut app = controlled_app(1, behavior);
        app.update(Message::AddRuleSet);
        assert!(app.snapshot.as_ref().unwrap().faulted);
        assert!(
            app.snapshot
                .as_ref()
                .unwrap()
                .contract_fault
                .as_ref()
                .is_some_and(|fault| fault.contains("absent") || fault.contains("preexisting"))
        );
    }
}

#[test]
fn receipts_must_match_actual_snapshot_kind_and_parent() {
    let mut kind_app = controlled_app(1, ApplyBehavior::ActualKindMismatch);
    kind_app.update(Message::AddRuleSet);
    assert!(kind_app.snapshot.as_ref().unwrap().faulted);
    assert!(
        kind_app
            .snapshot
            .as_ref()
            .unwrap()
            .contract_fault
            .as_deref()
            .unwrap()
            .contains("kind or parent")
    );

    let mut parent_app = controlled_app(1, ApplyBehavior::ActualParentMismatch);
    let parent = parent_app.selection.rule_set.unwrap();
    parent_app.update(Message::AddRule(parent));
    assert!(parent_app.snapshot.as_ref().unwrap().faulted);
    assert!(
        parent_app
            .snapshot
            .as_ref()
            .unwrap()
            .contract_fault
            .as_deref()
            .unwrap()
            .contains("kind or parent")
    );
}

#[test]
fn restore_rebind_must_reference_the_fresh_node_in_the_adopted_snapshot() {
    let mut app = controlled_app(2, ApplyBehavior::RebindNonexistent);
    let deleted = app.selection.rule.unwrap();
    app.update(Message::DeleteRule(deleted));
    app.update(Message::Undo);

    let session = app.snapshot.as_ref().unwrap();
    assert!(session.faulted);
    assert!(
        session
            .contract_fault
            .as_deref()
            .unwrap()
            .contains("absent from the adopted snapshot")
    );
    assert!(session.undo_stack.is_empty() && session.redo_stack.is_empty());
}

#[test]
fn identity_drift_faults_closed_but_all_ui_identity_reads_stay_admitted() {
    let mut app = controlled_app(1, ApplyBehavior::IdentityDrift);
    let admitted = app.snapshot.as_ref().unwrap().identity.clone();
    app.update(Message::RuleSetUrlPath("/identity-drift".into()));

    let session = app.snapshot.as_ref().unwrap();
    assert!(session.faulted);
    assert_eq!(session.identity, admitted);
    assert!(
        session
            .contract_fault
            .as_deref()
            .unwrap()
            .contains("identity drifted")
    );
    assert_eq!(app.title(), format!("{} — apimokka", admitted.name));
    let _ = crate::shell::top_bar::view(&app);
    let _ = crate::screens::workspace_menu::view(&app);
    let _ = crate::screens::settings::view(&app);
}
