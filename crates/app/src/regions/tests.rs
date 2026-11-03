//! Boundary decision: single-responsibility — pure tests of `Region`'s own
//! methods (`order_for`, `len`, `activation_message`, `cycle_value`, and
//! every region-building helper function in `regions.rs`), independent of
//! the reducer logic that drives them. Grows with each slice that extends
//! the region model, same as `app/region_cursor.rs` grows for the
//! integration side; not split per slice, since it is one coherent
//! table-lookup surface, not one file per slice.
//!
//! Task 027 (MK-023 slice 2) started this file; see `app/region_cursor.rs`
//! for the `Tab`/arrow/`Enter` integration tests.

use super::*;
use crate::app::{App, AppView};
use crate::message::Message;
use apimokka_model::AudienceMode;

fn fresh() -> App {
    App::new().0
}

/// `order_for`'s own fixed prefix on every workspace tab (task 034): the
/// top bar's three always-present fields — the mock workspace never has a
/// reload/restart pending by default, so `ReloadOrRestart` is absent.
/// Factored out since most of this file's `order_for` assertions need it
/// prepended now.
const TOP_BAR_PREFIX: [Region; 3] = [
    Region::TopBar(TopBarField::Switcher),
    Region::TopBar(TopBarField::SaveAll),
    Region::TopBar(TopBarField::StartStop),
];

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
    // reached on that tab. Task 034: a fresh workspace has no selection at
    // all, so the centre panel's own empty-state call to action is a
    // third region here too.
    let a = expert_in_workspace();
    assert_eq!(
        a.tab,
        crate::selection::WorkspaceTab::Routes,
        "test setup sanity"
    );
    let mut expected = TOP_BAR_PREFIX.to_vec();
    expected.extend([Region::TabBar, Region::Sidebar, Region::RoutesEmptyCta]);
    assert_eq!(Region::order_for(&a), expected);
}

#[test]
fn order_for_is_the_tab_bar_and_trace_regions_on_trace_with_no_event_selected() {
    use crate::selection::WorkspaceTab;

    // Task 030: TraceOutcomeAction/TraceReplay are omitted until an event is
    // selected (§4: "exist only when rendered").
    let mut a = expert_in_workspace();
    a.update(Message::SwitchTab(WorkspaceTab::Trace));
    let mut expected = TOP_BAR_PREFIX.to_vec();
    expected.extend([
        Region::TabBar,
        Region::TraceFilter,
        Region::TracePause,
        Region::TraceClear,
        Region::TraceEvents,
    ]);
    assert_eq!(Region::order_for(&a), expected);
}

