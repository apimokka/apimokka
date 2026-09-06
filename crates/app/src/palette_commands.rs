//! MK-033 — command palette table, shared by `view` and `update` (dev-team
//! handoff 002 pattern).
//!
//! `screens::command_palette::view` reads this table to build the rows the
//! user sees; `App::update` reads the same table to execute the row `Enter`
//! selects. One table, one filter function: the row Enter executes cannot
//! drift from the row the user sees highlighted, the same reasoning
//! `accelerator.rs` already applies to keyboard shortcuts.

use crate::accelerator::Accelerator;
use crate::app::App;
use crate::message::Message;
use crate::selection::WorkspaceTab;
use apimokka_i18n::Key;

/// One row in the command palette.
///
/// `label` is `fn(&App) -> Key`, not a plain `Key` — task 019 (D-9): the
/// server row's label must vary with `App::server_state` ("Start server" vs
/// "Stop server"), and `Command.message` was already a zero-argument `fn`
/// for the same reason (`Message::SwitchTab(Settings)` etc. need no
/// state). Every other row ignores its `&App` argument; this is the
/// narrowest change that keeps one shape for all seventeen rows rather than
/// adding a `Static`/`Dynamic` enum only the server row would use.
pub struct Command {
    pub label: fn(&App) -> Key,
    /// `None` when the command has no dedicated keyboard shortcut.
    pub shortcut: Option<Accelerator>,
    pub message: fn() -> Message,
}

pub const TABLE: &[Command] = &[
    Command {
        label: |_| Key::PaletteCmdUndo,
        shortcut: Some(Accelerator::Undo),
        message: || Message::Undo,
    },
    Command {
        label: |_| Key::PaletteCmdRedo,
        shortcut: Some(Accelerator::Redo),
        message: || Message::Redo,
    },
    Command {
        label: |_| Key::PaletteCmdSave,
        shortcut: Some(Accelerator::Save),
        message: || Message::Save,
    },
    Command {
        label: |_| Key::PaletteCmdAddRule,
        shortcut: None,
        message: || Message::AddRuleFromPalette,
    },
    Command {
        label: |_| Key::PaletteCmdAddRuleSet,
        shortcut: None,
        message: || Message::AddRuleSet,
    },
    Command {
        label: |_| Key::PaletteCmdTestRule,
        shortcut: None,
        message: || Message::TestRuleOpen,
    },
    Command {
        label: |_| Key::PaletteCmdToggleTrace,
        shortcut: None,
        message: || Message::ViewAllInTrace,
    },
    Command {
        label: |_| Key::PaletteCmdOpenValidation,
        shortcut: None,
        message: || Message::OpenValidationDrawer,
    },
    Command {
        label: |_| Key::PaletteCmdOpenSaveDiff,
        shortcut: None,
        message: || Message::OpenSaveDiffDrawer,
    },
    Command {
        // D-9: the only row that reads state. Mirrors
        // `shell/top_bar.rs`'s `srv_label` exactly, so the two can no more
        // drift than `view`/`update` can over the message table itself.
        label: |app| match app.server_state {
            crate::shell::top_bar::ServerState::Running => Key::PaletteCmdStopServer,
            _ => Key::PaletteCmdStartServer,
        },
        shortcut: None,
        message: || Message::StartStopServer,
    },
    Command {
        label: |_| Key::PaletteCmdReload,
        shortcut: Some(Accelerator::Reload),
        message: || Message::ReloadConfig,
    },
    Command {
        label: |_| Key::PaletteCmdRestart,
        shortcut: None,
        message: || Message::RestartServer,
    },
    Command {
        label: |_| Key::PaletteCmdSwitchWorkspace,
        shortcut: None,
        message: || Message::ToggleWorkspaceMenu,
    },
    Command {
        label: |_| Key::PaletteCmdToggleTheme,
        shortcut: None,
        message: || Message::ToggleTheme,
    },
    Command {
        label: |_| Key::PaletteCmdGoRoutes,
        shortcut: None,
        message: || Message::SwitchTab(WorkspaceTab::Routes),
    },
    Command {
        label: |_| Key::PaletteCmdGoTrace,
        shortcut: None,
        message: || Message::SwitchTab(WorkspaceTab::Trace),
    },
    Command {
        label: |_| Key::PaletteCmdGoSettings,
        shortcut: None,
        message: || Message::SwitchTab(WorkspaceTab::Settings),
    },
];

