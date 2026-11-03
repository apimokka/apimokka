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
fn tab_focuses_the_top_bars_switcher_first_inside_a_workspace() {
    // Task 034: the top bar renders above the tab bar
    // (`shell::view::view`'s own `.header(...)`), and the RFC's own Routes
    // focus order names it first — so the first `Tab` inside a workspace
    // now lands here, not on the tab bar.
    let mut a = expert_in_workspace();
    a.update(Message::TabPressed);
    assert_eq!(
        a.focused_region,
        Some(Region::TopBar(TopBarField::Switcher))
    );
}

#[test]
fn tab_reaches_the_tab_bar_after_the_top_bar_inside_a_workspace() {
    let mut a = expert_in_workspace();
    tab_until(&mut a, Region::TabBar);
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
    tab_until(&mut a, Region::TabBar);
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
    tab_until(&mut a, Region::TabBar);
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
    tab_until(&mut a, Region::TabBar);
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
    // Task 033: the confirm dialog is now itself a region (`order_for`'s
    // own override returns `vec![Region::ConfirmDialog]` while it is
    // open), so `Tab` has exactly one region to land on and stays there —
    // it must never reach the tab bar underneath, but it no longer leaves
    // `focused_region` at `None` to prove that; `ConfirmRequest` already
    // set it to `Region::ConfirmDialog` the moment the dialog opened
    // (task 026 §4.2's safety default), and that is what this asserts.
    let mut a = expert_in_workspace();
    a.update(Message::ConfirmRequest(
        crate::message::ConfirmAction::LeaveWorkspace,
    ));
    assert!(a.confirm_dialog.is_some(), "test setup sanity");
    a.update(Message::TabPressed);
    assert_eq!(
        a.focused_region,
        Some(crate::regions::Region::ConfirmDialog),
        "Tab must not focus the tab bar underneath an open dialog"
    );
}

#[test]
fn arrow_keys_stay_within_test_rule_while_it_is_open() {
    // Task 034: Test Rule is now a region like the confirm dialog
    // (`Overlay::TestRule`, `Message::TestRuleOpen` forcing the entry
    // position) rather than a `blocks_region_navigation` special case
    // that left `focused_region` at `None` — see this test's own old name
    // for the behaviour this replaced.
    let mut a = expert_in_workspace();
    a.update(Message::TestRuleOpen);
    assert!(a.test_rule.open, "test setup sanity");
    assert_eq!(
        a.focused_region,
        Some(Region::TestRule(TestRuleField::Run)),
        "test setup sanity: TestRuleOpen forces the entry position"
    );
    a.update(Message::ArrowDown);
    assert_eq!(
        a.focused_region,
        Some(Region::TestRule(TestRuleField::Run)),
        "arrows must stay within Test Rule's own region order, never reach the tab bar underneath"
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
    tab_until(a, Region::Sidebar);
}

// ── Tab order and cycling ──────────────────────────────────────────────────

#[test]
fn tab_cycles_tab_bar_sidebar_and_the_empty_cta_on_routes_and_wraps() {
    // Task 034 (the census, F2): a fresh workspace has no selection at all
    // (`RouteSelection::default()`), so the Routes centre panel's own
    // empty-state call to action is a region in this order too. Also task
    // 034: the top bar precedes the tab bar, so the cycle now wraps back
    // to it, not to the tab bar.
    let mut a = expert_in_workspace();
    tab_until(&mut a, Region::TabBar);
    a.update(Message::TabPressed);
    assert_eq!(a.focused_region, Some(Region::Sidebar));
    a.update(Message::TabPressed);
    assert_eq!(a.focused_region, Some(Region::RoutesEmptyCta));
    a.update(Message::TabPressed);
    assert_eq!(
        a.focused_region,
        Some(Region::TopBar(TopBarField::Switcher)),
        "wraps back to the first region in order"
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

    tab_until(&mut a, Region::Sidebar);
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
    // Task 031: the two disabled fields still get an id — a disabled
    // `text_input` is still click-focusable in iced (confirmed against its
    // own `update`: the mouse-press branch never checks `on_input.is_none()`),
    // so omitting the id would make keyboard focus-movement *less*
    // consistent with the pointer than giving it one.
    assert!(
        Region::Settings(SettingsField::LogFile)
            .focus_id()
            .is_some()
    );
    assert!(
        Region::Settings(SettingsField::QueueSize)
            .focus_id()
            .is_some()
    );
    assert!(Region::Settings(SettingsField::Tls).focus_id().is_none());
    assert!(Region::Settings(SettingsField::Theme).focus_id().is_none());
    assert!(Region::Wizard(WizardField::Name).focus_id().is_some());
    assert!(Region::Wizard(WizardField::QueueSize).focus_id().is_some());
    assert!(
        Region::Wizard(WizardField::TraceEnable)
            .focus_id()
            .is_none()
    );
    assert!(Region::Wizard(WizardField::Starter).focus_id().is_none());
}

// ── Task 031 (MK-023 slice 6): Settings and the Wizard ─────────────────────

use crate::regions::{SettingsField, WizardField};

fn settings_tab(a: &mut App) {
    a.update(Message::SwitchTab(WorkspaceTab::Settings));
}

fn guided_in_workspace() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(AudienceMode::Guided));
    a.update(Message::OpenWorkspace("test".into()));
    assert_eq!(a.view, AppView::Workspace, "test setup sanity");
    a
}

fn wizard_from_welcome() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    a.update(Message::GoWizard);
    assert_eq!(a.view, AppView::Wizard, "test setup sanity");
    a
}

// ── Tab order matches reading order ────────────────────────────────────────

#[test]
fn tab_walks_settings_in_reading_order_in_expert_mode() {
    let mut a = expert_in_workspace();
    settings_tab(&mut a);
    let expected = [
        Region::Settings(SettingsField::Theme),
        Region::Settings(SettingsField::AudienceMode),
        Region::Settings(SettingsField::Locale),
        Region::Settings(SettingsField::Host),
        Region::Settings(SettingsField::Port),
        Region::Settings(SettingsField::Tls),
        Region::Settings(SettingsField::LogFile),
        Region::Settings(SettingsField::TraceEnable),
        Region::Settings(SettingsField::QueueSize),
    ];
    tab_until(&mut a, Region::TabBar);
    for region in expected {
        a.update(Message::TabPressed);
        assert_eq!(a.focused_region, Some(region));
    }
}

