//! Boundary decision: single-responsibility — the region/cursor keyboard
//! model's one definition (`Region`, `Cursor`, and every screen's row/field
//! tables), across every slice that has extended it (027–030). Kept as one
//! file rather than one per slice because the model is one coherent thing a
//! reader needs to see together — splitting it by slice would scatter
//! `Region`'s own `match` arms across files instead.
//!
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

use crate::app::{App, AppView, Overlay};
use crate::message::{ConfirmAction, Message};
use crate::selection::WorkspaceTab;
use apimokka_i18n::Key;
use apimokka_model::mock::RecentWorkspace;
use apimokka_model::{
    BodyOp, HeaderOp, MatchTraceEvent, NodeId, RuleSetId, TraceOutcome, UrlPathOp,
};

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
    /// One rule-editor field or group (task 030, MK-023 slice 5) — a lone
    /// `text_input`/button/`pick_list` is its own one-item region; the
    /// method buttons and the respond-mode tabs are multi-item groups. Each
    /// `EditorField` is its own `Region`, not a sub-cursor within one shared
    /// "editor" region — task 030 §2 rejected the zone-then-arrow scheme
    /// precisely so Tab reaches every field directly, the way every other
    /// form a user has met already works. Declared only while a rule is
    /// selected and shown in the centre panel.
    Editor(EditorField),
    /// Trace's live filter — a one-item text-field region.
    TraceFilter,
    /// Trace's Pause/Resume button — a one-item region (the label toggles;
    /// the region does not).
    TracePause,
    /// Trace's Clear button.
    TraceClear,
    /// Trace's event list — identity-cursored by `event_id`, since the list
    /// grows while the server runs (task 030 §4).
    TraceEvents,
    /// The match-detail panel's outcome-specific action (Jump to rule / Jump
    /// to file / Create a rule for this path) — present only when the
    /// selected event's outcome actually provides one (task 030 §4: "exist
    /// only when rendered", derived in `order_for`, never assumed).
    TraceOutcomeAction,
    /// "Replay as Test Input" — present only when an event is selected.
    TraceReplay,
    /// One Settings control or group (task 031, MK-023 slice 6) — the same
    /// one-item/multi-item-group shapes slices 2–5 already established,
    /// plus the locale `pick_list`'s value-cycler shape from slice 5 §3.
    /// Theme and audience mode were originally designed to move to the
    /// command palette instead (withdrawn — task 031 §3: that would leave
    /// visible controls a keyboard user still could not reach, which is a
    /// worse inconsistency than the one it avoided).
    Settings(SettingsField),
    /// One Wizard control or group (task 031, MK-023 slice 6) — the wizard
    /// was never assigned to a slice at all (task 031 §1); this is the
    /// first slice to make it navigable.
    Wizard(WizardField),
    /// One fallback-file editor control (task 033, MK-023 slice 8) — one of
    /// the "three surfaces, not one" the Routes centre panel can show by
    /// sidebar selection (slice 7 found the first; this slice's own census
    /// is what found the other two). Declared only while a fallback file,
    /// not a rule, is selected (`centre_panel`'s own priority 2).
    FallbackEditor(FallbackField),
    /// One rule-set configuration control (task 033) — shown by the Routes
    /// centre panel when a rule set, not a rule or file, is selected
    /// (`centre_panel`'s own priority 4). The pane's own rule-link rows are
    /// deliberately **not** given a region here — see
    /// `rule_set_config_fields`'s own doc comment for why duplicating the
    /// sidebar's identical `Message::SelectRule` dispatch was judged not
    /// worth a second keyboard path to the same action.
    RuleSetConfig(RuleSetConfigField),
    /// One bottom-drawer control (task 033) — the validation panel or the
    /// save-diff panel, whichever `app.drawer` currently shows. Overrides
    /// every other region while open, the same shape `ConfirmDialog` below
    /// and the palette/test-rule dialogs already use, rather than
    /// coexisting with the screen underneath it (task 033 §2: "its
    /// diagnostic list cannot be used" is the defect; a cursor that could
    /// be in two places as once would be a worse one).
    Drawer(DrawerField),
    /// The confirm dialog's own two actions (task 033, closing the gap
    /// task 026 left on purpose): `Cancel` (`Cursor::Index(0)`, the entry
    /// position — `EnterPressed` with no prior key at all still cancels,
    /// preserving task 026 §4.2's safety property exactly) and `Proceed`
    /// (`Cursor::Index(1)`, reachable only by a deliberate `ArrowDown`
    /// then `Enter` — never by `Enter` alone; `Tab` moves *between*
    /// regions, and this dialog is the only one in `order_for` while it is
    /// open, so `Tab` just re-enters it at `Cancel`). Overrides every
    /// other region while the dialog is open, same precedence
    /// `EnterPressed`'s own dialog-priority chain already gives it.
    ConfirmDialog,
    /// One Test Rule control (task 034, MK-023 slice 9) — named in MK-023's
    /// own acceptance-clause inventory (scenario 3's "5 method buttons, Run
    /// and Close"). `test_rule.open` is now an overlay like `ConfirmDialog`
    /// (`App::topmost_overlay`'s own list), not a `blocks_region_navigation`
    /// special case. Task 026 §4.2's safe default survives: `Run` is the
    /// forced entry position (`Message::TestRuleOpen`/`ReplayAsTestInput`
    /// set it explicitly, the same mechanism `ConfirmRequest` uses), and
    /// the three text fields dispatch `TestRuleRun` instead of the usual
    /// `None` — the one deliberate exception to "a text-field region has no
    /// activation" (see `test_rule_fields`' own doc comment).
    TestRule(TestRuleField),
    /// One workspace-menu control (task 034) — the menu had no region at
    /// all before this slice (D-22 finding #2): while open, the keyboard
    /// acted on the screen behind it. Overrides every region below it in
    /// `App::topmost_overlay`'s order.
    WorkspaceMenu(WorkspaceMenuField),
    /// One dotted-path-assistant control (task 034) — named in the
    /// acceptance-clause inventory. Reachable today from
    /// `EditorField::BodyPathAssistant`'s own `Enter`; before this slice a
    /// keyboard user could open it and only `Escape` back out (task 033's
    /// fix was a leak-prevention gate only, not coverage). Overrides every
    /// region below it.
    PathAssistant(PathAssistantField),
    /// One control in the rule editor's "recent trace activity" footer
    /// (task 034) — the Routes focus order's own last-named entry. Lives
    /// below `Region::Editor`'s own WHEN/RESPOND columns in reading order
    /// (`trace_activity_section` renders below both in
    /// `screens::routes::rule_editor`), not inside either of them — same
    /// reason Wizard's `Cancel`/`Create` sit outside its own scrollable.
    TraceActivity(TraceActivityField),
    /// One feedback-banner control (task 034) — the banner (error / undo /
    /// notice, mutually exclusive) renders between the tab bar and the
    /// screen on every workspace tab, so its fields sit in `order_for`
    /// right after `TabBar`, before whichever tab's own regions follow.
    /// `Ctrl+Z` already reaches `Undo`; the rest had no route. Field list
    /// derived from `shell::view::active_banner`, the one function the view
    /// itself branches on — not a second reading of the same three `if`s.
    FeedbackBanner(FeedbackBannerField),
    /// The Routes centre panel's own empty-state call to action — "Add a
    /// rule" or "Add rule set", depending on whether the workspace has any
    /// rule sets yet. The census missed it in task 033 (it is the state the
    /// Routes tab opens in); mitigated already (the sidebar's own covered
    /// "+" rows dispatch the same messages) but still a real gap on its own
    /// terms. A bare one-item region: the message it dispatches is live
    /// state, not a fixed table, so there is no field enum to carry.
    RoutesEmptyCta,
    /// One top-bar control (task 034) — the workspace switcher and the
    /// global action buttons, the RFC's own Routes focus order's first two
    /// named entries ("Top bar identity" and "Server status / global
    /// actions"). Always visible across every workspace tab (`shell::view
    /// ::view`'s own `.header(top_bar::view(app))`), so these sit first in
    /// `order_for`'s `AppView::Workspace` arm, before `TabBar`.
    TopBar(TopBarField),
}

/// One field or group in the rule editor's reading order (task 030 §2).
/// Variants carrying a `usize` are scoped to one header/body condition row
/// by its position in `RulePayload::headers`/`body` — rows have no other
/// stable identity in this mockup's data model (unlike the sidebar's rule
/// sets and rules, which already have real ids).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorField {
    TestRule,
    Duplicate,
    MoveUp,
    MoveDown,
    Delete,
    UrlPath,
    /// A `pick_list` region (task 030 §3): arrows cycle the value in place
    /// and dispatch the same message the dropdown's own selection would,
    /// since iced 0.14 gives `pick_list` no way to open or focus from the
    /// keyboard at all.
    UrlPathOp,
    /// The 6 method buttons — a multi-item group.
    Method,
    /// Guided mode's "More matching criteria" / "Fewer" toggle. Absent in
    /// Expert mode (headers/body show directly there) and absent whenever
    /// Guided mode's toggle has nothing to disclose differently — it is
    /// always exactly one of the two labels when shown at all.
    MoreWhenToggle,
    HeaderName(usize),
    /// A `pick_list` region, same reasoning as `UrlPathOp`.
    HeaderOp(usize),
    /// Present only when the row's operator does not make the value
    /// irrelevant (`HeaderOp::value_irrelevant`) — mirrors the view's own
    /// `show_val` condition exactly.
    HeaderValue(usize),
    HeaderRemove(usize),
    AddHeader,
    BodyPath(usize),
    BodyPathAssistant(usize),
    /// A `pick_list` region, same reasoning as `UrlPathOp`.
    BodyOp(usize),
    /// Present only when the row's operator is not `Exists`/`Absent` —
    /// mirrors the view's own `show_val` condition exactly.
    BodyValue(usize),
    BodyRemove(usize),
    AddBody,
    /// The 2 respond-mode tabs (Inline text / Serve file) — a multi-item
    /// group.
    RespondMode,
    /// The respond body field — inline text or a file path depending on
    /// `RespondMode`; always exactly one is rendered, so this is one region
    /// regardless of which message typing into it dispatches.
    RespondBody,
    RespondStatus,
    RespondDelay,
    /// The per-rule weight/priority field, present only when the workspace's
    /// strategy needs one and (in Guided mode) the "more" disclosure is open.
    PerRuleField,
}

