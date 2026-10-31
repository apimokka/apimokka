//! Task 027 (MK-023 slice 2) reducer tests for `Tab`/`Shift+Tab` and the
//! cursor movement they and the arrows drive. `regions/tests.rs` covers
//! `Region`'s own table-lookup methods in isolation; these exercise the
//! `App::update` integration — the actual acceptance surface.
//!
//! The four edge cases task 027 §6 requires, for every region this slice
//! creates:
//!
//! 1. **clamp on shortening** — does not apply to either region here.
//!    `Region::len` is a fixed constant for both `ModePicker` (2) and
//!    `TabBar` (3); neither list can shorten at runtime, so there is no
//!    "cursor left out of range" state to reach. The clamping arithmetic in
//!    `App::move_region_cursor` exists for a future region whose `len` can
//!    change (the sidebar tree, the trace list — slices 4 and 5), not
//!    exercised by either concrete region this slice adds.
//! 2. **empty region has no cursor, `Enter` is a no-op** — same reasoning:
//!    neither region is ever empty, so this state cannot be reached through
//!    the public `Message` interface. The `len == 0` guards in
//!    `enter_region`/`move_region_cursor`/`focus_adjacent_region` are
//!    defensive for a future region, not tested here against a concrete
//!    case — a test manufacturing it would have to reach into private state
//!    no message can produce, which would not discriminate anything the
//!    *design* requires.
//! 3. **entering a region sets the cursor per §3.1** — tested below,
//!    for both regions.
//! 4. **whether the cursor can leave the visible scroll** — it cannot: the
//!    tab bar's three tabs and the mode picker's two cards are both always
//!    fully on screen, with no scrollable container around either. Stated
//!    plainly rather than claimed to scroll, per task 026's precedent for
//!    the same kind of non-applicable case.

use super::*;
use crate::message::Message;
use crate::regions::Region;
use crate::selection::WorkspaceTab;
use apimokka_model::AudienceMode;

fn expert_in_workspace() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    a.update(Message::OpenWorkspace("test".into()));
    assert_eq!(a.view, AppView::Workspace, "test setup sanity");
    a
}

// ── Tab / Shift+Tab: cycle the declared order and wrap (task 027 §6) ──────
//
// Both screens in this slice declare exactly one region, so "cycling"
// degenerates to "stays put, repeatably" — the wrap arithmetic is exercised
// (a one-element list wrapping to itself on every step), even though a
// multi-region screen is needed to see it move. Slice 3 (Welcome, Dashboard)
// is the first screen with more than one.

#[test]
fn tab_focuses_the_single_available_region_and_stays_there() {
    let mut a = App::new().0;
    assert_eq!(a.focused_region, None, "nothing focused at launch");
    a.update(Message::TabPressed);
    assert_eq!(a.focused_region, Some(Region::ModePicker));
    assert_eq!(a.region_cursor, Some(0));
    // Wraps to the same (only) region rather than clearing focus.
    a.update(Message::TabPressed);
    assert_eq!(a.focused_region, Some(Region::ModePicker));
    a.update(Message::ShiftTabPressed);
    assert_eq!(a.focused_region, Some(Region::ModePicker));
}

#[test]
fn tab_focuses_the_tab_bar_inside_a_workspace() {
    let mut a = expert_in_workspace();
    a.update(Message::TabPressed);
    assert_eq!(a.focused_region, Some(Region::TabBar));
    assert_eq!(
        a.region_cursor,
        Some(0),
        "Routes is active and at index 0 — entering starts there, not elsewhere"
    );
}

#[test]
fn tab_and_shift_tab_are_a_noop_on_welcome_and_dashboard() {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    assert_eq!(a.view, AppView::Welcome, "test setup sanity");
    a.update(Message::TabPressed);
    assert_eq!(
        a.focused_region, None,
        "no region declared for Welcome in this slice (slice 3)"
    );
    a.update(Message::ShiftTabPressed);
    assert_eq!(a.focused_region, None);
}

// ── §3.1: entering a region sets the cursor on the active item ───────────

#[test]
fn entering_the_tab_bar_starts_the_cursor_on_whichever_tab_is_active() {
    let mut a = expert_in_workspace();
    a.update(Message::SwitchTab(WorkspaceTab::Settings));
    // Focus is not yet established; Tab must land on the *active* tab
    // (Settings, index 2), not the first (Routes, index 0).
    a.update(Message::TabPressed);
    assert_eq!(a.focused_region, Some(Region::TabBar));
    assert_eq!(
        a.region_cursor,
        Some(2),
        "entering must start on the active tab, not the first (task 027 §3.1)"
    );
}

