//! Smoke + lifecycle tests. No iced_test: these exercise the pure update
//! reducer and the MK-038 two-buffer lifecycle, plus a view build smoke
//! test. They pin the invariants we fixed by hand across 0.6.x.
use super::*;
use crate::message::Message;
use crate::selection::WorkspaceTab;
use iced::widget::text_editor::Content;

fn fresh() -> App {
    // MK-046: App now starts at Welcome with no snapshot.
    // Tests that exercise workspace features call this helper, which
    // sets mode and loads the workspace so the snapshot is available.
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    // Load mock workspace and navigate to the Routes workbench.
    a.update(Message::OpenWorkspace("test".into()));
    a
}

/// An app at first launch, before the audience mode is chosen.
fn first_launch() -> App {
    App::new().0
}

fn first_fallback_path(a: &App) -> String {
    a.snapshot.as_ref().unwrap().fallback_files[0].path.clone()
}

// ── Selection / accordion invariants ──────────────────────────────────

#[test]
fn workspace_install_starts_with_no_unrelated_route_selection() {
    let a = fresh();
    assert_eq!(a.selection, RouteSelection::default());
    assert_eq!(a.rule_set_open, None);
}

#[test]
fn select_file_route_clears_rule_set() {
    // Regression: a stale rule_set selection used to make the rule-set
    // config view hijack the centre panel instead of the file editor.
    let mut a = fresh();
    let first_set = a.snapshot.as_ref().unwrap().rule_sets[0].id;
    a.update(Message::SelectRuleSet(first_set));
    assert!(a.selection.rule_set.is_some());
    let path = first_fallback_path(&a);
    a.update(Message::SelectFileRoute(path.clone()));
    assert_eq!(a.selection.file_route.as_deref(), Some(path.as_str()));
    assert!(
        a.selection.rule_set.is_none(),
        "file selection must clear rule_set"
    );
    assert!(a.selection.rule.is_none());
}

#[test]
fn select_script_clears_rule_set() {
    let mut a = fresh();
    let snap = a.snapshot.as_ref().unwrap();
    if let Some(s) = snap.middleware_scripts.first() {
        let path = s.path.clone();
        a.update(Message::SelectScript(path.clone()));
        assert_eq!(a.selection.script.as_deref(), Some(path.as_str()));
        assert!(a.selection.rule_set.is_none());
    }
}

#[test]
fn select_rule_set_is_single_open_accordion() {
    let mut a = fresh();
    let ids: Vec<_> = a
        .snapshot
        .as_ref()
        .unwrap()
        .rule_sets
        .iter()
        .map(|rs| rs.id)
        .collect();
    if ids.len() > 1 {
        a.update(Message::SelectRuleSet(ids[1]));
        assert_eq!(
            a.rule_set_open,
            Some(ids[1]),
            "selected set becomes the open one"
        );
        assert_eq!(a.selection.rule_set, Some(ids[1]));
        assert!(a.selection.rule.is_none());
    }
}

#[test]
fn toggle_sidebar_sections() {
    let mut a = fresh();
    assert!(!a.fallback_section_open);
    a.update(Message::ToggleFallbackSection);
    assert!(a.fallback_section_open);
    a.update(Message::ToggleMiddlewareSection);
    assert!(a.middleware_section_open);
}

// ── MK-038 fallback file lifecycle ────────────────────────────────────

#[test]
fn fallback_dirty_then_save_clean() {
    let mut a = fresh();
    let path = first_fallback_path(&a);
    a.update(Message::SelectFileRoute(path.clone()));
    assert!(!a.is_fallback_dirty(&path), "freshly opened file is clean");

    // Simulate an edit by replacing the draft buffer.
    a.fallback_drafts
        .insert(path.clone(), Content::with_text("{\"x\":1}"));
    assert!(a.is_fallback_dirty(&path), "modified draft is dirty");

    a.update(Message::FallbackFileSave);
    assert!(!a.is_fallback_dirty(&path), "save commits draft → clean");
}

