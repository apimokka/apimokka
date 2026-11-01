//! Boundary decision: single-responsibility — reducer tests for the region/
//! cursor keyboard model (`Tab`/`Shift+Tab`, the arrows, `Enter`) across
//! every slice that has extended it (027, 028, 029, 030). `regions/tests.rs`
//! covers `Region`'s own table-lookup methods in isolation; this file
//! exercises the `App::update` integration — the actual acceptance surface —
//! and grows with each slice because that surface is one coherent model, not
//! one file per slice.
//!
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
//!
//! Task 028 (MK-023 slice 3) makes all four live, for the first time, against
//! `DashboardWorkspaces` — the model's first region whose list can shorten,
//! empty, and scroll. Its tests sit in their own section below, after the
//! Tab/Shift+Tab tests this slice also extends to Welcome and Dashboard.

use super::*;
use crate::message::Message;
use crate::regions::{Cursor, Region};
use crate::selection::WorkspaceTab;
use apimokka_model::AudienceMode;

fn expert_in_workspace() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    a.update(Message::OpenWorkspace("test".into()));
    assert_eq!(a.view, AppView::Workspace, "test setup sanity");
    a
}

fn expert_at_dashboard() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    a.update(Message::GoDashboard);
    assert_eq!(a.view, AppView::Dashboard, "test setup sanity");
    a
}

/// Tabs from Dashboard's first region all the way to `DashboardWorkspaces`.
fn tab_into_dashboard_workspaces(a: &mut App) {
    a.update(Message::TabPressed); // DashboardCreate
    a.update(Message::TabPressed); // DashboardSearch
    a.update(Message::TabPressed); // DashboardWorkspaces
    assert_eq!(
        a.focused_region,
        Some(Region::DashboardWorkspaces),
        "test setup sanity"
    );
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
    assert_eq!(a.region_cursor, Some(Cursor::Index(0)));
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
        Some(Cursor::Index(0)),
        "Routes is active and at index 0 — entering starts there, not elsewhere"
    );
}

#[test]
fn tab_focuses_welcomes_single_region_and_stays_there() {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    assert_eq!(a.view, AppView::Welcome, "test setup sanity");
    a.update(Message::TabPressed);
    assert_eq!(a.focused_region, Some(Region::WelcomeActions));
    a.update(Message::TabPressed);
    assert_eq!(
        a.focused_region,
        Some(Region::WelcomeActions),
        "wraps to the same (only) region rather than clearing focus"
    );
    a.update(Message::ShiftTabPressed);
    assert_eq!(a.focused_region, Some(Region::WelcomeActions));
}

#[test]
fn tab_cycles_dashboards_three_regions_in_visual_order() {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    a.update(Message::GoDashboard);
    assert_eq!(a.view, AppView::Dashboard, "test setup sanity");

    a.update(Message::TabPressed);
    assert_eq!(a.focused_region, Some(Region::DashboardCreate));
    a.update(Message::TabPressed);
    assert_eq!(a.focused_region, Some(Region::DashboardSearch));
    a.update(Message::TabPressed);
    assert_eq!(a.focused_region, Some(Region::DashboardWorkspaces));
    a.update(Message::TabPressed);
    assert_eq!(
        a.focused_region,
        Some(Region::DashboardCreate),
        "Tab wraps back to the first region (task 028 §2)"
    );
    a.update(Message::ShiftTabPressed);
    assert_eq!(
        a.focused_region,
        Some(Region::DashboardWorkspaces),
        "Shift+Tab wraps the other way"
    );
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
        Some(Cursor::Index(2)),
        "entering must start on the active tab, not the first (task 027 §3.1)"
    );
}

#[test]
fn entering_the_mode_picker_starts_at_the_first_item_since_nothing_is_active() {
    let mut a = App::new().0;
    a.update(Message::TabPressed);
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Index(0)),
        "no mode is chosen yet, so there is no active item to resume — \
         the cursor starts at the first (task 027 §3.1, last line)"
    );
}

// ── Arrow keys move within the focused region, independent of Tab ────────

#[test]
fn arrow_keys_move_the_tab_bar_cursor_without_switching_tabs() {
    let mut a = expert_in_workspace();
    a.update(Message::TabPressed);
    assert_eq!(a.region_cursor, Some(Cursor::Index(0)));
    a.update(Message::ArrowDown);
    assert_eq!(a.region_cursor, Some(Cursor::Index(1)));
    assert_eq!(
        a.tab,
        WorkspaceTab::Routes,
        "moving the cursor must not activate anything — only Enter does (task 027 §3.2)"
    );
    a.update(Message::ArrowDown);
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Index(2)),
        "saturates, three tabs"
    );
    a.update(Message::ArrowDown);
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Index(2)),
        "must not wrap past the last tab"
    );
}

#[test]
fn enter_switches_the_tab_the_cursor_is_on() {
    let mut a = expert_in_workspace();
    a.update(Message::TabPressed);
    a.update(Message::ArrowDown);
    a.update(Message::ArrowDown);
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Index(2)),
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

// ── Task 028 (MK-023 slice 3): Welcome and Dashboard ──────────────────────
//
// §5's four edge cases, tested live against `DashboardWorkspaces` — the
// model's first identity-cursored region and the first region any of them
// can actually occur against (slices 1–2's regions are both fixed-length,
// recorded as N/A in this file's module doc above).

