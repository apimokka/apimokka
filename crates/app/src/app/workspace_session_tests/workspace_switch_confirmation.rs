//! Confirmation guard for switching, leaving, and creating workspaces while
//! drafts or fallback edits are pending, and how the guard interacts with a
//! save that fails partway through.

use super::*;
use crate::message::Message;
use apimokka_model::{
    ApplyFailure, EditOutcome, EditTransaction, MemoryWorkspace, PortSnapshot, RuntimeEffect,
    SaveFailure, SaveOutcome, ValidationReport, WorkspacePort,
};
use std::cell::RefCell;
use std::rc::Rc;

struct ObservedSaveFailure {
    written_files: Vec<String>,
    failed_file: String,
    unsaved_hint: RuntimeEffect,
    runtime_pending: RuntimeEffect,
}

struct MiddleSaveFailurePort {
    inner: MemoryWorkspace,
    observed: Rc<RefCell<Option<ObservedSaveFailure>>>,
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
