use super::*;
use crate::message::Message;
use apimokka_model::{
    ApplyFailure, CreationReceipt, Diagnostic, EditIntent, EditOutcome, EditTransaction,
    MemoryWorkspace, NodeId, PortSnapshot, SaveFailure, SaveOutcome, SemanticCreationKey, Severity,
    ValidationReport, WorkspaceNodeKind, WorkspacePort,
};
use std::cell::RefCell;
use std::rc::Rc;

fn expert() -> App {
    let mut app = App::new().0;
    app.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    app.update(Message::OpenWorkspace("test".into()));
    let first_rule = app.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    app.update(Message::SelectRule(first_rule));
    app
}

#[test]
fn invalid_rule_draft_survives_without_changing_canonical_state() {
    let mut app = expert();
    let rule_id = app.selection.rule.unwrap();
    let canonical_before = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule_id)
        .unwrap()
        .rule_match()
        .clone();

    app.update(Message::RuleSetMethod("PATCH".into()));

    let session = app.snapshot.as_ref().unwrap();
    assert_eq!(session.rule_draft(rule_id).unwrap().payload.method, "PATCH");
    assert_eq!(
        session.latest().rule(rule_id).unwrap().rule_match(),
        &canonical_before
    );
    assert!(app.last_problem.is_some());
}

#[test]
fn only_the_corrected_operation_dismisses_its_transient_problem() {
    let mut app = expert();

    app.update(Message::SettingsSetPort("abc".into()));
    assert_eq!(
        app.transient_problem_kind,
        Some(TransientProblemKind::Operation)
    );
    let rejected_title = app.last_problem.as_ref().unwrap().title.clone();

    app.update(Message::RuleSetMethod("POST".into()));
    assert_eq!(
        app.last_problem.as_ref().unwrap().title,
        rejected_title,
        "an unrelated successful edit must retain the field problem"
    );

    app.update(Message::SettingsSetPort("9000".into()));
    assert!(app.last_problem.is_none());
    assert_eq!(app.transient_problem_kind, None);
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.listener_port,
        9000
    );
}

#[test]
fn pending_condition_draft_survives_navigation() {
    let mut app = expert();
    let rule_id = app.selection.rule.unwrap();
    let other = app.snapshot.as_ref().unwrap().rule_sets[0].rules[1].id;
    app.update(Message::HeaderAdd);
    let index = app
        .snapshot
        .as_ref()
        .unwrap()
        .rule_draft(rule_id)
        .unwrap()
        .payload
        .headers
        .len()
        - 1;
    app.update(Message::HeaderSetName {
        index,
        value: "not a header".into(),
    });
    app.update(Message::SelectRule(other));
    app.update(Message::SelectRule(rule_id));

    let draft = app.snapshot.as_ref().unwrap().rule_draft(rule_id).unwrap();
    assert_eq!(draft.payload.headers[index].name, "not a header");
    assert!(matches!(
        draft.header_bindings[index],
        DraftBinding::Pending(_)
    ));
}

#[test]
fn semantic_noop_does_not_record_history_or_dirty_an_extra_file() {
    let mut app = expert();
    let method = app.selected_rule_payload().unwrap().method.clone();
    let history = app.undo_stack().len();
    let dirty = app.dirty_count;

    app.update(Message::RuleSetMethod(method));

    assert_eq!(app.undo_stack().len(), history);
    assert_eq!(app.dirty_count, dirty);
}

#[test]
fn absent_delay_remains_absent_when_an_unrelated_response_field_changes() {
    let mut app = expert();
    let rule_id = app.selection.rule.unwrap();
    app.update(Message::RespondSetDelay(String::new()));
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule_id)
            .unwrap()
            .respond()
            .delay_milliseconds(),
        None
    );
    app.snapshot.as_mut().unwrap().rule_drafts.remove(&rule_id);

    app.update(Message::RespondSetStatus("201 Created".into()));

    let session = app.snapshot.as_ref().unwrap();
    assert_eq!(
        session
            .latest()
            .rule(rule_id)
            .unwrap()
            .respond()
            .delay_milliseconds(),
        None
    );
    assert_eq!(session.rule_draft(rule_id).unwrap().response_delay, "");
}

#[test]
fn semantic_noop_projects_the_accepted_root_value_without_history_or_dirty_change() {
    let mut app = expert();
    let history = app.undo_stack().len();
    let dirty = app.dirty_count;
    let port = app.snapshot.as_ref().unwrap().root_settings.listener_port;
    let runtime_pending = app.snapshot.as_ref().unwrap().latest().runtime_pending();
    let unsaved_hint = app.snapshot.as_ref().unwrap().latest().unsaved_hint();

    app.update(Message::SettingsSetPort(format!("0{port}")));

    assert_eq!(
        app.snapshot.as_ref().unwrap().root_drafts.listener_port,
        port.to_string()
    );
    assert_eq!(app.undo_stack().len(), history);
    assert_eq!(app.dirty_count, dirty);
    assert_eq!(
        app.snapshot.as_ref().unwrap().latest().runtime_pending(),
        runtime_pending
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().latest().unsaved_hint(),
        unsaved_hint
    );
}

