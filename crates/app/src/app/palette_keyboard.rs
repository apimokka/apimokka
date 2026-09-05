//! Task 014 (MK-033 keyboard operability + MK-023 first-screen gap) reducer
//! tests. Pure `update` state transitions, per this task's own verification
//! guidance — no iced runtime is involved.

use super::*;
use crate::message::Message;

fn expert_in_workspace() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::OpenWorkspace("test".into()));
    assert_eq!(a.view, AppView::Workspace, "test setup sanity");
    a
}

// ── §1/§4: opening resets query and selection ───────────────────────────

#[test]
fn toggle_palette_opens_and_resets_query_and_selection() {
    let mut a = expert_in_workspace();
    a.command_palette.query = "stale query".into();
    a.command_palette.selected = Some(3);
    a.update(Message::ToggleCommandPalette);
    assert!(a.command_palette.open);
    assert_eq!(a.command_palette.query, "");
    assert_eq!(a.command_palette.selected, None);
}

#[test]
fn toggle_palette_closes_and_clears_selection_too() {
    let mut a = expert_in_workspace();
    a.update(Message::ToggleCommandPalette); // open
    a.command_palette.selected = Some(1);
    a.update(Message::ToggleCommandPalette); // close
    assert!(!a.command_palette.open);
    assert_eq!(a.command_palette.selected, None);
}

// ── D-4: the palette cannot be opened outside a workspace ───────────────

#[test]
fn toggle_palette_is_a_noop_before_an_audience_mode_is_chosen() {
    let mut a = App::new().0;
    assert!(a.audience_mode.is_none());
    a.update(Message::ToggleCommandPalette);
    assert!(
        !a.command_palette.open,
        "D-4: toggling before a mode is chosen must not open a palette nothing renders"
    );
}

#[test]
fn toggle_palette_is_a_noop_outside_the_workspace_view() {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    assert_eq!(a.view, AppView::Welcome, "test setup sanity");
    a.update(Message::ToggleCommandPalette);
    assert!(
        !a.command_palette.open,
        "D-4: screens::command_palette::view only renders inside AppView::Workspace"
    );
}

// ── §3: arrow-key navigation ─────────────────────────────────────────────

#[test]
fn arrow_down_from_no_selection_selects_the_first_row() {
    let mut a = expert_in_workspace();
    a.update(Message::ToggleCommandPalette);
    a.update(Message::ArrowDown);
    assert_eq!(a.command_palette.selected, Some(0));
}

#[test]
fn arrow_up_from_no_selection_also_selects_the_first_row() {
    let mut a = expert_in_workspace();
    a.update(Message::ToggleCommandPalette);
    a.update(Message::ArrowUp);
    assert_eq!(a.command_palette.selected, Some(0));
}

#[test]
fn arrow_keys_saturate_at_both_ends_rather_than_wrapping() {
    let mut a = expert_in_workspace();
    a.update(Message::ToggleCommandPalette);
    let len = crate::palette_commands::filtered_indices(&a, "").len();
    assert!(len > 1, "test needs more than one command to be meaningful");

    a.command_palette.selected = Some(0);
    a.update(Message::ArrowUp);
    assert_eq!(a.command_palette.selected, Some(0), "must not go below 0");

    a.command_palette.selected = Some(len - 1);
    a.update(Message::ArrowDown);
    assert_eq!(
        a.command_palette.selected,
        Some(len - 1),
        "must not go past the last row"
    );
}

#[test]
fn arrow_keys_are_a_noop_when_the_palette_is_closed() {
    let mut a = expert_in_workspace();
    assert!(!a.command_palette.open);
    a.update(Message::ArrowDown);
    assert_eq!(a.command_palette.selected, None);
}

// ── §3 edge case: typing a query that shortens the list clamps ──────────

#[test]
fn typing_a_query_that_shortens_the_list_clamps_the_selection() {
    let mut a = expert_in_workspace();
    a.update(Message::ToggleCommandPalette);
    a.update(Message::PaletteQuery("go".into()));
    let go_len = crate::palette_commands::filtered_indices(&a, "go").len();
    assert_eq!(go_len, 3, "\"go\" matches Go to Routes/Trace/Settings");
    a.command_palette.selected = Some(go_len - 1); // last row, "Go to Settings"

    a.update(Message::PaletteQuery("trace".into()));
    let trace_len = crate::palette_commands::filtered_indices(&a, "trace").len();
    assert_eq!(
        trace_len, 2,
        "\"trace\" matches \"Toggle live trace strip\" and \"Go to Trace\""
    );
    assert_eq!(
        a.command_palette.selected,
        Some(trace_len - 1),
        "selection must clamp into the shrunk list, never point past its end"
    );
}

