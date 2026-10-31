//! MK-023 slice 2 (task 027) and slice 3 (task 028) — the region/cursor
//! keyboard model.
//!
//! **One model, every screen.** At any moment at most one [`Region`] holds
//! the keyboard cursor, and the cursor names one item within it — a
//! [`Cursor`], which is an index where a region's list is fixed and short
//! (`ModePicker`, `TabBar`, `WelcomeActions`, `DashboardCreate`,
//! `DashboardSearch`) or an identity where the list can change under the
//! user (`DashboardWorkspaces`, filtered live by the search field — this
//! slice's first case, not a second cursor concept beside the first: the
//! design names both shapes as the same thing viewed under different list
//! volatility, and `Cursor` expresses that as one type rather than two
//! `App` fields). `Tab`/`Shift+Tab` move the focused region; `ArrowUp`/
//! `ArrowDown` move the cursor within it; `Enter` activates the cursor's
//! item. `App` owns the two pieces of state (`focused_region`,
//! `region_cursor`); this module is the single source `view` and `update`
//! both read, the same `accelerator.rs`/`palette_commands.rs` precedent D-9
//! is the lesson about.
//!
//! **Derive region validity, never trust stored state across a screen
//! change.** No message here enumerates every transition that could make a
//! screen's regions change (choosing a mode, opening or leaving a
//! workspace, …) and reset `focused_region` accordingly — that would be
//! exactly the kind of special-casing per transition the design warns
//! against. Instead `App::valid_focused_region` recomputes the current
//! screen's region order on every keystroke and discards a stored region
//! that is no longer in it, the same "derive, don't duplicate" reasoning
//! the design applies to cursor identity, applied one level up.

use crate::app::{App, AppView};
use crate::message::Message;
use crate::selection::WorkspaceTab;
use apimokka_i18n::Key;
use apimokka_model::mock::RecentWorkspace;

/// A screen-level keyboard region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    /// The first-run audience-mode picker (`screens/mode_picker.rs`).
    ModePicker,
    /// The Routes/Trace/Settings tab bar (`shell/tab_bar.rs`).
    TabBar,
    /// Welcome's two actions (task 028 §2).
    WelcomeActions,
    /// Dashboard's "Create workspace" header button (task 028 §2).
    DashboardCreate,
    /// Dashboard's search field (task 028 §2 / §3.1).
    DashboardSearch,
    /// Dashboard's filtered workspace list (task 028 §2 / §3.2) — the
    /// model's first identity-cursored region.
    DashboardWorkspaces,
}

/// The cursor's position within whichever region holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cursor {
    /// A position in a fixed, short list.
    Index(usize),
    /// A workspace's name, for `Region::DashboardWorkspaces` — stored
    /// instead of a position because the list is rebuilt on every
    /// keystroke in the search field (task 028 §3.2).
    WorkspaceName(String),
}

/// The tab bar's items, in Tab order. The one source `shell::tab_bar::view`
/// and this module both read, so the rendered order and the keyboard order
/// can never disagree (the D-9 lesson).
pub const TAB_BAR_ORDER: &[(WorkspaceTab, Key)] = &[
    (WorkspaceTab::Routes, Key::NavRoutes),
    (WorkspaceTab::Trace, Key::NavTrace),
    (WorkspaceTab::Settings, Key::NavSettings),
];

/// One row of [`WELCOME_ACTIONS`]: a label and the `Message` it dispatches,
/// as a function pointer rather than a stored `Message` — the same shape
/// `palette_commands::Command` uses, since `Message` has no `Copy`/`Clone`
/// that would let a table store one directly.
pub type WelcomeAction = (Key, fn() -> Message);

/// Welcome's two actions, in visual (and Tab) order (task 028 §2).
pub const WELCOME_ACTIONS: &[WelcomeAction] = &[
    (Key::WelcomeOpenWorkspace, || Message::GoDashboard),
    (Key::WelcomeCreateWorkspace, || Message::GoWizard),
];

/// The workspaces visible in Dashboard's filtered list right now — the one
/// source both `screens::dashboard::view` and `Region::DashboardWorkspaces`'s
/// cursor logic read, so the rendered list and the keyboard cursor's idea of
/// "what's visible" can never disagree (the D-9 lesson, same as the two
/// tables above). Matches `screens/dashboard.rs`'s former inline filter
/// exactly: substring match on name or path, case-insensitive, empty query
/// matches everything.
pub fn dashboard_filtered_workspaces(app: &App) -> Vec<RecentWorkspace> {
    let q = app.dash_search.to_lowercase();
    apimokka_model::mock::recent_workspaces()
        .into_iter()
        .filter(|ws| {
            q.is_empty()
                || ws.name.to_lowercase().contains(&q)
                || ws.path.to_lowercase().contains(&q)
        })
        .collect()
}