#[test]
fn welcome_enter_on_open_workspace_reaches_dashboard() {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    assert_eq!(a.view, AppView::Welcome, "test setup sanity");
    a.update(Message::TabPressed);
    assert_eq!(a.focused_region, Some(Region::WelcomeActions));
    assert_eq!(a.region_cursor, Some(Cursor::Index(0)));
    a.update(Message::EnterPressed);
    assert_eq!(a.view, AppView::Dashboard);
}

#[test]
fn welcome_enter_on_create_new_workspace_reaches_the_wizard() {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    a.update(Message::TabPressed);
    a.update(Message::ArrowDown);
    assert_eq!(a.region_cursor, Some(Cursor::Index(1)));
    a.update(Message::EnterPressed);
    assert_eq!(a.view, AppView::Wizard);
}

#[test]
fn dashboard_enter_on_create_reaches_the_wizard() {
    let mut a = expert_at_dashboard();
    a.update(Message::TabPressed);
    assert_eq!(a.focused_region, Some(Region::DashboardCreate));
    a.update(Message::EnterPressed);
    assert_eq!(a.view, AppView::Wizard);
}

// ── Edge case 3: entering DashboardWorkspaces resets to the first item ────
// (its `active_index` is always `None` — nothing is "the open workspace"
// while browsing, so there is no "active" item to resume per task 027 §3.1's
// own fallback clause.)

#[test]
fn entering_dashboard_workspaces_starts_the_cursor_on_the_first_item() {
    let mut a = expert_at_dashboard();
    tab_into_dashboard_workspaces(&mut a);
    assert_eq!(
        a.region_cursor,
        Some(Cursor::WorkspaceName("shop-api-mock".into())),
        "test setup's mock data puts shop-api-mock first"
    );
}

// ── Edge case 1: clamp on shortening ───────────────────────────────────────
// `Region::DashboardWorkspaces` has no "active" item to resume (above), so
// every *re-entry* already lands on a valid first item by construction —
// the one state that can actually drift is the cursor's own position
// *while the Workspaces region already holds it*, exercised directly here
// against `App::reclamp_dashboard_workspace_cursor`/`move_region_cursor`'s
// identity branch rather than through a specific pointer-free keystroke
// sequence (the filter can only change while `DashboardSearch` has native
// focus, which this slice's own `enter_region` always gives a fresh cursor
// on return — the defect this guards is a stale identity reaching `view`
// or `Enter`, not a specific route to it).

#[test]
fn typing_a_filter_that_removes_the_cursors_workspace_falls_back_to_a_surviving_item() {
    let mut a = expert_at_dashboard();
    tab_into_dashboard_workspaces(&mut a);
    a.update(Message::ArrowDown);
    assert_eq!(
        a.region_cursor,
        Some(Cursor::WorkspaceName("billing-sandbox".into())),
        "test setup sanity"
    );
    a.update(Message::DashSearch("qa".into()));
    assert_eq!(
        a.region_cursor,
        Some(Cursor::WorkspaceName("qa-edge-cases".into())),
        "billing-sandbox no longer matches \"qa\" — the cursor must fall back \
         to a surviving item immediately, not wait for the next arrow press"
    );
}

#[test]
fn move_region_cursor_self_heals_when_the_stored_identity_is_no_longer_in_the_list() {
    // Exercises `move_region_cursor`'s identity branch directly: a stored
    // name the current filtered list no longer contains must not panic or
    // stick — pressing an arrow key lands on a real item.
    let mut a = expert_at_dashboard();
    tab_into_dashboard_workspaces(&mut a);
    a.region_cursor = Some(Cursor::WorkspaceName("no-such-workspace".into()));
    a.update(Message::ArrowDown);
    assert_eq!(
        a.region_cursor,
        Some(Cursor::WorkspaceName("shop-api-mock".into())),
        "an unrecognised stored identity must resolve to a real item, not panic or stay stale"
    );
}

// ── Edge case 2: an empty region has no cursor, and Enter is a no-op ──────

#[test]
fn a_filter_matching_nothing_leaves_no_cursor_and_enter_does_not_open_anything() {
    let mut a = expert_at_dashboard();
    tab_into_dashboard_workspaces(&mut a);
    a.update(Message::DashSearch("no-such-workspace-matches-this".into()));
    assert_eq!(
        a.region_cursor, None,
        "no workspace matches — no cursor, per task 028 §5 edge case 2"
    );
    a.update(Message::EnterPressed);
    assert_eq!(
        a.view,
        AppView::Dashboard,
        "Enter on an empty region must be a no-op, not a panic or a stale open"
    );
}

#[test]
fn arrow_keys_are_a_noop_on_an_empty_filtered_list() {
    let mut a = expert_at_dashboard();
    tab_into_dashboard_workspaces(&mut a);
    a.update(Message::DashSearch("no-such-workspace-matches-this".into()));
    assert_eq!(a.region_cursor, None, "test setup sanity");
    a.update(Message::ArrowDown);
    assert_eq!(
        a.region_cursor, None,
        "must not panic or synthesize a cursor"
    );
    a.update(Message::ArrowUp);
    assert_eq!(a.region_cursor, None);
}

// ── Enter opens exactly the cursor's (post-filter) workspace ──────────────

