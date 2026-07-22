use super::runtime::{
    RuntimeCompletionDisposition, RuntimeRequestId, SavedConfigRevision, SessionGeneration,
};
use super::*;
use crate::message::Message;
use apimokka_model::{
    ApplyFailure, Diagnostic, EditOutcome, EditTransaction, MemoryWorkspace, PortSnapshot,
    RuntimeEffect, SaveFailure, SaveOutcome, Severity, ValidationIssue, ValidationReport,
    WorkspacePort,
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct ProbeControl {
    fault_apply: bool,
    validation_mismatch: bool,
    malformed_ack: bool,
    acknowledgement_diagnostic: bool,
    malformed_save: bool,
    malformed_save_progress: bool,
    fail_save_at: Option<usize>,
    acknowledge_reload: usize,
    acknowledge_restart: usize,
}

struct RuntimeProbePort {
    inner: MemoryWorkspace,
    control: Rc<RefCell<ProbeControl>>,
}

fn altered_snapshot(
    snapshot: PortSnapshot,
    identity_drift: bool,
    diagnostic: bool,
) -> PortSnapshot {
    let mut workspace = snapshot.into_legacy_workspace();
    if identity_drift {
        workspace.meta.name.push_str("-drift");
    }
    if diagnostic {
        workspace.diagnostics.push(Diagnostic {
            node_id: None,
            severity: Severity::Info,
            message: "runtime acknowledgement diagnostic".into(),
        });
    }
    MemoryWorkspace::new(workspace).unwrap().snapshot()
}

impl WorkspacePort for RuntimeProbePort {
    fn snapshot(&self) -> PortSnapshot {
        self.inner.snapshot()
    }

    fn apply(&mut self, transaction: EditTransaction) -> Result<EditOutcome, ApplyFailure> {
        let mut outcome = self.inner.apply(transaction)?;
        if self.control.borrow().fault_apply {
            outcome.creations.clear();
        }
        Ok(outcome)
    }

    fn validate(&self) -> ValidationReport {
        if self.control.borrow().validation_mismatch {
            ValidationReport {
                issues: vec![ValidationIssue {
                    node_id: None,
                    severity: Severity::Error,
                    message: "injected validation mismatch".into(),
                    location: None,
                }],
            }
        } else {
            self.inner.validate()
        }
    }

    fn save(&mut self) -> Result<SaveOutcome, SaveFailure> {
        let (fail_at, malformed, malformed_progress) = {
            let mut control = self.control.borrow_mut();
            (
                control.fail_save_at.take(),
                control.malformed_save,
                control.malformed_save_progress,
            )
        };
        if let Some(index) = fail_at {
            let dirty = self.inner.snapshot().dirty_files().to_vec();
            self.inner
                .inject_save_failure(dirty[index].path.clone())
                .unwrap();
        }
        let result = self.inner.save();
        match result {
            Ok(mut outcome) => {
                if malformed_progress && !outcome.written_files.is_empty() {
                    outcome.written_files.push(outcome.written_files[0].clone());
                }
                if malformed {
                    outcome.snapshot = altered_snapshot(outcome.snapshot, true, false);
                }
                Ok(outcome)
            }
            Err(mut failure) => {
                if malformed_progress && !failure.written_files.is_empty() {
                    failure.written_files.push(failure.written_files[0].clone());
                }
                if malformed {
                    failure.snapshot = Box::new(altered_snapshot(*failure.snapshot, true, false));
                }
                Err(failure)
            }
        }
    }

    fn acknowledge_reload(&mut self) -> PortSnapshot {
        let snapshot = self.inner.acknowledge_reload();
        let mut control = self.control.borrow_mut();
        control.acknowledge_reload += 1;
        altered_snapshot(
            snapshot,
            control.malformed_ack,
            control.acknowledgement_diagnostic,
        )
    }

    fn acknowledge_restart(&mut self) -> PortSnapshot {
        let snapshot = self.inner.acknowledge_restart();
        let mut control = self.control.borrow_mut();
        control.acknowledge_restart += 1;
        altered_snapshot(
            snapshot,
            control.malformed_ack,
            control.acknowledgement_diagnostic,
        )
    }
}

fn runtime_app() -> (App, Rc<RefCell<ProbeControl>>) {
    let mut app = App::new().0;
    app.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    app.update(Message::OpenWorkspace("runtime".into()));
    let generation = app.snapshot.as_ref().unwrap().generation();
    let seed = apimokka_model::mock::shop_api_canonical_seed();
    let control = Rc::new(RefCell::new(ProbeControl::default()));
    let mut inner = MemoryWorkspace::new(seed).unwrap();
    inner.save().unwrap();
    inner.acknowledge_restart();
    let session = WorkspaceSession::from_port_with_generation(
        Box::new(RuntimeProbePort {
            inner,
            control: control.clone(),
        }),
        workspace_session::PrototypeState::default(),
        generation,
    );
    let parent = session.rule_sets[0].id;
    let rule = session.rule_sets[0].rules[0].id;
    app.snapshot = Some(session);
    app.selection.select_rule(rule, parent);
    app.rule_set_open = Some(parent);
    app.server_state = ServerState::Running;
    app.runtime_auto_complete = false;
    app.recompute_dirty();
    (app, control)
}

fn save_reload(app: &mut App, level: &str) {
    app.update(Message::SettingsSetLogLevel(level.into()));
    app.update(Message::Save);
    assert_eq!(app.runtime_phase(), RuntimeEffect::Reload);
}

fn save_restart(app: &mut App, port: u16) {
    app.update(Message::SettingsSetPort(port.to_string()));
    app.update(Message::Save);
    assert_eq!(app.runtime_phase(), RuntimeEffect::Restart);
}

fn prepare_action(app: &mut App, action: RuntimeAction) {
    match action {
        RuntimeAction::Start => {
            save_reload(app, "debug");
            app.server_state = ServerState::Stopped;
        }
        RuntimeAction::Reload => save_reload(app, "debug"),
        RuntimeAction::Restart => save_restart(app, 9000),
        RuntimeAction::Stop => {
            save_reload(app, "debug");
            app.server_state = ServerState::Running;
        }
    }
}

fn dispatch_request(app: &mut App, action: RuntimeAction) -> RuntimeRequestToken {
    match action {
        RuntimeAction::Start | RuntimeAction::Stop => app.update(Message::StartStopServer),
        RuntimeAction::Reload => app.update(Message::ReloadConfig),
        RuntimeAction::Restart => app.update(Message::RestartServer),
    }
    let active = app.runtime_in_flight.expect("request must be admitted");
    assert_eq!(active.token.action, action);
    assert_eq!(
        active.token.consumed_revision.is_none(),
        action == RuntimeAction::Stop
    );
    active.token
}

fn dirty_two_files(app: &mut App, effect: RuntimeEffect, suffix: &str) {
    let first = app.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    let second = app.snapshot.as_ref().unwrap().rule_sets[1].rules[0].id;
    match effect {
        RuntimeEffect::Reload => {
            app.update(Message::SelectRule(first));
            app.update(Message::RuleSetUrlPath(format!("/{suffix}-first")));
            app.update(Message::SelectRule(second));
            app.update(Message::RuleSetUrlPath(format!("/{suffix}-second")));
        }
        RuntimeEffect::Restart => {
            app.update(Message::SettingsSetPort("9001".into()));
            app.update(Message::SelectRule(first));
            app.update(Message::RuleSetUrlPath(format!("/{suffix}-rule")));
        }
        RuntimeEffect::None => unreachable!(),
    }
    assert!(app.snapshot.as_ref().unwrap().latest().dirty_files().len() >= 2);
}

fn expected_completion_lifecycle(action: RuntimeAction, succeeded: bool) -> ServerState {
    match (action, succeeded) {
        (RuntimeAction::Start | RuntimeAction::Restart, true) => ServerState::Running,
        (RuntimeAction::Start | RuntimeAction::Restart, false) => ServerState::Error,
        (RuntimeAction::Reload, _) | (RuntimeAction::Stop, false) => ServerState::Running,
        (RuntimeAction::Stop, true) => ServerState::Stopped,
    }
}

#[test]
fn request_and_completion_tables_are_exhaustive() {
    for action in [
        RuntimeAction::Start,
        RuntimeAction::Reload,
        RuntimeAction::Restart,
        RuntimeAction::Stop,
    ] {
        for succeeded in [true, false] {
            let (mut app, control) = runtime_app();
            prepare_action(&mut app, action);
            let pending = app.runtime_phase();
            let token = dispatch_request(&mut app, action);
            assert_eq!(
                app.server_state,
                if matches!(action, RuntimeAction::Start | RuntimeAction::Restart) {
                    ServerState::Starting
                } else {
                    ServerState::Running
                }
            );
            if succeeded {
                app.update(Message::RuntimeSucceeded(token));
            } else {
                app.update(Message::RuntimeFailed {
                    token,
                    technical: "injected runtime failure".into(),
                });
            }
            assert_eq!(
                app.server_state,
                expected_completion_lifecycle(action, succeeded)
            );
            assert!(app.runtime_in_flight.is_none());
            if succeeded && action != RuntimeAction::Stop {
                assert_eq!(app.runtime_phase(), RuntimeEffect::None);
            } else {
                assert_eq!(app.runtime_phase(), pending);
            }
            let control = control.borrow();
            assert_eq!(
                control.acknowledge_reload,
                usize::from(succeeded && action == RuntimeAction::Reload)
            );
            assert_eq!(
                control.acknowledge_restart,
                usize::from(
                    succeeded && matches!(action, RuntimeAction::Start | RuntimeAction::Restart)
                )
            );
        }
    }
}

#[test]
fn retries_allocate_new_ids_and_late_or_duplicate_completions_are_stale() {
    for first_action in [RuntimeAction::Start, RuntimeAction::Restart] {
        let (mut app, _) = runtime_app();
        prepare_action(&mut app, first_action);
        let failed = dispatch_request(&mut app, first_action);
        app.update(Message::RuntimeFailed {
            token: failed,
            technical: "first failure".into(),
        });
        assert_eq!(app.server_state, ServerState::Error);
        app.update(Message::RuntimeSucceeded(failed));
        assert_eq!(app.server_state, ServerState::Error);

        let retry = dispatch_request(&mut app, RuntimeAction::Start);
        assert!(retry.request_id > failed.request_id);
        app.update(Message::RuntimeSucceeded(retry));
        assert_eq!(app.server_state, ServerState::Running);
        let settled = (
            app.server_state,
            app.runtime_phase(),
            app.last_problem.clone(),
        );
        app.update(Message::RuntimeSucceeded(retry));
        assert_eq!(
            (
                app.server_state,
                app.runtime_phase(),
                app.last_problem.clone()
            ),
            settled
        );
    }
}

#[test]
fn overlap_and_every_token_mismatch_are_ignored() {
    let (mut app, _) = runtime_app();
    prepare_action(&mut app, RuntimeAction::Reload);
    let token = dispatch_request(&mut app, RuntimeAction::Reload);
    app.update(Message::StartStopServer);
    assert_eq!(app.runtime_in_flight.unwrap().token, token);

    let mismatches = [
        RuntimeRequestToken {
            generation: SessionGeneration(token.generation.0 + 1),
            ..token
        },
        RuntimeRequestToken {
            request_id: RuntimeRequestId(token.request_id.0 + 1),
            ..token
        },
        RuntimeRequestToken {
            action: RuntimeAction::Restart,
            ..token
        },
        RuntimeRequestToken {
            consumed_revision: Some(SavedConfigRevision(token.consumed_revision.unwrap().0 + 1)),
            ..token
        },
    ];
    for mismatch in mismatches {
        app.update(Message::RuntimeSucceeded(mismatch));
        app.update(Message::RuntimeFailed {
            token: mismatch,
            technical: "mismatched failure".into(),
        });
        assert_eq!(app.runtime_in_flight.unwrap().token, token);
        assert_eq!(app.server_state, ServerState::Running);
    }
    app.update(Message::RuntimeSucceeded(token));
    assert!(app.runtime_in_flight.is_none());
}

#[test]
fn session_replacement_invalidates_the_old_generation() {
    let (mut app, _) = runtime_app();
    prepare_action(&mut app, RuntimeAction::Reload);
    let old = dispatch_request(&mut app, RuntimeAction::Reload);
    assert!(app.install_workspace(apimokka_model::mock::shop_api_canonical_seed()));
    let new_generation = app.snapshot.as_ref().unwrap().generation();
    assert!(new_generation > old.generation);
    assert!(app.runtime_in_flight.is_none());
    let settled = (app.server_state, app.runtime_phase(), app.selection.clone());
    app.update(Message::RuntimeSucceeded(old));
    assert_eq!(
        (app.server_state, app.runtime_phase(), app.selection.clone()),
        settled
    );
}

#[test]
fn revision_advances_only_for_effect_bearing_verified_progress() {
    let (mut app, control) = runtime_app();
    assert_eq!(
        app.snapshot.as_ref().unwrap().saved_config_revision(),
        SavedConfigRevision(0)
    );
    app.update(Message::Save);
    assert_eq!(
        app.snapshot.as_ref().unwrap().saved_config_revision(),
        SavedConfigRevision(0)
    );

    app.update(Message::SettingsSetLogLevel("debug".into()));
    control.borrow_mut().fail_save_at = Some(0);
    app.update(Message::Save);
    assert_eq!(
        app.snapshot.as_ref().unwrap().saved_config_revision(),
        SavedConfigRevision(0)
    );

    app.update(Message::Save);
    assert_eq!(
        app.snapshot.as_ref().unwrap().saved_config_revision(),
        SavedConfigRevision(1)
    );

    let first = app.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    let second = app.snapshot.as_ref().unwrap().rule_sets[1].rules[0].id;
    app.update(Message::SelectRule(first));
    app.update(Message::RuleSetUrlPath("/revision-first".into()));
    app.update(Message::SelectRule(second));
    app.update(Message::RuleSetUrlPath("/revision-second".into()));
    control.borrow_mut().fail_save_at = Some(1);
    app.update(Message::Save);
    assert_eq!(
        app.snapshot.as_ref().unwrap().saved_config_revision(),
        SavedConfigRevision(2)
    );
}

#[test]
fn newer_saved_revisions_never_acknowledge_an_older_success() {
    let (mut app, control) = runtime_app();
    prepare_action(&mut app, RuntimeAction::Reload);
    let token = dispatch_request(&mut app, RuntimeAction::Reload);
    save_reload(&mut app, "trace");
    assert!(
        app.snapshot.as_ref().unwrap().saved_config_revision() > token.consumed_revision.unwrap()
    );
    app.update(Message::RuntimeSucceeded(token));
    assert_eq!(control.borrow().acknowledge_reload, 0);
    assert_eq!(app.runtime_phase(), RuntimeEffect::Reload);
    assert!(
        app.last_problem
            .as_ref()
            .unwrap()
            .detail
            .contains("newer saved configuration still needs reload")
    );

    let (mut escalated, escalated_control) = runtime_app();
    prepare_action(&mut escalated, RuntimeAction::Reload);
    let token = dispatch_request(&mut escalated, RuntimeAction::Reload);
    save_restart(&mut escalated, 9000);
    escalated.update(Message::RuntimeSucceeded(token));
    assert_eq!(escalated_control.borrow().acknowledge_reload, 0);
    assert_eq!(escalated.runtime_phase(), RuntimeEffect::Restart);
    assert!(
        escalated
            .last_problem
            .as_ref()
            .unwrap()
            .detail
            .contains("newer saved configuration still needs restart")
    );
}

#[test]
fn zero_prefix_failure_keeps_revision_match_but_partial_failure_invalidates_it() {
    let (mut zero, zero_control) = runtime_app();
    prepare_action(&mut zero, RuntimeAction::Reload);
    let token = dispatch_request(&mut zero, RuntimeAction::Reload);
    zero.update(Message::RuleSetUrlPath("/zero-prefix".into()));
    zero_control.borrow_mut().fail_save_at = Some(0);
    zero.update(Message::Save);
    assert_eq!(
        zero.snapshot.as_ref().unwrap().saved_config_revision(),
        token.consumed_revision.unwrap()
    );
    zero.update(Message::RuntimeSucceeded(token));
    assert_eq!(zero_control.borrow().acknowledge_reload, 1);

    let (mut partial, partial_control) = runtime_app();
    prepare_action(&mut partial, RuntimeAction::Reload);
    let token = dispatch_request(&mut partial, RuntimeAction::Reload);
    let first = partial.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    let second = partial.snapshot.as_ref().unwrap().rule_sets[1].rules[0].id;
    partial.update(Message::SelectRule(first));
    partial.update(Message::RuleSetUrlPath("/partial-first".into()));
    partial.update(Message::SelectRule(second));
    partial.update(Message::RuleSetUrlPath("/partial-second".into()));
    partial_control.borrow_mut().fail_save_at = Some(1);
    partial.update(Message::Save);
    assert!(
        partial.snapshot.as_ref().unwrap().saved_config_revision()
            > token.consumed_revision.unwrap()
    );
    partial.update(Message::RuntimeSucceeded(token));
    assert_eq!(partial_control.borrow().acknowledge_reload, 0);
    assert_eq!(partial.runtime_phase(), RuntimeEffect::Reload);
}

#[test]
fn start_and_restart_interleavings_retain_newer_configuration() {
    for action in [RuntimeAction::Start, RuntimeAction::Restart] {
        let (mut app, control) = runtime_app();
        prepare_action(&mut app, action);
        let token = dispatch_request(&mut app, action);
        if action == RuntimeAction::Start {
            save_restart(&mut app, 9000);
        } else {
            app.update(Message::SettingsSetLogLevel("debug".into()));
            app.update(Message::Save);
            assert_eq!(app.runtime_phase(), RuntimeEffect::Restart);
        }
        app.update(Message::RuntimeSucceeded(token));
        assert_eq!(app.server_state, ServerState::Running);
        assert_eq!(control.borrow().acknowledge_restart, 0);
        assert_eq!(app.runtime_phase(), RuntimeEffect::Restart);
    }
}

#[test]
fn every_consuming_action_distinguishes_zero_and_partial_intervening_saves() {
    for action in [
        RuntimeAction::Start,
        RuntimeAction::Reload,
        RuntimeAction::Restart,
    ] {
        for effect in [RuntimeEffect::Reload, RuntimeEffect::Restart] {
            for failed_index in [0, 1] {
                let (mut app, control) = runtime_app();
                prepare_action(&mut app, action);
                let token = dispatch_request(&mut app, action);
                dirty_two_files(
                    &mut app,
                    effect,
                    &format!("{action:?}-{effect:?}-{failed_index}"),
                );
                control.borrow_mut().fail_save_at = Some(failed_index);
                app.update(Message::Save);
                let revision = app.snapshot.as_ref().unwrap().saved_config_revision();
                if failed_index == 0 {
                    assert_eq!(revision, token.consumed_revision.unwrap());
                } else {
                    assert!(revision > token.consumed_revision.unwrap());
                }
                app.update(Message::RuntimeSucceeded(token));
                let control = control.borrow();
                let should_ack = failed_index == 0;
                assert_eq!(
                    control.acknowledge_reload,
                    usize::from(should_ack && action == RuntimeAction::Reload)
                );
                assert_eq!(
                    control.acknowledge_restart,
                    usize::from(
                        should_ack
                            && matches!(action, RuntimeAction::Start | RuntimeAction::Restart)
                    )
                );
                if failed_index == 1 {
                    assert_ne!(app.runtime_phase(), RuntimeEffect::None);
                }
            }
        }
    }
}

#[test]
fn centralized_fault_helper_enforces_the_lifecycle_only_matrix() {
    for action in [
        RuntimeAction::Start,
        RuntimeAction::Reload,
        RuntimeAction::Restart,
        RuntimeAction::Stop,
    ] {
        for succeeded in [true, false] {
            let (mut app, control) = runtime_app();
            prepare_action(&mut app, action);
            let pending = app.runtime_phase();
            let token = dispatch_request(&mut app, action);
            let lifecycle_at_fault = app.server_state;
            app.snapshot.as_mut().unwrap().enter_future_fault_for_test(
                "synthetic future fault".into(),
                workspace_session::ContractFaultAdoption::PostCommit,
            );
            assert!(app.enter_session_fault_if_any());
            assert_eq!(app.server_state, lifecycle_at_fault);
            assert_eq!(
                app.runtime_in_flight.unwrap().disposition,
                RuntimeCompletionDisposition::LifecycleOnly
            );
            let primary = app.last_problem.clone();
            if succeeded {
                app.update(Message::RuntimeSucceeded(token));
            } else {
                app.update(Message::RuntimeFailed {
                    token,
                    technical: "lifecycle-only failure".into(),
                });
            }
            assert_eq!(
                app.server_state,
                expected_completion_lifecycle(action, succeeded)
            );
            assert_eq!(app.runtime_phase(), pending);
            assert_eq!(app.last_problem, primary);
            assert!(app.runtime_in_flight.is_none());
            assert_eq!(control.borrow().acknowledge_reload, 0);
            assert_eq!(control.borrow().acknowledge_restart, 0);
            let settled = (
                app.server_state,
                app.runtime_phase(),
                app.last_problem.clone(),
            );
            app.update(Message::RuntimeSucceeded(token));
            assert_eq!(
                (
                    app.server_state,
                    app.runtime_phase(),
                    app.last_problem.clone()
                ),
                settled
            );
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum InjectedFaultSource {
    Apply,
    Validation,
    SaveVerified,
    SaveUnverified,
}

#[test]
fn current_fault_sources_downgrade_every_active_action() {
    for source in [
        InjectedFaultSource::Apply,
        InjectedFaultSource::Validation,
        InjectedFaultSource::SaveVerified,
        InjectedFaultSource::SaveUnverified,
    ] {
        for action in [
            RuntimeAction::Start,
            RuntimeAction::Reload,
            RuntimeAction::Restart,
            RuntimeAction::Stop,
        ] {
            for succeeded in [true, false] {
                let (mut app, control) = runtime_app();
                prepare_action(&mut app, action);
                let pending = app.runtime_phase();
                let token = dispatch_request(&mut app, action);
                let revision_at_request = app.snapshot.as_ref().unwrap().saved_config_revision();
                match source {
                    InjectedFaultSource::Apply => {
                        control.borrow_mut().fault_apply = true;
                        app.update(Message::AddRuleSet);
                    }
                    InjectedFaultSource::Validation => {
                        control.borrow_mut().validation_mismatch = true;
                        app.update(Message::OpenValidationDrawer);
                    }
                    InjectedFaultSource::SaveVerified => {
                        app.update(Message::RuleSetUrlPath("/save-fault".into()));
                        control.borrow_mut().malformed_save = true;
                        app.update(Message::Save);
                    }
                    InjectedFaultSource::SaveUnverified => {
                        app.update(Message::RuleSetUrlPath("/unverified-save-fault".into()));
                        control.borrow_mut().malformed_save_progress = true;
                        app.update(Message::Save);
                    }
                }
                assert!(
                    app.snapshot.as_ref().unwrap().faulted,
                    "source={source:?} action={action:?} succeeded={succeeded}"
                );
                assert_eq!(
                    app.runtime_in_flight.unwrap().disposition,
                    RuntimeCompletionDisposition::LifecycleOnly
                );
                let pending_at_completion = app.runtime_phase();
                let revision_at_completion = app.snapshot.as_ref().unwrap().saved_config_revision();
                match source {
                    InjectedFaultSource::SaveVerified => {
                        assert!(revision_at_completion > revision_at_request);
                    }
                    InjectedFaultSource::SaveUnverified => {
                        assert_eq!(revision_at_completion, revision_at_request);
                    }
                    InjectedFaultSource::Apply | InjectedFaultSource::Validation => {
                        assert_eq!(pending_at_completion, pending);
                        assert_eq!(revision_at_completion, revision_at_request);
                    }
                }
                let primary = app.last_problem.clone();
                let snapshot_at_completion =
                    format!("{:?}", app.snapshot.as_ref().unwrap().latest());
                if succeeded {
                    app.update(Message::RuntimeSucceeded(token));
                } else {
                    app.update(Message::RuntimeFailed {
                        token,
                        technical: "completion after fault".into(),
                    });
                }
                assert_eq!(
                    app.server_state,
                    expected_completion_lifecycle(action, succeeded)
                );
                assert_eq!(app.runtime_phase(), pending_at_completion);
                assert_eq!(app.last_problem, primary);
                assert!(app.runtime_in_flight.is_none());
                assert_eq!(
                    format!("{:?}", app.snapshot.as_ref().unwrap().latest()),
                    snapshot_at_completion
                );
                assert_eq!(
                    app.snapshot.as_ref().unwrap().saved_config_revision(),
                    revision_at_completion
                );
                assert_eq!(control.borrow().acknowledge_reload, 0);
                assert_eq!(control.borrow().acknowledge_restart, 0);

                let settled = (
                    app.server_state,
                    app.runtime_phase(),
                    app.last_problem.clone(),
                    format!("{:?}", app.snapshot.as_ref().unwrap().latest()),
                );
                app.update(Message::RuntimeSucceeded(token));
                app.update(Message::RuntimeFailed {
                    token,
                    technical: "stale completion after fault".into(),
                });
                assert_eq!(
                    (
                        app.server_state,
                        app.runtime_phase(),
                        app.last_problem.clone(),
                        format!("{:?}", app.snapshot.as_ref().unwrap().latest()),
                    ),
                    settled
                );
                assert_eq!(control.borrow().acknowledge_reload, 0);
                assert_eq!(control.borrow().acknowledge_restart, 0);
            }
        }
    }
}

#[test]
fn save_fault_revision_uses_only_verified_progress() {
    let (mut verified, verified_control) = runtime_app();
    prepare_action(&mut verified, RuntimeAction::Reload);
    let token = dispatch_request(&mut verified, RuntimeAction::Reload);
    verified.update(Message::RuleSetUrlPath("/verified-save-fault".into()));
    verified_control.borrow_mut().malformed_save = true;
    verified.update(Message::Save);
    assert!(verified.snapshot.as_ref().unwrap().faulted);
    assert!(
        verified.snapshot.as_ref().unwrap().saved_config_revision()
            > token.consumed_revision.unwrap()
    );
    assert_eq!(
        verified.runtime_in_flight.unwrap().disposition,
        RuntimeCompletionDisposition::LifecycleOnly
    );

    let (mut unverified, unverified_control) = runtime_app();
    prepare_action(&mut unverified, RuntimeAction::Reload);
    let token = dispatch_request(&mut unverified, RuntimeAction::Reload);
    unverified.update(Message::RuleSetUrlPath("/unverified-save-fault".into()));
    unverified_control.borrow_mut().malformed_save_progress = true;
    unverified.update(Message::Save);
    assert!(unverified.snapshot.as_ref().unwrap().faulted);
    assert_eq!(
        unverified
            .snapshot
            .as_ref()
            .unwrap()
            .saved_config_revision(),
        token.consumed_revision.unwrap()
    );
    assert_eq!(
        unverified.runtime_in_flight.unwrap().disposition,
        RuntimeCompletionDisposition::LifecycleOnly
    );
}

#[test]
fn acknowledgement_adopts_diagnostics_and_malformed_ack_faults_after_retirement() {
    let (mut adopted, adopted_control) = runtime_app();
    prepare_action(&mut adopted, RuntimeAction::Reload);
    adopted_control.borrow_mut().acknowledgement_diagnostic = true;
    let token = dispatch_request(&mut adopted, RuntimeAction::Reload);
    adopted.update(Message::RuntimeSucceeded(token));
    assert!(adopted.runtime_in_flight.is_none());
    assert!(
        adopted
            .snapshot
            .as_ref()
            .unwrap()
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "runtime acknowledgement diagnostic")
    );

    let (mut malformed, malformed_control) = runtime_app();
    prepare_action(&mut malformed, RuntimeAction::Restart);
    malformed_control.borrow_mut().malformed_ack = true;
    let token = dispatch_request(&mut malformed, RuntimeAction::Restart);
    malformed.update(Message::RuntimeSucceeded(token));
    assert_eq!(malformed.server_state, ServerState::Running);
    assert!(malformed.runtime_in_flight.is_none());
    assert!(malformed.snapshot.as_ref().unwrap().faulted);
    assert_eq!(
        malformed.transient_problem_kind,
        Some(TransientProblemKind::PostCommitContract)
    );
    assert_eq!(malformed_control.borrow().acknowledge_restart, 1);
}

#[test]
fn phase_drives_availability_independently_from_lifecycle() {
    let (mut app, _) = runtime_app();
    save_reload(&mut app, "debug");
    assert!(app.runtime_request_available(RuntimeAction::Reload));
    assert!(!app.runtime_request_available(RuntimeAction::Restart));
    app.server_state = ServerState::Stopped;
    assert!(!app.runtime_request_available(RuntimeAction::Reload));
    assert!(app.runtime_request_available(RuntimeAction::Start));
    let _ = crate::shell::top_bar::view(&app);

    app.server_state = ServerState::Running;
    save_restart(&mut app, 9000);
    assert!(!app.runtime_request_available(RuntimeAction::Reload));
    assert!(app.runtime_request_available(RuntimeAction::Restart));
}