impl Region {
    /// The regions navigable on the app's current screen, in `Tab` order.
    /// Mirrors `App::view`'s own precedence exactly (the mode picker
    /// short-circuits `view()` regardless of `self.view`, so it must
    /// short-circuit here too): at most one of the `view`-level branches
    /// below is ever non-empty.
    pub fn order_for(app: &App) -> &'static [Region] {
        if app.audience_mode.is_none() {
            &[Region::ModePicker]
        } else {
            match app.view {
                AppView::Workspace => &[Region::TabBar],
                AppView::Welcome => &[Region::WelcomeActions],
                AppView::Dashboard => &[
                    Region::DashboardCreate,
                    Region::DashboardSearch,
                    Region::DashboardWorkspaces,
                ],
                AppView::Wizard => &[],
            }
        }
    }

    /// Whether this region's cursor is a `Cursor::WorkspaceName` rather than
    /// a `Cursor::Index` (task 027's "identity where the list can change"
    /// rule; task 028 is the model's first case of it).
    pub fn uses_identity_cursor(self) -> bool {
        matches!(self, Region::DashboardWorkspaces)
    }

    /// Number of items in this region right now. Fixed for every region
    /// except `DashboardWorkspaces`, whose length depends on the live
    /// search filter.
    pub fn len(self, app: &App) -> usize {
        match self {
            Region::ModePicker => crate::screens::mode_picker::OPTIONS.len(),
            Region::TabBar => TAB_BAR_ORDER.len(),
            Region::WelcomeActions => WELCOME_ACTIONS.len(),
            Region::DashboardCreate | Region::DashboardSearch => 1,
            Region::DashboardWorkspaces => dashboard_filtered_workspaces(app).len(),
        }
    }

    /// The index that counts as "active" — where entering this region
    /// starts the cursor (task 027 §3.1). `None` when nothing is active yet,
    /// in which case the cursor starts at the first item instead.
    /// `DashboardWorkspaces` always returns `None`: nothing is "the open
    /// workspace" while choosing one to open.
    pub fn active_index(self, app: &App) -> Option<usize> {
        match self {
            // Nothing is chosen until `Enter`; the mode picker never has a
            // "currently selected" mode to resume on.
            Region::ModePicker => None,
            Region::TabBar => TAB_BAR_ORDER.iter().position(|&(tab, _)| tab == app.tab),
            Region::WelcomeActions
            | Region::DashboardCreate
            | Region::DashboardSearch
            | Region::DashboardWorkspaces => None,
        }
    }

    /// For `DashboardWorkspaces`, the identity at position `index` in the
    /// *current* filtered list — `App::enter_region` uses this to translate
    /// an index (from `active_index`/the first-item fallback) into the
    /// `Cursor::WorkspaceName` this region actually stores. `None` for every
    /// index-cursored region (never called for them) or an out-of-range
    /// index.
    pub fn identity_at(self, app: &App, index: usize) -> Option<Cursor> {
        match self {
            Region::DashboardWorkspaces => dashboard_filtered_workspaces(app)
                .get(index)
                .map(|ws| Cursor::WorkspaceName(ws.name.clone())),
            _ => None,
        }
    }

    /// The `Message` `Enter` dispatches for the item at `cursor`, or `None`
    /// if `cursor` names an out-of-range index, a filtered-away workspace,
    /// or (`DashboardSearch`) a region whose own interaction is typing, not
    /// activation.
    pub fn activation_message(self, cursor: &Cursor) -> Option<Message> {
        match (self, cursor) {
            (Region::ModePicker, Cursor::Index(i)) => crate::screens::mode_picker::OPTIONS
                .get(*i)
                .map(|&(_, _, mode)| Message::ChooseAudienceMode(mode)),
            (Region::TabBar, Cursor::Index(i)) => TAB_BAR_ORDER
                .get(*i)
                .map(|&(tab, _)| Message::SwitchTab(tab)),
            (Region::WelcomeActions, Cursor::Index(i)) => {
                WELCOME_ACTIONS.get(*i).map(|&(_, message)| message())
            }
            (Region::DashboardCreate, Cursor::Index(0)) => Some(Message::GoWizard),
            (Region::DashboardSearch, _) => None,
            (Region::DashboardWorkspaces, Cursor::WorkspaceName(name)) => {
                Some(Message::OpenWorkspace(name.clone()))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "regions/tests.rs"]
mod tests;