#[test]
fn undo_and_redo_preserve_unrelated_invalid_rule_and_root_drafts() {
    let mut app = expert();
    let rule_id = app.selection.rule.unwrap();
    app.update(Message::RuleSetMethod("PATCH".into()));
    app.update(Message::SettingsSetHost("invalid-ip".into()));
    app.update(Message::RespondSetStatus("201 Created".into()));
    app.update(Message::RespondSetDelay("-1".into()));

    app.update(Message::Undo);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .rule_draft(rule_id)
            .unwrap()
            .payload
            .method,
        "PATCH"
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_drafts.listener_ip,
        "invalid-ip"
    );
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .rule_draft(rule_id)
            .unwrap()
            .response_delay,
        "-1"
    );

    app.update(Message::Redo);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .rule_draft(rule_id)
            .unwrap()
            .payload
            .method,
        "PATCH"
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_drafts.listener_ip,
        "invalid-ip"
    );
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .rule_draft(rule_id)
            .unwrap()
            .response_delay,
        "-1"
    );

    let mut rule_app = expert();
    let rule_id = rule_app.selection.rule.unwrap();
    rule_app.update(Message::RuleSetUrlPath("/field-scoped".into()));
    rule_app.update(Message::RuleSetMethod("PATCH".into()));
    rule_app.update(Message::Undo);
    assert_eq!(
        rule_app
            .snapshot
            .as_ref()
            .unwrap()
            .rule_draft(rule_id)
            .unwrap()
            .payload
            .method,
        "PATCH"
    );
}

#[test]
fn refresh_acknowledgement_preserves_unrelated_invalid_drafts() {
    let mut app = expert();
    let rule = app.selection.rule.unwrap();
    app.update(Message::RuleSetMethod("PATCH".into()));
    app.update(Message::SettingsSetHost("invalid-ip".into()));
    app.server_state = crate::shell::top_bar::ServerState::ReloadPending;

    app.update(Message::ReloadConfig);

    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .rule_draft(rule)
            .unwrap()
            .payload
            .method,
        "PATCH"
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_drafts.listener_ip,
        "invalid-ip"
    );
}

#[test]
fn delete_restore_rebinds_older_rule_history() {
    let mut app = expert();
    let old_id = app.selection.rule.unwrap();
    let original = app.selected_rule_payload().unwrap().url_path.clone();
    app.update(Message::RuleSetUrlPath("/history-rebind".into()));
    app.update(Message::DeleteRule(old_id));
    app.update(Message::Undo);
    let rebound = app.selection.rule.unwrap();
    assert_ne!(rebound, old_id);

    app.update(Message::Undo);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule(rebound)
            .unwrap()
            .1
            .payload
            .url_path,
        original
    );
}

#[test]
fn repeated_restore_rebinds_history_from_the_previous_live_generation() {
    let mut app = expert();
    let original = app.selected_rule_payload().unwrap().url_path.clone();
    let first = app.selection.rule.unwrap();
    app.update(Message::RuleSetUrlPath("/generation".into()));
    app.update(Message::DeleteRule(first));
    app.update(Message::Undo);
    app.update(Message::Undo);
    app.update(Message::Redo);
    app.update(Message::Redo);
    app.update(Message::Undo);
    let third = app.selection.rule.unwrap();
    app.update(Message::Undo);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule(third)
            .unwrap()
            .1
            .payload
            .url_path,
        original
    );
}

#[test]
fn remove_rule_set_undo_restores_exact_root_position() {
    let mut app = expert();
    app.update(Message::AddRuleSet);
    let ids = app
        .snapshot
        .as_ref()
        .unwrap()
        .rule_sets
        .iter()
        .map(|set| set.id)
        .collect::<Vec<_>>();
    let removed = ids[0];
    app.update(Message::DeleteRuleSet(removed));
    app.update(Message::ConfirmProceed);
    app.update(Message::Undo);

    assert_eq!(
        app.snapshot.as_ref().unwrap().rule_sets[0].file.path,
        "rules/main.toml"
    );
}

#[test]
fn prototype_edit_is_history_backed_but_not_workspace_dirty() {
    let mut app = expert();
    app.update(Message::Save);
    let rule_id = app.selection.rule.unwrap();
    let dirty_before = app.dirty_count;
    let before = app.snapshot.as_ref().unwrap().prototype.rule_extras[&rule_id].clone();

    app.update(Message::RuleWeightChanged("17".into()));
    assert_eq!(app.dirty_count, dirty_before);
    assert_eq!(
        app.snapshot.as_ref().unwrap().prototype.rule_extras[&rule_id].weight,
        Some(17)
    );
    app.update(Message::Undo);
    assert_eq!(
        app.snapshot.as_ref().unwrap().prototype.rule_extras[&rule_id],
        before
    );
}

#[test]
fn condition_add_undo_redo_uses_new_identity() {
    let mut app = expert();
    let rule_id = app.selection.rule.unwrap();
    app.update(Message::HeaderAdd);
    let index = app
        .snapshot
        .as_ref()
        .unwrap()
        .rule_draft(rule_id)
        .unwrap()
        .payload
        .headers
        .len()
        - 1;
    app.update(Message::HeaderSetName {
        index,
        value: "X-New".into(),
    });
    let first_id = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule_id)
        .unwrap()
        .conditions()
        .headers
        .last()
        .unwrap()
        .id;
    app.update(Message::Undo);
    app.update(Message::Redo);
    let second_id = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule_id)
        .unwrap()
        .conditions()
        .headers
        .last()
        .unwrap()
        .id;
    assert_ne!(first_id, second_id);
}

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

