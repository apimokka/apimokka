//! Central app state and update (MK-021, MK-035).
//!
//! Boundary decision: split — beyond the draft-editing domains already
//! extracted to `app/drafts.rs`, the residual `impl App` block still holds
//! several distinguishable groups that mirror the shape of domains already
//! pulled out elsewhere: undo/redo history and its draft-resynchronization,
//! archiving, and the save/workspace-open-close-create lifecycle.
//! `update()` itself (~968 lines of `Message` dispatch) is a single
//! function and not decomposable the same way. This extraction is beyond
//! RFC MK-057's mandated scope (which named only the draft-editing
//! extraction for this file) and is recorded here as a follow-up rather
//! than executed in this task.

use apimokka_i18n::{Key, Locale};
use apimokka_model::workspace_port::{map_response, map_rule_match, parse_rule_set_path};
use apimokka_model::{
    CollectionEdit, ConditionEdit, EditIntent, EditOutcome, EditTransaction, NodeId, ResponseMode,
    RuleEditPayload, RuleSetId, WorkspaceEditValue, WorkspaceNodeKind, WorkspaceRootKey, mock,
};
use iced::{Element, Subscription, Theme};

use crate::match_test::{TestRequest, TestRuleResult};
use crate::message::{ConfirmAction, Message};
use crate::screens;
use crate::selection::{DrawerMode, RouteSelection, RouteTarget, WorkspaceTab};
use crate::shell;
use crate::shell::top_bar::ServerState;

mod workspace_session;
pub use workspace_session::{ConditionFamily, DraftBinding, WorkspaceSession};
mod global_save;
pub use global_save::{
    FallbackSaveError, FallbackSaveFailure, FallbackSaveReport, FallbackSkipReason,
    GlobalSaveCompletion, GlobalSaveReport, ProgressTrust, SaveIntegrity, WorkspaceSaveProgress,
    WorkspaceSaveReport,
};
mod runtime;
use runtime::SessionGeneration;
pub use runtime::{RuntimeAction, RuntimeInFlight, RuntimeRequestToken};
mod drafts;

// ── Theme choice ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeChoice {
    #[default]
    Light,
    Dark,
    HighContrastLight,
    HighContrastDark,
}

impl ThemeChoice {
    /// MK-050: cycle Light → Dark → HC Light → HC Dark → Light.
    pub fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::HighContrastLight,
            Self::HighContrastLight => Self::HighContrastDark,
            Self::HighContrastDark => Self::Light,
        }
    }

    /// The snora Design token preset for this choice (MK-050).
    pub fn tokens(self) -> snora::design::Tokens {
        match self {
            Self::Light => snora::design::Tokens::light(),
            Self::Dark => snora::design::Tokens::dark(),
            Self::HighContrastLight => snora::design::Tokens::high_contrast_light(),
            Self::HighContrastDark => snora::design::Tokens::high_contrast_dark(),
        }
    }

    /// The iced Theme for this choice. Derived from the same snora Design
    /// tokens as `tokens()` via `snora::design::theme`, so stock iced
    /// widgets (`text_input`, `pick_list`, `scrollable`, the window
    /// background) follow the same palette as snora's own primitives in
    /// all four presets (RFC MK-058 phase 2) — previously only the two
    /// high-contrast presets were token-derived; Light and Dark drew stock
    /// widgets from iced's own built-in palette, a second, inconsistent
    /// source.
    pub fn iced(self) -> Theme {
        snora::design::theme(&self.tokens())
    }

    /// Whether this is a dark-family theme (for glyph/contrast decisions).
    pub fn is_dark(self) -> bool {
        matches!(self, Self::Dark | Self::HighContrastDark)
    }

    #[allow(dead_code)]
    pub fn glyph(self) -> &'static str {
        if self.is_dark() { "☀" } else { "☾" }
    }

    /// i18n label key for the theme picker (MK-050).
    pub fn label_key(self) -> apimokka_i18n::Key {
        use apimokka_i18n::Key;
        match self {
            Self::Light => Key::ThemeLight,
            Self::Dark => Key::ThemeDark,
            Self::HighContrastLight => Key::ThemeHighContrastLight,
            Self::HighContrastDark => Key::ThemeHighContrastDark,
        }
    }

    pub fn all() -> [ThemeChoice; 4] {
        [
            Self::Light,
            Self::Dark,
            Self::HighContrastLight,
            Self::HighContrastDark,
        ]
    }
}

// ── Outer view ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppView {
    #[default]
    Welcome,
    Dashboard,
    Wizard,
    Workspace,
}

// ── Wizard state ──────────────────────────────────────────────────────────────

// ── MK-048: Wizard starter options ───────────────────────────────────────────

/// Which starter content to generate when the wizard creates a workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WizardStarter {
    /// No rules, no rule sets — start completely empty.
    Empty,
    /// One rule set with a single `GET /health → 200 OK` rule.
    #[default]
    Minimal,
    /// The full shop-API mock (two rule sets, weighted/priority strategies,
    /// fallback files, middleware — useful for exploring all features).
    ShopApi,
}

// ── Wizard ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WizardState {
    pub name: String,
    pub folder: String,
    pub host: String,
    pub port: String,
    pub tls: bool,
    pub starter: WizardStarter,
    pub queue_size: String,
    pub section_open: [bool; 3],
}

impl Default for WizardState {
    fn default() -> Self {
        Self {
            name: String::new(),
            folder: String::new(),
            host: "127.0.0.1".into(),
            port: "8080".into(),
            tls: false,
            starter: WizardStarter::Minimal,
            queue_size: "1024".into(),
            section_open: [false; 3],
        }
    }
}

// ── Overlay states ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct CommandPaletteState {
    pub open: bool,
    pub query: String,
    /// Task 014 §3 (MK-033 line 120): keyboard selection, indexed into the
    /// *filtered* list `palette_commands::filtered_indices` produces for the
    /// current `query` — not the full table. `None` = nothing selected.
    pub selected: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ConfirmDialogState {
    pub action: ConfirmAction,
    pub title: Key,
    pub body: Key,
}

#[derive(Debug, Clone, Default)]
pub struct TestRuleState {
    pub open: bool,
    pub method: String,
    pub url_path: String,
    pub headers_text: String,
    pub body: String,
    pub result: Option<TestRuleResult>,
}

#[derive(Debug, Clone, Default)]
pub struct PathAssistantState {
    pub open: bool,
    pub target_index: usize,
    pub json_input: String,
    pub selected_path: String,
}

// ── MK-039: undo entry for reversible actions ─────────────────────────────────

// ── MK-045: typed undo/redo command log ───────────────────────────────────────

