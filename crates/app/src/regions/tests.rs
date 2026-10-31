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
fn order_for_is_the_tab_bar_inside_a_workspace() {
    let a = expert_in_workspace();
    assert_eq!(Region::order_for(&a), &[Region::TabBar]);
}

#[test]
fn order_for_is_empty_on_welcome_and_dashboard() {
    let mut a = fresh();
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    assert_eq!(a.view, AppView::Welcome, "test setup sanity");
    assert_eq!(Region::order_for(&a), &[] as &[Region]);

    a.update(Message::GoDashboard);
    assert_eq!(a.view, AppView::Dashboard, "test setup sanity");
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
    assert_eq!(
        Region::ModePicker.len(),
        crate::screens::mode_picker::OPTIONS.len()
    );
    assert_eq!(Region::ModePicker.len(), 2);
}

#[test]
fn tab_bar_has_three_items_matching_its_order_table() {
    assert_eq!(Region::TabBar.len(), TAB_BAR_ORDER.len());
    assert_eq!(Region::TabBar.len(), 3);
}

#[test]
fn mode_picker_activation_message_matches_its_options_table() {
    for (i, &(_, _, mode)) in crate::screens::mode_picker::OPTIONS.iter().enumerate() {
        assert!(matches!(
            Region::ModePicker.activation_message(i),
            Some(Message::ChooseAudienceMode(m)) if m == mode
        ));
    }
    assert!(Region::ModePicker.activation_message(99).is_none());
}

#[test]
fn tab_bar_activation_message_matches_its_order_table() {
    for (i, &(tab, _)) in TAB_BAR_ORDER.iter().enumerate() {
        assert!(matches!(
            Region::TabBar.activation_message(i),
            Some(Message::SwitchTab(t)) if t == tab
        ));
    }
    assert!(Region::TabBar.activation_message(99).is_none());
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
