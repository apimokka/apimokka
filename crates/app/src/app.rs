//! Central app state and update (MK-021, MK-035).

use apimokka_i18n::{Key, Locale};
use apimokka_model::{
    BodyConditionPayload, BodyOp, HeaderConditionPayload, HeaderOp, mock,
    snapshot::WorkspaceSnapshot,
};
use iced::{Element, Subscription, Theme};

use crate::match_test::{TestRequest, TestRuleResult};
use crate::message::{ConfirmAction, Message};
use crate::screens;
use crate::selection::{DrawerMode, RouteSelection, WorkspaceTab};
use crate::shell;
use crate::shell::top_bar::ServerState;

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

    /// The iced Theme for this choice. Standard Light/Dark use iced's native
    /// themes for visual continuity; the high-contrast modes build a custom
    /// iced palette from the snora high-contrast tokens (MK-050).
    pub fn iced(self) -> Theme {
        use snora::design::style::color::to_iced_color;
        match self {
            Self::Light => Theme::Light,
            Self::Dark => Theme::Dark,
            Self::HighContrastLight | Self::HighContrastDark => {
                let t = self.tokens();
                let pal = iced::theme::Palette {
                    background: to_iced_color(t.palette.background),
                    text: to_iced_color(t.palette.text_primary),
                    primary: to_iced_color(t.palette.accent),
                    success: to_iced_color(t.palette.success),
                    warning: to_iced_color(t.palette.warning),
                    danger: to_iced_color(t.palette.danger),
                };
                let name = match self {
                    Self::HighContrastLight => "apimokka-hc-light",
                    _ => "apimokka-hc-dark",
                };
                Theme::custom(name.to_string(), pal)
            }
        }
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

pub const UNDO_STACK_DEPTH: usize = 25;

/// A reversible operation. Pushed to undo_stack before each edit; applied
/// in reverse on undo; pushed to redo_stack so redo re-applies it.
#[derive(Debug, Clone)]
pub enum UndoCommand {
    /// A rule was deleted; undo re-inserts it at the same position.
    DeleteRule {
        rule_set: apimokka_model::RuleSetId,
        index: usize,
        #[allow(dead_code)]
        rule_id: apimokka_model::NodeId, // for forward redo (accessible via rule.id)
        rule: apimokka_model::snapshot::RuleView,
    },
    /// A rule was added; undo removes it.
    AddRule {
        rule_set: apimokka_model::RuleSetId,
        rule_id: apimokka_model::NodeId,
    },
    /// A rule was moved; undo moves it back to `from_index`.
    MoveRule {
        rule_set: apimokka_model::RuleSetId,
        rule_id: apimokka_model::NodeId,
        from_index: usize,
    },
    /// The URL path field was edited; undo restores `old_value`.
    EditUrlPath {
        rule_id: apimokka_model::NodeId,
        old_value: String, // value before the edit
        new_value: String, // value after the edit (for redo)
    },
}

impl UndoCommand {
    pub fn banner_key(&self) -> apimokka_i18n::Key {
        match self {
            Self::DeleteRule { .. } => apimokka_i18n::Key::UndoRuleDeleted,
            Self::AddRule { .. } => apimokka_i18n::Key::UndoRuleAdded,
            Self::MoveRule { .. } => apimokka_i18n::Key::UndoRuleMoved,
            Self::EditUrlPath { .. } => apimokka_i18n::Key::UndoUrlPathEdited,
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
    pub snapshot: Option<WorkspaceSnapshot>,
    pub selection: RouteSelection,
    pub server_state: ServerState,

    pub dirty_count: usize,
    pub save_pending_restart: bool,

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
    pub show_problem_details: bool,
    pub audience_mode: Option<apimokka_model::AudienceMode>,
    // ── MK-041 layout density toggles ─────────────────────────────────────
    pub rule_when_more: bool,
    pub settings_advanced_more: bool,
    pub rule_set_config_more: bool,
    // ── MK-045: undo / redo stacks ────────────────────────────────────────
    /// Undoable commands, most-recent last. Capped at UNDO_STACK_DEPTH.
    pub undo_stack: Vec<UndoCommand>,
    /// Redoable commands. Cleared on any new edit.
    pub redo_stack: Vec<UndoCommand>,
    /// Transient success / info notice.
    pub notice: Option<String>,
}

impl App {
    pub fn new() -> (Self, iced::Task<Message>) {
        // MK-046: no snapshot on first launch — Welcome screen shows first.
        let snapshot: Option<apimokka_model::snapshot::WorkspaceSnapshot> = None;
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
            dirty_count: 0,
            save_pending_restart: false,
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
            fallback_saved: mock_fallback_content(),
            fallback_drafts: std::collections::HashMap::new(),
            fallback_status_saved: std::collections::HashMap::new(),
            fallback_status_draft: std::collections::HashMap::new(),
            last_problem: None,
            show_problem_details: false,
            audience_mode: None, // None → first-run picker shown
            rule_when_more: false,
            settings_advanced_more: false,
            rule_set_config_more: false,
            notice: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };
        (app, iced::Task::none())
    }

    pub fn title(&self) -> String {
        match &self.snapshot {
            Some(s) => format!("{} — apimokka", s.meta.name),
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
                self.last_problem = None;
            }
            Message::ProblemAction => {
                if self.last_problem.is_some() {
                    self.tab = crate::selection::WorkspaceTab::Settings;
                    self.last_problem = None;
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
                if let Some(snap) = &mut self.snapshot {
                    snap.root_settings.strategy = strategy;
                }
            }
            Message::RuleWeightChanged(s) => {
                if let Some(id) = self.selection.rule {
                    if let Some(snap) = &mut self.snapshot {
                        for rs in &mut snap.rule_sets {
                            if let Some(rule) = rs.rules.iter_mut().find(|r| r.id == id) {
                                rule.payload.weight = s.parse::<u32>().ok();
                                rs.file.dirty = true;
                                break;
                            }
                        }
                    }
                }
                self.auto_save_rules();
            }
            Message::RulePriorityChanged(s) => {
                if let Some(id) = self.selection.rule {
                    if let Some(snap) = &mut self.snapshot {
                        for rs in &mut snap.rule_sets {
                            if let Some(rule) = rs.rules.iter_mut().find(|r| r.id == id) {
                                rule.payload.priority = s.parse::<i32>().ok();
                                rs.file.dirty = true;
                                break;
                            }
                        }
                    }
                }
                self.auto_save_rules();
            }
            Message::ToggleSettingsAdvancedMore => {
                self.settings_advanced_more = !self.settings_advanced_more;
            }

            // Navigation
            Message::GoWelcome => {
                self.view = AppView::Welcome;
                self.snapshot = None;
            }
            Message::GoDashboard => {
                self.view = AppView::Dashboard;
            }
            Message::GoWizard => {
                self.wizard = WizardState::default();
                self.view = AppView::Wizard;
            }
            Message::OpenWorkspace(name) => {
                let _ = name;
                self.workspace_menu_open = false;
                self.snapshot = Some(mock::shop_api_mock());
                if let Some(s) = &self.snapshot {
                    self.selection.rule_set = s.rule_sets.first().map(|rs| rs.id);
                    self.selection.rule = s
                        .rule_sets
                        .first()
                        .and_then(|rs| rs.rules.first())
                        .map(|r| r.id);
                }
                self.view = AppView::Workspace;
                self.tab = WorkspaceTab::Routes;
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
                self.command_palette.open = !self.command_palette.open;
                self.command_palette.query = String::new();
            }
            Message::PaletteQuery(q) => {
                self.command_palette.query = q;
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
                self.server_state = match self.server_state {
                    ServerState::Stopped => ServerState::Running,
                    _ => ServerState::Stopped,
                };
            }
            Message::ReloadConfig => {
                if self.server_state == ServerState::ReloadPending {
                    self.server_state = ServerState::Running;
                }
            }
            Message::RestartServer => {
                self.server_state = ServerState::Running;
                self.save_pending_restart = false;
            }

            // Save
            Message::Save | Message::SaveAll => {
                self.simulate_save();
                self.notice = Some(self.t(Key::FallbackSavedHint).to_string());
            }
            Message::DiscardChanges => {
                self.discard_all_changes();
            }

            // Drawer
            Message::OpenValidationDrawer => {
                self.drawer = Some(DrawerMode::Validation);
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

                // MK-048: starter choice drives the initial content.
                self.snapshot = Some(match self.wizard.starter {
                    WizardStarter::Empty => mock::blank_workspace(&name, &host, port, tls),
                    WizardStarter::Minimal => mock::minimal_workspace(&name, &host, port, tls),
                    WizardStarter::ShopApi => {
                        let mut ws = mock::shop_api_mock();
                        ws.meta.name = name.clone();
                        ws.meta.path = format!("~/{name}/apimock.toml");
                        ws.root_settings.listener_ip = host;
                        ws.root_settings.listener_port = port;
                        ws.root_settings.tls_enabled = tls;
                        ws
                    }
                });
                // Select the first rule / rule set if the starter provided them.
                if let Some(s) = &self.snapshot {
                    self.selection.rule_set = s.rule_sets.first().map(|rs| rs.id);
                    self.rule_set_open = self.selection.rule_set;
                    self.selection.rule = s
                        .rule_sets
                        .first()
                        .and_then(|rs| rs.rules.first())
                        .map(|r| r.id);
                }
                self.view = AppView::Workspace;
                self.tab = WorkspaceTab::Routes;
                self.server_state = crate::shell::top_bar::ServerState::Stopped;
                let notice_name = if self.wizard.name.trim().is_empty() {
                    "my-mock".to_string()
                } else {
                    self.wizard.name.trim().to_string()
                };
                self.notice = Some(format!(
                    "Workspace \"{notice_name}\" created. {}",
                    match self.wizard.starter {
                        WizardStarter::Empty => "Add a rule set to get started.",
                        WizardStarter::Minimal => "A starter GET /health rule is ready.",
                        WizardStarter::ShopApi => "Shop API example rules are loaded.",
                    }
                ));
            }
            Message::WizardCancel => {
                self.view = AppView::Welcome;
            }

            // Selection
            Message::SelectRuleSet(id) => {
                self.selection.rule_set = Some(id);
                self.selection.rule = None;
                // Accordion: opening a rule set closes others
                self.rule_set_open = Some(id);
            }
            Message::SelectRule(id) => {
                self.selection.rule = Some(id);
                self.selection.file_route = None;
                self.selection.script = None;
                if let Some(snap) = &self.snapshot {
                    for rs in &snap.rule_sets {
                        if rs.rules.iter().any(|r| r.id == id) {
                            self.selection.rule_set = Some(rs.id);
                            // Accordion: open the parent rule set
                            self.rule_set_open = Some(rs.id);
                            break;
                        }
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
                self.selection.file_route = Some(s);
                self.selection.rule = None;
                self.selection.rule_set = None; // prevent rule-set config taking priority
                self.selection.script = None;
            }
            Message::SelectScript(s) => {
                self.selection.script = Some(s);
                self.selection.rule = None;
                self.selection.rule_set = None; // prevent rule-set config taking priority
                self.selection.file_route = None;
            }
            Message::AddRuleSet => {
                // MK-048: create a real RuleSetView with a generated filename.
                let mut undo_cmd: Option<UndoCommand> = None;
                if let Some(snap) = &mut self.snapshot {
                    use apimokka_model::{
                        NodeValidation, RuleSetId,
                        node::{ConfigFileKind, ConfigFileView},
                        snapshot::RuleSetView,
                    };
                    let n = snap.rule_sets.len() + 1;
                    let path = format!("rules/rule-set-{n}.toml");
                    let rs_id = RuleSetId(apimokka_model::NodeId::new());
                    let rs = RuleSetView {
                        id: rs_id,
                        file: ConfigFileView {
                            kind: ConfigFileKind::RuleSet,
                            path: path.clone(),
                            dirty: true,
                        },
                        rules: vec![],
                        validation: NodeValidation::default(),
                    };
                    snap.rule_sets.push(rs);
                    self.selection.rule_set = Some(rs_id);
                    self.rule_set_open = Some(rs_id);
                    self.selection.rule = None;
                    undo_cmd = Some(UndoCommand::AddRule {
                        // repurpose for rule-set undo stub
                        rule_set: rs_id,
                        rule_id: apimokka_model::NodeId::new(),
                    });
                }
                let _ = undo_cmd; // TODO: UndoCommand::AddRuleSet in future RFC
                self.recompute_dirty();
            }
            Message::AddRule(rs_id) => {
                // MK-045: adding a rule is undoable (undo removes the new rule).
                // Stub adds a placeholder rule and records it on the stack.
                let mut undo_cmd: Option<UndoCommand> = None;
                if let Some(snap) = &mut self.snapshot {
                    if let Some(rs) = snap.rule_sets.iter_mut().find(|rs| rs.id == rs_id) {
                        use apimokka_model::{
                            NodeId, NodeValidation, rule::RulePayload, snapshot::RuleView,
                        };
                        let new_id = NodeId::new();
                        let new_rule = RuleView {
                            id: new_id,
                            payload: RulePayload::default(),
                            validation: NodeValidation::default(),
                            matched_by_latest_trace: false,
                        };
                        rs.rules.push(new_rule);
                        rs.file.dirty = true;
                        self.selection.rule = Some(new_id);
                        self.selection.rule_set = Some(rs_id);
                        self.rule_set_open = Some(rs_id);
                        undo_cmd = Some(UndoCommand::AddRule {
                            rule_set: rs_id,
                            rule_id: new_id,
                        });
                    }
                }
                if let Some(cmd) = undo_cmd {
                    self.push_undo(cmd);
                }
                self.auto_save_rules();
            }
            Message::MoveRuleUp(id) => {
                let mut undo_cmd: Option<UndoCommand> = None;
                if let Some(snap) = &mut self.snapshot {
                    for rs in &mut snap.rule_sets {
                        if let Some(i) = rs.rules.iter().position(|r| r.id == id) {
                            if i > 0 {
                                rs.rules.swap(i, i - 1);
                                rs.file.dirty = true;
                                undo_cmd = Some(UndoCommand::MoveRule {
                                    rule_set: rs.id,
                                    rule_id: id,
                                    from_index: i,
                                });
                            }
                            break;
                        }
                    }
                }
                if let Some(cmd) = undo_cmd {
                    self.push_undo(cmd);
                }
                self.auto_save_rules();
            }
            Message::MoveRuleDown(id) => {
                let mut undo_cmd: Option<UndoCommand> = None;
                if let Some(snap) = &mut self.snapshot {
                    for rs in &mut snap.rule_sets {
                        if let Some(i) = rs.rules.iter().position(|r| r.id == id) {
                            if i + 1 < rs.rules.len() {
                                rs.rules.swap(i, i + 1);
                                rs.file.dirty = true;
                                undo_cmd = Some(UndoCommand::MoveRule {
                                    rule_set: rs.id,
                                    rule_id: id,
                                    from_index: i,
                                });
                            }
                            break;
                        }
                    }
                }
                if let Some(cmd) = undo_cmd {
                    self.push_undo(cmd);
                }
                self.auto_save_rules();
            }
            Message::DeleteRuleSet(id) => {
                self.update(Message::ConfirmRequest(ConfirmAction::DeleteRuleSet(id)));
            }
            Message::DeleteRule(id) => {
                // MK-045: push to undo stack before removing.
                let mut undo_cmd: Option<UndoCommand> = None;
                if let Some(snap) = &mut self.snapshot {
                    for rs in &mut snap.rule_sets {
                        if let Some(index) = rs.rules.iter().position(|r| r.id == id) {
                            let rule = rs.rules.remove(index);
                            rs.file.dirty = true;
                            let rid = rule.id;
                            undo_cmd = Some(UndoCommand::DeleteRule {
                                rule_set: rs.id,
                                index,
                                rule_id: rid,
                                rule,
                            });
                            break;
                        }
                    }
                }
                if let Some(cmd) = undo_cmd {
                    self.push_undo(cmd);
                }
                if self.selection.rule == Some(id) {
                    self.selection.rule = None;
                }
                self.auto_save_rules();
            }
            Message::DuplicateRule(id) => {
                // MK-049: clone the rule with a fresh NodeId, insert after original.
                let mut undo_cmd: Option<UndoCommand> = None;
                if let Some(snap) = &mut self.snapshot {
                    for rs in &mut snap.rule_sets {
                        if let Some(pos) = rs.rules.iter().position(|r| r.id == id) {
                            use apimokka_model::NodeId;
                            let new_id = NodeId::new();
                            let mut copy = rs.rules[pos].clone();
                            copy.id = new_id;
                            copy.matched_by_latest_trace = false;
                            let insert_at = pos + 1;
                            rs.rules.insert(insert_at, copy);
                            rs.file.dirty = true;
                            self.selection.rule = Some(new_id);
                            undo_cmd = Some(UndoCommand::AddRule {
                                rule_set: rs.id,
                                rule_id: new_id,
                            });
                            break;
                        }
                    }
                }
                if let Some(cmd) = undo_cmd {
                    self.push_undo(cmd);
                }
                self.auto_save_rules();
            }
            // Rule edits — auto-save via with_rule
            Message::RuleSetUrlPath(v) => {
                // MK-045: capture old URL path before overwriting.
                if let Some(id) = self.selection.rule {
                    let old = self
                        .snapshot
                        .as_ref()
                        .and_then(|s| s.find_rule(id).map(|(_, r)| r.payload.url_path.clone()))
                        .unwrap_or_default();
                    if old != v {
                        self.push_undo(UndoCommand::EditUrlPath {
                            rule_id: id,
                            old_value: old,
                            new_value: v.clone(),
                        });
                    }
                }
                self.with_rule(|r| r.payload.url_path = v);
            }
            Message::RuleSetUrlPathOp(op) => {
                self.with_rule(|r| r.payload.url_path_op = Some(op));
            }
            Message::RuleSetUrlPathEnabled(v) => {
                self.with_rule(|r| {
                    if !v {
                        r.payload.url_path.clear();
                        r.payload.url_path_op = None;
                    }
                });
            }
            Message::RuleSetMethod(m) => {
                self.with_rule(|r| r.payload.method = m);
            }
            Message::HeaderAdd => {
                self.with_rule(|r| {
                    r.payload.headers.push(HeaderConditionPayload {
                        name: String::new(),
                        op: HeaderOp::Equal,
                        value: String::new(),
                    })
                });
            }
            Message::HeaderRemove(i) => {
                self.with_rule(|r| {
                    if i < r.payload.headers.len() {
                        r.payload.headers.remove(i);
                    }
                });
            }
            Message::HeaderSetName { index, value } => {
                self.with_rule(|r| {
                    if index < r.payload.headers.len() {
                        r.payload.headers[index].name = value;
                    }
                });
            }
            Message::HeaderSetOp { index, op } => {
                self.with_rule(|r| {
                    if index < r.payload.headers.len() {
                        r.payload.headers[index].op = op;
                    }
                });
            }
            Message::HeaderSetValue { index, value } => {
                self.with_rule(|r| {
                    if index < r.payload.headers.len() {
                        r.payload.headers[index].value = value;
                    }
                });
            }
            Message::HeaderClearAll => {
                self.with_rule(|r| r.payload.headers.clear());
            }
            Message::BodyAdd => {
                self.with_rule(|r| {
                    r.payload.body.push(BodyConditionPayload {
                        path: String::new(),
                        op: BodyOp::Equal,
                        value: String::new(),
                    })
                });
            }
            Message::BodyRemove(i) => {
                self.with_rule(|r| {
                    if i < r.payload.body.len() {
                        r.payload.body.remove(i);
                    }
                });
            }
            Message::BodySetPath { index, value } => {
                self.with_rule(|r| {
                    if index < r.payload.body.len() {
                        r.payload.body[index].path = value;
                    }
                });
            }
            Message::BodySetOp { index, op } => {
                self.with_rule(|r| {
                    if index < r.payload.body.len() {
                        r.payload.body[index].op = op;
                    }
                });
            }
            Message::BodySetValue { index, value } => {
                self.with_rule(|r| {
                    if index < r.payload.body.len() {
                        r.payload.body[index].value = value;
                    }
                });
            }
            Message::BodyClearAll => {
                self.with_rule(|r| r.payload.body.clear());
            }
            Message::BodyOpenPathAssistant(i) => {
                self.path_assistant.open = true;
                self.path_assistant.target_index = i;
                self.path_assistant.json_input = String::new();
                self.path_assistant.selected_path = String::new();
            }
            Message::RespondSetMode(m) => {
                self.with_rule(|r| r.payload.respond.mode = m);
            }
            Message::RespondSetText(v) => {
                self.with_rule(|r| r.payload.respond.text = v);
            }
            Message::RespondSetFilePath(v) => {
                self.with_rule(|r| r.payload.respond.file_path = v);
            }
            Message::RespondSetStatus(v) => {
                self.with_rule(|r| r.payload.respond.status = v);
            }
            Message::RespondSetDelay(v) => {
                self.with_rule(|r| r.payload.respond.delay_milliseconds = v.parse().unwrap_or(0));
            }
            Message::RuleSetWeight(v) => {
                self.with_rule(|r| r.payload.weight = v.parse().ok());
            }
            Message::RuleSetPriority(v) => {
                self.with_rule(|r| r.payload.priority = v.parse().ok());
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
                    self.selected_rule().map(|rule| &rule.payload),
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
                            if let Some(s) = &mut self.snapshot {
                                s.rule_sets.retain(|rs| rs.id != id);
                                if self.selection.rule_set == Some(id) {
                                    self.selection.rule_set = s.rule_sets.first().map(|rs| rs.id);
                                    self.selection.rule = None;
                                }
                            }
                            self.auto_save_rules();
                        }
                        ConfirmAction::DiscardChanges => {
                            self.discard_all_changes();
                        }
                        ConfirmAction::SwitchWorkspace(name) => {
                            self.update(Message::OpenWorkspace(name));
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
            Message::SettingsSetName(v) => {
                if let Some(s) = &mut self.snapshot {
                    s.meta.name = v;
                }
            }
            Message::SettingsSetHost(v) => {
                if let Some(s) = &mut self.snapshot {
                    s.root_settings.listener_ip = v;
                }
                self.trigger_restart();
            }
            Message::SettingsSetPort(v) => {
                if let Some(s) = &mut self.snapshot {
                    s.root_settings.listener_port = v.parse().unwrap_or(8080);
                }
                self.trigger_restart();
            }
            Message::SettingsSetTls(v) => {
                if let Some(s) = &mut self.snapshot {
                    s.root_settings.tls_enabled = v;
                }
                self.trigger_restart();
            }
            Message::SettingsSetLogLevel(v) => {
                if let Some(s) = &mut self.snapshot {
                    s.root_settings.log_level = v;
                }
                self.trigger_reload();
            }
            Message::SettingsSetStrategy(st) => {
                if let Some(s) = &mut self.snapshot {
                    s.root_settings.strategy = st;
                }
                self.trigger_reload();
            }
            Message::SettingsSetTraceEnabled(v) => {
                if let Some(s) = &mut self.snapshot {
                    s.root_settings.trace_enabled = v;
                }
                self.trigger_reload();
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
                if let Some(path) = self.selection.file_route.clone() {
                    if let Some(content) = self.fallback_drafts.get(&path) {
                        let raw = content.text();
                        // Pretty-print only if the draft parses; otherwise keep as-is.
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) {
                            if let Ok(pretty) = serde_json::to_string_pretty(&val) {
                                self.fallback_drafts.insert(
                                    path,
                                    iced::widget::text_editor::Content::with_text(&pretty),
                                );
                                self.recompute_dirty();
                            }
                        }
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
                if self.selection.file_route.is_some() {
                    if let Some(path) = self.selection.file_route.clone() {
                        if self.is_fallback_dirty(&path) {
                            self.update(Message::ConfirmRequest(ConfirmAction::RevertFile(path)));
                        }
                    }
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Helpers
    // ─────────────────────────────────────────────────────────────────────────

    // ── MK-045: undo / redo helpers ───────────────────────────────────────────

    /// Push a command to the undo stack. Clears the redo stack.
    fn push_undo(&mut self, cmd: UndoCommand) {
        self.redo_stack.clear();
        self.undo_stack.push(cmd);
        if self.undo_stack.len() > UNDO_STACK_DEPTH {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the top command: apply its INVERSE, push the command to redo_stack.
    fn apply_undo(&mut self) {
        let Some(cmd) = self.undo_stack.pop() else {
            return;
        };
        self.apply_inverse(&cmd);
        self.redo_stack.push(cmd);
        self.auto_save_rules();
    }

    /// Redo the top command: apply it FORWARD, push the command back to undo_stack.
    fn apply_redo(&mut self) {
        let Some(cmd) = self.redo_stack.pop() else {
            return;
        };
        self.apply_forward(&cmd);
        self.undo_stack.push(cmd);
        self.auto_save_rules();
    }

    /// Apply the INVERSE of `cmd` (what undo does).
    fn apply_inverse(&mut self, cmd: &UndoCommand) {
        let Some(snap) = &mut self.snapshot else {
            return;
        };
        match cmd {
            // Undo delete → re-insert the rule
            UndoCommand::DeleteRule {
                rule_set,
                index,
                rule,
                ..
            } => {
                if let Some(rs) = snap.rule_sets.iter_mut().find(|rs| rs.id == *rule_set) {
                    let at = (*index).min(rs.rules.len());
                    rs.rules.insert(at, rule.clone());
                    rs.file.dirty = true;
                    self.selection.rule = Some(rule.id);
                }
            }
            // Undo add → remove the rule by id
            UndoCommand::AddRule { rule_set, rule_id } => {
                if let Some(rs) = snap.rule_sets.iter_mut().find(|rs| rs.id == *rule_set) {
                    if let Some(i) = rs.rules.iter().position(|r| r.id == *rule_id) {
                        rs.rules.remove(i);
                        rs.file.dirty = true;
                        if self.selection.rule == Some(*rule_id) {
                            self.selection.rule = None;
                        }
                    }
                }
            }
            // Undo move → move the rule back to `from_index`
            UndoCommand::MoveRule {
                rule_set,
                rule_id,
                from_index,
            } => {
                if let Some(rs) = snap.rule_sets.iter_mut().find(|rs| rs.id == *rule_set) {
                    if let Some(cur) = rs.rules.iter().position(|r| r.id == *rule_id) {
                        let rule = rs.rules.remove(cur);
                        let at = (*from_index).min(rs.rules.len());
                        rs.rules.insert(at, rule);
                        rs.file.dirty = true;
                    }
                }
            }
            // Undo url-path edit → restore old_value
            UndoCommand::EditUrlPath {
                rule_id, old_value, ..
            } => {
                for rs in &mut snap.rule_sets {
                    if let Some(r) = rs.rules.iter_mut().find(|r| r.id == *rule_id) {
                        r.payload.url_path = old_value.clone();
                        rs.file.dirty = true;
                        break;
                    }
                }
            }
        }
    }

    /// Apply `cmd` in the FORWARD direction (what redo does).
    fn apply_forward(&mut self, cmd: &UndoCommand) {
        let Some(snap) = &mut self.snapshot else {
            return;
        };
        match cmd {
            // Redo delete → remove the rule by id
            UndoCommand::DeleteRule { rule_set, rule, .. } => {
                if let Some(rs) = snap.rule_sets.iter_mut().find(|rs| rs.id == *rule_set) {
                    if let Some(i) = rs.rules.iter().position(|r| r.id == rule.id) {
                        rs.rules.remove(i);
                        rs.file.dirty = true;
                        if self.selection.rule == Some(rule.id) {
                            self.selection.rule = None;
                        }
                    }
                }
            }
            // Redo add → re-insert the rule (we need the payload on the AddRule command)
            // Since AddRule only has rule_id we can't fully redo — skip silently.
            UndoCommand::AddRule { .. } => {
                // AddRule redo is not fully supported in this iteration.
                // (Would require storing the full RuleView on AddRule.)
            }
            // Redo move → move the rule FROM from_index to where it was moved TO
            // We store from_index (before move); after swap it's at from_index ± 1.
            // Re-doing = calling MoveRuleDown/Up again — infer from position.
            UndoCommand::MoveRule {
                rule_set,
                rule_id,
                from_index,
            } => {
                if let Some(rs) = snap.rule_sets.iter_mut().find(|rs| rs.id == *rule_set) {
                    if let Some(cur) = rs.rules.iter().position(|r| r.id == *rule_id) {
                        // The rule is currently at `from_index` (restored by undo).
                        // Move it back where it was after the original operation.
                        let target = if cur + 1 <= rs.rules.len() - 1 {
                            cur + 1
                        } else {
                            cur
                        };
                        if target != cur {
                            rs.rules.swap(cur, target);
                            rs.file.dirty = true;
                        }
                        let _ = from_index;
                    }
                }
            }
            // Redo url-path edit → restore new_value
            UndoCommand::EditUrlPath {
                rule_id, new_value, ..
            } => {
                for rs in &mut snap.rule_sets {
                    if let Some(r) = rs.rules.iter_mut().find(|r| r.id == *rule_id) {
                        r.payload.url_path = new_value.clone();
                        rs.file.dirty = true;
                        break;
                    }
                }
            }
        }
    }

    fn with_rule(&mut self, f: impl FnOnce(&mut apimokka_model::snapshot::RuleView)) {
        if let (Some(snap), Some(id)) = (self.snapshot.as_mut(), self.selection.rule) {
            if let Some(r) = snap.find_rule_mut(id) {
                f(r);
            }
            if let Some(rs_id) = self.selection.rule_set {
                if let Some(rs) = snap.find_rule_set_mut(rs_id) {
                    rs.file.dirty = true;
                }
            }
            self.dirty_count = snap.dirty_file_count();
        }
        self.auto_save_rules();
    }

    /// Rule auto-save (MK-035): clears rule dirty flags ONLY. Fallback file
    /// drafts are explicitly saved (MK-038) and must never be committed as a
    /// side effect of rule editing.
    fn auto_save_rules(&mut self) {
        if let Some(s) = &mut self.snapshot {
            for rs in &mut s.rule_sets {
                rs.file.dirty = false;
            }
        }
        self.recompute_dirty();
    }

    /// Global save: clears rule dirty flags AND commits all dirty fallback
    /// drafts (MK-038). Used by top-bar Save / ⌘S.
    fn simulate_save(&mut self) {
        if let Some(s) = &mut self.snapshot {
            for rs in &mut s.rule_sets {
                rs.file.dirty = false;
            }
        }
        let dirty_paths: Vec<String> = self
            .fallback_drafts
            .keys()
            .filter(|p| self.is_fallback_dirty(p))
            .cloned()
            .collect();
        for p in dirty_paths {
            self.commit_fallback_draft(&p);
        }
        self.recompute_dirty();
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
        if let Some(draft) = self.fallback_drafts.get(path) {
            self.fallback_saved.insert(path.to_string(), draft.text());
        }
        if let Some(status) = self.fallback_status_draft.get(path) {
            self.fallback_status_saved
                .insert(path.to_string(), status.clone());
        }
    }

    /// Recompute the top-bar dirty counter: dirty rule files + dirty
    /// fallback files. Derived, never event-counted.
    fn recompute_dirty(&mut self) {
        let rule_dirty = self
            .snapshot
            .as_ref()
            .map(|s| s.dirty_file_count())
            .unwrap_or(0);
        let fallback_dirty = self
            .fallback_drafts
            .keys()
            .filter(|p| self.is_fallback_dirty(p))
            .count();
        self.dirty_count = rule_dirty + fallback_dirty;
    }

    /// Discard everything (MK-038): every dirty fallback draft is reset to
    /// its saved baseline, rule dirty flags are cleared, and the counter is
    /// recomputed. Used by the save-diff drawer's Discard action.
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
        if let Some(s) = &mut self.snapshot {
            for rs in &mut s.rule_sets {
                rs.file.dirty = false;
            }
        }
        self.recompute_dirty();
    }

    fn trigger_reload(&mut self) {
        if self.server_state == ServerState::Running {
            self.server_state = ServerState::ReloadPending;
        }
        self.dirty_count += 1;
    }

    fn trigger_restart(&mut self) {
        if matches!(
            self.server_state,
            ServerState::Running | ServerState::ReloadPending
        ) {
            self.server_state = ServerState::RestartRequired;
        }
        self.save_pending_restart = true;
        self.dirty_count += 1;
    }

    pub(crate) fn selected_rule(&self) -> Option<&apimokka_model::snapshot::RuleView> {
        let id = self.selection.rule?;
        self.snapshot.as_ref()?.find_rule(id).map(|(_, r)| r)
    }
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
        use iced::keyboard::{self, key::Named};
        keyboard::listen().map(|event| {
            if let iced::keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                match key {
                    iced::keyboard::Key::Named(Named::Escape) => {
                        return Message::EscapePressed;
                    }
                    iced::keyboard::Key::Character(ref c)
                        if c.as_str() == "k" && (modifiers.command() || modifiers.control()) =>
                    {
                        return Message::ToggleCommandPalette;
                    }
                    // MK-045: ⌘Z / Ctrl+Z → Undo; ⌘⇧Z / Ctrl+Shift+Z / Ctrl+Y → Redo
                    iced::keyboard::Key::Character(ref c)
                        if c.as_str() == "z"
                            && (modifiers.command() || modifiers.control())
                            && !modifiers.shift() =>
                    {
                        return Message::Undo;
                    }
                    iced::keyboard::Key::Character(ref c)
                        if c.as_str() == "z"
                            && (modifiers.command() || modifiers.control())
                            && modifiers.shift() =>
                    {
                        return Message::Redo;
                    }
                    iced::keyboard::Key::Character(ref c)
                        if c.as_str() == "y" && modifiers.control() && !modifiers.command() =>
                    {
                        return Message::Redo;
                    }
                    _ => {}
                }
            }
            Message::Noop
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