/// Indices into [`TABLE`] whose label matches `query` (case-insensitive
/// substring), in table order. The single source both `view` (which rows to
/// show) and `update` (which index arrow keys/Enter operate on) read, so
/// they can never disagree about what "row 2" means.
///
/// Task 019 (D-9): now filters against the label the row actually displays
/// right now, `(cmd.label)(app)` — not a fixed string. For the server row
/// this makes filtering state-dependent: typing `"stop"` matches only while
/// the server is running, `"start"` only while it is stopped. **Decided
/// deliberately, not a side effect left unexamined**: the row's identity in
/// the palette *is* whatever it currently displays — a user searching
/// `"stop"` is looking for the action currently named "Stop server", which
/// exists only while the server runs. Matching the word for a state the row
/// is not currently in would surface a row whose visible label does not
/// contain what was typed, which is a stranger result than not matching at
/// all.
pub fn filtered_indices(app: &App, query: &str) -> Vec<usize> {
    let q = query.to_lowercase();
    TABLE
        .iter()
        .enumerate()
        .filter(|(_, cmd)| q.is_empty() || app.t((cmd.label)(app)).to_lowercase().contains(&q))
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// D-8: two rows dispatching the identical `Message` read as the same
    /// command with two labels -- `PaletteCmdSettings`/`PaletteCmdGoSettings`
    /// both fired `SwitchTab(Settings)` before this fix. `Message` derives
    /// no `PartialEq` (and adding it is out of this task's scope), so this
    /// compares the `Debug` rendering -- exact for every variant `TABLE`
    /// actually uses, and cheap insurance against the duplicate returning.
    #[test]
    fn no_two_table_entries_dispatch_the_same_message() {
        let app = App::new().0;
        let mut seen = std::collections::HashSet::new();
        for cmd in TABLE {
            let rendered = format!("{:?}", (cmd.message)());
            assert!(
                seen.insert(rendered.clone()),
                "{:?} dispatches {rendered}, already dispatched by an earlier row",
                (cmd.label)(&app)
            );
        }
    }

    /// D-9: the server row's label must track `server_state`, the whole
    /// defect in one test. `App::new()` seeds `server_state: Running`
    /// (`app.rs`), so the running case needs no setup and the stopped case
    /// is the one that must be constructed explicitly.
    #[test]
    fn server_row_label_tracks_server_state_in_both_directions() {
        let server_row = TABLE
            .iter()
            .find(|cmd| {
                format!("{:?}", (cmd.message)()) == format!("{:?}", Message::StartStopServer)
            })
            .expect("a row dispatches StartStopServer");

        let mut app = App::new().0;
        assert_eq!(
            app.server_state,
            crate::shell::top_bar::ServerState::Running,
            "test setup sanity"
        );
        assert_eq!((server_row.label)(&app), Key::PaletteCmdStopServer);

        app.server_state = crate::shell::top_bar::ServerState::Stopped;
        assert_eq!((server_row.label)(&app), Key::PaletteCmdStartServer);
    }

    /// Task 019's filtering decision, asserted rather than left implicit:
    /// the server row matches the word for its *current* label only.
    ///
    /// Checks membership of the server row's own index rather than a raw
    /// match count: `"start"` is also a substring of "Restart server"
    /// (`re-start`), a pre-existing, unrelated collision this test must not
    /// be fragile against.
    #[test]
    fn server_row_filtering_matches_only_the_currently_displayed_label() {
        let server_row_index = TABLE
            .iter()
            .position(|cmd| {
                format!("{:?}", (cmd.message)()) == format!("{:?}", Message::StartStopServer)
            })
            .expect("a row dispatches StartStopServer");

        let mut app = App::new().0;
        app.server_state = crate::shell::top_bar::ServerState::Running;
        assert!(
            filtered_indices(&app, "stop").contains(&server_row_index),
            "\"stop\" should find the row while it reads \"Stop server\""
        );
        assert!(
            !filtered_indices(&app, "start").contains(&server_row_index),
            "\"start\" should not find it while it reads \"Stop server\""
        );

        app.server_state = crate::shell::top_bar::ServerState::Stopped;
        assert!(
            filtered_indices(&app, "start").contains(&server_row_index),
            "\"start\" should find the row while it reads \"Start server\""
        );
        assert!(
            !filtered_indices(&app, "stop").contains(&server_row_index),
            "\"stop\" should not find it while it reads \"Start server\""
        );
    }
}
