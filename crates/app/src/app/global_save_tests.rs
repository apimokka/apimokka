use super::*;
use crate::message::Message;
use crate::selection::DrawerMode;
use apimokka_model::workspace_port::parse_workspace_relative_path;
use apimokka_model::{
    ApplyFailure, EditOutcome, EditTransaction, FileDiff, MemoryWorkspace, PortSnapshot,
    RuntimeEffect, SaveFailure, SaveOutcome, ValidationReport, WorkspacePort,
};
use iced::widget::text_editor::Content;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Default)]
enum SaveMutation {
    #[default]
    None,
    Omit,
    Duplicate,
    Reorder,
    Unexpected,
    PathDiffMismatch,
    DirtySnapshot,
    PhaseMismatch,
    FullPrefixFailure,
    StructuralMismatch,
    WrongFailedFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawCapture {
    progress: WorkspaceSaveProgress,
    unsaved_hint: RuntimeEffect,
    runtime_pending: RuntimeEffect,
}

#[derive(Default)]
struct SaveProbeControl {
    fail_at: Option<usize>,
    mutation: SaveMutation,
    raw: Option<RawCapture>,
}

struct SaveProbePort {
    inner: MemoryWorkspace,
    control: Rc<RefCell<SaveProbeControl>>,
}

impl WorkspacePort for SaveProbePort {
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
        let before = self.inner.snapshot();
        let (fail_at, mutation) = {
            let mut control = self.control.borrow_mut();
            (control.fail_at.take(), control.mutation)
        };
        if let Some(index) = fail_at {
            self.inner
                .inject_save_failure(before.dirty_files()[index].path.clone())
                .unwrap();
        }
        let unexpected = parse_workspace_relative_path("path", "unexpected.toml").unwrap();
        let result = match self.inner.save() {
            Ok(mut outcome) => {
                match mutation {
                    SaveMutation::None
                    | SaveMutation::FullPrefixFailure
                    | SaveMutation::WrongFailedFile => {}
                    SaveMutation::Omit => {
                        outcome.written_files.pop();
                        outcome.diffs.pop();
                    }
                    SaveMutation::Duplicate => {
                        if let (Some(path), Some(diff)) = (
                            outcome.written_files.first().cloned(),
                            outcome.diffs.first().cloned(),
                        ) {
                            outcome.written_files.push(path);
                            outcome.diffs.push(diff);
                        }
                    }
                    SaveMutation::Reorder => {
                        outcome.written_files.swap(0, 1);
                        outcome.diffs.swap(0, 1);
                    }
                    SaveMutation::Unexpected => {
                        if outcome.diffs.is_empty() {
                            outcome.written_files.push(unexpected.clone());
                            outcome.diffs.push(FileDiff {
                                path: unexpected,
                                effect: RuntimeEffect::None,
                            });
                        } else {
                            outcome.written_files[0] = unexpected.clone();
                            outcome.diffs[0].path = unexpected;
                        }
                    }
                    SaveMutation::PathDiffMismatch => {
                        outcome.written_files[0] = unexpected;
                    }
                    SaveMutation::DirtySnapshot => outcome.snapshot = before,
                    SaveMutation::PhaseMismatch => {
                        outcome.runtime_pending = different_effect(outcome.runtime_pending);
                    }
                    SaveMutation::StructuralMismatch => {
                        outcome.snapshot.contract_test_workspace_mut().rule_sets[0]
                            .rules
                            .remove(0);
                    }
                }
                Ok(outcome)
            }
            Err(mut failure) => {
                match mutation {
                    SaveMutation::None => {}
                    SaveMutation::Omit => {
                        failure.written_files.pop();
                        failure.diffs.pop();
                    }
                    SaveMutation::Duplicate => {
                        if let (Some(path), Some(diff)) = (
                            failure.written_files.first().cloned(),
                            failure.diffs.first().cloned(),
                        ) {
                            failure.written_files.push(path);
                            failure.diffs.push(diff);
                        }
                    }
                    SaveMutation::Reorder => {
                        failure.written_files.swap(0, 1);
                        failure.diffs.swap(0, 1);
                    }
                    SaveMutation::Unexpected => {
                        if failure.diffs.is_empty() {
                            failure.written_files.push(unexpected.clone());
                            failure.diffs.push(FileDiff {
                                path: unexpected,
                                effect: RuntimeEffect::None,
                            });
                        } else {
                            failure.written_files[0] = unexpected.clone();
                            failure.diffs[0].path = unexpected;
                        }
                    }
                    SaveMutation::PathDiffMismatch => {
                        failure.written_files[0] = unexpected;
                    }
                    SaveMutation::DirtySnapshot => failure.snapshot = Box::new(before),
                    SaveMutation::PhaseMismatch => {
                        failure.runtime_pending = different_effect(failure.runtime_pending);
                    }
                    SaveMutation::FullPrefixFailure => {
                        let failed = before.dirty_files()[failure.written_files.len()].clone();
                        failure.written_files.push(failed.path.clone());
                        failure.diffs.push(failed);
                    }
                    SaveMutation::StructuralMismatch => {
                        failure.snapshot.contract_test_workspace_mut().rule_sets[0]
                            .rules
                            .remove(0);
                    }
                    SaveMutation::WrongFailedFile => {
                        failure.failed_file = unexpected;
                    }
                }
                Err(failure)
            }
        };
        let raw = match &result {
            Ok(outcome) => RawCapture {
                progress: WorkspaceSaveProgress::Saved {
                    written_files: outcome.written_files.clone(),
                    diffs: outcome.diffs.clone(),
                },
                unsaved_hint: outcome.unsaved_hint,
                runtime_pending: outcome.runtime_pending,
            },
            Err(failure) => RawCapture {
                progress: WorkspaceSaveProgress::Failed {
                    written_files: failure.written_files.clone(),
                    diffs: failure.diffs.clone(),
                    failed_file: failure.failed_file.clone(),
                    cause: failure.cause.clone(),
                },
                unsaved_hint: failure.unsaved_hint,
                runtime_pending: failure.runtime_pending,
            },
        };
        self.control.borrow_mut().raw = Some(raw);
        result
    }