#[test]
fn tab_walks_the_wizard_in_reading_order_once_every_section_is_open() {
    let mut a = wizard_from_welcome();
    a.update(Message::WizardToggleSection(0));
    a.update(Message::WizardToggleSection(1));
    a.update(Message::WizardToggleSection(2));
    let expected = [
        Region::Wizard(WizardField::Name),
        Region::Wizard(WizardField::Folder),
        Region::Wizard(WizardField::ServerToggle),
        Region::Wizard(WizardField::Host),
        Region::Wizard(WizardField::Port),
        Region::Wizard(WizardField::Tls),
        Region::Wizard(WizardField::StarterToggle),
        Region::Wizard(WizardField::Starter),
        Region::Wizard(WizardField::TraceToggle),
        Region::Wizard(WizardField::TraceEnable),
        Region::Wizard(WizardField::QueueSize),
        Region::Wizard(WizardField::Cancel),
        Region::Wizard(WizardField::Create),
    ];
    for region in expected {
        a.update(Message::TabPressed);
        assert_eq!(a.focused_region, Some(region));
    }
}

// ── Conditional regions: Settings' advanced disclosure ─────────────────────

#[test]
fn settings_advanced_fields_are_gated_by_guided_modes_disclosure() {
    let mut a = guided_in_workspace();
    settings_tab(&mut a);
    assert!(
        Region::order_for(&a).contains(&Region::Settings(SettingsField::AdvancedToggle)),
        "the toggle itself is always present in Guided mode"
    );
    assert!(
        !Region::order_for(&a).contains(&Region::Settings(SettingsField::LogFile)),
        "collapsed: the advanced fields are not declared at all"
    );

    a.update(Message::ToggleSettingsAdvancedMore);
    // Task 031 §2: the toggle renders *after* the advanced fields once
    // expanded, mirroring `screens::settings::view`'s own push order exactly
    // — not merely "all four present", which order_for alone wouldn't catch.
    let order = Region::order_for(&a);
    assert_eq!(
        &order[order.len() - 4..],
        &[
            Region::Settings(SettingsField::LogFile),
            Region::Settings(SettingsField::TraceEnable),
            Region::Settings(SettingsField::QueueSize),
            Region::Settings(SettingsField::AdvancedToggle),
        ]
    );
}

#[test]
fn settings_advanced_toggle_closing_does_not_strand_the_cursor_on_a_vanished_field() {
    // Task 030's stale-cursor guard (`valid_focused_region`), confirmed
    // against this slice's own conditional region rather than assumed.
    let mut a = guided_in_workspace();
    settings_tab(&mut a);
    a.update(Message::ToggleSettingsAdvancedMore);
    tab_until(&mut a, Region::Settings(SettingsField::LogFile));
    assert_eq!(
        a.focused_region,
        Some(Region::Settings(SettingsField::LogFile))
    );

    a.update(Message::ToggleSettingsAdvancedMore); // collapse
    assert_eq!(
        a.valid_focused_region(),
        None,
        "task 027's existing mechanism generalises to this case with no change"
    );
}

// ── Conditional regions: the Wizard's section disclosures ─────────────────

#[test]
fn wizard_section_bodies_are_gated_by_their_own_open_state() {
    let mut a = wizard_from_welcome();
    assert!(!Region::order_for(&a).contains(&Region::Wizard(WizardField::Host)));
    assert!(!Region::order_for(&a).contains(&Region::Wizard(WizardField::Starter)));
    assert!(!Region::order_for(&a).contains(&Region::Wizard(WizardField::TraceEnable)));

    a.update(Message::WizardToggleSection(0));
    assert!(Region::order_for(&a).contains(&Region::Wizard(WizardField::Host)));
    assert!(
        !Region::order_for(&a).contains(&Region::Wizard(WizardField::Starter)),
        "each section's body is gated independently"
    );
}

#[test]
fn wizard_section_closing_does_not_strand_the_cursor_on_a_vanished_field() {
    let mut a = wizard_from_welcome();
    a.update(Message::WizardToggleSection(0));
    tab_until(&mut a, Region::Wizard(WizardField::Host));
    assert_eq!(a.focused_region, Some(Region::Wizard(WizardField::Host)));

    a.update(Message::WizardToggleSection(0)); // collapse
    assert_eq!(a.valid_focused_region(), None);
}

// ── Multi-item groups resume on the current value ───────────────────────

#[test]
fn settings_theme_group_resumes_on_the_current_theme_and_moves_within_it() {
    use crate::app::ThemeChoice;

    let mut a = expert_in_workspace();
    a.update(Message::SetTheme(ThemeChoice::HighContrastLight));
    settings_tab(&mut a);
    tab_until(&mut a, Region::Settings(SettingsField::Theme));
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Index(2)),
        "HighContrastLight is ThemeChoice::all()'s 3rd entry, not the first"
    );
    a.update(Message::ArrowDown);
    assert_eq!(a.region_cursor, Some(Cursor::Index(3)));
}

#[test]
fn settings_audience_mode_group_resumes_on_the_current_mode() {
    // expert_in_workspace() is already in Expert mode — AudienceMode::all()'s
    // 2nd entry, not the first, which is the point being tested.
    let mut a = expert_in_workspace();
    settings_tab(&mut a);
    tab_until(&mut a, Region::Settings(SettingsField::AudienceMode));
    assert_eq!(a.region_cursor, Some(Cursor::Index(1)));
}

#[test]
fn wizard_starter_group_resumes_on_the_current_starter_choice() {
    let mut a = wizard_from_welcome();
    a.update(Message::WizardSetStarter(
        crate::app::WizardStarter::ShopApi,
    ));
    a.update(Message::WizardToggleSection(1));
    tab_until(&mut a, Region::Wizard(WizardField::Starter));
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Index(1)),
        "ShopApi is WIZARD_STARTER_ORDER's 2nd entry, not the first"
    );
}

// ── §3: the locale pick_list cycles the value, asserting the message ──────

#[test]
fn settings_locale_cycles_forward_and_back_and_saturates_at_both_ends() {
    let mut a = expert_in_workspace();
    assert_eq!(a.locale, apimokka_i18n::Locale::En, "test setup sanity");
    settings_tab(&mut a);
    tab_until(&mut a, Region::Settings(SettingsField::Locale));

    a.update(Message::ArrowDown);
    assert_eq!(a.locale, apimokka_i18n::Locale::Ja);
    a.update(Message::ArrowDown);
    assert_eq!(
        a.locale,
        apimokka_i18n::Locale::Ja,
        "2 options — saturates at the last, never wraps"
    );
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Index(0)),
        "a pick_list region is always one item — the cursor never moves"
    );

    a.update(Message::ArrowUp);
    assert_eq!(a.locale, apimokka_i18n::Locale::En);
    a.update(Message::ArrowUp);
    assert_eq!(
        a.locale,
        apimokka_i18n::Locale::En,
        "saturates at the first"
    );
}

#[test]
fn settings_locale_cycle_dispatches_change_locale() {
    let a = expert_in_workspace();
    let msg = Region::Settings(SettingsField::Locale).cycle_value(&a, 1);
    assert!(matches!(
        msg,
        Some(Message::ChangeLocale(apimokka_i18n::Locale::Ja))
    ));
}