/// One field or group in Settings' reading order (task 031 §2). Mirrors
/// `screens::settings::view`'s own branching for the Logs/Trace sections
/// (Guided mode's advanced disclosure vs. Expert mode's always-shown
/// sections) exactly — the same "derive, don't duplicate" discipline
/// `editor_fields` already applies to the rule editor's conditional rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    /// The 4 theme buttons — a multi-item group.
    Theme,
    /// Guided / Expert — a multi-item group.
    AudienceMode,
    /// A `pick_list` region, same reasoning as the rule editor's `UrlPathOp`
    /// (task 030 §3) — D-19 applies here too, and is not this slice's to
    /// fix (task 031 §4).
    Locale,
    Host,
    Port,
    /// Enable TLS — a checkbox; `Enter` toggles it.
    Tls,
    /// The Guided-mode "More/Fewer settings" disclosure — absent in Expert
    /// mode, where the Logs/Trace sections below are always shown directly.
    AdvancedToggle,
    /// A disabled `text_input` (task 031 §8, §4.2's known risk): no
    /// `on_input` is wired in `screens::settings.rs`, so typing does
    /// nothing — the same inert-on-keyboard shape already established for
    /// `SidebarRow::AddFallbackFile` (task 029): kept reachable rather than
    /// skipped, because clicking it *also* still gives it native focus
    /// (verified against `iced_widget::text_input`'s own `update`: the
    /// mouse-press handler sets `state.is_focused` unconditionally, never
    /// checking `self.on_input.is_none()` — only the key-handling branches
    /// check that, which is what makes typing a no-op). Skipping the Tab
    /// stop would make keyboard navigation *less* consistent with the
    /// pointer than including it, not more.
    LogFile,
    /// Enable trace — a checkbox; `Enter` toggles it. Unlike the Wizard's
    /// own trace checkbox (`WizardField::TraceEnable`), this one has a real
    /// `on_toggle` wired, so it is live, not another disabled case.
    TraceEnable,
    /// A disabled `text_input`, same reasoning as `LogFile` — found at
    /// source, not assumed from the handoff's table, which named this
    /// alongside `Host`/`Port` as if it were an ordinary editable field.
    /// `screens::settings.rs`'s Trace section wires no `on_input` for it at
    /// all (unlike the Wizard's own queue-size field, which does).
    QueueSize,
}

/// One field or group in the Wizard's reading order (task 031 §2). Mirrors
/// `screens::wizard::view`'s own section order and `app.wizard.section_open`
/// disclosure exactly — required (name, folder), then Server/Starter/Trace,
/// each its own toggle region plus its body's fields only while open, then
/// the action bar. The wizard was never assigned to a slice before this one
/// (task 031 §1): `Region::order_for`'s `AppView::Wizard` arm was `vec![]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardField {
    Name,
    Folder,
    /// The Server section's own collapsible header button.
    ServerToggle,
    /// Present only while the Server section is open.
    Host,
    /// Present only while the Server section is open.
    Port,
    /// Present only while the Server section is open — a checkbox; `Enter`
    /// toggles it.
    Tls,
    /// The Starter section's own collapsible header button.
    StarterToggle,
    /// The 3 starter radios — a multi-item group, present only while the
    /// Starter section is open.
    Starter,
    /// The Trace section's own collapsible header button.
    TraceToggle,
    /// A disabled checkbox (task 031 §8, §4.2's known risk): rendered
    /// `checkbox(true)` with no `.on_toggle` at all in
    /// `screens::wizard.rs` — always on, un-settable from either input
    /// device. The parallel case `SettingsField::LogFile`'s own doc comment
    /// points to: kept reachable rather than skipped, `Enter` a no-op, for
    /// the same consistency reasoning. Present only while the Trace section
    /// is open.
    TraceEnable,
    /// Present only while the Trace section is open. Unlike Settings' own
    /// queue-size field, this one has a real `on_input` wired
    /// (`Message::WizardSetQueueSize`) — live, not another disabled case.
    QueueSize,
    Cancel,
    /// Not named in task 031 §2's table, which lists only the form fields —
    /// included because scenario 2 ("create a workspace from the wizard")
    /// is literally "activate this button", and the same table's
    /// `Host`/`Port`/… being reachable while `Create` stayed a pointer-only
    /// action would be exactly the inconsistency task 031 §3 reversed
    /// theme/audience mode over.
    Create,
}

/// One control in the fallback-file editor's reading order (task 033 §2).
/// Mirrors `screens::routes::fallback_editor::fallback_file_editor`'s own
/// layout: the content editor, then the footer's status field, Format,
/// Revert and Save, in that order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackField {
    /// The `text_editor` itself — natively focusable, confirmed at source
    /// the same way `text_input` already was for D-18: its own
    /// `ButtonPressed` handler captures the event (positioning its native
    /// cursor), `ButtonReleased` does not, so the same `mouse_area::
    /// on_release` click-focus mechanism applies unchanged.
    Content,
    Status,
    Format,
    /// Present always, inert when the file is not dirty — a click does
    /// nothing then either (`fallback_file_editor`'s own `revert_btn` has
    /// no `.on_press` at all when `!dirty`), the same "`Enter` must agree
    /// with what a click already does" reasoning task 031 §8 already
    /// established for `SettingsField::LogFile`/`QueueSize`.
    Revert,
    /// Same reasoning as `Revert`.
    Save,
}

/// One control in the rule-set configuration pane's reading order (task 033
/// §2). The pane's own rule rows are deliberately excluded: they dispatch
/// the identical `Message::SelectRule` the sidebar's own rows already do,
/// and this design has no other instance of two regions on the same screen
/// reaching the same action — adding one here would be a second keyboard
/// path to a destination the sidebar already reaches, not a new capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleSetConfigField {
    Delete,
    /// A `pick_list` region, same `is_value_cycler` shape as the editor's
    /// three operator pickers and Settings' locale (D-19's mechanism,
    /// `Enter`-cycles-forward and the `⇅` handle included).
    Strategy,
    /// Guided mode's own disclosure — absent in Expert, where the strategy
    /// picker always shows directly. Mirrors
    /// `rule_set_config`'s own `app.rule_set_config_more` branching
    /// exactly, including that the toggle renders *after* the picker once
    /// expanded, not before (the same order task 031 §2 established for
    /// Settings' advanced disclosure).
    MoreToggle,
    AddRule,
}

/// One control in the bottom drawer's reading order (task 033 §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerField {
    Close,
    /// Scoped by position, not by the diagnostic's own `NodeId` — checked
    /// at source, not assumed: a single rule can carry more than one
    /// validation issue, so two rows can share the same `target`, the same
    /// "no stable per-row identity" situation task 030 found for header
    /// and body condition rows, answered the same way. Indexes only the
    /// *clickable* subset (`target.is_some()`), matching exactly which rows
    /// `validation_content` itself wraps in a button.
    JumpToDiagnostic(usize),
    /// Present only in `DrawerMode::SaveDiff`, and only once
    /// `save_diff_total` is non-zero — mirrors `save_diff_content`'s own
    /// early-return-to-the-all-clear-state exactly.
    Discard,
    /// Same presence condition as `Discard`.
    SaveAll,
}

/// One control in the Test Rule dialog's reading order (task 034 §3) —
/// `screens::test_rule::view`'s own top-to-bottom, left-to-right layout:
/// the header's `✕`, the method group, the three request fields, then the
/// footer's `Close`/`Run`. Two distinct controls dispatch
/// `Message::TestRuleClose` (the header `✕` and the footer's ghost
/// button) — both real, independently focusable widgets, named for their
/// position rather than merged into one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestRuleField {
    /// The header's `✕`.
    Dismiss,
    /// The 5 method buttons — a multi-item group, no "Any" option (unlike
    /// the rule editor's own `EditorField::Method`): a test request always
    /// has one concrete method.
    Method,
    /// Dispatches `Message::TestRuleRun` on activation, not `None` — see
    /// this enum's own top-level doc comment for why these three text
    /// fields are the deliberate exception.
    Path,
    /// Same exception as `Path`.
    Headers,
    /// Same exception as `Path`.
    Body,
    /// The footer's ghost `Close` button.
    Close,
    /// The forced entry position (`Message::TestRuleOpen`/
    /// `ReplayAsTestInput` set `focused_region`/`region_cursor` to this the
    /// moment the dialog opens, the same mechanism `ConfirmRequest` uses
    /// for `ConfirmDialog`'s own `Cancel`) — task 026 §4.2's oldest
    /// behaviour in this cycle, preserved: a bare `Enter` with no prior Tab
    /// still runs the dry run.
    Run,
}

/// One control in the workspace menu's reading order (task 034 §3) — the
/// menu had no region at all before this slice (D-22 finding #2).
/// `mock::recent_workspaces()` is a fixed list for the lifetime of one
/// open menu (nothing filters or searches it, unlike Dashboard's own
/// workspace list), so a plain index-cursored group is enough — no
/// identity cursor needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceMenuField {
    /// The recent-workspace rows — a multi-item group, present only when
    /// the list is non-empty.
    Recent,
    Open,
    Create,
}

