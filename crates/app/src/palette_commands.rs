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
pub struct Command {
    pub label: Key,
    /// `None` when the command has no dedicated keyboard shortcut.
    pub shortcut: Option<Accelerator>,
    pub message: fn() -> Message,
}

pub const TABLE: &[Command] = &[
    Command {
        label: Key::PaletteCmdUndo,
        shortcut: Some(Accelerator::Undo),
        message: || Message::Undo,
    },
    Command {
        label: Key::PaletteCmdRedo,
        shortcut: Some(Accelerator::Redo),
        message: || Message::Redo,
    },
    Command {
        label: Key::PaletteCmdSave,
        shortcut: Some(Accelerator::Save),
        message: || Message::Save,
    },
    Command {
        label: Key::PaletteCmdAddRule,
        shortcut: None,
        message: || Message::AddRuleFromPalette,
    },
    Command {
        label: Key::PaletteCmdAddRuleSet,
        shortcut: None,
        message: || Message::AddRuleSet,
    },
    Command {
        label: Key::PaletteCmdTestRule,
        shortcut: None,
        message: || Message::TestRuleOpen,
    },
    Command {
        label: Key::PaletteCmdToggleTrace,
        shortcut: None,
        message: || Message::ViewAllInTrace,
    },
    Command {
        label: Key::PaletteCmdOpenValidation,
        shortcut: None,
        message: || Message::OpenValidationDrawer,
    },
    Command {
        label: Key::PaletteCmdOpenSaveDiff,
        shortcut: None,
        message: || Message::OpenSaveDiffDrawer,
    },
    Command {
        label: Key::PaletteCmdStartServer,
        shortcut: None,
        message: || Message::StartStopServer,
    },
    Command {
        label: Key::PaletteCmdReload,
        shortcut: Some(Accelerator::Reload),
        message: || Message::ReloadConfig,
    },
    Command {
        label: Key::PaletteCmdRestart,
        shortcut: None,
        message: || Message::RestartServer,
    },
    Command {
        label: Key::PaletteCmdSwitchWorkspace,
        shortcut: None,
        message: || Message::ToggleWorkspaceMenu,
    },
    Command {
        label: Key::PaletteCmdSettings,
        shortcut: None,
        message: || Message::SwitchTab(WorkspaceTab::Settings),
    },
    Command {
        label: Key::PaletteCmdToggleTheme,
        shortcut: None,
        message: || Message::ToggleTheme,
    },
    Command {
        label: Key::PaletteCmdGoRoutes,
        shortcut: None,
        message: || Message::SwitchTab(WorkspaceTab::Routes),
    },
    Command {
        label: Key::PaletteCmdGoTrace,
        shortcut: None,
        message: || Message::SwitchTab(WorkspaceTab::Trace),
    },
    Command {
        label: Key::PaletteCmdGoSettings,
        shortcut: None,
        message: || Message::SwitchTab(WorkspaceTab::Settings),
    },
];

/// Indices into [`TABLE`] whose label matches `query` (case-insensitive
/// substring), in table order. The single source both `view` (which rows to
/// show) and `update` (which index arrow keys/Enter operate on) read, so
/// they can never disagree about what "row 2" means.
pub fn filtered_indices(app: &App, query: &str) -> Vec<usize> {
    let q = query.to_lowercase();
    TABLE
        .iter()
        .enumerate()
        .filter(|(_, cmd)| q.is_empty() || app.t(cmd.label).to_lowercase().contains(&q))
        .map(|(i, _)| i)
        .collect()
}