// ── Disabled fields: reachable, inert on Enter (task 031 §8/§4.2) ─────────

#[test]
fn settings_log_file_and_queue_size_are_reachable_but_enter_is_a_noop() {
    let mut a = expert_in_workspace();
    settings_tab(&mut a);
    tab_until(&mut a, Region::Settings(SettingsField::LogFile));
    let before = a.snapshot.as_ref().unwrap().root_settings.log_file.clone();
    a.update(Message::EnterPressed);
    assert_eq!(a.snapshot.as_ref().unwrap().root_settings.log_file, before);

    a.update(Message::TabPressed); // TraceEnable
    a.update(Message::TabPressed); // QueueSize
    assert_eq!(
        a.focused_region,
        Some(Region::Settings(SettingsField::QueueSize))
    );
    let before_q = a
        .snapshot
        .as_ref()
        .unwrap()
        .prototype
        .trace
        .as_ref()
        .map(|t| t.queue_size);
    a.update(Message::EnterPressed);
    assert_eq!(
        a.snapshot
            .as_ref()
            .unwrap()
            .prototype
            .trace
            .as_ref()
            .map(|t| t.queue_size),
        before_q,
        "no on_input is wired for this field at all — Enter must agree \
         with what a click already does: nothing"
    );
}

#[test]
fn settings_trace_enable_is_live_unlike_the_wizards_own_trace_checkbox() {
    let mut a = expert_in_workspace();
    settings_tab(&mut a);
    tab_until(&mut a, Region::Settings(SettingsField::TraceEnable));
    let before = a
        .snapshot
        .as_ref()
        .unwrap()
        .prototype
        .trace
        .as_ref()
        .map(|t| t.enabled);
    a.update(Message::EnterPressed);
    assert_ne!(
        a.snapshot
            .as_ref()
            .unwrap()
            .prototype
            .trace
            .as_ref()
            .map(|t| t.enabled),
        before,
        "unlike Settings' own LogFile/QueueSize, this checkbox has a real \
         on_toggle wired"
    );
}

#[test]
fn wizard_trace_checkbox_is_reachable_but_enter_is_a_noop() {
    let mut a = wizard_from_welcome();
    a.update(Message::WizardToggleSection(2));
    tab_until(&mut a, Region::Wizard(WizardField::TraceEnable));
    a.update(Message::EnterPressed);
    // No on_toggle is wired in `screens::wizard.rs` at all — nothing to
    // assert changed; reaching here without panicking, with the cursor
    // still on the same region, is the whole test.
    assert_eq!(
        a.focused_region,
        Some(Region::Wizard(WizardField::TraceEnable))
    );
}

// ── Scenario 2's wizard half, keyboard-completable end to end ─────────────

#[test]
fn enter_on_wizard_create_creates_the_workspace_from_a_fresh_welcome_launch() {
    let mut a = wizard_from_welcome();
    a.update(Message::WizardSetName("payments-mock".into()));
    tab_until(&mut a, Region::Wizard(WizardField::Create));
    a.update(Message::EnterPressed);
    assert_eq!(a.view, AppView::Workspace);
    assert_eq!(a.snapshot.as_ref().unwrap().identity.name, "payments-mock");
}

// ── Task 032 — the final slice ─────────────────────────────────────────────

// ── D-18: a click enters its text-field region ─────────────────────────────

#[test]
fn region_clicked_enters_a_one_item_text_field_region_from_nothing_focused() {
    let mut a = expert_at_dashboard();
    assert_eq!(a.focused_region, None, "test setup sanity");
    a.update(Message::RegionClicked(Region::DashboardSearch));
    assert_eq!(a.focused_region, Some(Region::DashboardSearch));
    assert_eq!(a.region_cursor, Some(Cursor::Index(0)));
}

#[test]
fn region_clicked_moves_focus_away_from_whatever_tab_last_left_it_on() {
    let mut a = expert_in_workspace();
    let health_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(health_id));
    tab_until(&mut a, Region::Sidebar);
    assert_eq!(a.focused_region, Some(Region::Sidebar), "test setup sanity");

    a.update(Message::RegionClicked(Region::Editor(
        EditorField::RespondStatus,
    )));
    assert_eq!(
        a.focused_region,
        Some(Region::Editor(EditorField::RespondStatus))
    );
    assert_eq!(a.region_cursor, Some(Cursor::Index(0)));
}

// ── D-19: `Enter` cycles forward on all four value cyclers ─────────────────

#[test]
fn enter_dispatches_the_same_message_arrow_down_would_on_all_four_cyclers() {
    let mut a = expert_in_workspace();
    select_rule_with_conditions(&mut a);

    for region in [
        Region::Editor(EditorField::UrlPathOp),
        Region::Editor(EditorField::HeaderOp(0)),
        Region::Editor(EditorField::BodyOp(0)),
        Region::Settings(SettingsField::Locale),
    ] {
        // `Message` derives no `PartialEq` — same `Debug`-rendering
        // comparison `palette_commands.rs`'s own tests already use.
        assert_eq!(
            format!("{:?}", region.activation_message(&a, &Cursor::Index(0))),
            format!("{:?}", region.cycle_value(&a, 1)),
            "{region:?}: `Enter` must dispatch the same message `ArrowDown` would"
        );
    }
}

#[test]
fn enter_on_url_path_op_actually_advances_the_rules_value() {
    let mut a = expert_in_workspace();
    let health_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(health_id));
    tab_until(&mut a, Region::Editor(EditorField::UrlPathOp));

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
    a.update(Message::EnterPressed);
    assert_ne!(
        payload_op(&a),
        Some(apimokka_model::UrlPathOp::Equal),
        "Enter must have moved the value forward, not left it a no-op"
    );
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Index(0)),
        "a pick_list region is always one item — the cursor never moves"
    );
}

#[test]
fn enter_on_settings_locale_cycles_forward_and_updates_the_real_locale() {
    let mut a = expert_in_workspace();
    settings_tab(&mut a);
    tab_until(&mut a, Region::Settings(SettingsField::Locale));
    assert_eq!(a.locale, apimokka_i18n::Locale::En, "test setup sanity");
    a.update(Message::EnterPressed);
    assert_eq!(a.locale, apimokka_i18n::Locale::Ja);
}

// ── D-20: scroll-into-view for the six previously-unwired scrollables ──────

#[test]
fn scroll_snap_target_is_none_when_the_key_is_not_a_navigation_key() {
    let mut a = expert_in_workspace();
    settings_tab(&mut a);
    tab_until(&mut a, Region::Settings(SettingsField::Theme));
    assert_eq!(a.scroll_snap_target(false), None);
}

