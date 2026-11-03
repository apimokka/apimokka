//! Task 034 §6: "make the census executable." Walks every file under
//! `screens/` and `shell/`, counts each one's own `.on_press(`/`.on_input(`/
//! `.on_toggle(`/`.on_action(` plus the 5 `widgets::*_btn` helper calls
//! (`primary_btn`/`secondary_btn`/`ghost_btn`/`danger_btn`/`icon_btn`), and
//! asserts the counts against the recorded table below — the same
//! tree-walk shape `app/dialog_default_action.rs`'s own
//! `no_text_input_sets_on_submit` already uses for D-6.
//!
//! **This counts literal source occurrences, not logical controls.** A
//! shared helper called from one site but rendering several buttons at
//! runtime (`shell/top_bar.rs`'s own `action_btn`, used for Save and for
//! whichever of Reload/Restart applies, and again for Start/Stop) counts
//! once, not three times — the same undercount `rule_editor.rs`'s own
//! per-row helpers (`HeaderName`/`HeaderOp`/… built once per row in a
//! `.map()`) have for a different reason, in the other direction. A count
//! of true *controls* would need to interpret the code, not scan its text,
//! which task 034 §9's own named risk already anticipated ("may be noisy
//! against helper macros or multi-line calls... say so and propose the
//! narrower guard you can defend") — this is that narrower guard, stated
//! plainly rather than oversold.
//!
//! **What it does catch**: a `.on_press(`/`.on_input(`/`.on_toggle(`/
//! `.on_action(`/`widgets::*_btn(` call added to or removed from a file
//! without this table changing to match — exactly the signal a new,
//! un-triaged control leaves behind, which is the census's whole point:
//! the table going stale is itself the alarm.
//!
//! **When this test fails**: a file's count changed. Read that file, find
//! what was added or removed, decide whether it needs a `Region` (or
//! already has one covering it), then update the count below to match —
//! do not loosen the needle list or delete a row to make this pass.

const NEEDLES: &[&str] = &[
    ".on_press(",
    ".on_input(",
    ".on_toggle(",
    ".on_action(",
    "widgets::primary_btn(",
    "widgets::secondary_btn(",
    "widgets::ghost_btn(",
    "widgets::danger_btn(",
    "widgets::icon_btn(",
];

fn count_controls(contents: &str) -> usize {
    NEEDLES
        .iter()
        .map(|needle| contents.matches(needle).count())
        .sum()
}

/// One row per file under `screens/` or `shell/`, relative to `src/` —
/// the same path format `all_source_files` returns. Recorded 2026-11-03
/// (task 034); update on failure per this module's own doc comment.
const EXPECTED_CONTROL_COUNTS: &[(&str, usize)] = &[
    ("screens/command_palette.rs", 3),
    ("screens/confirm_dialog.rs", 2),
    ("screens/dashboard.rs", 3),
    ("screens/dotted_path.rs", 5),
    ("screens/mode_picker.rs", 1),
    ("screens/mod.rs", 0),
    ("screens/routes.rs", 2),
    ("screens/routes/fallback_editor.rs", 5),
    ("screens/routes/rule_editor.rs", 24),
    ("screens/routes/rule_set_config.rs", 4),
    ("screens/routes/script_viewer.rs", 0),
    ("screens/routes/sidebar.rs", 8),
    ("screens/routes/trace_activity.rs", 2),
    ("screens/settings.rs", 7),
    ("screens/test_rule.rs", 7),
    ("screens/trace.rs", 8),
    ("screens/welcome.rs", 2),
    ("screens/wizard.rs", 9),
    ("screens/workspace_menu.rs", 3),
    ("shell/bottom_drawer.rs", 4),
    ("shell/mod.rs", 0),
    ("shell/tab_bar.rs", 1),
    ("shell/top_bar.rs", 2),
    ("shell/view.rs", 6),
];

#[test]
fn every_screens_and_shell_file_matches_its_recorded_control_count() {
    let files: std::collections::BTreeMap<String, String> =
        crate::app::tests_mk023::all_source_files()
            .into_iter()
            .filter(|(path, _)| path.starts_with("screens/") || path.starts_with("shell/"))
            .collect();

    let mut mismatches = Vec::new();
    for &(path, expected) in EXPECTED_CONTROL_COUNTS {
        match files.get(path) {
            Some(contents) => {
                let actual = count_controls(contents);
                if actual != expected {
                    mismatches.push(format!("{path}: recorded {expected}, counted {actual}"));
                }
            }
            None => mismatches.push(format!("{path}: recorded but no longer exists")),
        }
    }

    let recorded: std::collections::BTreeSet<&str> =
        EXPECTED_CONTROL_COUNTS.iter().map(|&(p, _)| p).collect();
    for path in files.keys() {
        if !recorded.contains(path.as_str()) {
            mismatches.push(format!("{path}: new file, not in the recorded table"));
        }
    }

    assert!(
        mismatches.is_empty(),
        "the census table in app/control_census.rs is stale: {mismatches:#?}"
    );
}