struct MiddleSaveFailurePort {
    inner: MemoryWorkspace,
    observed: Rc<RefCell<Option<ObservedSaveFailure>>>,
}

struct ValidationMismatchPort {
    inner: MemoryWorkspace,
    report: ValidationReport,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectionAdoptionSource {
    SaveSuccess,
    SaveFailure,
    ReloadAcknowledgement,
    RestartAcknowledgement,
}

struct SelectionAdoptionPort {
    inner: MemoryWorkspace,
    selected_rule: NodeId,
    source: SelectionAdoptionSource,
    removed: bool,
}

impl SelectionAdoptionPort {
    fn remove_selected_rule(&mut self) {
        if self.removed {
            return;
        }
        self.inner
            .apply(
                EditTransaction::new(vec![EditIntent::DeleteRule {
                    id: self.selected_rule,
                }])
                .unwrap(),
            )
            .unwrap();
        self.removed = true;
    }
}

impl WorkspacePort for SelectionAdoptionPort {
    fn snapshot(&self) -> PortSnapshot {
        self.inner.snapshot()
    }

    fn apply(&mut self, transaction: EditTransaction) -> Result<EditOutcome, ApplyFailure> {
        self.inner.apply(transaction)
    }

    fn validate(&self) -> ValidationReport {
        self.inner.validate()
    }

    fn save(&mut self) -> Result<SaveOutcome, SaveFailure> {
        match self.source {
            SelectionAdoptionSource::SaveSuccess => {
                self.remove_selected_rule();
                self.inner.save()
            }
            SelectionAdoptionSource::SaveFailure => {
                self.remove_selected_rule();
                let failed_file = self.inner.snapshot().dirty_files()[0].path.clone();
                self.inner.inject_save_failure(failed_file).unwrap();
                self.inner.save()
            }
            SelectionAdoptionSource::ReloadAcknowledgement
            | SelectionAdoptionSource::RestartAcknowledgement => self.inner.save(),
        }
    }

    fn acknowledge_reload(&mut self) -> PortSnapshot {
        if self.source == SelectionAdoptionSource::ReloadAcknowledgement {
            self.remove_selected_rule();
        }
        self.inner.acknowledge_reload()
    }

    fn acknowledge_restart(&mut self) -> PortSnapshot {
        if self.source == SelectionAdoptionSource::RestartAcknowledgement {
            self.remove_selected_rule();
        }
        self.inner.acknowledge_restart()
    }
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

struct ObservedSaveFailure {
    written_files: Vec<String>,
    failed_file: String,
    unsaved_hint: RuntimeEffect,
    runtime_pending: RuntimeEffect,
}

impl WorkspacePort for MiddleSaveFailurePort {
    fn snapshot(&self) -> PortSnapshot {
        self.inner.snapshot()
    }

    fn apply(&mut self, transaction: EditTransaction) -> Result<EditOutcome, ApplyFailure> {
        self.inner.apply(transaction)
    }

    fn validate(&self) -> ValidationReport {
        self.inner.validate()
    }

    fn save(&mut self) -> Result<SaveOutcome, SaveFailure> {
        let dirty = self.inner.snapshot().dirty_files().to_vec();
        assert!(dirty.len() >= 3, "test requires a middle save target");
        self.inner
            .inject_save_failure(dirty[1].path.clone())
            .unwrap();
        let result = self.inner.save();
        if let Err(failure) = &result {
            *self.observed.borrow_mut() = Some(ObservedSaveFailure {
                written_files: failure
                    .written_files
                    .iter()
                    .map(|path| path.as_str().to_owned())
                    .collect(),
                failed_file: failure.failed_file.as_str().to_owned(),
                unsaved_hint: failure.unsaved_hint,
                runtime_pending: failure.runtime_pending,
            });
        }
        result
    }

    fn acknowledge_reload(&mut self) -> PortSnapshot {
        self.inner.acknowledge_reload()
    }