#[test]
fn scroll_snap_target_picks_the_when_column_for_a_when_field() {
    let mut a = expert_in_workspace();
    let health_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(health_id));
    tab_until(&mut a, Region::Editor(EditorField::UrlPathOp));
    let (id, _y) = a.scroll_snap_target(true).expect("a target");
    assert_eq!(id, crate::screens::routes::WHEN_SCROLLABLE_ID);
}

#[test]
fn scroll_snap_target_picks_the_respond_column_not_the_when_one() {
    // Task 031 §11's own named risk: this is the one place the fix can be
    // wired to the wrong target and still look plausible.
    let mut a = expert_in_workspace();
    let health_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(health_id));
    tab_until(&mut a, Region::Editor(EditorField::RespondStatus));
    let (id, _y) = a.scroll_snap_target(true).expect("a target");
    assert_eq!(id, crate::screens::routes::RESPOND_SCROLLABLE_ID);
}

#[test]
fn scroll_snap_target_for_the_trace_event_list() {
    let mut a = expert_in_workspace();
    trace_tab(&mut a);
    tab_until(&mut a, Region::TraceEvents);
    let (id, _y) = a.scroll_snap_target(true).expect("a target");
    assert_eq!(id, crate::screens::trace::EVENTS_SCROLLABLE_ID);
}

#[test]
fn scroll_snap_target_for_the_match_detail_panel() {
    let mut a = expert_in_workspace();
    trace_tab(&mut a);
    a.update(Message::SelectTraceEvent(1)); // Matched
    tab_until(&mut a, Region::TraceOutcomeAction);
    let (id, _y) = a.scroll_snap_target(true).expect("a target");
    assert_eq!(id, crate::screens::trace::DETAIL_SCROLLABLE_ID);
}

#[test]
fn scroll_snap_target_for_settings() {
    let mut a = expert_in_workspace();
    settings_tab(&mut a);
    tab_until(&mut a, Region::Settings(SettingsField::Host));
    let (id, _y) = a.scroll_snap_target(true).expect("a target");
    assert_eq!(id, crate::screens::settings::SCROLLABLE_ID);
}

#[test]
fn update_and_dispatch_snaps_on_tab_pressed_not_only_on_arrow_keys() {
    // Regression, in two parts:
    //
    // 1. Every region this slice wires scroll-into-view for is reached by
    //    `Tab`, not an arrow (each field is its own one-item region, unlike
    //    the sidebar/Dashboard/palette's shared-region-plus-arrows shape
    //    `is_navigation_key` was written for) — a first version of
    //    `scroll_snap_target` gated on `is_navigation_key` alone and never
    //    fired on `Tab` at all.
    //
    // 2. `focus_release_decision` *also* fires on the same `Tab` (every
    //    region this slice adds has no `focus_id`, so landing on one is an
    //    `Unfocus`) — a second version fixed (1) by gating on `Tab` but kept
    //    every block as an early `return`, so the pre-existing `Unfocus`
    //    `Task` masked the new scroll one every time, silently. `units()`
    //    on a single `Task` can't tell "fired" from "didn't" by itself
    //    (`Task::none()` is the only 0), which is why this compares two
    //    transitions rather than asserting on one in isolation — a
    //    `Task::batch` of both is what landing on a non-text, scrolling
    //    region now returns, one more unit than landing on a non-scrolling
    //    one.
    let mut a = expert_in_workspace();
    settings_tab(&mut a);
    tab_until(&mut a, Region::TopBar(TopBarField::StartStop)); // the top bar's own last field
    let baseline = a.update_and_dispatch(Message::TabPressed); // TabBar: Unfocus only, not a scroll target
    assert_eq!(a.focused_region, Some(Region::TabBar));
    let with_scroll = a.update_and_dispatch(Message::TabPressed); // Settings(Theme): Unfocus *and* scroll
    assert_eq!(
        a.focused_region,
        Some(Region::Settings(SettingsField::Theme))
    );
    assert_eq!(
        with_scroll.units(),
        baseline.units() + 1,
        "landing on a scrolling region must batch a snap_to Task alongside \
         the Unfocus one, not have it silently dropped"
    );
}

#[test]
fn scroll_snap_target_for_the_wizard_excludes_cancel_and_create() {
    let mut a = wizard_from_welcome();
    tab_until(&mut a, Region::Wizard(WizardField::Folder));
    let (id, _y) = a.scroll_snap_target(true).expect("a target");
    assert_eq!(id, crate::screens::wizard::SCROLLABLE_ID);

    tab_until(&mut a, Region::Wizard(WizardField::Create));
    assert_eq!(
        a.scroll_snap_target(true),
        None,
        "Cancel/Create sit in the action bar below the scrollable, not inside it"
    );
}

// ── Task 033 (MK-023 slice 8): the three pre-existing scroll dispatches, ──
// folded into `scroll_snap_target` per slice 7's review — the regression
// §6's last edge case exists for: the refactor must not silently drop one,
// which is precisely how D-20 reached live the first time.

#[test]
fn scroll_snap_target_still_fires_for_the_palette_results_list() {
    let mut a = expert_in_workspace();
    a.update(Message::ToggleCommandPalette);
    a.update(Message::ArrowDown);
    a.update(Message::ArrowDown);
    assert_eq!(a.command_palette.selected, Some(1), "test setup sanity");
    let (id, y) = a.scroll_snap_target(true).expect("a target");
    assert_eq!(id, crate::screens::command_palette::RESULTS_SCROLLABLE_ID);
    assert!(y > 0.0, "row 1 of many must not snap back to the top");
}

#[test]
fn scroll_snap_target_still_fires_for_the_dashboard_workspace_list() {
    let mut a = expert_at_dashboard();
    tab_into_dashboard_workspaces(&mut a);
    a.update(Message::ArrowDown);
    let (id, y) = a.scroll_snap_target(true).expect("a target");
    assert_eq!(id, crate::screens::dashboard::WORKSPACES_SCROLLABLE_ID);
    assert!(
        y > 0.0,
        "the second workspace must not snap back to the top"
    );
}

#[test]
fn scroll_snap_target_still_fires_for_the_sidebar() {
    let mut a = expert_in_workspace();
    tab_into_sidebar(&mut a);
    a.update(Message::ArrowDown);
    let (id, y) = a.scroll_snap_target(true).expect("a target");
    assert_eq!(id, crate::screens::routes::SIDEBAR_SCROLLABLE_ID);
    assert!(y > 0.0, "the second row must not snap back to the top");
}

// ── Task 033: the fallback editor, the rule-set configuration pane, and ──
// the bottom drawer — the three named uncovered surfaces (§2), plus their
// own scroll targets.