    fn acknowledge_reload(&mut self) -> PortSnapshot {
        self.inner.acknowledge_reload()
    }

    fn acknowledge_restart(&mut self) -> PortSnapshot {
        self.inner.acknowledge_restart()
    }
}

fn different_effect(effect: RuntimeEffect) -> RuntimeEffect {
    match effect {
        RuntimeEffect::Restart => RuntimeEffect::Reload,
        RuntimeEffect::None | RuntimeEffect::Reload => RuntimeEffect::Restart,
    }
}

fn global_save_app() -> (App, Rc<RefCell<SaveProbeControl>>) {
    let mut app = App::new().0;
    app.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    app.update(Message::OpenWorkspace("global-save".into()));
    let generation = app.snapshot.as_ref().unwrap().generation();
    let control = Rc::new(RefCell::new(SaveProbeControl::default()));
    let mut inner = MemoryWorkspace::new(apimokka_model::mock::shop_api_canonical_seed()).unwrap();
    inner.save().unwrap();
    inner.acknowledge_restart();
    app.snapshot = Some(WorkspaceSession::from_port_with_generation(
        Box::new(SaveProbePort {
            inner,
            control: control.clone(),
        }),
        workspace_session::PrototypeState::default(),
        generation,
    ));
    app.selection = crate::selection::RouteSelection::default();
    app.recompute_dirty();
    (app, control)
}

fn dirty_workspace(app: &mut App, count: usize) -> Vec<FileDiff> {
    assert!(count <= 3);
    if count >= 1 {
        app.update(Message::SettingsSetPort("9001".into()));
    }
    if count >= 2 {
        let first = app.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        app.update(Message::SelectRule(first));
        app.update(Message::RuleSetUrlPath("/save-first".into()));
    }
    if count >= 3 {
        let second = app.snapshot.as_ref().unwrap().rule_sets[1].rules[0].id;
        app.update(Message::SelectRule(second));
        app.update(Message::RuleSetUrlPath("/save-second".into()));
    }
    let dirty = app
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .dirty_files()
        .to_vec();
    assert_eq!(dirty.len(), count);
    dirty
}

#[derive(Debug, Clone, Copy)]
struct MalformedCase {
    name: &'static str,
    dirty_count: usize,
    fail_at: Option<usize>,
    mutation: SaveMutation,
    trust: ProgressTrust,
}