#[test]
fn enter_opens_the_workspace_the_cursor_is_on_after_filtering() {
    let mut a = expert_at_dashboard();
    tab_into_dashboard_workspaces(&mut a);
    a.update(Message::DashSearch("sandbox".into()));
    assert_eq!(
        a.region_cursor,
        Some(Cursor::WorkspaceName("billing-sandbox".into())),
        "test setup sanity: only billing-sandbox matches"
    );
    a.update(Message::EnterPressed);
    assert_eq!(a.view, AppView::Workspace);
}

// ── §3.1: leaving DashboardSearch unfocuses it ────────────────────────────
// `App::update` is a pure reducer and never calls `iced`'s focus operations
// itself (`update_and_dispatch` does, verified live in this slice's
// keyboard-only capture) — what the reducer alone can promise, and what
// this pins, is that `focused_region` itself leaves `DashboardSearch`
// exactly when the design says it should, which is the signal
// `update_and_dispatch` reads to decide to unfocus.

#[test]
fn tabbing_past_search_leaves_it_as_the_focused_region() {
    let mut a = expert_at_dashboard();
    a.update(Message::TabPressed); // DashboardCreate
    a.update(Message::TabPressed); // DashboardSearch
    assert_eq!(a.focused_region, Some(Region::DashboardSearch));
    a.update(Message::TabPressed); // DashboardWorkspaces
    assert_ne!(
        a.focused_region,
        Some(Region::DashboardSearch),
        "Tab must move focus off Search — update_and_dispatch unfocuses it on this transition"
    );
}

// ── Task 029 (MK-023 slice 4): the routes sidebar tree ────────────────────
//
// §7's four edge cases, live again, this time against `Region::Sidebar` —
// the model's second identity-cursored region and the first whose identity
// list has real internal structure (an accordion, two independent toggles)
// rather than a single flat filter.

use crate::regions::SidebarRow;
use apimokka_model::{NodeId, RuleSetId};

fn rule_set_ids(a: &App) -> Vec<RuleSetId> {
    a.snapshot
        .as_ref()
        .expect("test setup sanity")
        .rule_sets
        .iter()
        .map(|rs| rs.id)
        .collect()
}

fn rule_ids_in(a: &App, rs_id: RuleSetId) -> Vec<NodeId> {
    a.snapshot
        .as_ref()
        .expect("test setup sanity")
        .find_rule_set(rs_id)
        .expect("test setup sanity")
        .rules
        .iter()
        .map(|r| r.id)
        .collect()
}

fn tab_into_sidebar(a: &mut App) {
    a.update(Message::TabPressed); // TabBar
    a.update(Message::TabPressed); // Sidebar
    assert_eq!(a.focused_region, Some(Region::Sidebar), "test setup sanity");
}

// ── Tab order and cycling ──────────────────────────────────────────────────

#[test]
fn tab_cycles_tab_bar_and_sidebar_on_routes_and_wraps() {
    let mut a = expert_in_workspace();
    a.update(Message::TabPressed);
    assert_eq!(a.focused_region, Some(Region::TabBar));
    a.update(Message::TabPressed);
    assert_eq!(a.focused_region, Some(Region::Sidebar));
    a.update(Message::TabPressed);
    assert_eq!(
        a.focused_region,
        Some(Region::TabBar),
        "wraps back to the tab bar (task 029 §2's two-region Tab order)"
    );
}

// ── §3.1 / Edge case 3: entering resets to the active item, or the first ──

#[test]
fn entering_the_sidebar_starts_on_the_first_rule_set_header_when_nothing_is_selected() {
    let mut a = expert_in_workspace();
    let rs_ids = rule_set_ids(&a);
    tab_into_sidebar(&mut a);
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::RuleSetHeader(rs_ids[0]))),
        "nothing is selected yet, so the cursor starts at the first row (task 027 §3.1)"
    );
}

#[test]
fn entering_the_sidebar_resumes_on_the_currently_selected_rule() {
    let mut a = expert_in_workspace();
    let rs_ids = rule_set_ids(&a);
    let rule_ids = rule_ids_in(&a, rs_ids[1]);
    a.update(Message::SelectRule(rule_ids[0]));
    assert_eq!(
        a.focused_region, None,
        "selecting via message alone does not move focus"
    );

    a.update(Message::TabPressed); // TabBar
    a.update(Message::TabPressed); // Sidebar
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::Rule(rule_ids[0]))),
        "entering resumes on the selected rule (§3.1's active-item clause), \
         not the first row — the sidebar's `active_index` mirrors the centre \
         panel's own rule>file>script>rule-set priority"
    );
}

// ── ArrowUp/ArrowDown move within the flattened sequence ──────────────────

#[test]
fn arrow_down_moves_from_the_first_rule_set_header_to_the_second() {
    let mut a = expert_in_workspace();
    let rs_ids = rule_set_ids(&a);
    tab_into_sidebar(&mut a);
    a.update(Message::ArrowDown);
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::RuleSetHeader(rs_ids[1]))),
        "both rule sets start collapsed, so their headers are adjacent rows"
    );
}

#[test]
fn arrow_down_saturates_at_the_last_row() {
    let mut a = expert_in_workspace();
    tab_into_sidebar(&mut a);
    // Rows with nothing expanded: two rule-set headers, AddRuleSet,
    // FallbackHeader, MiddlewareHeader — five total.
    for _ in 0..10 {
        a.update(Message::ArrowDown);
    }
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::MiddlewareHeader)),
        "must not wrap past the last row"
    );
}

// ── §4: ArrowRight expands, or moves to the first child when expanded ─────