/// One control in the dotted-path assistant's reading order (task 034 §3)
/// — named in MK-023's own acceptance-clause inventory. Mirrors
/// `screens::dotted_path::view`'s own layout: the header's `✕`, the JSON
/// paste field, each parsed leaf's `Use` row, then the footer's
/// `Cancel`/`Insert path`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathAssistantField {
    Close,
    /// The JSON paste field — typing is the interaction, same as every
    /// other text-field region; no activation message.
    Json,
    /// Scoped by position in `dotted_path::parse_leaves`'s own output,
    /// re-derived fresh at activation time rather than captured once — the
    /// same "activation re-reads the live list" shape `DrawerField::
    /// JumpToDiagnostic` already uses.
    UseLeaf(usize),
    Cancel,
    Insert,
}

/// One control in the rule editor's "recent trace activity" footer (task
/// 034 §3) — the Routes focus order's own last-named entry. Mirrors
/// `screens::routes::trace_activity::trace_activity_section`'s own layout:
/// the header's "View all in Trace", then up to 3 "Jump to trace event"
/// rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceActivityField {
    ViewAll,
    /// Scoped by position among `recent_matching_events`'s own output
    /// (at most 3), re-derived fresh at activation time — the events
    /// themselves carry a stable `event_id`, but the *position* in this
    /// short, capped list is what Tab order and the cursor both track, the
    /// same shape `DrawerField::JumpToDiagnostic` already uses for a
    /// capped, re-derived list.
    JumpToEvent(usize),
}

/// One control in the feedback banner's reading order (task 034 §3) — the
/// banner is mutually exclusive by construction (error > undo > notice),
/// so this enum's own fields span all three rather than one each; which
/// ones actually exist right now comes from `feedback_banner_fields`,
/// derived from `shell::view::active_banner` — the same function the view
/// itself branches on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackBannerField {
    /// The error banner's Show/Hide technical detail toggle — present only
    /// when the problem carries a `technical_detail`.
    DetailsToggle,
    /// The error banner's own action button — present only when the
    /// problem carries an `action_label`.
    Action,
    /// Dismisses whichever banner is showing — `Message::DismissProblem`
    /// for the error banner, `Message::DismissNotice` for the other two.
    /// Re-derives which from `shell::view::active_banner` at activation
    /// time rather than fixing it to the enum variant, since the same
    /// field name covers three different banners.
    Dismiss,
    /// The undo banner's own `Undo` button — `Ctrl+Z` already reaches this
    /// message; this is the `Tab`-reachable route task 034 §7 also
    /// requires evidence for.
    Undo,
}

/// One control in the top bar's reading order (task 034 §3) — named in the
/// RFC's own Routes focus order (its first two entries). `ReloadOrRestart`
/// is absent entirely, not merely inert, when neither action is available
/// right now — mirrors `shell::top_bar::view`'s own `config_btn` exactly
/// (a plain `Space`, not a disabled button, in that case). The other three
/// are present always, inert when not actionable, the same shape
/// `FallbackField::Revert`/`Save` already established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopBarField {
    Switcher,
    SaveAll,
    ReloadOrRestart,
    StartStop,
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
    /// An event's own id, for `Region::TraceEvents` — stored instead of a
    /// position because the list grows while the server runs (task 030 §4).
    TraceEvent(u64),
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

/// The Wizard's 3 starter radios, in visual (and Tab) order (task 031 §2) —
/// the one source `screens::wizard::view`'s own radio list and
/// `Region::Wizard(WizardField::Starter)`'s cursor logic both read.
pub const WIZARD_STARTER_ORDER: &[crate::app::WizardStarter] = &[
    crate::app::WizardStarter::Minimal,
    crate::app::WizardStarter::ShopApi,
    crate::app::WizardStarter::Empty,
];

/// The 5 configurable HTTP methods, in display order (task 035, D-26) —
/// the one source `screens::test_rule::view`'s own method group,
/// `screens::routes::rule_editor`'s own method group, and this module's
/// `len`/`active_index`/`activation_message` for both `EditorField::Method`
/// and `TestRuleField::Method` all read. Four call sites spelled the same
/// list, or a variant of it, separately before this slice: two plain
/// copies (Test Rule's view and region), and two 6-entry variants of the
/// rule editor's own group (an "Any"-display-prefixed one for the view and
/// `active_index`, and an `""`-value-prefixed one for `activation_message`'s
/// dispatch) — derived below from this one list, not written again.
pub const HTTP_METHODS: [&str; 5] = ["GET", "POST", "PUT", "PATCH", "DELETE"];

/// [`HTTP_METHODS`] with one extra leading option prepended — the rule
/// editor's own 6-item `Method` group, which (unlike Test Rule's) has an
/// "unconstrained" position at index 0. `leading` is `"Any"` for the
/// group's own *display* labels (the view's buttons, and `active_index`'s
/// own matching against them) and `""` for its *dispatch value* (what
/// `Message::RuleSetMethod` actually carries for that position) — two
/// different representations of the same position, not two different
/// lists.
pub fn editor_method_group(leading: &'static str) -> [&'static str; 6] {
    [
        leading,
        HTTP_METHODS[0],
        HTTP_METHODS[1],
        HTTP_METHODS[2],
        HTTP_METHODS[3],
        HTTP_METHODS[4],
    ]
}

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

/// The rule editor's own text-input/respond-field id strings, generated the
/// same way for the view (`.id(...)`) and for `Region::focus_id`'s
/// per-region focus dispatch (task 030 §5, generalising task 029's single
/// fixed id into one id *per* text-field region). `None` for a field with
/// no native text focus of its own.
pub fn editor_text_field_id(field: EditorField) -> Option<String> {
    match field {
        EditorField::UrlPath => Some("editor-url-path".into()),
        EditorField::HeaderName(i) => Some(format!("editor-header-name-{i}")),
        EditorField::HeaderValue(i) => Some(format!("editor-header-value-{i}")),
        EditorField::BodyPath(i) => Some(format!("editor-body-path-{i}")),
        EditorField::BodyValue(i) => Some(format!("editor-body-value-{i}")),
        EditorField::RespondBody => Some("editor-respond-body".into()),
        EditorField::RespondStatus => Some("editor-respond-status".into()),
        EditorField::RespondDelay => Some("editor-respond-delay".into()),
        EditorField::PerRuleField => Some("editor-per-rule-field".into()),
        _ => None,
    }
}

/// Settings' own text-input id strings, same reasoning as
/// `editor_text_field_id` — one lookup `Region::focus_id` and
/// `screens::settings::view`'s `.id(...)` calls both read. `None` for every
/// field with no native text focus of its own, including the two disabled
/// fields (`LogFile`, `QueueSize`) — the id still lets `focus()` move
/// native focus there, which is only consistent with a disabled field still
/// being click-focusable (see `SettingsField::LogFile`'s own doc comment).
pub fn settings_field_id(field: SettingsField) -> Option<String> {
    match field {
        SettingsField::Host => Some("mk023-settings-host".into()),
        SettingsField::Port => Some("mk023-settings-port".into()),
        SettingsField::LogFile => Some("mk023-settings-log-file".into()),
        SettingsField::QueueSize => Some("mk023-settings-queue-size".into()),
        _ => None,
    }
}

/// The Wizard's own text-input id strings, same reasoning as
/// `settings_field_id`.
pub fn wizard_field_id(field: WizardField) -> Option<String> {
    match field {
        WizardField::Name => Some("mk023-wizard-name".into()),
        WizardField::Folder => Some("mk023-wizard-folder".into()),
        WizardField::Host => Some("mk023-wizard-host".into()),
        WizardField::Port => Some("mk023-wizard-port".into()),
        WizardField::QueueSize => Some("mk023-wizard-queue-size".into()),
        _ => None,
    }
}

/// The currently selected rule's id and the payload it is actually editing
/// (a pending draft if one exists, else the rule's own payload) — the exact
/// lookup `screens::routes::centre_panel` performs before calling
/// `rule_editor()`, repeated here so the region model and the view can never
/// disagree about which payload is on screen.
fn editor_context(app: &App) -> Option<(NodeId, &apimokka_model::RulePayload)> {
    let rule_id = app.selection.rule?;
    let snap = app.snapshot.as_ref()?;
    let (_, rule) = snap.find_rule(rule_id)?;
    let payload = snap
        .rule_draft(rule_id)
        .map(|draft| &draft.payload)
        .unwrap_or(&rule.payload);
    Some((rule_id, payload))
}

/// The rule editor's fields, flattened into reading order (task 030 §2) —
/// the one source `screens::routes::rule_editor` renders from and
/// `Region::Editor`'s methods read. Mirrors `rule_editor()`'s own
/// visibility conditions exactly: Guided mode's headers/body disclosure,
/// each row's operator-dependent value field, and the per-rule field's
/// strategy/disclosure gate.
pub fn editor_fields(app: &App) -> Vec<EditorField> {
    let mut fields = Vec::new();
    let Some((_, p)) = editor_context(app) else {
        return fields;
    };

    fields.extend([
        EditorField::TestRule,
        EditorField::Duplicate,
        EditorField::MoveUp,
        EditorField::MoveDown,
        EditorField::Delete,
        EditorField::UrlPath,
        EditorField::UrlPathOp,
        EditorField::Method,
    ]);

    let show_when_more = !app.shows_scaffolding() || app.rule_when_more;
    if show_when_more {
        for i in 0..p.headers.len() {
            fields.push(EditorField::HeaderName(i));
            fields.push(EditorField::HeaderOp(i));
            if !p.headers[i].op.value_irrelevant() {
                fields.push(EditorField::HeaderValue(i));
            }
            fields.push(EditorField::HeaderRemove(i));
        }
        fields.push(EditorField::AddHeader);
        for i in 0..p.body.len() {
            fields.push(EditorField::BodyPath(i));
            fields.push(EditorField::BodyPathAssistant(i));
            fields.push(EditorField::BodyOp(i));
            if p.body[i].op != BodyOp::Exists && p.body[i].op != BodyOp::Absent {
                fields.push(EditorField::BodyValue(i));
            }
            fields.push(EditorField::BodyRemove(i));
        }
        fields.push(EditorField::AddBody);
    }
    if app.shows_scaffolding() {
        fields.push(EditorField::MoreWhenToggle);
    }

    fields.push(EditorField::RespondMode);
    fields.push(EditorField::RespondBody);
    fields.push(EditorField::RespondStatus);
    fields.push(EditorField::RespondDelay);

    let active_strategy = app
        .snapshot
        .as_ref()
        .map(|s| s.root_settings.strategy)
        .unwrap_or(apimokka_model::settings::Strategy::FirstMatch);
    if active_strategy.needs_per_rule_field() && show_when_more {
        fields.push(EditorField::PerRuleField);
    }

    fields
}