    fn acknowledge_restart(&mut self) -> PortSnapshot {
        self.inner.acknowledge_restart()
    }
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

fn selection_adoption_app(source: SelectionAdoptionSource) -> (App, NodeId, RuleSetId) {
    let mut app = expert();
    let seed = apimokka_model::mock::shop_api_canonical_seed();
    let selected_rule = seed.rule_sets[0].rules[0].id;
    let parent = seed.rule_sets[0].id;
    let port = SelectionAdoptionPort {
        inner: MemoryWorkspace::new(seed).unwrap(),
        selected_rule,
        source,
        removed: false,
    };
    app.snapshot = Some(WorkspaceSession::from_port(
        Box::new(port),
        workspace_session::PrototypeState::default(),
    ));
    app.selection.select_rule(selected_rule, parent);
    app.rule_set_open = Some(parent);
    (app, selected_rule, parent)
}

fn dirty_first_fallback(app: &mut App, content: &str) -> (String, String) {
    let path = app.snapshot.as_ref().unwrap().fallback_files[0]
        .path
        .clone();
    let baseline = app.fallback_saved[&path].clone();
    app.update(Message::SelectFileRoute(path.clone()));
    app.fallback_drafts.insert(
        path.clone(),
        iced::widget::text_editor::Content::with_text(content),
    );
    app.fallback_status_draft
        .insert(path.clone(), "503 Service Unavailable".into());
    app.recompute_dirty();
    assert!(app.is_fallback_dirty(&path));
    (path, baseline)
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

#[test]
fn condition_focus_binds_pending_identity_and_clears_on_route_change() {
    let mut app = expert();
    let rule = app.selection.rule.unwrap();
    app.update(Message::HeaderAdd);
    let pending = app
        .snapshot
        .as_ref()
        .unwrap()
        .condition_focus
        .clone()
        .unwrap();
    assert_eq!(pending.rule_id, rule);
    assert_eq!(pending.family, ConditionFamily::Header);
    assert!(matches!(pending.binding, DraftBinding::Pending(_)));

    let index = app
        .snapshot
        .as_ref()
        .unwrap()
        .rule_draft(rule)
        .unwrap()
        .header_bindings
        .len()
        - 1;
    app.update(Message::HeaderSetName {
        index,
        value: "x-focus".into(),
    });
    assert!(matches!(
        app.snapshot
            .as_ref()
            .unwrap()
            .condition_focus
            .as_ref()
            .map(|focus| &focus.binding),
        Some(DraftBinding::Existing(_))
    ));

    let other_set = app.snapshot.as_ref().unwrap().rule_sets[1].id;
    app.update(Message::SelectRuleSet(other_set));
    assert!(app.snapshot.as_ref().unwrap().condition_focus.is_none());
}

#[test]
fn selected_removal_falls_back_only_to_the_captured_parent() {
    let mut app = expert();
    let rule = app.selection.rule.unwrap();
    let parent = app.selection.rule_set.unwrap();
    app.update(Message::DeleteRule(rule));
    assert_eq!(app.selection.rule, None);
    assert_eq!(app.selection.rule_set, Some(parent));

    app.update(Message::DeleteRuleSet(parent));
    app.update(Message::ConfirmProceed);
    assert_eq!(app.selection, RouteSelection::default());
}

#[test]
fn created_and_restored_nodes_select_the_receipt_or_verified_rebind() {
    let mut app = expert();

    app.update(Message::AddRuleSet);
    let created_set = app.selection.rule_set.unwrap();
    assert_eq!(app.selection.rule, None);
    assert!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule_set(created_set)
            .is_some()
    );

    let parent = app.snapshot.as_ref().unwrap().rule_sets[0].id;
    app.update(Message::SelectRuleSet(parent));
    app.update(Message::AddRule(parent));
    let created_rule = app.selection.rule.unwrap();
    assert_eq!(app.selection.rule_set, Some(parent));
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule(created_rule)
            .unwrap()
            .0
            .id,
        parent
    );

    app.update(Message::DuplicateRule(created_rule));
    let duplicate = app.selection.rule.unwrap();
    assert_ne!(duplicate, created_rule);
    assert_eq!(app.selection.rule_set, Some(parent));
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule(duplicate)
            .unwrap()
            .0
            .id,
        parent
    );

    app.update(Message::DeleteRule(duplicate));
    assert_eq!(app.selection.rule, None);
    app.update(Message::Undo);
    let rebound = app.selection.rule.unwrap();
    assert_ne!(rebound, duplicate);
    assert_eq!(app.selection.rule_set, Some(parent));
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule(rebound)
            .unwrap()
            .0
            .id,
        parent
    );
}

#[test]
fn save_success_and_failure_reconcile_their_adopted_snapshots() {
    for source in [
        SelectionAdoptionSource::SaveSuccess,
        SelectionAdoptionSource::SaveFailure,
    ] {
        let (mut app, removed, parent) = selection_adoption_app(source);
        app.update(Message::Save);

        assert!(app.snapshot.as_ref().unwrap().find_rule(removed).is_none());
        assert_eq!(app.selection.rule, None);
        assert_eq!(app.selection.rule_set, Some(parent));
        assert_eq!(
            app.last_problem.is_some(),
            source == SelectionAdoptionSource::SaveFailure
        );
    }
}

#[test]
fn reload_and_restart_acknowledgements_reconcile_their_adopted_snapshots() {
    for source in [
        SelectionAdoptionSource::ReloadAcknowledgement,
        SelectionAdoptionSource::RestartAcknowledgement,
    ] {
        let (mut app, removed, parent) = selection_adoption_app(source);
        match source {
            SelectionAdoptionSource::ReloadAcknowledgement => {
                app.server_state = crate::shell::top_bar::ServerState::ReloadPending;
                app.update(Message::ReloadConfig);
            }
            SelectionAdoptionSource::RestartAcknowledgement => {
                app.update(Message::RestartServer);
            }
            SelectionAdoptionSource::SaveSuccess | SelectionAdoptionSource::SaveFailure => {
                unreachable!()
            }
        }

        assert!(app.snapshot.as_ref().unwrap().find_rule(removed).is_none());
        assert_eq!(app.selection.rule, None);
        assert_eq!(app.selection.rule_set, Some(parent));
    }
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

#[test]
fn workspace_replacement_requires_confirmation_and_cancel_preserves_session() {
    let mut app = expert();
    let old_rule = app.selection.rule.unwrap();
    app.update(Message::RuleSetMethod("PATCH".into()));
    app.update(Message::OpenWorkspace("another".into()));

    assert!(matches!(
        app.confirm_dialog.as_ref().map(|dialog| &dialog.action),
        Some(crate::message::ConfirmAction::SwitchWorkspace(name)) if name == "another"
    ));
    assert_eq!(app.selection.rule, Some(old_rule));
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .rule_draft(old_rule)
            .unwrap()
            .payload
            .method,
        "PATCH"
    );

    app.update(Message::ConfirmCancel);
    assert!(app.confirm_dialog.is_none());
    assert_eq!(app.selection.rule, Some(old_rule));

    app.update(Message::OpenWorkspace("another".into()));
    app.update(Message::ConfirmProceed);
    let session = app.snapshot.as_ref().unwrap();
    assert!(!session.faulted);
    assert!(session.rule_drafts.is_empty());
    assert!(session.undo_stack.is_empty() && session.redo_stack.is_empty());
}