#[test]
fn arrow_right_on_a_collapsed_rule_set_header_expands_and_selects_it() {
    let mut a = expert_in_workspace();
    let rs_ids = rule_set_ids(&a);
    tab_into_sidebar(&mut a);
    assert_eq!(a.rule_set_open, None, "test setup sanity");
    a.update(Message::ArrowRight);
    assert_eq!(
        a.rule_set_open,
        Some(rs_ids[0]),
        "task 029 §4: ArrowRight on a collapsed header activates it — \
         SelectRuleSet both expands and selects"
    );
    assert_eq!(a.selection.rule_set, Some(rs_ids[0]));
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::RuleSetHeader(rs_ids[0]))),
        "the cursor stays on the header — Right and Enter coincide here, \
         they do not also move the cursor"
    );
}

#[test]
fn arrow_right_on_an_expanded_rule_set_header_moves_to_its_first_rule() {
    let mut a = expert_in_workspace();
    let rs_ids = rule_set_ids(&a);
    let rule_ids = rule_ids_in(&a, rs_ids[0]);
    tab_into_sidebar(&mut a);
    a.update(Message::ArrowRight); // expand
    a.update(Message::ArrowRight); // now expanded: move to first child
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::Rule(rule_ids[0]))),
        "task 029 §4: Right on an already-expanded header moves to its first child"
    );
}

#[test]
fn arrow_right_on_a_collapsed_fallback_header_toggles_it_open() {
    let mut a = expert_in_workspace();
    tab_into_sidebar(&mut a);
    a.update(Message::ArrowDown); // RuleSetHeader(errors)
    a.update(Message::ArrowDown); // AddRuleSet
    a.update(Message::ArrowDown); // FallbackHeader
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::FallbackHeader)),
        "test setup sanity"
    );
    assert!(!a.fallback_section_open, "test setup sanity");
    a.update(Message::ArrowRight);
    assert!(a.fallback_section_open);
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::FallbackHeader)),
        "expanding must not also move the cursor off the header"
    );
}

#[test]
fn arrow_right_is_a_noop_on_a_leaf_row() {
    let mut a = expert_in_workspace();
    let rs_ids = rule_set_ids(&a);
    tab_into_sidebar(&mut a);
    a.update(Message::ArrowRight); // expand rule set 0
    a.update(Message::ArrowRight); // move to first rule
    let cursor_before = a.region_cursor.clone();
    let rule_set_open_before = a.rule_set_open;
    a.update(Message::ArrowRight); // a rule has no children to expand into
    assert_eq!(
        a.region_cursor, cursor_before,
        "a leaf row has nothing to expand"
    );
    assert_eq!(a.rule_set_open, rule_set_open_before);
    let _ = rs_ids;
}

// ── §4: ArrowLeft collapses, or moves to the parent ───────────────────────

#[test]
fn arrow_left_collapses_an_expanded_rule_set_header_without_changing_selection() {
    let mut a = expert_in_workspace();
    let rs_ids = rule_set_ids(&a);
    tab_into_sidebar(&mut a);
    a.update(Message::ArrowRight); // expand + select rule set 0
    assert_eq!(a.rule_set_open, Some(rs_ids[0]), "test setup sanity");
    a.update(Message::ArrowLeft);
    assert_eq!(
        a.rule_set_open, None,
        "task 029 §4: the keyboard-only collapse path — no message closes \
         a rule set on its own, so ArrowLeft sets `rule_set_open` directly"
    );
    assert_eq!(
        a.selection.rule_set,
        Some(rs_ids[0]),
        "collapsing must not change what is selected — only the pointer-\
         equivalent action (another SelectRuleSet) does that"
    );
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::RuleSetHeader(rs_ids[0]))),
        "the cursor stays on the header it just collapsed"
    );
}

#[test]
fn arrow_left_on_a_collapsed_rule_set_header_is_a_noop() {
    let mut a = expert_in_workspace();
    tab_into_sidebar(&mut a);
    assert_eq!(a.rule_set_open, None, "test setup sanity");
    a.update(Message::ArrowLeft);
    assert_eq!(
        a.rule_set_open, None,
        "no parent above a top-level, already-collapsed header"
    );
}

#[test]
fn arrow_left_on_a_rule_moves_to_its_rule_sets_header_without_collapsing_it() {
    let mut a = expert_in_workspace();
    let rs_ids = rule_set_ids(&a);
    let rule_ids = rule_ids_in(&a, rs_ids[0]);
    tab_into_sidebar(&mut a);
    a.update(Message::ArrowRight); // expand rule set 0
    a.update(Message::ArrowRight); // move to first rule
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::Rule(rule_ids[0]))),
        "test setup sanity"
    );
    a.update(Message::ArrowLeft);
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::RuleSetHeader(rs_ids[0]))),
        "first Left moves to the parent header"
    );
    assert_eq!(
        a.rule_set_open,
        Some(rs_ids[0]),
        "moving to the parent must not also collapse it — a second Left does that"
    );
    a.update(Message::ArrowLeft);
    assert_eq!(
        a.rule_set_open, None,
        "second Left, now on the header itself, collapses it"
    );
}

#[test]
fn arrow_left_on_a_fallback_file_moves_to_the_fallback_header() {
    let mut a = expert_in_workspace();
    tab_into_sidebar(&mut a);
    a.update(Message::ArrowDown); // RuleSetHeader(errors)
    a.update(Message::ArrowDown); // AddRuleSet
    a.update(Message::ArrowDown); // FallbackHeader
    a.update(Message::ArrowRight); // open it
    a.update(Message::ArrowRight); // move to first fallback file
    assert!(
        matches!(
            &a.region_cursor,
            Some(Cursor::Sidebar(SidebarRow::FallbackFile(_)))
        ),
        "test setup sanity"
    );
    a.update(Message::ArrowLeft);
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::FallbackHeader))
    );
}