pub const UNDO_STACK_DEPTH: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleMatchDraftField {
    UrlPath,
    UrlPathOp,
    Method,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RespondDraftField {
    Mode,
    Text,
    FilePath,
    Status,
    Delay,
}

/// A reversible operation. Pushed to undo_stack before each edit; applied
/// in reverse on undo; pushed to redo_stack so redo re-applies it.
#[derive(Debug, Clone)]
pub enum HistoryEntry {
    /// A rule was moved between exact presentation indices.
    MoveRule {
        rule_id: apimokka_model::NodeId,
        before_index: usize,
        after_index: usize,
    },
    RuleMatch {
        rule_id: NodeId,
        field: RuleMatchDraftField,
        before: apimokka_model::RuleMatch,
        after: apimokka_model::RuleMatch,
    },
    Respond {
        rule_id: NodeId,
        field: RespondDraftField,
        before: apimokka_model::RespondDefinition,
        after: apimokka_model::RespondDefinition,
    },
    RootSetting {
        before: apimokka_model::RootSettingEdit,
        after: apimokka_model::RootSettingEdit,
    },
    RulePrototype {
        rule_id: NodeId,
        before: workspace_session::RulePrototype,
        after: workspace_session::RulePrototype,
    },
    TracePrototype {
        before: workspace_session::TracePrototypeSettings,
        after: workspace_session::TracePrototypeSettings,
    },
    HeaderAdd {
        rule_id: NodeId,
        key: apimokka_model::SemanticCreationKey,
        condition: apimokka_model::HeaderCondition,
        current_id: NodeId,
    },
    HeaderUpdate {
        current_id: NodeId,
        before: apimokka_model::HeaderCondition,
        after: apimokka_model::HeaderCondition,
    },
    HeaderRemove {
        rule_id: NodeId,
        index: usize,
        key: apimokka_model::SemanticCreationKey,
        condition: apimokka_model::HeaderCondition,
        current_id: NodeId,
    },
    HeadersClear {
        rule_id: NodeId,
        entries: Vec<(
            usize,
            apimokka_model::SemanticCreationKey,
            apimokka_model::HeaderCondition,
            NodeId,
        )>,
    },
    BodyAdd {
        rule_id: NodeId,
        key: apimokka_model::SemanticCreationKey,
        condition: apimokka_model::BodyCondition,
        current_id: NodeId,
    },
    BodyUpdate {
        current_id: NodeId,
        before: apimokka_model::BodyCondition,
        after: apimokka_model::BodyCondition,
    },
    BodyRemove {
        rule_id: NodeId,
        index: usize,
        key: apimokka_model::SemanticCreationKey,
        condition: apimokka_model::BodyCondition,
        current_id: NodeId,
    },
    BodiesClear {
        rule_id: NodeId,
        entries: Vec<(
            usize,
            apimokka_model::SemanticCreationKey,
            apimokka_model::BodyCondition,
            NodeId,
        )>,
    },
    AddedSubtree {
        archive: apimokka_model::ArchivedSubtree,
        current_root: NodeId,
        bindings: Vec<(NodeId, NodeId)>,
        prototypes: Vec<(NodeId, workspace_session::RulePrototype)>,
    },
    RemovedSubtree {
        archive: apimokka_model::ArchivedSubtree,
        current_root: NodeId,
        bindings: Vec<(NodeId, NodeId)>,
        prototypes: Vec<(NodeId, workspace_session::RulePrototype)>,
    },
}

impl HistoryEntry {
    pub fn banner_key(&self) -> apimokka_i18n::Key {
        match self {
            Self::MoveRule { .. } => apimokka_i18n::Key::UndoRuleMoved,
            Self::RuleMatch { .. } => apimokka_i18n::Key::UndoUrlPathEdited,
            Self::Respond { .. } => apimokka_i18n::Key::UndoUrlPathEdited,
            Self::RootSetting { .. } | Self::RulePrototype { .. } | Self::TracePrototype { .. } => {
                apimokka_i18n::Key::UndoRuleMoved
            }
            Self::HeaderAdd { .. }
            | Self::HeaderUpdate { .. }
            | Self::HeaderRemove { .. }
            | Self::HeadersClear { .. }
            | Self::BodyAdd { .. }
            | Self::BodyUpdate { .. }
            | Self::BodyRemove { .. }
            | Self::BodiesClear { .. } => apimokka_i18n::Key::UndoRuleMoved,
            Self::AddedSubtree { .. } => apimokka_i18n::Key::UndoRuleAdded,
            Self::RemovedSubtree { .. } => apimokka_i18n::Key::UndoRuleDeleted,
        }
    }
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct App {
    pub view: AppView,
    pub locale: Locale,
    pub theme_choice: ThemeChoice,

    pub wizard: WizardState,
    pub dash_search: String,

    pub tab: WorkspaceTab,
    /// Transitional field name retained for screen/test compatibility. The
    /// value is the complete MK-053 session, not a mutable render snapshot.
    pub snapshot: Option<WorkspaceSession>,
    pub selection: RouteSelection,
    pub server_state: ServerState,
    pub runtime_in_flight: Option<RuntimeInFlight>,
    runtime_auto_complete: bool,
    next_session_generation: u64,

    pub dirty_count: usize,
    pub last_save_report: Option<GlobalSaveReport>,

    pub trace: Vec<apimokka_model::MatchTraceEvent>,
    pub selected_trace: Option<u64>,
    pub trace_paused: bool,
    /// Live filter string for the trace event list. Empty = show all. (MK-042)
    pub trace_filter: String,

    pub workspace_menu_open: bool,
    pub command_palette: CommandPaletteState,
    pub confirm_dialog: Option<ConfirmDialogState>,
    pub test_rule: TestRuleState,
    pub path_assistant: PathAssistantState,
    pub drawer: Option<DrawerMode>,

    // ── Sidebar collapse/accordion state ─────────────────────────────────
    /// Which rule set is currently expanded (accordion: only one at a time).
    pub rule_set_open: Option<apimokka_model::RuleSetId>,
    /// Whether the "Fallback files" section is expanded in the sidebar.
    pub fallback_section_open: bool,
    /// Whether the "Middleware scripts" section is expanded in the sidebar.
    pub middleware_section_open: bool,

    // ── Fallback file editor state (MK-038 two-buffer lifecycle) ──────────
    /// Saved baseline per file path — what is "on disk".
    pub fallback_saved: std::collections::HashMap<String, String>,
    /// Draft editor buffers per file path; created lazily on first open.
    /// `text_editor::Content` is stateful (rope) and intentionally not Clone.
    pub fallback_drafts: std::collections::HashMap<String, iced::widget::text_editor::Content>,
    /// Saved status code per file path (default "200 OK").
    pub fallback_status_saved: std::collections::HashMap<String, String>,
    /// Draft status code per file path.
    pub fallback_status_draft: std::collections::HashMap<String, String>,

    // ── MK-039: friendly feedback state ───────────────────────────────────
    /// A friendly error banner currently shown (None = no error).
    pub last_problem: Option<apimokka_model::FriendlyProblem>,
    pub transient_problem_kind: Option<TransientProblemKind>,
    transient_problem_operation: Option<TransientOperation>,
    pub show_problem_details: bool,
    pub audience_mode: Option<apimokka_model::AudienceMode>,
    /// Task 014 §4 (MK-023 first-screen gap): keyboard selection into
    /// `screens::mode_picker::OPTIONS`, shown only while `audience_mode` is
    /// `None`. `None` = nothing selected yet.
    pub mode_picker_selected: Option<usize>,
    // ── MK-041 layout density toggles ─────────────────────────────────────
    pub rule_when_more: bool,
    pub settings_advanced_more: bool,
    pub rule_set_config_more: bool,
    /// Transient success / info notice.
    pub notice: Option<String>,
}

enum DiagnosticNavigation {
    RuleSet(RuleSetId),
    Rule {
        id: NodeId,
        parent: RuleSetId,
    },
    Condition {
        id: NodeId,
        rule: NodeId,
        parent: RuleSetId,
        family: ConditionFamily,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransientProblemKind {
    Operation,
    Save,
    Runtime,
    Admission,
    PostCommitContract,
    NonAdoptingReadContract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransientOperation {
    Root(WorkspaceRootKey),
    RuleMatch {
        rule_id: NodeId,
        field: RuleMatchDraftField,
    },
    Respond {
        rule_id: NodeId,
        field: RespondDraftField,
    },
    Header {
        rule_id: NodeId,
        index: usize,
    },
    Body {
        rule_id: NodeId,
        index: usize,
    },
}

impl App {
    pub fn new() -> (Self, iced::Task<Message>) {
        // MK-046: no snapshot on first launch — Welcome screen shows first.
        let snapshot: Option<WorkspaceSession> = None;
        let sel = RouteSelection::default();
        let initial_rule_set_open = None;

        let app = Self {
            view: AppView::Welcome, // MK-046: start at Welcome, not Workspace
            locale: Locale::En,
            theme_choice: ThemeChoice::Light,
            wizard: WizardState::default(),
            dash_search: String::new(),
            tab: WorkspaceTab::Routes,
            snapshot,
            selection: sel,
            server_state: ServerState::Running,
            runtime_in_flight: None,
            runtime_auto_complete: true,
            next_session_generation: 1,
            dirty_count: 0,
            last_save_report: None,
            trace: mock::sample_trace_events(),
            selected_trace: None,
            trace_paused: false,
            trace_filter: String::new(),
            workspace_menu_open: false,
            command_palette: CommandPaletteState::default(),
            confirm_dialog: None,
            test_rule: TestRuleState::default(),
            path_assistant: PathAssistantState::default(),
            drawer: None,
            // Accordion: first rule set open by default (last added = first in list)
            rule_set_open: initial_rule_set_open,
            fallback_section_open: false,
            middleware_section_open: false,
            fallback_saved: std::collections::HashMap::new(),
            fallback_drafts: std::collections::HashMap::new(),
            fallback_status_saved: std::collections::HashMap::new(),
            fallback_status_draft: std::collections::HashMap::new(),
            last_problem: None,
            transient_problem_kind: None,
            transient_problem_operation: None,
            show_problem_details: false,
            audience_mode: None, // None → first-run picker shown
            mode_picker_selected: None,
            rule_when_more: false,
            settings_advanced_more: false,
            rule_set_config_more: false,
            notice: None,
        };
        (app, iced::Task::none())
    }

    pub fn title(&self) -> String {
        match &self.snapshot {
            Some(s) => format!("{} — apimokka", s.identity.name),
            None => "apimokka".into(),
        }
    }

    pub fn theme(&self) -> Theme {
        self.theme_choice.iced()
    }

    pub fn t(&self, key: Key) -> &'static str {
        self.locale.t(key)
    }

    /// MK-040: whether explanatory scaffolding (expanded hints, plain glosses)
    /// should render inline. True only in Guided mode. Defaults to false until
    /// the first-run picker is answered (the picker covers the window anyway).
    pub fn shows_scaffolding(&self) -> bool {
        matches!(
            self.audience_mode,
            Some(apimokka_model::AudienceMode::Guided)
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Update
    // ─────────────────────────────────────────────────────────────────────────

    pub fn update(&mut self, msg: Message) {
        if self
            .snapshot
            .as_ref()
            .is_some_and(|session| session.faulted)
            && is_workspace_mutation(&msg)
        {
            self.enter_session_fault_if_any();
            return;
        }
        match msg {
            Message::Noop => {}

            // Sidebar section toggles
            Message::ToggleFallbackSection => {
                self.fallback_section_open = !self.fallback_section_open;
            }
            Message::ToggleMiddlewareSection => {
                self.middleware_section_open = !self.middleware_section_open;
            }

            // MK-039/045: undo / redo
            Message::UndoLast | Message::Undo => {
                self.apply_undo();
            }
            Message::Redo => {
                self.apply_redo();
            }
            Message::DismissNotice => {
                // MK-049 audit: dismissing the banner clears the transient notice
                // text only. The undo_stack is NOT cleared — ⌘Z still works after
                // the user dismisses the "Deleted rule" banner.
                self.notice = None;
            }
            Message::DismissProblem => {
                self.clear_transient_problem();
            }
            Message::ProblemAction => {
                if self.last_problem.is_some() {
                    self.tab = crate::selection::WorkspaceTab::Settings;
                    self.clear_transient_problem();
                }
            }

            // MK-040: audience modes
            Message::ChooseAudienceMode(mode) => {
                self.audience_mode = Some(mode);
                // Expert shows technical detail inline by default; Guided hides it.
                self.show_problem_details = mode == apimokka_model::AudienceMode::Expert;
                // MK-041: reset density toggles when entering Guided so advanced
                // sections start collapsed (as designed). Expert leaves them as-is
                // since they are not rendered differently in Expert mode.
                if mode == apimokka_model::AudienceMode::Guided {
                    self.rule_when_more = false;
                    self.settings_advanced_more = false;
                    self.rule_set_config_more = false;
                }
            }
            Message::ToggleProblemDetails => {
                self.show_problem_details = !self.show_problem_details;
            }

            Message::ToggleRuleSetConfigMore => {
                self.rule_set_config_more = !self.rule_set_config_more;
            }
            Message::ToggleRuleWhenMore => {
                self.rule_when_more = !self.rule_when_more;
            }

            // MK-043: strategy / weight / priority
            Message::RuleSetSetStrategy(strategy) => {
                self.update_root_setting(
                    WorkspaceRootKey::ServiceStrategy,
                    WorkspaceEditValue::Enum(strategy.label().into()),
                );
            }
            Message::RuleWeightChanged(s) => {
                self.update_rule_prototype(|prototype| prototype.weight = s.parse::<u32>().ok());
            }
            Message::RulePriorityChanged(s) => {
                self.update_rule_prototype(|prototype| prototype.priority = s.parse::<i32>().ok());
            }
            Message::ToggleSettingsAdvancedMore => {
                self.settings_advanced_more = !self.settings_advanced_more;
            }

            // Navigation
            Message::GoWelcome => {
                if self.requires_workspace_confirmation() {
                    self.update(Message::ConfirmRequest(ConfirmAction::LeaveWorkspace));
                } else {
                    self.leave_workspace();
                }
            }
            Message::GoDashboard => {
                self.view = AppView::Dashboard;
            }
            Message::GoWizard => {
                self.wizard = WizardState::default();
                self.view = AppView::Wizard;
            }
            Message::OpenWorkspace(name) => {
                self.workspace_menu_open = false;
                if self.requires_workspace_confirmation() {
                    self.update(Message::ConfirmRequest(ConfirmAction::SwitchWorkspace(
                        name,
                    )));
                } else {
                    self.open_workspace(name);
                }
            }
            Message::SwitchTab(t) => {
                self.tab = t;
            }

            // Locale / theme
            Message::ChangeLocale(l) => {
                self.locale = l;
            }
            Message::ToggleTheme => {
                self.theme_choice = self.theme_choice.toggle();
            }
            Message::SetTheme(choice) => {
                self.theme_choice = choice;
            }

            // Keyboard
            Message::EscapePressed => {
                if self.confirm_dialog.is_some() {
                    self.confirm_dialog = None;
                } else if self.workspace_menu_open {
                    self.workspace_menu_open = false;
                } else if self.command_palette.open {
                    self.command_palette.open = false;
                } else if self.test_rule.open {
                    self.test_rule.open = false;
                } else if self.path_assistant.open {
                    self.path_assistant.open = false;
                } else if self.drawer.is_some() {
                    self.drawer = None;
                }
            }
            Message::ToggleCommandPalette => {
                // D-4 (found executing M8's capture, 2026-09-05):
                // `screens::command_palette::view` renders in exactly one
                // place, inside `AppView::Workspace` (`shell/view.rs`). Off
                // that view, flipping `open` produced a silent no-op that
                // once looked like a failed keystroke. Task 014 §5 option 2:
                // make the toggle itself inert outside its one rendering
                // location, so the state can no longer go out of sync with
                // what renders.
                if self.audience_mode.is_some() && self.view == AppView::Workspace {
                    self.command_palette.open = !self.command_palette.open;
                    self.command_palette.query = String::new();
                    self.command_palette.selected = None;
                }
            }
            Message::PaletteQuery(q) => {
                self.command_palette.query = q;
                let filtered_len =
                    crate::palette_commands::filtered_indices(self, &self.command_palette.query)
                        .len();
                self.command_palette.selected =
                    clamp_selection(self.command_palette.selected, filtered_len);
            }
            Message::ArrowUp => {
                if self.command_palette.open {
                    let filtered_len = crate::palette_commands::filtered_indices(
                        self,
                        &self.command_palette.query,
                    )
                    .len();
                    self.command_palette.selected =
                        move_selection(self.command_palette.selected, filtered_len, -1);
                } else if self.audience_mode.is_none() {
                    self.mode_picker_selected = move_selection(
                        self.mode_picker_selected,
                        screens::mode_picker::OPTIONS.len(),
                        -1,
                    );
                }
            }
            Message::ArrowDown => {
                if self.command_palette.open {
                    let filtered_len = crate::palette_commands::filtered_indices(
                        self,
                        &self.command_palette.query,
                    )
                    .len();
                    self.command_palette.selected =
                        move_selection(self.command_palette.selected, filtered_len, 1);
                } else if self.audience_mode.is_none() {
                    self.mode_picker_selected = move_selection(
                        self.mode_picker_selected,
                        screens::mode_picker::OPTIONS.len(),
                        1,
                    );
                }
            }
            Message::EnterPressed => {
                // Re-dispatches the selected row's own `Message`, exactly as
                // clicking it would (`GoWelcome`'s `ConfirmRequest`
                // re-dispatch above is the same idiom). This is why the
                // shared table matters: the message executed here is read
                // from the identical table and filter `view` used to render
                // the row the user is looking at, so the two can never
                // disagree about what "selected" points to.
                if self.command_palette.open {
                    if let Some(pos) = self.command_palette.selected {
                        let filtered = crate::palette_commands::filtered_indices(
                            self,
                            &self.command_palette.query,
                        );
                        if let Some(&table_index) = filtered.get(pos) {
                            let selected_message =
                                (crate::palette_commands::TABLE[table_index].message)();
                            self.update(selected_message);
                        }
                    }
                } else if self.audience_mode.is_none()
                    && let Some(pos) = self.mode_picker_selected
                    && let Some(&(_, _, mode)) = screens::mode_picker::OPTIONS.get(pos)
                {
                    self.update(Message::ChooseAudienceMode(mode));
                }
            }

            // Workspace menu
            Message::ToggleWorkspaceMenu => {
                self.workspace_menu_open = !self.workspace_menu_open;
            }
            Message::CloseWorkspaceMenu => {
                self.workspace_menu_open = false;
            }

            // Server actions
            Message::StartStopServer => {
                let action = match self.server_state {
                    ServerState::Stopped | ServerState::Error => Some(RuntimeAction::Start),
                    ServerState::Running => Some(RuntimeAction::Stop),
                    ServerState::Starting => None,
                };
                if let Some(action) = action {
                    self.request_runtime(action);
                }
            }
            Message::ReloadConfig => {
                self.request_runtime(RuntimeAction::Reload);
            }
            Message::RestartServer => {
                self.request_runtime(RuntimeAction::Restart);
            }
            Message::RuntimeSucceeded(token) => {
                self.complete_runtime_success(token);
            }
            Message::RuntimeFailed { token, technical } => {
                self.complete_runtime_failure(token, technical);
            }

            // Save
            Message::Save | Message::SaveAll => {
                if self.save_workspace_and_fallbacks() == Some(GlobalSaveCompletion::Complete) {
                    self.notice = Some(self.t(Key::FallbackSavedHint).to_string());
                }
            }
            Message::DiscardChanges => {
                self.discard_all_changes();
            }

            // Drawer
            Message::OpenValidationDrawer => {
                self.drawer = Some(DrawerMode::Validation);
                if let Some(session) = self.snapshot.as_mut()
                    && !session.faulted
                    && matches!(
                        session.validate(),
                        workspace_session::SessionValidationResult::ContractFault
                    )
                {
                    self.enter_session_fault_if_any();
                }
            }
            Message::OpenSaveDiffDrawer => {
                self.drawer = Some(DrawerMode::SaveDiff);
            }
            Message::CloseDrawer => {
                self.drawer = None;
            }

            // Dashboard
            Message::DashSearch(q) => {
                self.dash_search = q;
            }
            Message::DashPinToggle(_) => {}

            // Wizard
            Message::WizardSetName(v) => {
                self.wizard.name = v;
            }
            Message::WizardSetStarter(s) => {
                self.wizard.starter = s;
            } // MK-048
            Message::WizardSetFolder(v) => {
                self.wizard.folder = v;
            }
            Message::WizardSetHost(v) => {
                self.wizard.host = v;
            }
            Message::WizardSetPort(v) => {
                self.wizard.port = v;
            }
            Message::WizardSetTls(v) => {
                self.wizard.tls = v;
            }
            Message::WizardSetQueueSize(v) => {
                self.wizard.queue_size = v;
            }
            Message::WizardToggleSection(i) => {
                if i < 3 {
                    self.wizard.section_open[i] = !self.wizard.section_open[i];
                }
            }
            Message::WizardCreate => {
                if self.requires_workspace_confirmation() {
                    self.update(Message::ConfirmRequest(ConfirmAction::CreateWorkspace));
                } else {
                    self.create_workspace_from_wizard();
                }
            }
            Message::WizardCancel => {
                self.view = if self.snapshot.is_some() {
                    AppView::Workspace
                } else {
                    AppView::Welcome
                };
            }

            // Selection
            Message::SelectRuleSet(id) => {
                self.selection.select_rule_set(id);
                if let Some(session) = self.snapshot.as_mut() {
                    session.clear_condition_focus_unless_rule(None);
                }
                // Accordion: opening a rule set closes others
                self.rule_set_open = Some(id);
            }
            Message::SelectRule(id) => {
                let parent = self
                    .snapshot
                    .as_ref()
                    .and_then(|snapshot| snapshot.find_rule(id).map(|(rule_set, _)| rule_set.id));
                if let Some(parent) = parent {
                    self.selection.select_rule(id, parent);
                    self.rule_set_open = Some(parent);
                    if let Some(session) = self.snapshot.as_mut() {
                        session.clear_condition_focus_unless_rule(Some(id));
                    }
                }
            }
            Message::SelectFileRoute(s) => {
                // MK-038: opening a file the first time creates its draft
                // from the saved baseline (Untracked → Clean).
                if !self.fallback_drafts.contains_key(&s) {
                    let saved = self.fallback_saved.get(&s).cloned().unwrap_or_default();
                    self.fallback_drafts.insert(
                        s.clone(),
                        iced::widget::text_editor::Content::with_text(&saved),
                    );
                }
                if !self.fallback_status_draft.contains_key(&s) {
                    let saved = self
                        .fallback_status_saved
                        .get(&s)
                        .cloned()
                        .unwrap_or_else(|| "200 OK".into());
                    self.fallback_status_draft.insert(s.clone(), saved);
                }
                self.selection.select_fallback(s);
                if let Some(session) = self.snapshot.as_mut() {
                    session.clear_condition_focus_unless_rule(None);
                }
            }
            Message::SelectScript(s) => {
                self.selection.select_script(s);
                if let Some(session) = self.snapshot.as_mut() {
                    session.clear_condition_focus_unless_rule(None);
                }
            }
            Message::AddRuleSet => {
                let Some(session) = self.snapshot.as_mut() else {
                    return;
                };
                let n = session.rule_sets.len() + 1;
                let path = format!("rules/rule-set-{n}.toml");
                let key = session.creation_key("rule-set");
                let path = match parse_rule_set_path(&path) {
                    Ok(path) => path,
                    Err(error) => {
                        self.present_workspace_problem(
                            "Rule-set creation rejected",
                            error.to_string(),
                        );
                        return;
                    }
                };
                if let Some(outcome) =
                    self.apply_workspace_intent(EditIntent::AddRuleSet { path, key })
                    && let Some(receipt) = outcome
                        .creations
                        .iter()
                        .find(|receipt| receipt.kind == WorkspaceNodeKind::RuleSet)
                {
                    let id = RuleSetId(receipt.new_id);
                    self.selection.select_rule_set(id);
                    self.rule_set_open = Some(id);
                    if let Some(archive) = self.archive_rule_set(id) {
                        let prototypes = self.subtree_prototypes(&archive);
                        let bindings = subtree_bindings(&archive);
                        self.push_undo(HistoryEntry::AddedSubtree {
                            archive,
                            current_root: id.0,
                            bindings,
                            prototypes,
                        });
                    }
                }
            }
            Message::AddRule(rs_id) => {
                let Some(session) = self.snapshot.as_mut() else {
                    return;
                };
                let Some(insertion_index) = session
                    .rule_sets
                    .iter()
                    .find(|rs| rs.id == rs_id)
                    .map(|rs| rs.rules.len())
                else {
                    return;
                };
                let key = session.creation_key("rule");
                let rule = RuleEditPayload {
                    rule_match: map_rule_match("", None, "").expect("blank rule is valid"),
                    headers: CollectionEdit::Preserve,
                    body: CollectionEdit::Preserve,
                    respond: map_response(ResponseMode::Inline, "", "", "", "")
                        .expect("blank response is valid"),
                };
                if let Some(outcome) = self.apply_workspace_intent(EditIntent::AddRule {
                    parent: rs_id,
                    insertion_index,
                    rule,
                    key,
                }) && let Some(receipt) = outcome
                    .creations
                    .iter()
                    .find(|receipt| receipt.kind == WorkspaceNodeKind::Rule)
                {
                    let new_id = receipt.new_id;
                    self.selection.select_rule(new_id, rs_id);
                    self.rule_set_open = Some(rs_id);
                    if let Some(archive) = self.archive_rule(new_id) {
                        let prototypes = self.subtree_prototypes(&archive);
                        let bindings = subtree_bindings(&archive);
                        self.push_undo(HistoryEntry::AddedSubtree {
                            archive,
                            current_root: new_id,
                            bindings,
                            prototypes,
                        });
                    }
                }
            }
            Message::MoveRuleUp(id) => {
                let index = self
                    .snapshot
                    .as_ref()
                    .and_then(|snapshot| snapshot.find_rule(id))
                    .and_then(|(set, _)| set.rules.iter().position(|rule| rule.id == id));
                if let Some(index) = index.filter(|index| *index > 0)
                    && self
                        .apply_workspace_intent(EditIntent::MoveRule {
                            id,
                            new_index: index - 1,
                        })
                        .is_some()
                {
                    self.push_undo(HistoryEntry::MoveRule {
                        rule_id: id,
                        before_index: index,
                        after_index: index - 1,
                    });
                }
            }
            Message::MoveRuleDown(id) => {
                let target = self
                    .snapshot
                    .as_ref()
                    .and_then(|snapshot| snapshot.find_rule(id))
                    .and_then(|(set, _)| {
                        let index = set.rules.iter().position(|rule| rule.id == id)?;
                        (index + 1 < set.rules.len()).then_some(index + 1)
                    });
                if let Some(new_index) = target
                    && self
                        .apply_workspace_intent(EditIntent::MoveRule { id, new_index })
                        .is_some()
                {
                    self.push_undo(HistoryEntry::MoveRule {
                        rule_id: id,
                        before_index: new_index - 1,
                        after_index: new_index,
                    });
                }
            }
            Message::DeleteRuleSet(id) => {
                self.update(Message::ConfirmRequest(ConfirmAction::DeleteRuleSet(id)));
            }
            Message::DeleteRule(id) => {
                let archive = self.archive_rule(id);
                let prototypes = archive
                    .as_ref()
                    .map(|archive| self.subtree_prototypes(archive))
                    .unwrap_or_default();
                let deleted = self
                    .apply_workspace_intent(EditIntent::DeleteRule { id })
                    .is_some();
                if deleted && let Some(archive) = archive {
                    self.push_undo(HistoryEntry::RemovedSubtree {
                        bindings: subtree_bindings(&archive),
                        archive,
                        current_root: id,
                        prototypes,
                    });
                }
            }
            Message::DuplicateRule(id) => {
                let Some(session) = self.snapshot.as_mut() else {
                    return;
                };
                let Some((parent, insertion_index, source)) =
                    session.find_rule(id).and_then(|(set, _)| {
                        let index = set.rules.iter().position(|rule| rule.id == id)?;
                        let source = session.latest().rule(id)?.clone();
                        Some((set.id, index + 1, source))
                    })
                else {
                    return;
                };
                let headers = source
                    .conditions()
                    .headers
                    .iter()
                    .map(|condition| ConditionEdit::Create {
                        key: session.creation_key("header"),
                        condition: condition.condition.clone(),
                    })
                    .collect();
                let body = source
                    .conditions()
                    .body
                    .iter()
                    .map(|condition| ConditionEdit::Create {
                        key: session.creation_key("body"),
                        condition: condition.condition.clone(),
                    })
                    .collect();
                let key = session.creation_key("rule");
                let rule = RuleEditPayload {
                    rule_match: source.rule_match().clone(),
                    headers: CollectionEdit::Replace(headers),
                    body: CollectionEdit::Replace(body),
                    respond: source.respond().clone(),
                };
                if let Some(outcome) = self.apply_workspace_intent(EditIntent::AddRule {
                    parent,
                    insertion_index,
                    rule,
                    key,
                }) && let Some(receipt) = outcome
                    .creations
                    .iter()
                    .find(|receipt| receipt.kind == WorkspaceNodeKind::Rule)
                {
                    let new_id = receipt.new_id;
                    self.selection.select_rule(new_id, parent);
                    if let Some(extra) = self
                        .snapshot
                        .as_ref()
                        .and_then(|session| session.prototype.rule_extras.get(&id))
                        .cloned()
                        && let Some(session) = self.snapshot.as_mut()
                    {
                        session.prototype.rule_extras.insert(new_id, extra);
                    }
                    if let Some(archive) = self.archive_rule(new_id) {
                        let prototypes = self.subtree_prototypes(&archive);
                        let bindings = subtree_bindings(&archive);
                        self.push_undo(HistoryEntry::AddedSubtree {
                            archive,
                            current_root: new_id,
                            bindings,
                            prototypes,
                        });
                    }
                }
            }
            // Rule edits are draft-first and port-backed.
            Message::RuleSetUrlPath(v) => {
                self.update_rule_core(RuleMatchDraftField::UrlPath, |payload| {
                    payload.url_path = v;
                    if payload.url_path.is_empty() {
                        payload.url_path_op = None;
                    } else if payload.url_path_op.is_none() {
                        payload.url_path_op = Some(apimokka_model::UrlPathOp::Equal);
                    }
                });
            }
            Message::RuleSetUrlPathOp(op) => {
                self.update_rule_core(RuleMatchDraftField::UrlPathOp, |payload| {
                    payload.url_path_op = Some(op);
                });
            }
            Message::RuleSetUrlPathEnabled(v) => {
                self.update_rule_core(RuleMatchDraftField::UrlPathOp, |r| {
                    if !v {
                        r.url_path.clear();
                        r.url_path_op = None;
                    }
                });
            }
            Message::RuleSetMethod(m) => {
                self.update_rule_core(RuleMatchDraftField::Method, |payload| payload.method = m);
            }
            Message::HeaderAdd => {
                self.add_header_draft();
            }
            Message::HeaderRemove(i) => {
                self.remove_header_draft(i);
            }
            Message::HeaderSetName { index, value } => {
                self.update_header_draft(index, |condition| condition.name = value);
            }
            Message::HeaderSetOp { index, op } => {
                self.update_header_draft(index, |condition| condition.op = op);
            }
            Message::HeaderSetValue { index, value } => {
                self.update_header_draft(index, |condition| condition.value = value);
            }
            Message::HeaderClearAll => {
                self.clear_header_drafts();
            }
            Message::BodyAdd => {
                self.add_body_draft();
            }
            Message::BodyRemove(i) => {
                self.remove_body_draft(i);
            }
            Message::BodySetPath { index, value } => {
                self.update_body_draft(index, |condition| condition.path = value);
            }
            Message::BodySetOp { index, op } => {
                self.update_body_draft(index, |condition| condition.op = op);
            }
            Message::BodySetValue { index, value } => {
                self.update_body_draft(index, |condition| condition.value = value);
            }
            Message::BodyClearAll => {
                self.clear_body_drafts();
            }
            Message::BodyOpenPathAssistant(i) => {
                self.path_assistant.open = true;
                self.path_assistant.target_index = i;
                self.path_assistant.json_input = String::new();
                self.path_assistant.selected_path = String::new();
            }
            Message::RespondSetMode(m) => {
                self.update_response_draft(RespondDraftField::Mode, |respond| respond.mode = m);
            }
            Message::RespondSetText(v) => {
                self.update_response_draft(RespondDraftField::Text, |respond| respond.text = v);
            }
            Message::RespondSetFilePath(v) => {
                self.update_response_draft(RespondDraftField::FilePath, |respond| {
                    respond.file_path = v;
                });
            }
            Message::RespondSetStatus(v) => {
                self.update_response_draft(RespondDraftField::Status, |respond| respond.status = v);
            }
            Message::RespondSetDelay(v) => {
                self.update_response_delay_draft(v);
            }
            Message::RuleSetWeight(v) => {
                self.update_rule_prototype(|prototype| prototype.weight = v.parse().ok());
            }
            Message::RuleSetPriority(v) => {
                self.update_rule_prototype(|prototype| prototype.priority = v.parse().ok());
            }

            // Trace
            Message::JumpToTraceEvent(eid) => {
                // Switch to Trace tab and select the specific event.
                self.tab = crate::selection::WorkspaceTab::Trace;
                self.selected_trace = Some(eid);
            }
            Message::ViewAllInTrace => {
                self.tab = crate::selection::WorkspaceTab::Trace;
            }
            Message::TracePauseToggle => {
                self.trace_paused = !self.trace_paused;
            }
            Message::TraceClear => {
                self.trace.clear();
                self.selected_trace = None;
            }
            Message::SelectTraceEvent(id) => {
                self.selected_trace = Some(id);
            }
            Message::TraceFilterChanged(s) => {
                self.trace_filter = s;
            }
            Message::AddRuleFromPalette => {
                self.command_palette.open = false;
                self.tab = crate::selection::WorkspaceTab::Routes;
                let rs_id = self
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.rule_sets.first())
                    .map(|rs| rs.id);
                if let Some(id) = rs_id {
                    self.update(Message::SelectRuleSet(id));
                    self.update(Message::AddRule(id));
                }
            }
            Message::AddRuleForPath(path) => {
                self.tab = crate::selection::WorkspaceTab::Routes;
                let rs_id = self
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.rule_sets.first())
                    .map(|rs| rs.id);
                if let Some(id) = rs_id {
                    self.update(Message::AddRule(id));
                }
                self.update(Message::RuleSetUrlPath(path));
            }
            Message::JumpToRule(id) => {
                self.tab = crate::selection::WorkspaceTab::Routes;
                self.drawer = None; // MK-044: close drawer when navigating
                self.update(Message::SelectRule(id));
            }
            Message::JumpToFile(path) => {
                self.tab = crate::selection::WorkspaceTab::Routes;
                self.update(Message::SelectFileRoute(path));
            }
            Message::JumpToDiagnostic(id) => {
                self.navigate_to_diagnostic(id);
            }

            // Test rule
            Message::TestRuleOpen => {
                let (method, url_path) = self
                    .selected_rule()
                    .map(|r| {
                        let method = if r.payload.method.is_empty() {
                            "GET".to_owned()
                        } else {
                            r.payload.method.clone()
                        };
                        (method, r.payload.url_path.clone())
                    })
                    .unwrap_or_default();
                self.test_rule.method = method;
                self.test_rule.url_path = url_path;
                self.test_rule.result = None;
                self.test_rule.open = true;
            }
            Message::TestRuleClose => {
                self.test_rule.open = false;
            }
            Message::ReplayAsTestInput(eid) => {
                if let Some(ev) = self.trace.iter().find(|e| e.event_id == eid) {
                    let method = ev.request.method.clone();
                    let url_path = ev.request.url_path.clone();
                    let headers_text = ev
                        .request
                        .headers
                        .iter()
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let body = ev.request.body_preview.clone().unwrap_or_default();
                    self.test_rule.method = method;
                    self.test_rule.url_path = url_path;
                    self.test_rule.headers_text = headers_text;
                    self.test_rule.body = body;
                }
                self.test_rule.result = None;
                self.test_rule.open = true;
            }
            Message::TestRuleSetMethod(v) => {
                self.test_rule.method = v;
                self.test_rule.result = None;
            }
            Message::TestRuleSetPath(v) => {
                self.test_rule.url_path = v;
                self.test_rule.result = None;
            }
            Message::TestRuleSetHeaders(v) => {
                self.test_rule.headers_text = v;
                self.test_rule.result = None;
            }
            Message::TestRuleSetBody(v) => {
                self.test_rule.body = v;
                self.test_rule.result = None;
            }
            Message::TestRuleRun => {
                self.test_rule.result = Some(crate::match_test::evaluate(
                    self.selected_rule_payload(),
                    TestRequest {
                        method: &self.test_rule.method,
                        url_path: &self.test_rule.url_path,
                        headers: &self.test_rule.headers_text,
                        body: &self.test_rule.body,
                    },
                ));
            }

            // Dotted-path assistant
            Message::PathAssistantOpen(i) => {
                self.path_assistant.open = true;
                self.path_assistant.target_index = i;
                self.path_assistant.json_input = String::new();
                self.path_assistant.selected_path = String::new();
            }
            Message::PathAssistantClose => {
                self.path_assistant.open = false;
            }
            Message::PathAssistantSetJson(v) => {
                self.path_assistant.json_input = v;
            }
            Message::PathAssistantSelectPath(p) => {
                self.path_assistant.selected_path = p;
            }
            Message::PathAssistantInsert => {
                let path = self.path_assistant.selected_path.clone();
                let index = self.path_assistant.target_index;
                self.path_assistant.open = false;
                self.update(Message::BodySetPath { index, value: path });
            }

            // Confirm dialog
            Message::ConfirmRequest(action) => {
                let (title, body) = match &action {
                    ConfirmAction::DeleteRuleSet(_) => {
                        (Key::ConfirmDeleteRuleSet, Key::ConfirmDeleteRuleSetBody)
                    }
                    ConfirmAction::DiscardChanges => {
                        (Key::ConfirmDiscardChanges, Key::ConfirmDiscardChangesBody)
                    }
                    ConfirmAction::SwitchWorkspace(_) => {
                        (Key::ConfirmSwitchWorkspace, Key::ConfirmSwitchWorkspaceBody)
                    }
                    ConfirmAction::LeaveWorkspace => {
                        (Key::ConfirmSwitchWorkspace, Key::ConfirmSwitchWorkspaceBody)
                    }
                    ConfirmAction::CreateWorkspace => {
                        (Key::ConfirmSwitchWorkspace, Key::ConfirmSwitchWorkspaceBody)
                    }
                    ConfirmAction::RevertFile(_) => {
                        (Key::ConfirmRevertFile, Key::ConfirmRevertFileBody)
                    }
                };
                self.confirm_dialog = Some(ConfirmDialogState {
                    action,
                    title,
                    body,
                });
            }
            Message::ConfirmCancel => {
                self.confirm_dialog = None;
            }
            Message::ConfirmProceed => {
                if let Some(d) = self.confirm_dialog.take() {
                    match d.action {
                        ConfirmAction::DeleteRuleSet(id) => {
                            let archive = self.archive_rule_set(id);
                            let prototypes = archive
                                .as_ref()
                                .map(|archive| self.subtree_prototypes(archive))
                                .unwrap_or_default();
                            let removed = self
                                .apply_workspace_intent(EditIntent::RemoveRuleSet { id })
                                .is_some();
                            if removed && let Some(archive) = archive {
                                self.push_undo(HistoryEntry::RemovedSubtree {
                                    bindings: subtree_bindings(&archive),
                                    archive,
                                    current_root: id.0,
                                    prototypes,
                                });
                            }
                        }
                        ConfirmAction::DiscardChanges => {
                            self.discard_all_changes();
                        }
                        ConfirmAction::SwitchWorkspace(name) => {
                            self.open_workspace(name);
                        }
                        ConfirmAction::LeaveWorkspace => {
                            self.leave_workspace();
                        }
                        ConfirmAction::CreateWorkspace => {
                            self.create_workspace_from_wizard();
                        }
                        ConfirmAction::RevertFile(path) => {
                            // MK-038: draft ← saved (Dirty → Clean)
                            let saved = self.fallback_saved.get(&path).cloned().unwrap_or_default();
                            self.fallback_drafts.insert(
                                path.clone(),
                                iced::widget::text_editor::Content::with_text(&saved),
                            );
                            let status = self
                                .fallback_status_saved
                                .get(&path)
                                .cloned()
                                .unwrap_or_else(|| "200 OK".into());
                            self.fallback_status_draft.insert(path, status);
                            self.recompute_dirty();
                        }
                    }
                }
            }

            // Settings
            Message::SettingsSetHost(v) => {
                if let Some(session) = self.snapshot.as_mut() {
                    session.root_drafts.listener_ip = v.clone();
                }
                self.update_root_setting(
                    WorkspaceRootKey::ListenerIpAddress,
                    WorkspaceEditValue::String(v),
                );
            }
            Message::SettingsSetPort(v) => {
                if let Some(session) = self.snapshot.as_mut() {
                    session.root_drafts.listener_port = v.clone();
                }
                let value = match v.parse::<i64>() {
                    Ok(value) => value,
                    Err(error) => {
                        self.present_operation_problem(
                            TransientOperation::Root(WorkspaceRootKey::ListenerPort),
                            "Listener port rejected",
                            error.to_string(),
                        );
                        return;
                    }
                };
                self.update_root_setting(
                    WorkspaceRootKey::ListenerPort,
                    WorkspaceEditValue::Integer(value),
                );
            }
            Message::SettingsSetTls(v) => {
                self.update_root_setting(
                    WorkspaceRootKey::TlsEnabled,
                    WorkspaceEditValue::Boolean(v),
                );
            }
            Message::SettingsSetLogLevel(v) => {
                self.update_root_setting(WorkspaceRootKey::LogLevel, WorkspaceEditValue::Enum(v));
            }
            Message::SettingsSetStrategy(st) => {
                self.update_root_setting(
                    WorkspaceRootKey::ServiceStrategy,
                    WorkspaceEditValue::Enum(st.label().into()),
                );
            }
            Message::SettingsSetTraceEnabled(v) => {
                if let Some(trace) = self
                    .snapshot
                    .as_mut()
                    .and_then(|session| session.prototype.trace.as_mut())
                {
                    let before = trace.clone();
                    trace.enabled = v;
                    let after = trace.clone();
                    if before != after {
                        self.push_undo(HistoryEntry::TracePrototype { before, after });
                    }
                }
            }

            // ── Fallback file editor (MK-038) ─────────────────────────────────
            Message::FallbackEditorAction(action) => {
                if let Some(path) = self.selection.file_route.clone() {
                    if let Some(content) = self.fallback_drafts.get_mut(&path) {
                        content.perform(action);
                    }
                    self.recompute_dirty();
                }
            }
            Message::FallbackFileSetStatus(v) => {
                if let Some(path) = self.selection.file_route.clone() {
                    self.fallback_status_draft.insert(path, v);
                    self.recompute_dirty();
                }
            }
            Message::FallbackFileFormat => {
                if let Some(path) = self.selection.file_route.clone()
                    && let Some(content) = self.fallback_drafts.get(&path)
                {
                    let raw = content.text();
                    // Pretty-print only if the draft parses; otherwise keep as-is.
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw)
                        && let Ok(pretty) = serde_json::to_string_pretty(&val)
                    {
                        self.fallback_drafts
                            .insert(path, iced::widget::text_editor::Content::with_text(&pretty));
                        self.recompute_dirty();
                    }
                }
            }
            Message::FallbackFileSave => {
                if let Some(path) = self.selection.file_route.clone() {
                    self.commit_fallback_draft(&path);
                    self.recompute_dirty();
                }
            }
            Message::FallbackFileRevert => {
                if self.selection.file_route.is_some()
                    && let Some(path) = self.selection.file_route.clone()
                    && self.is_fallback_dirty(&path)
                {
                    self.update(Message::ConfirmRequest(ConfirmAction::RevertFile(path)));
                }
            }
        }
    }

    /// Runtime entry point (wired in `main.rs`): `update`, plus the `Task`s
    /// iced's own runtime needs that `update` itself cannot issue.
    ///
    /// `update` above is a pure reducer — every one of this codebase's
    /// app-level tests calls it directly and relies on that purity, per
    /// task 014's own verification guidance. Moving a widget's focus
    /// (MK-033 lines 38, 95, 118: the search field must focus when the
    /// palette opens) or its scroll position (task 017 D-5: keep the
    /// selection visible as arrow keys move it) are not state mutations
    /// `update` can express; both only happen through a `Task` returned to
    /// iced's runtime. Rather than making `update` return `Task<Message>` —
    /// which would force all 486+ existing `.update(...)` call sites in
    /// tests to handle a `#[must_use]` value they have no runtime to poll —
    /// both genuine side effects are isolated here, in the thin wrapper
    /// only `main.rs` calls.
    pub fn update_and_dispatch(&mut self, msg: Message) -> iced::Task<Message> {
        let opening_palette =
            matches!(msg, Message::ToggleCommandPalette) && !self.command_palette.open;
        let is_arrow_key = matches!(msg, Message::ArrowUp | Message::ArrowDown);
        self.update(msg);

        if opening_palette && self.command_palette.open {
            return iced::widget::operation::focus(screens::command_palette::SEARCH_INPUT_ID);
        }

        // D-5: `scrollable` has no "scroll this child into view" operation —
        // only an absolute/relative viewport offset (confirmed against
        // `iced_runtime::widget::operation::scrollable`; nothing under
        // `advanced` exposes per-row bounds either). This estimates the
        // right offset from the selection's position in the filtered list
        // rather than its pixel position, which is not measured anywhere:
        // row 0 of N -> top of the viewport, the last row -> bottom, evenly
        // spaced between. Every row is the same fixed shape (one text line
        // plus an optional shortcut chip), so a linear map is a reasonable
        // approximation of the real layout, not merely a guess — verified
        // live against the actual scrollbar position, not assumed.
        if is_arrow_key
            && self.command_palette.open
            && let Some(selected) = self.command_palette.selected
        {
            let filtered_len =
                crate::palette_commands::filtered_indices(self, &self.command_palette.query).len();
            let last_index = filtered_len.saturating_sub(1).max(1) as f32;
            let y = selected as f32 / last_index;
            return iced::widget::operation::snap_to(
                screens::command_palette::RESULTS_SCROLLABLE_ID,
                iced::widget::scrollable::RelativeOffset { x: 0.0, y },
            );
        }

        iced::Task::none()
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn apply_workspace_intent(&mut self, intent: EditIntent) -> Option<EditOutcome> {
        self.apply_workspace_transaction(vec![intent])
    }

    fn apply_workspace_operation(
        &mut self,
        operation: TransientOperation,
        intent: EditIntent,
    ) -> Option<EditOutcome> {
        let outcome = self.apply_workspace_intent(intent);
        if outcome.is_none() && self.transient_problem_kind == Some(TransientProblemKind::Operation)
        {
            self.transient_problem_operation = Some(operation);
        }
        outcome
    }

    fn apply_workspace_transaction(&mut self, intents: Vec<EditIntent>) -> Option<EditOutcome> {
        if self
            .snapshot
            .as_ref()
            .is_some_and(|session| session.faulted)
        {
            self.enter_session_fault_if_any();
            return None;
        }
        let transaction = match EditTransaction::new(intents) {
            Ok(transaction) => transaction,
            Err(error) => {
                self.present_workspace_problem("Configuration edit rejected", error.to_string());
                return None;
            }
        };
        let selection_target = self.capture_selection_target();
        let result = self.snapshot.as_mut()?.apply(transaction);
        match result {
            workspace_session::SessionApplyResult::Validated(outcome) => {
                self.reconcile_selection(selection_target);
                self.recompute_dirty();
                Some(*outcome)
            }
            workspace_session::SessionApplyResult::ApplyFailure(failure) => {
                self.present_workspace_problem(
                    "Configuration edit rejected",
                    failure.diagnostic.message,
                );
                None
            }
            workspace_session::SessionApplyResult::ContractFault => {
                self.reconcile_selection(selection_target);
                self.enter_session_fault_if_any();
                None
            }
        }
    }

    fn capture_selection_target(&self) -> RouteTarget {
        self.snapshot
            .as_ref()
            .map(|session| self.selection.capture(session))
            .unwrap_or(RouteTarget::None)
    }

    fn navigate_to_diagnostic(&mut self, id: NodeId) {
        let target = self.snapshot.as_ref().and_then(|session| {
            if let Some(rule_set) = session.find_rule_set(RuleSetId(id)) {
                return Some(DiagnosticNavigation::RuleSet(rule_set.id));
            }
            if let Some((rule_set, _)) = session.find_rule(id) {
                return Some(DiagnosticNavigation::Rule {
                    id,
                    parent: rule_set.id,
                });
            }
            for rule in session.latest().rules() {
                if rule
                    .conditions()
                    .headers
                    .iter()
                    .any(|condition| condition.id == id)
                {
                    let parent = session.find_rule(rule.rule_id())?.0.id;
                    return Some(DiagnosticNavigation::Condition {
                        id,
                        rule: rule.rule_id(),
                        parent,
                        family: ConditionFamily::Header,
                    });
                }
                if rule
                    .conditions()
                    .body
                    .iter()
                    .any(|condition| condition.id == id)
                {
                    let parent = session.find_rule(rule.rule_id())?.0.id;
                    return Some(DiagnosticNavigation::Condition {
                        id,
                        rule: rule.rule_id(),
                        parent,
                        family: ConditionFamily::Body,
                    });
                }
            }
            None
        });
        match target {
            Some(DiagnosticNavigation::RuleSet(id)) => {
                self.selection.select_rule_set(id);
                self.rule_set_open = Some(id);
            }
            Some(DiagnosticNavigation::Rule { id, parent }) => {
                self.selection.select_rule(id, parent);
                self.rule_set_open = Some(parent);
            }
            Some(DiagnosticNavigation::Condition {
                id,
                rule,
                parent,
                family,
            }) => {
                self.selection.select_rule(rule, parent);
                self.rule_set_open = Some(parent);
                if let Some(session) = self.snapshot.as_mut() {
                    session.focus_condition(rule, family, DraftBinding::Existing(id));
                }
            }
            None => return,
        }
        self.tab = WorkspaceTab::Routes;
        self.drawer = None;
    }

    fn reconcile_selection(&mut self, target: RouteTarget) {
        if let Some(session) = self.snapshot.as_mut() {
            self.selection.reconcile(session, target);
            session.clear_condition_focus_unless_rule(self.selection.rule);
            self.rule_set_open = self.selection.rule_set;
        } else {
            self.selection = RouteSelection::default();
            self.rule_set_open = None;
        }
    }

    fn present_workspace_problem(&mut self, title: &str, technical: String) {
        self.transient_problem_kind = Some(TransientProblemKind::Operation);
        self.transient_problem_operation = None;
        self.last_problem = Some(
            apimokka_model::FriendlyProblem::new(
                title,
                "The canonical workspace was not changed. Correct the field and try again.",
                None,
            )
            .with_technical(technical),
        );
    }

    fn present_operation_problem(
        &mut self,
        operation: TransientOperation,
        title: &str,
        technical: String,
    ) {
        self.present_workspace_problem(title, technical);
        self.transient_problem_operation = Some(operation);
    }

    fn clear_operation_problem(&mut self, operation: TransientOperation) {
        if self.transient_problem_kind == Some(TransientProblemKind::Operation)
            && self.transient_problem_operation == Some(operation)
        {
            self.clear_transient_problem();
        }
    }

    fn clear_transient_problem(&mut self) {
        self.last_problem = None;
        self.transient_problem_kind = None;
        self.transient_problem_operation = None;
    }

    fn present_adopted_workspace_problem(&mut self, title: &str, technical: String) {
        self.transient_problem_kind = Some(TransientProblemKind::PostCommitContract);
        self.transient_problem_operation = None;
        self.last_problem = Some(
            apimokka_model::FriendlyProblem::new(
                title,
                "The workspace returned changed canonical state, but its result violated the editing contract. Reload the workspace before editing again.",
                None,
            )
            .with_technical(technical),
        );
    }

    fn present_cached_workspace_problem(&mut self, title: &str, technical: String) {
        self.transient_problem_kind = Some(TransientProblemKind::NonAdoptingReadContract);
        self.transient_problem_operation = None;
        self.last_problem = Some(
            apimokka_model::FriendlyProblem::new(
                title,
                "The cached canonical workspace was retained, but validation no longer matches the workspace port. Reload before editing again.",
                None,
            )
            .with_technical(technical),
        );
    }

    // Semantic history compensation is implemented through the workspace port.
    fn push_undo(&mut self, command: HistoryEntry) {
        let Some(session) = self.snapshot.as_mut() else {
            return;
        };
        session.redo_stack.clear();
        session.undo_stack.push(command);
        if session.undo_stack.len() > UNDO_STACK_DEPTH {
            session.undo_stack.remove(0);
        }
    }

    fn apply_undo(&mut self) {
        let Some(mut command) = self
            .snapshot
            .as_mut()
            .and_then(|session| session.undo_stack.pop())
        else {
            return;
        };
        let success = match &mut command {
            HistoryEntry::MoveRule {
                rule_id,
                before_index,
                ..
            } => self
                .apply_workspace_intent(EditIntent::MoveRule {
                    id: *rule_id,
                    new_index: *before_index,
                })
                .is_some(),
            HistoryEntry::RuleMatch {
                rule_id, before, ..
            } => self.apply_rule_match_compensation(*rule_id, before.clone()),
            HistoryEntry::Respond {
                rule_id, before, ..
            } => self
                .apply_workspace_intent(EditIntent::UpdateRespond {
                    id: *rule_id,
                    respond: before.clone(),
                })
                .is_some(),
            HistoryEntry::RootSetting { before, .. } => self
                .apply_workspace_intent(EditIntent::UpdateRootSetting(before.clone()))
                .is_some(),
            HistoryEntry::RulePrototype {
                rule_id, before, ..
            } => {
                if let Some(session) = self.snapshot.as_mut() {
                    session
                        .prototype
                        .rule_extras
                        .insert(*rule_id, before.clone());
                    true
                } else {
                    false
                }
            }
            HistoryEntry::TracePrototype { before, .. } => {
                if let Some(session) = self.snapshot.as_mut() {
                    session.prototype.trace = Some(before.clone());
                    true
                } else {
                    false
                }
            }
            HistoryEntry::HeaderAdd { current_id, .. } => self
                .apply_workspace_intent(EditIntent::RemoveHeaderCondition { id: *current_id })
                .is_some(),
            HistoryEntry::HeaderUpdate {
                current_id, before, ..
            } => self
                .apply_workspace_intent(EditIntent::UpdateHeaderCondition {
                    id: *current_id,
                    condition: before.clone(),
                })
                .is_some(),
            HistoryEntry::HeaderRemove {
                rule_id,
                index,
                key,
                condition,
                current_id,
            } => {
                let mut entries = vec![(*index, key.clone(), condition.clone(), *current_id)];
                let success = self.restore_headers(*rule_id, &mut entries);
                *current_id = entries[0].3;
                success
            }
            HistoryEntry::HeadersClear { rule_id, entries } => {
                self.restore_headers(*rule_id, entries)
            }
            HistoryEntry::BodyAdd { current_id, .. } => self
                .apply_workspace_intent(EditIntent::RemoveBodyCondition { id: *current_id })
                .is_some(),
            HistoryEntry::BodyUpdate {
                current_id, before, ..
            } => self
                .apply_workspace_intent(EditIntent::UpdateBodyCondition {
                    id: *current_id,
                    condition: before.clone(),
                })
                .is_some(),
            HistoryEntry::BodyRemove {
                rule_id,
                index,
                key,
                condition,
                current_id,
            } => {
                let mut entries = vec![(*index, key.clone(), condition.clone(), *current_id)];
                let success = self.restore_bodies(*rule_id, &mut entries);
                *current_id = entries[0].3;
                success
            }
            HistoryEntry::BodiesClear { rule_id, entries } => {
                self.restore_bodies(*rule_id, entries)
            }
            HistoryEntry::AddedSubtree {
                archive,
                current_root,
                ..
            } => {
                let intent = match archive
                    .nodes()
                    .iter()
                    .find(|node| node.old_id == archive.former_root())
                    .map(|node| node.payload.kind())
                {
                    Some(WorkspaceNodeKind::RuleSet) => EditIntent::RemoveRuleSet {
                        id: RuleSetId(*current_root),
                    },
                    Some(WorkspaceNodeKind::Rule) => EditIntent::DeleteRule { id: *current_root },
                    _ => return,
                };
                self.apply_workspace_intent(intent).is_some()
            }
            HistoryEntry::RemovedSubtree {
                archive,
                current_root,
                bindings,
                prototypes,
            } => self.restore_history_subtree(archive.clone(), current_root, bindings, prototypes),
        };
        if success {
            self.sync_history_drafts(&command);
        }
        if let Some(session) = self.snapshot.as_mut() {
            if success {
                session.redo_stack.push(command);
            } else if !session.faulted {
                session.undo_stack.push(command);
            }
        }
    }

    fn apply_redo(&mut self) {
        let Some(mut command) = self
            .snapshot
            .as_mut()
            .and_then(|session| session.redo_stack.pop())
        else {
            return;
        };
        let success = match &mut command {
            HistoryEntry::MoveRule {
                rule_id,
                after_index,
                ..
            } => self
                .apply_workspace_intent(EditIntent::MoveRule {
                    id: *rule_id,
                    new_index: *after_index,
                })
                .is_some(),
            HistoryEntry::RuleMatch { rule_id, after, .. } => {
                self.apply_rule_match_compensation(*rule_id, after.clone())
            }
            HistoryEntry::Respond { rule_id, after, .. } => self
                .apply_workspace_intent(EditIntent::UpdateRespond {
                    id: *rule_id,
                    respond: after.clone(),
                })
                .is_some(),
            HistoryEntry::RootSetting { after, .. } => self
                .apply_workspace_intent(EditIntent::UpdateRootSetting(after.clone()))
                .is_some(),
            HistoryEntry::RulePrototype { rule_id, after, .. } => {
                if let Some(session) = self.snapshot.as_mut() {
                    session
                        .prototype
                        .rule_extras
                        .insert(*rule_id, after.clone());
                    true
                } else {
                    false
                }
            }
            HistoryEntry::TracePrototype { after, .. } => {
                if let Some(session) = self.snapshot.as_mut() {
                    session.prototype.trace = Some(after.clone());
                    true
                } else {
                    false
                }
            }
            HistoryEntry::HeaderAdd {
                rule_id,
                key,
                condition,
                current_id,
            } => {
                let outcome = self.apply_workspace_intent(EditIntent::AddHeaderCondition {
                    rule_id: *rule_id,
                    condition: condition.clone(),
                    key: key.clone(),
                });
                if let Some(receipt) = outcome.and_then(|outcome| {
                    outcome
                        .creations
                        .into_iter()
                        .find(|receipt| receipt.kind == WorkspaceNodeKind::HeaderCondition)
                }) {
                    let old_id = *current_id;
                    *current_id = receipt.new_id;
                    self.apply_history_id_map(&std::collections::HashMap::from([(
                        old_id,
                        receipt.new_id,
                    )]));
                    true
                } else {
                    false
                }
            }
            HistoryEntry::HeaderUpdate {
                current_id, after, ..
            } => self
                .apply_workspace_intent(EditIntent::UpdateHeaderCondition {
                    id: *current_id,
                    condition: after.clone(),
                })
                .is_some(),
            HistoryEntry::HeaderRemove { current_id, .. } => self
                .apply_workspace_intent(EditIntent::RemoveHeaderCondition { id: *current_id })
                .is_some(),
            HistoryEntry::HeadersClear { entries, .. } => self
                .apply_workspace_transaction(
                    entries
                        .iter()
                        .map(|entry| EditIntent::RemoveHeaderCondition { id: entry.3 })
                        .collect(),
                )
                .is_some(),
            HistoryEntry::BodyAdd {
                rule_id,
                key,
                condition,
                current_id,
            } => {
                let outcome = self.apply_workspace_intent(EditIntent::AddBodyCondition {
                    rule_id: *rule_id,
                    condition: condition.clone(),
                    key: key.clone(),
                });
                if let Some(receipt) = outcome.and_then(|outcome| {
                    outcome
                        .creations
                        .into_iter()
                        .find(|receipt| receipt.kind == WorkspaceNodeKind::BodyCondition)
                }) {
                    let old_id = *current_id;
                    *current_id = receipt.new_id;
                    self.apply_history_id_map(&std::collections::HashMap::from([(
                        old_id,
                        receipt.new_id,
                    )]));
                    true
                } else {
                    false
                }
            }
            HistoryEntry::BodyUpdate {
                current_id, after, ..
            } => self
                .apply_workspace_intent(EditIntent::UpdateBodyCondition {
                    id: *current_id,
                    condition: after.clone(),
                })
                .is_some(),
            HistoryEntry::BodyRemove { current_id, .. } => self
                .apply_workspace_intent(EditIntent::RemoveBodyCondition { id: *current_id })
                .is_some(),
            HistoryEntry::BodiesClear { entries, .. } => self
                .apply_workspace_transaction(
                    entries
                        .iter()
                        .map(|entry| EditIntent::RemoveBodyCondition { id: entry.3 })
                        .collect(),
                )
                .is_some(),
            HistoryEntry::AddedSubtree {
                archive,
                current_root,
                bindings,
                prototypes,
            } => self.restore_history_subtree(archive.clone(), current_root, bindings, prototypes),
            HistoryEntry::RemovedSubtree {
                archive,
                current_root,
                ..
            } => {
                let intent = match archive
                    .nodes()
                    .iter()
                    .find(|node| node.old_id == archive.former_root())
                    .map(|node| node.payload.kind())
                {
                    Some(WorkspaceNodeKind::RuleSet) => EditIntent::RemoveRuleSet {
                        id: RuleSetId(*current_root),
                    },
                    Some(WorkspaceNodeKind::Rule) => EditIntent::DeleteRule { id: *current_root },
                    _ => return,
                };
                self.apply_workspace_intent(intent).is_some()
            }
        };
        if success {
            self.sync_history_drafts(&command);
        }
        if let Some(session) = self.snapshot.as_mut() {
            if success {
                session.undo_stack.push(command);
            } else if !session.faulted {
                session.redo_stack.push(command);
            }
        }
    }

    fn apply_rule_match_compensation(
        &mut self,
        id: NodeId,
        rule_match: apimokka_model::RuleMatch,
    ) -> bool {
        let Some(mut rule) = self.current_rule_edit(id) else {
            return false;
        };
        rule.rule_match = rule_match;
        self.apply_workspace_intent(EditIntent::UpdateRule { id, rule })
            .is_some()
    }

    fn sync_root_setting_draft(&mut self, key: WorkspaceRootKey) {
        let Some(session) = self.snapshot.as_mut() else {
            return;
        };
        match key {
            WorkspaceRootKey::ListenerIpAddress => {
                session.root_drafts.listener_ip = session.root_settings.listener_ip.clone();
            }
            WorkspaceRootKey::ListenerPort => {
                session.root_drafts.listener_port = session.root_settings.listener_port.to_string();
            }
            _ => {}
        }
    }

    fn sync_rule_match_draft(&mut self, rule_id: NodeId, field: RuleMatchDraftField) {
        let projected = self.snapshot.as_ref().and_then(|session| {
            let canonical = session.latest().rule(rule_id)?.rule_match();
            Some((
                canonical.url_path().unwrap_or_default().to_owned(),
                canonical.url_path_op(),
                canonical.method().unwrap_or_default().to_owned(),
            ))
        });
        if let (Some((url_path, url_path_op, method)), Some(draft)) = (
            projected,
            self.snapshot
                .as_mut()
                .and_then(|session| session.rule_drafts.get_mut(&rule_id)),
        ) {
            match field {
                RuleMatchDraftField::UrlPath => {
                    draft.payload.url_path = url_path;
                    draft.payload.url_path_op = url_path_op;
                }
                RuleMatchDraftField::UrlPathOp => draft.payload.url_path_op = url_path_op,
                RuleMatchDraftField::Method => draft.payload.method = method,
            }
        }
    }

    fn sync_respond_draft(&mut self, rule_id: NodeId, field: RespondDraftField) {
        let projected = self.snapshot.as_ref().and_then(|session| {
            let respond = session.latest().rule(rule_id)?.respond();
            let mode = if respond.file_path().is_some() {
                apimokka_model::snapshot::RespondMode::ServeFile
            } else {
                apimokka_model::snapshot::RespondMode::InlineText
            };
            Some((
                apimokka_model::respond::RespondPayload {
                    mode,
                    text: respond.text().unwrap_or_default().to_owned(),
                    file_path: respond
                        .file_path()
                        .map(|path| path.as_str().to_owned())
                        .unwrap_or_default(),
                    status: respond.status().unwrap_or_default().to_owned(),
                    delay_milliseconds: respond.delay_milliseconds().unwrap_or_default(),
                },
                respond
                    .delay_milliseconds()
                    .map(|delay| delay.to_string())
                    .unwrap_or_default(),
            ))
        });
        if let (Some((respond, delay)), Some(draft)) = (
            projected,
            self.snapshot
                .as_mut()
                .and_then(|session| session.rule_drafts.get_mut(&rule_id)),
        ) {
            match field {
                RespondDraftField::Mode => draft.payload.respond.mode = respond.mode,
                RespondDraftField::Text => draft.payload.respond.text = respond.text,
                RespondDraftField::FilePath => {
                    draft.payload.respond.file_path = respond.file_path;
                }
                RespondDraftField::Status => draft.payload.respond.status = respond.status,
                RespondDraftField::Delay => {
                    draft.payload.respond.delay_milliseconds = respond.delay_milliseconds;
                    draft.response_delay = delay;
                }
            }
        }
    }

    fn header_parent(&self, condition_id: NodeId) -> Option<NodeId> {
        self.snapshot
            .as_ref()?
            .latest()
            .rules()
            .iter()
            .find_map(|rule| {
                rule.conditions()
                    .headers
                    .iter()
                    .any(|condition| condition.id == condition_id)
                    .then_some(rule.rule_id())
            })
    }

    fn body_parent(&self, condition_id: NodeId) -> Option<NodeId> {
        self.snapshot
            .as_ref()?
            .latest()
            .rules()
            .iter()
            .find_map(|rule| {
                rule.conditions()
                    .body
                    .iter()
                    .any(|condition| condition.id == condition_id)
                    .then_some(rule.rule_id())
            })
    }

    fn sync_header_history_item(
        &mut self,
        rule_id: NodeId,
        condition_id: NodeId,
        preferred_index: Option<usize>,
    ) {
        let projected = self.snapshot.as_ref().and_then(|session| {
            let canonical = session.latest().rule(rule_id)?;
            let position = canonical
                .conditions()
                .headers
                .iter()
                .position(|condition| condition.id == condition_id)?;
            let payload = session
                .find_rule(rule_id)?
                .1
                .payload
                .headers
                .get(position)?
                .clone();
            Some((position, payload))
        });
        let Some(draft) = self
            .snapshot
            .as_mut()
            .and_then(|session| session.rule_drafts.get_mut(&rule_id))
        else {
            return;
        };
        let draft_position = draft
            .header_bindings
            .iter()
            .position(|binding| *binding == DraftBinding::Existing(condition_id));
        match (projected, draft_position) {
            (Some((_, payload)), Some(index)) => draft.payload.headers[index] = payload,
            (Some((canonical_index, payload)), None) => {
                let index = preferred_index
                    .unwrap_or(canonical_index)
                    .min(draft.payload.headers.len());
                draft.payload.headers.insert(index, payload);
                draft
                    .header_bindings
                    .insert(index, DraftBinding::Existing(condition_id));
            }
            (None, Some(index)) => {
                draft.payload.headers.remove(index);
                draft.header_bindings.remove(index);
            }
            (None, None) => {}
        }
    }

    fn sync_body_history_item(
        &mut self,
        rule_id: NodeId,
        condition_id: NodeId,
        preferred_index: Option<usize>,
    ) {
        let projected = self.snapshot.as_ref().and_then(|session| {
            let canonical = session.latest().rule(rule_id)?;
            let position = canonical
                .conditions()
                .body
                .iter()
                .position(|condition| condition.id == condition_id)?;
            let payload = session
                .find_rule(rule_id)?
                .1
                .payload
                .body
                .get(position)?
                .clone();
            Some((position, payload))
        });
        let Some(draft) = self
            .snapshot
            .as_mut()
            .and_then(|session| session.rule_drafts.get_mut(&rule_id))
        else {
            return;
        };
        let draft_position = draft
            .body_bindings
            .iter()
            .position(|binding| *binding == DraftBinding::Existing(condition_id));
        match (projected, draft_position) {
            (Some((_, payload)), Some(index)) => draft.payload.body[index] = payload,
            (Some((canonical_index, payload)), None) => {
                let index = preferred_index
                    .unwrap_or(canonical_index)
                    .min(draft.payload.body.len());
                draft.payload.body.insert(index, payload);
                draft
                    .body_bindings
                    .insert(index, DraftBinding::Existing(condition_id));
            }
            (None, Some(index)) => {
                draft.payload.body.remove(index);
                draft.body_bindings.remove(index);
            }
            (None, None) => {}
        }
    }

    fn sync_history_drafts(&mut self, command: &HistoryEntry) {
        match command {
            HistoryEntry::RuleMatch { rule_id, field, .. } => {
                self.sync_rule_match_draft(*rule_id, *field);
            }
            HistoryEntry::Respond { rule_id, field, .. } => {
                self.sync_respond_draft(*rule_id, *field);
            }
            HistoryEntry::RootSetting { before, .. } => {
                self.sync_root_setting_draft(before.key());
            }
            HistoryEntry::HeaderAdd {
                rule_id,
                current_id,
                ..
            } => self.sync_header_history_item(*rule_id, *current_id, None),
            HistoryEntry::HeaderUpdate { current_id, .. } => {
                if let Some(rule_id) = self.header_parent(*current_id) {
                    self.sync_header_history_item(rule_id, *current_id, None);
                }
            }
            HistoryEntry::HeaderRemove {
                rule_id,
                index,
                current_id,
                ..
            } => self.sync_header_history_item(*rule_id, *current_id, Some(*index)),
            HistoryEntry::HeadersClear { rule_id, entries } => {
                for (index, _, _, current_id) in entries {
                    self.sync_header_history_item(*rule_id, *current_id, Some(*index));
                }
            }
            HistoryEntry::BodyAdd {
                rule_id,
                current_id,
                ..
            } => self.sync_body_history_item(*rule_id, *current_id, None),
            HistoryEntry::BodyUpdate { current_id, .. } => {
                if let Some(rule_id) = self.body_parent(*current_id) {
                    self.sync_body_history_item(rule_id, *current_id, None);
                }
            }
            HistoryEntry::BodyRemove {
                rule_id,
                index,
                current_id,
                ..
            } => self.sync_body_history_item(*rule_id, *current_id, Some(*index)),
            HistoryEntry::BodiesClear { rule_id, entries } => {
                for (index, _, _, current_id) in entries {
                    self.sync_body_history_item(*rule_id, *current_id, Some(*index));
                }
            }
            HistoryEntry::MoveRule { .. }
            | HistoryEntry::RulePrototype { .. }
            | HistoryEntry::TracePrototype { .. }
            | HistoryEntry::AddedSubtree { .. }
            | HistoryEntry::RemovedSubtree { .. } => {}
        }
    }

    fn restore_headers(
        &mut self,
        rule_id: NodeId,
        entries: &mut [(
            usize,
            apimokka_model::SemanticCreationKey,
            apimokka_model::HeaderCondition,
            NodeId,
        )],
    ) -> bool {
        let Some(canonical) = self
            .snapshot
            .as_ref()
            .and_then(|session| session.latest().rule(rule_id))
            .cloned()
        else {
            return false;
        };
        let mut headers = canonical
            .conditions()
            .headers
            .iter()
            .map(|condition| ConditionEdit::Existing {
                id: condition.id,
                condition: condition.condition.clone(),
            })
            .collect::<Vec<_>>();
        for (index, key, condition, _) in entries.iter() {
            headers.insert(
                (*index).min(headers.len()),
                ConditionEdit::Create {
                    key: key.clone(),
                    condition: condition.clone(),
                },
            );
        }
        let Some(mut rule) = self.current_rule_edit(rule_id) else {
            return false;
        };
        rule.headers = CollectionEdit::Replace(headers);
        let Some(outcome) =
            self.apply_workspace_intent(EditIntent::UpdateRule { id: rule_id, rule })
        else {
            return false;
        };
        let mut rebound = std::collections::HashMap::new();
        for (_, key, _, current_id) in entries.iter_mut() {
            let Some(receipt) = outcome.creations.iter().find(|receipt| {
                receipt.key == *key && receipt.kind == WorkspaceNodeKind::HeaderCondition
            }) else {
                return false;
            };
            rebound.insert(*current_id, receipt.new_id);
            *current_id = receipt.new_id;
        }
        self.apply_history_id_map(&rebound);
        true
    }

    fn restore_history_subtree(
        &mut self,
        archive: apimokka_model::ArchivedSubtree,
        current_root: &mut NodeId,
        bindings: &mut [(NodeId, NodeId)],
        prototypes: &[(NodeId, workspace_session::RulePrototype)],
    ) -> bool {
        let former_root = archive.former_root();
        let root_kind = archive
            .nodes()
            .iter()
            .find(|node| node.old_id == former_root)
            .map(|node| node.payload.kind());
        let Some(outcome) = self.apply_workspace_intent(EditIntent::RestoreSubtree { archive })
        else {
            return false;
        };
        let Some(root) = outcome
            .rebound_nodes
            .iter()
            .find(|rebind| rebind.old_id == former_root)
        else {
            return false;
        };
        let rebound_current = bindings
            .iter()
            .filter_map(|(archive_id, current_id)| {
                outcome
                    .rebound_nodes
                    .iter()
                    .find(|rebind| rebind.old_id == *archive_id)
                    .map(|rebind| (*current_id, rebind.new_id))
            })
            .collect::<std::collections::HashMap<_, _>>();
        for (archive_id, current_id) in bindings.iter_mut() {
            if let Some(new_id) = outcome
                .rebound_nodes
                .iter()
                .find(|rebind| rebind.old_id == *archive_id)
                .map(|rebind| rebind.new_id)
            {
                *current_id = new_id;
            }
        }
        *current_root = root.new_id;
        match root_kind {
            Some(WorkspaceNodeKind::Rule) => {
                if let Some(parent) = self.snapshot.as_ref().and_then(|session| {
                    session
                        .find_rule(root.new_id)
                        .map(|(rule_set, _)| rule_set.id)
                }) {
                    self.selection.select_rule(root.new_id, parent);
                }
            }
            Some(WorkspaceNodeKind::RuleSet) => {
                self.selection.select_rule_set(RuleSetId(root.new_id));
            }
            _ => {}
        }
        self.apply_history_id_map(&rebound_current);
        if let Some(session) = self.snapshot.as_mut() {
            for (old_id, prototype) in prototypes {
                if let Some(new_id) = outcome
                    .rebound_nodes
                    .iter()
                    .find(|rebind| rebind.old_id == *old_id)
                    .map(|rebind| rebind.new_id)
                {
                    session
                        .prototype
                        .rule_extras
                        .insert(new_id, prototype.clone());
                }
            }
        }
        true
    }

    fn apply_history_id_map(&mut self, map: &std::collections::HashMap<NodeId, NodeId>) {
        let Some(session) = self.snapshot.as_mut() else {
            return;
        };
        if let Some(rule) = self.selection.rule.and_then(|id| map.get(&id).copied()) {
            self.selection.rule = Some(rule);
        }
        if let Some(set) = self
            .selection
            .rule_set
            .and_then(|id| map.get(&id.0).copied())
        {
            self.selection.rule_set = Some(RuleSetId(set));
        }
        let old_drafts = std::mem::take(&mut session.rule_drafts);
        session.rule_drafts = old_drafts
            .into_iter()
            .map(|(id, mut draft)| {
                for binding in draft
                    .header_bindings
                    .iter_mut()
                    .chain(draft.body_bindings.iter_mut())
                {
                    if let DraftBinding::Existing(id) = binding
                        && let Some(new_id) = map.get(id)
                    {
                        *id = *new_id;
                    }
                }
                (map.get(&id).copied().unwrap_or(id), draft)
            })
            .collect();
        let old_extras = std::mem::take(&mut session.prototype.rule_extras);
        session.prototype.rule_extras = old_extras
            .into_iter()
            .map(|(id, value)| (map.get(&id).copied().unwrap_or(id), value))
            .collect();
        for command in session
            .undo_stack
            .iter_mut()
            .chain(session.redo_stack.iter_mut())
        {
            rebind_command(command, map);
        }
    }

    fn restore_bodies(
        &mut self,
        rule_id: NodeId,
        entries: &mut [(
            usize,
            apimokka_model::SemanticCreationKey,
            apimokka_model::BodyCondition,
            NodeId,
        )],
    ) -> bool {
        let Some(canonical) = self
            .snapshot
            .as_ref()
            .and_then(|session| session.latest().rule(rule_id))
            .cloned()
        else {
            return false;
        };
        let mut body = canonical
            .conditions()
            .body
            .iter()
            .map(|condition| ConditionEdit::Existing {
                id: condition.id,
                condition: condition.condition.clone(),
            })
            .collect::<Vec<_>>();
        for (index, key, condition, _) in entries.iter() {
            body.insert(
                (*index).min(body.len()),
                ConditionEdit::Create {
                    key: key.clone(),
                    condition: condition.clone(),
                },
            );
        }
        let Some(mut rule) = self.current_rule_edit(rule_id) else {
            return false;
        };
        rule.body = CollectionEdit::Replace(body);
        let Some(outcome) =
            self.apply_workspace_intent(EditIntent::UpdateRule { id: rule_id, rule })
        else {
            return false;
        };
        let mut rebound = std::collections::HashMap::new();
        for (_, key, _, current_id) in entries.iter_mut() {
            let Some(receipt) = outcome.creations.iter().find(|receipt| {
                receipt.key == *key && receipt.kind == WorkspaceNodeKind::BodyCondition
            }) else {
                return false;
            };
            rebound.insert(*current_id, receipt.new_id);
            *current_id = receipt.new_id;
        }
        self.apply_history_id_map(&rebound);
        true
    }

    fn archive_rule(&mut self, rule_id: NodeId) -> Option<apimokka_model::ArchivedSubtree> {
        let (parent, insertion_index, canonical) = {
            let session = self.snapshot.as_ref()?;
            let (set, _) = session.find_rule(rule_id)?;
            let index = set.rules.iter().position(|rule| rule.id == rule_id)?;
            (set.id, index, session.latest().rule(rule_id)?.clone())
        };
        let session = self.snapshot.as_mut()?;
        let mut nodes = vec![apimokka_model::ArchivedNode {
            old_id: rule_id,
            parent: None,
            key: session.creation_key("archive-rule"),
            payload: apimokka_model::ArchivedNodePayload::Rule(RuleEditPayload {
                rule_match: canonical.rule_match().clone(),
                headers: CollectionEdit::Preserve,
                body: CollectionEdit::Preserve,
                respond: canonical.respond().clone(),
            }),
        }];
        nodes.extend(canonical.conditions().headers.iter().map(|condition| {
            apimokka_model::ArchivedNode {
                old_id: condition.id,
                parent: Some(rule_id),
                key: session.creation_key("archive-header"),
                payload: apimokka_model::ArchivedNodePayload::HeaderCondition(
                    condition.condition.clone(),
                ),
            }
        }));
        nodes.extend(canonical.conditions().body.iter().map(|condition| {
            apimokka_model::ArchivedNode {
                old_id: condition.id,
                parent: Some(rule_id),
                key: session.creation_key("archive-body"),
                payload: apimokka_model::ArchivedNodePayload::BodyCondition(
                    condition.condition.clone(),
                ),
            }
        }));
        apimokka_model::ArchivedSubtree::new(
            rule_id,
            apimokka_model::RestorePlacement::Rule {
                parent,
                insertion_index,
            },
            nodes,
        )
        .ok()
    }

    fn subtree_prototypes(
        &self,
        archive: &apimokka_model::ArchivedSubtree,
    ) -> Vec<(NodeId, workspace_session::RulePrototype)> {
        let Some(session) = self.snapshot.as_ref() else {
            return Vec::new();
        };
        archive
            .nodes()
            .iter()
            .filter(|node| node.payload.kind() == WorkspaceNodeKind::Rule)
            .filter_map(|node| {
                session
                    .prototype
                    .rule_extras
                    .get(&node.old_id)
                    .cloned()
                    .map(|prototype| (node.old_id, prototype))
            })
            .collect()
    }

    fn archive_rule_set(&mut self, set_id: RuleSetId) -> Option<apimokka_model::ArchivedSubtree> {
        let (insertion_index, path, rules) = {
            let session = self.snapshot.as_ref()?;
            let index = session.rule_sets.iter().position(|set| set.id == set_id)?;
            let set = &session.rule_sets[index];
            let rules = set
                .rules
                .iter()
                .map(|rule| session.latest().rule(rule.id).cloned())
                .collect::<Option<Vec<_>>>()?;
            (index, parse_rule_set_path(&set.file.path).ok()?, rules)
        };
        let session = self.snapshot.as_mut()?;
        let mut nodes = vec![apimokka_model::ArchivedNode {
            old_id: set_id.0,
            parent: None,
            key: session.creation_key("archive-rule-set"),
            payload: apimokka_model::ArchivedNodePayload::RuleSet { path },
        }];
        for canonical in rules {
            let rule_id = canonical.rule_id();
            nodes.push(apimokka_model::ArchivedNode {
                old_id: rule_id,
                parent: Some(set_id.0),
                key: session.creation_key("archive-rule"),
                payload: apimokka_model::ArchivedNodePayload::Rule(RuleEditPayload {
                    rule_match: canonical.rule_match().clone(),
                    headers: CollectionEdit::Preserve,
                    body: CollectionEdit::Preserve,
                    respond: canonical.respond().clone(),
                }),
            });
            nodes.extend(canonical.conditions().headers.iter().map(|condition| {
                apimokka_model::ArchivedNode {
                    old_id: condition.id,
                    parent: Some(rule_id),
                    key: session.creation_key("archive-header"),
                    payload: apimokka_model::ArchivedNodePayload::HeaderCondition(
                        condition.condition.clone(),
                    ),
                }
            }));
            nodes.extend(canonical.conditions().body.iter().map(|condition| {
                apimokka_model::ArchivedNode {
                    old_id: condition.id,
                    parent: Some(rule_id),
                    key: session.creation_key("archive-body"),
                    payload: apimokka_model::ArchivedNodePayload::BodyCondition(
                        condition.condition.clone(),
                    ),
                }
            }));
        }
        apimokka_model::ArchivedSubtree::new(
            set_id.0,
            apimokka_model::RestorePlacement::RuleSetRoot { insertion_index },
            nodes,
        )
        .ok()
    }

    /// Global save runs the workspace port first, then commits fallback
    /// drafts in canonical byte order only after workspace success.
    fn save_workspace_and_fallbacks(&mut self) -> Option<GlobalSaveCompletion> {
        self.save_workspace_and_fallbacks_with(|_, _, _| Ok(()))
    }

    fn save_workspace_and_fallbacks_with<F>(
        &mut self,
        mut write_fallback: F,
    ) -> Option<GlobalSaveCompletion>
    where
        F: FnMut(&str, &str, &str) -> Result<(), FallbackSaveError>,
    {
        if self
            .snapshot
            .as_ref()
            .is_some_and(|session| session.faulted)
        {
            self.enter_session_fault_if_any();
            return None;
        }
        self.notice = None;
        let mut dirty_fallback_keys: Vec<String> = self
            .fallback_drafts
            .keys()
            .filter(|path| self.is_fallback_dirty(path))
            .cloned()
            .collect();
        dirty_fallback_keys.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        let selection_target = self.capture_selection_target();
        let session = self.snapshot.as_mut()?;
        let result = session.save();
        let (workspace, fallback) = match result {
            workspace_session::SessionSaveResult::Saved(workspace) => {
                self.reconcile_selection(selection_target);
                if self.transient_problem_kind == Some(TransientProblemKind::Save) {
                    self.clear_transient_problem();
                }
                let mut written_keys = Vec::new();
                let mut fallback_failure = None;
                for (index, key) in dirty_fallback_keys.iter().enumerate() {
                    let content = self
                        .fallback_drafts
                        .get(key)
                        .map(|draft| draft.text())
                        .unwrap_or_default();
                    let status = self
                        .fallback_status_draft
                        .get(key)
                        .cloned()
                        .or_else(|| self.fallback_status_saved.get(key).cloned())
                        .unwrap_or_else(|| "200 OK".into());
                    match write_fallback(key, &content, &status) {
                        Ok(()) => {
                            self.commit_fallback_values(key, content, status);
                            written_keys.push(key.clone());
                        }
                        Err(cause) => {
                            fallback_failure = Some((index, cause));
                            break;
                        }
                    }
                }
                let fallback = if let Some((index, cause)) = fallback_failure {
                    FallbackSaveReport::Failed {
                        written_keys,
                        failure: FallbackSaveFailure {
                            key: dirty_fallback_keys[index].clone(),
                            cause,
                        },
                        remaining_keys: dirty_fallback_keys[index..].to_vec(),
                    }
                } else {
                    FallbackSaveReport::Completed { written_keys }
                };
                (workspace, fallback)
            }
            workspace_session::SessionSaveResult::SaveFailure(workspace) => {
                self.reconcile_selection(selection_target);
                (
                    workspace,
                    FallbackSaveReport::NotEntered {
                        reason: FallbackSkipReason::WorkspaceFailed,
                        remaining_keys: dirty_fallback_keys,
                    },
                )
            }
            workspace_session::SessionSaveResult::ContractFault(workspace) => {
                self.reconcile_selection(selection_target);
                self.enter_session_fault_if_any();
                (
                    workspace,
                    FallbackSaveReport::NotEntered {
                        reason: FallbackSkipReason::WorkspaceContractFault,
                        remaining_keys: dirty_fallback_keys,
                    },
                )
            }
            workspace_session::SessionSaveResult::AlreadyFaulted => {
                self.enter_session_fault_if_any();
                return None;
            }
        };
        let report = GlobalSaveReport {
            workspace,
            fallback,
        };
        let completion = report.completion();
        let integrity_fault = matches!(
            &report.workspace.integrity,
            SaveIntegrity::ContractFault { .. }
        );
        self.last_save_report = Some(report);
        if completion != GlobalSaveCompletion::Complete {
            self.drawer = Some(DrawerMode::SaveDiff);
            if integrity_fault {
                self.present_save_integrity_problem();
            } else {
                self.present_global_save_problem(completion);
            }
        }
        self.recompute_dirty();
        Some(completion)
    }

    fn present_save_integrity_problem(&mut self) {
        let SaveIntegrity::ContractFault {
            reason,
            progress_trust,
        } = &self
            .last_save_report
            .as_ref()
            .expect("save integrity problem requires a stored report")
            .workspace
            .integrity
        else {
            return;
        };
        let recovery = match progress_trust {
            ProgressTrust::Verified => {
                "The verified saved prefix was retained, but the workspace result violated the save contract. Reload the workspace before editing or retrying."
            }
            ProgressTrust::Unverified => {
                "Workspace commit status could not be verified because the adapter result violated the save contract. Reload the workspace before editing or retrying."
            }
        };
        self.transient_problem_kind = Some(TransientProblemKind::PostCommitContract);
        self.transient_problem_operation = None;
        self.last_problem = Some(
            apimokka_model::FriendlyProblem::new("Workspace reload required", recovery, None)
                .with_technical(reason.clone()),
        );
    }

    fn present_global_save_problem(&mut self, completion: GlobalSaveCompletion) {
        let (summary, recovery, technical) = match self
            .last_save_report
            .as_ref()
            .expect("save problem requires a stored report")
        {
            GlobalSaveReport {
                workspace:
                    WorkspaceSaveReport {
                        progress: WorkspaceSaveProgress::Failed { cause, .. },
                        ..
                    },
                ..
            } => (
                "Workspace save failed",
                "Review the last save attempt. The verified prefix was retained; retry the remaining scopes.",
                cause.detail().to_owned(),
            ),
            GlobalSaveReport {
                fallback: FallbackSaveReport::Failed { failure, .. },
                ..
            } => (
                "Fallback save failed",
                "Review the last save attempt. Successfully written scopes were retained; retry the remaining fallback files.",
                failure.cause.detail().to_owned(),
            ),
            _ => (
                "Save did not complete",
                "Review the last save attempt before retrying.",
                format!("global save completion: {completion:?}"),
            ),
        };
        self.transient_problem_kind = Some(TransientProblemKind::Save);
        self.transient_problem_operation = None;
        self.last_problem = Some(
            apimokka_model::FriendlyProblem::new(summary, recovery, None).with_technical(technical),
        );
    }

    // ── MK-038 fallback lifecycle helpers ─────────────────────────────────

    /// Normalised text comparison: trailing newline differences are not edits.
    fn normalize(s: &str) -> &str {
        s.strip_suffix('\n').unwrap_or(s)
    }

    /// dirty(path) := draft != saved (content or status code).
    pub fn is_fallback_dirty(&self, path: &str) -> bool {
        let content_dirty = match (
            self.fallback_drafts.get(path),
            self.fallback_saved.get(path),
        ) {
            (Some(draft), Some(saved)) => Self::normalize(&draft.text()) != Self::normalize(saved),
            (Some(draft), None) => !draft.text().trim().is_empty(),
            _ => false,
        };
        let status_dirty = match (
            self.fallback_status_draft.get(path),
            self.fallback_status_saved.get(path),
        ) {
            (Some(d), Some(s)) => d != s,
            (Some(d), None) => d != "200 OK",
            _ => false,
        };
        content_dirty || status_dirty
    }

    /// json_valid(path) := the draft parses as JSON. Warns, never blocks.
    pub fn fallback_json_valid(&self, path: &str) -> bool {
        self.fallback_drafts
            .get(path)
            .map(|c| serde_json::from_str::<serde_json::Value>(&c.text()).is_ok())
            .unwrap_or(true)
    }

    /// Save: saved ← draft (Dirty → Clean). Takes effect on next request;
    /// no server reload required for fallback files.
    fn commit_fallback_draft(&mut self, path: &str) {
        let content = self
            .fallback_drafts
            .get(path)
            .map(|draft| draft.text())
            .unwrap_or_default();
        let status = self
            .fallback_status_draft
            .get(path)
            .cloned()
            .or_else(|| self.fallback_status_saved.get(path).cloned())
            .unwrap_or_else(|| "200 OK".into());
        self.commit_fallback_values(path, content, status);
    }

    fn commit_fallback_values(&mut self, path: &str, content: String, status: String) {
        self.fallback_saved.insert(path.to_string(), content);
        self.fallback_status_saved.insert(path.to_string(), status);
    }

    /// Recompute the top-bar dirty counter: dirty rule files + dirty
    /// fallback files. Derived, never event-counted.
    fn recompute_dirty(&mut self) {
        let rule_dirty = self
            .snapshot
            .as_ref()
            .map(|s| s.latest().dirty_files().len())
            .unwrap_or(0);
        let fallback_dirty = self
            .fallback_drafts
            .keys()
            .filter(|p| self.is_fallback_dirty(p))
            .count();
        self.dirty_count = rule_dirty + fallback_dirty;
    }

    /// Reset fallback drafts to their saved baselines. Canonical workspace
    /// edits stay port-owned and are not mutated through this legacy drawer.
    fn discard_all_changes(&mut self) {
        let dirty_paths: Vec<String> = self
            .fallback_drafts
            .keys()
            .filter(|p| self.is_fallback_dirty(p))
            .cloned()
            .collect();
        for path in dirty_paths {
            let saved = self.fallback_saved.get(&path).cloned().unwrap_or_default();
            self.fallback_drafts.insert(
                path.clone(),
                iced::widget::text_editor::Content::with_text(&saved),
            );
            let status = self
                .fallback_status_saved
                .get(&path)
                .cloned()
                .unwrap_or_else(|| "200 OK".into());
            self.fallback_status_draft.insert(path, status);
        }
        self.recompute_dirty();
    }

    fn trigger_reload(&mut self) {
        self.recompute_dirty();
    }

    fn trigger_restart(&mut self) {
        self.recompute_dirty();
    }

    fn create_workspace_from_wizard(&mut self) {
        let name = if self.wizard.name.trim().is_empty() {
            "my-mock".to_string()
        } else {
            self.wizard.name.trim().to_string()
        };
        let host = if self.wizard.host.trim().is_empty() {
            "127.0.0.1".to_string()
        } else {
            self.wizard.host.trim().to_string()
        };
        let port = self.wizard.port.trim().parse::<u16>().unwrap_or(8080);
        let tls = self.wizard.tls;
        let seed = match self.wizard.starter {
            WizardStarter::Empty => mock::blank_workspace(&name, &host, port, tls),
            WizardStarter::Minimal => mock::minimal_workspace(&name, &host, port, tls),
            WizardStarter::ShopApi => {
                let mut workspace = mock::shop_api_canonical_seed();
                workspace.meta.name = name.clone();
                workspace.meta.path = format!("~/{name}/apimock.toml");
                workspace.root_settings.listener_ip = host;
                workspace.root_settings.listener_port = port;
                workspace.root_settings.tls_enabled = tls;
                workspace
            }
        };
        if !self.install_workspace(seed) {
            return;
        }
        self.view = AppView::Workspace;
        self.tab = WorkspaceTab::Routes;
        self.server_state = ServerState::Stopped;
        self.notice = Some(format!(
            "Workspace \"{name}\" created. {}",
            match self.wizard.starter {
                WizardStarter::Empty => "Add a rule set to get started.",
                WizardStarter::Minimal => "A starter GET /health rule is ready.",
                WizardStarter::ShopApi => "Shop API example rules are loaded.",
            }
        ));
    }

    fn requires_workspace_confirmation(&self) -> bool {
        self.snapshot
            .as_ref()
            .is_some_and(|session| self.dirty_count > 0 || session.has_pending_drafts())
    }

    fn open_workspace(&mut self, _name: String) {
        if self.install_workspace(mock::shop_api_canonical_seed()) {
            self.view = AppView::Workspace;
            self.tab = WorkspaceTab::Routes;
        }
    }

    fn leave_workspace(&mut self) {
        self.view = AppView::Welcome;
        self.snapshot = None;
        self.runtime_in_flight = None;
        self.last_save_report = None;
        self.selection = RouteSelection::default();
        self.reset_fallback_state(
            std::collections::HashMap::new(),
            std::collections::HashMap::new(),
        );
        self.dirty_count = 0;
        self.clear_transient_problem();
    }

    fn reset_fallback_state(
        &mut self,
        saved: std::collections::HashMap<String, String>,
        status_saved: std::collections::HashMap<String, String>,
    ) {
        self.fallback_saved = saved;
        self.fallback_drafts.clear();
        self.fallback_status_saved = status_saved;
        self.fallback_status_draft.clear();
        self.fallback_section_open = false;
    }

    pub(crate) fn install_workspace(&mut self, seed: apimokka_model::WorkspaceSnapshot) -> bool {
        let generation = SessionGeneration(self.next_session_generation);
        let session = match WorkspaceSession::new_with_generation(seed, generation) {
            Ok(session) => session,
            Err(error) => {
                self.transient_problem_kind = Some(TransientProblemKind::Admission);
                self.transient_problem_operation = None;
                self.last_problem = Some(
                    apimokka_model::FriendlyProblem::new(
                        "The workspace could not be opened",
                        "The workspace contains configuration that cannot be admitted. Correct it and try again.",
                        None,
                    )
                    .with_technical(error.to_string()),
                );
                return false;
            }
        };
        self.next_session_generation = self
            .next_session_generation
            .checked_add(1)
            .expect("session generation overflow");
        let content_catalog = mock_fallback_content();
        let fallback_saved = session
            .fallback_files
            .iter()
            .map(|file| {
                (
                    file.path.clone(),
                    content_catalog.get(&file.path).cloned().unwrap_or_default(),
                )
            })
            .collect();
        let fallback_status_saved = session
            .fallback_files
            .iter()
            .map(|file| (file.path.clone(), "200 OK".to_string()))
            .collect();
        self.snapshot = Some(session);
        self.last_save_report = None;
        self.reset_fallback_state(fallback_saved, fallback_status_saved);
        self.selection = RouteSelection::default();
        self.rule_set_open = None;
        self.runtime_in_flight = None;
        self.server_state = ServerState::Running;
        self.clear_transient_problem();
        self.recompute_dirty();
        true
    }

    pub(crate) fn selected_rule(&self) -> Option<&apimokka_model::snapshot::RuleView> {
        let id = self.selection.rule?;
        self.snapshot.as_ref()?.find_rule(id).map(|(_, r)| r)
    }

    pub(crate) fn selected_rule_payload(&self) -> Option<&apimokka_model::RulePayload> {
        let id = self.selection.rule?;
        if let Some(draft) = self.snapshot.as_ref()?.rule_draft(id) {
            return Some(&draft.payload);
        }
        self.selected_rule().map(|rule| &rule.payload)
    }

    pub(crate) fn undo_stack(&self) -> &[HistoryEntry] {
        self.snapshot
            .as_ref()
            .map(|session| session.undo_stack.as_slice())
            .unwrap_or(&[])
    }

    #[cfg(test)]
    pub(crate) fn redo_stack(&self) -> &[HistoryEntry] {
        self.snapshot
            .as_ref()
            .map(|session| session.redo_stack.as_slice())
            .unwrap_or(&[])
    }
}

fn rebind_command(command: &mut HistoryEntry, map: &std::collections::HashMap<NodeId, NodeId>) {
    let rebind = |id: &mut NodeId| {
        if let Some(new_id) = map.get(id) {
            *id = *new_id;
        }
    };
    match command {
        HistoryEntry::MoveRule { rule_id, .. }
        | HistoryEntry::RuleMatch { rule_id, .. }
        | HistoryEntry::Respond { rule_id, .. }
        | HistoryEntry::RulePrototype { rule_id, .. } => rebind(rule_id),
        HistoryEntry::HeaderUpdate { current_id, .. }
        | HistoryEntry::BodyUpdate { current_id, .. } => rebind(current_id),
        HistoryEntry::HeaderAdd {
            rule_id,
            current_id,
            ..
        }
        | HistoryEntry::HeaderRemove {
            rule_id,
            current_id,
            ..
        }
        | HistoryEntry::BodyAdd {
            rule_id,
            current_id,
            ..
        }
        | HistoryEntry::BodyRemove {
            rule_id,
            current_id,
            ..
        } => {
            rebind(rule_id);
            rebind(current_id);
        }
        HistoryEntry::HeadersClear { rule_id, entries } => {
            rebind(rule_id);
            for entry in entries {
                rebind(&mut entry.3);
            }
        }
        HistoryEntry::BodiesClear { rule_id, entries } => {
            rebind(rule_id);
            for entry in entries {
                rebind(&mut entry.3);
            }
        }
        HistoryEntry::AddedSubtree {
            archive,
            current_root,
            bindings,
            ..
        }
        | HistoryEntry::RemovedSubtree {
            archive,
            current_root,
            bindings,
            ..
        } => {
            rebind(current_root);
            for (_, current_id) in bindings {
                rebind(current_id);
            }
            if let apimokka_model::RestorePlacement::Rule {
                parent,
                insertion_index,
            } = archive.placement()
                && let Some(new_parent) = map.get(&parent.0)
                && let Ok(rebuilt) = apimokka_model::ArchivedSubtree::new(
                    archive.former_root(),
                    apimokka_model::RestorePlacement::Rule {
                        parent: RuleSetId(*new_parent),
                        insertion_index,
                    },
                    archive.nodes().to_vec(),
                )
            {
                *archive = rebuilt;
            }
        }
        HistoryEntry::RootSetting { .. } | HistoryEntry::TracePrototype { .. } => {}
    }
}

/// Task 014 §3/§4 — the selected-row-plus-arrow-keys idiom shared by the
/// command palette and the mode picker. Clamps a selection index to a list
/// that may have shrunk (typing narrows the palette's filter): never selects
/// out of range, and an empty list always clears the selection.
fn clamp_selection(selected: Option<usize>, list_len: usize) -> Option<usize> {
    if list_len == 0 {
        return None;
    }
    selected.map(|i| i.min(list_len - 1))
}

/// Moves a selection index up (`delta < 0`) or down (`delta > 0`) within a
/// list of `list_len` rows. From no selection, either direction selects the
/// first row. Saturates at both ends rather than wrapping — MK-033 does not
/// specify wraparound, and this task does not add it to the mode picker
/// either.
fn move_selection(selected: Option<usize>, list_len: usize, delta: isize) -> Option<usize> {
    if list_len == 0 {
        return None;
    }
    let next = match selected {
        None => 0,
        Some(i) => (i as isize + delta).clamp(0, list_len as isize - 1) as usize,
    };
    Some(next)
}

fn is_workspace_mutation(message: &Message) -> bool {
    matches!(
        message,
        Message::UndoLast
            | Message::Undo
            | Message::Redo
            | Message::Save
            | Message::SaveAll
            | Message::DiscardChanges
            | Message::AddRuleSet
            | Message::AddRule(_)
            | Message::MoveRuleUp(_)
            | Message::MoveRuleDown(_)
            | Message::DeleteRuleSet(_)
            | Message::DeleteRule(_)
            | Message::DuplicateRule(_)
            | Message::RuleSetUrlPath(_)
            | Message::RuleSetUrlPathOp(_)
            | Message::RuleSetUrlPathEnabled(_)
            | Message::RuleSetMethod(_)
            | Message::HeaderAdd
            | Message::HeaderRemove(_)
            | Message::HeaderSetName { .. }
            | Message::HeaderSetOp { .. }
            | Message::HeaderSetValue { .. }
            | Message::HeaderClearAll
            | Message::BodyAdd
            | Message::BodyRemove(_)
            | Message::BodySetPath { .. }
            | Message::BodySetOp { .. }
            | Message::BodySetValue { .. }
            | Message::BodyClearAll
            | Message::RespondSetMode(_)
            | Message::RespondSetText(_)
            | Message::RespondSetFilePath(_)
            | Message::RespondSetStatus(_)
            | Message::RespondSetDelay(_)
            | Message::RuleWeightChanged(_)
            | Message::RulePriorityChanged(_)
            | Message::RuleSetWeight(_)
            | Message::RuleSetPriority(_)
            | Message::RuleSetSetStrategy(_)
            | Message::SettingsSetHost(_)
            | Message::SettingsSetPort(_)
            | Message::SettingsSetTls(_)
            | Message::SettingsSetLogLevel(_)
            | Message::SettingsSetStrategy(_)
            | Message::SettingsSetTraceEnabled(_)
    )
}

fn subtree_bindings(archive: &apimokka_model::ArchivedSubtree) -> Vec<(NodeId, NodeId)> {
    archive
        .nodes()
        .iter()
        .map(|node| (node.old_id, node.old_id))
        .collect()
}

#[cfg(test)]
#[path = "app/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "app/density.rs"]
mod tests_mk041;

#[cfg(test)]
#[path = "app/trace.rs"]
mod tests_mk042;

#[cfg(test)]
#[path = "app/strategy.rs"]
mod tests_mk043;

#[cfg(test)]
#[path = "app/drawers.rs"]
mod tests_mk044;

#[cfg(test)]
#[path = "app/history.rs"]
mod tests_mk045;

#[cfg(test)]
#[path = "app/workspace_session_tests.rs"]
mod workspace_session_tests;

#[cfg(test)]
#[path = "app/runtime_tests.rs"]
mod runtime_tests;

#[cfg(test)]
#[path = "app/global_save_tests.rs"]
mod global_save_tests;

#[cfg(test)]
#[path = "app/navigation.rs"]
mod tests_mk046;

#[cfg(test)]
#[path = "app/workspace_creation.rs"]
mod tests_mk047;

#[cfg(test)]
#[path = "app/rule_set_creation.rs"]
mod tests_mk048;

#[cfg(test)]
#[path = "app/rule_duplication.rs"]
mod tests_mk049;

#[cfg(test)]
#[path = "app/themes.rs"]
mod tests_mk050;

#[cfg(test)]
#[path = "app/palette_keyboard.rs"]
mod tests_mk033;

// ── App view / subscription (not in the impl block above) ─────────────────────

impl App {
    // ─────────────────────────────────────────────────────────────────────────
    // View and subscription
    // ─────────────────────────────────────────────────────────────────────────

    pub fn view(&self) -> Element<'_, Message> {
        // MK-046: If no audience mode has been chosen yet (first launch or
        // session), render the mode picker as a full-screen. This ensures the
        // user always sees the picker before any workspace content.
        if self.audience_mode.is_none() {
            return screens::mode_picker::view(self);
        }
        match self.view {
            AppView::Welcome => screens::welcome::view(self),
            AppView::Dashboard => screens::dashboard::view(self),
            AppView::Wizard => screens::wizard::view(self),
            AppView::Workspace => shell::view::view(self),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        use iced::keyboard::key::Named;

        // Deliberately `event::listen_with`, not `keyboard::listen()` —
        // and deliberately ignoring the `status` it reports.
        //
        // Found live, driving the app with `wtype` for task 014's own
        // acceptance evidence, not by reading the code: `keyboard::listen()`
        // only ever sees events no widget has already captured
        // (`iced_futures::keyboard::listen` filters to
        // `event::Status::Ignored`). `iced_widget::text_input` captures
        // `Escape` unconditionally whenever it is focused, as its own
        // default unfocus behaviour (`text_input.rs`, `shell.capture_event()`
        // on `Named::Escape`) — regardless of whether this application sets
        // `.on_submit`. Once MK-033's auto-focus (this task, §1) put a
        // stable focus target inside the palette, that made `Esc` need two
        // presses to actually close it: one press for `text_input`'s own
        // capture, a second to reach `EscapePressed` here. A global
        // "close whatever is open" shortcut must not depend on which widget
        // happens to hold focus, so this listens to every keyboard event,
        // captured or not.
        //
        // Arrow keys and `Enter` are not affected by this specific capture
        // — `text_input` only captures `Enter` when `.on_submit` is set,
        // which the palette's search field never does — but are handled the
        // same uncaptured-or-not way here, for one genuinely global keyboard
        // path rather than two subscriptions with different semantics.
        iced::event::listen_with(|event, _status, _window| {
            let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) =
                event
            else {
                return None;
            };
            if key == iced::keyboard::Key::Named(Named::Escape) {
                return Some(Message::EscapePressed);
            }
            // Task 014 §3/§4: dispatched unconditionally — `update` decides
            // whether the palette or the mode picker (or neither) is
            // listening.
            if key == iced::keyboard::Key::Named(Named::ArrowUp) {
                return Some(Message::ArrowUp);
            }
            if key == iced::keyboard::Key::Named(Named::ArrowDown) {
                return Some(Message::ArrowDown);
            }
            if key == iced::keyboard::Key::Named(Named::Enter) {
                return Some(Message::EnterPressed);
            }
            // Dev-team handoff 002: key/modifier matching lives in the
            // accelerator table, not inline arms, so it cannot drift from
            // the palette's displayed shortcuts.
            crate::accelerator::match_pressed(&key, modifiers)
        })
    }
}

// ── Mock fallback content (seeded for the mockup) ─────────────────────────────

fn mock_fallback_content() -> std::collections::HashMap<String, String> {
    let mut m = std::collections::HashMap::new();
    m.insert(
        "responses/health.json".into(),
        r#"{
  "status": "ok",
  "uptime": 99.97,
  "version": "1.4.2"
}"#
        .into(),
    );
    m.insert(
        "responses/users.json".into(),
        r#"[
  {
    "id": 1,
    "name": "Alice Kato",
    "email": "alice@example.com",
    "role": "admin"
  },
  {
    "id": 2,
    "name": "Bob Tanaka",
    "email": "bob@example.com",
    "role": "member"
  }
]"#
        .into(),
    );
    m.insert(
        "responses/order-created.json".into(),
        r#"{
  "id": "ord-001",
  "status": "created",
  "total": 99.50,
  "currency": "USD",
  "items": [
    { "sku": "widget-x", "qty": 2, "price": 49.75 }
  ]
}"#
        .into(),
    );
    m
}
