//! Task 026 (MK-023 slice 1, D-6 and the `Enter`-in-dialog defect task 024's
//! audit found) reducer tests. Pure `update`/`global_key_message` checks — no
//! iced runtime, same verification guidance as task 014's own tests.

use super::*;
use crate::message::{ConfirmAction, Message};

fn expert_in_workspace() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::OpenWorkspace("test".into()));
    assert_eq!(a.view, AppView::Workspace, "test setup sanity");
    a
}

// ── §4.2: Test Rule's default action ─────────────────────────────────────

#[test]
fn enter_runs_the_test_when_the_test_rule_dialog_is_open() {
    let mut a = expert_in_workspace();
    a.update(Message::TestRuleOpen);
    assert!(a.test_rule.open, "test setup sanity");
    let before = a.test_rule.result.clone();
    a.update(Message::EnterPressed);
    assert_ne!(
        a.test_rule.result, before,
        "Enter should have executed Message::TestRuleRun and populated a result"
    );
}

// ── §4.2: the confirm dialog's default is Cancel, never Proceed ──────────

#[test]
fn enter_cancels_the_confirm_dialog_rather_than_proceeding() {
    let mut a = expert_in_workspace();
    a.update(Message::ConfirmRequest(ConfirmAction::LeaveWorkspace));
    assert!(a.confirm_dialog.is_some(), "test setup sanity");

    a.update(Message::EnterPressed);

    assert!(
        a.confirm_dialog.is_none(),
        "Enter must resolve the dialog (as Cancel), not leave it open"
    );
    // The safety property this slice exists to establish: assert the
    // *absence* of the destructive effect, not only which message fired.
    // `ConfirmProceed` on `LeaveWorkspace` calls `leave_workspace()`, which
    // sets `view = Welcome` and clears `snapshot` — neither happened.
    assert_eq!(
        a.view,
        AppView::Workspace,
        "Enter must not have left the workspace"
    );
    assert!(
        a.snapshot.is_some(),
        "Enter must not have discarded the open workspace"
    );
}

// ── §4.2 precedence: confirm_dialog wins over the other two ──────────────

#[test]
fn confirm_dialog_takes_enter_precedence_over_the_palette() {
    let mut a = expert_in_workspace();
    a.update(Message::ToggleCommandPalette);
    assert!(a.command_palette.open, "test setup sanity");
    a.update(Message::ConfirmRequest(ConfirmAction::LeaveWorkspace));
    assert!(a.confirm_dialog.is_some(), "test setup sanity");

    a.update(Message::EnterPressed);

    assert!(
        a.confirm_dialog.is_none(),
        "the confirm dialog must have been the one Enter resolved"
    );
    assert!(
        a.command_palette.open,
        "the palette must be untouched — Enter must not also execute its \
         selected row while a confirm dialog sits over it"
    );
}

#[test]
fn confirm_dialog_takes_enter_precedence_over_test_rule() {
    let mut a = expert_in_workspace();
    a.update(Message::TestRuleOpen);
    assert!(a.test_rule.open, "test setup sanity");
    let result_before = a.test_rule.result.clone();
    a.update(Message::ConfirmRequest(ConfirmAction::LeaveWorkspace));
    assert!(a.confirm_dialog.is_some(), "test setup sanity");

    a.update(Message::EnterPressed);

    assert!(a.confirm_dialog.is_none());
    assert!(a.test_rule.open, "the dialog itself stays open");
    assert_eq!(
        a.test_rule.result, result_before,
        "Test Rule must not also have run while a confirm dialog sits over it"
    );
}

// ── §6: Enter with no dialog open is unchanged for the palette and the ───
// mode picker — a regression guard on task 014's own behaviour.

#[test]
fn enter_still_executes_the_selected_palette_row_with_no_dialog_open() {
    let mut a = expert_in_workspace();
    a.update(Message::ToggleCommandPalette);
    a.update(Message::PaletteQuery("theme".into()));
    a.update(Message::ArrowDown);
    let theme_before = a.theme_choice;
    a.update(Message::EnterPressed);
    assert_ne!(a.theme_choice, theme_before);
}

#[test]
fn enter_still_confirms_the_mode_picker_with_no_dialog_open() {
    let mut a = App::new().0;
    a.update(Message::ArrowDown);
    a.update(Message::EnterPressed);
    assert_eq!(
        a.audience_mode,
        Some(crate::screens::mode_picker::OPTIONS[0].2)
    );
}