// ── Edge case 1: clamp on shortening, including the accordion case ───────

#[test]
fn accordion_clamp_opening_rule_set_b_while_the_cursor_sits_on_a_rule_in_a() {
    let mut a = expert_in_workspace();
    let rs_ids = rule_set_ids(&a);
    let rule_ids_a = rule_ids_in(&a, rs_ids[0]);
    tab_into_sidebar(&mut a);
    a.update(Message::ArrowRight); // expand rule set A (rs_ids[0])
    a.update(Message::ArrowRight); // cursor onto its first rule
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::Rule(rule_ids_a[0]))),
        "test setup sanity"
    );
    // Opening B closes A in the accordion — same message a pointer click on
    // B's header would send (task 029 §4: "no change to pointer behaviour").
    a.update(Message::SelectRuleSet(rs_ids[1]));
    assert_eq!(
        a.rule_set_open,
        Some(rs_ids[1]),
        "test setup sanity: B is open, A is not"
    );
    assert_ne!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::Rule(rule_ids_a[0]))),
        "the cursor's row no longer exists once A collapses"
    );
    assert!(
        a.region_cursor.is_some(),
        "task 029 §7 edge case 1: a valid cursor must survive the collapse, \
         not just not-crash"
    );
    let rows = crate::regions::sidebar_rows(&a);
    assert!(
        rows.contains(&match a.region_cursor.clone().unwrap() {
            Cursor::Sidebar(r) => r,
            _ => panic!("wrong cursor kind"),
        }),
        "the surviving cursor must name a row that is actually still visible"
    );
}

// ── Edge case 2: an empty sidebar has no cursor, and Enter is a no-op ─────

#[test]
fn a_sidebar_with_no_snapshot_has_no_cursor_and_enter_does_not_panic() {
    // Not reachable through the public `Message` interface while on the
    // Routes tab (`AppView::Workspace` only follows a successful
    // `install_workspace`, which always leaves a `Some` snapshot) — the same
    // honesty slice 2 gave its own `len == 0` branches. Exercised directly
    // against the defensive path itself, the same idiom as
    // `move_region_cursor_self_heals_when_the_stored_identity_is_no_longer_in_the_list`
    // above.
    let mut a = expert_in_workspace();
    tab_into_sidebar(&mut a);
    a.snapshot = None;
    a.update(Message::ArrowDown);
    assert_eq!(
        a.region_cursor, None,
        "no snapshot means no rows — the empty region has no cursor"
    );
    a.update(Message::EnterPressed);
    assert_eq!(
        a.view,
        AppView::Workspace,
        "Enter on an empty region is a no-op, not a panic"
    );
}

// ── Enter activates exactly the cursor's row ──────────────────────────────

#[test]
fn enter_on_a_fallback_file_opens_it() {
    let mut a = expert_in_workspace();
    let path = a
        .snapshot
        .as_ref()
        .unwrap()
        .fallback_files
        .first()
        .unwrap()
        .path
        .clone();
    tab_into_sidebar(&mut a);
    a.update(Message::ArrowDown); // RuleSetHeader(errors)
    a.update(Message::ArrowDown); // AddRuleSet
    a.update(Message::ArrowDown); // FallbackHeader
    a.update(Message::ArrowRight); // open it
    a.update(Message::ArrowRight); // move to first fallback file
    a.update(Message::EnterPressed);
    assert_eq!(a.selection.file_route, Some(path));
}

#[test]
fn enter_on_the_inert_add_fallback_file_row_is_a_noop() {
    let mut a = expert_in_workspace();
    tab_into_sidebar(&mut a);
    a.update(Message::ArrowDown); // RuleSetHeader(errors)
    a.update(Message::ArrowDown); // AddRuleSet
    a.update(Message::ArrowDown); // FallbackHeader
    a.update(Message::ArrowRight); // open it
    let fallback_count = a.snapshot.as_ref().unwrap().fallback_files.len();
    for _ in 0..fallback_count {
        a.update(Message::ArrowDown);
    }
    a.update(Message::ArrowDown); // AddFallbackFile
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Sidebar(SidebarRow::AddFallbackFile)),
        "test setup sanity"
    );
    let selection_before = a.selection.clone();
    a.update(Message::EnterPressed);
    assert_eq!(
        a.selection, selection_before,
        "no file I/O in this mockup — matches the row's own on_press-less button"
    );
}

// ── §5: the generalised focus-release decision, tested at the reducer level ─

#[test]
fn focus_release_decision_is_a_noop_without_a_transition() {
    assert_eq!(
        super::focus_release_decision(Some(Region::Sidebar), Some(Region::Sidebar)),
        None,
        "same region before and after: nothing to release, or Trace's filter \
         field would lose focus on every keystroke-adjacent message"
    );
    assert_eq!(super::focus_release_decision(None, None), None);
}