#[test]
fn confirmed_open_resets_fallback_content_and_status_for_the_new_identity() {
    let mut app = expert();
    let (path, baseline) = dirty_first_fallback(&mut app, "{\"old-workspace\":true}");
    app.update(Message::OpenWorkspace("another".into()));
    app.update(Message::ConfirmCancel);
    assert_eq!(app.fallback_saved[&path], baseline);
    assert_eq!(
        app.fallback_drafts[&path].text(),
        "{\"old-workspace\":true}"
    );
    assert_eq!(app.fallback_status_draft[&path], "503 Service Unavailable");

    app.update(Message::OpenWorkspace("another".into()));
    app.update(Message::ConfirmProceed);

    assert!(app.fallback_drafts.is_empty());
    assert!(app.fallback_status_draft.is_empty());
    assert_eq!(app.fallback_saved[&path], baseline);
    assert_eq!(app.fallback_status_saved[&path], "200 OK");
    assert!(!app.is_fallback_dirty(&path));
    assert_eq!(
        app.dirty_count,
        app.snapshot.as_ref().unwrap().latest().dirty_files().len()
    );
}

#[test]
fn leaving_for_welcome_uses_the_same_pending_draft_confirmation_guard() {
    let mut app = expert();
    app.update(Message::SettingsSetHost("invalid-ip".into()));
    app.update(Message::GoWelcome);
    assert!(matches!(
        app.confirm_dialog.as_ref().map(|dialog| &dialog.action),
        Some(crate::message::ConfirmAction::LeaveWorkspace)
    ));
    assert!(app.snapshot.is_some());
    app.update(Message::ConfirmCancel);
    assert!(app.snapshot.is_some());

    app.update(Message::GoWelcome);
    app.update(Message::ConfirmProceed);
    assert!(app.snapshot.is_none());
    assert_eq!(app.view, AppView::Welcome);
}

#[test]
fn confirmed_leave_discards_fallback_state_before_reopen() {
    let mut app = expert();
    let (path, baseline) = dirty_first_fallback(&mut app, "{\"leave\":true}");
    app.update(Message::GoWelcome);
    app.update(Message::ConfirmProceed);
    assert!(app.fallback_saved.is_empty());
    assert!(app.fallback_drafts.is_empty());
    assert!(app.fallback_status_saved.is_empty());
    assert!(app.fallback_status_draft.is_empty());

    app.update(Message::OpenWorkspace("reopen".into()));
    assert_eq!(app.fallback_saved[&path], baseline);
    assert!(app.fallback_drafts.is_empty());
    assert!(!app.is_fallback_dirty(&path));
    assert_eq!(
        app.dirty_count,
        app.snapshot.as_ref().unwrap().latest().dirty_files().len()
    );
}

#[test]
fn wizard_creation_is_confirmed_atomically_and_resets_fallback_state() {
    let mut app = expert();
    let old_identity = app.snapshot.as_ref().unwrap().identity.clone();
    let (path, baseline) = dirty_first_fallback(&mut app, "{\"wizard-old\":true}");
    let rule = app
        .selection
        .rule
        .unwrap_or_else(|| app.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id);
    app.update(Message::SelectRule(rule));
    app.update(Message::RuleSetUrlPath("/canonical-dirty".into()));
    app.update(Message::RuleSetMethod("PATCH".into()));
    app.update(Message::SettingsSetHost("invalid-ip".into()));
    app.update(Message::GoWizard);
    app.update(Message::WizardSetName("wizard-new".into()));
    app.update(Message::WizardSetStarter(WizardStarter::ShopApi));
    app.update(Message::WizardCreate);

    assert!(matches!(
        app.confirm_dialog.as_ref().map(|dialog| &dialog.action),
        Some(crate::message::ConfirmAction::CreateWorkspace)
    ));
    assert_eq!(app.snapshot.as_ref().unwrap().identity, old_identity);
    assert_eq!(app.fallback_drafts[&path].text(), "{\"wizard-old\":true}");
    app.update(Message::ConfirmCancel);
    app.update(Message::WizardCancel);
    assert_eq!(app.view, AppView::Workspace);
    assert_eq!(app.snapshot.as_ref().unwrap().identity, old_identity);
    assert_eq!(app.fallback_drafts[&path].text(), "{\"wizard-old\":true}");

    app.update(Message::GoWizard);
    app.update(Message::WizardSetName("wizard-new".into()));
    app.update(Message::WizardSetStarter(WizardStarter::ShopApi));
    app.update(Message::WizardCreate);
    app.update(Message::ConfirmProceed);

    assert_eq!(app.snapshot.as_ref().unwrap().identity.name, "wizard-new");
    assert!(app.fallback_drafts.is_empty());
    assert!(app.fallback_status_draft.is_empty());
    assert_eq!(app.fallback_saved[&path], baseline);
    assert_eq!(app.fallback_status_saved[&path], "200 OK");
    assert!(!app.is_fallback_dirty(&path));
    assert_eq!(
        app.dirty_count,
        app.snapshot.as_ref().unwrap().latest().dirty_files().len()
    );
}