#[test]
fn fallback_json_validity_predicate() {
    let mut a = fresh();
    let path = first_fallback_path(&a);
    a.update(Message::SelectFileRoute(path.clone()));

    a.fallback_drafts
        .insert(path.clone(), Content::with_text("{not valid"));
    assert!(!a.fallback_json_valid(&path), "broken JSON is invalid");

    a.fallback_drafts
        .insert(path.clone(), Content::with_text("{\"ok\":true}"));
    assert!(a.fallback_json_valid(&path), "well-formed JSON is valid");
}

#[test]
fn rule_edit_does_not_commit_fallback_drafts() {
    // The load-bearing separation: editing a rule must never silently
    // commit a dirty fallback file draft.
    let mut a = fresh();
    let path = first_fallback_path(&a);
    a.update(Message::SelectFileRoute(path.clone()));
    a.fallback_drafts
        .insert(path.clone(), Content::with_text("{\"edited\":1}"));
    assert!(a.is_fallback_dirty(&path));

    let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(rule_id));
    a.update(Message::RuleSetUrlPath("/dirty-rule".into()));
    assert!(
        a.is_fallback_dirty(&path),
        "a rule edit must not commit fallback file drafts"
    );
    assert!(
        !a.snapshot
            .as_ref()
            .unwrap()
            .latest()
            .dirty_files()
            .is_empty()
    );
}

#[test]
fn global_save_commits_fallback_drafts() {
    let mut a = fresh();
    let path = first_fallback_path(&a);
    a.update(Message::SelectFileRoute(path.clone()));
    a.fallback_drafts
        .insert(path.clone(), Content::with_text("{\"edited\":2}"));
    assert!(a.is_fallback_dirty(&path));

    a.update(Message::Save); // global Save
    assert!(
        !a.is_fallback_dirty(&path),
        "global save commits all drafts"
    );
}

// ── View build smoke tests (no rendering, just tree construction) ──────

#[test]
fn screen_views_build_without_panic() {
    for tab in [
        WorkspaceTab::Routes,
        WorkspaceTab::Trace,
        WorkspaceTab::Settings,
    ] {
        let mut a = fresh();
        a.tab = tab;
        // Element is built and dropped — catches view-construction panics.
        let _ = match tab {
            WorkspaceTab::Routes => crate::screens::routes::view(&a),
            WorkspaceTab::Trace => crate::screens::trace::view(&a),
            WorkspaceTab::Settings => crate::screens::settings::view(&a),
        };
    }
}

#[test]
fn routes_view_builds_for_each_selection() {
    // Each centre-panel branch (rule / file / script / rule-set config)
    // must build without panicking.
    let mut a = fresh();
    let snap = a.snapshot.as_ref().unwrap();
    let rule_id = snap.rule_sets[0].rules[0].id;
    let rs_id = snap.rule_sets[0].id;
    let file = snap.fallback_files[0].path.clone();

    a.update(Message::SelectRule(rule_id));
    let _ = crate::screens::routes::view(&a);

    a.update(Message::SelectRuleSet(rs_id)); // rule set config (no rule)
    let _ = crate::screens::routes::view(&a);

    a.update(Message::SelectFileRoute(file));
    let _ = crate::screens::routes::view(&a);
}

// ── MK-039: non-modal undo + feedback ─────────────────────────────────

#[test]
fn delete_rule_is_reversible_without_dialog() {
    let mut a = fresh();
    let (rs_id, rule_id, before) = {
        let snap = a.snapshot.as_ref().unwrap();
        let rs = &snap.rule_sets[0];
        (rs.id, rs.rules[0].id, rs.rules.len())
    };

    a.update(Message::DeleteRule(rule_id));
    // No confirm dialog for this low-risk action.
    assert!(
        a.confirm_dialog.is_none(),
        "delete rule must not open a dialog"
    );
    // Rule is gone and an undo is offered.
    let after = a
        .snapshot
        .as_ref()
        .unwrap()
        .rule_sets
        .iter()
        .find(|rs| rs.id == rs_id)
        .unwrap()
        .rules
        .len();
    assert_eq!(after, before - 1);
    assert!(!a.undo_stack().is_empty(), "an undo entry must be offered");

    // Undo restores it at the same index.
    a.update(Message::UndoLast);
    let restored = a
        .snapshot
        .as_ref()
        .unwrap()
        .rule_sets
        .iter()
        .find(|rs| rs.id == rs_id)
        .unwrap()
        .rules
        .len();
    assert_eq!(restored, before);
    assert!(a.undo_stack().is_empty(), "undo stack is empty after use");
}