use crate::regions::{DrawerField, FallbackField, RuleSetConfigField, TestRuleField, TopBarField};

fn select_first_fallback_file(a: &mut App) -> String {
    let path = a.snapshot.as_ref().unwrap().fallback_files[0].path.clone();
    a.update(Message::SelectFileRoute(path.clone()));
    path
}

fn select_first_rule_set(a: &mut App) -> apimokka_model::RuleSetId {
    let id = a.snapshot.as_ref().unwrap().rule_sets[0].id;
    a.update(Message::SelectRuleSet(id));
    id
}

#[test]
fn tab_reaches_every_fallback_editor_field_in_reading_order() {
    let mut a = expert_in_workspace();
    select_first_fallback_file(&mut a);
    for field in [
        FallbackField::Content,
        FallbackField::Status,
        FallbackField::Format,
        FallbackField::Revert,
        FallbackField::Save,
    ] {
        tab_until(&mut a, Region::FallbackEditor(field));
    }
}

#[test]
fn scroll_snap_target_is_none_for_every_fallback_editor_field() {
    // §6's scroll edge case, N/A with a reason: `fallback_file_editor`'s
    // own layout has no scrollable at all (header, then a `Length::Fill`
    // content card, then the footer — confirmed against the view, not
    // assumed), so there is nothing for any of its five fields to clamp
    // to, unlike the rule-set configuration pane and the drawer below.
    let mut a = expert_in_workspace();
    select_first_fallback_file(&mut a);
    for field in [
        FallbackField::Content,
        FallbackField::Status,
        FallbackField::Format,
        FallbackField::Revert,
        FallbackField::Save,
    ] {
        tab_until(&mut a, Region::FallbackEditor(field));
        assert_eq!(a.scroll_snap_target(true), None);
    }
}

#[test]
fn tab_reaches_every_rule_set_config_field_in_reading_order() {
    let mut a = expert_in_workspace();
    select_first_rule_set(&mut a);
    for field in [
        RuleSetConfigField::Delete,
        RuleSetConfigField::Strategy,
        RuleSetConfigField::AddRule,
    ] {
        tab_until(&mut a, Region::RuleSetConfig(field));
    }
}

#[test]
fn scroll_snap_target_for_rule_set_config_excludes_delete_but_covers_the_rest() {
    // `Delete` sits in the header above the scrollable, not inside it —
    // the same exclusion Wizard's own `Cancel`/`Create` get.
    let mut a = expert_in_workspace();
    select_first_rule_set(&mut a);
    tab_until(&mut a, Region::RuleSetConfig(RuleSetConfigField::Delete));
    assert_eq!(a.scroll_snap_target(true), None);

    tab_until(&mut a, Region::RuleSetConfig(RuleSetConfigField::Strategy));
    let (id, _y) = a.scroll_snap_target(true).expect("a target");
    assert_eq!(id, crate::screens::routes::RULE_SET_CONFIG_SCROLLABLE_ID);

    tab_until(&mut a, Region::RuleSetConfig(RuleSetConfigField::AddRule));
    let (id, _y) = a.scroll_snap_target(true).expect("a target");
    assert_eq!(id, crate::screens::routes::RULE_SET_CONFIG_SCROLLABLE_ID);
}

#[test]
fn enter_on_rule_set_config_strategy_dispatches_the_same_message_arrow_down_would() {
    // The fifth value cycler (D-19's mechanism) — the same contract as the
    // other four (`enter_dispatches_the_same_message_arrow_down_would_on_all_four_cyclers`
    // above), exercised separately because it needs a rule-*set*
    // selection in context, which that shared loop does not have (it
    // selects a rule, which would make `rule_set_config_context` return
    // `None`).
    let mut a = expert_in_workspace();
    select_first_rule_set(&mut a);
    let region = Region::RuleSetConfig(RuleSetConfigField::Strategy);
    assert_eq!(
        format!("{:?}", region.activation_message(&a, &Cursor::Index(0))),
        format!("{:?}", region.cycle_value(&a, 1)),
        "`Enter` must dispatch the same message `ArrowDown` would"
    );
}

#[test]
fn tab_reaches_every_drawer_field_in_reading_order() {
    let mut a = expert_in_workspace();
    a.update(Message::OpenSaveDiffDrawer);
    for field in [
        DrawerField::Close,
        DrawerField::Discard,
        DrawerField::SaveAll,
    ] {
        tab_until(&mut a, Region::Drawer(field));
    }
}

#[test]
fn scroll_snap_target_for_the_drawer_excludes_close_but_covers_the_rest() {
    let mut a = expert_in_workspace();
    a.update(Message::OpenSaveDiffDrawer);
    tab_until(&mut a, Region::Drawer(DrawerField::Close));
    assert_eq!(a.scroll_snap_target(true), None);

    tab_until(&mut a, Region::Drawer(DrawerField::Discard));
    let (id, _y) = a.scroll_snap_target(true).expect("a target");
    assert_eq!(id, crate::shell::bottom_drawer::SCROLLABLE_ID);

    tab_until(&mut a, Region::Drawer(DrawerField::SaveAll));
    let (id, _y) = a.scroll_snap_target(true).expect("a target");
    assert_eq!(id, crate::shell::bottom_drawer::SCROLLABLE_ID);
}

// ── Required evidence (§7), reducer-level: Scenario 5's revert step ───────

#[test]
fn fallback_revert_opens_the_confirm_dialog_and_proceed_needs_a_deliberate_arrow_down() {
    // A bare `Enter` right after opening cancels (task 026 §4.2's safety
    // default); `Proceed` — the destructive action, discarding the draft —
    // is reachable only by a deliberate `ArrowDown` first, never by
    // `Enter` alone.
    let mut a = expert_in_workspace();
    let path = select_first_fallback_file(&mut a);
    a.fallback_drafts.insert(
        path.clone(),
        iced::widget::text_editor::Content::with_text("{\"x\":1}"),
    );
    assert!(a.is_fallback_dirty(&path), "test setup sanity");

    tab_until(&mut a, Region::FallbackEditor(FallbackField::Revert));
    a.update(Message::EnterPressed);
    assert!(a.confirm_dialog.is_some(), "test setup sanity");
    assert_eq!(
        a.focused_region,
        Some(Region::ConfirmDialog),
        "ConfirmRequest must move the cursor onto the dialog immediately"
    );

    // A bare Enter cancels.
    a.update(Message::EnterPressed);
    assert!(
        a.confirm_dialog.is_none(),
        "a bare Enter with no deliberate ArrowDown must cancel, not revert"
    );
    assert!(
        a.is_fallback_dirty(&path),
        "cancelling must not have reverted anything"
    );

    // Redo, and this time move to Proceed deliberately before Enter.
    tab_until(&mut a, Region::FallbackEditor(FallbackField::Revert));
    a.update(Message::EnterPressed);
    assert!(a.confirm_dialog.is_some(), "test setup sanity");
    a.update(Message::ArrowDown);
    assert_eq!(
        a.region_cursor,
        Some(Cursor::Index(1)),
        "ArrowDown must move the cursor onto Proceed"
    );
    a.update(Message::EnterPressed);
    assert!(a.confirm_dialog.is_none());
    assert!(
        !a.is_fallback_dirty(&path),
        "a deliberate ArrowDown then Enter must have reverted the draft"
    );
}