#[test]
fn focus_release_decision_unfocuses_on_entering_a_region_without_a_text_field() {
    assert_eq!(
        super::focus_release_decision(Some(Region::DashboardSearch), Some(Region::Sidebar)),
        Some(super::FocusRelease::Unfocus)
    );
    assert_eq!(
        super::focus_release_decision(Some(Region::TabBar), Some(Region::Sidebar)),
        Some(super::FocusRelease::Unfocus),
        "task 029 §5: generalised — this transition involves no text-field \
         region at all, which is exactly the D-18 case a click into a \
         rule-editor field needs released on the next Tab"
    );
}

#[test]
fn focus_release_decision_focuses_on_entering_a_text_field_region() {
    assert_eq!(
        super::focus_release_decision(Some(Region::DashboardCreate), Some(Region::DashboardSearch)),
        Some(super::FocusRelease::Focus(
            crate::screens::dashboard::SEARCH_INPUT_ID.into()
        ))
    );
}

#[test]
fn focus_release_decision_carries_the_editors_own_per_field_id() {
    use crate::regions::EditorField;

    // Task 030 §5: generalises `Focus` to carry *which* id, since the editor
    // has many text-field regions rather than one fixed one.
    assert_eq!(
        super::focus_release_decision(
            Some(Region::Editor(EditorField::Method)),
            Some(Region::Editor(EditorField::UrlPath))
        ),
        Some(super::FocusRelease::Focus("editor-url-path".into()))
    );
}

// ── Task 030 (MK-023 slice 5): the rule editor and the trace screen ────────

use crate::regions::EditorField;
use apimokka_model::HeaderOp;

/// `shop-api-mock`'s third rule in `main.toml` — the only mock rule with
/// both a header and body conditions (one header, two body), so it is the
/// one rule that exercises every row kind the editor can show.
fn rule_with_conditions(a: &App) -> NodeId {
    a.snapshot.as_ref().expect("test setup sanity").rule_sets[0].rules[2].id
}

fn select_rule_with_conditions(a: &mut App) -> NodeId {
    let id = rule_with_conditions(a);
    a.update(Message::SelectRule(id));
    id
}

/// Tabs forward until `target` is the focused region, or panics after a
/// generous bound — a stuck Tab chain should fail the test, not hang it.
fn tab_until(a: &mut App, target: Region) {
    for _ in 0..40 {
        a.update(Message::TabPressed);
        if a.focused_region == Some(target) {
            return;
        }
    }
    panic!(
        "Tab never reached {target:?}; focused_region is {:?}",
        a.focused_region
    );
}

// ── Tab order matches reading order ────────────────────────────────────────

#[test]
fn editor_fields_order_matches_reading_order_for_a_simple_rule() {
    use crate::regions::editor_fields;

    let mut a = expert_in_workspace();
    let health_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(health_id));
    assert_eq!(
        editor_fields(&a),
        vec![
            EditorField::TestRule,
            EditorField::Duplicate,
            EditorField::MoveUp,
            EditorField::MoveDown,
            EditorField::Delete,
            EditorField::UrlPath,
            EditorField::UrlPathOp,
            EditorField::Method,
            EditorField::AddHeader,
            EditorField::AddBody,
            EditorField::RespondMode,
            EditorField::RespondBody,
            EditorField::RespondStatus,
            EditorField::RespondDelay,
            EditorField::PerRuleField,
        ],
        "GET /health has no conditions, so both lists are empty but still \
         present via their Add rows — shop-api-mock's strategy is \
         WeightedRandom (test setup sanity, confirmed at the fixture), so \
         PerRuleField (the weight) is present; no MoreWhenToggle (Expert \
         mode shows both cards directly)"
    );
}

#[test]
fn editor_fields_includes_every_row_for_a_rule_with_conditions() {
    use crate::regions::editor_fields;

    let mut a = expert_in_workspace();
    select_rule_with_conditions(&mut a);
    assert_eq!(
        editor_fields(&a),
        vec![
            EditorField::TestRule,
            EditorField::Duplicate,
            EditorField::MoveUp,
            EditorField::MoveDown,
            EditorField::Delete,
            EditorField::UrlPath,
            EditorField::UrlPathOp,
            EditorField::Method,
            EditorField::HeaderName(0),
            EditorField::HeaderOp(0),
            EditorField::HeaderValue(0),
            EditorField::HeaderRemove(0),
            EditorField::AddHeader,
            EditorField::BodyPath(0),
            EditorField::BodyPathAssistant(0),
            EditorField::BodyOp(0),
            EditorField::BodyValue(0),
            EditorField::BodyRemove(0),
            EditorField::BodyPath(1),
            EditorField::BodyPathAssistant(1),
            EditorField::BodyOp(1),
            EditorField::BodyValue(1),
            EditorField::BodyRemove(1),
            EditorField::AddBody,
            EditorField::RespondMode,
            EditorField::RespondBody,
            EditorField::RespondStatus,
            EditorField::RespondDelay,
            EditorField::PerRuleField,
        ],
        "reading order: WHEN column top to bottom (url path, method, each \
         header row, each body row), then RESPOND column"
    );
}

#[test]
fn header_value_field_is_absent_once_its_operator_makes_it_irrelevant() {
    use crate::regions::editor_fields;

    let mut a = expert_in_workspace();
    select_rule_with_conditions(&mut a);
    assert!(
        editor_fields(&a).contains(&EditorField::HeaderValue(0)),
        "test setup sanity: Contains has a value"
    );
    a.update(Message::HeaderSetOp {
        index: 0,
        op: HeaderOp::Exists,
    });
    assert!(
        !editor_fields(&a).contains(&EditorField::HeaderValue(0)),
        "Exists ignores the value field — mirrors the view's own show_val condition"
    );
}