// ── §3 edge case: an empty filtered list has no selection, Enter is a no-op ──

#[test]
fn a_query_matching_nothing_clears_the_selection() {
    let mut a = expert_in_workspace();
    a.update(Message::ToggleCommandPalette);
    a.update(Message::ArrowDown);
    assert_eq!(a.command_palette.selected, Some(0));

    a.update(Message::PaletteQuery("xyz-no-such-command".into()));
    assert_eq!(
        a.command_palette.selected, None,
        "an empty filtered list must have no selection"
    );
}

#[test]
fn enter_is_a_noop_when_the_filtered_list_is_empty() {
    let mut a = expert_in_workspace();
    a.update(Message::ToggleCommandPalette);
    a.update(Message::PaletteQuery("xyz-no-such-command".into()));
    let tab_before = a.tab;
    let theme_before = a.theme_choice;
    a.update(Message::EnterPressed);
    assert_eq!(a.tab, tab_before, "Enter with no selection must do nothing");
    assert_eq!(a.theme_choice, theme_before);
    assert!(
        a.command_palette.open,
        "and must not close the palette either"
    );
}

// ── §2: Enter executes the row Enter is actually pointed at ─────────────

#[test]
fn enter_executes_the_selected_rows_message_under_a_filter() {
    let mut a = expert_in_workspace();
    a.update(Message::ToggleCommandPalette);
    a.update(Message::PaletteQuery("theme".into()));
    assert_eq!(
        crate::palette_commands::filtered_indices(&a, "theme").len(),
        1,
        "\"theme\" matches exactly \"Toggle theme\""
    );
    a.update(Message::ArrowDown); // selects the sole filtered row
    let theme_before = a.theme_choice;
    a.update(Message::EnterPressed);
    assert_ne!(
        a.theme_choice, theme_before,
        "Enter should have executed Message::ToggleTheme for the filtered row, \
         not some other row from the unfiltered table at the same index"
    );
}

// ── §4: the mode picker uses the same selected-row-plus-Enter idiom ──────

#[test]
fn mode_picker_arrow_keys_move_between_the_two_options() {
    let mut a = App::new().0;
    assert!(a.audience_mode.is_none());
    a.update(Message::ArrowDown);
    assert_eq!(a.mode_picker_selected, Some(0));
    a.update(Message::ArrowDown);
    assert_eq!(
        a.mode_picker_selected,
        Some(1),
        "two options, saturates at 1"
    );
    a.update(Message::ArrowDown);
    assert_eq!(a.mode_picker_selected, Some(1), "must not wrap back to 0");
    a.update(Message::ArrowUp);
    assert_eq!(a.mode_picker_selected, Some(0));
}

#[test]
fn mode_picker_enter_confirms_the_visibly_selected_card() {
    let mut a = App::new().0;
    a.update(Message::ArrowDown);
    a.update(Message::ArrowDown);
    assert_eq!(a.mode_picker_selected, Some(1));
    a.update(Message::EnterPressed);
    assert_eq!(
        a.audience_mode,
        Some(crate::screens::mode_picker::OPTIONS[1].2),
        "Enter must confirm exactly the option at the selected index"
    );
}

#[test]
fn mode_picker_enter_is_a_noop_with_no_selection() {
    let mut a = App::new().0;
    assert_eq!(a.mode_picker_selected, None);
    a.update(Message::EnterPressed);
    assert!(
        a.audience_mode.is_none(),
        "no selection, nothing to confirm"
    );
}

#[test]
fn mode_picker_arrow_keys_are_a_noop_once_a_mode_is_chosen() {
    let mut a = expert_in_workspace();
    a.mode_picker_selected = None;
    a.update(Message::ArrowDown);
    assert_eq!(
        a.mode_picker_selected, None,
        "arrow keys must not touch the mode picker once it is no longer shown"
    );
}
