use super::Key;

pub fn t(key: Key) -> &'static str {
    match key {
        Key::AppName => "apimokka",
        Key::Tagline => "Visual HTTP mock authoring",

        Key::BtnClose => "Close",
        Key::BtnCancel => "Cancel",
        Key::BtnCreate => "Create workspace",
        Key::BtnSave => "Save",
        Key::BtnDiscard => "Discard",
        Key::BtnAdd => "Add",
        Key::BtnDelete => "Delete",
        Key::BtnDuplicate => "Duplicate",
        Key::BtnMoveUp => "Move up",
        Key::BtnMoveDown => "Move down",
        Key::BtnReplay => "Replay",
        Key::BtnRunTest => "Run test",
        Key::BtnCopyPath => "Copy path",
        Key::BtnOpenRule => "Open this rule",
        Key::BtnInsertPath => "Insert path",
        Key::BtnJumpToIssue => "Jump to first issue",
        Key::BtnSaveAll => "Save all",
        Key::BtnReload => "Reload",
        Key::BtnRestart => "Restart",
        Key::BtnStartServer => "Start server",
        Key::BtnStopServer => "Stop server",
        Key::BtnOpenWorkspace => "Open workspace",
        Key::BtnCreateWorkspace => "Create new workspace",

        Key::StatusRunning => "Running",
        Key::StatusStopped => "Stopped",
        Key::StatusStarting => "Starting",
        Key::StatusReloadPending => "Reload pending",
        Key::StatusRestartRequired => "Restart required",
        Key::StatusReloadOnStart => "Reload applies on start",
        Key::StatusRestartOnStart => "Restart applies on start",
        Key::StatusError => "Error",
        Key::StatusSaved => "Saved",
        Key::StatusUnsaved => "Unsaved",
        Key::StatusSaving => "Saving…",
        Key::StatusSaveError => "Save error",
        Key::StatusTracePaused => "Trace paused",
        Key::StatusTraceConnecting => "Trace connecting",
        Key::StatusTraceError => "Trace error",

        Key::NavRoutes => "Routes",
        Key::NavTrace => "Trace",
        Key::NavScripts => "Scripts",
        Key::NavSettings => "Settings",

        Key::BtnCommandPalette => "Command palette",
        Key::TraceStripToggle => "Toggle live trace",

        Key::WelcomeHeroTagline => "Visual HTTP mock authoring",
        Key::WelcomeOpenWorkspace => "Open workspace",
        Key::WelcomeCreateWorkspace => "Create new workspace",
        Key::WelcomeNoRecents => "Create a workspace to start authoring mock endpoints.",
        Key::WelcomeHowTitle => "How requests are handled",
        Key::WelcomeHowMiddleware => "Middleware scripts",
        Key::WelcomeHowRuleSets => "Rule sets",
        Key::WelcomeHowFallback => "Fallback files",

        Key::DashTitle => "Workspaces",
        Key::DashSearchPlaceholder => "Search workspaces…",
        Key::DashPinnedSection => "Pinned",
        Key::DashRecentSection => "Recent",
        Key::DashLastOpened => "Last opened",
        Key::DashPinToggle => "Pin workspace",

        Key::WizardTitle => "Create workspace",
        Key::WizardFieldName => "Workspace name",
        Key::WizardFieldFolder => "Parent folder",
        Key::WizardSectionServer => "Server defaults",
        Key::WizardSectionServerHint => "Default: 127.0.0.1 : 8080, no TLS",
        Key::WizardSectionStarter => "Starter content",
        Key::WizardSectionStarterHint => "Default: basic REST + error samples",
        Key::WizardSectionTrace => "Trace",
        Key::WizardSectionTraceHint => "Default: enabled, 1024-event queue",
        Key::WizardFieldHost => "Host",
        Key::WizardFieldPort => "Port",
        Key::WizardFieldTls => "Enable TLS",
        Key::WizardFieldTlsCert => "TLS certificate path",
        Key::WizardFieldTlsKey => "TLS key path",
        Key::WizardStarterTemplate => "Starter template",
        Key::WizardStarterMinimal => "Minimal — GET /health → 200  (recommended starting point)",
        Key::WizardStarterShopApi => {
            "Shop API example — explores all features (two rule sets, strategies, fallback files)"
        }
        Key::WizardStarterEmpty => "Empty workspace — no rules",
        Key::WizardTraceEnable => "Enable trace",
        Key::WizardQueueSize => "Queue size",
        Key::WizardValidationNameRequired => "Workspace name cannot be empty.",
        Key::WizardValidationFolderInvalid => "Parent folder path is invalid.",
        Key::WizardValidationPortInvalid => "Port must be a number between 1 and 65535.",
        Key::WizardValidationFolderHasWorkspace => "This folder already contains a workspace.",
        Key::WizardOverwriteGuide => "Choose another folder or confirm overwrite.",

        Key::WorkspaceMenuCurrent => "Current workspace",
        Key::WorkspaceMenuOpen => "Open workspace…",
        Key::WorkspaceMenuCreate => "Create new workspace…",

        Key::RoutesRuleSets => "Rule sets",
        Key::RoutesFallbackFiles => "Fallback files",
        Key::RoutesMiddleware => "Middleware scripts",
        Key::BtnAddRuleSet => "Add rule set",
        Key::BtnAddRule => "Add rule",
        Key::WhenLabel => "WHEN",
        Key::RespondLabel => "RESPOND",
        Key::WhenArrow => "→",

        Key::UrlPathCardTitle => "URL path",
        Key::UrlPathField => "/api/orders",
        Key::UrlPathOperator => "Operator",
        Key::UrlPathHint => "Use /api/orders, not the full URL.",

        Key::MethodCardTitle => "Method",
        Key::MethodAny => "Any",

        Key::HeadersCardTitle => "Headers",
        Key::HeaderColumnName => "Name",
        Key::HeaderColumnOp => "Operator",
        Key::HeaderColumnValue => "Value",
        Key::BtnAddHeader => "Add header",

        Key::BodyCardTitle => "Body conditions",
        Key::BodyColumnPath => "Path",
        Key::BodyColumnOp => "Operator",
        Key::BodyColumnValue => "Value",
        Key::BtnAddBodyCondition => "Add body condition",
        Key::BodyJsonpathWarn => "Use dotted path syntax, e.g. user.id. JSONPath is not supported.",
        Key::BodyDottedPathHint => "Dotted path such as user.id or items.0.name.",

        Key::RespondCardTitle => "Response",
        Key::RespondModeInline => "Inline text",
        Key::RespondModeFile => "Serve file",
        Key::RespondStatusLabel => "Status",
        Key::RespondDelayLabel => "Delay",
        Key::RespondDelayUnit => "ms",
        Key::RespondMutexHint => "Inline text and a served file cannot both be set.",

        Key::InspectorTitle => "Rule inspector",
        Key::InspectorValidationTitle => "Validation",
        Key::InspectorValidationOk => "No issues",
        Key::InspectorStrategyTitle => "Strategy",
        Key::InspectorWeightLabel => "Weight",
        Key::InspectorPriorityLabel => "Priority",
        Key::InspectorActionsTitle => "Quick actions",

        Key::SidebarDirtyMarker => "●",
        Key::SidebarMatchedMarker => "Matched",

        Key::EmptyNoRuleSelected => "Choose a rule from the left, or create a new endpoint.",
        Key::EmptyNoRuleSelectedCta => "Add rule",
        Key::EmptyBlankWorkspace => "Add a rule set to start mocking requests.",
        Key::EmptyRuleSetNoRules => "This rule set has no rules yet.",

        Key::TraceTitle => "Trace",
        Key::TraceFilterMethod => "Method",
        Key::TraceFilterOutcome => "Outcome",
        Key::TraceFilterPath => "Path contains…",
        Key::TracePause => "Pause",
        Key::TraceResume => "Resume",
        Key::TraceClear => "Clear",
        Key::TraceEmptyMessage => "No requests observed yet. Trigger your app or curl the server.",
        Key::TraceDroppedEvents => "events dropped (queue full)",
        Key::TraceMatchedLabel => "Matched",
        Key::TraceFallbackLabel => "Fallback",
        Key::TraceMissLabel => "Miss",
        Key::TraceErrorLabel => "Error",

        Key::DetailTitle => "Match detail",
        Key::DetailRequest => "Request",
        Key::DetailOutcome => "Outcome",
        Key::DetailResponse => "Response",
        Key::DetailMatchReasoning => "Match reasoning",
        Key::DetailClosestRule => "Closest rule",
        Key::DetailConditionExpected => "Expected",
        Key::DetailConditionActual => "Actual",
        Key::DetailConditionResult => "Result",
        Key::DetailConditionMatched => "Matched",
        Key::DetailConditionFailed => "Failed",
        Key::DetailFallbackExplanation => "Served by fallback file (no matching rule).",
        // MK-042
        Key::DetailMatchedRuleSet => "Rule set",
        Key::DetailMatchedRule => "Rule",
        Key::DetailJumpToRule => "Jump to rule",
        Key::DetailFallbackFile => "Fallback file",
        Key::DetailFallbackStatus => "Status",
        Key::DetailJumpToFile => "Jump to file",
        Key::DetailMissStatus => "Status",
        Key::DetailMissExplanation => "No rule matched this request.",
        Key::DetailMissCreateCta => "Create rule for this path",
        Key::DetailErrorKind => "Error kind",
        Key::DetailErrorMessage => "Message",
        Key::DetailDroppedWarning => "events dropped before this one (queue full)",
        // end MK-042 => "No rule matched. This path was served by the fallback file.",
        Key::DetailErrorExplanation => {
            "A middleware script threw an error before rule matching ran."
        }
        Key::BtnReplayAsTestInput => "Replay as test input",
        Key::BtnOpenMatchedRule => "Open matched rule",
        Key::BtnCopyRequest => "Copy request",

        Key::SettingsTitle => "Settings",
        Key::SettingsSectionGeneral => "General",
        Key::SettingsSectionServer => "Server",
        Key::SettingsSectionLogs => "Logs",
        Key::SettingsSectionTrace => "Trace",
        Key::SettingsSectionStrategy => "Strategy",
        Key::SettingsImpactSaveOnly => "Save only",
        Key::SettingsImpactReload => "Reload required after saving",
        Key::SettingsImpactRestart => "Restart required after saving",
        Key::SettingsWorkspaceName => "Workspace name",
        Key::SettingsHost => "Host",
        Key::SettingsPort => "Port",
        Key::SettingsTls => "Enable TLS",
        Key::SettingsTlsCert => "TLS certificate path",
        Key::SettingsTlsKey => "TLS key path",
        Key::SettingsLogFile => "Log file path",
        Key::SettingsLogLevel => "Log level",
        Key::SettingsTraceEnable => "Enable trace",
        Key::SettingsTraceTransport => "Transport",
        Key::SettingsTraceQueueSize => "Queue size",
        Key::SettingsStrategy => "Rule-selection strategy",
        Key::SettingsFooterClean => "All changes saved.",
        Key::SettingsFooterUnsaved => "Unsaved changes — save to apply.",
        Key::SettingsFooterReload => "Reload required to take effect.",
        Key::SettingsFooterRestart => "Restart required to take effect.",

        Key::ScriptsTitle => "Scripts",
        Key::ScriptsEmptyMessage => "No middleware scripts in this workspace.",
        Key::ScriptsEmptyExplanation => {
            "Middleware scripts run before rule matching and can transform requests."
        }

        Key::DrawerValidationTitle => "Validation",
        Key::DrawerValidationErrors => "Errors",
        Key::DrawerValidationWarnings => "Warnings",
        Key::DrawerValidationInfo => "Info",
        Key::DrawerOpenDiagnostic => "Open",
        Key::DrawerSaveDiffTitle => "Save diff",
        Key::DrawerSaveDiffCount => "files will be written",
        Key::DrawerSaveDiffModified => "Modified",
        Key::DrawerSaveDiffCreated => "Created",
        Key::DrawerSaveDiffRemoved => "Removed",
        Key::DrawerViewDiff => "View diff",

        Key::PaletteTitle => "Command palette",
        Key::PaletteSearch => "Search commands…",
        Key::PaletteNoMatch => "No matching commands",
        Key::PaletteCmdSave => "Save workspace",
        Key::PaletteCmdAddRule => "Add rule",
        Key::PaletteCmdAddRuleSet => "Add rule set",
        Key::PaletteCmdTestRule => "Test current rule",
        Key::PaletteCmdToggleTrace => "Toggle live trace strip",
        Key::PaletteCmdOpenValidation => "Open validation drawer",
        Key::PaletteCmdOpenSaveDiff => "Open save diff",
        Key::PaletteCmdStartServer => "Start server",
        Key::PaletteCmdStopServer => "Stop server",
        Key::PaletteCmdReload => "Reload config",
        Key::PaletteCmdRestart => "Restart server",
        Key::PaletteCmdSwitchWorkspace => "Switch workspace",
        Key::PaletteCmdSettings => "Open settings",
        Key::PaletteCmdToggleTheme => "Toggle theme",
        Key::PaletteCmdLocale => "Change locale",
        Key::PaletteCmdGoRoutes => "Go to Routes",
        Key::PaletteCmdGoTrace => "Go to Trace",
        Key::PaletteCmdGoScripts => "Go to Scripts",
        Key::PaletteCmdGoSettings => "Go to Settings",

        Key::TestRuleTitle => "Test rule",
        Key::TestRuleHint => "Dry-run match against the selected rule. No network traffic.",
        Key::TestRuleMethod => "Method",
        Key::TestRulePath => "Path",
        Key::TestRuleHeaders => "Headers (name: value, one per line)",
        Key::TestRuleBody => "Body (JSON)",
        Key::TestRuleResultHint => "Run the test to see condition-by-condition result.",
        Key::TestRuleMatched => "✓ Matched",
        Key::TestRuleNoMatch => "◯ No match",
        Key::TestRuleUnsupported => "? Unable to verify",
        Key::TestRuleError => "! Error",
        Key::TestRuleUnableVerify => {
            "This rule uses conditions unavailable in the adopted matcher. Run for details."
        }
        Key::TestRuleConditionPassed => "passed",
        Key::TestRuleConditionFailed => "failed",
        Key::TestRuleConditionUnsupported => "unsupported",
        Key::TestRuleConditionError => "error",
        Key::TestRuleReasonUnsupportedMethod => "configured method is unsupported",
        Key::TestRuleReasonUnsupportedOperator => "operator is unsupported",
        Key::TestRuleReasonNoSelection => "no rule is selected",
        Key::TestRuleReasonInvalidMethod => "method is invalid",
        Key::TestRuleReasonInvalidHeader => "header input is invalid",
        Key::TestRuleReasonDuplicateHeader => "duplicate header",
        Key::TestRuleReasonInvalidBody => "body is not valid JSON",
        Key::TestRuleReasonInvalidConfig => "configured value is invalid",
        Key::TestRuleScopeSelection => "Selection",
        Key::TestRuleScopeRequestMethod => "Request method",
        Key::TestRuleScopeHeaderLine => "Header line",
        Key::TestRuleScopeRequestBody => "Request body",

        Key::DottedPathTitle => "Dotted-path assistant",
        Key::DottedPathPasteLabel => "Paste sample JSON",
        Key::DottedPathTreeLabel => "JSON tree",
        Key::DottedPathSelectedLabel => "Selected path:",
        Key::DottedPathJsonError => "Invalid JSON — check the input.",
        Key::DottedPathEmpty => "Paste sample JSON to build a path.",
        Key::DottedPathJsonpathHint => {
            "Use dotted path syntax, e.g. user.id. JSONPath is not supported."
        }
        Key::BtnUse => "Use",

        Key::ConfirmProceed => "Proceed",

        Key::ConfirmDeleteRule => "Delete rule?",
        Key::ConfirmDeleteRuleBody => "This removes the selected rule.",
        Key::ConfirmDeleteRuleSet => "Delete rule set?",
        Key::ConfirmDeleteRuleSetBody => "This removes the file and its rules.",
        Key::ConfirmDiscardChanges => "Discard unsaved changes?",
        Key::ConfirmDiscardChangesBody => "Edits will be lost.",
        Key::ConfirmSwitchWorkspace => "Switch workspaces?",
        Key::ConfirmSwitchWorkspaceBody => "Unsaved edits will be lost.",
        Key::ConfirmOverwriteWorkspace => "Overwrite existing workspace?",
        Key::ConfirmOverwriteWorkspaceBody => "Existing files in the folder will be replaced.",

        Key::SettingsSectionAppearance => "Appearance",
        Key::SettingsTheme => "Theme",
        Key::SettingsThemeLight => "Light",
        Key::ThemeLight => "Light",
        Key::ThemeDark => "Dark",
        Key::ThemeHighContrastLight => "High Contrast Light",
        Key::ThemeHighContrastDark => "High Contrast Dark",
        Key::SettingsThemeDark => "Dark",
        Key::SettingsKeyboardSection => "Keyboard shortcuts",
        Key::SettingsPaletteShortcut => "Command palette  \u{2318}K · Ctrl+K",

        Key::HintBodyPath => {
            "Matches the JSON request body by dotted path (user.id, items.0.sku).              Not JSONPath \u{2014} $.foo will not work."
        }
        Key::HintUrlOp => {
            "How the incoming URL path is compared: Equal, StartsWith, Contains,              EndsWith, WildCard, NotEqual."
        }
        Key::RuleSetConfigStrategy => "Strategy",
        Key::RuleSetConfigMoreOptions => "More rule-set options",
        Key::RuleSetConfigFewerOptions => "Fewer rule-set options",
        Key::DrawerValidationOk => "\u{2713} No validation issues",
        Key::DrawerValidationWorkspace => "Workspace",
        Key::DrawerSaveDiffChangedRules => "rules \u{00B7}",
        Key::DrawerSaveDiffFallbackMod => "JSON content modified",

        Key::RuleEditorValidationWarning => "Validation warning",
        Key::RuleWeightLabel => "Weight",
        Key::RuleWeightHint => {
            "Relative likelihood this rule is chosen when WeightedRandom is active. Default: 1."
        }
        Key::RulePriorityLabel => "Priority",
        Key::RulePriorityHint => {
            "Highest priority wins when Priority strategy is active. Negative values allowed."
        }

        Key::HintStrategy => {
            "How a winner is chosen when several rules match the same request              (FirstMatch, WeightedRandom, Priority, RoundRobin)."
        }
        Key::HintHeaderOp => {
            "How the header value is compared. Exists and Absent ignore the value field."
        }
        Key::UndoLabel => "Undo",
        Key::RedoLabel => "Redo",
        Key::UndoRuleDeleted => "Deleted rule",
        Key::UndoRuleAdded => "Added rule",
        Key::UndoRuleMoved => "Moved rule",
        Key::UndoUrlPathEdited => "URL path changed",
        Key::UndoRedoAvailable => "Redo available",
        Key::PaletteCmdUndo => "Undo",
        Key::PaletteCmdRedo => "Redo",
        Key::NoticeRuleDeleted => "Rule deleted",
        Key::DisabledNeedUrlPath => "Enter a URL path first",
        Key::DisabledNeedContent => "Add response content first",
        Key::ErrorActionRetry => "Retry",
        Key::ErrorActionOpenSettings => "Open Settings",

        Key::LayoutMoreWhen => "More matching criteria",
        Key::LayoutFewerWhen => "Fewer matching criteria",
        Key::LayoutMoreSettings => "More settings",
        Key::LayoutFewerSettings => "Fewer settings",
        Key::LayoutActiveHeader => "header",
        Key::LayoutActiveBody => "body",

        Key::ModePickerTitle => "How would you like apimokka to guide you?",
        Key::ModeGuidedTitle => "Guided",
        Key::ModeGuidedDesc => {
            "Show extra explanations as you work. Best if HTTP mocking is newer to you."
        }
        Key::ModeExpertTitle => "Expert",
        Key::ModeExpertDesc => {
            "Compact view, no extra explanations. Best if you already know your way around."
        }
        Key::ModePickerHint => "You can change this any time in Settings.",
        Key::ModePickerContinue => "Continue",
        Key::SettingsAudienceMode => "Guidance",
        Key::ErrorShowDetails => "Show details",
        Key::ErrorHideDetails => "Hide details",

        Key::LocaleEn => "EN",
        Key::LocaleJa => "JA",

        Key::FallbackEditorHeading => "File-based route",
        Key::FallbackServesLabel => "Serves:",
        Key::FallbackRouteExplanation => {
            "This file is returned when no rule matches a request to this path. \
             Edit the JSON below to change the response body."
        }
        Key::FallbackContentLabel => "Response body",
        Key::FallbackStatusLabel => "Status code",
        Key::FallbackFormatJson => "Format JSON",
        Key::BtnAddFallbackFile => "Add file",
        Key::FallbackEmptyHint => "Select a file from the sidebar to edit its content.",
        Key::BtnRevert => "Revert",
        Key::FallbackJsonValid => "✓ Valid JSON",
        Key::FallbackJsonInvalid => "⚠ Invalid JSON — will be served as-is",
        Key::FallbackSavedHint => "Saved — changes take effect on the next request.",
        Key::FallbackUnsavedHint => "Unsaved changes.",
        Key::ConfirmRevertFile => "Revert file?",
        Key::ConfirmRevertFileBody => "Unsaved edits to this file will be discarded.",
    }
}
