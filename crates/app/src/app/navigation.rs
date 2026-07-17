use super::*;
use crate::message::Message;

#[test]
fn app_starts_at_welcome_with_no_snapshot() {
    let a = App::new().0;
    assert!(
        matches!(a.view, AppView::Welcome),
        "app must start at Welcome, not Workspace"
    );
    assert!(
        a.snapshot.is_none(),
        "no snapshot until user opens a workspace"
    );
    assert!(
        a.audience_mode.is_none(),
        "no audience mode until first-run picker is answered"
    );
}

#[test]
fn mode_picker_view_renders_full_screen_before_mode_chosen() {
    let a = App::new().0;
    // App::view() should return the mode picker, not the workspace shell.
    // We verify by building the element — if it panics, the test fails.
    let _ = a.view();
}

#[test]
fn choosing_mode_then_opening_workspace_reaches_routes() {
    let mut a = App::new().0;
    // First: the mode picker shows — choose Expert
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    assert!(matches!(a.view, AppView::Welcome));

    // Click "Open workspace" → Dashboard → click workspace
    a.update(Message::GoDashboard);
    assert!(matches!(a.view, AppView::Dashboard));

    a.update(Message::OpenWorkspace("payments-mock".into()));
    assert!(matches!(a.view, AppView::Workspace));
    assert!(a.snapshot.is_some(), "snapshot loaded after OpenWorkspace");
    assert_eq!(a.tab, crate::selection::WorkspaceTab::Routes);
}

#[test]
fn wizard_flow_opens_workspace() {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::GoWizard);
    assert!(matches!(a.view, AppView::Wizard));
    a.update(Message::WizardCreate);
    assert!(matches!(a.view, AppView::Workspace));
    assert!(a.snapshot.is_some());
}

#[test]
fn welcome_screen_builds_after_mode_is_chosen() {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Guided,
    ));
    // After choosing, App::view() delegates to the Welcome screen
    let _ = a.view();
}