// ── Task 034 (MK-023 slice 9): the inventory, and the one overlay ────────
// precedence source (D-22).

use crate::regions::{
    FeedbackBannerField, PathAssistantField, TraceActivityField, WorkspaceMenuField,
};

#[test]
fn tab_reaches_every_top_bar_field_in_reading_order() {
    let mut a = expert_in_workspace();
    for field in [
        TopBarField::Switcher,
        TopBarField::SaveAll,
        TopBarField::StartStop,
    ] {
        tab_until(&mut a, Region::TopBar(field));
    }
}

#[test]
fn tab_reaches_every_test_rule_field_in_reading_order() {
    let mut a = expert_in_workspace();
    a.update(Message::TestRuleOpen);
    for field in [
        TestRuleField::Dismiss,
        TestRuleField::Method,
        TestRuleField::Path,
        TestRuleField::Headers,
        TestRuleField::Body,
        TestRuleField::Close,
        TestRuleField::Run,
    ] {
        tab_until(&mut a, Region::TestRule(field));
    }
}

#[test]
fn test_rule_enter_runs_the_dry_run_from_the_entry_position_and_from_each_text_field() {
    // Task 034 §6: task 026's own oldest behaviour in this cycle,
    // preserved explicitly rather than assumed.
    let mut a = expert_in_workspace();
    a.update(Message::TestRuleOpen);
    assert_eq!(
        a.focused_region,
        Some(Region::TestRule(TestRuleField::Run)),
        "test setup sanity: the entry position"
    );
    a.update(Message::EnterPressed);
    assert!(
        a.test_rule.result.is_some(),
        "Enter from the entry position must run it"
    );

    for field in [
        TestRuleField::Path,
        TestRuleField::Headers,
        TestRuleField::Body,
    ] {
        a.test_rule.result = None;
        tab_until(&mut a, Region::TestRule(field));
        a.update(Message::EnterPressed);
        assert!(
            a.test_rule.result.is_some(),
            "Enter from {field:?} must also run it — the one deliberate exception"
        );
    }
}

#[test]
fn tab_reaches_every_workspace_menu_field_in_reading_order() {
    let mut a = expert_in_workspace();
    a.workspace_menu_open = true;
    for field in [
        WorkspaceMenuField::Recent,
        WorkspaceMenuField::Open,
        WorkspaceMenuField::Create,
    ] {
        tab_until(&mut a, Region::WorkspaceMenu(field));
    }
}

#[test]
fn tab_reaches_every_path_assistant_field_in_reading_order() {
    let mut a = expert_in_workspace();
    a.path_assistant.open = true;
    a.path_assistant.json_input = r#"{"id":1}"#.into();
    for field in [
        PathAssistantField::Close,
        PathAssistantField::Json,
        PathAssistantField::UseLeaf(0),
        PathAssistantField::Cancel,
        PathAssistantField::Insert,
    ] {
        tab_until(&mut a, Region::PathAssistant(field));
    }
}

#[test]
fn scroll_snap_target_for_the_path_assistants_leaf_tree() {
    let mut a = expert_in_workspace();
    a.path_assistant.open = true;
    a.path_assistant.json_input = r#"{"id":1,"name":"x","age":2}"#.into();
    tab_until(&mut a, Region::PathAssistant(PathAssistantField::Close));
    assert_eq!(
        a.scroll_snap_target(true),
        None,
        "Close sits above the leaf tree's own scrollable, not inside it"
    );
    tab_until(
        &mut a,
        Region::PathAssistant(PathAssistantField::UseLeaf(1)),
    );
    let (id, _y) = a.scroll_snap_target(true).expect("a target");
    assert_eq!(id, crate::screens::dotted_path::TREE_SCROLLABLE_ID);
}

#[test]
fn tab_reaches_every_trace_activity_field_in_reading_order() {
    let mut a = expert_in_workspace();
    let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(rule_id));
    tab_until(&mut a, Region::TraceActivity(TraceActivityField::ViewAll));
}

#[test]
fn tab_reaches_every_feedback_banner_field_when_a_notice_is_showing() {
    let mut a = expert_in_workspace();
    a.notice = Some("Saved".into());
    tab_until(&mut a, Region::FeedbackBanner(FeedbackBannerField::Dismiss));
}

// ── D-22: the confirm dialog restores the prior cursor on close ──────────

#[test]
fn confirm_dialog_restores_the_prior_region_and_cursor_on_cancel() {
    let mut a = expert_in_workspace();
    let rs_id = rule_set_ids(&a)[0];
    a.update(Message::SelectRuleSet(rs_id));
    tab_until(&mut a, Region::RuleSetConfig(RuleSetConfigField::Delete));
    let before = (a.focused_region, a.region_cursor.clone());

    a.update(Message::ConfirmRequest(
        crate::message::ConfirmAction::LeaveWorkspace,
    ));
    assert_eq!(
        a.focused_region,
        Some(Region::ConfirmDialog),
        "test setup sanity"
    );
    a.update(Message::ConfirmCancel);

    assert_eq!(
        (a.focused_region, a.region_cursor),
        before,
        "cancelling must put the cursor back where it was, not strand it at the tab bar"
    );
}

#[test]
fn confirm_dialog_restores_even_when_proceed_invalidated_what_it_named() {
    // Task 034, review answer 2's own safety property: a restore that
    // `Proceed` invalidated must simply not survive the next keystroke,
    // not corrupt anything in the meantime.
    let mut a = expert_in_workspace();
    let rs_id = rule_set_ids(&a)[0];
    a.update(Message::SelectRuleSet(rs_id));
    tab_until(&mut a, Region::RuleSetConfig(RuleSetConfigField::Delete));
    let before = (a.focused_region, a.region_cursor.clone());

    a.update(Message::ConfirmRequest(
        crate::message::ConfirmAction::DeleteRuleSet(rs_id),
    ));
    a.update(Message::ConfirmProceed);

    assert_eq!(
        (a.focused_region, a.region_cursor.clone()),
        before,
        "the raw restore happens regardless"
    );
    assert!(
        !Region::order_for(&a).contains(&Region::RuleSetConfig(RuleSetConfigField::Delete)),
        "the rule set this named no longer exists"
    );
    // The next keystroke self-heals: `valid_focused_region` will not find
    // the stale region in the new `order_for`.
    a.update(Message::ArrowDown);
    assert_ne!(
        a.focused_region, before.0,
        "a stale restore must not survive the next keystroke"
    );
}

