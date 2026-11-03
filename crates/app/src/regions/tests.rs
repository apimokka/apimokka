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
fn order_for_is_the_tab_bar_and_trace_regions_on_trace_with_no_event_selected() {
    use crate::selection::WorkspaceTab;

    // Task 030: TraceOutcomeAction/TraceReplay are omitted until an event is
    // selected (§4: "exist only when rendered").
    let mut a = expert_in_workspace();
    a.update(Message::SwitchTab(WorkspaceTab::Trace));
    assert_eq!(
        Region::order_for(&a),
        &[
            Region::TabBar,
            Region::TraceFilter,
            Region::TracePause,
            Region::TraceClear,
            Region::TraceEvents,
        ]
    );
}

#[test]
fn order_for_in_expert_mode_includes_settings_logs_and_trace_fields_directly() {
    // Task 031 (MK-023 slice 6): Expert mode shows the Logs/Trace sections
    // unconditionally — no `AdvancedToggle` region at all, unlike Guided.
    use crate::regions::SettingsField;
    use crate::selection::WorkspaceTab;

    let mut a = expert_in_workspace();
    a.update(Message::SwitchTab(WorkspaceTab::Settings));
    assert_eq!(
        Region::order_for(&a),
        &[
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
        ]
    );
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
    assert_eq!(
        Region::order_for(&a),
        &[
            Region::TabBar,
            Region::Sidebar,
            Region::FallbackEditor(FallbackField::Content),
            Region::FallbackEditor(FallbackField::Status),
            Region::FallbackEditor(FallbackField::Format),
            Region::FallbackEditor(FallbackField::Revert),
            Region::FallbackEditor(FallbackField::Save),
        ]
    );
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
    assert_eq!(
        Region::order_for(&a),
        &[
            Region::TabBar,
            Region::Sidebar,
            Region::RuleSetConfig(RuleSetConfigField::Delete),
            Region::RuleSetConfig(RuleSetConfigField::Strategy),
            Region::RuleSetConfig(RuleSetConfigField::AddRule),
        ]
    );
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
    assert!(matches!(
        Region::Drawer(DrawerField::Discard).activation_message(&a, &Cursor::Index(0)),
        Some(Message::DiscardChanges)
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
fn order_for_is_empty_while_the_path_assistant_is_open() {
    // The path assistant paints over the screen behind it (`shell::view`'s
    // own `Dialog` compositing) but has no `Region` of its own and was
    // never in `App::blocks_region_navigation` — without this, `Tab`/
    // arrow/`Enter` would silently act on the hidden background screen's
    // own regions instead.
    let mut a = expert_in_workspace();
    a.path_assistant.open = true;
    assert_eq!(Region::order_for(&a), Vec::<Region>::new());
}