#[test]
fn failed_session_admission_preserves_the_complete_old_session() {
    let mut app = expert();
    let rule = app.selection.rule.unwrap();
    app.update(Message::RuleSetUrlPath(
        "/dirty-before-failed-admission".into(),
    ));
    app.update(Message::RuleSetMethod("PATCH".into()));
    app.update(Message::RuleWeightChanged("23".into()));
    let (fallback_path, fallback_baseline) =
        dirty_first_fallback(&mut app, "{\"failed-admission\":true}");
    let selection_before = app.selection.clone();
    let identity = app.snapshot.as_ref().unwrap().identity.clone();
    let history = app.undo_stack().len();
    let prototype = app.snapshot.as_ref().unwrap().prototype.rule_extras[&rule].clone();
    let mut invalid = apimokka_model::mock::shop_api_canonical_seed();
    invalid.root_settings.listener_ip = "not-an-ip".into();

    assert!(!app.install_workspace(invalid));

    let session = app.snapshot.as_ref().unwrap();
    assert_eq!(session.identity, identity);
    assert_eq!(app.selection.rule, selection_before.rule);
    assert_eq!(app.selection.rule_set, selection_before.rule_set);
    assert_eq!(app.selection.file_route, selection_before.file_route);
    assert_eq!(session.undo_stack.len(), history);
    assert_eq!(session.prototype.rule_extras[&rule], prototype);
    assert_eq!(session.rule_draft(rule).unwrap().payload.method, "PATCH");
    assert_eq!(app.fallback_saved[&fallback_path], fallback_baseline);
    assert_eq!(
        app.fallback_drafts[&fallback_path].text(),
        "{\"failed-admission\":true}"
    );
    assert_eq!(
        app.fallback_status_draft[&fallback_path],
        "503 Service Unavailable"
    );
    assert!(app.is_fallback_dirty(&fallback_path));
}

#[test]
fn middle_workspace_save_failure_adopts_prefix_and_preserves_fallback_baselines() {
    let mut app = expert();
    let seed = apimokka_model::mock::shop_api_canonical_seed();
    let observed = Rc::new(RefCell::new(None));
    let session = WorkspaceSession::from_port(
        Box::new(MiddleSaveFailurePort {
            inner: MemoryWorkspace::new(seed).unwrap(),
            observed: observed.clone(),
        }),
        workspace_session::PrototypeState::default(),
    );
    let first_rule = session.rule_sets[0].rules[0].id;
    let second_rule = session.rule_sets[1].rules[0].id;
    app.snapshot = Some(session);
    app.selection.rule_set = Some(app.snapshot.as_ref().unwrap().rule_sets[0].id);
    app.selection.rule = Some(first_rule);
    app.update(Message::RuleSetUrlPath("/save-first".into()));
    app.update(Message::SelectRule(second_rule));
    app.update(Message::RuleSetUrlPath("/save-second".into()));
    app.update(Message::SettingsSetHost("127.0.0.2".into()));
    let dirty_before = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .dirty_files()
        .to_vec();
    assert!(dirty_before.len() >= 3);

    let fallback = app.fallback_saved.keys().next().unwrap().clone();
    let baseline = app.fallback_saved[&fallback].clone();
    let status_baseline = app.fallback_status_saved[&fallback].clone();
    app.fallback_drafts.insert(
        fallback.clone(),
        iced::widget::text_editor::Content::with_text("{\"pending\":true}"),
    );
    app.fallback_status_draft
        .insert(fallback.clone(), "503 Service Unavailable".into());

    app.update(Message::Save);

    let dirty_after = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .dirty_files()
        .to_vec();
    let observed = observed.borrow();
    let failure = observed.as_ref().unwrap();
    assert_eq!(
        failure.written_files,
        vec![dirty_before[0].path.as_str().to_owned()]
    );
    assert_eq!(failure.failed_file, dirty_before[1].path.as_str());
    assert_eq!(
        dirty_after
            .iter()
            .map(|diff| diff.path.as_str())
            .collect::<Vec<_>>(),
        dirty_before[1..]
            .iter()
            .map(|diff| diff.path.as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(failure.runtime_pending, dirty_before[0].effect);
    assert_eq!(
        app.snapshot.as_ref().unwrap().latest().runtime_pending(),
        failure.runtime_pending
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().latest().unsaved_hint(),
        failure.unsaved_hint
    );
    assert_eq!(app.fallback_saved[&fallback], baseline);
    assert_eq!(app.fallback_status_saved[&fallback], status_baseline);
    assert_eq!(
        app.fallback_status_draft[&fallback],
        "503 Service Unavailable"
    );
    assert!(app.is_fallback_dirty(&fallback));
    assert!(app.last_problem.is_some());
}

#[test]
fn header_and_body_update_remove_and_clear_round_trip_both_directions() {
    let mut header_app = expert();
    let rule = header_app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rules()
        .iter()
        .find(|rule| !rule.conditions().headers.is_empty())
        .unwrap()
        .rule_id();
    header_app.update(Message::SelectRule(rule));
    assert!(
        !header_app
            .selected_rule_payload()
            .unwrap()
            .headers
            .is_empty()
    );
    let original_name = header_app.selected_rule_payload().unwrap().headers[0]
        .name
        .clone();
    header_app.update(Message::HeaderSetName {
        index: 0,
        value: "X-Round-Trip".into(),
    });
    header_app.update(Message::Undo);
    assert_eq!(
        header_app.selected_rule_payload().unwrap().headers[0].name,
        original_name
    );
    header_app.update(Message::Redo);
    assert_eq!(
        header_app.selected_rule_payload().unwrap().headers[0].name,
        "x-round-trip"
    );
    let header_count = header_app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule)
        .unwrap()
        .conditions()
        .headers
        .len();
    header_app.update(Message::HeaderRemove(0));
    header_app.update(Message::Undo);
    assert_eq!(
        header_app
            .snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule)
            .unwrap()
            .conditions()
            .headers
            .len(),
        header_count
    );
    header_app.update(Message::Redo);
    assert_eq!(
        header_app
            .snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule)
            .unwrap()
            .conditions()
            .headers
            .len(),
        header_count - 1
    );

    let mut body_app = expert();
    let rule = body_app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rules()
        .iter()
        .find(|rule| !rule.conditions().body.is_empty())
        .unwrap()
        .rule_id();
    body_app.update(Message::SelectRule(rule));
    assert!(!body_app.selected_rule_payload().unwrap().body.is_empty());
    let original_path = body_app.selected_rule_payload().unwrap().body[0]
        .path
        .clone();
    body_app.update(Message::BodySetPath {
        index: 0,
        value: "user.name".into(),
    });
    body_app.update(Message::Undo);
    assert_eq!(
        body_app.selected_rule_payload().unwrap().body[0].path,
        original_path
    );
    body_app.update(Message::Redo);
    assert_eq!(
        body_app.selected_rule_payload().unwrap().body[0].path,
        "user.name"
    );
    let body_count = body_app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule)
        .unwrap()
        .conditions()
        .body
        .len();
    body_app.update(Message::BodyRemove(0));
    body_app.update(Message::Undo);
    assert_eq!(
        body_app
            .snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule)
            .unwrap()
            .conditions()
            .body
            .len(),
        body_count
    );
    body_app.update(Message::Redo);
    assert_eq!(
        body_app
            .snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule)
            .unwrap()
            .conditions()
            .body
            .len(),
        body_count - 1
    );
    body_app.update(Message::Undo);
    body_app.update(Message::BodyClearAll);
    body_app.update(Message::Undo);
    assert_eq!(
        body_app
            .snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule)
            .unwrap()
            .conditions()
            .body
            .len(),
        body_count
    );
    body_app.update(Message::Redo);
    assert!(
        body_app
            .snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule)
            .unwrap()
            .conditions()
            .body
            .is_empty()
    );
}