// ── §4.1 / §6: a captured key event does not reach the application ───────

#[test]
fn a_captured_key_is_dropped_except_escape() {
    use iced::keyboard::{Key, Modifiers, key::Named};

    let captured = iced::event::Status::Captured;
    let ignored = iced::event::Status::Ignored;

    let enter = iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: Key::Named(Named::Enter),
        modified_key: Key::Named(Named::Enter),
        physical_key: iced::keyboard::key::Physical::Unidentified(
            iced::keyboard::key::NativeCode::Unidentified,
        ),
        location: iced::keyboard::Location::Standard,
        modifiers: Modifiers::empty(),
        text: None,
        repeat: false,
    });
    // `Message` has no `PartialEq` (some variants carry non-comparable
    // payloads elsewhere in the enum), so these match on shape, not `==`.
    assert!(
        global_key_message(&enter, captured).is_none(),
        "a captured Enter must not reach the application"
    );
    assert!(
        matches!(
            global_key_message(&enter, ignored),
            Some(Message::EnterPressed)
        ),
        "an ignored Enter must reach the application, as before"
    );

    let escape = iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: Key::Named(Named::Escape),
        modified_key: Key::Named(Named::Escape),
        physical_key: iced::keyboard::key::Physical::Unidentified(
            iced::keyboard::key::NativeCode::Unidentified,
        ),
        location: iced::keyboard::Location::Standard,
        modifiers: Modifiers::empty(),
        text: None,
        repeat: false,
    });
    assert!(
        matches!(
            global_key_message(&escape, captured),
            Some(Message::EscapePressed)
        ),
        "Escape is the one named exception: it must reach the application \
         even when a focused text_input has already captured it (task 026 §4.1)"
    );
}

// ── §4.3: no `text_input` anywhere sets `.on_submit` ──────────────────────

/// Every `.rs` file under `crates/app/src`, relative to `src/`. Walks the
/// real tree at test time (`env!("CARGO_MANIFEST_DIR")` is the crate root,
/// fixed at compile time, independent of the test binary's working
/// directory) rather than an enumerated list, so a `text_input` added to a
/// file nobody listed cannot silently escape the guard below — see F1 on the
/// review of this task: a hand-enumerated list is airtight exactly where the
/// risk is lowest and silent exactly where a future change would introduce
/// the hazard.
pub(crate) fn all_source_files() -> Vec<(String, String)> {
    fn walk(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<(String, String)>) {
        for entry in std::fs::read_dir(dir).expect("src/ must be readable") {
            let entry = entry.expect("dir entry must be readable");
            let path = entry.path();
            if path.is_dir() {
                walk(&path, root, out);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                // Normalised to `/` regardless of host OS (this gate runs on
                // windows-latest too), so a caller can compare against a
                // plain forward-slash literal.
                let relative = path
                    .strip_prefix(root)
                    .expect("walked path must be under root")
                    .to_string_lossy()
                    .replace('\\', "/");
                let contents = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("reading {relative}: {e}"));
                out.push((relative, contents));
            }
        }
    }
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    walk(&src, &src, &mut files);
    files
}

#[test]
fn no_text_input_sets_on_submit() {
    // The design forbids it permanently (task 026 §4.3): with `on_submit`
    // unset, `Enter` is never captured, so D-6's double-fire cannot occur no
    // matter what global key set this application grows to. A source scan,
    // not a reducer property — there is nothing in `App` state for
    // "a text_input has on_submit set" to be a property of.
    //
    // Scans every source file, not a hand-picked list that names today's
    // `text_input` sites: the hazard this guards against is a *future*
    // `text_input`, which by definition would not be on a list written
    // before it existed.
    let mut failures = Vec::new();
    for (path, contents) in all_source_files() {
        // This file's own source necessarily contains the literal string
        // being searched for (it is the needle, right above, in quotes) —
        // exclude it rather than the check flagging itself.
        if path != "app/dialog_default_action.rs" && contents.contains(".on_submit(") {
            failures.push(path);
        }
    }
    assert!(
        failures.is_empty(),
        "these files set `.on_submit` on a text_input, which D-6 forbids: {failures:?}"
    );
}