/// Whether `field` belongs to the rule editor's RESPOND column rather than
/// its WHEN column — task 032 (D-20): the editor's two scrollables scroll
/// independently, so `App::update_and_dispatch` must pick the matching one
/// rather than always the same one. `editor_fields(app)` filtered by this
/// predicate, in either direction, is each column's own field list in its
/// own reading order — the same list `editor_fields` already establishes,
/// not a second one that could drift from it.
pub fn is_respond_field(field: EditorField) -> bool {
    matches!(
        field,
        EditorField::RespondMode
            | EditorField::RespondBody
            | EditorField::RespondStatus
            | EditorField::RespondDelay
            | EditorField::PerRuleField
    )
}

/// The fallback file the editor is actually showing, if any — the same
/// lookup `centre_panel` performs before calling `fallback_file_editor`,
/// repeated here so the region model and the view can never disagree.
/// `None` whenever a rule is also selected: `centre_panel`'s own priority 1
/// (rule) outranks priority 2 (fallback file), and `SelectFileRoute` clears
/// `selection.rule` but the reverse is not guaranteed, so this checks it
/// explicitly rather than assuming the two are already mutually exclusive.
fn fallback_editor_context(app: &App) -> Option<&apimokka_model::node::FileNodeView> {
    if app.selection.rule.is_some() {
        return None;
    }
    let path = app.selection.file_route.as_ref()?;
    let snap = app.snapshot.as_ref()?;
    snap.fallback_files
        .iter()
        .find(|f| &f.name == path || &f.path == path)
}

/// The fallback editor's fields, in reading order — empty whenever
/// `fallback_editor_context` is `None`. `Content` first (it renders above
/// the footer), then the footer's own left-to-right order.
pub fn fallback_fields(app: &App) -> Vec<FallbackField> {
    if fallback_editor_context(app).is_none() {
        return Vec::new();
    }
    vec![
        FallbackField::Content,
        FallbackField::Status,
        FallbackField::Format,
        FallbackField::Revert,
        FallbackField::Save,
    ]
}

/// The `Id` string `settings_field_id`/`wizard_field_id`'s own pattern
/// generalises to the fallback editor's two focusable fields.
pub fn fallback_field_id(field: FallbackField) -> Option<String> {
    match field {
        FallbackField::Content => Some("mk023-fallback-content".into()),
        FallbackField::Status => Some("mk023-fallback-status".into()),
        _ => None,
    }
}

/// The rule set the configuration pane is actually showing, if any —
/// mirrors `centre_panel`'s own priority 4 exactly: only once none of
/// priorities 1–3 (rule, fallback file, script) claim the selection.
fn rule_set_config_context(app: &App) -> Option<&apimokka_model::snapshot::RuleSetView> {
    if app.selection.rule.is_some()
        || app.selection.file_route.is_some()
        || app.selection.script.is_some()
    {
        return None;
    }
    let rs_id = app.selection.rule_set?;
    let snap = app.snapshot.as_ref()?;
    snap.rule_sets.iter().find(|rs| rs.id == rs_id)
}

/// The rule-set configuration pane's fields, in reading order — empty
/// whenever `rule_set_config_context` is `None`. Mirrors
/// `rule_set_config`'s own three-way branching for the strategy picker
/// exactly (Guided collapsed / Guided expanded / Expert), the same shape
/// task 031 §2 already established for Settings' advanced disclosure: the
/// toggle renders *after* the picker once expanded, not before, and the
/// picker is absent entirely while collapsed.
pub fn rule_set_config_fields(app: &App) -> Vec<RuleSetConfigField> {
    if rule_set_config_context(app).is_none() {
        return Vec::new();
    }
    let mut fields = vec![RuleSetConfigField::Delete];
    if app.shows_scaffolding() {
        if app.rule_set_config_more {
            fields.push(RuleSetConfigField::Strategy);
            fields.push(RuleSetConfigField::MoreToggle);
        } else {
            fields.push(RuleSetConfigField::MoreToggle);
        }
    } else {
        fields.push(RuleSetConfigField::Strategy);
    }
    fields.push(RuleSetConfigField::AddRule);
    fields
}

/// Diagnostic rows the validation drawer actually renders as a clickable
/// button — `target.is_some()` is `validation_content`'s own condition for
/// wrapping a row in one, repeated here rather than assumed. Scoped by
/// position (`DrawerField::JumpToDiagnostic`'s own doc comment has why):
/// `target` is a `NodeId`, and a single rule can carry more than one
/// validation issue, so two rows can share one — checked against
/// `durable_diagnostic_rows` directly rather than assumed, the same
/// "confirmed, not assumed" standard task 030 held header/body rows to.
fn drawer_jump_target_count(app: &App) -> usize {
    crate::shell::bottom_drawer::durable_diagnostic_rows(app)
        .iter()
        .filter(|row| row.target.is_some())
        .count()
}

/// The bottom drawer's fields, in reading order — empty whenever
/// `app.drawer` is `None`. `Close` first (it renders above the body in
/// both modes), then whichever mode's own fields apply.
pub fn drawer_fields(app: &App) -> Vec<DrawerField> {
    let Some(mode) = app.drawer else {
        return Vec::new();
    };
    let mut fields = vec![DrawerField::Close];
    match mode {
        crate::selection::DrawerMode::Validation => {
            fields.extend((0..drawer_jump_target_count(app)).map(DrawerField::JumpToDiagnostic));
        }
        crate::selection::DrawerMode::SaveDiff => {
            if crate::shell::bottom_drawer::save_diff_total(app) > 0 {
                fields.push(DrawerField::Discard);
                fields.push(DrawerField::SaveAll);
            }
        }
    }
    fields
}

/// The Test Rule dialog's fields, in reading order — empty whenever the
/// dialog is closed. `Run` is forced to be the entry position by
/// `Message::TestRuleOpen`/`ReplayAsTestInput` directly, not by this
/// list's own ordering — see `TestRuleField::Run`'s own doc comment.
pub fn test_rule_fields(app: &App) -> Vec<TestRuleField> {
    if !app.test_rule.open {
        return Vec::new();
    }
    vec![
        TestRuleField::Dismiss,
        TestRuleField::Method,
        TestRuleField::Path,
        TestRuleField::Headers,
        TestRuleField::Body,
        TestRuleField::Close,
        TestRuleField::Run,
    ]
}

/// The `Id` string `fallback_field_id`'s own pattern generalises to Test
/// Rule's three text fields.
pub fn test_rule_field_id(field: TestRuleField) -> Option<String> {
    match field {
        TestRuleField::Path => Some("mk023-test-rule-path".into()),
        TestRuleField::Headers => Some("mk023-test-rule-headers".into()),
        TestRuleField::Body => Some("mk023-test-rule-body".into()),
        _ => None,
    }
}

/// The workspace menu's fields, in reading order — empty whenever the menu
/// is closed. `Recent` is omitted entirely when the list is empty, the
/// same "absent rather than an empty group" shape `RuleSetConfigField`'s
/// own absent `Strategy` (Guided, collapsed) already establishes.
pub fn workspace_menu_fields(app: &App) -> Vec<WorkspaceMenuField> {
    if !app.workspace_menu_open {
        return Vec::new();
    }
    let mut fields = Vec::new();
    if !apimokka_model::mock::recent_workspaces().is_empty() {
        fields.push(WorkspaceMenuField::Recent);
    }
    fields.push(WorkspaceMenuField::Open);
    fields.push(WorkspaceMenuField::Create);
    fields
}

/// The dotted-path assistant's fields, in reading order — empty whenever
/// the assistant is closed.
pub fn path_assistant_fields(app: &App) -> Vec<PathAssistantField> {
    if !app.path_assistant.open {
        return Vec::new();
    }
    let mut fields = vec![PathAssistantField::Close, PathAssistantField::Json];
    let leaf_count =
        crate::screens::dotted_path::parse_leaves(&app.path_assistant.json_input).len();
    fields.extend((0..leaf_count).map(PathAssistantField::UseLeaf));
    fields.push(PathAssistantField::Cancel);
    fields.push(PathAssistantField::Insert);
    fields
}

/// The `Id` string `fallback_field_id`'s own pattern generalises to the
/// assistant's one text field.
pub fn path_assistant_field_id(field: PathAssistantField) -> Option<String> {
    match field {
        PathAssistantField::Json => Some("mk023-path-assistant-json".into()),
        _ => None,
    }
}

