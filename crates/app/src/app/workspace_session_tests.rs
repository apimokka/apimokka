//! Split by scenario domain (RFC MK-057): draft persistence, validation
//! contract, selection/focus, selection adoption, malformed port contract,
//! workspace-switch confirmation, and edit-history round trips. `expert()`
//! is the one fixture shared across every domain; each domain's own
//! port-mock infrastructure lives beside the tests that use it.

use super::*;
use crate::message::Message;

#[path = "workspace_session_tests/draft_persistence.rs"]
mod draft_persistence;
#[path = "workspace_session_tests/edit_history_round_trips.rs"]
mod edit_history_round_trips;
#[path = "workspace_session_tests/malformed_port_contract.rs"]
mod malformed_port_contract;
#[path = "workspace_session_tests/selection_adoption.rs"]
mod selection_adoption;
#[path = "workspace_session_tests/selection_and_focus.rs"]
mod selection_and_focus;
#[path = "workspace_session_tests/validation_contract.rs"]
mod validation_contract;
#[path = "workspace_session_tests/workspace_switch_confirmation.rs"]
mod workspace_switch_confirmation;

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
