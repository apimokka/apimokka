//! All messages (MK-021, MK-035).

use apimokka_model::respond::RespondMode;
use apimokka_model::settings::Strategy;
use apimokka_model::{BodyOp, HeaderOp, NodeId, RuleSetId, UrlPathOp};

use crate::selection::WorkspaceTab;

/// Destructive-action categories for the confirm dialog (MK-034).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ConfirmAction {
    /// Kept here in case a future RFC re-introduces a delete-rule confirmation;
    /// currently delete-rule is non-modal (MK-039) so this variant is never dispatched.
    /// Replaced at runtime by Message::DeleteRule which goes through the undo stack.
    DeleteRuleSet(RuleSetId),
    DiscardChanges,
    SwitchWorkspace(String),
    LeaveWorkspace,
    CreateWorkspace,
    /// Revert a fallback file's draft to its saved baseline (MK-038).
    RevertFile(String),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    // ── Navigation ────────────────────────────────────────────────────────
    GoWelcome,
    GoDashboard,
    GoWizard,
    OpenWorkspace(String),
    SwitchTab(WorkspaceTab),

    // ── Locale / theme ────────────────────────────────────────────────────
    ChangeLocale(apimokka_i18n::Locale),
    /// Task 032 §6: the palette's locale row (`Key::PaletteCmdLocale`,
    /// orphaned since D-10) — state-independent, same shape as
    /// `ToggleTheme`, since `palette_commands::Command.message` is a
    /// zero-argument `fn` that cannot read which locale is current.
    ToggleLocale,
    ToggleTheme,
    /// MK-050: pick a specific theme (Light / Dark / HC Light / HC Dark).
    SetTheme(crate::app::ThemeChoice),
    /// Task 032 §6: the palette's audience-mode row — same reasoning as
    /// `ToggleLocale`.
    ToggleAudienceMode,

    // ── Keyboard ──────────────────────────────────────────────────────────
    EscapePressed,
    ToggleCommandPalette,
    /// MK-033 §3 / task 014 §4: moves the palette's row selection when it is
    /// open. Otherwise moves the keyboard cursor within the screen's
    /// currently focused region (task 027, MK-023 slice 2) — the mode
    /// picker's cards, the tab bar, or whichever later region holds focus.
    /// A no-op where neither applies.
    ArrowUp,
    ArrowDown,
    /// Task 029 (MK-023 slice 4): on the sidebar, expands the cursor's row,
    /// or moves to its first child when already expanded; on every other
    /// region, a no-op (no region has Left/Right semantics of its own yet).
    /// Captured by a focused `text_input` (unlike `ArrowUp`/`ArrowDown`/
    /// `Tab`) — see `App::update_and_dispatch`'s §5 generalisation.
    ArrowRight,
    /// The reverse of `ArrowRight`: collapses the cursor's row, or moves to
    /// its parent when it has none to collapse.
    ArrowLeft,
    /// Task 027 (MK-023 slice 2): moves the keyboard cursor to the next
    /// region in the current screen's declared order, wrapping. The dialog
    /// region (task 026) takes priority when one is open, same as `Enter`.
    TabPressed,
    /// The reverse of `TabPressed`.
    ShiftTabPressed,
    /// MK-033 line 92 / task 014 §4: executes the palette's selected row when
    /// it is open; otherwise activates the focused region's cursor item
    /// (task 027) — confirming the mode picker's card, switching the tab
    /// bar's tab, or (task 026) the open dialog's default action, which
    /// takes priority over all of the above. A no-op where none applies.
    EnterPressed,
    /// Task 032 (D-18): a click on a text-field region's own widget enters
    /// that region — `focused_region` and `region_cursor` follow native
    /// focus instead of staying wherever `Tab` last left them. Dispatched
    /// from `iced::widget::mouse_area::on_release`, not `on_press`: a
    /// focused `text_input`'s own `update()` captures `ButtonPressed`
    /// (positioning its native text cursor) but not `ButtonReleased`, so
    /// `on_release` is the one mouse-area hook that still fires on a click
    /// landing on the field itself, confirmed against
    /// `iced_widget::text_input`'s own event handling rather than assumed.
    RegionClicked(crate::regions::Region),

    // ── Workspace switcher ────────────────────────────────────────────────
    ToggleWorkspaceMenu,
    CloseWorkspaceMenu,

    // ── Server actions ────────────────────────────────────────────────────
    StartStopServer,
    ReloadConfig,
    RestartServer,
    RuntimeSucceeded(crate::app::RuntimeRequestToken),
    RuntimeFailed {
        token: crate::app::RuntimeRequestToken,
        technical: String,
    },

    // ── Save ──────────────────────────────────────────────────────────────
    Save,
    SaveAll,
    DiscardChanges,

    // ── Bottom drawer ─────────────────────────────────────────────────────
    OpenValidationDrawer,
    OpenSaveDiffDrawer,
    CloseDrawer,

    // ── Wizard ────────────────────────────────────────────────────────────
    WizardSetName(String),
    WizardSetStarter(crate::app::WizardStarter), // MK-048
    WizardSetFolder(String),
    WizardToggleSection(usize), // which section to expand/collapse
    WizardSetHost(String),
    WizardSetPort(String),
    WizardSetTls(bool),
    WizardSetQueueSize(String),
    WizardCreate,
    WizardCancel,

    // ── Rule set tree ─────────────────────────────────────────────────────
    SelectRuleSet(RuleSetId),
    SelectRule(NodeId),
    AddRuleSet,
    AddRule(RuleSetId),
    DeleteRuleSet(RuleSetId), // goes through confirm
    DeleteRule(NodeId),       // goes through confirm
    DuplicateRule(NodeId),
    MoveRuleUp(NodeId),
    MoveRuleDown(NodeId),
    SelectFileRoute(String),
    SelectScript(String),

    // ── Rule editor: URL path ─────────────────────────────────────────────
    RuleSetUrlPath(String),
    RuleSetUrlPathOp(UrlPathOp),
    RuleSetUrlPathEnabled(bool),

    // ── Rule editor: method ───────────────────────────────────────────────
    RuleSetMethod(String),

    // ── Rule editor: headers ──────────────────────────────────────────────
    HeaderAdd,
    HeaderRemove(usize),
    HeaderSetName {
        index: usize,
        value: String,
    },
    HeaderSetOp {
        index: usize,
        op: HeaderOp,
    },
    HeaderSetValue {
        index: usize,
        value: String,
    },
    HeaderClearAll,

    // ── Rule editor: body conditions ──────────────────────────────────────
    BodyAdd,
    BodyRemove(usize),
    BodySetPath {
        index: usize,
        value: String,
    },
    BodySetOp {
        index: usize,
        op: BodyOp,
    },
    BodySetValue {
        index: usize,
        value: String,
    },
    BodyClearAll,
    BodyOpenPathAssistant(usize), // open dotted-path assistant for row N

    // ── Rule editor: respond ──────────────────────────────────────────────
    RespondSetMode(RespondMode),
    RespondSetText(String),
    RespondSetFilePath(String),
    RespondSetStatus(String),
    RespondSetDelay(String),

    // ── Strategy (inspector) ──────────────────────────────────────────────
    RuleSetWeight(String),
    RuleSetPriority(String),

    // ── Trace tab ────────────────────────────────────────────────────────
    TracePauseToggle,
    TraceClear,
    SelectTraceEvent(u64),
    /// MK-042: live filter input changed.
    TraceFilterChanged(String),
    /// MK-042: Miss CTA — add a rule pre-populated with this URL path, then jump to Routes.
    AddRuleForPath(String),
    /// MK-044: palette "Add rule" — add to first rule set and navigate.
    AddRuleFromPalette,
    /// MK-042: from trace detail "Jump to rule" — select the rule AND switch to Routes tab.
    JumpToRule(apimokka_model::NodeId),
    /// MK-042: from trace detail "Jump to file" — select the file AND switch to Routes tab.
    JumpToFile(String),
    /// Navigate a durable validation row to its live rule set, rule, or condition owner.
    JumpToDiagnostic(apimokka_model::NodeId),
    /// Switch to Trace tab and select a specific event — rule-editor jump-link.
    JumpToTraceEvent(u64),
    /// Switch to Trace tab ("View all in Trace" link).
    ViewAllInTrace,

    // ── Test rule dialog ──────────────────────────────────────────────────
    TestRuleOpen,
    TestRuleClose,
    ReplayAsTestInput(u64),
    TestRuleSetMethod(String),
    TestRuleSetPath(String),
    TestRuleSetHeaders(String),
    TestRuleSetBody(String),
    TestRuleRun,

    // ── Dotted-path assistant ─────────────────────────────────────────────
    PathAssistantOpen(usize), // body row index
    PathAssistantClose,
    PathAssistantSetJson(String),
    PathAssistantSelectPath(String),
    PathAssistantInsert,

    // ── Confirm dialog ────────────────────────────────────────────────────
    ConfirmRequest(ConfirmAction),
    ConfirmProceed,
    ConfirmCancel,

    // ── Command palette ───────────────────────────────────────────────────
    PaletteQuery(String),

    // ── Settings ──────────────────────────────────────────────────────────
    SettingsSetHost(String),
    SettingsSetPort(String),
    SettingsSetTls(bool),
    SettingsSetLogLevel(String),
    SettingsSetStrategy(Strategy),
    SettingsSetTraceEnabled(bool),

    // ── Dashboard ─────────────────────────────────────────────────────────
    DashSearch(String),
    DashPinToggle(String),

    // ── Sidebar section toggles ───────────────────────────────────────────
    ToggleFallbackSection,
    ToggleMiddlewareSection,

    // ── MK-039: friendly feedback ─────────────────────────────────────────
    UndoLast,
    DismissNotice,
    DismissProblem,
    ProblemAction,

    // ── MK-045: undo / redo ───────────────────────────────────────────────
    Undo,
    Redo,

    // ── MK-040: audience modes ────────────────────────────────────────────
    /// First-run picker answered, or Settings changed the mode.
    ChooseAudienceMode(apimokka_model::AudienceMode),
    /// Toggle the technical-detail disclosure on the error banner.
    ToggleProblemDetails,

    // ── MK-041: layout density ────────────────────────────────────────────
    ToggleRuleWhenMore,
    ToggleSettingsAdvancedMore,

    // ── MK-043: strategy / weight / priority ──────────────────────────────
    RuleSetSetStrategy(apimokka_model::Strategy),
    RuleWeightChanged(String),
    RulePriorityChanged(String),
    ToggleRuleSetConfigMore,

    // ── Noop ─────────────────────────────────────────────────────────────
    Noop,

    // ── Fallback file editor (file-system routing, MK-038) ────────────────
    /// An edit interaction inside the multi-line JSON editor.
    FallbackEditorAction(iced::widget::text_editor::Action),
    /// Pretty-print the current draft.
    FallbackFileFormat,
    /// Commit the selected file's draft to its saved baseline.
    FallbackFileSave,
    /// Request revert of the selected file's draft (routes through confirm).
    FallbackFileRevert,
    /// Set the draft status code for the selected file.
    FallbackFileSetStatus(String),
}