#[test]
fn save_sets_a_success_notice() {
    let mut a = fresh();
    let path = first_fallback_path(&a);
    a.update(Message::SelectFileRoute(path.clone()));
    a.fallback_drafts.insert(
        path,
        iced::widget::text_editor::Content::with_text("{\"a\":1}"),
    );
    a.update(Message::Save);
    assert!(a.notice.is_some(), "save shows a success notice");
    a.update(Message::DismissNotice);
    assert!(a.notice.is_none() && a.undo_stack().is_empty());
}

#[test]
fn problem_action_routes_to_settings() {
    let mut a = fresh();
    a.last_problem = Some(apimokka_model::FriendlyProblem::port_in_use(8080));
    a.update(Message::ProblemAction);
    assert_eq!(a.tab, crate::selection::WorkspaceTab::Settings);
    assert!(a.last_problem.is_none(), "problem cleared after action");
}

#[test]
fn body_size_meets_comfort_floor() {
    // MK-039 comfort: body text is at least 16 px.
    assert!(crate::theme::size::BODY >= 16.0);
    assert!(crate::theme::touch::COMFORTABLE >= 52.0);
}

// ── MK-040: audience modes ────────────────────────────────────────────

#[test]
fn first_launch_has_no_mode_then_choice_persists() {
    use apimokka_model::AudienceMode;
    let mut a = first_launch();
    assert!(a.audience_mode.is_none(), "first launch shows the picker");

    a.update(Message::ChooseAudienceMode(AudienceMode::Guided));
    assert_eq!(a.audience_mode, Some(AudienceMode::Guided));
    // The picker is gated on audience_mode being None, so a Some value
    // means it will not show again.
    assert!(a.audience_mode.is_some());
}

#[test]
fn guided_shows_scaffolding_expert_does_not() {
    use apimokka_model::AudienceMode;
    let mut a = first_launch();
    a.update(Message::ChooseAudienceMode(AudienceMode::Guided));
    assert!(a.shows_scaffolding());
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    assert!(!a.shows_scaffolding());
}

#[test]
fn choosing_expert_expands_problem_details_by_default() {
    use apimokka_model::AudienceMode;
    let mut a = first_launch();
    a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
    assert!(
        a.show_problem_details,
        "Expert expands technical detail inline"
    );
    a.update(Message::ChooseAudienceMode(AudienceMode::Guided));
    assert!(!a.show_problem_details, "Guided collapses technical detail");
    // And it can be toggled regardless of mode.
    a.update(Message::ToggleProblemDetails);
    assert!(a.show_problem_details);
}

#[test]
fn vocabulary_is_identical_between_modes() {
    // The core MK-040 guarantee: switching mode never changes a domain
    // label. We sample the field/card titles that carry hints.
    use apimokka_i18n::Key;
    let mut a = first_launch();
    let keys = [
        Key::UrlPathCardTitle,
        Key::MethodCardTitle,
        Key::HeadersCardTitle,
        Key::BodyCardTitle,
    ];
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Guided,
    ));
    let guided: Vec<&str> = keys.iter().map(|k| a.t(*k)).collect();
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    let expert: Vec<&str> = keys.iter().map(|k| a.t(*k)).collect();
    assert_eq!(
        guided, expert,
        "domain vocabulary must not change with mode"
    );
}

#[test]
fn mode_picker_view_builds() {
    let a = first_launch();
    let _ = crate::screens::mode_picker::view(&a);
}

#[test]
fn error_banner_builds_in_both_modes() {
    // The banner renders technical detail inline (Expert) or behind a
    // toggle (Guided); both must build.
    for mode in apimokka_model::AudienceMode::all() {
        let mut a = first_launch();
        a.update(Message::ChooseAudienceMode(mode));
        a.last_problem = Some(apimokka_model::FriendlyProblem::port_in_use(8080));
        let _ = crate::shell::view::view(&a);
    }
}