#[test]
fn malformed_workspace_envelopes_preserve_raw_evidence_and_classify_trust() {
    let cases = [
        MalformedCase {
            name: "outcome zero phase",
            dirty_count: 0,
            fail_at: None,
            mutation: SaveMutation::PhaseMismatch,
            trust: ProgressTrust::Verified,
        },
        MalformedCase {
            name: "outcome full phase",
            dirty_count: 3,
            fail_at: None,
            mutation: SaveMutation::PhaseMismatch,
            trust: ProgressTrust::Verified,
        },
        MalformedCase {
            name: "outcome structural mismatch",
            dirty_count: 0,
            fail_at: None,
            mutation: SaveMutation::StructuralMismatch,
            trust: ProgressTrust::Verified,
        },
        MalformedCase {
            name: "outcome omitted",
            dirty_count: 3,
            fail_at: None,
            mutation: SaveMutation::Omit,
            trust: ProgressTrust::Unverified,
        },
        MalformedCase {
            name: "outcome duplicate",
            dirty_count: 3,
            fail_at: None,
            mutation: SaveMutation::Duplicate,
            trust: ProgressTrust::Unverified,
        },
        MalformedCase {
            name: "outcome reordered",
            dirty_count: 3,
            fail_at: None,
            mutation: SaveMutation::Reorder,
            trust: ProgressTrust::Unverified,
        },
        MalformedCase {
            name: "outcome unexpected",
            dirty_count: 3,
            fail_at: None,
            mutation: SaveMutation::Unexpected,
            trust: ProgressTrust::Unverified,
        },
        MalformedCase {
            name: "outcome path mismatch",
            dirty_count: 3,
            fail_at: None,
            mutation: SaveMutation::PathDiffMismatch,
            trust: ProgressTrust::Unverified,
        },
        MalformedCase {
            name: "outcome dirty baseline",
            dirty_count: 3,
            fail_at: None,
            mutation: SaveMutation::DirtySnapshot,
            trust: ProgressTrust::Unverified,
        },
        MalformedCase {
            name: "failure zero phase",
            dirty_count: 3,
            fail_at: Some(0),
            mutation: SaveMutation::PhaseMismatch,
            trust: ProgressTrust::Verified,
        },
        MalformedCase {
            name: "failure prefix phase",
            dirty_count: 3,
            fail_at: Some(1),
            mutation: SaveMutation::PhaseMismatch,
            trust: ProgressTrust::Verified,
        },
        MalformedCase {
            name: "failure structural mismatch",
            dirty_count: 3,
            fail_at: Some(0),
            mutation: SaveMutation::StructuralMismatch,
            trust: ProgressTrust::Verified,
        },
        MalformedCase {
            name: "failure omitted",
            dirty_count: 3,
            fail_at: Some(1),
            mutation: SaveMutation::Omit,
            trust: ProgressTrust::Unverified,
        },
        MalformedCase {
            name: "failure duplicate",
            dirty_count: 3,
            fail_at: Some(1),
            mutation: SaveMutation::Duplicate,
            trust: ProgressTrust::Unverified,
        },
        MalformedCase {
            name: "failure reordered",
            dirty_count: 3,
            fail_at: Some(2),
            mutation: SaveMutation::Reorder,
            trust: ProgressTrust::Unverified,
        },
        MalformedCase {
            name: "failure unexpected",
            dirty_count: 3,
            fail_at: Some(1),
            mutation: SaveMutation::Unexpected,
            trust: ProgressTrust::Unverified,
        },
        MalformedCase {
            name: "failure path mismatch",
            dirty_count: 3,
            fail_at: Some(1),
            mutation: SaveMutation::PathDiffMismatch,
            trust: ProgressTrust::Unverified,
        },
        MalformedCase {
            name: "failure wrong failed file",
            dirty_count: 3,
            fail_at: Some(1),
            mutation: SaveMutation::WrongFailedFile,
            trust: ProgressTrust::Unverified,
        },
        MalformedCase {
            name: "failure dirty suffix",
            dirty_count: 3,
            fail_at: Some(1),
            mutation: SaveMutation::DirtySnapshot,
            trust: ProgressTrust::Unverified,
        },
        MalformedCase {
            name: "failure full prefix",
            dirty_count: 3,
            fail_at: Some(2),
            mutation: SaveMutation::FullPrefixFailure,
            trust: ProgressTrust::Unverified,
        },
    ];

    for case in cases {
        let (mut app, control) = global_save_app();
        dirty_workspace(&mut app, case.dirty_count);
        {
            let mut control = control.borrow_mut();
            control.fail_at = case.fail_at;
            control.mutation = case.mutation;
        }
        app.update(Message::Save);
        let raw = control.borrow().raw.clone().unwrap();
        let report = app.last_save_report.as_ref().unwrap();
        assert_eq!(report.workspace.progress, raw.progress, "{}", case.name);
        assert_eq!(
            report.workspace.unsaved_hint, raw.unsaved_hint,
            "{}",
            case.name
        );
        assert_eq!(
            report.workspace.runtime_pending, raw.runtime_pending,
            "{}",
            case.name
        );
        assert_eq!(
            report.workspace.integrity,
            SaveIntegrity::ContractFault {
                reason: app
                    .snapshot
                    .as_ref()
                    .unwrap()
                    .contract_fault
                    .clone()
                    .unwrap(),
                progress_trust: case.trust,
            },
            "{}",
            case.name
        );
        assert!(matches!(
            report.fallback,
            FallbackSaveReport::NotEntered {
                reason: FallbackSkipReason::WorkspaceContractFault,
                ..
            }
        ));
        assert_eq!(
            report.completion(),
            if case.trust == ProgressTrust::Unverified {
                GlobalSaveCompletion::Indeterminate
            } else if report.workspace.progress.written_files().is_empty() {
                GlobalSaveCompletion::Failed
            } else {
                GlobalSaveCompletion::Partial
            },
            "{}",
            case.name
        );
        assert_eq!(app.drawer, Some(DrawerMode::SaveDiff), "{}", case.name);
        let detail = &app.last_problem.as_ref().unwrap().detail;
        if case.trust == ProgressTrust::Unverified {
            assert!(detail.contains("could not be verified"), "{}", case.name);
        } else {
            assert!(detail.contains("verified saved prefix"), "{}", case.name);
        }
    }
}

