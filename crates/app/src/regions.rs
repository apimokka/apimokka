//! MK-023 slice 2 (task 027) — the region/cursor keyboard model.
//!
//! **One model, every screen.** At any moment at most one [`Region`] holds
//! the keyboard cursor, and the cursor names one item within it by index
//! (both regions here are fixed and short — `len` never changes at runtime —
//! so an index is sufficient; the design reserves identity-stored cursors
//! for regions whose list can change under the user, which neither of these
//! is). `Tab`/`Shift+Tab` move the focused region; `ArrowUp`/`ArrowDown` move
//! the cursor within it; `Enter` activates the cursor's item. `App` owns the
//! two pieces of state (`focused_region`, `region_cursor`); this module is
//! the single source `view` and `update` both read, the same
//! `accelerator.rs`/`palette_commands.rs` precedent D-9 is the lesson about.
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

/// A screen-level keyboard region. Two exist in this slice; later slices
/// add variants here without reworking the mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    /// The first-run audience-mode picker (`screens/mode_picker.rs`).
    /// Replaces the old `mode_picker_selected` special case.
    ModePicker,
    /// The Routes/Trace/Settings tab bar (`shell/tab_bar.rs`).
    TabBar,
}

/// The tab bar's items, in Tab order. The one source `shell::tab_bar::view`
/// and this module both read, so the rendered order and the keyboard order
/// can never disagree (the D-9 lesson).
pub const TAB_BAR_ORDER: &[(WorkspaceTab, Key)] = &[
    (WorkspaceTab::Routes, Key::NavRoutes),
    (WorkspaceTab::Trace, Key::NavTrace),
    (WorkspaceTab::Settings, Key::NavSettings),
];

impl Region {
    /// The regions navigable on the app's current screen, in `Tab` order.
    /// Mirrors `App::view`'s own precedence exactly (the mode picker
    /// short-circuits `view()` regardless of `self.view`, so it must
    /// short-circuit here too): at most one of these two is ever non-empty.
    pub fn order_for(app: &App) -> &'static [Region] {
        if app.audience_mode.is_none() {
            &[Region::ModePicker]
        } else if app.view == AppView::Workspace {
            &[Region::TabBar]
        } else {
            &[]
        }
    }

    /// Number of items in this region. Fixed for both regions this slice
    /// declares.
    pub fn len(self) -> usize {
        match self {
            Region::ModePicker => crate::screens::mode_picker::OPTIONS.len(),
            Region::TabBar => TAB_BAR_ORDER.len(),
        }
    }

    /// The index that counts as "active" — where entering this region
    /// starts the cursor (task 027 §3.1). `None` when nothing is active yet,
    /// in which case the cursor starts at the first item instead.
    pub fn active_index(self, app: &App) -> Option<usize> {
        match self {
            // Nothing is chosen until `Enter`; the mode picker never has a
            // "currently selected" mode to resume on.
            Region::ModePicker => None,
            Region::TabBar => TAB_BAR_ORDER.iter().position(|&(tab, _)| tab == app.tab),
        }
    }

    /// The `Message` `Enter` dispatches for the item at `index`, or `None`
    /// if `index` is out of range (an empty or shrunk region).
    pub fn activation_message(self, index: usize) -> Option<Message> {
        match self {
            Region::ModePicker => crate::screens::mode_picker::OPTIONS
                .get(index)
                .map(|&(_, _, mode)| Message::ChooseAudienceMode(mode)),
            Region::TabBar => TAB_BAR_ORDER
                .get(index)
                .map(|&(tab, _)| Message::SwitchTab(tab)),
        }
    }
}

#[cfg(test)]
#[path = "regions/tests.rs"]
mod tests;