// ── D-22: Escape dismisses from the same source `order_for` reads ────────

#[test]
fn escape_dismisses_the_confirm_dialog_over_the_palette_and_restores_focus() {
    let mut a = expert_in_workspace();
    tab_until(&mut a, Region::TabBar);
    let before = (a.focused_region, a.region_cursor.clone());
    a.update(Message::ToggleCommandPalette);
    a.update(Message::ConfirmRequest(
        crate::message::ConfirmAction::LeaveWorkspace,
    ));
    a.update(Message::EscapePressed);
    assert!(
        a.confirm_dialog.is_none(),
        "the dialog, not the palette, closed"
    );
    assert!(
        a.command_palette.open,
        "the palette underneath is untouched"
    );
    assert_eq!(
        (a.focused_region, a.region_cursor),
        before,
        "Escape restores the cursor exactly as Cancel does"
    );
}

#[test]
fn escape_closes_the_palette_over_the_workspace_menu_not_the_menu() {
    // The exact bug D-22 finding #3 named: `ToggleWorkspaceMenu` closes
    // the palette, but the reverse was never true, so `Escape` used to
    // dismiss the menu while the palette sat above it, reachable.
    let mut a = expert_in_workspace();
    a.workspace_menu_open = true;
    a.update(Message::ToggleCommandPalette);
    assert!(a.command_palette.open, "test setup sanity");
    a.update(Message::EscapePressed);
    assert!(!a.command_palette.open, "the palette, on top, closed");
    assert!(
        a.workspace_menu_open,
        "the menu underneath must still be open"
    );
}

#[test]
fn escape_closes_the_path_assistant_over_the_drawer_not_the_drawer() {
    let mut a = expert_in_workspace();
    a.path_assistant.open = true;
    a.update(Message::OpenValidationDrawer);
    assert!(a.drawer.is_some(), "test setup sanity: still open");
    a.update(Message::EscapePressed);
    assert!(!a.path_assistant.open, "the path assistant, on top, closed");
    assert!(
        a.drawer.is_some(),
        "the drawer underneath must still be open"
    );
}

#[test]
fn escape_closes_the_workspace_menu_over_the_drawer_not_the_drawer() {
    let mut a = expert_in_workspace();
    a.workspace_menu_open = true;
    a.update(Message::OpenValidationDrawer);
    assert!(a.drawer.is_some(), "test setup sanity: still open");
    a.update(Message::EscapePressed);
    assert!(!a.workspace_menu_open, "the menu, on top, closed");
    assert!(
        a.drawer.is_some(),
        "the drawer underneath must still be open"
    );
}

// ── D-24's confirmation half ───────────────────────────────────────────────

#[test]
fn drawer_discard_opens_the_confirm_dialog_instead_of_discarding_immediately() {
    // Task 034 (D-24's confirmation half only — the scope half, what
    // `discard_all_changes` actually discards, is explicitly not this
    // slice's to fix, per the handoff §4.2). Dirties a fallback file
    // specifically, since the mock's own default dirty item (`main.toml`)
    // is a rule set, which `discard_all_changes` never touched before this
    // slice and still does not — this test is about the *routing* fix
    // (through the confirm dialog, not direct), not the scope gap.
    let mut a = expert_in_workspace();
    let path = a.snapshot.as_ref().unwrap().fallback_files[0].path.clone();
    a.fallback_drafts.insert(
        path.clone(),
        iced::widget::text_editor::Content::with_text("{\"x\":1}"),
    );
    a.recompute_dirty();
    assert!(a.is_fallback_dirty(&path), "test setup sanity");

    a.update(Message::OpenSaveDiffDrawer);
    tab_until(&mut a, Region::Drawer(DrawerField::Discard));

    a.update(Message::EnterPressed);
    assert!(
        a.confirm_dialog.is_some(),
        "Discard must open a confirmation, not discard directly"
    );
    assert!(
        a.is_fallback_dirty(&path),
        "nothing discarded yet — only the dialog opened"
    );

    a.update(Message::ArrowDown); // to Proceed
    a.update(Message::EnterPressed);
    assert!(
        !a.is_fallback_dirty(&path),
        "Proceed actually discards the fallback draft"
    );
}

// ── Scroll visibility: N/A, with a reason, for every region kind task ────
// 034 adds that has no scrollable at all — stated plainly rather than
// substituted away, the same standard task 033 already held `FallbackField`
// to.

#[test]
fn scroll_snap_target_is_none_for_every_test_rule_field() {
    // `screens::test_rule::view` has no scrollable — a plain `column`,
    // fixed width.
    let mut a = expert_in_workspace();
    a.update(Message::TestRuleOpen);
    for field in [
        TestRuleField::Dismiss,
        TestRuleField::Method,
        TestRuleField::Path,
        TestRuleField::Headers,
        TestRuleField::Body,
        TestRuleField::Close,
        TestRuleField::Run,
    ] {
        tab_until(&mut a, Region::TestRule(field));
        assert_eq!(a.scroll_snap_target(true), None);
    }
}

#[test]
fn scroll_snap_target_is_none_for_the_top_bar_and_the_routes_empty_cta() {
    // Neither `shell::top_bar::view` nor the Routes empty-state call to
    // action sit inside any scrollable.
    let mut a = expert_in_workspace();
    for field in [
        TopBarField::Switcher,
        TopBarField::SaveAll,
        TopBarField::StartStop,
    ] {
        tab_until(&mut a, Region::TopBar(field));
        assert_eq!(a.scroll_snap_target(true), None);
    }
    tab_until(&mut a, Region::RoutesEmptyCta);
    assert_eq!(a.scroll_snap_target(true), None);
}

#[test]
fn scroll_snap_target_is_none_for_trace_activity_and_the_feedback_banner() {
    // `trace_activity_section` renders below both the WHEN/RESPOND
    // scrollables, outside either; the feedback banner sits between the
    // tab bar and the screen, also outside any scrollable.
    let mut a = expert_in_workspace();
    let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(rule_id));
    tab_until(&mut a, Region::TraceActivity(TraceActivityField::ViewAll));
    assert_eq!(a.scroll_snap_target(true), None);

    let mut b = expert_in_workspace();
    b.notice = Some("Saved".into());
    tab_until(&mut b, Region::FeedbackBanner(FeedbackBannerField::Dismiss));
    assert_eq!(b.scroll_snap_target(true), None);
}