#[test]
fn more_when_toggle_and_condition_rows_follow_guided_modes_disclosure() {
    use crate::regions::editor_fields;

    let mut a = expert_in_workspace();
    select_rule_with_conditions(&mut a);
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Guided,
    ));
    assert!(
        !a.rule_when_more,
        "test setup sanity: Guided starts collapsed"
    );

    let collapsed = editor_fields(&a);
    assert!(
        collapsed.contains(&EditorField::MoreWhenToggle),
        "the toggle itself is always a region when Guided"
    );
    assert!(
        !collapsed
            .iter()
            .any(|f| matches!(f, EditorField::HeaderName(_) | EditorField::BodyPath(_))),
        "collapsed: no header/body rows exist to Tab to at all"
    );

    a.update(Message::ToggleRuleWhenMore);
    let expanded = editor_fields(&a);
    assert!(expanded.contains(&EditorField::HeaderName(0)));
    assert!(expanded.contains(&EditorField::BodyPath(0)));
}

#[test]
fn tab_reaches_the_editor_after_the_sidebar_on_the_routes_tab() {
    let mut a = expert_in_workspace();
    let health_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(health_id));
    tab_until(&mut a, Region::TabBar);
    a.update(Message::TabPressed); // Sidebar
    assert_eq!(a.focused_region, Some(Region::Sidebar));
    a.update(Message::TabPressed); // first editor field
    assert_eq!(
        a.focused_region,
        Some(Region::Editor(EditorField::TestRule))
    );
}

// ── §3: a pick_list region cycles the value, asserting the message ────────

#[test]
fn url_path_op_cycles_forward_and_saturates_at_the_last_option() {
    let mut a = expert_in_workspace();
    let health_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(health_id));
    tab_until(&mut a, Region::Editor(EditorField::UrlPathOp));

    // Health starts at UrlPathOp::Equal (index 0) — confirmed by the mock
    // fixture directly, not assumed.
    let payload_op = |a: &App| {
        a.snapshot
            .as_ref()
            .unwrap()
            .rule_draft(health_id)
            .map(|d| d.payload.url_path_op)
            .unwrap_or_else(|| {
                a.snapshot
                    .as_ref()
                    .unwrap()
                    .find_rule(health_id)
                    .unwrap()
                    .1
                    .payload
                    .url_path_op
            })
    };
    assert_eq!(payload_op(&a), Some(apimokka_model::UrlPathOp::Equal));

    for _ in 0..10 {
        a.update(Message::ArrowDown);
    }
    assert_eq!(
        payload_op(&a),
        Some(*apimokka_model::UrlPathOp::all().last().unwrap()),
        "ArrowDown cycles forward through all() and saturates at the end — \
         never wraps, the same rule every other region's arrows follow"
    );
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Index(0)),
        "a pick_list region is always one item — the cursor never moves, only the value does"
    );
}

#[test]
fn body_op_cycle_dispatches_the_same_message_the_dropdown_would() {
    // Asserts the message, not the widget — task 030 §8's own framing,
    // since there is no popup to inspect.
    let mut a = expert_in_workspace();
    select_rule_with_conditions(&mut a);
    let region = Region::Editor(EditorField::BodyOp(0));
    let msg = region.cycle_value(&a, 1);
    assert!(
        matches!(msg, Some(Message::BodySetOp { index: 0, op }) if op != apimokka_model::BodyOp::Equal),
        "BodyOp::Equal (index 0) moving forward must land on a different \
         operator via the exact message BodySetOp dispatches"
    );
}

// ── Multi-item groups: Method (6), RespondMode (2) ─────────────────────────

#[test]
fn method_group_resumes_on_the_rules_current_method_and_moves_within_it() {
    let mut a = expert_in_workspace();
    let orders_id = select_rule_with_conditions(&mut a); // POST
    tab_until(&mut a, Region::Editor(EditorField::Method));
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Index(2)),
        "POST is index 2 in [Any, GET, POST, PUT, PATCH, DELETE]"
    );
    a.update(Message::ArrowDown);
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Index(3)),
        "moves within the group, PUT"
    );
    a.update(Message::EnterPressed);
    let method = |a: &App| {
        a.snapshot
            .as_ref()
            .unwrap()
            .find_rule(orders_id)
            .unwrap()
            .1
            .payload
            .method
            .clone()
    };
    assert_eq!(
        method(&a),
        "PUT",
        "Enter activates whichever method the cursor is on"
    );
}

#[test]
fn respond_mode_group_resumes_on_the_rules_current_mode() {
    let mut a = expert_in_workspace();
    select_rule_with_conditions(&mut a); // ServeFile mode
    tab_until(&mut a, Region::Editor(EditorField::RespondMode));
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Index(1)),
        "ServeFile is index 1 — the rule's own mode, not the first item"
    );
}

// ── Trace: Tab order, the identity cursor, and conditional regions ────────

fn trace_tab(a: &mut App) {
    a.update(Message::SwitchTab(crate::selection::WorkspaceTab::Trace));
}

#[test]
fn trace_outcome_action_is_absent_with_no_event_selected_and_present_once_one_is() {
    let mut a = expert_in_workspace();
    trace_tab(&mut a);
    assert!(!Region::order_for(&a).contains(&Region::TraceOutcomeAction));
    assert!(!Region::order_for(&a).contains(&Region::TraceReplay));

    a.update(Message::SelectTraceEvent(1)); // Matched
    assert!(Region::order_for(&a).contains(&Region::TraceOutcomeAction));
    assert!(Region::order_for(&a).contains(&Region::TraceReplay));
}