#[test]
fn malformed_full_snapshots_are_adopted_faulted_and_reconciled_without_revision_advance() {
    for (fail_at, dirty_count) in [(None, 0), (Some(0), 3)] {
        let (mut app, control) = global_save_app();
        dirty_workspace(&mut app, dirty_count);
        let removed_rule = app.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        app.update(Message::SelectRule(removed_rule));
        let revision = app.snapshot.as_ref().unwrap().saved_config_revision();
        {
            let mut control = control.borrow_mut();
            control.fail_at = fail_at;
            control.mutation = SaveMutation::StructuralMismatch;
        }

        app.update(Message::Save);

        let raw = control.borrow().raw.clone().unwrap();
        let report = app.last_save_report.as_ref().unwrap();
        assert_eq!(report.workspace.progress, raw.progress);
        assert_eq!(report.workspace.unsaved_hint, raw.unsaved_hint);
        assert_eq!(report.workspace.runtime_pending, raw.runtime_pending);
        assert!(matches!(
            report.workspace.integrity,
            SaveIntegrity::ContractFault {
                progress_trust: ProgressTrust::Verified,
                ..
            }
        ));
        assert_eq!(
            app.snapshot.as_ref().unwrap().saved_config_revision(),
            revision
        );
        assert!(app.snapshot.as_ref().unwrap().faulted);
        assert!(matches!(
            report.fallback,
            FallbackSaveReport::NotEntered {
                reason: FallbackSkipReason::WorkspaceContractFault,
                ..
            }
        ));
        assert!(
            app.snapshot
                .as_ref()
                .unwrap()
                .find_rule(removed_rule)
                .is_none()
        );
        assert_eq!(app.selection.rule, None);
    }
}

fn dirty_fallbacks(app: &mut App) -> Vec<String> {
    let mut keys = app.fallback_saved.keys().cloned().collect::<Vec<_>>();
    keys.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    for (index, key) in keys.iter().enumerate() {
        app.fallback_drafts.insert(
            key.clone(),
            Content::with_text(&format!("{{\"changed\":{index}}}")),
        );
        app.fallback_status_draft
            .insert(key.clone(), format!("20{index} Changed"));
    }
    keys
}

