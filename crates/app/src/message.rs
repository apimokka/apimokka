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
    ToggleTheme,
    /// MK-050: pick a specific theme (Light / Dark / HC Light / HC Dark).
    SetTheme(crate::app::ThemeChoice),

    // ── Keyboard ──────────────────────────────────────────────────────────
    EscapePressed,
    ToggleCommandPalette,
    /// MK-033 §3 / task 014 §4: moves the palette's row selection, or the
    /// mode picker's, depending on which is showing. A no-op otherwise.
    ArrowUp,
    ArrowDown,
    /// MK-033 line 92 / task 014 §4: executes the palette's selected row, or
    /// confirms the mode picker's selected card. A no-op otherwise.
    EnterPressed,

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