#[test]
fn scroll_snap_target_is_none_for_the_workspace_menus_open_and_create() {
    // `Length::Shrink` on the recent-workspace list's own `scrollable` —
    // it never exceeds its content, so there is nothing to scroll into
    // view, for any of its three fields.
    let mut a = expert_in_workspace();
    a.workspace_menu_open = true;
    for field in [
        WorkspaceMenuField::Recent,
        WorkspaceMenuField::Open,
        WorkspaceMenuField::Create,
    ] {
        tab_until(&mut a, Region::WorkspaceMenu(field));
        assert_eq!(a.scroll_snap_target(true), None);
    }
}

// ── Task 035 (MK-023 slice 10): overlays the view actually draws ─────────

#[test]
fn workspace_menu_open_from_the_top_bar_does_not_trap_the_dashboard() {
    // Task 035 §2(a), driven by messages, not by setting state directly:
    // four keystrokes from a fresh workspace reach the Dashboard with
    // `workspace_menu_open` left stale `true` (`Message::GoDashboard`
    // does not clear it — only `OpenWorkspace`, a different row, does).
    // Before this slice, `topmost_overlay()` kept reporting
    // `WorkspaceMenu` there, trapping the keyboard on three invisible
    // menu items with none of the Dashboard's own controls reachable.
    let mut a = expert_in_workspace();
    tab_until(&mut a, Region::TopBar(TopBarField::Switcher));
    a.update(Message::EnterPressed); // opens the menu
    assert!(a.workspace_menu_open, "test setup sanity");
    tab_until(&mut a, Region::WorkspaceMenu(WorkspaceMenuField::Open));
    a.update(Message::EnterPressed); // "Open another workspace" → GoDashboard

    assert_eq!(a.view, AppView::Dashboard, "test setup sanity");
    assert!(
        a.workspace_menu_open,
        "test setup sanity: the flag is deliberately left stale, not cleared on transition"
    );
    assert_eq!(
        a.topmost_overlay(),
        None,
        "nothing on the Dashboard draws a workspace menu"
    );
    assert_eq!(
        Region::order_for(&a),
        &[
            Region::DashboardCreate,
            Region::DashboardSearch,
            Region::DashboardWorkspaces,
        ]
    );

    // The Dashboard's own search field is reachable and live, driven by
    // `Tab`, not by reading state directly.
    tab_until(&mut a, Region::DashboardSearch);
    a.update(Message::DashSearch("shop".into()));
    assert_eq!(a.dash_search, "shop");
}

fn first_dashboard_workspace_name(a: &App) -> String {
    crate::regions::dashboard_filtered_workspaces(a)[0]
        .name
        .clone()
}

#[test]
fn confirm_dialog_is_drawn_and_cancellable_from_the_dashboard() {
    // Task 035 §2(b): pre-existing, both input modes — a pointer click on
    // a dirty Dashboard's workspace row used to raise a confirm dialog
    // nothing drew. `App::view` now attaches it over the Dashboard too.
    let mut a = expert_in_workspace();
    assert!(
        a.requires_workspace_confirmation(),
        "test setup sanity: the mock starts dirty"
    );
    a.update(Message::GoDashboard);
    assert_eq!(a.view, AppView::Dashboard, "test setup sanity");

    let name = first_dashboard_workspace_name(&a);
    a.update(Message::OpenWorkspace(name));
    assert!(a.confirm_dialog.is_some(), "test setup sanity: raised");
    assert_eq!(a.topmost_overlay(), Some(Overlay::ConfirmDialog));
    assert_eq!(
        a.view,
        AppView::Dashboard,
        "the screen underneath is the one the user was on"
    );

    a.update(Message::EnterPressed); // Cancel is the entry position
    assert!(a.confirm_dialog.is_none());
    assert_eq!(
        a.view,
        AppView::Dashboard,
        "cancelling stays on the Dashboard"
    );
}

#[test]
fn confirm_dialog_on_the_dashboard_also_cancels_on_escape() {
    let mut a = expert_in_workspace();
    a.update(Message::GoDashboard);
    let name = first_dashboard_workspace_name(&a);
    a.update(Message::OpenWorkspace(name));
    assert!(a.confirm_dialog.is_some(), "test setup sanity");

    a.update(Message::EscapePressed);
    assert!(a.confirm_dialog.is_none());
    assert_eq!(a.view, AppView::Dashboard);
}

#[test]
fn confirm_dialog_is_drawn_and_cancellable_from_the_wizard() {
    // Task 035 §2(c): `WizardCreate` with unsaved edits raised
    // `ConfirmRequest(CreateWorkspace)` while nothing drew it — Create
    // appeared dead.
    let mut a = expert_in_workspace();
    assert!(
        a.requires_workspace_confirmation(),
        "test setup sanity: the mock starts dirty"
    );
    a.update(Message::GoWizard);
    assert_eq!(a.view, AppView::Wizard, "test setup sanity");

    a.update(Message::WizardCreate);
    assert!(a.confirm_dialog.is_some(), "test setup sanity: raised");
    assert_eq!(a.topmost_overlay(), Some(Overlay::ConfirmDialog));
    assert_eq!(
        a.view,
        AppView::Wizard,
        "the screen underneath is the Wizard"
    );

    a.update(Message::EnterPressed);
    assert!(a.confirm_dialog.is_none());
    assert_eq!(a.view, AppView::Wizard, "cancelling stays on the Wizard");
}

// ── D-26: one source per HTTP-method table ────────────────────────────────

#[test]
fn test_rule_method_buttons_agree_with_the_region_models_own_list() {
    // One assertion that the view's own list (`screens::test_rule::view`
    // now reads `regions::HTTP_METHODS` directly, the same constant named
    // here) and the region model's `len`/activation agree, so a sixth
    // method cannot be added to one alone (task 035 §5).
    let a = expert_in_workspace();
    assert_eq!(
        Region::TestRule(TestRuleField::Method).len(&a),
        crate::regions::HTTP_METHODS.len()
    );
    for (i, method) in crate::regions::HTTP_METHODS.iter().enumerate() {
        assert!(matches!(
            Region::TestRule(TestRuleField::Method)
                .activation_message(&a, &Cursor::Index(i)),
            Some(Message::TestRuleSetMethod(m)) if m == *method
        ));
    }
}

#[test]
fn rule_editor_method_buttons_agree_with_the_region_models_own_list() {
    let mut a = expert_in_workspace();
    let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(rule_id));
    let group = crate::regions::editor_method_group("Any");
    assert_eq!(Region::Editor(EditorField::Method).len(&a), group.len());
    let values = crate::regions::editor_method_group("");
    for (i, value) in values.iter().enumerate() {
        assert!(matches!(
            Region::Editor(EditorField::Method)
                .activation_message(&a, &Cursor::Index(i)),
            Some(Message::RuleSetMethod(m)) if m == *value
        ));
    }
}