/// The rule the rule editor's "recent trace activity" footer is actually
/// showing, if any — the identical lookup `centre_panel`'s own priority 1
/// performs, repeated here (rather than reusing `editor_context`, which
/// returns the draft *payload*, not the `RuleView` `recent_matching_events`
/// needs) so the region model and the view can never disagree.
fn trace_activity_context(app: &App) -> Option<&apimokka_model::snapshot::RuleView> {
    let rule_id = app.selection.rule?;
    let snap = app.snapshot.as_ref()?;
    snap.find_rule(rule_id).map(|(_, rule)| rule)
}

/// The rule editor's "recent trace activity" footer's fields, in reading
/// order — empty whenever no rule is selected. `JumpToEvent` is scoped by
/// position in `recent_matching_events`'s own output (at most 3), the one
/// source both this and `screens::routes::trace_activity` read, so the two
/// can never disagree about which event position N points to.
pub fn trace_activity_fields(app: &App) -> Vec<TraceActivityField> {
    let Some(rule) = trace_activity_context(app) else {
        return Vec::new();
    };
    let count = crate::screens::routes::trace_activity::recent_matching_events(app, rule).len();
    let mut fields = vec![TraceActivityField::ViewAll];
    fields.extend((0..count).map(TraceActivityField::JumpToEvent));
    fields
}

/// The feedback banner's fields, in reading order — empty whenever no
/// banner is showing. Derived from `shell::view::active_banner`, the one
/// function `shell::view::feedback_banner` itself branches on (task 034's
/// own named risk: "derive its field list from the same function
/// `feedback_banner` branches on, not from a second reading of the same
/// conditions").
pub fn feedback_banner_fields(app: &App) -> Vec<FeedbackBannerField> {
    match crate::shell::view::active_banner(app) {
        Some(crate::shell::view::ActiveBanner::Error(p)) => {
            let mut fields = Vec::new();
            if p.technical_detail.is_some() {
                fields.push(FeedbackBannerField::DetailsToggle);
            }
            if p.action_label.is_some() {
                fields.push(FeedbackBannerField::Action);
            }
            fields.push(FeedbackBannerField::Dismiss);
            fields
        }
        Some(crate::shell::view::ActiveBanner::Undo) => {
            vec![FeedbackBannerField::Undo, FeedbackBannerField::Dismiss]
        }
        Some(crate::shell::view::ActiveBanner::Notice) => vec![FeedbackBannerField::Dismiss],
        None => Vec::new(),
    }
}

/// Whether the Routes centre panel is showing its own empty-state call to
/// action right now — mirrors `centre_panel`'s own fallthrough exactly
/// (every one of its four priorities — rule, fallback file, script, rule
/// set — misses). The census missed this control in task 033; it is the
/// state the Routes tab opens in.
fn routes_shows_empty_cta(app: &App) -> bool {
    app.snapshot.is_some()
        && app.selection.rule.is_none()
        && app.selection.file_route.is_none()
        && app.selection.script.is_none()
        && app.selection.rule_set.is_none()
}

/// The empty-state call to action's own message — the one source both
/// `centre_panel`'s own button and `activation_message` call, rather than
/// two readings of the same expression that could drift.
pub(crate) fn routes_empty_cta_message(app: &App) -> Option<Message> {
    let snap = app.snapshot.as_ref()?;
    if snap.rule_sets.is_empty() {
        return Some(Message::AddRuleSet);
    }
    Some(if let Some(rs_id) = app.selection.rule_set {
        Message::AddRule(rs_id)
    } else {
        snap.rule_sets
            .first()
            .map(|rs| Message::AddRule(rs.id))
            .unwrap_or(Message::Noop)
    })
}

/// The top bar's fields, in reading order — empty whenever there is no
/// open workspace (`ws_section`'s own guard). `ReloadOrRestart` is omitted
/// entirely when `shell::top_bar::reload_or_restart` has nothing to offer
/// — the one source both this and the view read.
pub fn top_bar_fields(app: &App) -> Vec<TopBarField> {
    if app.snapshot.is_none() {
        return Vec::new();
    }
    let mut fields = vec![TopBarField::Switcher, TopBarField::SaveAll];
    if crate::shell::top_bar::reload_or_restart(app).is_some() {
        fields.push(TopBarField::ReloadOrRestart);
    }
    fields.push(TopBarField::StartStop);
    fields
}

/// Events passing the live trace filter, newest first — the one source
/// `screens::trace::view` and `Region::TraceEvents` both read. Matches
/// `trace.rs`'s former private `filtered_events` exactly: case-insensitive
/// substring on path, method, or outcome label; empty filter matches all.
pub fn filtered_trace_events(app: &App) -> Vec<&MatchTraceEvent> {
    let q = app.trace_filter.to_lowercase();
    app.trace
        .iter()
        .filter(|ev| {
            q.is_empty()
                || ev.request.url_path.to_lowercase().contains(&q)
                || ev.request.method.to_lowercase().contains(&q)
                || ev.outcome.label().contains(q.as_str())
        })
        .collect()
}

fn current_trace_event(app: &App) -> Option<&MatchTraceEvent> {
    let eid = app.selected_trace?;
    app.trace.iter().find(|e| e.event_id == eid)
}

/// The match-detail panel's outcome-specific action, if the selected
/// event's outcome provides one — mirrors `trace.rs`'s `outcome_matched`/
/// `outcome_fallback`/`outcome_miss` exactly, including `outcome_matched`'s
/// guard that the matched rule still exists. `Error` has none.
fn trace_outcome_action_message(app: &App) -> Option<Message> {
    let ev = current_trace_event(app)?;
    match &ev.outcome {
        TraceOutcome::Matched {
            rule_set_index,
            rule_index,
        } => {
            let snap = app.snapshot.as_ref()?;
            let rs = snap.rule_sets.get(*rule_set_index)?;
            let rule = rs.rules.get(*rule_index)?;
            Some(Message::JumpToRule(rule.id))
        }
        TraceOutcome::Fallback { file_path, .. } => Some(Message::JumpToFile(file_path.clone())),
        TraceOutcome::Miss { .. } => Some(Message::AddRuleForPath(ev.request.url_path.clone())),
        TraceOutcome::Error { .. } => None,
    }
}