#[test]
fn trace_outcome_action_message_matches_each_outcome_kind() {
    let mut a = expert_in_workspace();
    trace_tab(&mut a);

    a.update(Message::SelectTraceEvent(1)); // Matched -> rule_sets[0].rules[2]
    let expected_rule = a.snapshot.as_ref().unwrap().rule_sets[0].rules[2].id;
    assert!(matches!(
        Region::TraceOutcomeAction.activation_message(&a, &Cursor::Index(0)),
        Some(Message::JumpToRule(id)) if id == expected_rule
    ));

    a.update(Message::SelectTraceEvent(2)); // Fallback
    assert!(matches!(
        Region::TraceOutcomeAction.activation_message(&a, &Cursor::Index(0)),
        Some(Message::JumpToFile(path)) if path == "responses/health.json"
    ));

    a.update(Message::SelectTraceEvent(3)); // Miss
    assert!(matches!(
        Region::TraceOutcomeAction.activation_message(&a, &Cursor::Index(0)),
        Some(Message::AddRuleForPath(path)) if path == "/api/unknown"
    ));

    a.update(Message::SelectTraceEvent(4)); // Error
    assert!(
        !Region::order_for(&a).contains(&Region::TraceOutcomeAction),
        "Error has no outcome action at all — not merely a no-op one"
    );
}

#[test]
fn conditional_regions_do_not_survive_a_change_that_removes_them() {
    // Task 030 §11's risk: a stale cursor into a vanished conditional
    // region. `valid_focused_region` already guards this generically
    // (task 027) — confirmed here rather than assumed, per the handoff's
    // own instruction.
    let mut a = expert_in_workspace();
    trace_tab(&mut a);
    a.update(Message::SelectTraceEvent(1)); // Matched: has an outcome action
    tab_until(&mut a, Region::TraceOutcomeAction);
    assert_eq!(a.focused_region, Some(Region::TraceOutcomeAction));

    a.update(Message::SelectTraceEvent(4)); // Error: no outcome action
    assert_eq!(
        a.valid_focused_region(),
        None,
        "the region the cursor pointed at no longer exists"
    );
}

#[test]
fn trace_events_identity_cursor_survives_an_event_arriving_while_focused() {
    // This mockup has no live event-arrival simulation to drive through a
    // public `Message` (`app.trace` is seeded once and only ever shrinks,
    // via `TraceClear`) — exercised directly against the defensive path,
    // the same honesty already given for the sidebar's "no snapshot" edge
    // case two tasks ago.
    let mut a = expert_in_workspace();
    trace_tab(&mut a);
    tab_until(&mut a, Region::TraceEvents);
    a.update(Message::ArrowDown);
    let cursor_before = a.region_cursor.clone();
    assert!(
        matches!(cursor_before, Some(Cursor::TraceEvent(_))),
        "test setup sanity"
    );

    let new_event = apimokka_model::MatchTraceEvent {
        event_id: 999,
        time: "12:05:00.000".into(),
        duration_ms: 1,
        request: apimokka_model::trace::RequestSummary {
            method: "GET".into(),
            url_path: "/new".into(),
            headers: vec![],
            body_preview: None,
        },
        outcome: apimokka_model::TraceOutcome::Miss {
            status: "404 Not Found".into(),
        },
        dropped_count: 0,
    };
    a.trace.insert(0, new_event);
    a.update(Message::Noop); // any message re-runs the generic reclamp

    assert_eq!(
        a.region_cursor, cursor_before,
        "an event arriving elsewhere in the list must not disturb a cursor \
         already on a different, still-present event"
    );
}

#[test]
fn trace_events_identity_cursor_self_heals_if_its_own_event_is_cleared() {
    let mut a = expert_in_workspace();
    trace_tab(&mut a);
    tab_until(&mut a, Region::TraceEvents);
    assert!(a.region_cursor.is_some(), "test setup sanity");
    a.update(Message::TraceClear);
    assert_eq!(
        a.region_cursor, None,
        "edge case 2: an emptied list leaves no cursor, not a stale one"
    );
    a.update(Message::EnterPressed);
    assert_eq!(
        a.selected_trace, None,
        "Enter on an empty region is a no-op"
    );
}

// ── `focus_id` ───────────────────────────────────────────────────────────

#[test]
fn focus_id_is_some_only_for_the_editors_actual_text_inputs() {
    assert!(Region::Editor(EditorField::UrlPath).focus_id().is_some());
    assert!(
        Region::Editor(EditorField::HeaderName(0))
            .focus_id()
            .is_some()
    );
    assert!(
        Region::Editor(EditorField::RespondBody)
            .focus_id()
            .is_some()
    );
    assert!(
        Region::Editor(EditorField::UrlPathOp).focus_id().is_none(),
        "a pick_list, not a text_input"
    );
    assert!(
        Region::Editor(EditorField::Method).focus_id().is_none(),
        "a button group"
    );
    assert!(Region::Editor(EditorField::AddHeader).focus_id().is_none());
    assert!(Region::TraceFilter.focus_id().is_some());
    assert!(Region::TracePause.focus_id().is_none());
    assert!(Region::Sidebar.focus_id().is_none());
}
