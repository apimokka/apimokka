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

// ── Stack basics ─────────────────────────────────────────────────────

#[test]
fn delete_rule_uses_stack_and_undo_restores() {
    let mut a = expert();
    let (rs_id, rule_id, before) = {
        let snap = a.snapshot.as_ref().unwrap();
        let rs = &snap.rule_sets[0];
        (rs.id, rs.rules[0].id, rs.rules.len())
    };
    a.update(Message::DeleteRule(rule_id));
    assert!(a.confirm_dialog.is_none(), "no dialog for delete rule");
    assert_eq!(
        a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
        before - 1
    );
    assert!(
        matches!(a.undo_stack.last(), Some(UndoCommand::DeleteRule { .. })),
        "undo stack should have DeleteRule"
    );
    assert!(a.redo_stack.is_empty());

    a.update(Message::Undo);
    assert_eq!(
        a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
        before,
        "undo restores the rule"
    );
    assert!(a.undo_stack.is_empty(), "undo stack is empty after undo");
    assert!(
        matches!(a.redo_stack.last(), Some(UndoCommand::DeleteRule { .. })),
        "redo stack has the forward command"
    );

    _ = rs_id; // suppress warning
}

#[test]
fn redo_reapplies_after_undo() {
    let mut a = expert();
    let (rule_id, before) = {
        let snap = a.snapshot.as_ref().unwrap();
        (snap.rule_sets[0].rules[0].id, snap.rule_sets[0].rules.len())
    };
    a.update(Message::DeleteRule(rule_id));
    a.update(Message::Undo); // restore
    assert_eq!(
        a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
        before
    );
    a.update(Message::Redo); // delete again
    assert_eq!(
        a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
        before - 1
    );
}

#[test]
fn add_rule_is_undoable() {
    let mut a = expert();
    let rs_id = a.snapshot.as_ref().unwrap().rule_sets[0].id;
    let before = a.snapshot.as_ref().unwrap().rule_sets[0].rules.len();
    a.update(Message::AddRule(rs_id));
    assert_eq!(
        a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
        before + 1
    );
    assert!(matches!(
        a.undo_stack.last(),
        Some(UndoCommand::AddRule { .. })
    ));

    a.update(Message::Undo);
    assert_eq!(
        a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
        before,
        "undo removes the added rule"
    );
}

#[test]
fn move_rule_is_undoable() {
    let mut a = expert();
    let snap = a.snapshot.as_ref().unwrap();
    let rule_id = snap.rule_sets[0].rules[0].id;
    let rule_1_id = snap.rule_sets[0].rules[1].id;
    drop(snap);

    a.update(Message::MoveRuleDown(rule_id));
    // rule[0] and rule[1] should be swapped
    let snap = a.snapshot.as_ref().unwrap();
    assert_eq!(snap.rule_sets[0].rules[1].id, rule_id);
    assert_eq!(snap.rule_sets[0].rules[0].id, rule_1_id);
    drop(snap);

    a.update(Message::Undo);
    let snap = a.snapshot.as_ref().unwrap();
    assert_eq!(
        snap.rule_sets[0].rules[0].id, rule_id,
        "undo restores original order"
    );
}

#[test]
fn url_path_edit_is_undoable() {
    let mut a = expert();
    let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::SelectRule(rule_id));
    a.update(Message::RuleSetUrlPath("/original".into()));
    // Push a second edit so we can undo to /original
    a.update(Message::RuleSetUrlPath("/modified".into()));
    let path = a.snapshot.as_ref().unwrap().rule_sets[0]
        .rules
        .iter()
        .find(|r| r.id == rule_id)
        .map(|r| r.payload.url_path.clone())
        .unwrap();
    assert_eq!(path, "/modified");

    a.update(Message::Undo);
    let path = a.snapshot.as_ref().unwrap().rule_sets[0]
        .rules
        .iter()
        .find(|r| r.id == rule_id)
        .map(|r| r.payload.url_path.clone())
        .unwrap();
    assert_eq!(path, "/original", "undo restores previous URL path");
}

#[test]
fn new_edit_clears_redo_stack() {
    let mut a = expert();
    let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::DeleteRule(rule_id));
    a.update(Message::Undo);
    assert!(
        !a.redo_stack.is_empty(),
        "redo should be available after undo"
    );

    // New edit should clear redo
    let rs_id = a.snapshot.as_ref().unwrap().rule_sets[0].id;
    a.update(Message::AddRule(rs_id));
    assert!(a.redo_stack.is_empty(), "new edit must clear redo stack");
}

#[test]
fn undo_redo_keyboard_shortcut_exists() {
    // Smoke: Undo and Redo messages are in the enum and handled.
    let mut a = expert();
    // Neither crashes when stacks are empty.
    a.update(Message::Undo);
    a.update(Message::Redo);
}

#[test]
fn dismiss_notice_does_not_clear_undo_stack() {
    // Regression: DismissNotice previously called retain(|_| false) which
    // cleared the undo stack. The banner should dismiss independently of undo.
    let mut a = expert();
    let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::DeleteRule(rule_id));
    assert!(
        !a.undo_stack.is_empty(),
        "undo stack should have entry after delete"
    );

    a.update(Message::DismissNotice);
    assert!(
        !a.undo_stack.is_empty(),
        "dismissing the notice banner must NOT clear the undo stack"
    );

    // ⌘Z should still work after dismissal
    a.update(Message::Undo);
    assert!(
        a.undo_stack.is_empty(),
        "stack consumed by undo after dismissal"
    );
}