impl Region {
    /// The regions navigable on the app's current screen, in `Tab` order.
    /// Mirrors `App::view`'s own precedence exactly (the mode picker
    /// short-circuits `view()` regardless of `self.view`, so it must
    /// short-circuit here too): at most one of the `view`-level branches
    /// below is ever non-empty. Owned (`Vec`, not `&'static [Region]`) since
    /// task 030: the editor's fields and the trace screen's conditional
    /// regions depend on live state, not just which screen is showing.
    pub fn order_for(app: &App) -> Vec<Region> {
        // Task 034 (D-22): every overlay's override now comes from the one
        // source `App::topmost_overlay` computes, instead of a second,
        // hand-written precedence here that had drifted from it (that
        // version ranked the drawer above the path assistant; the fixed
        // one does not — see `topmost_overlay`'s own doc comment for the
        // full reasoning). The palette keeps its own mechanism entirely
        // outside the region model (it is not a `Region`), so its own
        // entry returns empty, the same shape `blocks_region_navigation`
        // already gave it.
        match app.topmost_overlay() {
            Some(Overlay::ModePicker) => return vec![Region::ModePicker],
            Some(Overlay::ConfirmDialog) => return vec![Region::ConfirmDialog],
            Some(Overlay::CommandPalette) => return vec![],
            Some(Overlay::TestRule) => {
                return test_rule_fields(app)
                    .into_iter()
                    .map(Region::TestRule)
                    .collect();
            }
            Some(Overlay::PathAssistant) => {
                return path_assistant_fields(app)
                    .into_iter()
                    .map(Region::PathAssistant)
                    .collect();
            }
            Some(Overlay::WorkspaceMenu) => {
                return workspace_menu_fields(app)
                    .into_iter()
                    .map(Region::WorkspaceMenu)
                    .collect();
            }
            Some(Overlay::Drawer) => {
                return drawer_fields(app).into_iter().map(Region::Drawer).collect();
            }
            None => {}
        }
        match app.view {
            AppView::Workspace => {
                // Task 034: the top bar renders above the tab bar
                // (`shell::view::view`'s own `.header(...)`), and the RFC's
                // own Routes focus order names it first — so its fields
                // come before `TabBar`, not after.
                let mut order: Vec<Region> = top_bar_fields(app)
                    .into_iter()
                    .map(Region::TopBar)
                    .collect();
                order.push(Region::TabBar);
                // Task 034: the feedback banner renders between the tab bar
                // and the screen on every tab (`shell::view::view`'s own
                // layout) — its fields sit here, before the tab-specific
                // ones below, in the same visual order.
                order.extend(
                    feedback_banner_fields(app)
                        .into_iter()
                        .map(Region::FeedbackBanner),
                );
                match app.tab {
                    // Task 029: the sidebar only renders on the Routes tab
                    // (`screens::routes::view` is only reached there).
                    WorkspaceTab::Routes => {
                        order.push(Region::Sidebar);
                        // Task 033 (the census): the centre panel shows one
                        // of four things by sidebar selection
                        // (`centre_panel`'s own priority order) — only the
                        // first three contribute regions, each self-guarded
                        // to empty when its own precondition does not hold,
                        // so these three `extend`s are mutually exclusive
                        // in practice without needing an `if`/`else if`
                        // chain to say so here too.
                        order.extend(editor_fields(app).into_iter().map(Region::Editor));
                        // Task 034: the rule editor's own "recent trace
                        // activity" footer — below both WHEN/RESPOND
                        // columns in reading order, so after all of
                        // `editor_fields`, not interleaved with it.
                        order.extend(
                            trace_activity_fields(app)
                                .into_iter()
                                .map(Region::TraceActivity),
                        );
                        order.extend(fallback_fields(app).into_iter().map(Region::FallbackEditor));
                        // Priority 3 (script viewer) is read-only — nothing
                        // to add, confirmed at source, not missed.
                        order.extend(
                            rule_set_config_fields(app)
                                .into_iter()
                                .map(Region::RuleSetConfig),
                        );
                        // Task 034 (the census, F2): the empty state the
                        // Routes tab opens in — mutually exclusive with all
                        // four priorities above, self-guarded the same way.
                        if routes_shows_empty_cta(app) {
                            order.push(Region::RoutesEmptyCta);
                        }
                    }
                    WorkspaceTab::Trace => {
                        order.push(Region::TraceFilter);
                        order.push(Region::TracePause);
                        order.push(Region::TraceClear);
                        // Always declared, even when the filtered list is
                        // empty — `len` is 0 there, the same "empty region,
                        // no cursor" shape slices 3/4 already established,
                        // not a reason to omit the region itself.
                        order.push(Region::TraceEvents);
                        // Task 030 §4: these two exist only when rendered —
                        // derived here, never assumed, so a stale cursor can
                        // never point at a vanished match-detail action.
                        if trace_outcome_action_message(app).is_some() {
                            order.push(Region::TraceOutcomeAction);
                        }
                        if current_trace_event(app).is_some() {
                            order.push(Region::TraceReplay);
                        }
                    }
                    // Task 031 (MK-023 slice 6): mirrors
                    // `screens::settings::view`'s own three-way branching
                    // exactly (Guided collapsed / Guided expanded / Expert)
                    // — the toggle renders *after* the advanced fields once
                    // expanded, not before, since that is their actual
                    // order on screen.
                    WorkspaceTab::Settings => {
                        order.push(Region::Settings(SettingsField::Theme));
                        order.push(Region::Settings(SettingsField::AudienceMode));
                        order.push(Region::Settings(SettingsField::Locale));
                        order.push(Region::Settings(SettingsField::Host));
                        order.push(Region::Settings(SettingsField::Port));
                        order.push(Region::Settings(SettingsField::Tls));
                        if app.shows_scaffolding() {
                            if app.settings_advanced_more {
                                order.push(Region::Settings(SettingsField::LogFile));
                                order.push(Region::Settings(SettingsField::TraceEnable));
                                order.push(Region::Settings(SettingsField::QueueSize));
                            }
                            order.push(Region::Settings(SettingsField::AdvancedToggle));
                        } else {
                            order.push(Region::Settings(SettingsField::LogFile));
                            order.push(Region::Settings(SettingsField::TraceEnable));
                            order.push(Region::Settings(SettingsField::QueueSize));
                        }
                    }
                }
                order
            }
            AppView::Welcome => vec![Region::WelcomeActions],
            AppView::Dashboard => vec![
                Region::DashboardCreate,
                Region::DashboardSearch,
                Region::DashboardWorkspaces,
            ],
            // Task 031 §1: never assigned to a slice at all until now.
            // Mirrors `screens::wizard::view`'s own order exactly: required
            // fields, then each of the 3 collapsible sections (its own
            // toggle, then its body's fields only while open), then the
            // action bar.
            AppView::Wizard => {
                let mut order = vec![
                    Region::Wizard(WizardField::Name),
                    Region::Wizard(WizardField::Folder),
                    Region::Wizard(WizardField::ServerToggle),
                ];
                if app.wizard.section_open[0] {
                    order.push(Region::Wizard(WizardField::Host));
                    order.push(Region::Wizard(WizardField::Port));
                    order.push(Region::Wizard(WizardField::Tls));
                }
                order.push(Region::Wizard(WizardField::StarterToggle));
                if app.wizard.section_open[1] {
                    order.push(Region::Wizard(WizardField::Starter));
                }
                order.push(Region::Wizard(WizardField::TraceToggle));
                if app.wizard.section_open[2] {
                    order.push(Region::Wizard(WizardField::TraceEnable));
                    order.push(Region::Wizard(WizardField::QueueSize));
                }
                order.push(Region::Wizard(WizardField::Cancel));
                order.push(Region::Wizard(WizardField::Create));
                order
            }
        }
    }

    /// Whether this region's cursor is an identity (`Cursor::WorkspaceName`/
    /// `Cursor::Sidebar`/`Cursor::TraceEvent`) rather than a plain
    /// `Cursor::Index` — task 027's "identity where the list can change"
    /// rule.
    pub fn uses_identity_cursor(self) -> bool {
        matches!(
            self,
            Region::DashboardWorkspaces | Region::Sidebar | Region::TraceEvents
        )
    }

    /// The `Id` string `update_and_dispatch` focuses when this region is
    /// entered, for every text-field region — `Region::DashboardSearch`'s
    /// and `Region::TraceFilter`'s fixed constants, or one of the editor's
    /// many generated per-row ids. `None` for every other region. One
    /// lookup point generalises task 028's single fixed id (good for only
    /// one region) to every text-field region this and later slices add.
    pub fn focus_id(self) -> Option<String> {
        match self {
            Region::DashboardSearch => Some(crate::screens::dashboard::SEARCH_INPUT_ID.into()),
            Region::TraceFilter => Some(crate::screens::trace::FILTER_INPUT_ID.into()),
            Region::Editor(field) => editor_text_field_id(field),
            Region::Settings(field) => settings_field_id(field),
            Region::Wizard(field) => wizard_field_id(field),
            Region::FallbackEditor(field) => fallback_field_id(field),
            Region::TestRule(field) => test_rule_field_id(field),
            Region::PathAssistant(field) => path_assistant_field_id(field),
            _ => None,
        }
    }

    /// Whether this region's arrows change a value in place rather than
    /// moving the cursor — task 030 §3: the only behaviour a `pick_list`
    /// can expose to the keyboard in iced 0.14, which implements no
    /// `Focusable` and exposes no way to open its popup except a pointer
    /// click.
    pub fn is_value_cycler(self) -> bool {
        matches!(
            self,
            Region::Editor(EditorField::UrlPathOp)
                | Region::Editor(EditorField::HeaderOp(_))
                | Region::Editor(EditorField::BodyOp(_))
                | Region::Settings(SettingsField::Locale)
                | Region::RuleSetConfig(RuleSetConfigField::Strategy)
        )
    }

    /// The message an arrow key dispatches for a `pick_list` region
    /// (`is_value_cycler`), moving `delta` positions through the option
    /// list and saturating at both ends — never wrapping, the same rule
    /// every other region's arrows already follow. `None` for every other
    /// region, or if the rule/row the picker belongs to no longer exists.
    pub fn cycle_value(self, app: &App, delta: isize) -> Option<Message> {
        fn step<T: Copy + PartialEq>(options: &[T], current: Option<T>, delta: isize) -> T {
            let pos = current
                .and_then(|c| options.iter().position(|&o| o == c))
                .map(|i| i as isize)
                .unwrap_or(if delta > 0 { -1 } else { 0 });
            let next = (pos + delta).clamp(0, options.len() as isize - 1);
            options[next as usize]
        }

        match self {
            Region::Editor(EditorField::UrlPathOp) => {
                let (_, p) = editor_context(app)?;
                let options = UrlPathOp::all();
                Some(Message::RuleSetUrlPathOp(step(
                    &options,
                    p.url_path_op,
                    delta,
                )))
            }
            Region::Editor(EditorField::HeaderOp(i)) => {
                let (_, p) = editor_context(app)?;
                let h = p.headers.get(i)?;
                let options = HeaderOp::all();
                Some(Message::HeaderSetOp {
                    index: i,
                    op: step(&options, Some(h.op), delta),
                })
            }
            Region::Editor(EditorField::BodyOp(i)) => {
                let (_, p) = editor_context(app)?;
                let b = p.body.get(i)?;
                let options = BodyOp::all();
                Some(Message::BodySetOp {
                    index: i,
                    op: step(&options, Some(b.op), delta),
                })
            }
            // Task 031 §3/§4: same `pick_list` compromise as the editor's
            // three cyclers, D-19 and all — not fixed here, per the
            // handoff's own instruction that one remedy in slice 7 should
            // cover all four at once.
            Region::Settings(SettingsField::Locale) => {
                let options = apimokka_i18n::Locale::all();
                Some(Message::ChangeLocale(step(
                    options,
                    Some(app.locale),
                    delta,
                )))
            }
            // Task 033 (D-19's mechanism, the fourth cycler's sibling): same
            // compromise, not fixed here either — one remedy already
            // applied to all four in slice 7 covers this fifth instance
            // with no change.
            Region::RuleSetConfig(RuleSetConfigField::Strategy) => {
                rule_set_config_context(app)?;
                let strategy = app
                    .snapshot
                    .as_ref()
                    .map(|s| s.root_settings.strategy)
                    .unwrap_or(apimokka_model::settings::Strategy::FirstMatch);
                let options = apimokka_model::settings::Strategy::all();
                Some(Message::RuleSetSetStrategy(step(
                    &options,
                    Some(strategy),
                    delta,
                )))
            }
            _ => None,
        }
    }

