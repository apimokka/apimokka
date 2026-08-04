//! Selection reconciliation when a save, reload, or restart acknowledgement
//! adopts a snapshot in which the selected rule was concurrently removed.

use super::*;
use crate::message::Message;
use apimokka_model::workspace_port::map_root_setting;
use apimokka_model::{
    ApplyFailure, EditIntent, EditOutcome, EditTransaction, MemoryWorkspace, NodeId, PortSnapshot,
    RuleSetId, SaveFailure, SaveOutcome, ValidationReport, WorkspacePort,
};

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

fn selection_adoption_app(source: SelectionAdoptionSource) -> (App, NodeId, RuleSetId) {
    let mut app = expert();
    let seed = apimokka_model::mock::shop_api_canonical_seed();
    let selected_rule = seed.rule_sets[0].rules[0].id;
    let parent = seed.rule_sets[0].id;
    let mut inner = MemoryWorkspace::new(seed).unwrap();
    let pending_edit = match source {
        SelectionAdoptionSource::ReloadAcknowledgement => Some(map_root_setting(
            WorkspaceRootKey::LogLevel,
            WorkspaceEditValue::Enum("debug".into()),
        )),
        SelectionAdoptionSource::RestartAcknowledgement => Some(map_root_setting(
            WorkspaceRootKey::ListenerPort,
            WorkspaceEditValue::Integer(9000),
        )),
        SelectionAdoptionSource::SaveSuccess | SelectionAdoptionSource::SaveFailure => None,
    };
    if let Some(edit) = pending_edit {
        inner
            .apply(
                EditTransaction::new(vec![EditIntent::UpdateRootSetting(edit.unwrap())]).unwrap(),
            )
            .unwrap();
        inner.save().unwrap();
    }
    let port = SelectionAdoptionPort {
        inner,
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
