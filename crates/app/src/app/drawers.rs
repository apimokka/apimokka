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
