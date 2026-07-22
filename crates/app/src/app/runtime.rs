use super::{App, TransientProblemKind};
use crate::message::Message;
use crate::shell::top_bar::ServerState;
use apimokka_model::{FriendlyProblem, RuntimeEffect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionGeneration(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeRequestId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SavedConfigRevision(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeAction {
    Start,
    Reload,
    Restart,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeRequestToken {
    pub generation: SessionGeneration,
    pub request_id: RuntimeRequestId,
    pub action: RuntimeAction,
    pub consumed_revision: Option<SavedConfigRevision>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCompletionDisposition {
    Full,
    LifecycleOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeInFlight {
    pub token: RuntimeRequestToken,
    pub disposition: RuntimeCompletionDisposition,
}

impl App {
    pub(crate) fn runtime_phase(&self) -> RuntimeEffect {
        self.snapshot
            .as_ref()
            .map(|session| session.latest().runtime_pending())
            .unwrap_or(RuntimeEffect::None)
    }

    pub(crate) fn runtime_request_available(&self, action: RuntimeAction) -> bool {
        if self.runtime_in_flight.is_some() {
            return false;
        }
        let Some(session) = self.snapshot.as_ref() else {
            return false;
        };
        if session.faulted {
            return false;
        }
        matches!(
            (action, self.server_state, self.runtime_phase()),
            (
                RuntimeAction::Start,
                ServerState::Stopped | ServerState::Error,
                _
            ) | (RuntimeAction::Stop, ServerState::Running, _)
                | (
                    RuntimeAction::Reload,
                    ServerState::Running,
                    RuntimeEffect::Reload
                )
                | (
                    RuntimeAction::Restart,
                    ServerState::Running,
                    RuntimeEffect::Restart
                )
        )
    }

    pub(super) fn request_runtime(&mut self, action: RuntimeAction) {
        if !self.runtime_request_available(action) {
            return;
        }
        let token = self
            .snapshot
            .as_mut()
            .expect("availability requires an installed session")
            .issue_runtime_request(action);
        self.runtime_in_flight = Some(RuntimeInFlight {
            token,
            disposition: RuntimeCompletionDisposition::Full,
        });
        if matches!(action, RuntimeAction::Start | RuntimeAction::Restart) {
            self.server_state = ServerState::Starting;
        }
        if self.runtime_auto_complete {
            self.update(Message::RuntimeSucceeded(token));
        }
    }

    pub(super) fn complete_runtime_success(&mut self, token: RuntimeRequestToken) {
        let Some(active) = self
            .runtime_in_flight
            .filter(|active| active.token == token)
        else {
            return;
        };
        self.runtime_in_flight = None;
        if active.disposition == RuntimeCompletionDisposition::LifecycleOnly {
            self.apply_lifecycle_only_completion(token.action, true);
            return;
        }

        match token.action {
            RuntimeAction::Stop => {
                self.server_state = ServerState::Stopped;
                self.clear_runtime_problem();
            }
            RuntimeAction::Start | RuntimeAction::Reload | RuntimeAction::Restart => {
                self.server_state = ServerState::Running;
                let current_revision = self
                    .snapshot
                    .as_ref()
                    .expect("matching token requires its installed session")
                    .saved_config_revision();
                if token.consumed_revision != Some(current_revision) {
                    self.present_newer_saved_configuration();
                    return;
                }
                let selection_target = self.capture_selection_target();
                let session = self
                    .snapshot
                    .as_mut()
                    .expect("matching token requires its installed session");
                match token.action {
                    RuntimeAction::Reload => session.acknowledge_reload(),
                    RuntimeAction::Start | RuntimeAction::Restart => session.acknowledge_restart(),
                    RuntimeAction::Stop => unreachable!(),
                }
                self.reconcile_selection(selection_target);
                if self.enter_session_fault_if_any() {
                    return;
                }
                self.clear_runtime_problem();
                self.recompute_dirty();
            }
        }
    }

    pub(super) fn complete_runtime_failure(
        &mut self,
        token: RuntimeRequestToken,
        technical: String,
    ) {
        let Some(active) = self
            .runtime_in_flight
            .filter(|active| active.token == token)
        else {
            return;
        };
        self.runtime_in_flight = None;
        if active.disposition == RuntimeCompletionDisposition::LifecycleOnly {
            self.apply_lifecycle_only_completion(token.action, false);
            return;
        }
        self.server_state = match token.action {
            RuntimeAction::Start | RuntimeAction::Restart => ServerState::Error,
            RuntimeAction::Reload | RuntimeAction::Stop => ServerState::Running,
        };
        self.transient_problem_kind = Some(TransientProblemKind::Runtime);
        self.transient_problem_operation = None;
        self.last_problem = Some(
            FriendlyProblem::new(
                "Runtime action failed",
                "The saved-configuration runtime phase was retained. Retry the available server action.",
                None,
            )
            .with_technical(technical),
        );
    }

    fn apply_lifecycle_only_completion(&mut self, action: RuntimeAction, succeeded: bool) {
        self.server_state = match (action, succeeded) {
            (RuntimeAction::Start | RuntimeAction::Restart, true) => ServerState::Running,
            (RuntimeAction::Start | RuntimeAction::Restart, false) => ServerState::Error,
            (RuntimeAction::Reload, _) | (RuntimeAction::Stop, false) => ServerState::Running,
            (RuntimeAction::Stop, true) => ServerState::Stopped,
        };
    }

    fn present_newer_saved_configuration(&mut self) {
        let action = match self.runtime_phase() {
            RuntimeEffect::Reload => "reload",
            RuntimeEffect::Restart => "restart",
            RuntimeEffect::None => "runtime action",
        };
        self.transient_problem_kind = Some(TransientProblemKind::Runtime);
        self.transient_problem_operation = None;
        self.last_problem = Some(FriendlyProblem::new(
            "Newer saved configuration is pending",
            format!("Runtime action succeeded; newer saved configuration still needs {action}."),
            None,
        ));
    }

    fn clear_runtime_problem(&mut self) {
        if self.transient_problem_kind == Some(TransientProblemKind::Runtime) {
            self.clear_transient_problem();
        }
    }

    pub(super) fn enter_session_fault_if_any(&mut self) -> bool {
        let fault = self.snapshot.as_ref().and_then(|session| {
            session.faulted.then(|| {
                (
                    session.contract_fault.clone(),
                    session.contract_fault_adoption,
                )
            })
        });
        let Some((technical, adoption)) = fault else {
            return false;
        };
        if let Some(active) = self.runtime_in_flight.as_mut() {
            active.disposition = RuntimeCompletionDisposition::LifecycleOnly;
        }
        let technical = technical.unwrap_or_else(|| "workspace session is faulted".into());
        match adoption {
            Some(super::workspace_session::ContractFaultAdoption::NonAdoptingRead) => {
                self.present_cached_workspace_problem("Workspace reload required", technical)
            }
            _ => self.present_adopted_workspace_problem("Workspace reload required", technical),
        }
        self.recompute_dirty();
        true
    }
}
