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
use apimokka_model::{NodeId, RuleSetId};

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
    /// Routes' left sidebar (task 029, MK-023 slice 4) — rule sets, their
    /// rules, fallback files, middleware scripts, and the add actions,
    /// flattened into one navigable list (`sidebar_rows`). Only declared
    /// while the Routes tab is showing (`Region::order_for`); Trace and
    /// Settings are slice 5's.
    Sidebar,
}

/// One row of the sidebar's flattened visible sequence (task 029 §3). Rows
/// are identified by whatever already identifies them in the data model —
/// a `RuleSetId`, a `NodeId`, a file path, a script path — plus their kind,
/// since the same path/id shape can appear at more than one position
/// (e.g. a rule set's own id and the `AddRule` row beneath it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidebarRow {
    RuleSetHeader(RuleSetId),
    Rule(NodeId),
    /// The "+ Add rule" row under an expanded rule set, scoped to it.
    AddRule(RuleSetId),
    AddRuleSet,
    FallbackHeader,
    FallbackFile(String),
    AddFallbackFile,
    MiddlewareHeader,
    MiddlewareScript(String),
    AddScript,
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
    /// A row's own identity, for `Region::Sidebar` — stored instead of a
    /// position because the accordion and the two section toggles can all
    /// change which rows are visible out from under the cursor (task 029 §3).
    Sidebar(SidebarRow),
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