#[test]
fn entering_the_mode_picker_starts_at_the_first_item_since_nothing_is_active() {
    let mut a = App::new().0;
    a.update(Message::TabPressed);
    assert_eq!(
        a.region_cursor,
        Some(0),
        "no mode is chosen yet, so there is no active item to resume — \
         the cursor starts at the first (task 027 §3.1, last line)"
    );
}

// ── Arrow keys move within the focused region, independent of Tab ────────

#[test]
fn arrow_keys_move_the_tab_bar_cursor_without_switching_tabs() {
    let mut a = expert_in_workspace();
    a.update(Message::TabPressed);
    assert_eq!(a.region_cursor, Some(0));
    a.update(Message::ArrowDown);
    assert_eq!(a.region_cursor, Some(1));
    assert_eq!(
        a.tab,
        WorkspaceTab::Routes,
        "moving the cursor must not activate anything — only Enter does (task 027 §3.2)"
    );
    a.update(Message::ArrowDown);
    assert_eq!(a.region_cursor, Some(2), "saturates, three tabs");
    a.update(Message::ArrowDown);
    assert_eq!(a.region_cursor, Some(2), "must not wrap past the last tab");
}

#[test]
fn enter_switches_the_tab_the_cursor_is_on() {
    let mut a = expert_in_workspace();
    a.update(Message::TabPressed);
    a.update(Message::ArrowDown);
    a.update(Message::ArrowDown);
    assert_eq!(
        a.region_cursor,
        Some(2),
        "test setup sanity: cursor on Settings"
    );
    a.update(Message::EnterPressed);
    assert_eq!(a.tab, WorkspaceTab::Settings);
}

// ── The open palette also blocks Tab/arrow region navigation ──────────────
// (review finding F1: `blocks_region_navigation` named the palette in the
// submission's own description but omitted it from the code — `ArrowUp`/
// `ArrowDown` already check `command_palette.open` in their own `if` arm, so
// the bug was `Tab` reaching through to the tab bar underneath an open
// palette, unseen, which this test pins against.)

#[test]
fn tab_does_not_reach_the_tab_bar_while_the_palette_is_open() {
    let mut a = expert_in_workspace();
    a.update(Message::ToggleCommandPalette);
    assert!(a.command_palette.open, "test setup sanity");
    a.update(Message::TabPressed);
    assert_eq!(
        a.focused_region, None,
        "Tab must not focus the tab bar underneath the open palette"
    );
}

// ── A dialog on top of the screen blocks Tab/arrow region navigation ─────
// (not named in task 027's scope, but a correctness gap the reducer's own
// existing dialog precedence would otherwise leave open — see
// `App::blocks_region_navigation`).

#[test]
fn tab_does_not_reach_the_tab_bar_while_the_confirm_dialog_is_open() {
    let mut a = expert_in_workspace();
    a.update(Message::ConfirmRequest(
        crate::message::ConfirmAction::LeaveWorkspace,
    ));
    assert!(a.confirm_dialog.is_some(), "test setup sanity");
    a.update(Message::TabPressed);
    assert_eq!(
        a.focused_region, None,
        "Tab must not focus the tab bar underneath an open dialog"
    );
}

#[test]
fn arrow_keys_do_not_reach_the_tab_bar_while_test_rule_is_open() {
    let mut a = expert_in_workspace();
    a.update(Message::TestRuleOpen);
    assert!(a.test_rule.open, "test setup sanity");
    a.update(Message::ArrowDown);
    assert_eq!(
        a.focused_region, None,
        "arrows must not focus the tab bar underneath the open dialog"
    );
}

// ── Tab is never captured (the design's own key-budget table), but the
// subscription's gate must still apply uniformly — same check as task 026's
// `a_captured_key_is_dropped_except_escape`, extended to this slice's key.

#[test]
fn tab_is_gated_by_capture_status_like_every_other_non_escape_key() {
    use iced::keyboard::{Key, Modifiers, key::Named};

    let tab_event = |shift: bool| {
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: Key::Named(Named::Tab),
            modified_key: Key::Named(Named::Tab),
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers: if shift {
                Modifiers::SHIFT
            } else {
                Modifiers::empty()
            },
            text: None,
            repeat: false,
        })
    };

    assert!(
        super::global_key_message(&tab_event(false), iced::event::Status::Captured).is_none(),
        "a captured Tab must not reach the application (it never is in \
         practice — text_input does not capture it — but the gate must not \
         special-case it either)"
    );
    assert!(
        matches!(
            super::global_key_message(&tab_event(false), iced::event::Status::Ignored),
            Some(Message::TabPressed)
        ),
        "plain Tab"
    );
    assert!(
        matches!(
            super::global_key_message(&tab_event(true), iced::event::Status::Ignored),
            Some(Message::ShiftTabPressed)
        ),
        "Shift+Tab"
    );
}
