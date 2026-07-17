use super::*;
use crate::message::Message;

fn expert() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::OpenWorkspace("test".into()));
    a
}

// ── DuplicateRule ────────────────────────────────────────────────────

#[test]
fn duplicate_rule_creates_copy_after_original() {
    let mut a = expert();
    let snap = a.snapshot.as_ref().unwrap();
    let orig = snap.rule_sets[0].rules[0].id;
    let before = snap.rule_sets[0].rules.len();
    let orig_path = snap.rule_sets[0].rules[0].payload.url_path.clone();
    drop(snap);

    a.update(Message::DuplicateRule(orig));

    let snap = a.snapshot.as_ref().unwrap();
    assert_eq!(
        snap.rule_sets[0].rules.len(),
        before + 1,
        "duplicate adds one rule"
    );
    // The copy is inserted right after the original
    assert_eq!(
        snap.rule_sets[0].rules[1].payload.url_path, orig_path,
        "copy has the same URL path"
    );
    assert_ne!(snap.rule_sets[0].rules[1].id, orig, "copy has a fresh ID");
    assert_eq!(
        a.selection.rule,
        Some(snap.rule_sets[0].rules[1].id),
        "the copy is selected after duplication"
    );
}

#[test]
fn duplicate_rule_is_undoable() {
    let mut a = expert();
    let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    let before = a.snapshot.as_ref().unwrap().rule_sets[0].rules.len();

    a.update(Message::DuplicateRule(rule_id));
    assert_eq!(
        a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
        before + 1
    );

    a.update(Message::Undo);
    assert_eq!(
        a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
        before,
        "undo removes the duplicated rule"
    );
}

// ── ConfirmAction::DeleteRule removed ────────────────────────────────

#[test]
fn delete_rule_set_still_works_via_confirm() {
    // Verifying the confirm dialog still handles DeleteRuleSet correctly
    // (the remaining live variant after DeleteRule was removed).
    let mut a = expert();
    let rs_id = a.snapshot.as_ref().unwrap().rule_sets[0].id;
    let before = a.snapshot.as_ref().unwrap().rule_sets.len();

    a.update(Message::DeleteRuleSet(rs_id));
    assert!(
        a.confirm_dialog.is_some(),
        "DeleteRuleSet requires confirmation"
    );

    a.update(Message::ConfirmProceed);
    assert_eq!(
        a.snapshot.as_ref().unwrap().rule_sets.len(),
        before - 1,
        "rule set removed after confirmation"
    );
}