/// The sidebar's rows, flattened into display order — collapsed children
/// simply absent (task 029 §3). The one source `screens::routes::sidebar`
/// renders from and `Region::Sidebar`'s cursor logic reads (the D-9 lesson,
/// same as the two tables and `dashboard_filtered_workspaces` above): no
/// parallel tree walker exists anywhere in this module or in the view.
///
/// Order, matching `sidebar.rs`'s own rendering exactly: each rule set
/// header, and when open its rules then its `+ Add rule`; `+ Add rule set`;
/// the Fallback files header, and when open its files then `+ Add fallback
/// file`; the Middleware scripts header, and when open its scripts then
/// `+ Add script`.
pub fn sidebar_rows(app: &App) -> Vec<SidebarRow> {
    let mut rows = Vec::new();
    let Some(snap) = app.snapshot.as_ref() else {
        return rows;
    };
    for rs in &snap.rule_sets {
        rows.push(SidebarRow::RuleSetHeader(rs.id));
        if app.rule_set_open == Some(rs.id) {
            for rule in &rs.rules {
                rows.push(SidebarRow::Rule(rule.id));
            }
            rows.push(SidebarRow::AddRule(rs.id));
        }
    }
    rows.push(SidebarRow::AddRuleSet);
    rows.push(SidebarRow::FallbackHeader);
    if app.fallback_section_open {
        for f in &snap.fallback_files {
            rows.push(SidebarRow::FallbackFile(f.path.clone()));
        }
        rows.push(SidebarRow::AddFallbackFile);
    }
    rows.push(SidebarRow::MiddlewareHeader);
    if app.middleware_section_open {
        for s in &snap.middleware_scripts {
            rows.push(SidebarRow::MiddlewareScript(s.path.clone()));
        }
        rows.push(SidebarRow::AddScript);
    }
    rows
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
                // Task 029: the sidebar only renders on the Routes tab
                // (`screens::routes::view` is only reached there) — mirrors
                // that precedence exactly, the same reasoning this function's
                // mode-picker short-circuit already uses one level up.
                AppView::Workspace if app.tab == WorkspaceTab::Routes => {
                    &[Region::TabBar, Region::Sidebar]
                }
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

    /// Whether this region's cursor is an identity (`Cursor::WorkspaceName`/
    /// `Cursor::Sidebar`) rather than a plain `Cursor::Index` — task 027's
    /// "identity where the list can change" rule.
    pub fn uses_identity_cursor(self) -> bool {
        matches!(self, Region::DashboardWorkspaces | Region::Sidebar)
    }

    /// Whether this region contains a native, focusable `text_input` — task
    /// 029 §5: entering a region without one must release any text field
    /// that still holds native keyboard focus (D-18's blast radius), derived
    /// from this property rather than naming regions one by one so a future
    /// region inherits the same guarantee automatically.
    pub fn contains_text_field(self) -> bool {
        matches!(self, Region::DashboardSearch)
    }

    /// Number of items in this region right now. Fixed for every region
    /// except the two identity-cursored ones, whose length depends on live
    /// state (the search filter; the accordion and section toggles).
    pub fn len(self, app: &App) -> usize {
        match self {
            Region::ModePicker => crate::screens::mode_picker::OPTIONS.len(),
            Region::TabBar => TAB_BAR_ORDER.len(),
            Region::WelcomeActions => WELCOME_ACTIONS.len(),
            Region::DashboardCreate | Region::DashboardSearch => 1,
            Region::DashboardWorkspaces | Region::Sidebar => self.identity_list(app).len(),
        }
    }

    /// The index that counts as "active" — where entering this region
    /// starts the cursor (task 027 §3.1). `None` when nothing is active yet,
    /// in which case the cursor starts at the first item instead.
    /// `DashboardWorkspaces` always returns `None`: nothing is "the open
    /// workspace" while choosing one to open. `Sidebar` resumes whatever the
    /// centre panel currently shows, in the same rule > file > script > rule
    /// set priority `screens::routes::centre_panel` itself uses (so the two
    /// can never disagree about what "currently showing" means).
    pub fn active_index(self, app: &App) -> Option<usize> {
        match self {
            // Nothing is chosen until `Enter`; the mode picker never has a
            // "currently selected" mode to resume on.
            Region::ModePicker => None,
            Region::TabBar => TAB_BAR_ORDER.iter().position(|&(tab, _)| tab == app.tab),
            Region::WelcomeActions | Region::DashboardCreate | Region::DashboardSearch => None,
            Region::DashboardWorkspaces => None,
            Region::Sidebar => {
                let target = if let Some(rule) = app.selection.rule {
                    Some(SidebarRow::Rule(rule))
                } else if let Some(path) = &app.selection.file_route {
                    Some(SidebarRow::FallbackFile(path.clone()))
                } else if let Some(path) = &app.selection.script {
                    Some(SidebarRow::MiddlewareScript(path.clone()))
                } else {
                    app.selection.rule_set.map(SidebarRow::RuleSetHeader)
                };
                target.and_then(|t| sidebar_rows(app).iter().position(|r| *r == t))
            }
        }
    }

    /// The live identity list for an identity-cursored region — the one
    /// source `identity_at`, `App::move_region_cursor`'s identity branch and
    /// `App::reclamp_identity_cursor` all read, so a region's cursor and its
    /// rendered list can never disagree about what currently exists (the D-9
    /// lesson, generalised from task 028's single case to task 029's second
    /// one). Empty for every index-cursored region.
    pub fn identity_list(self, app: &App) -> Vec<Cursor> {
        match self {
            Region::DashboardWorkspaces => dashboard_filtered_workspaces(app)
                .into_iter()
                .map(|ws| Cursor::WorkspaceName(ws.name))
                .collect(),
            Region::Sidebar => sidebar_rows(app).into_iter().map(Cursor::Sidebar).collect(),
            _ => Vec::new(),
        }
    }

    /// The identity at position `index` in the *current* identity list —
    /// `App::enter_region` uses this to translate an index (from
    /// `active_index`/the first-item fallback) into the `Cursor` an
    /// identity-cursored region actually stores. `None` for every
    /// index-cursored region (never called for them) or an out-of-range
    /// index.
    pub fn identity_at(self, app: &App, index: usize) -> Option<Cursor> {
        self.identity_list(app).into_iter().nth(index)
    }

    /// The `Message` `Enter` (or, on the sidebar, `ArrowRight` on a
    /// collapsed header — task 029 §4) dispatches for the item at `cursor`,
    /// or `None` if `cursor` names an out-of-range index, a
    /// filtered-away/collapsed-away identity, or a region whose own
    /// interaction is typing or expansion, not activation.
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
            (Region::Sidebar, Cursor::Sidebar(row)) => match row {
                // Collapsed or expanded, this is the one message the header
                // already has (task 029 §4): it both selects and expands,
                // so `Enter`/`ArrowRight` on a collapsed header coincide —
                // stated plainly rather than invented, since it follows from
                // `SelectRuleSet` fusing the two in the data model.
                SidebarRow::RuleSetHeader(id) => Some(Message::SelectRuleSet(*id)),
                SidebarRow::Rule(id) => Some(Message::SelectRule(*id)),
                SidebarRow::AddRule(rs_id) => Some(Message::AddRule(*rs_id)),
                SidebarRow::AddRuleSet => Some(Message::AddRuleSet),
                SidebarRow::FallbackHeader => Some(Message::ToggleFallbackSection),
                SidebarRow::FallbackFile(path) => Some(Message::SelectFileRoute(path.clone())),
                // Inert in this mockup — matches the row's own button, which
                // has no `.on_press` at all (no file I/O; `DisabledNoFileIo`
                // is shown beneath it). Keyboard activation must agree with
                // what clicking it already does: nothing.
                SidebarRow::AddFallbackFile => None,
                SidebarRow::MiddlewareHeader => Some(Message::ToggleMiddlewareSection),
                SidebarRow::MiddlewareScript(path) => Some(Message::SelectScript(path.clone())),
                SidebarRow::AddScript => None,
            },
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "regions/tests.rs"]
mod tests;