    /// Number of items in this region right now. Fixed for every one-item
    /// region; the two multi-item groups (`Method`: 6, `RespondMode`: 2)
    /// are fixed too, just not 1; the identity-cursored regions depend on
    /// live state (the search filter; the accordion and section toggles;
    /// the trace list).
    pub fn len(self, app: &App) -> usize {
        match self {
            Region::ModePicker => crate::screens::mode_picker::OPTIONS.len(),
            Region::TabBar => TAB_BAR_ORDER.len(),
            Region::WelcomeActions => WELCOME_ACTIONS.len(),
            Region::DashboardCreate | Region::DashboardSearch => 1,
            Region::DashboardWorkspaces | Region::Sidebar | Region::TraceEvents => {
                self.identity_list(app).len()
            }
            Region::Editor(EditorField::Method) => editor_method_group("").len(),
            Region::Editor(EditorField::RespondMode) => 2,
            Region::Editor(_) => 1,
            Region::TraceFilter | Region::TracePause | Region::TraceClear => 1,
            Region::TraceOutcomeAction | Region::TraceReplay => 1,
            Region::Settings(SettingsField::Theme) => crate::app::ThemeChoice::all().len(),
            Region::Settings(SettingsField::AudienceMode) => {
                apimokka_model::AudienceMode::all().len()
            }
            Region::Settings(_) => 1,
            Region::Wizard(WizardField::Starter) => WIZARD_STARTER_ORDER.len(),
            Region::Wizard(_) => 1,
            Region::FallbackEditor(_) => 1,
            Region::RuleSetConfig(_) => 1,
            Region::Drawer(_) => 1,
            // Cancel, Proceed.
            Region::ConfirmDialog => 2,
            Region::TestRule(TestRuleField::Method) => HTTP_METHODS.len(),
            Region::TestRule(_) => 1,
            Region::WorkspaceMenu(WorkspaceMenuField::Recent) => {
                apimokka_model::mock::recent_workspaces().len()
            }
            Region::WorkspaceMenu(_) => 1,
            Region::PathAssistant(_) => 1,
            Region::TraceActivity(_) => 1,
            Region::FeedbackBanner(_) => 1,
            Region::RoutesEmptyCta => 1,
            Region::TopBar(_) => 1,
        }
    }

