//! Task 027 (MK-023 slice 2) — pure tests of `Region`'s own methods: the
//! table-lookup layer, independent of the reducer logic that drives it (see
//! `app/region_cursor.rs` for the `Tab`/arrow/`Enter` integration tests).

use super::*;
use crate::app::{App, AppView};
use crate::message::Message;
use apimokka_model::AudienceMode;

fn fresh() -> App {
    App::new().0
}

fn expert_in_workspace() -> App {
    let mut a = fresh();
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    a.update(Message::OpenWorkspace("test".into()));
    assert_eq!(a.view, AppView::Workspace, "test setup sanity");
    a
}

// ── order_for: mirrors App::view's own precedence ─────────────────────────

#[test]
fn order_for_is_the_mode_picker_before_a_mode_is_chosen() {
    let a = fresh();
    assert_eq!(Region::order_for(&a), &[Region::ModePicker]);
}

#[test]
fn order_for_is_the_tab_bar_and_sidebar_on_the_routes_tab() {
    // Task 029: Routes is the tab OpenWorkspace lands on, and the sidebar
    // only renders there — mirrors `screens::routes::view` only being
    // reached on that tab.
    let a = expert_in_workspace();
    assert_eq!(
        a.tab,
        crate::selection::WorkspaceTab::Routes,
        "test setup sanity"
    );
    assert_eq!(Region::order_for(&a), &[Region::TabBar, Region::Sidebar]);
}

#[test]
fn order_for_is_only_the_tab_bar_on_trace_and_settings() {
    use crate::selection::WorkspaceTab;

    let mut a = expert_in_workspace();
    a.update(Message::SwitchTab(WorkspaceTab::Trace));
    assert_eq!(Region::order_for(&a), &[Region::TabBar]);
    a.update(Message::SwitchTab(WorkspaceTab::Settings));
    assert_eq!(Region::order_for(&a), &[Region::TabBar]);
}

#[test]
fn order_for_is_one_region_on_welcome_and_three_in_order_on_dashboard() {
    let mut a = fresh();
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    assert_eq!(a.view, AppView::Welcome, "test setup sanity");
    assert_eq!(Region::order_for(&a), &[Region::WelcomeActions]);

    a.update(Message::GoDashboard);
    assert_eq!(a.view, AppView::Dashboard, "test setup sanity");
    assert_eq!(
        Region::order_for(&a),
        &[
            Region::DashboardCreate,
            Region::DashboardSearch,
            Region::DashboardWorkspaces,
        ],
        "task 028 §2: visual order is Tab order"
    );
}

#[test]
fn order_for_is_empty_on_the_wizard() {
    let mut a = fresh();
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    a.update(Message::GoWizard);
    assert_eq!(a.view, AppView::Wizard, "test setup sanity");
    assert_eq!(Region::order_for(&a), &[] as &[Region]);
}

#[test]
fn order_for_prefers_the_mode_picker_even_if_view_somehow_says_workspace() {
    // Mirrors `App::view`'s own short-circuit exactly (`audience_mode.is_none()`
    // is checked before `self.view` there), so the two can never disagree
    // about which screen is actually showing.
    let mut a = fresh();
    a.view = AppView::Workspace;
    assert_eq!(Region::order_for(&a), &[Region::ModePicker]);
}

// ── len / activation_message: fixed tables ────────────────────────────────

#[test]
fn mode_picker_has_two_items_matching_its_options_table() {
    let a = fresh();
    assert_eq!(
        Region::ModePicker.len(&a),
        crate::screens::mode_picker::OPTIONS.len()
    );
    assert_eq!(Region::ModePicker.len(&a), 2);
}

#[test]
fn tab_bar_has_three_items_matching_its_order_table() {
    let a = expert_in_workspace();
    assert_eq!(Region::TabBar.len(&a), TAB_BAR_ORDER.len());
    assert_eq!(Region::TabBar.len(&a), 3);
}

#[test]
fn welcome_actions_has_two_items_matching_its_table() {
    let a = fresh();
    assert_eq!(Region::WelcomeActions.len(&a), WELCOME_ACTIONS.len());
    assert_eq!(Region::WelcomeActions.len(&a), 2);
}

#[test]
fn mode_picker_activation_message_matches_its_options_table() {
    for (i, &(_, _, mode)) in crate::screens::mode_picker::OPTIONS.iter().enumerate() {
        assert!(matches!(
            Region::ModePicker.activation_message(&Cursor::Index(i)),
            Some(Message::ChooseAudienceMode(m)) if m == mode
        ));
    }
    assert!(
        Region::ModePicker
            .activation_message(&Cursor::Index(99))
            .is_none()
    );
}

#[test]
fn tab_bar_activation_message_matches_its_order_table() {
    for (i, &(tab, _)) in TAB_BAR_ORDER.iter().enumerate() {
        assert!(matches!(
            Region::TabBar.activation_message(&Cursor::Index(i)),
            Some(Message::SwitchTab(t)) if t == tab
        ));
    }
    assert!(
        Region::TabBar
            .activation_message(&Cursor::Index(99))
            .is_none()
    );
}