#[test]
fn order_for_in_expert_mode_includes_settings_logs_and_trace_fields_directly() {
    // Task 031 (MK-023 slice 6): Expert mode shows the Logs/Trace sections
    // unconditionally — no `AdvancedToggle` region at all, unlike Guided.
    use crate::regions::SettingsField;
    use crate::selection::WorkspaceTab;

    let mut a = expert_in_workspace();
    a.update(Message::SwitchTab(WorkspaceTab::Settings));
    let mut expected = TOP_BAR_PREFIX.to_vec();
    expected.extend([
        Region::TabBar,
        Region::Settings(SettingsField::Theme),
        Region::Settings(SettingsField::AudienceMode),
        Region::Settings(SettingsField::Locale),
        Region::Settings(SettingsField::Host),
        Region::Settings(SettingsField::Port),
        Region::Settings(SettingsField::Tls),
        Region::Settings(SettingsField::LogFile),
        Region::Settings(SettingsField::TraceEnable),
        Region::Settings(SettingsField::QueueSize),
    ]);
    assert_eq!(Region::order_for(&a), expected);
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
fn order_for_on_the_wizard_is_required_fields_then_each_sections_toggle() {
    // Task 031 §1/§2: the wizard was never assigned to a slice before this
    // one (`order_for` returned `vec![]`). Fresh wizard state starts every
    // section collapsed (`section_open: [false; 3]`), so none of their
    // bodies are regions yet — only their own toggles are.
    use crate::regions::WizardField;

    let mut a = fresh();
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    a.update(Message::GoWizard);
    assert_eq!(a.view, AppView::Wizard, "test setup sanity");
    assert_eq!(
        Region::order_for(&a),
        &[
            Region::Wizard(WizardField::Name),
            Region::Wizard(WizardField::Folder),
            Region::Wizard(WizardField::ServerToggle),
            Region::Wizard(WizardField::StarterToggle),
            Region::Wizard(WizardField::TraceToggle),
            Region::Wizard(WizardField::Cancel),
            Region::Wizard(WizardField::Create),
        ]
    );
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
    let a = fresh();
    for (i, &(_, _, mode)) in crate::screens::mode_picker::OPTIONS.iter().enumerate() {
        assert!(matches!(
            Region::ModePicker.activation_message(&a, &Cursor::Index(i)),
            Some(Message::ChooseAudienceMode(m)) if m == mode
        ));
    }
    assert!(
        Region::ModePicker
            .activation_message(&a, &Cursor::Index(99))
            .is_none()
    );
}

#[test]
fn tab_bar_activation_message_matches_its_order_table() {
    let a = expert_in_workspace();
    for (i, &(tab, _)) in TAB_BAR_ORDER.iter().enumerate() {
        assert!(matches!(
            Region::TabBar.activation_message(&a, &Cursor::Index(i)),
            Some(Message::SwitchTab(t)) if t == tab
        ));
    }
    assert!(
        Region::TabBar
            .activation_message(&a, &Cursor::Index(99))
            .is_none()
    );
}

#[test]
fn welcome_actions_activation_message_matches_its_table() {
    let a = fresh();
    assert!(matches!(
        Region::WelcomeActions.activation_message(&a, &Cursor::Index(0)),
        Some(Message::GoDashboard)
    ));
    assert!(matches!(
        Region::WelcomeActions.activation_message(&a, &Cursor::Index(1)),
        Some(Message::GoWizard)
    ));
    assert!(
        Region::WelcomeActions
            .activation_message(&a, &Cursor::Index(99))
            .is_none()
    );
}

// ── Task 028: the identity-cursored Dashboard workspace list ─────────────

#[test]
fn dashboard_create_is_one_item_that_opens_the_wizard() {
    let a = fresh();
    assert_eq!(Region::DashboardCreate.len(&a), 1);
    assert!(matches!(
        Region::DashboardCreate.activation_message(&a, &Cursor::Index(0)),
        Some(Message::GoWizard)
    ));
}

#[test]
fn dashboard_search_is_one_item_whose_enter_is_a_noop() {
    let a = fresh();
    assert_eq!(Region::DashboardSearch.len(&a), 1);
    assert!(
        Region::DashboardSearch
            .activation_message(&a, &Cursor::Index(0))
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
        Region::DashboardWorkspaces.activation_message(&a, &Cursor::WorkspaceName(name.clone())),
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

// ── Task 033 (MK-023 slice 8): the fallback editor ────────────────────────

fn guided_in_workspace() -> App {
    let mut a = fresh();
    a.update(Message::ChooseAudienceMode(AudienceMode::Guided));
    a.update(Message::OpenWorkspace("test".into()));
    a
}

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
fn fallback_fields_are_empty_until_a_fallback_file_is_selected() {
    let a = expert_in_workspace();
    assert_eq!(fallback_fields(&a), Vec::new());
}

#[test]
fn fallback_fields_are_content_status_format_revert_save_in_reading_order() {
    let mut a = expert_in_workspace();
    select_first_fallback_file(&mut a);
    assert_eq!(
        fallback_fields(&a),
        vec![
            FallbackField::Content,
            FallbackField::Status,
            FallbackField::Format,
            FallbackField::Revert,
            FallbackField::Save,
        ]
    );
}

#[test]
fn fallback_fields_defer_to_a_rule_selection_per_centre_panels_priority() {
    // `centre_panel`'s priority 1 (rule) outranks priority 2 (fallback
    // file); `SelectFileRoute` clears `selection.rule`, but the reverse is
    // not guaranteed, so `fallback_editor_context` checks it explicitly
    // rather than assuming the two stay mutually exclusive.
    let mut a = expert_in_workspace();
    select_first_fallback_file(&mut a);
    a.selection.rule = Some(a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id);
    assert_eq!(fallback_fields(&a), Vec::new());
}

#[test]
fn fallback_field_id_covers_only_the_two_text_fields() {
    assert_eq!(
        fallback_field_id(FallbackField::Content),
        Some("mk023-fallback-content".to_string())
    );
    assert_eq!(
        fallback_field_id(FallbackField::Status),
        Some("mk023-fallback-status".to_string())
    );
    assert_eq!(fallback_field_id(FallbackField::Format), None);
    assert_eq!(fallback_field_id(FallbackField::Revert), None);
    assert_eq!(fallback_field_id(FallbackField::Save), None);
}

#[test]
fn fallback_content_and_status_have_no_activation_message() {
    // Typing is the interaction for both; same precedent as every other
    // text-field region (`dashboard_search_is_one_item_whose_enter_is_a_noop`
    // above).
    let mut a = expert_in_workspace();
    select_first_fallback_file(&mut a);
    assert!(
        Region::FallbackEditor(FallbackField::Content)
            .activation_message(&a, &Cursor::Index(0))
            .is_none()
    );
    assert!(
        Region::FallbackEditor(FallbackField::Status)
            .activation_message(&a, &Cursor::Index(0))
            .is_none()
    );
}

#[test]
fn fallback_format_is_always_available_but_revert_and_save_gate_on_dirty() {
    let mut a = expert_in_workspace();
    let path = select_first_fallback_file(&mut a);

    assert!(matches!(
        Region::FallbackEditor(FallbackField::Format).activation_message(&a, &Cursor::Index(0)),
        Some(Message::FallbackFileFormat)
    ));
    assert!(
        Region::FallbackEditor(FallbackField::Revert)
            .activation_message(&a, &Cursor::Index(0))
            .is_none(),
        "a freshly-opened file is clean — same gate the Revert button's own `on_press` uses"
    );
    assert!(
        Region::FallbackEditor(FallbackField::Save)
            .activation_message(&a, &Cursor::Index(0))
            .is_none()
    );

    a.fallback_drafts.insert(
        path.clone(),
        iced::widget::text_editor::Content::with_text("{\"x\":1}"),
    );
    assert!(a.is_fallback_dirty(&path), "test setup sanity");
    assert!(matches!(
        Region::FallbackEditor(FallbackField::Revert).activation_message(&a, &Cursor::Index(0)),
        Some(Message::FallbackFileRevert)
    ));
    assert!(matches!(
        Region::FallbackEditor(FallbackField::Save).activation_message(&a, &Cursor::Index(0)),
        Some(Message::FallbackFileSave)
    ));
}

#[test]
fn order_for_includes_the_fallback_editor_fields_when_a_file_is_selected() {
    let mut a = expert_in_workspace();
    select_first_fallback_file(&mut a);
    let mut expected = TOP_BAR_PREFIX.to_vec();
    expected.extend([
        Region::TabBar,
        Region::Sidebar,
        Region::FallbackEditor(FallbackField::Content),
        Region::FallbackEditor(FallbackField::Status),
        Region::FallbackEditor(FallbackField::Format),
        Region::FallbackEditor(FallbackField::Revert),
        Region::FallbackEditor(FallbackField::Save),
    ]);
    assert_eq!(Region::order_for(&a), expected);
}

// ── Task 033: the rule-set configuration pane ─────────────────────────────

#[test]
fn rule_set_config_fields_are_empty_until_a_rule_set_is_selected_alone() {
    let a = expert_in_workspace();
    assert_eq!(rule_set_config_fields(&a), Vec::new());
}

#[test]
fn rule_set_config_fields_defer_to_a_rule_selection_per_centre_panels_priority() {
    let mut a = expert_in_workspace();
    select_first_rule_set(&mut a);
    a.selection.rule = Some(a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id);
    assert_eq!(rule_set_config_fields(&a), Vec::new());
}

#[test]
fn rule_set_config_fields_in_expert_mode_show_the_strategy_picker_directly() {
    let mut a = expert_in_workspace();
    select_first_rule_set(&mut a);
    assert_eq!(
        rule_set_config_fields(&a),
        vec![
            RuleSetConfigField::Delete,
            RuleSetConfigField::Strategy,
            RuleSetConfigField::AddRule,
        ]
    );
}

#[test]
fn rule_set_config_fields_in_guided_mode_follow_the_mores_disclosure() {
    // Mirrors Settings' own advanced disclosure exactly (task 031 §2): the
    // picker is absent entirely while collapsed, and renders *after* the
    // toggle, not before, once expanded.
    let mut a = guided_in_workspace();
    select_first_rule_set(&mut a);
    assert!(
        !a.rule_set_config_more,
        "test setup sanity: Guided starts collapsed"
    );
    assert_eq!(
        rule_set_config_fields(&a),
        vec![
            RuleSetConfigField::Delete,
            RuleSetConfigField::MoreToggle,
            RuleSetConfigField::AddRule,
        ]
    );

    a.rule_set_config_more = true;
    assert_eq!(
        rule_set_config_fields(&a),
        vec![
            RuleSetConfigField::Delete,
            RuleSetConfigField::Strategy,
            RuleSetConfigField::MoreToggle,
            RuleSetConfigField::AddRule,
        ]
    );
}

#[test]
fn strategy_is_the_fifth_value_cycler() {
    assert!(Region::RuleSetConfig(RuleSetConfigField::Strategy).is_value_cycler());
    assert!(!Region::RuleSetConfig(RuleSetConfigField::Delete).is_value_cycler());
    assert!(!Region::RuleSetConfig(RuleSetConfigField::MoreToggle).is_value_cycler());
    assert!(!Region::RuleSetConfig(RuleSetConfigField::AddRule).is_value_cycler());
}

#[test]
fn strategy_cycle_value_is_none_without_a_rule_set_in_context() {
    let a = expert_in_workspace();
    assert!(
        Region::RuleSetConfig(RuleSetConfigField::Strategy)
            .cycle_value(&a, 1)
            .is_none()
    );
}

#[test]
fn strategy_cycles_forward_through_every_option_and_saturates_at_the_last() {
    let mut a = expert_in_workspace();
    select_first_rule_set(&mut a);
    let options = apimokka_model::settings::Strategy::all();
    let start = a.snapshot.as_ref().unwrap().root_settings.strategy;
    let start_pos = options
        .iter()
        .position(|&s| s == start)
        .expect("test setup sanity");

    for pos in start_pos..options.len() - 1 {
        let Some(Message::RuleSetSetStrategy(next)) =
            Region::RuleSetConfig(RuleSetConfigField::Strategy).cycle_value(&a, 1)
        else {
            panic!("expected a RuleSetSetStrategy message at position {pos}");
        };
        assert_eq!(next, options[pos + 1]);
        a.update(Message::RuleSetSetStrategy(next));
    }

    // Saturates at the last option rather than wrapping — the same rule
    // every other region's arrows already follow.
    assert!(matches!(
        Region::RuleSetConfig(RuleSetConfigField::Strategy).cycle_value(&a, 1),
        Some(Message::RuleSetSetStrategy(last)) if last == *options.last().unwrap()
    ));
}

#[test]
fn delete_add_rule_and_more_toggle_activation_messages() {
    let mut a = expert_in_workspace();
    let rs_id = select_first_rule_set(&mut a);
    assert!(matches!(
        Region::RuleSetConfig(RuleSetConfigField::Delete)
            .activation_message(&a, &Cursor::Index(0)),
        Some(Message::DeleteRuleSet(id)) if id == rs_id
    ));
    assert!(matches!(
        Region::RuleSetConfig(RuleSetConfigField::AddRule)
            .activation_message(&a, &Cursor::Index(0)),
        Some(Message::AddRule(id)) if id == rs_id
    ));
    assert!(matches!(
        Region::RuleSetConfig(RuleSetConfigField::MoreToggle)
            .activation_message(&a, &Cursor::Index(0)),
        Some(Message::ToggleRuleSetConfigMore)
    ));
}

#[test]
fn order_for_includes_the_rule_set_config_fields_when_a_rule_set_is_selected() {
    let mut a = expert_in_workspace();
    select_first_rule_set(&mut a);
    let mut expected = TOP_BAR_PREFIX.to_vec();
    expected.extend([
        Region::TabBar,
        Region::Sidebar,
        Region::RuleSetConfig(RuleSetConfigField::Delete),
        Region::RuleSetConfig(RuleSetConfigField::Strategy),
        Region::RuleSetConfig(RuleSetConfigField::AddRule),
    ]);
    assert_eq!(Region::order_for(&a), expected);
}

// ── Task 033: the bottom drawer ───────────────────────────────────────────

#[test]
fn drawer_fields_are_empty_when_the_drawer_is_closed() {
    let a = expert_in_workspace();
    assert_eq!(a.drawer, None, "test setup sanity");
    assert_eq!(drawer_fields(&a), Vec::new());
}

#[test]
fn drawer_fields_validation_mode_lists_close_then_only_the_targeted_rows() {
    let mut a = expert_in_workspace();
    let mut seed = apimokka_model::mock::shop_api_canonical_seed();
    let targeted = seed.rule_sets[0].rules[0].id;
    for rs in &mut seed.rule_sets {
        rs.validation.issues.clear();
        for r in &mut rs.rules {
            r.validation.issues.clear();
        }
    }
    seed.diagnostics = vec![
        apimokka_model::Diagnostic {
            node_id: None,
            severity: apimokka_model::Severity::Info,
            message: "untargeted".into(),
        },
        apimokka_model::Diagnostic {
            node_id: Some(targeted),
            severity: apimokka_model::Severity::Warning,
            message: "targeted".into(),
        },
    ];
    assert!(a.install_workspace(seed));
    a.update(Message::OpenValidationDrawer);
    assert_eq!(
        drawer_fields(&a),
        vec![DrawerField::Close, DrawerField::JumpToDiagnostic(0)],
        "only the one row with a target contributes a JumpToDiagnostic region — \
         `drawer_jump_target_count`'s own scope, checked against \
         `durable_diagnostic_rows` rather than assumed"
    );
}

#[test]
fn drawer_fields_recompute_when_the_list_empties_and_when_it_grows() {
    // §6: "the drawer's list behaves when it empties and when its contents
    // change."
    let mut a = expert_in_workspace();
    let mut seed = apimokka_model::mock::shop_api_canonical_seed();
    seed.diagnostics.clear();
    for rs in &mut seed.rule_sets {
        rs.validation.issues.clear();
        for r in &mut rs.rules {
            r.validation.issues.clear();
        }
    }
    assert!(a.install_workspace(seed.clone()));
    a.update(Message::OpenValidationDrawer);
    assert_eq!(
        drawer_fields(&a),
        vec![DrawerField::Close],
        "an empty list contributes nothing beyond Close"
    );

    let rule_a = seed.rule_sets[0].rules[0].id;
    let rule_b = seed.rule_sets[0].rules[1].id;
    seed.diagnostics = vec![
        apimokka_model::Diagnostic {
            node_id: Some(rule_a),
            severity: apimokka_model::Severity::Warning,
            message: "first".into(),
        },
        apimokka_model::Diagnostic {
            node_id: Some(rule_b),
            severity: apimokka_model::Severity::Warning,
            message: "second".into(),
        },
    ];
    assert!(a.install_workspace(seed));
    assert_eq!(
        drawer_fields(&a),
        vec![
            DrawerField::Close,
            DrawerField::JumpToDiagnostic(0),
            DrawerField::JumpToDiagnostic(1),
        ],
        "the list grows by exactly the new targeted rows, in \
         `durable_diagnostic_rows`'s own reading order"
    );
}

#[test]
fn drawer_jump_to_diagnostic_activation_message_skips_untargeted_rows_by_position() {
    let mut a = expert_in_workspace();
    let mut seed = apimokka_model::mock::shop_api_canonical_seed();
    let targeted = seed.rule_sets[0].rules[0].id;
    for rs in &mut seed.rule_sets {
        rs.validation.issues.clear();
        for r in &mut rs.rules {
            r.validation.issues.clear();
        }
    }
    seed.diagnostics = vec![
        apimokka_model::Diagnostic {
            node_id: None,
            severity: apimokka_model::Severity::Info,
            message: "untargeted-first".into(),
        },
        apimokka_model::Diagnostic {
            node_id: Some(targeted),
            severity: apimokka_model::Severity::Warning,
            message: "targeted-second".into(),
        },
    ];
    assert!(a.install_workspace(seed));
    a.update(Message::OpenValidationDrawer);

    assert!(
        matches!(
            Region::Drawer(DrawerField::JumpToDiagnostic(0))
                .activation_message(&a, &Cursor::Index(0)),
            Some(Message::JumpToDiagnostic(id)) if id == targeted
        ),
        "position 0 among *targeted* rows is the second row overall — the \
         first has no target, so it does not consume a position"
    );
    assert!(
        Region::Drawer(DrawerField::JumpToDiagnostic(1))
            .activation_message(&a, &Cursor::Index(0))
            .is_none(),
        "there is only one targeted row"
    );
}

#[test]
fn drawer_fields_save_diff_mode_gates_discard_and_save_all_on_total() {
    let mut a = expert_in_workspace();
    a.update(Message::OpenSaveDiffDrawer);
    assert!(
        crate::shell::bottom_drawer::save_diff_total(&a) > 0,
        "test setup sanity: the mock starts with an unsaved rule set"
    );
    assert_eq!(
        drawer_fields(&a),
        vec![
            DrawerField::Close,
            DrawerField::Discard,
            DrawerField::SaveAll,
        ]
    );

    a.update(Message::Save);
    assert_eq!(
        crate::shell::bottom_drawer::save_diff_total(&a),
        0,
        "test setup sanity"
    );
    assert_eq!(
        drawer_fields(&a),
        vec![DrawerField::Close],
        "nothing left to discard or save"
    );
}

#[test]
fn drawer_close_discard_save_all_activation_messages() {
    let a = fresh();
    assert!(matches!(
        Region::Drawer(DrawerField::Close).activation_message(&a, &Cursor::Index(0)),
        Some(Message::CloseDrawer)
    ));
    // Task 034 (D-24's confirmation half): routed through the confirm
    // dialog rather than discarding directly.
    assert!(matches!(
        Region::Drawer(DrawerField::Discard).activation_message(&a, &Cursor::Index(0)),
        Some(Message::ConfirmRequest(
            crate::message::ConfirmAction::DiscardChanges
        ))
    ));
    assert!(matches!(
        Region::Drawer(DrawerField::SaveAll).activation_message(&a, &Cursor::Index(0)),
        Some(Message::SaveAll)
    ));
}

#[test]
fn order_for_is_the_drawer_fields_while_it_is_open() {
    let mut a = expert_in_workspace();
    a.update(Message::OpenSaveDiffDrawer);
    assert_eq!(
        Region::order_for(&a),
        &[
            Region::Drawer(DrawerField::Close),
            Region::Drawer(DrawerField::Discard),
            Region::Drawer(DrawerField::SaveAll),
        ]
    );
}

// ── Task 033: the confirm dialog ──────────────────────────────────────────

#[test]
fn confirm_dialog_is_a_two_item_region_cancel_then_proceed() {
    let a = fresh();
    assert_eq!(Region::ConfirmDialog.len(&a), 2);
    assert!(matches!(
        Region::ConfirmDialog.activation_message(&a, &Cursor::Index(0)),
        Some(Message::ConfirmCancel)
    ));
    assert!(matches!(
        Region::ConfirmDialog.activation_message(&a, &Cursor::Index(1)),
        Some(Message::ConfirmProceed)
    ));
    assert!(
        Region::ConfirmDialog
            .activation_message(&a, &Cursor::Index(99))
            .is_none()
    );
    assert_eq!(
        Region::ConfirmDialog.active_index(&a),
        None,
        "`Cancel` (index 0) is the entry position via `enter_region`'s own \
         fallback, not an active-item highlight"
    );
}

#[test]
fn order_for_is_only_the_confirm_dialog_while_it_is_open() {
    let mut a = expert_in_workspace();
    a.update(Message::ConfirmRequest(
        crate::message::ConfirmAction::LeaveWorkspace,
    ));
    assert_eq!(Region::order_for(&a), &[Region::ConfirmDialog]);
}

// ── Task 033: the census's one-line fix for the path assistant's leak ─────

#[test]
fn order_for_is_the_path_assistant_fields_while_it_is_open() {
    // Task 034: the path assistant is now a real region (named in MK-023's
    // own acceptance-clause inventory), overriding every region behind it
    // the same shape every other dialog-shaped overlay already uses —
    // superseding task 033's own leak-prevention-only `vec![]` fix.
    let mut a = expert_in_workspace();
    a.path_assistant.open = true;
    assert_eq!(
        Region::order_for(&a),
        &[
            Region::PathAssistant(PathAssistantField::Close),
            Region::PathAssistant(PathAssistantField::Json),
            Region::PathAssistant(PathAssistantField::Cancel),
            Region::PathAssistant(PathAssistantField::Insert),
        ],
        "no leaves are parsed from an empty JSON input"
    );
}

#[test]
fn path_assistant_fields_add_one_use_leaf_per_parsed_leaf() {
    let mut a = expert_in_workspace();
    a.path_assistant.open = true;
    a.path_assistant.json_input = r#"{"id":123,"name":"x"}"#.into();
    assert_eq!(
        path_assistant_fields(&a),
        vec![
            PathAssistantField::Close,
            PathAssistantField::Json,
            PathAssistantField::UseLeaf(0),
            PathAssistantField::UseLeaf(1),
            PathAssistantField::Cancel,
            PathAssistantField::Insert,
        ]
    );
}

#[test]
fn path_assistant_use_leaf_activation_re_derives_the_live_leaf_list() {
    let mut a = expert_in_workspace();
    a.path_assistant.open = true;
    a.path_assistant.json_input = r#"{"id":123,"name":"x"}"#.into();
    assert!(matches!(
        Region::PathAssistant(PathAssistantField::UseLeaf(0))
            .activation_message(&a, &Cursor::Index(0)),
        Some(Message::PathAssistantSelectPath(p)) if p == "id"
    ));
    assert!(
        Region::PathAssistant(PathAssistantField::UseLeaf(2))
            .activation_message(&a, &Cursor::Index(0))
            .is_none(),
        "only two leaves exist"
    );
}

#[test]
fn path_assistant_close_json_cancel_insert_activation_messages() {
    let a = fresh();
    assert!(matches!(
        Region::PathAssistant(PathAssistantField::Close).activation_message(&a, &Cursor::Index(0)),
        Some(Message::PathAssistantClose)
    ));
    assert!(
        Region::PathAssistant(PathAssistantField::Json)
            .activation_message(&a, &Cursor::Index(0))
            .is_none(),
        "typing is the interaction"
    );
    assert!(matches!(
        Region::PathAssistant(PathAssistantField::Cancel).activation_message(&a, &Cursor::Index(0)),
        Some(Message::PathAssistantClose)
    ));
    assert!(matches!(
        Region::PathAssistant(PathAssistantField::Insert).activation_message(&a, &Cursor::Index(0)),
        Some(Message::PathAssistantInsert)
    ));
}

// ── Task 034: the Test Rule dialog ────────────────────────────────────────

#[test]
fn test_rule_fields_are_empty_until_it_is_open() {
    let a = expert_in_workspace();
    assert_eq!(test_rule_fields(&a), Vec::new());
}

#[test]
fn test_rule_fields_are_in_reading_order_when_open() {
    let mut a = expert_in_workspace();
    a.update(Message::TestRuleOpen);
    assert_eq!(
        test_rule_fields(&a),
        vec![
            TestRuleField::Dismiss,
            TestRuleField::Method,
            TestRuleField::Path,
            TestRuleField::Headers,
            TestRuleField::Body,
            TestRuleField::Close,
            TestRuleField::Run,
        ]
    );
}

#[test]
fn test_rule_method_active_index_resumes_the_current_method() {
    let mut a = expert_in_workspace();
    a.update(Message::TestRuleOpen);
    a.test_rule.method = "PUT".into();
    assert_eq!(
        Region::TestRule(TestRuleField::Method).active_index(&a),
        Some(2)
    );
}

#[test]
fn test_rule_text_fields_dispatch_run_instead_of_none() {
    // The one deliberate exception to "a text-field region has no
    // activation" (task 034 §3) — `Enter` must still run the dry run from
    // these three fields, task 026's own oldest behaviour in this cycle.
    let a = fresh();
    for field in [
        TestRuleField::Path,
        TestRuleField::Headers,
        TestRuleField::Body,
    ] {
        assert!(matches!(
            Region::TestRule(field).activation_message(&a, &Cursor::Index(0)),
            Some(Message::TestRuleRun)
        ));
    }
}

#[test]
fn test_rule_dismiss_close_run_and_method_activation_messages() {
    let a = fresh();
    assert!(matches!(
        Region::TestRule(TestRuleField::Dismiss).activation_message(&a, &Cursor::Index(0)),
        Some(Message::TestRuleClose)
    ));
    assert!(matches!(
        Region::TestRule(TestRuleField::Close).activation_message(&a, &Cursor::Index(0)),
        Some(Message::TestRuleClose)
    ));
    assert!(matches!(
        Region::TestRule(TestRuleField::Run).activation_message(&a, &Cursor::Index(0)),
        Some(Message::TestRuleRun)
    ));
    assert!(matches!(
        Region::TestRule(TestRuleField::Method).activation_message(&a, &Cursor::Index(1)),
        Some(Message::TestRuleSetMethod(m)) if m == "POST"
    ));
}

#[test]
fn order_for_is_the_test_rule_fields_while_it_is_open() {
    let mut a = expert_in_workspace();
    a.update(Message::TestRuleOpen);
    assert_eq!(
        Region::order_for(&a),
        test_rule_fields(&a)
            .into_iter()
            .map(Region::TestRule)
            .collect::<Vec<_>>()
    );
}

// ── Task 034: the workspace menu ──────────────────────────────────────────

#[test]
fn workspace_menu_fields_are_empty_until_it_is_open() {
    let a = expert_in_workspace();
    assert_eq!(workspace_menu_fields(&a), Vec::new());
}

#[test]
fn workspace_menu_fields_are_recent_open_create_when_open() {
    let mut a = expert_in_workspace();
    a.workspace_menu_open = true;
    assert!(
        !apimokka_model::mock::recent_workspaces().is_empty(),
        "test setup sanity: the mock recent-workspace list is non-empty"
    );
    assert_eq!(
        workspace_menu_fields(&a),
        vec![
            WorkspaceMenuField::Recent,
            WorkspaceMenuField::Open,
            WorkspaceMenuField::Create,
        ]
    );
}

#[test]
fn workspace_menu_recent_activation_message_opens_the_named_workspace() {
    let mut a = expert_in_workspace();
    a.workspace_menu_open = true;
    let name = apimokka_model::mock::recent_workspaces()[0].name.clone();
    assert!(matches!(
        Region::WorkspaceMenu(WorkspaceMenuField::Recent)
            .activation_message(&a, &Cursor::Index(0)),
        Some(Message::OpenWorkspace(n)) if n == name
    ));
}

#[test]
fn workspace_menu_open_and_create_activation_messages() {
    let a = fresh();
    assert!(matches!(
        Region::WorkspaceMenu(WorkspaceMenuField::Open).activation_message(&a, &Cursor::Index(0)),
        Some(Message::GoDashboard)
    ));
    assert!(matches!(
        Region::WorkspaceMenu(WorkspaceMenuField::Create).activation_message(&a, &Cursor::Index(0)),
        Some(Message::GoWizard)
    ));
}

#[test]
fn order_for_is_the_workspace_menu_fields_while_it_is_open() {
    let mut a = expert_in_workspace();
    a.workspace_menu_open = true;
    assert_eq!(
        Region::order_for(&a),
        &[
            Region::WorkspaceMenu(WorkspaceMenuField::Recent),
            Region::WorkspaceMenu(WorkspaceMenuField::Open),
            Region::WorkspaceMenu(WorkspaceMenuField::Create),
        ]
    );
}

// ── Task 034: the rule editor's "recent trace activity" footer ───────────

#[test]
fn trace_activity_fields_are_empty_until_a_rule_is_selected() {
    let a = expert_in_workspace();
    assert_eq!(trace_activity_fields(&a), Vec::new());
}

#[test]
fn trace_activity_fields_are_view_all_only_with_no_matching_events() {
    let mut a = expert_in_workspace();
    let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(rule_id));
    a.trace.clear();
    assert_eq!(trace_activity_fields(&a), vec![TraceActivityField::ViewAll]);
}

#[test]
fn trace_activity_view_all_activation_message() {
    let a = fresh();
    assert!(matches!(
        Region::TraceActivity(TraceActivityField::ViewAll)
            .activation_message(&a, &Cursor::Index(0)),
        Some(Message::ViewAllInTrace)
    ));
}

// ── Task 034: the feedback banner ─────────────────────────────────────────

#[test]
fn feedback_banner_fields_are_empty_with_nothing_to_show() {
    let a = fresh();
    assert_eq!(feedback_banner_fields(&a), Vec::new());
}

#[test]
fn feedback_banner_error_fields_depend_on_detail_and_action_presence() {
    let mut a = fresh();
    a.last_problem = Some(apimokka_model::FriendlyProblem::new("t", "d", None));
    assert_eq!(
        feedback_banner_fields(&a),
        vec![FeedbackBannerField::Dismiss],
        "no technical detail, no action label — only Dismiss"
    );

    a.last_problem = Some(apimokka_model::FriendlyProblem::new(
        "t",
        "d",
        Some("Fix it"),
    ));
    assert_eq!(
        feedback_banner_fields(&a),
        vec![FeedbackBannerField::Action, FeedbackBannerField::Dismiss]
    );

    let mut p = apimokka_model::FriendlyProblem::new("t", "d", Some("Fix it"));
    p.technical_detail = Some("errno 13".into());
    a.last_problem = Some(p);
    assert_eq!(
        feedback_banner_fields(&a),
        vec![
            FeedbackBannerField::DetailsToggle,
            FeedbackBannerField::Action,
            FeedbackBannerField::Dismiss,
        ]
    );
}

#[test]
fn feedback_banner_dismiss_re_derives_the_message_from_the_active_banner() {
    let mut a = fresh();
    a.last_problem = Some(apimokka_model::FriendlyProblem::new("t", "d", None));
    assert!(matches!(
        Region::FeedbackBanner(FeedbackBannerField::Dismiss)
            .activation_message(&a, &Cursor::Index(0)),
        Some(Message::DismissProblem)
    ));

    a.last_problem = None;
    a.notice = Some("saved".into());
    assert!(matches!(
        Region::FeedbackBanner(FeedbackBannerField::Dismiss)
            .activation_message(&a, &Cursor::Index(0)),
        Some(Message::DismissNotice)
    ));
}

// ── Task 034: the top bar ─────────────────────────────────────────────────

#[test]
fn top_bar_fields_are_empty_without_an_open_workspace() {
    let a = fresh();
    assert_eq!(top_bar_fields(&a), Vec::new());
}

#[test]
fn top_bar_fields_omit_reload_or_restart_when_neither_is_available() {
    let a = expert_in_workspace();
    assert!(
        crate::shell::top_bar::reload_or_restart(&a).is_none(),
        "test setup sanity"
    );
    assert_eq!(
        top_bar_fields(&a),
        vec![
            TopBarField::Switcher,
            TopBarField::SaveAll,
            TopBarField::StartStop
        ]
    );
}

#[test]
fn top_bar_save_all_gates_on_dirty_count_the_same_as_the_button() {
    let mut a = expert_in_workspace();
    let has_cursor = Region::TopBar(TopBarField::SaveAll);
    if a.dirty_count == 0 {
        assert!(
            has_cursor
                .activation_message(&a, &Cursor::Index(0))
                .is_none()
        );
        a.dirty_count = 1;
    }
    assert!(matches!(
        has_cursor.activation_message(&a, &Cursor::Index(0)),
        Some(Message::Save)
    ));
}

#[test]
fn top_bar_switcher_activation_message() {
    let a = fresh();
    assert!(matches!(
        Region::TopBar(TopBarField::Switcher).activation_message(&a, &Cursor::Index(0)),
        Some(Message::ToggleWorkspaceMenu)
    ));
}

// ── Task 034 (D-22): one source for overlay precedence ───────────────────

#[test]
fn topmost_overlay_ranks_mode_picker_above_everything_even_a_stray_confirm_dialog() {
    // Direct state construction, not a reachable sequence: confirm_dialog
    // cannot normally be `Some` before an audience mode is chosen, but the
    // ranking itself must hold unconditionally, not merely "whenever the
    // app happens to reach this combination".
    let mut a = fresh();
    a.confirm_dialog = Some(crate::app::ConfirmDialogState {
        action: crate::message::ConfirmAction::LeaveWorkspace,
        title: apimokka_i18n::Key::ConfirmSwitchWorkspace,
        body: apimokka_i18n::Key::ConfirmSwitchWorkspaceBody,
        restore_region: None,
        restore_cursor: None,
    });
    assert_eq!(a.audience_mode, None, "test setup sanity");
    assert_eq!(Region::order_for(&a), &[Region::ModePicker]);
}

#[test]
fn topmost_overlay_ranks_confirm_dialog_above_the_palette() {
    // A real, reachable pair: `ConfirmRequest` can fire while the palette
    // is open (D-7's own closing logic is for other dialogs, not this
    // one).
    let mut a = expert_in_workspace();
    a.update(Message::ToggleCommandPalette);
    assert!(a.command_palette.open, "test setup sanity");
    a.update(Message::ConfirmRequest(
        crate::message::ConfirmAction::LeaveWorkspace,
    ));
    assert_eq!(Region::order_for(&a), &[Region::ConfirmDialog]);
}

#[test]
fn topmost_overlay_ranks_the_palette_above_test_rule() {
    // A real, reachable pair: `Ctrl+K` is a global accelerator gated on
    // nothing to do with Test Rule.
    let mut a = expert_in_workspace();
    a.update(Message::TestRuleOpen);
    assert!(a.test_rule.open, "test setup sanity");
    a.update(Message::ToggleCommandPalette);
    assert!(a.command_palette.open, "test setup sanity");
    assert_eq!(Region::order_for(&a), Vec::<Region>::new());
}

#[test]
fn topmost_overlay_ranks_the_palette_above_the_path_assistant() {
    // Same reasoning as the Test Rule pair above — the exact repro the
    // review used for D-22's own drawer/path-assistant finding starts this
    // way.
    let mut a = expert_in_workspace();
    a.path_assistant.open = true;
    a.update(Message::ToggleCommandPalette);
    assert!(a.command_palette.open, "test setup sanity");
    assert_eq!(Region::order_for(&a), Vec::<Region>::new());
}

#[test]
fn topmost_overlay_ranks_the_path_assistant_above_the_drawer() {
    // The exact pair D-22 was filed for: both open at once, keyboard must
    // drive the one actually on top.
    let mut a = expert_in_workspace();
    a.path_assistant.open = true;
    a.update(Message::OpenValidationDrawer);
    assert!(a.drawer.is_some(), "test setup sanity: still open");
    assert_eq!(
        Region::order_for(&a),
        path_assistant_fields(&a)
            .into_iter()
            .map(Region::PathAssistant)
            .collect::<Vec<_>>()
    );
}

#[test]
fn topmost_overlay_ranks_the_workspace_menu_above_the_drawer() {
    // A real, reachable pair: opening the drawer from the palette closes
    // the palette but touches nothing else, so a workspace menu left open
    // underneath survives it.
    let mut a = expert_in_workspace();
    a.workspace_menu_open = true;
    a.update(Message::OpenValidationDrawer);
    assert!(a.drawer.is_some(), "test setup sanity: still open");
    assert_eq!(
        Region::order_for(&a),
        &[
            Region::WorkspaceMenu(WorkspaceMenuField::Recent),
            Region::WorkspaceMenu(WorkspaceMenuField::Open),
            Region::WorkspaceMenu(WorkspaceMenuField::Create),
        ]
    );
}

// Pairs ruled out rather than built, per task 034 §2's own "no need to
// manufacture combinations the application cannot reach" allowance:
//
// - **Test Rule vs the path assistant.** Each one's own region override
//   makes `order_for` return only its own fields while it is open, so
//   neither's own trigger (`EditorField::TestRule`/`BodyPathAssistant`) is
//   `Tab`-reachable while the other is open; both live behind the
//   rule-editor region, which neither overlay leaves reachable. No
//   keyboard path reaches this combination.
// - **Test Rule vs the workspace menu**, and **the path assistant vs the
//   workspace menu.** Same shape: `shell/view.rs`'s menu backdrop
//   (`on_close_menus`) catches any click outside the menu, and the menu's
//   own region override blocks `Tab` from reaching anything else while it
//   is open — structurally, not just "untested".