#[test]
fn fallback_failures_are_ordered_atomic_and_retry_only_remaining_scopes() {
    for failed_index in 0..3 {
        let (mut app, _) = global_save_app();
        let keys = dirty_fallbacks(&mut app);
        let revision = app.snapshot.as_ref().unwrap().saved_config_revision();
        let before_content = app.fallback_saved.clone();
        let before_status = app.fallback_status_saved.clone();
        let mut calls = Vec::new();
        let completion = app
            .save_workspace_and_fallbacks_with(|key, _, _| {
                calls.push(key.to_owned());
                if key == keys[failed_index] {
                    Err(FallbackSaveError::new("injected fallback failure"))
                } else {
                    Ok(())
                }
            })
            .unwrap();
        assert_eq!(calls, keys[..=failed_index]);
        assert_eq!(
            app.snapshot.as_ref().unwrap().saved_config_revision(),
            revision
        );
        assert_eq!(
            completion,
            if failed_index == 0 {
                GlobalSaveCompletion::Failed
            } else {
                GlobalSaveCompletion::Partial
            }
        );
        assert_eq!(app.drawer, Some(DrawerMode::SaveDiff));
        assert_eq!(app.transient_problem_kind, Some(TransientProblemKind::Save));
        let report = app.last_save_report.as_ref().unwrap();
        assert_eq!(
            report.workspace.unsaved_hint,
            app.snapshot.as_ref().unwrap().latest().unsaved_hint()
        );
        assert_eq!(
            report.workspace.runtime_pending,
            app.snapshot.as_ref().unwrap().latest().runtime_pending()
        );
        assert_eq!(
            report.fallback,
            FallbackSaveReport::Failed {
                written_keys: keys[..failed_index].to_vec(),
                failure: FallbackSaveFailure {
                    key: keys[failed_index].clone(),
                    cause: FallbackSaveError::new("injected fallback failure"),
                },
                remaining_keys: keys[failed_index..].to_vec(),
            }
        );
        for key in &keys[..failed_index] {
            assert_eq!(app.fallback_saved[key], app.fallback_drafts[key].text());
            assert_eq!(
                app.fallback_status_saved[key],
                app.fallback_status_draft[key]
            );
            assert!(!app.is_fallback_dirty(key));
        }
        for key in &keys[failed_index..] {
            assert_eq!(app.fallback_saved[key], before_content[key]);
            assert_eq!(app.fallback_status_saved[key], before_status[key]);
            assert!(app.is_fallback_dirty(key));
        }

        let mut retry_calls = Vec::new();
        assert_eq!(
            app.save_workspace_and_fallbacks_with(|key, _, _| {
                retry_calls.push(key.to_owned());
                Ok(())
            }),
            Some(GlobalSaveCompletion::Complete)
        );
        assert_eq!(retry_calls, keys[failed_index..]);
        assert_eq!(
            app.snapshot.as_ref().unwrap().saved_config_revision(),
            revision
        );
        assert!(keys.iter().all(|key| !app.is_fallback_dirty(key)));
        assert_eq!(
            app.last_save_report.as_ref().unwrap().fallback,
            FallbackSaveReport::Completed {
                written_keys: keys[failed_index..].to_vec()
            }
        );
    }
}