    /// The index that counts as "active" — where entering this region
    /// starts the cursor (task 027 §3.1). `None` when nothing is active yet,
    /// in which case the cursor starts at the first item instead.
    /// `DashboardWorkspaces` always returns `None`: nothing is "the open
    /// workspace" while choosing one to open. `Sidebar` resumes whatever the
    /// centre panel currently shows, in the same rule > file > script > rule
    /// set priority `screens::routes::centre_panel` itself uses (so the two
    /// can never disagree about what "currently showing" means). The two
    /// multi-item editor groups resume on the value the rule's payload
    /// already has; every one-item region has nothing to resume (always the
    /// single item it is).
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
            Region::Editor(EditorField::Method) => {
                let (_, p) = editor_context(app)?;
                let methods = editor_method_group("Any");
                methods.iter().position(|&m| {
                    if m == "Any" {
                        p.method.is_empty()
                    } else {
                        p.method == m
                    }
                })
            }
            Region::Editor(EditorField::RespondMode) => {
                let (_, p) = editor_context(app)?;
                Some(
                    if p.respond.mode == apimokka_model::respond::RespondMode::InlineText {
                        0
                    } else {
                        1
                    },
                )
            }
            Region::Editor(_) => None,
            Region::TraceFilter | Region::TracePause | Region::TraceClear => None,
            Region::TraceEvents => None,
            Region::TraceOutcomeAction | Region::TraceReplay => None,
            Region::Settings(SettingsField::Theme) => crate::app::ThemeChoice::all()
                .iter()
                .position(|&c| c == app.theme_choice),
            Region::Settings(SettingsField::AudienceMode) => app.audience_mode.and_then(|m| {
                apimokka_model::AudienceMode::all()
                    .iter()
                    .position(|&x| x == m)
            }),
            Region::Settings(_) => None,
            Region::Wizard(WizardField::Starter) => WIZARD_STARTER_ORDER
                .iter()
                .position(|&s| s == app.wizard.starter),
            Region::Wizard(_) => None,
            Region::FallbackEditor(_) => None,
            Region::RuleSetConfig(_) => None,
            Region::Drawer(_) => None,
            // Always starts on Cancel (index 0) — task 026 §4.2's safety
            // property: a bare `Enter` with no `Tab` at all must still
            // cancel, never proceed.
            Region::ConfirmDialog => None,
            // Not read in practice — `Message::TestRuleOpen`/
            // `ReplayAsTestInput` force `Run`'s own entry directly, the
            // same reason `ConfirmDialog` returns `None` here despite
            // always starting on a specific item.
            Region::TestRule(TestRuleField::Method) => HTTP_METHODS
                .iter()
                .position(|&m| app.test_rule.method.to_uppercase() == m),
            Region::TestRule(_) => None,
            Region::WorkspaceMenu(WorkspaceMenuField::Recent) => {
                let cur = app.snapshot.as_ref()?.identity.name.as_str();
                apimokka_model::mock::recent_workspaces()
                    .iter()
                    .position(|ws| ws.name == cur)
            }
            Region::WorkspaceMenu(_) => None,
            Region::PathAssistant(_) => None,
            Region::TraceActivity(_) => None,
            Region::FeedbackBanner(_) => None,
            Region::RoutesEmptyCta => None,
            Region::TopBar(_) => None,
        }
    }

    /// The live identity list for an identity-cursored region — the one
    /// source `identity_at`, `App::move_region_cursor`'s identity branch and
    /// `App::reclamp_identity_cursor` all read, so a region's cursor and its
    /// rendered list can never disagree about what currently exists (the D-9
    /// lesson, generalised from task 028's single case through task 029's
    /// second one to task 030's third). Empty for every index-cursored
    /// region.
    pub fn identity_list(self, app: &App) -> Vec<Cursor> {
        match self {
            Region::DashboardWorkspaces => dashboard_filtered_workspaces(app)
                .into_iter()
                .map(|ws| Cursor::WorkspaceName(ws.name))
                .collect(),
            Region::Sidebar => sidebar_rows(app).into_iter().map(Cursor::Sidebar).collect(),
            Region::TraceEvents => filtered_trace_events(app)
                .into_iter()
                .map(|ev| Cursor::TraceEvent(ev.event_id))
                .collect(),
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
    /// interaction is typing, cycling, or expansion, not activation.
    pub fn activation_message(self, app: &App, cursor: &Cursor) -> Option<Message> {
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
            (Region::Editor(field), Cursor::Index(i)) => {
                let (rule_id, p) = editor_context(app)?;
                match field {
                    EditorField::TestRule => {
                        let has_when = !p.url_path.is_empty()
                            || !p.method.is_empty()
                            || !p.headers.is_empty()
                            || !p.body.is_empty();
                        has_when.then_some(Message::TestRuleOpen)
                    }
                    EditorField::Duplicate => Some(Message::DuplicateRule(rule_id)),
                    EditorField::MoveUp => Some(Message::MoveRuleUp(rule_id)),
                    EditorField::MoveDown => Some(Message::MoveRuleDown(rule_id)),
                    EditorField::Delete => Some(Message::DeleteRule(rule_id)),
                    EditorField::UrlPath => None,
                    // D-19 (task 032): `Enter` on a value-cycler advances it
                    // one step forward — the same message `ArrowDown` would
                    // dispatch via `cycle_value` — so the natural key for an
                    // apparently-openable control is not dead.
                    EditorField::UrlPathOp => self.cycle_value(app, 1),
                    EditorField::Method => editor_method_group("")
                        .get(*i)
                        .map(|m| Message::RuleSetMethod(m.to_string())),
                    EditorField::MoreWhenToggle => Some(Message::ToggleRuleWhenMore),
                    EditorField::HeaderName(_) | EditorField::HeaderValue(_) => None,
                    // D-19: same reasoning as `UrlPathOp` above.
                    EditorField::HeaderOp(_) => self.cycle_value(app, 1),
                    EditorField::HeaderRemove(i) => Some(Message::HeaderRemove(i)),
                    EditorField::AddHeader => Some(Message::HeaderAdd),
                    EditorField::BodyPath(_) | EditorField::BodyValue(_) => None,
                    // D-19: same reasoning as `UrlPathOp` above.
                    EditorField::BodyOp(_) => self.cycle_value(app, 1),
                    EditorField::BodyPathAssistant(i) => Some(Message::PathAssistantOpen(i)),
                    EditorField::BodyRemove(i) => Some(Message::BodyRemove(i)),
                    EditorField::AddBody => Some(Message::BodyAdd),
                    EditorField::RespondMode => {
                        use apimokka_model::respond::RespondMode;
                        let modes = [RespondMode::InlineText, RespondMode::ServeFile];
                        modes.get(*i).map(|&m| Message::RespondSetMode(m))
                    }
                    EditorField::RespondBody
                    | EditorField::RespondStatus
                    | EditorField::RespondDelay
                    | EditorField::PerRuleField => None,
                }
            }
            (Region::TraceFilter, _) => None,
            (Region::TracePause, _) => Some(Message::TracePauseToggle),
            (Region::TraceClear, _) => Some(Message::TraceClear),
            (Region::TraceEvents, Cursor::TraceEvent(id)) => Some(Message::SelectTraceEvent(*id)),
            (Region::TraceOutcomeAction, _) => trace_outcome_action_message(app),
            (Region::TraceReplay, _) => {
                current_trace_event(app).map(|ev| Message::ReplayAsTestInput(ev.event_id))
            }
            (Region::Settings(SettingsField::Theme), Cursor::Index(i)) => {
                crate::app::ThemeChoice::all()
                    .get(*i)
                    .map(|&c| Message::SetTheme(c))
            }
            (Region::Settings(SettingsField::AudienceMode), Cursor::Index(i)) => {
                apimokka_model::AudienceMode::all()
                    .get(*i)
                    .map(|&m| Message::ChooseAudienceMode(m))
            }
            // D-19 (task 032): same reasoning as `UrlPathOp` above — applies
            // to all four cyclers, not only the editor's three.
            (Region::Settings(SettingsField::Locale), _) => self.cycle_value(app, 1),
            // Typing or disabled — not activation (task 031 §8: `LogFile`
            // and `QueueSize` have no `on_input` wired at all; `Enter` must
            // agree with the click that already does nothing).
            (
                Region::Settings(
                    SettingsField::Host
                    | SettingsField::Port
                    | SettingsField::LogFile
                    | SettingsField::QueueSize,
                ),
                _,
            ) => None,
            (Region::Settings(SettingsField::Tls), _) => {
                let snap = app.snapshot.as_ref()?;
                Some(Message::SettingsSetTls(!snap.root_settings.tls_enabled))
            }
            (Region::Settings(SettingsField::AdvancedToggle), _) => {
                Some(Message::ToggleSettingsAdvancedMore)
            }
            (Region::Settings(SettingsField::TraceEnable), _) => {
                let enabled = app
                    .snapshot
                    .as_ref()?
                    .prototype
                    .trace
                    .as_ref()
                    .map(|t| t.enabled)
                    .unwrap_or(false);
                Some(Message::SettingsSetTraceEnabled(!enabled))
            }
            (Region::Wizard(WizardField::ServerToggle), _) => Some(Message::WizardToggleSection(0)),
            (Region::Wizard(WizardField::StarterToggle), _) => {
                Some(Message::WizardToggleSection(1))
            }
            (Region::Wizard(WizardField::TraceToggle), _) => Some(Message::WizardToggleSection(2)),
            (Region::Wizard(WizardField::Starter), Cursor::Index(i)) => WIZARD_STARTER_ORDER
                .get(*i)
                .map(|&s| Message::WizardSetStarter(s)),
            (Region::Wizard(WizardField::Tls), _) => Some(Message::WizardSetTls(!app.wizard.tls)),
            (Region::Wizard(WizardField::Cancel), _) => Some(Message::WizardCancel),
            (Region::Wizard(WizardField::Create), _) => Some(Message::WizardCreate),
            // Typing or disabled — not activation. `TraceEnable` here is
            // the Wizard's hardcoded-`true`, no-`on_toggle` checkbox (task
            // 031 §8), the parallel case `SettingsField::LogFile`'s doc
            // comment points to.
            (
                Region::Wizard(
                    WizardField::Name
                    | WizardField::Folder
                    | WizardField::Host
                    | WizardField::Port
                    | WizardField::TraceEnable
                    | WizardField::QueueSize,
                ),
                _,
            ) => None,
            // Task 033: the fallback editor. `Content` is a `text_editor`,
            // not activation — typing is its own interaction, same as every
            // other text-field region.
            (Region::FallbackEditor(FallbackField::Content), _) => None,
            (Region::FallbackEditor(FallbackField::Status), _) => None,
            (Region::FallbackEditor(FallbackField::Format), _) => Some(Message::FallbackFileFormat),
            (Region::FallbackEditor(FallbackField::Revert), _) => {
                let path = app.selection.file_route.clone()?;
                app.is_fallback_dirty(&path)
                    .then_some(Message::FallbackFileRevert)
            }
            (Region::FallbackEditor(FallbackField::Save), _) => {
                let path = app.selection.file_route.clone()?;
                app.is_fallback_dirty(&path)
                    .then_some(Message::FallbackFileSave)
            }
            // Task 033: the rule-set configuration pane.
            (Region::RuleSetConfig(RuleSetConfigField::Delete), _) => {
                let rs = rule_set_config_context(app)?;
                Some(Message::DeleteRuleSet(rs.id))
            }
            // D-19's mechanism (`cycle_value`'s own doc comment above).
            (Region::RuleSetConfig(RuleSetConfigField::Strategy), _) => self.cycle_value(app, 1),
            (Region::RuleSetConfig(RuleSetConfigField::MoreToggle), _) => {
                Some(Message::ToggleRuleSetConfigMore)
            }
            (Region::RuleSetConfig(RuleSetConfigField::AddRule), _) => {
                let rs = rule_set_config_context(app)?;
                Some(Message::AddRule(rs.id))
            }
            // Task 033: the bottom drawer.
            (Region::Drawer(DrawerField::Close), _) => Some(Message::CloseDrawer),
            (Region::Drawer(DrawerField::JumpToDiagnostic(i)), _) => {
                let rows = crate::shell::bottom_drawer::durable_diagnostic_rows(app);
                rows.iter()
                    .filter(|row| row.target.is_some())
                    .nth(i)
                    .and_then(|row| row.target)
                    .map(Message::JumpToDiagnostic)
            }
            // Task 034 (D-24's confirmation half): routed through the
            // confirm dialog instead of discarding directly — a
            // confirmation already existed for this (`ConfirmAction::
            // DiscardChanges`, with its own title/body keys) and dispatched
            // nothing; two keystrokes could silently discard unsaved work.
            (Region::Drawer(DrawerField::Discard), _) => {
                Some(Message::ConfirmRequest(ConfirmAction::DiscardChanges))
            }
            (Region::Drawer(DrawerField::SaveAll), _) => Some(Message::SaveAll),
            // Task 033: the confirm dialog. `Cursor::Index(0)` is `Cancel`
            // (the entry position — task 026 §4.2's safety property);
            // `Cursor::Index(1)` is `Proceed`, reachable only by a
            // deliberate `ArrowDown` first.
            (Region::ConfirmDialog, Cursor::Index(0)) => Some(Message::ConfirmCancel),
            (Region::ConfirmDialog, Cursor::Index(1)) => Some(Message::ConfirmProceed),
            // Task 034: the Test Rule dialog. `Path`/`Headers`/`Body` are
            // the one deliberate exception to "a text-field region has no
            // activation" — see `TestRuleField`'s own doc comment.
            (Region::TestRule(TestRuleField::Dismiss), _) => Some(Message::TestRuleClose),
            (Region::TestRule(TestRuleField::Method), Cursor::Index(i)) => HTTP_METHODS
                .get(*i)
                .map(|m| Message::TestRuleSetMethod(m.to_string())),
            (Region::TestRule(TestRuleField::Method), _) => None,
            (
                Region::TestRule(
                    TestRuleField::Path | TestRuleField::Headers | TestRuleField::Body,
                ),
                _,
            ) => Some(Message::TestRuleRun),
            (Region::TestRule(TestRuleField::Close), _) => Some(Message::TestRuleClose),
            (Region::TestRule(TestRuleField::Run), _) => Some(Message::TestRuleRun),
            // Task 034: the workspace menu.
            (Region::WorkspaceMenu(WorkspaceMenuField::Recent), Cursor::Index(i)) => {
                apimokka_model::mock::recent_workspaces()
                    .get(*i)
                    .map(|ws| Message::OpenWorkspace(ws.name.clone()))
            }
            (Region::WorkspaceMenu(WorkspaceMenuField::Open), _) => Some(Message::GoDashboard),
            (Region::WorkspaceMenu(WorkspaceMenuField::Create), _) => Some(Message::GoWizard),
            // Task 034: the dotted-path assistant.
            (Region::PathAssistant(PathAssistantField::Close), _) => {
                Some(Message::PathAssistantClose)
            }
            (Region::PathAssistant(PathAssistantField::Json), _) => None,
            (Region::PathAssistant(PathAssistantField::UseLeaf(i)), _) => {
                crate::screens::dotted_path::parse_leaves(&app.path_assistant.json_input)
                    .get(i)
                    .map(|(path, _)| Message::PathAssistantSelectPath(path.clone()))
            }
            (Region::PathAssistant(PathAssistantField::Cancel), _) => {
                Some(Message::PathAssistantClose)
            }
            (Region::PathAssistant(PathAssistantField::Insert), _) => {
                Some(Message::PathAssistantInsert)
            }
            // Task 034: the rule editor's "recent trace activity" footer.
            (Region::TraceActivity(TraceActivityField::ViewAll), _) => {
                Some(Message::ViewAllInTrace)
            }
            (Region::TraceActivity(TraceActivityField::JumpToEvent(i)), _) => {
                let rule = trace_activity_context(app)?;
                crate::screens::routes::trace_activity::recent_matching_events(app, rule)
                    .get(i)
                    .map(|ev| Message::JumpToTraceEvent(ev.event_id))
            }
            // Task 034: the feedback banner. `Dismiss` re-derives which
            // banner is showing rather than fixing one message to the
            // field — see `FeedbackBannerField::Dismiss`'s own doc comment.
            (Region::FeedbackBanner(FeedbackBannerField::DetailsToggle), _) => {
                Some(Message::ToggleProblemDetails)
            }
            (Region::FeedbackBanner(FeedbackBannerField::Action), _) => {
                Some(Message::ProblemAction)
            }
            (Region::FeedbackBanner(FeedbackBannerField::Dismiss), _) => {
                match crate::shell::view::active_banner(app) {
                    Some(crate::shell::view::ActiveBanner::Error(_)) => {
                        Some(Message::DismissProblem)
                    }
                    Some(
                        crate::shell::view::ActiveBanner::Undo
                        | crate::shell::view::ActiveBanner::Notice,
                    ) => Some(Message::DismissNotice),
                    None => None,
                }
            }
            (Region::FeedbackBanner(FeedbackBannerField::Undo), _) => Some(Message::Undo),
            // Task 034: the Routes centre panel's own empty-state call to
            // action.
            (Region::RoutesEmptyCta, _) => routes_empty_cta_message(app),
            // Task 034: the top bar.
            (Region::TopBar(TopBarField::Switcher), _) => Some(Message::ToggleWorkspaceMenu),
            (Region::TopBar(TopBarField::SaveAll), _) => {
                (app.dirty_count > 0).then_some(Message::Save)
            }
            (Region::TopBar(TopBarField::ReloadOrRestart), _) => {
                crate::shell::top_bar::reload_or_restart(app).map(|(_, msg)| msg)
            }
            (Region::TopBar(TopBarField::StartStop), _) => {
                crate::shell::top_bar::start_stop_message(app)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "regions/tests.rs"]
mod tests;