#[test]
fn welcome_actions_activation_message_matches_its_table() {
    assert!(matches!(
        Region::WelcomeActions.activation_message(&Cursor::Index(0)),
        Some(Message::GoDashboard)
    ));
    assert!(matches!(
        Region::WelcomeActions.activation_message(&Cursor::Index(1)),
        Some(Message::GoWizard)
    ));
    assert!(
        Region::WelcomeActions
            .activation_message(&Cursor::Index(99))
            .is_none()
    );
}

// ── Task 028: the identity-cursored Dashboard workspace list ─────────────

#[test]
fn dashboard_create_is_one_item_that_opens_the_wizard() {
    let a = fresh();
    assert_eq!(Region::DashboardCreate.len(&a), 1);
    assert!(matches!(
        Region::DashboardCreate.activation_message(&Cursor::Index(0)),
        Some(Message::GoWizard)
    ));
}

#[test]
fn dashboard_search_is_one_item_whose_enter_is_a_noop() {
    let a = fresh();
    assert_eq!(Region::DashboardSearch.len(&a), 1);
    assert!(
        Region::DashboardSearch
            .activation_message(&Cursor::Index(0))
            .is_none(),
        "task 028 §2: typing is the interaction, not Enter"
    );
}

#[test]
fn dashboard_workspaces_uses_identity_cursor_and_the_others_do_not() {
    assert!(Region::DashboardWorkspaces.uses_identity_cursor());
    assert!(!Region::DashboardCreate.uses_identity_cursor());
    assert!(!Region::DashboardSearch.uses_identity_cursor());
    assert!(!Region::WelcomeActions.uses_identity_cursor());
    assert!(!Region::ModePicker.uses_identity_cursor());
    assert!(!Region::TabBar.uses_identity_cursor());
}

#[test]
fn dashboard_workspaces_len_matches_the_live_filter() {
    let mut a = fresh();
    let unfiltered = dashboard_filtered_workspaces(&a).len();
    assert_eq!(Region::DashboardWorkspaces.len(&a), unfiltered);
    assert!(unfiltered > 0, "test setup sanity: mock data exists");

    a.dash_search = "does-not-match-anything".into();
    assert_eq!(Region::DashboardWorkspaces.len(&a), 0);
}

#[test]
fn dashboard_workspaces_identity_at_names_the_workspace_at_that_position() {
    let a = fresh();
    let names: Vec<String> = dashboard_filtered_workspaces(&a)
        .into_iter()
        .map(|ws| ws.name)
        .collect();
    assert_eq!(
        Region::DashboardWorkspaces.identity_at(&a, 0),
        Some(Cursor::WorkspaceName(names[0].clone()))
    );
    assert_eq!(
        Region::DashboardWorkspaces.identity_at(&a, names.len()),
        None,
        "out of range"
    );
}

#[test]
fn identity_at_is_none_for_every_index_cursored_region() {
    let a = fresh();
    assert_eq!(Region::ModePicker.identity_at(&a, 0), None);
    assert_eq!(Region::TabBar.identity_at(&a, 0), None);
    assert_eq!(Region::WelcomeActions.identity_at(&a, 0), None);
    assert_eq!(Region::DashboardCreate.identity_at(&a, 0), None);
    assert_eq!(Region::DashboardSearch.identity_at(&a, 0), None);
}

#[test]
fn dashboard_workspaces_activation_message_opens_the_named_workspace() {
    let a = fresh();
    let name = dashboard_filtered_workspaces(&a)[0].name.clone();
    assert!(matches!(
        Region::DashboardWorkspaces.activation_message(&Cursor::WorkspaceName(name.clone())),
        Some(Message::OpenWorkspace(n)) if n == name
    ));
    // `activation_message` trusts the `Cursor` it is handed — it has no `App`
    // to check a name against, by design (`len`/`identity_at` take `&App`
    // precisely so the *cursor* is kept valid as the filter changes;
    // `activation_message` only has to translate an already-valid cursor
    // into its `Message`). The "filtered-away identity" edge case is a
    // property of `App::move_region_cursor`/`reclamp_dashboard_workspace_cursor`
    // keeping the cursor honest, exercised at that level in
    // `app/region_cursor.rs`, not here.
}

// ── active_index: task 027 §3.1 ────────────────────────────────────────────

#[test]
fn mode_picker_has_no_active_index_before_a_choice_is_made() {
    let a = fresh();
    assert_eq!(Region::ModePicker.active_index(&a), None);
}

#[test]
fn tab_bar_active_index_tracks_the_currently_displayed_tab() {
    use crate::selection::WorkspaceTab;

    let mut a = expert_in_workspace();
    assert_eq!(a.tab, WorkspaceTab::Routes, "test setup sanity");
    assert_eq!(Region::TabBar.active_index(&a), Some(0));

    a.update(Message::SwitchTab(WorkspaceTab::Trace));
    assert_eq!(Region::TabBar.active_index(&a), Some(1));

    a.update(Message::SwitchTab(WorkspaceTab::Settings));
    assert_eq!(Region::TabBar.active_index(&a), Some(2));
}