#[test]
fn every_supported_root_key_and_response_redo_round_trip() {
    let mut app = expert();
    let initial = app.snapshot.as_ref().unwrap().root_settings.clone();
    let strategy = if initial.strategy == apimokka_model::settings::Strategy::FirstMatch {
        apimokka_model::settings::Strategy::RoundRobin
    } else {
        apimokka_model::settings::Strategy::FirstMatch
    };
    let edits = [
        Message::SettingsSetHost("127.0.0.2".into()),
        Message::SettingsSetPort("4567".into()),
        Message::SettingsSetTls(!initial.tls_enabled),
        Message::SettingsSetLogLevel("debug".into()),
        Message::SettingsSetStrategy(strategy),
    ];
    for edit in edits {
        app.update(edit);
        app.update(Message::Undo);
        app.update(Message::Redo);
    }
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.listener_ip,
        "127.0.0.2"
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.listener_port,
        4567
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.tls_enabled,
        !initial.tls_enabled
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.log_level,
        "debug"
    );

    let rule = app.selection.rule.unwrap();
    app.update(Message::RespondSetStatus("202 Accepted".into()));
    app.update(Message::Undo);
    app.update(Message::Redo);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule)
            .unwrap()
            .respond()
            .status(),
        Some("202 Accepted")
    );
}

#[test]
fn rule_and_rule_set_add_remove_redo_with_new_identity() {
    let mut app = expert();
    let set = app.selection.rule_set.unwrap();
    let rule_count = app
        .snapshot
        .as_ref()
        .unwrap()
        .find_rule_set(set)
        .unwrap()
        .rules
        .len();
    app.update(Message::AddRule(set));
    let first_added = app.selection.rule.unwrap();
    app.update(Message::Undo);
    app.update(Message::Redo);
    let second_added = app.selection.rule.unwrap();
    assert_ne!(first_added, second_added);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule_set(set)
            .unwrap()
            .rules
            .len(),
        rule_count + 1
    );

    app.update(Message::DeleteRule(second_added));
    app.update(Message::Undo);
    let restored = app.selection.rule.unwrap();
    app.update(Message::Redo);
    assert!(app.snapshot.as_ref().unwrap().find_rule(restored).is_none());

    let set_count = app.snapshot.as_ref().unwrap().rule_sets.len();
    app.update(Message::AddRuleSet);
    let added_set = app.selection.rule_set.unwrap();
    app.update(Message::Undo);
    app.update(Message::Redo);
    let rebound_set = app.selection.rule_set.unwrap();
    assert_ne!(added_set, rebound_set);
    assert_eq!(
        app.snapshot.as_ref().unwrap().rule_sets.len(),
        set_count + 1
    );
    app.update(Message::DeleteRuleSet(rebound_set));
    app.update(Message::ConfirmProceed);
    app.update(Message::Undo);
    let restored_set = app.selection.rule_set.unwrap();
    app.update(Message::Redo);
    assert!(
        app.snapshot
            .as_ref()
            .unwrap()
            .find_rule_set(restored_set)
            .is_none()
    );
}

