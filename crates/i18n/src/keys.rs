//! MK-036 — Canonical string key enum.
//!
//! Every user-visible string has a variant here. The exhaustive match in
//! `en.rs` and `ja.rs` guarantees no key is ever missing a translation.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // ── Shared / global ──────────────────────────────────────────────────
    AppName,
    Tagline,
    BtnClose,
    BtnCancel,
    BtnCreate,
    BtnSave,
    BtnDiscard,
    BtnAdd,
    BtnDelete,
    BtnDuplicate,
    BtnMoveUp,
    BtnMoveDown,
    BtnReplay,
    BtnRunTest,
    BtnCopyPath,
    BtnOpenRule,
    BtnInsertPath,
    BtnJumpToIssue,
    BtnSaveAll,
    BtnReload,
    BtnRestart,
    BtnStartServer,
    BtnStopServer,
    BtnOpenWorkspace,
    BtnCreateWorkspace,

    // ── Status chips ──────────────────────────────────────────────────────
    StatusRunning,
    StatusStopped,
    StatusStarting,
    StatusReloadPending,
    StatusRestartRequired,
    StatusReloadOnStart,
    StatusRestartOnStart,
    StatusError,
    StatusSaved,
    StatusUnsaved, // "Unsaved (N)" — caller appends count
    StatusSaving,
    StatusSaveError,
    StatusTracePaused,
    StatusTraceConnecting,
    StatusTraceError,

    // ── Left rail ─────────────────────────────────────────────────────────
    NavRoutes,
    NavTrace,
    NavScripts,
    NavSettings,

    // ── Top bar ───────────────────────────────────────────────────────────
    BtnCommandPalette,
    TraceStripToggle,

    // ── Welcome (S-00) ────────────────────────────────────────────────────
    WelcomeHeroTagline,
    WelcomeOpenWorkspace,
    WelcomeCreateWorkspace,
    WelcomeNoRecents,
    WelcomeHowTitle,
    WelcomeHowMiddleware,
    WelcomeHowRuleSets,
    WelcomeHowFallback,

    // ── Dashboard (S-01) ─────────────────────────────────────────────────
    DashTitle,
    DashSearchPlaceholder,
    DashPinnedSection,
    DashRecentSection,
    DashLastOpened,
    DashPinToggle,

    // ── Wizard (S-02) ─────────────────────────────────────────────────────
    WizardTitle,
    WizardFieldName,
    WizardFieldFolder,
    WizardSectionServer,
    WizardSectionServerHint,
    WizardSectionStarter,
    WizardSectionStarterHint,
    WizardSectionTrace,
    WizardSectionTraceHint,
    WizardFieldHost,
    WizardFieldPort,
    WizardFieldTls,
    WizardFieldTlsCert,
    WizardFieldTlsKey,
    WizardStarterTemplate,
    WizardStarterMinimal, // "Minimal (GET /health → 200)"
    WizardStarterShopApi, // "Shop API example (two rule sets, strategies, fallback files)"
    WizardStarterEmpty,   // "Empty workspace"
    WizardTraceEnable,
    WizardQueueSize,
    WizardValidationNameRequired,
    WizardValidationFolderInvalid,
    WizardValidationPortInvalid,
    WizardValidationFolderHasWorkspace,
    WizardOverwriteGuide,

    // ── Workspace shell (S-03) ────────────────────────────────────────────
    WorkspaceMenuCurrent,
    WorkspaceMenuOpen,
    WorkspaceMenuCreate,

    // ── Routes (S-05) ────────────────────────────────────────────────────
    RoutesRuleSets,
    RoutesFallbackFiles,
    RoutesMiddleware,
    BtnAddRuleSet,
    BtnAddRule,
    WhenLabel,
    RespondLabel,
    WhenArrow, // "→"

    // URL path card
    UrlPathCardTitle,
    UrlPathField,
    UrlPathOperator,
    UrlPathHint,

    // Method card
    MethodCardTitle,
    MethodAny,

    // Headers card
    HeadersCardTitle,
    HeaderColumnName,
    HeaderColumnOp,
    HeaderColumnValue,
    BtnAddHeader,

    // Body card
    BodyCardTitle,
    BodyColumnPath,
    BodyColumnOp,
    BodyColumnValue,
    BtnAddBodyCondition,
    BodyJsonpathWarn, // "Use dotted path, not JSONPath"
    BodyDottedPathHint,

    // Respond card
    RespondCardTitle,
    RespondModeInline,
    RespondModeFile,
    RespondStatusLabel,
    RespondDelayLabel,
    RespondDelayUnit, // "ms"
    RespondMutexHint,

    // Rule inspector
    InspectorTitle,
    InspectorValidationTitle,
    InspectorValidationOk,
    InspectorStrategyTitle,
    InspectorWeightLabel,
    InspectorPriorityLabel,
    InspectorActionsTitle,

    // Left sidebar states
    SidebarDirtyMarker,   // "●"
    SidebarMatchedMarker, // "Matched"

    // Empty states
    EmptyNoRuleSelected,
    EmptyNoRuleSelectedCta,
    EmptyRuleSetNoRules,
    EmptyBlankWorkspace, // "Add your first rule set to start mocking"

    // ── Trace (S-11 / S-12) ──────────────────────────────────────────────
    TraceTitle,
    TraceFilterMethod,
    TraceFilterOutcome,
    TraceFilterPath,
    TracePause,
    TraceResume,
    TraceClear,
    TraceEmptyMessage,
    TraceDroppedEvents, // "N events dropped (queue full)"
    TraceMatchedLabel,
    TraceFallbackLabel,
    TraceMissLabel,
    TraceErrorLabel,

    // Match detail
    DetailTitle,
    DetailRequest,
    DetailOutcome,
    DetailResponse,
    DetailMatchReasoning,
    DetailClosestRule,
    DetailConditionExpected,
    DetailConditionActual,
    DetailConditionResult,
    DetailConditionMatched,
    DetailConditionFailed,
    DetailFallbackExplanation,
    // MK-042: outcome-specific detail labels
    DetailMatchedRuleSet,  // "Rule set"
    DetailMatchedRule,     // "Rule"
    DetailJumpToRule,      // "Jump to rule"
    DetailFallbackFile,    // "Fallback file"
    DetailFallbackStatus,  // "Status"
    DetailJumpToFile,      // "Jump to file"
    DetailMissStatus,      // "Status"
    DetailMissExplanation, // "No rule matched this request."
    DetailMissCreateCta,   // "Create rule for this path"
    DetailErrorKind,       // "Error kind"
    DetailErrorMessage,    // "Message"
    DetailDroppedWarning,  // "N events dropped before this one (queue full)"
    DetailErrorExplanation,
    BtnReplayAsTestInput,
    BtnOpenMatchedRule,
    BtnCopyRequest,

    // ── Appearance section (moved from top bar)
    SettingsSectionAppearance,
    SettingsTheme,
    SettingsThemeLight,
    SettingsThemeDark,
    // MK-050: snora design theme presets
    ThemeLight,
    ThemeDark,
    ThemeHighContrastLight,
    ThemeHighContrastDark,
    SettingsKeyboardSection,
    SettingsPaletteShortcut,

    // ── Settings (S-13) ──────────────────────────────────────────────────
    SettingsTitle,
    SettingsSectionGeneral,
    SettingsSectionServer,
    SettingsSectionLogs,
    SettingsSectionTrace,
    SettingsSectionStrategy,
    SettingsImpactSaveOnly,
    SettingsImpactReload,
    SettingsImpactRestart,
    SettingsWorkspaceName,
    SettingsHost,
    SettingsPort,
    SettingsTls,
    SettingsTlsCert,
    SettingsTlsKey,
    SettingsLogFile,
    SettingsLogLevel,
    SettingsTraceEnable,
    SettingsTraceTransport,
    SettingsTraceQueueSize,
    SettingsStrategy,
    SettingsFooterClean,
    SettingsFooterUnsaved,
    SettingsFooterReload,
    SettingsFooterRestart,

    // ── Scripts (S-14) ───────────────────────────────────────────────────
    ScriptsTitle,
    ScriptsEmptyMessage,
    ScriptsEmptyExplanation,

    // ── Bottom drawer ─────────────────────────────────────────────────────
    DrawerValidationTitle,
    DrawerValidationErrors,   // "Errors N"
    DrawerValidationWarnings, // "Warnings N"
    DrawerValidationInfo,
    DrawerOpenDiagnostic,
    DrawerSaveDiffTitle,
    DrawerSaveDiffCount, // "N files will be written"
    DrawerSaveDiffModified,
    DrawerSaveDiffCreated,
    DrawerSaveDiffRemoved,
    DrawerViewDiff,
    DrawerLastSaveAttempt,
    DrawerCurrentUnsaved,
    SaveCompletionComplete,
    SaveCompletionPartial,
    SaveCompletionFailed,
    SaveCompletionIndeterminate,
    SaveIntegrityFailure,
    SaveVerifiedWritten,
    SaveReportedWritten,
    SaveReportedDiffs,
    SaveReportedFailure,
    SaveFailedFile,
    SaveFallbackWritten,
    SaveRemainingScopes,
    SaveAttemptPhases,
    SaveUnsavedPhase,
    SavePendingPhase,
    SaveNone,
    SavePhaseNone,
    SavePhaseReload,
    SavePhaseRestart,

    // ── Command palette ───────────────────────────────────────────────────
    PaletteTitle,
    PaletteSearch,
    PaletteNoMatch,
    PaletteCmdSave,
    PaletteCmdAddRule,
    PaletteCmdAddRuleSet,
    PaletteCmdTestRule,
    PaletteCmdToggleTrace,
    PaletteCmdOpenValidation,
    PaletteCmdOpenSaveDiff,
    PaletteCmdStartServer,
    PaletteCmdStopServer,
    PaletteCmdReload,
    PaletteCmdRestart,
    PaletteCmdSwitchWorkspace,
    PaletteCmdSettings,
    PaletteCmdToggleTheme,
    PaletteCmdLocale,
    PaletteCmdGoRoutes,
    PaletteCmdGoTrace,
    PaletteCmdGoScripts,
    PaletteCmdGoSettings,

    // ── Test rule dialog ──────────────────────────────────────────────────
    TestRuleTitle,
    TestRuleHint,
    TestRuleMethod,
    TestRulePath,
    TestRuleHeaders,
    TestRuleBody,
    TestRuleResultHint,
    TestRuleMatched,
    TestRuleNoMatch,
    TestRuleUnsupported,
    TestRuleError,
    TestRuleUnableVerify,
    TestRuleConditionPassed,
    TestRuleConditionFailed,
    TestRuleConditionUnsupported,
    TestRuleConditionError,
    TestRuleReasonUnsupportedMethod,
    TestRuleReasonUnsupportedOperator,
    TestRuleReasonNoSelection,
    TestRuleReasonInvalidMethod,
    TestRuleReasonInvalidHeader,
    TestRuleReasonDuplicateHeader,
    TestRuleReasonInvalidBody,
    TestRuleReasonInvalidConfig,
    TestRuleScopeSelection,
    TestRuleScopeRequestMethod,
    TestRuleScopeHeaderLine,
    TestRuleScopeRequestBody,

    // ── Dotted-path assistant ─────────────────────────────────────────────
    DottedPathTitle,
    DottedPathPasteLabel,
    DottedPathTreeLabel,
    DottedPathSelectedLabel,
    DottedPathJsonError,
    DottedPathEmpty,
    DottedPathJsonpathHint, // inline warning for $.foo syntax
    BtnUse,

    // ── Confirm dialog ────────────────────────────────────────────────────
    ConfirmProceed,

    ConfirmDeleteRule,
    ConfirmDeleteRuleBody,
    ConfirmDeleteRuleSet,
    ConfirmDeleteRuleSetBody,
    ConfirmDiscardChanges,
    ConfirmDiscardChangesBody,
    ConfirmSwitchWorkspace,
    ConfirmSwitchWorkspaceBody,
    ConfirmOverwriteWorkspace,
    ConfirmOverwriteWorkspaceBody,

    // ── Fallback file editor (file-system routing) ───────────────────────
    FallbackEditorHeading,
    FallbackServesLabel,
    FallbackRouteExplanation,
    FallbackContentLabel,
    FallbackStatusLabel,
    FallbackFormatJson,
    BtnAddFallbackFile,
    FallbackEmptyHint,
    BtnRevert,
    FallbackJsonValid,
    FallbackJsonInvalid,
    FallbackSavedHint,
    FallbackUnsavedHint,
    ConfirmRevertFile,
    ConfirmRevertFileBody,

    // ── MK-039: hints, undo, friendly errors ─────────────────────────────
    HintBodyPath,
    HintUrlOp,
    HintStrategy,
    // MK-043: strategy UI
    RuleSetConfigStrategy,     // "Strategy" section heading in rule set config
    RuleSetConfigMoreOptions,  // "More rule-set options" toggle (Guided)
    RuleSetConfigFewerOptions, // "Fewer rule-set options" toggle (Guided)
    RuleEditorValidationWarning, // "Rule has validation issues:" strip
    // MK-044: bottom drawer
    DrawerValidationOk,         // "✓ No validation issues"
    DrawerValidationWorkspace,  // "Workspace"
    DrawerSaveDiffChangedRules, // "rules ·" (before summary list)
    DrawerSaveDiffFallbackMod,  // "JSON content modified" // "Rule has validation issues:" strip
    RuleWeightLabel,            // "Weight" — shown when WeightedRandom
    RuleWeightHint,             // help text for weight field
    RulePriorityLabel,          // "Priority" — shown when Priority
    RulePriorityHint,           // help text for priority field
    HintHeaderOp,
    UndoLabel,
    RedoLabel,
    UndoRuleDeleted,   // "Deleted rule"
    UndoRuleAdded,     // "Added rule"
    UndoRuleMoved,     // "Moved rule"
    UndoUrlPathEdited, // "URL path changed"
    UndoRedoAvailable, // "Redo available"
    PaletteCmdUndo,    // "Undo"
    PaletteCmdRedo,    // "Redo"
    NoticeRuleDeleted,
    DisabledNeedUrlPath,
    DisabledNeedContent,
    ErrorActionRetry,
    ErrorActionOpenSettings,

    // ── MK-041: layout density toggles ──────────────────────────────────
    LayoutMoreWhen,
    LayoutFewerWhen,
    LayoutMoreSettings,
    LayoutFewerSettings,
    LayoutActiveHeader, // "{n} header" (pluralised in code)
    LayoutActiveBody,   // "{n} body" (pluralised in code)

    // ── MK-040: audience modes ───────────────────────────────────────────
    ModePickerTitle,
    ModeGuidedTitle,
    ModeGuidedDesc,
    ModeExpertTitle,
    ModeExpertDesc,
    ModePickerHint,
    ModePickerContinue,
    SettingsAudienceMode,
    ErrorShowDetails,
    ErrorHideDetails,

    // ── Locale ────────────────────────────────────────────────────────────
    LocaleEn,
    LocaleJa,
}