#[test]
fn workspace_failures_preserve_prefix_skip_fallback_and_retry_remaining_scopes() {
    for failed_index in 0..3 {
        let (mut app, control) = global_save_app();
        let dirty = dirty_workspace(&mut app, 3);
        let fallback_keys = dirty_fallbacks(&mut app);
        control.borrow_mut().fail_at = Some(failed_index);
        app.update(Message::Save);
        let report = app.last_save_report.as_ref().unwrap();
        assert_eq!(
            report.workspace.progress,
            WorkspaceSaveProgress::Failed {
                written_files: dirty[..failed_index]
                    .iter()
                    .map(|diff| diff.path.clone())
                    .collect(),
                diffs: dirty[..failed_index].to_vec(),
                failed_file: dirty[failed_index].path.clone(),
                cause: match &report.workspace.progress {
                    WorkspaceSaveProgress::Failed { cause, .. } => cause.clone(),
                    WorkspaceSaveProgress::Saved { .. } => unreachable!(),
                },
            }
        );
        assert_eq!(
            report.fallback,
            FallbackSaveReport::NotEntered {
                reason: FallbackSkipReason::WorkspaceFailed,
                remaining_keys: fallback_keys.clone(),
            }
        );
        assert_eq!(
            app.snapshot.as_ref().unwrap().latest().dirty_files(),
            &dirty[failed_index..]
        );
        assert_eq!(
            report.workspace.unsaved_hint,
            app.snapshot.as_ref().unwrap().latest().unsaved_hint()
        );
        assert_eq!(
            report.workspace.runtime_pending,
            app.snapshot.as_ref().unwrap().latest().runtime_pending()
        );
        assert_eq!(
            report.completion(),
            if failed_index == 0 {
                GlobalSaveCompletion::Failed
            } else {
                GlobalSaveCompletion::Partial
            }
        );
        assert_eq!(app.drawer, Some(DrawerMode::SaveDiff));
        assert_eq!(app.transient_problem_kind, Some(TransientProblemKind::Save));
        assert!(fallback_keys.iter().all(|key| app.is_fallback_dirty(key)));

        app.update(Message::Save);
        let retry = app.last_save_report.as_ref().unwrap();
        assert_eq!(retry.completion(), GlobalSaveCompletion::Complete);
        assert_eq!(
            retry.workspace.progress,
            WorkspaceSaveProgress::Saved {
                written_files: dirty[failed_index..]
                    .iter()
                    .map(|diff| diff.path.clone())
                    .collect(),
                diffs: dirty[failed_index..].to_vec(),
            }
        );
        assert!(fallback_keys.iter().all(|key| !app.is_fallback_dirty(key)));
    }
}

#[test]
fn reports_are_historical_replaced_and_reset_with_the_session() {
    let (mut app, _) = global_save_app();
    app.update(Message::Save);
    let no_op = app.last_save_report.clone().unwrap();
    assert_eq!(no_op.completion(), GlobalSaveCompletion::Complete);
    assert!(no_op.workspace.progress.written_files().is_empty());

    dirty_workspace(&mut app, 1);
    assert_eq!(app.last_save_report, Some(no_op.clone()));
    app.update(Message::Save);
    assert_ne!(app.last_save_report, Some(no_op));

    let historical = app.last_save_report.clone();
    let generation = app.snapshot.as_ref().unwrap().generation();
    let mut invalid = apimokka_model::mock::shop_api_canonical_seed();
    invalid.rule_sets[0].rules[0].id = invalid.rule_sets[0].id.0;
    assert!(!app.install_workspace(invalid));
    assert_eq!(app.snapshot.as_ref().unwrap().generation(), generation);
    assert_eq!(app.last_save_report, historical);

    let fallback = app.fallback_saved.keys().next().unwrap().clone();
    app.update(Message::SelectFileRoute(fallback.clone()));
    app.fallback_drafts
        .insert(fallback, Content::with_text("{\"individual\":true}"));
    let historical = app.last_save_report.clone();
    app.update(Message::FallbackFileSave);
    assert_eq!(app.last_save_report, historical);

    assert!(app.install_workspace(apimokka_model::mock::shop_api_canonical_seed()));
    assert!(app.last_save_report.is_none());
    app.update(Message::Save);
    assert!(app.last_save_report.is_some());
    app.leave_workspace();
    assert!(app.last_save_report.is_none());
}

fn has_label(lines: &[String], label: &str) -> bool {
    lines
        .iter()
        .any(|line| line.starts_with(&format!("{label}:")))
}

fn assert_common_presentation(app: &App, completion: apimokka_i18n::Key) -> Vec<String> {
    let lines = crate::shell::bottom_drawer::last_save_report_lines(app);
    assert_eq!(lines[0], app.t(completion));
    assert!(has_label(
        &lines,
        app.t(apimokka_i18n::Key::SaveAttemptPhases)
    ));
    let _ = crate::shell::bottom_drawer::view(app);
    lines
}