#[test]
fn duplicate_rule_preserves_subtree_prototype_and_trace_history() {
    let mut app = expert();
    let source = app.selection.rule.unwrap();
    let source_conditions = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(source)
        .unwrap()
        .conditions()
        .clone();
    let source_prototype = app.snapshot.as_ref().unwrap().prototype.rule_extras[&source].clone();
    app.update(Message::DuplicateRule(source));
    let duplicate = app.selection.rule.unwrap();
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(duplicate)
            .unwrap()
            .conditions()
            .headers
            .len(),
        source_conditions.headers.len()
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().prototype.rule_extras[&duplicate],
        source_prototype
    );
    app.update(Message::Undo);
    app.update(Message::Redo);
    let rebound = app.selection.rule.unwrap();
    assert_eq!(
        app.snapshot.as_ref().unwrap().prototype.rule_extras[&rebound],
        source_prototype
    );

    let trace_before = app
        .snapshot
        .as_ref()
        .unwrap()
        .prototype
        .trace
        .clone()
        .unwrap();
    app.update(Message::SettingsSetTraceEnabled(!trace_before.enabled));
    app.update(Message::Undo);
    assert_eq!(
        app.snapshot.as_ref().unwrap().prototype.trace.as_ref(),
        Some(&trace_before)
    );
    app.update(Message::Redo);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .prototype
            .trace
            .as_ref()
            .unwrap()
            .enabled,
        !trace_before.enabled
    );
}

#[test]
fn root_draft_rejection_and_history_keep_identity_consistent() {
    let mut app = expert();
    let original = app
        .snapshot
        .as_ref()
        .unwrap()
        .root_settings
        .listener_ip
        .clone();
    app.update(Message::SettingsSetHost("invalid-ip".into()));
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_drafts.listener_ip,
        "invalid-ip"
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.listener_ip,
        original
    );

    app.update(Message::SettingsSetHost("127.0.0.2".into()));
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.listener_ip,
        "127.0.0.2"
    );
    app.update(Message::Undo);
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_settings.listener_ip,
        original
    );
    assert_eq!(
        app.snapshot.as_ref().unwrap().root_drafts.listener_ip,
        original
    );
}

#[test]
fn response_invalid_delay_stays_draft_and_valid_edit_round_trips() {
    let mut app = expert();
    let rule_id = app.selection.rule.unwrap();
    let before = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule_id)
        .unwrap()
        .respond()
        .clone();
    app.update(Message::RespondSetDelay("-1".into()));
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .rule_draft(rule_id)
            .unwrap()
            .response_delay,
        "-1"
    );
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule_id)
            .unwrap()
            .respond(),
        &before
    );

    app.update(Message::RespondSetDelay("25".into()));
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule_id)
            .unwrap()
            .respond()
            .delay_milliseconds(),
        Some(25)
    );
    app.update(Message::Undo);
    assert_eq!(
        app.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule_id)
            .unwrap()
            .respond(),
        &before
    );
}

#[test]
fn clear_conditions_is_one_exact_undo_step() {
    let mut app = expert();
    let rule_id = app.selection.rule.unwrap();
    app.update(Message::HeaderAdd);
    let index = app
        .snapshot
        .as_ref()
        .unwrap()
        .rule_draft(rule_id)
        .unwrap()
        .payload
        .headers
        .len()
        - 1;
    app.update(Message::HeaderSetName {
        index,
        value: "x-clear-test".into(),
    });
    let before = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule_id)
        .unwrap()
        .conditions()
        .headers
        .iter()
        .map(|condition| condition.condition.clone())
        .collect::<Vec<_>>();
    assert!(!before.is_empty());
    let history = app.undo_stack().len();
    app.update(Message::HeaderClearAll);
    assert!(
        app.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .rule(rule_id)
            .unwrap()
            .conditions()
            .headers
            .is_empty()
    );
    assert_eq!(app.undo_stack().len(), history + 1);
    app.update(Message::Undo);
    let after = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rule(rule_id)
        .unwrap()
        .conditions()
        .headers
        .iter()
        .map(|condition| condition.condition.clone())
        .collect::<Vec<_>>();
    assert_eq!(after, before);
}

#[test]
fn semantic_history_is_capped_at_fifty_entries() {
    let mut app = expert();
    for value in 0..55 {
        app.update(Message::RuleWeightChanged(value.to_string()));
    }
    assert_eq!(app.undo_stack().len(), 50);
}

#[test]
fn move_up_undo_redo_uses_the_recorded_after_index() {
    let mut app = expert();
    let moved = app.snapshot.as_ref().unwrap().rule_sets[0].rules[1].id;
    app.update(Message::MoveRuleUp(moved));
    assert_eq!(
        app.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id,
        moved
    );
    app.update(Message::Undo);
    assert_eq!(
        app.snapshot.as_ref().unwrap().rule_sets[0].rules[1].id,
        moved
    );
    app.update(Message::Redo);
    assert_eq!(
        app.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id,
        moved
    );
}
