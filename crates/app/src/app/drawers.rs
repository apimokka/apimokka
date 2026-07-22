use super::*;
use crate::message::Message;
use crate::selection::DrawerMode;

fn expert() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::OpenWorkspace("test".into()));
    a
}

// ── JumpToRule closes the drawer ────────────────────────────────────

#[test]
fn jump_to_rule_closes_drawer() {
    let mut a = expert();
    a.drawer = Some(DrawerMode::Validation);
    let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::JumpToRule(rule_id));
    assert!(a.drawer.is_none(), "JumpToRule must close the drawer");
    assert_eq!(a.tab, crate::selection::WorkspaceTab::Routes);
    assert_eq!(a.selection.rule, Some(rule_id));
}

// ── AddRuleFromPalette ──────────────────────────────────────────────

#[test]
fn add_rule_from_palette_closes_palette_and_navigates() {
    let mut a = expert();
    a.command_palette.open = true;
    a.tab = crate::selection::WorkspaceTab::Trace;
    a.update(Message::AddRuleFromPalette);
    assert!(!a.command_palette.open, "palette should close");
    assert_eq!(a.tab, crate::selection::WorkspaceTab::Routes);
    // The first rule set is selected (accordion opened).
    assert!(
        a.selection.rule_set.is_some(),
        "a rule set should be selected/opened after AddRuleFromPalette"
    );
}

// ── Drawer view smoke tests ─────────────────────────────────────────

#[test]
fn validation_drawer_builds_with_issues_and_clean() {
    // Mock has one rule set with validation issues, one without.
    let mut a = expert();
    a.drawer = Some(DrawerMode::Validation);
    let _ = crate::shell::view::view(&a); // should not panic
}

#[test]
fn save_diff_drawer_builds_with_dirty_and_clean() {
    let mut a = expert();
    a.drawer = Some(DrawerMode::SaveDiff);
    // Snapshot already has main.toml as dirty in the mock.
    let _ = crate::shell::view::view(&a);
}

#[test]
fn save_diff_drawer_builds_with_no_changes() {
    let mut a = expert();
    a.drawer = Some(DrawerMode::SaveDiff);
    a.update(Message::Save);
    let _ = crate::shell::view::view(&a);
}

#[test]
fn validation_drawer_builds_when_all_clean() {
    let mut a = expert();
    a.drawer = Some(DrawerMode::Validation);
    let mut seed = apimokka_model::mock::shop_api_canonical_seed();
    for rs in &mut seed.rule_sets {
        for rule in &mut rs.rules {
            rule.validation.issues.clear();
        }
    }
    seed.diagnostics.clear();
    assert!(a.install_workspace(seed));
    let _ = crate::shell::view::view(&a);
}

#[test]
fn durable_rows_preserve_cross_source_order_and_ignore_transient_problem() {
    let mut a = expert();
    let mut seed = apimokka_model::mock::shop_api_canonical_seed();
    for rule_set in &mut seed.rule_sets {
        rule_set.validation.issues.clear();
        for rule in &mut rule_set.rules {
            rule.validation.issues.clear();
        }
    }
    let set = seed.rule_sets[0].id;
    let rule = seed.rule_sets[0].rules[0].id;
    let owning_file = seed.rule_sets[0].file.path.clone();
    seed.diagnostics = vec![
        apimokka_model::Diagnostic {
            node_id: None,
            severity: apimokka_model::Severity::Info,
            message: "workspace-first".into(),
        },
        apimokka_model::Diagnostic {
            node_id: Some(rule),
            severity: apimokka_model::Severity::Warning,
            message: "workspace-targeted-second".into(),
        },
    ];
    seed.rule_sets[0].validation.issues = vec![apimokka_model::ValidationIssue {
        node_id: Some(set.0),
        severity: apimokka_model::Severity::Error,
        message: "rule-set-third".into(),
        location: Some("rule-set.location".into()),
    }];
    seed.rule_sets[0].rules[0].validation.issues = vec![apimokka_model::ValidationIssue {
        node_id: None,
        severity: apimokka_model::Severity::Warning,
        message: "rule-fourth".into(),
        location: Some("rule.location".into()),
    }];
    assert!(a.install_workspace(seed));
    a.last_problem = Some(apimokka_model::FriendlyProblem::new(
        "transient",
        "must not become durable",
        None,
    ));
    a.transient_problem_kind = Some(TransientProblemKind::Operation);

    let rows = crate::shell::bottom_drawer::durable_diagnostic_rows(&a);
    assert_eq!(
        rows.iter()
            .map(|row| row.message.as_str())
            .collect::<Vec<_>>(),
        vec![
            "workspace-first",
            "workspace-targeted-second",
            "rule-set-third",
            "rule-fourth",
        ]
    );
    assert_eq!(rows[0].target, None);
    assert_eq!(rows[1].target, Some(rule));
    assert_eq!(rows[2].target, Some(set.0));
    assert_eq!(rows[3].target, Some(rule));
    assert!(rows[3].scope.starts_with(&format!("{owning_file} · ")));
    assert!(
        rows.iter()
            .all(|row| row.message != "must not become durable")
    );

    for locale in [apimokka_i18n::Locale::En, apimokka_i18n::Locale::Ja] {
        a.update(Message::ChangeLocale(locale));
        a.drawer = Some(DrawerMode::Validation);
        let _ = crate::shell::view::view(&a);
    }
}

#[test]
fn diagnostic_navigation_opens_rule_set_rule_and_condition_owner() {
    let mut a = expert();
    let (set, rule, condition) = a
        .snapshot
        .as_ref()
        .unwrap()
        .latest()
        .rules()
        .iter()
        .find_map(|canonical| {
            let condition = canonical.conditions().headers.first()?.id;
            let set = a
                .snapshot
                .as_ref()
                .unwrap()
                .find_rule(canonical.rule_id())?
                .0
                .id;
            Some((set, canonical.rule_id(), condition))
        })
        .unwrap();

    a.drawer = Some(DrawerMode::Validation);
    a.update(Message::JumpToDiagnostic(set.0));
    assert_eq!(a.selection.rule_set, Some(set));
    assert_eq!(a.selection.rule, None);

    a.drawer = Some(DrawerMode::Validation);
    a.update(Message::JumpToDiagnostic(rule));
    assert_eq!(a.selection.rule, Some(rule));

    a.drawer = Some(DrawerMode::Validation);
    a.update(Message::JumpToDiagnostic(condition));
    assert_eq!(a.selection.rule, Some(rule));
    assert!(matches!(
        a.snapshot
            .as_ref()
            .unwrap()
            .condition_focus
            .as_ref()
            .map(|focus| (&focus.family, &focus.binding)),
        Some((ConditionFamily::Header, DraftBinding::Existing(id))) if *id == condition
    ));
    assert!(a.drawer.is_none());
    assert_eq!(a.tab, crate::selection::WorkspaceTab::Routes);
}