#[test]
fn last_attempt_drawer_truthfully_presents_all_save_states_in_both_locales_and_modes() {
    for locale in [Locale::En, Locale::Ja] {
        for mode in [
            apimokka_model::AudienceMode::Guided,
            apimokka_model::AudienceMode::Expert,
        ] {
            for (failed_index, completion) in [
                (0, apimokka_i18n::Key::SaveCompletionFailed),
                (1, apimokka_i18n::Key::SaveCompletionPartial),
            ] {
                let (mut app, control) = global_save_app();
                dirty_workspace(&mut app, 3);
                control.borrow_mut().fail_at = Some(failed_index);
                app.update(Message::Save);
                app.locale = locale;
                app.audience_mode = Some(mode);
                let lines = assert_common_presentation(&app, completion);
                assert!(has_label(
                    &lines,
                    app.t(apimokka_i18n::Key::SaveVerifiedWritten)
                ));
                assert!(has_label(&lines, app.t(apimokka_i18n::Key::SaveFailedFile)));
                assert!(has_label(
                    &lines,
                    app.t(apimokka_i18n::Key::SaveRemainingScopes)
                ));
            }

            let (mut app, control) = global_save_app();
            control.borrow_mut().mutation = SaveMutation::StructuralMismatch;
            app.update(Message::Save);
            app.locale = locale;
            app.audience_mode = Some(mode);
            let lines = assert_common_presentation(&app, apimokka_i18n::Key::SaveCompletionFailed);
            assert!(has_label(
                &lines,
                app.t(apimokka_i18n::Key::SaveIntegrityFailure)
            ));
            assert!(has_label(
                &lines,
                app.t(apimokka_i18n::Key::SaveVerifiedWritten)
            ));
            assert!(has_label(
                &lines,
                app.t(apimokka_i18n::Key::SaveRemainingScopes)
            ));

            let (mut app, control) = global_save_app();
            dirty_workspace(&mut app, 3);
            {
                let mut control = control.borrow_mut();
                control.fail_at = Some(1);
                control.mutation = SaveMutation::WrongFailedFile;
            }
            app.update(Message::Save);
            app.locale = locale;
            app.audience_mode = Some(mode);
            let raw = control.borrow().raw.clone().unwrap();
            let lines =
                assert_common_presentation(&app, apimokka_i18n::Key::SaveCompletionIndeterminate);
            assert!(has_label(
                &lines,
                app.t(apimokka_i18n::Key::SaveIntegrityFailure)
            ));
            assert!(has_label(
                &lines,
                app.t(apimokka_i18n::Key::SaveReportedWritten)
            ));
            assert!(has_label(
                &lines,
                app.t(apimokka_i18n::Key::SaveReportedDiffs)
            ));
            assert!(has_label(
                &lines,
                app.t(apimokka_i18n::Key::SaveReportedFailure)
            ));
            assert!(!has_label(
                &lines,
                app.t(apimokka_i18n::Key::SaveFailedFile)
            ));
            let WorkspaceSaveProgress::Failed {
                failed_file, cause, ..
            } = raw.progress
            else {
                unreachable!()
            };
            assert!(lines.iter().any(|line| line.contains(failed_file.as_str())));
            assert!(lines.iter().any(|line| line.contains(cause.detail())));
            assert!(has_label(
                &lines,
                app.t(apimokka_i18n::Key::SaveRemainingScopes)
            ));

            let (mut app, _) = global_save_app();
            let fallback_keys = dirty_fallbacks(&mut app);
            let failed = fallback_keys[0].clone();
            app.save_workspace_and_fallbacks_with(|key, _, _| {
                if key == failed {
                    Err(FallbackSaveError::new("presentation fallback failure"))
                } else {
                    Ok(())
                }
            });
            app.locale = locale;
            app.audience_mode = Some(mode);
            let lines = assert_common_presentation(&app, apimokka_i18n::Key::SaveCompletionFailed);
            assert!(has_label(
                &lines,
                app.t(apimokka_i18n::Key::SaveVerifiedWritten)
            ));
            assert!(has_label(
                &lines,
                app.t(apimokka_i18n::Key::SaveFallbackWritten)
            ));
            assert!(has_label(&lines, app.t(apimokka_i18n::Key::SaveFailedFile)));
            assert!(has_label(
                &lines,
                app.t(apimokka_i18n::Key::SaveRemainingScopes)
            ));
        }
    }
}
