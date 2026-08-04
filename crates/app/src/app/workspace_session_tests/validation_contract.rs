//! Validation equality, mismatch, and ordering contract between the cached
//! canonical workspace and what the port reports.

use super::*;
use crate::message::Message;
use apimokka_model::{
    ApplyFailure, Diagnostic, EditOutcome, EditTransaction, MemoryWorkspace, NodeId, PortSnapshot,
    SaveFailure, SaveOutcome, Severity, ValidationReport, WorkspacePort,
};

struct ValidationMismatchPort {
    inner: MemoryWorkspace,
    report: ValidationReport,
}

impl WorkspacePort for ValidationMismatchPort {
    fn snapshot(&self) -> PortSnapshot {
        self.inner.snapshot()
    }

    fn apply(&mut self, transaction: EditTransaction) -> Result<EditOutcome, ApplyFailure> {
        self.inner.apply(transaction)
    }

    fn validate(&self) -> ValidationReport {
        self.report.clone()
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

#[test]
fn explicit_validation_equality_is_non_mutating() {
    let mut app = expert();
    let selection = app.selection.clone();
    let dirty = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .dirty_files()
        .to_vec();
    app.update(Message::OpenValidationDrawer);

    let session = app.snapshot.as_ref().unwrap();
    assert!(!session.faulted);
    assert_eq!(app.selection, selection);
    assert_eq!(app.transient_problem_kind, None);
    assert_eq!(session.latest().dirty_files(), dirty);
    assert!(app.last_problem.is_none());
}

#[test]
fn validation_mismatch_faults_without_adoption_and_preserves_cached_truth() {
    let mut app = expert();
    let seed = apimokka_model::mock::shop_api_canonical_seed();
    let session = WorkspaceSession::from_port(
        Box::new(ValidationMismatchPort {
            inner: MemoryWorkspace::new(seed).unwrap(),
            report: ValidationReport::default(),
        }),
        workspace_session::PrototypeState::default(),
    );
    let rule = session.rule_sets[0].rules[0].id;
    let parent = session.rule_sets[0].id;
    app.snapshot = Some(session);
    app.selection.select_rule(rule, parent);
    app.update(Message::HeaderAdd);
    app.update(Message::RuleWeightChanged("37".into()));
    app.update(Message::SettingsSetHost("invalid-ip".into()));

    let cached_name = app.snapshot.as_ref().unwrap().meta.name.clone();
    let cached_path = app.snapshot.as_ref().unwrap().meta.path.clone();
    let selection = app.selection.clone();
    assert!(!app.undo_stack().is_empty());
    assert!(app.snapshot.as_ref().unwrap().condition_focus.is_some());

    app.update(Message::OpenValidationDrawer);

    let session = app.snapshot.as_ref().unwrap();
    assert!(session.faulted);
    assert_eq!(
        session.contract_fault_adoption,
        Some(workspace_session::ContractFaultAdoption::NonAdoptingRead)
    );
    assert_eq!(session.meta.name, cached_name);
    assert_eq!(session.meta.path, cached_path);
    assert_eq!(app.selection, selection);
    assert_eq!(
        app.transient_problem_kind,
        Some(TransientProblemKind::NonAdoptingReadContract)
    );
    assert_eq!(session.prototype.rule_extras[&rule].weight, Some(37));
    assert!(session.rule_drafts.is_empty());
    assert!(session.condition_focus.is_none());
    assert!(session.undo_stack.is_empty() && session.redo_stack.is_empty());
    assert!(
        app.last_problem
            .as_ref()
            .unwrap()
            .detail
            .contains("cached canonical workspace was retained")
    );
    let original_problem = app.last_problem.clone();
    app.update(Message::OpenValidationDrawer);
    assert_eq!(app.last_problem, original_problem);
}

#[test]
fn validation_order_and_unknown_targets_are_contract_checked() {
    let seed = apimokka_model::mock::shop_api_canonical_seed();
    let inner = MemoryWorkspace::new(seed).unwrap();
    let mut reordered = inner.validate();
    reordered.issues.reverse();
    let mut app = expert();
    app.snapshot = Some(WorkspaceSession::from_port(
        Box::new(ValidationMismatchPort {
            inner,
            report: reordered,
        }),
        workspace_session::PrototypeState::default(),
    ));
    app.update(Message::OpenValidationDrawer);
    assert!(app.snapshot.as_ref().unwrap().faulted);
    assert!(
        app.last_problem
            .as_ref()
            .unwrap()
            .technical_detail
            .as_deref()
            .unwrap()
            .contains("first difference at index")
    );

    let mut unknown_seed = apimokka_model::mock::shop_api_canonical_seed();
    unknown_seed.diagnostics.push(Diagnostic {
        node_id: Some(NodeId::new()),
        severity: Severity::Error,
        message: "unknown target".into(),
    });
    let mut unknown = expert();
    unknown.snapshot = Some(WorkspaceSession::new(unknown_seed).unwrap());
    unknown.update(Message::OpenValidationDrawer);
    assert!(unknown.snapshot.as_ref().unwrap().faulted);
    assert!(
        unknown
            .snapshot
            .as_ref()
            .unwrap()
            .contract_fault
            .as_deref()
            .unwrap()
            .contains("unknown editable node")
    );
}
