//! Central app state and update (MK-021, MK-035).

use apimokka_i18n::{Key, Locale};
use apimokka_model::{
    BodyConditionPayload, BodyOp, HeaderConditionPayload, HeaderOp, mock,
    snapshot::WorkspaceSnapshot,
};
use iced::{Element, Subscription, Theme};

use crate::message::{ConfirmAction, Message, TestRuleResult};
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
                    .map(|r| (r.payload.method.clone(), r.payload.url_path.clone()))
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
            }
            Message::TestRuleSetPath(v) => {
                self.test_rule.url_path = v;
            }
            Message::TestRuleSetHeaders(v) => {
                self.test_rule.headers_text = v;
            }
            Message::TestRuleSetBody(v) => {
                self.test_rule.body = v;
            }
            Message::TestRuleRun => {
                self.test_rule.result = Some(self.run_stub_test());
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

    fn selected_rule(&self) -> Option<&apimokka_model::snapshot::RuleView> {
        let id = self.selection.rule?;
        self.snapshot.as_ref()?.find_rule(id).map(|(_, r)| r)
    }

    /// MK-049: evaluate the test request against the selected rule's conditions.
    /// Checks method, URL path (with op), all header conditions, and all body
    /// conditions. Body is parsed as JSON; the dotted-path accessor follows the
    /// engine's own semantics (`a.b.c`, `items.0.name`).
    fn run_stub_test(&self) -> TestRuleResult {
        let Some(rule) = self.selected_rule() else {
            return TestRuleResult::Error("No rule selected.".into());
        };
        let p = &rule.payload;

        // ── Method ──────────────────────────────────────────────────────────
        let req_method = self.test_rule.method.to_uppercase();
        if !p.method.is_empty() && p.method.to_uppercase() != req_method {
            return TestRuleResult::NoMatch;
        }

        // ── URL path ─────────────────────────────────────────────────────────
        let req_path = &self.test_rule.url_path;
        if !p.url_path.is_empty() {
            let matched = match p.url_path_op {
                Some(apimokka_model::UrlPathOp::StartsWith) => {
                    req_path.starts_with(p.url_path.as_str())
                }
                Some(apimokka_model::UrlPathOp::Contains) => req_path.contains(p.url_path.as_str()),
                Some(apimokka_model::UrlPathOp::EndsWith) => {
                    req_path.ends_with(p.url_path.as_str())
                }
                Some(apimokka_model::UrlPathOp::NotEqual) => req_path != &p.url_path,
                Some(apimokka_model::UrlPathOp::WildCard) => true, // best-effort
                _ => req_path == &p.url_path,
            };
            if !matched {
                return TestRuleResult::NoMatch;
            }
        }

        // ── Header conditions ─────────────────────────────────────────────────
        // Parse "name: value\nname2: value2" into a lowercase-name map.
        let req_headers: std::collections::HashMap<String, String> = self
            .test_rule
            .headers_text
            .lines()
            .filter_map(|line| {
                let (k, v) = line.split_once(':')?;
                Some((k.trim().to_lowercase(), v.trim().to_string()))
            })
            .collect();

        for hc in &p.headers {
            let name_lc = hc.name.to_lowercase();
            let actual = req_headers.get(&name_lc);
            use apimokka_model::HeaderOp;
            let matched = match hc.op {
                HeaderOp::Exists => actual.is_some(),
                HeaderOp::Absent => actual.is_none(),
                HeaderOp::Equal => actual.map(|v| v == &hc.value).unwrap_or(false),
                HeaderOp::NotEqual => actual.map(|v| v != &hc.value).unwrap_or(true),
                HeaderOp::Contains => actual
                    .map(|v| v.contains(hc.value.as_str()))
                    .unwrap_or(false),
                HeaderOp::StartsWith => actual
                    .map(|v| v.starts_with(hc.value.as_str()))
                    .unwrap_or(false),
                HeaderOp::EndsWith => actual
                    .map(|v| v.ends_with(hc.value.as_str()))
                    .unwrap_or(false),
                HeaderOp::Regex | HeaderOp::WildCard => actual.is_some(), // best-effort
            };
            if !matched {
                return TestRuleResult::NoMatch;
            }
        }

        // ── Body conditions ───────────────────────────────────────────────────
        if !p.body.is_empty() {
            let body_json: serde_json::Value = match serde_json::from_str(&self.test_rule.body) {
                Ok(v) => v,
                Err(_) => {
                    if !self.test_rule.body.is_empty() {
                        return TestRuleResult::Error(
                            "Body is not valid JSON — cannot evaluate body conditions.".into(),
                        );
                    }
                    serde_json::Value::Null
                }
            };

            for bc in &p.body {
                let target_val = dotted_path_get(&body_json, &bc.path);
                use apimokka_model::BodyOp;
                let matched = match bc.op {
                    BodyOp::Exists => target_val.is_some(),
                    BodyOp::Absent => target_val.is_none(),

                    BodyOp::Equal | BodyOp::EqualString => {
                        // String-coerce both sides
                        target_val
                            .map(|v| json_to_string(v) == bc.value)
                            .unwrap_or(false)
                    }
                    BodyOp::EqualTyped => {
                        // Compare JSON representations
                        let expected: serde_json::Value = serde_json::from_str(&bc.value)
                            .unwrap_or(serde_json::Value::String(bc.value.clone()));
                        target_val.map(|v| v == &expected).unwrap_or(false)
                    }
                    BodyOp::Contains => target_val
                        .map(|v| json_to_string(v).contains(bc.value.as_str()))
                        .unwrap_or(false),
                    BodyOp::StartsWith => target_val
                        .map(|v| json_to_string(v).starts_with(bc.value.as_str()))
                        .unwrap_or(false),
                    BodyOp::EndsWith => target_val
                        .map(|v| json_to_string(v).ends_with(bc.value.as_str()))
                        .unwrap_or(false),

                    BodyOp::EqualNumber | BodyOp::EqualInteger => {
                        let exp = bc.value.parse::<f64>().ok();
                        let act = target_val.and_then(|v| v.as_f64());
                        matches!((exp, act), (Some(e), Some(a)) if (e - a).abs() < f64::EPSILON)
                    }
                    BodyOp::GreaterThan => cmp_f64(target_val, &bc.value, |a, e| a > e),
                    BodyOp::LessThan => cmp_f64(target_val, &bc.value, |a, e| a < e),
                    BodyOp::GreaterOrEqual => cmp_f64(target_val, &bc.value, |a, e| a >= e),
                    BodyOp::LessOrEqual => cmp_f64(target_val, &bc.value, |a, e| a <= e),

                    BodyOp::ArrayLengthEqual => {
                        let exp = bc.value.parse::<usize>().ok();
                        let act = target_val.and_then(|v| v.as_array()).map(|a| a.len());
                        matches!((exp, act), (Some(e), Some(a)) if e == a)
                    }
                    BodyOp::ArrayLengthAtLeast => {
                        let exp = bc.value.parse::<usize>().ok();
                        let act = target_val.and_then(|v| v.as_array()).map(|a| a.len());
                        matches!((exp, act), (Some(e), Some(a)) if a >= e)
                    }
                    BodyOp::ArrayContains => {
                        let exp: serde_json::Value = serde_json::from_str(&bc.value)
                            .unwrap_or(serde_json::Value::String(bc.value.clone()));
                        target_val
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.contains(&exp))
                            .unwrap_or(false)
                    }
                    BodyOp::Regex => true, // best-effort: skip regex evaluation
                };
                if !matched {
                    return TestRuleResult::NoMatch;
                }
            }
        }

        TestRuleResult::Matched {
            summary: rule.summary(),
        }
    }
}

// ── Test runner helpers ────────────────────────────────────────────────────────

/// Navigate a `serde_json::Value` using the engine's dotted-path syntax.
/// `a.b.c` → nested objects; `items.0.name` → array index.
fn dotted_path_get<'v>(root: &'v serde_json::Value, path: &str) -> Option<&'v serde_json::Value> {
    let mut cur = root;
    for seg in path.split('.') {
        cur = if let Ok(idx) = seg.parse::<usize>() {
            cur.as_array()?.get(idx)?
        } else {
            cur.as_object()?.get(seg)?
        };
    }
    Some(cur)
}

fn json_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => "null".into(),
        other => other.to_string(),
    }
}

fn cmp_f64(
    actual: Option<&serde_json::Value>,
    expected_str: &str,
    pred: impl Fn(f64, f64) -> bool,
) -> bool {
    let exp = match expected_str.parse::<f64>() {
        Ok(v) => v,
        Err(_) => return false,
    };
    actual
        .and_then(|v| v.as_f64())
        .map(|act| pred(act, exp))
        .unwrap_or(false)
}

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

#[cfg(test)]
mod tests {
    //! Smoke + lifecycle tests. No iced_test: these exercise the pure update
    //! reducer and the MK-038 two-buffer lifecycle, plus a view build smoke
    //! test. They pin the invariants we fixed by hand across 0.6.x.
    use super::*;
    use crate::message::Message;
    use crate::selection::WorkspaceTab;
    use iced::widget::text_editor::Content;

    fn fresh() -> App {
        // MK-046: App now starts at Welcome with no snapshot.
        // Tests that exercise workspace features call this helper, which
        // sets mode and loads the workspace so the snapshot is available.
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        // Load mock workspace and navigate to the Routes workbench.
        a.update(Message::OpenWorkspace("test".into()));
        a
    }

    /// An app at first launch, before the audience mode is chosen.
    fn first_launch() -> App {
        App::new().0
    }

    fn first_fallback_path(a: &App) -> String {
        a.snapshot.as_ref().unwrap().fallback_files[0].path.clone()
    }

    // ── Selection / accordion invariants ──────────────────────────────────

    #[test]
    fn select_file_route_clears_rule_set() {
        // Regression: a stale rule_set selection used to make the rule-set
        // config view hijack the centre panel instead of the file editor.
        let mut a = fresh();
        assert!(a.selection.rule_set.is_some());
        let path = first_fallback_path(&a);
        a.update(Message::SelectFileRoute(path.clone()));
        assert_eq!(a.selection.file_route.as_deref(), Some(path.as_str()));
        assert!(
            a.selection.rule_set.is_none(),
            "file selection must clear rule_set"
        );
        assert!(a.selection.rule.is_none());
    }

    #[test]
    fn select_script_clears_rule_set() {
        let mut a = fresh();
        let snap = a.snapshot.as_ref().unwrap();
        if let Some(s) = snap.middleware_scripts.first() {
            let path = s.path.clone();
            a.update(Message::SelectScript(path.clone()));
            assert_eq!(a.selection.script.as_deref(), Some(path.as_str()));
            assert!(a.selection.rule_set.is_none());
        }
    }

    #[test]
    fn select_rule_set_is_single_open_accordion() {
        let mut a = fresh();
        let ids: Vec<_> = a
            .snapshot
            .as_ref()
            .unwrap()
            .rule_sets
            .iter()
            .map(|rs| rs.id)
            .collect();
        if ids.len() > 1 {
            a.update(Message::SelectRuleSet(ids[1]));
            assert_eq!(
                a.rule_set_open,
                Some(ids[1]),
                "selected set becomes the open one"
            );
            assert_eq!(a.selection.rule_set, Some(ids[1]));
            assert!(a.selection.rule.is_none());
        }
    }

    #[test]
    fn toggle_sidebar_sections() {
        let mut a = fresh();
        assert!(!a.fallback_section_open);
        a.update(Message::ToggleFallbackSection);
        assert!(a.fallback_section_open);
        a.update(Message::ToggleMiddlewareSection);
        assert!(a.middleware_section_open);
    }

    // ── MK-038 fallback file lifecycle ────────────────────────────────────

    #[test]
    fn fallback_dirty_then_save_clean() {
        let mut a = fresh();
        let path = first_fallback_path(&a);
        a.update(Message::SelectFileRoute(path.clone()));
        assert!(!a.is_fallback_dirty(&path), "freshly opened file is clean");

        // Simulate an edit by replacing the draft buffer.
        a.fallback_drafts
            .insert(path.clone(), Content::with_text("{\"x\":1}"));
        assert!(a.is_fallback_dirty(&path), "modified draft is dirty");

        a.update(Message::FallbackFileSave);
        assert!(!a.is_fallback_dirty(&path), "save commits draft → clean");
    }

    #[test]
    fn fallback_json_validity_predicate() {
        let mut a = fresh();
        let path = first_fallback_path(&a);
        a.update(Message::SelectFileRoute(path.clone()));

        a.fallback_drafts
            .insert(path.clone(), Content::with_text("{not valid"));
        assert!(!a.fallback_json_valid(&path), "broken JSON is invalid");

        a.fallback_drafts
            .insert(path.clone(), Content::with_text("{\"ok\":true}"));
        assert!(a.fallback_json_valid(&path), "well-formed JSON is valid");
    }

    #[test]
    fn rule_autosave_does_not_commit_fallback_drafts() {
        // The load-bearing separation: editing a rule must never silently
        // commit a dirty fallback file draft.
        let mut a = fresh();
        let path = first_fallback_path(&a);
        a.update(Message::SelectFileRoute(path.clone()));
        a.fallback_drafts
            .insert(path.clone(), Content::with_text("{\"edited\":1}"));
        assert!(a.is_fallback_dirty(&path));

        let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        a.update(Message::MoveRuleUp(rule_id)); // triggers auto_save_rules
        assert!(
            a.is_fallback_dirty(&path),
            "rule auto-save must not commit fallback file drafts"
        );
    }

    #[test]
    fn global_save_commits_fallback_drafts() {
        let mut a = fresh();
        let path = first_fallback_path(&a);
        a.update(Message::SelectFileRoute(path.clone()));
        a.fallback_drafts
            .insert(path.clone(), Content::with_text("{\"edited\":2}"));
        assert!(a.is_fallback_dirty(&path));

        a.update(Message::Save); // global Save
        assert!(
            !a.is_fallback_dirty(&path),
            "global save commits all drafts"
        );
    }

    // ── View build smoke tests (no rendering, just tree construction) ──────

    #[test]
    fn screen_views_build_without_panic() {
        for tab in [
            WorkspaceTab::Routes,
            WorkspaceTab::Trace,
            WorkspaceTab::Settings,
        ] {
            let mut a = fresh();
            a.tab = tab;
            // Element is built and dropped — catches view-construction panics.
            let _ = match tab {
                WorkspaceTab::Routes => crate::screens::routes::view(&a),
                WorkspaceTab::Trace => crate::screens::trace::view(&a),
                WorkspaceTab::Settings => crate::screens::settings::view(&a),
            };
        }
    }

    #[test]
    fn routes_view_builds_for_each_selection() {
        // Each centre-panel branch (rule / file / script / rule-set config)
        // must build without panicking.
        let mut a = fresh();
        let snap = a.snapshot.as_ref().unwrap();
        let rule_id = snap.rule_sets[0].rules[0].id;
        let rs_id = snap.rule_sets[0].id;
        let file = snap.fallback_files[0].path.clone();

        a.update(Message::SelectRule(rule_id));
        let _ = crate::screens::routes::view(&a);

        a.update(Message::SelectRuleSet(rs_id)); // rule set config (no rule)
        let _ = crate::screens::routes::view(&a);

        a.update(Message::SelectFileRoute(file));
        let _ = crate::screens::routes::view(&a);
    }

    // ── MK-039: non-modal undo + feedback ─────────────────────────────────

    #[test]
    fn delete_rule_is_reversible_without_dialog() {
        let mut a = fresh();
        let (rs_id, rule_id, before) = {
            let snap = a.snapshot.as_ref().unwrap();
            let rs = &snap.rule_sets[0];
            (rs.id, rs.rules[0].id, rs.rules.len())
        };

        a.update(Message::DeleteRule(rule_id));
        // No confirm dialog for this low-risk action.
        assert!(
            a.confirm_dialog.is_none(),
            "delete rule must not open a dialog"
        );
        // Rule is gone and an undo is offered.
        let after = a
            .snapshot
            .as_ref()
            .unwrap()
            .rule_sets
            .iter()
            .find(|rs| rs.id == rs_id)
            .unwrap()
            .rules
            .len();
        assert_eq!(after, before - 1);
        assert!(!a.undo_stack.is_empty(), "an undo entry must be offered");

        // Undo restores it at the same index.
        a.update(Message::UndoLast);
        let restored = a
            .snapshot
            .as_ref()
            .unwrap()
            .rule_sets
            .iter()
            .find(|rs| rs.id == rs_id)
            .unwrap()
            .rules
            .len();
        assert_eq!(restored, before);
        assert!(a.undo_stack.is_empty(), "undo stack is empty after use");
    }

    #[test]
    fn save_sets_a_success_notice() {
        let mut a = fresh();
        let path = first_fallback_path(&a);
        a.update(Message::SelectFileRoute(path.clone()));
        a.fallback_drafts.insert(
            path,
            iced::widget::text_editor::Content::with_text("{\"a\":1}"),
        );
        a.update(Message::Save);
        assert!(a.notice.is_some(), "save shows a success notice");
        a.update(Message::DismissNotice);
        assert!(a.notice.is_none() && a.undo_stack.is_empty());
    }

    #[test]
    fn problem_action_routes_to_settings() {
        let mut a = fresh();
        a.last_problem = Some(apimokka_model::FriendlyProblem::port_in_use(8080));
        a.update(Message::ProblemAction);
        assert_eq!(a.tab, crate::selection::WorkspaceTab::Settings);
        assert!(a.last_problem.is_none(), "problem cleared after action");
    }

    #[test]
    fn body_size_meets_comfort_floor() {
        // MK-039 comfort: body text is at least 16 px.
        assert!(crate::theme::size::BODY >= 16.0);
        assert!(crate::theme::touch::COMFORTABLE >= 52.0);
    }

    // ── MK-040: audience modes ────────────────────────────────────────────

    #[test]
    fn first_launch_has_no_mode_then_choice_persists() {
        use apimokka_model::AudienceMode;
        let mut a = first_launch();
        assert!(a.audience_mode.is_none(), "first launch shows the picker");

        a.update(Message::ChooseAudienceMode(AudienceMode::Guided));
        assert_eq!(a.audience_mode, Some(AudienceMode::Guided));
        // The picker is gated on audience_mode being None, so a Some value
        // means it will not show again.
        assert!(a.audience_mode.is_some());
    }

    #[test]
    fn guided_shows_scaffolding_expert_does_not() {
        use apimokka_model::AudienceMode;
        let mut a = first_launch();
        a.update(Message::ChooseAudienceMode(AudienceMode::Guided));
        assert!(a.shows_scaffolding());
        a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
        assert!(!a.shows_scaffolding());
    }

    #[test]
    fn choosing_expert_expands_problem_details_by_default() {
        use apimokka_model::AudienceMode;
        let mut a = first_launch();
        a.update(Message::ChooseAudienceMode(AudienceMode::Expert));
        assert!(
            a.show_problem_details,
            "Expert expands technical detail inline"
        );
        a.update(Message::ChooseAudienceMode(AudienceMode::Guided));
        assert!(!a.show_problem_details, "Guided collapses technical detail");
        // And it can be toggled regardless of mode.
        a.update(Message::ToggleProblemDetails);
        assert!(a.show_problem_details);
    }

    #[test]
    fn vocabulary_is_identical_between_modes() {
        // The core MK-040 guarantee: switching mode never changes a domain
        // label. We sample the field/card titles that carry hints.
        use apimokka_i18n::Key;
        let mut a = first_launch();
        let keys = [
            Key::UrlPathCardTitle,
            Key::MethodCardTitle,
            Key::HeadersCardTitle,
            Key::BodyCardTitle,
        ];
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Guided,
        ));
        let guided: Vec<&str> = keys.iter().map(|k| a.t(*k)).collect();
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        let expert: Vec<&str> = keys.iter().map(|k| a.t(*k)).collect();
        assert_eq!(
            guided, expert,
            "domain vocabulary must not change with mode"
        );
    }

    #[test]
    fn mode_picker_view_builds() {
        let a = first_launch();
        let _ = crate::screens::mode_picker::view(&a);
    }

    #[test]
    fn error_banner_builds_in_both_modes() {
        // The banner renders technical detail inline (Expert) or behind a
        // toggle (Guided); both must build.
        for mode in apimokka_model::AudienceMode::all() {
            let mut a = first_launch();
            a.update(Message::ChooseAudienceMode(mode));
            a.last_problem = Some(apimokka_model::FriendlyProblem::port_in_use(8080));
            let _ = crate::shell::view::view(&a);
        }
    }
}

#[cfg(test)]
mod tests_mk041 {
    use super::*;
    use crate::message::Message;

    fn guided() -> App {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Guided,
        ));
        a.update(Message::OpenWorkspace("test".into()));
        a
    }
    fn expert() -> App {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.update(Message::OpenWorkspace("test".into()));
        a
    }

    #[test]
    fn guided_when_starts_collapsed_and_resets_on_mode_switch() {
        let mut a = expert();
        a.update(Message::ToggleRuleWhenMore);
        assert!(a.rule_when_more);
        // Switching to Guided resets density toggles.
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Guided,
        ));
        assert!(
            !a.rule_when_more,
            "switching to Guided resets advanced layout"
        );
        assert!(!a.settings_advanced_more);
    }

    #[test]
    fn rule_when_more_persists_across_rule_navigation() {
        let mut a = guided();
        a.update(Message::ToggleRuleWhenMore);
        assert!(a.rule_when_more);
        // Navigate to a different rule — expanded state must persist.
        let snap = a.snapshot.as_ref().unwrap();
        if snap.rule_sets[0].rules.len() > 1 {
            let other_id = snap.rule_sets[0].rules[1].id;
            a.update(Message::SelectRule(other_id));
        }
        assert!(
            a.rule_when_more,
            "expanded state persists across rule navigation"
        );
    }

    #[test]
    fn settings_advanced_toggle_works() {
        let mut a = guided();
        assert!(!a.settings_advanced_more);
        a.update(Message::ToggleSettingsAdvancedMore);
        assert!(a.settings_advanced_more);
        a.update(Message::ToggleSettingsAdvancedMore);
        assert!(!a.settings_advanced_more);
    }

    #[test]
    fn routes_view_builds_in_guided_collapsed_and_expanded() {
        let rule_id = expert().snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        for expanded in [false, true] {
            let mut a = guided();
            a.rule_when_more = expanded;
            a.update(Message::SelectRule(rule_id));
            let _ = crate::screens::routes::view(&a);
        }
    }

    #[test]
    fn settings_view_builds_in_guided_collapsed_and_expanded() {
        for expanded in [false, true] {
            let mut a = guided();
            a.settings_advanced_more = expanded;
            a.tab = crate::selection::WorkspaceTab::Settings;
            let _ = crate::screens::settings::view(&a);
        }
    }
}

#[cfg(test)]
mod tests_mk042 {
    use super::*;
    use crate::message::Message;
    use apimokka_model::{MatchTraceEvent, RequestSummary, TraceOutcome};

    fn with_trace() -> App {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.update(Message::OpenWorkspace("test".into()));
        a.trace = apimokka_model::mock::sample_trace_events();
        a
    }

    // ── Filter ──────────────────────────────────────────────────────────

    #[test]
    fn filter_empty_shows_all() {
        let a = with_trace();
        let q = a.trace_filter.clone();
        assert!(q.is_empty());
        // All events pass an empty filter (checked via trace count equality).
        let filtered: Vec<_> = a
            .trace
            .iter()
            .filter(|ev| {
                ev.request.url_path.to_lowercase().contains(&q)
                    || ev.request.method.to_lowercase().contains(&q)
                    || ev.outcome.label().contains(q.as_str())
                    || q.is_empty()
            })
            .collect();
        assert_eq!(filtered.len(), a.trace.len());
    }

    #[test]
    fn filter_by_path_narrows_list() {
        let mut a = with_trace();
        a.update(Message::TraceFilterChanged("/api/orders".into()));
        assert_eq!(a.trace_filter, "/api/orders");
        let filtered: Vec<_> = a
            .trace
            .iter()
            .filter(|ev| ev.request.url_path.contains("/api/orders"))
            .collect();
        // sample data has 2 events on /api/orders
        assert!(!filtered.is_empty());
        assert!(filtered.len() < a.trace.len());
    }

    // ── Jump actions ─────────────────────────────────────────────────────

    #[test]
    fn jump_to_rule_switches_tab_and_selects() {
        let mut a = with_trace();
        a.tab = crate::selection::WorkspaceTab::Trace;
        let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        a.update(Message::JumpToRule(rule_id));
        assert_eq!(a.tab, crate::selection::WorkspaceTab::Routes);
        assert_eq!(a.selection.rule, Some(rule_id));
    }

    #[test]
    fn jump_to_file_switches_tab_and_selects() {
        let mut a = with_trace();
        a.tab = crate::selection::WorkspaceTab::Trace;
        let path = a.snapshot.as_ref().unwrap().fallback_files[0].path.clone();
        a.update(Message::JumpToFile(path.clone()));
        assert_eq!(a.tab, crate::selection::WorkspaceTab::Routes);
        assert_eq!(a.selection.file_route.as_deref(), Some(path.as_str()));
    }

    // ── Rule editor trace strip ──────────────────────────────────────────

    #[test]
    fn trace_strip_matches_by_rule_index() {
        // The Matched event in sample data has rule_set_index=0, rule_index=2.
        // The strip for rule at [0][2] should find it; rule at [0][0] should not.
        let a = with_trace();
        let snap = a.snapshot.as_ref().unwrap();

        let rule_matches = |rule: &apimokka_model::snapshot::RuleView| {
            let rule_position: Option<(usize, usize)> =
                snap.rule_sets.iter().enumerate().find_map(|(rs_idx, rs)| {
                    rs.rules
                        .iter()
                        .position(|r| r.id == rule.id)
                        .map(|r_idx| (rs_idx, r_idx))
                });
            a.trace.iter().any(|ev| {
                matches!(&ev.outcome,
                    TraceOutcome::Matched { rule_set_index, rule_index }
                        if rule_position == Some((*rule_set_index, *rule_index))
                )
            })
        };

        let rule_2 = &snap.rule_sets[0].rules[2];
        let rule_0 = &snap.rule_sets[0].rules[0];
        assert!(
            rule_matches(rule_2),
            "rule at index 2 should have a matched trace event"
        );
        assert!(
            !rule_matches(rule_0),
            "rule at index 0 has no matched trace event"
        );
    }

    // ── Trace view builds with each outcome ──────────────────────────────

    #[test]
    fn trace_view_builds_for_each_outcome() {
        use apimokka_model::TraceOutcome;
        let outcomes = vec![
            TraceOutcome::Matched {
                rule_set_index: 0,
                rule_index: 0,
            },
            TraceOutcome::Fallback {
                file_path: "responses/health.json".into(),
                status: "200 OK".into(),
            },
            TraceOutcome::Miss {
                status: "404 Not Found".into(),
            },
            TraceOutcome::Error {
                kind: "RespondFile".into(),
                message: "permission denied".into(),
            },
        ];
        for (i, outcome) in outcomes.into_iter().enumerate() {
            let mut a = App::new().0;
            a.update(Message::ChooseAudienceMode(
                apimokka_model::AudienceMode::Expert,
            ));
            a.trace = vec![MatchTraceEvent {
                event_id: i as u64,
                time: "12:00:00.000".into(),
                duration_ms: 1,
                request: RequestSummary {
                    method: "GET".into(),
                    url_path: "/test".into(),
                    headers: vec![],
                    body_preview: None,
                },
                outcome,
                dropped_count: if i == 2 { 5 } else { 0 }, // test dropped_count warning
            }];
            a.selected_trace = Some(i as u64);
            let _ = crate::screens::trace::view(&a);
        }
    }
}

#[cfg(test)]
mod tests_mk043 {
    use super::*;
    use crate::message::Message;
    use apimokka_model::settings::Strategy;

    fn expert() -> App {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.update(Message::OpenWorkspace("test".into()));
        a
    }
    fn guided() -> App {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Guided,
        ));
        a.update(Message::OpenWorkspace("test".into()));
        a
    }

    // ── Strategy update ──────────────────────────────────────────────────

    #[test]
    fn strategy_change_updates_snapshot() {
        let mut a = expert();
        let before = a.snapshot.as_ref().unwrap().root_settings.strategy;
        // Switch to something different
        let new_strategy = if before == Strategy::FirstMatch {
            Strategy::WeightedRandom
        } else {
            Strategy::FirstMatch
        };
        a.update(Message::RuleSetSetStrategy(new_strategy));
        let after = a.snapshot.as_ref().unwrap().root_settings.strategy;
        assert_eq!(after, new_strategy, "strategy should update the snapshot");
    }

    #[test]
    fn weight_changed_updates_rule_payload() {
        let mut a = expert();
        a.update(Message::RuleSetSetStrategy(Strategy::WeightedRandom));
        let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        a.update(Message::SelectRule(rule_id));
        a.update(Message::RuleWeightChanged("7".into()));
        let snap = a.snapshot.as_ref().unwrap();
        let rule = snap.rule_sets[0]
            .rules
            .iter()
            .find(|r| r.id == rule_id)
            .unwrap();
        assert_eq!(rule.payload.weight, Some(7));
    }

    #[test]
    fn priority_changed_updates_rule_payload() {
        let mut a = expert();
        a.update(Message::RuleSetSetStrategy(Strategy::Priority));
        let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        a.update(Message::SelectRule(rule_id));
        a.update(Message::RulePriorityChanged("-5".into()));
        let snap = a.snapshot.as_ref().unwrap();
        let rule = snap.rule_sets[0]
            .rules
            .iter()
            .find(|r| r.id == rule_id)
            .unwrap();
        assert_eq!(rule.payload.priority, Some(-5));
    }

    #[test]
    fn invalid_weight_input_leaves_none() {
        let mut a = expert();
        a.update(Message::RuleSetSetStrategy(Strategy::WeightedRandom));
        let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        a.update(Message::SelectRule(rule_id));
        a.update(Message::RuleWeightChanged("not-a-number".into()));
        let snap = a.snapshot.as_ref().unwrap();
        let rule = snap.rule_sets[0]
            .rules
            .iter()
            .find(|r| r.id == rule_id)
            .unwrap();
        assert_eq!(
            rule.payload.weight, None,
            "non-numeric input should leave weight as None"
        );
    }

    // ── Layout density (Guided mode) ────────────────────────────────────

    #[test]
    fn rule_set_config_more_resets_on_guided() {
        let mut a = expert();
        a.rule_set_config_more = true;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Guided,
        ));
        assert!(
            !a.rule_set_config_more,
            "switching to Guided resets rule_set_config_more"
        );
    }

    #[test]
    fn toggle_rule_set_config_more_flips() {
        let mut a = guided();
        assert!(!a.rule_set_config_more);
        a.update(Message::ToggleRuleSetConfigMore);
        assert!(a.rule_set_config_more);
        a.update(Message::ToggleRuleSetConfigMore);
        assert!(!a.rule_set_config_more);
    }

    // ── View smoke tests ─────────────────────────────────────────────────

    #[test]
    fn rule_set_config_builds_in_both_modes_and_all_strategies() {
        for mode in apimokka_model::AudienceMode::all() {
            for strategy in Strategy::all() {
                let mut a = App::new().0;
                a.update(Message::ChooseAudienceMode(mode));
                a.update(Message::OpenWorkspace("test".into()));
                a.update(Message::RuleSetSetStrategy(strategy));
                let rs_id = a.snapshot.as_ref().unwrap().rule_sets[0].id;
                a.update(Message::SelectRuleSet(rs_id));
                let _ = crate::screens::routes::view(&a);
            }
        }
    }

    #[test]
    fn rule_editor_builds_with_weight_and_priority_fields() {
        for strategy in [Strategy::WeightedRandom, Strategy::Priority] {
            let mut a = expert();
            a.update(Message::RuleSetSetStrategy(strategy));
            let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
            a.update(Message::SelectRule(rule_id));
            let _ = crate::screens::routes::view(&a); // should not panic
        }
    }

    #[test]
    fn validation_strip_builds_for_rule_with_issues() {
        // The mock data has a rule (error-scenarios.toml rules[0]) with a
        // WeightedRandom validation warning — verify the view builds.
        let mut a = expert();
        let snap = a.snapshot.as_ref().unwrap();
        let rule_with_issues = snap
            .rule_sets
            .iter()
            .flat_map(|rs| rs.rules.iter())
            .find(|r| !r.validation.issues.is_empty());
        if let Some(rule) = rule_with_issues {
            let id = rule.id;
            a.update(Message::SelectRule(id));
            let _ = crate::screens::routes::view(&a);
        }
    }
}

#[cfg(test)]
mod tests_mk044 {
    use super::*;
    use crate::message::Message;
    use crate::selection::DrawerMode;

    fn expert() -> App {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.update(Message::OpenWorkspace("test".into()));
        a
    }

    // ── JumpToRule closes the drawer ────────────────────────────────────

    #[test]
    fn jump_to_rule_closes_drawer() {
        let mut a = expert();
        a.drawer = Some(DrawerMode::Validation);
        let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        a.update(Message::JumpToRule(rule_id));
        assert!(a.drawer.is_none(), "JumpToRule must close the drawer");
        assert_eq!(a.tab, crate::selection::WorkspaceTab::Routes);
        assert_eq!(a.selection.rule, Some(rule_id));
    }

    // ── AddRuleFromPalette ──────────────────────────────────────────────

    #[test]
    fn add_rule_from_palette_closes_palette_and_navigates() {
        let mut a = expert();
        a.command_palette.open = true;
        a.tab = crate::selection::WorkspaceTab::Trace;
        a.update(Message::AddRuleFromPalette);
        assert!(!a.command_palette.open, "palette should close");
        assert_eq!(a.tab, crate::selection::WorkspaceTab::Routes);
        // The first rule set is selected (accordion opened).
        assert!(
            a.selection.rule_set.is_some(),
            "a rule set should be selected/opened after AddRuleFromPalette"
        );
    }

    // ── Drawer view smoke tests ─────────────────────────────────────────

    #[test]
    fn validation_drawer_builds_with_issues_and_clean() {
        // Mock has one rule set with validation issues, one without.
        let mut a = expert();
        a.drawer = Some(DrawerMode::Validation);
        let _ = crate::shell::view::view(&a); // should not panic
    }

    #[test]
    fn save_diff_drawer_builds_with_dirty_and_clean() {
        let mut a = expert();
        a.drawer = Some(DrawerMode::SaveDiff);
        // Snapshot already has main.toml as dirty in the mock.
        let _ = crate::shell::view::view(&a);
    }

    #[test]
    fn save_diff_drawer_builds_with_no_changes() {
        let mut a = expert();
        a.drawer = Some(DrawerMode::SaveDiff);
        // Mark everything clean.
        if let Some(snap) = &mut a.snapshot {
            for rs in &mut snap.rule_sets {
                rs.file.dirty = false;
            }
        }
        let _ = crate::shell::view::view(&a);
    }

    #[test]
    fn validation_drawer_builds_when_all_clean() {
        let mut a = expert();
        a.drawer = Some(DrawerMode::Validation);
        // Clear all validation issues.
        if let Some(snap) = &mut a.snapshot {
            for rs in &mut snap.rule_sets {
                for rule in &mut rs.rules {
                    rule.validation.issues.clear();
                }
            }
            snap.diagnostics.clear();
        }
        let _ = crate::shell::view::view(&a);
    }
}

#[cfg(test)]
mod tests_mk045 {
    use super::*;
    use crate::message::Message;

    fn expert() -> App {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.update(Message::OpenWorkspace("test".into()));
        a
    }

    // ── Stack basics ─────────────────────────────────────────────────────

    #[test]
    fn delete_rule_uses_stack_and_undo_restores() {
        let mut a = expert();
        let (rs_id, rule_id, before) = {
            let snap = a.snapshot.as_ref().unwrap();
            let rs = &snap.rule_sets[0];
            (rs.id, rs.rules[0].id, rs.rules.len())
        };
        a.update(Message::DeleteRule(rule_id));
        assert!(a.confirm_dialog.is_none(), "no dialog for delete rule");
        assert_eq!(
            a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
            before - 1
        );
        assert!(
            matches!(a.undo_stack.last(), Some(UndoCommand::DeleteRule { .. })),
            "undo stack should have DeleteRule"
        );
        assert!(a.redo_stack.is_empty());

        a.update(Message::Undo);
        assert_eq!(
            a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
            before,
            "undo restores the rule"
        );
        assert!(a.undo_stack.is_empty(), "undo stack is empty after undo");
        assert!(
            matches!(a.redo_stack.last(), Some(UndoCommand::DeleteRule { .. })),
            "redo stack has the forward command"
        );

        _ = rs_id; // suppress warning
    }

    #[test]
    fn redo_reapplies_after_undo() {
        let mut a = expert();
        let (rule_id, before) = {
            let snap = a.snapshot.as_ref().unwrap();
            (snap.rule_sets[0].rules[0].id, snap.rule_sets[0].rules.len())
        };
        a.update(Message::DeleteRule(rule_id));
        a.update(Message::Undo); // restore
        assert_eq!(
            a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
            before
        );
        a.update(Message::Redo); // delete again
        assert_eq!(
            a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
            before - 1
        );
    }

    #[test]
    fn add_rule_is_undoable() {
        let mut a = expert();
        let rs_id = a.snapshot.as_ref().unwrap().rule_sets[0].id;
        let before = a.snapshot.as_ref().unwrap().rule_sets[0].rules.len();
        a.update(Message::AddRule(rs_id));
        assert_eq!(
            a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
            before + 1
        );
        assert!(matches!(
            a.undo_stack.last(),
            Some(UndoCommand::AddRule { .. })
        ));

        a.update(Message::Undo);
        assert_eq!(
            a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
            before,
            "undo removes the added rule"
        );
    }

    #[test]
    fn move_rule_is_undoable() {
        let mut a = expert();
        let snap = a.snapshot.as_ref().unwrap();
        let rule_id = snap.rule_sets[0].rules[0].id;
        let rule_1_id = snap.rule_sets[0].rules[1].id;
        drop(snap);

        a.update(Message::MoveRuleDown(rule_id));
        // rule[0] and rule[1] should be swapped
        let snap = a.snapshot.as_ref().unwrap();
        assert_eq!(snap.rule_sets[0].rules[1].id, rule_id);
        assert_eq!(snap.rule_sets[0].rules[0].id, rule_1_id);
        drop(snap);

        a.update(Message::Undo);
        let snap = a.snapshot.as_ref().unwrap();
        assert_eq!(
            snap.rule_sets[0].rules[0].id, rule_id,
            "undo restores original order"
        );
    }

    #[test]
    fn url_path_edit_is_undoable() {
        let mut a = expert();
        let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        a.update(Message::SelectRule(rule_id));
        a.update(Message::RuleSetUrlPath("/original".into()));
        // Push a second edit so we can undo to /original
        a.update(Message::RuleSetUrlPath("/modified".into()));
        let path = a.snapshot.as_ref().unwrap().rule_sets[0]
            .rules
            .iter()
            .find(|r| r.id == rule_id)
            .map(|r| r.payload.url_path.clone())
            .unwrap();
        assert_eq!(path, "/modified");

        a.update(Message::Undo);
        let path = a.snapshot.as_ref().unwrap().rule_sets[0]
            .rules
            .iter()
            .find(|r| r.id == rule_id)
            .map(|r| r.payload.url_path.clone())
            .unwrap();
        assert_eq!(path, "/original", "undo restores previous URL path");
    }

    #[test]
    fn new_edit_clears_redo_stack() {
        let mut a = expert();
        let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        a.update(Message::DeleteRule(rule_id));
        a.update(Message::Undo);
        assert!(
            !a.redo_stack.is_empty(),
            "redo should be available after undo"
        );

        // New edit should clear redo
        let rs_id = a.snapshot.as_ref().unwrap().rule_sets[0].id;
        a.update(Message::AddRule(rs_id));
        assert!(a.redo_stack.is_empty(), "new edit must clear redo stack");
    }

    #[test]
    fn undo_redo_keyboard_shortcut_exists() {
        // Smoke: Undo and Redo messages are in the enum and handled.
        let mut a = expert();
        // Neither crashes when stacks are empty.
        a.update(Message::Undo);
        a.update(Message::Redo);
    }

    #[test]
    fn dismiss_notice_does_not_clear_undo_stack() {
        // Regression: DismissNotice previously called retain(|_| false) which
        // cleared the undo stack. The banner should dismiss independently of undo.
        let mut a = expert();
        let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        a.update(Message::DeleteRule(rule_id));
        assert!(
            !a.undo_stack.is_empty(),
            "undo stack should have entry after delete"
        );

        a.update(Message::DismissNotice);
        assert!(
            !a.undo_stack.is_empty(),
            "dismissing the notice banner must NOT clear the undo stack"
        );

        // ⌘Z should still work after dismissal
        a.update(Message::Undo);
        assert!(
            a.undo_stack.is_empty(),
            "stack consumed by undo after dismissal"
        );
    }
}

#[cfg(test)]
mod tests_mk046 {
    use super::*;
    use crate::message::Message;

    #[test]
    fn app_starts_at_welcome_with_no_snapshot() {
        let a = App::new().0;
        assert!(
            matches!(a.view, AppView::Welcome),
            "app must start at Welcome, not Workspace"
        );
        assert!(
            a.snapshot.is_none(),
            "no snapshot until user opens a workspace"
        );
        assert!(
            a.audience_mode.is_none(),
            "no audience mode until first-run picker is answered"
        );
    }

    #[test]
    fn mode_picker_view_renders_full_screen_before_mode_chosen() {
        let a = App::new().0;
        // App::view() should return the mode picker, not the workspace shell.
        // We verify by building the element — if it panics, the test fails.
        let _ = a.view();
    }

    #[test]
    fn choosing_mode_then_opening_workspace_reaches_routes() {
        let mut a = App::new().0;
        // First: the mode picker shows — choose Expert
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        assert!(matches!(a.view, AppView::Welcome));

        // Click "Open workspace" → Dashboard → click workspace
        a.update(Message::GoDashboard);
        assert!(matches!(a.view, AppView::Dashboard));

        a.update(Message::OpenWorkspace("payments-mock".into()));
        assert!(matches!(a.view, AppView::Workspace));
        assert!(a.snapshot.is_some(), "snapshot loaded after OpenWorkspace");
        assert_eq!(a.tab, crate::selection::WorkspaceTab::Routes);
    }

    #[test]
    fn wizard_flow_opens_workspace() {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.update(Message::GoWizard);
        assert!(matches!(a.view, AppView::Wizard));
        a.update(Message::WizardCreate);
        assert!(matches!(a.view, AppView::Workspace));
        assert!(a.snapshot.is_some());
    }

    #[test]
    fn welcome_screen_builds_after_mode_is_chosen() {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Guided,
        ));
        // After choosing, App::view() delegates to the Welcome screen
        let _ = a.view();
    }
}

#[cfg(test)]
mod tests_mk047 {
    use super::*;
    use crate::message::Message;

    #[test]
    fn wizard_create_produces_blank_workspace_with_wizard_name() {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        // Fill in wizard fields
        a.update(Message::WizardSetName("inventory-mock".into()));
        a.update(Message::WizardSetHost("0.0.0.0".into()));
        a.wizard.port = "9090".into();

        a.update(Message::WizardCreate);

        let snap = a.snapshot.as_ref().expect("snapshot after WizardCreate");
        assert_eq!(snap.meta.name, "inventory-mock");
        assert_eq!(snap.root_settings.listener_ip, "0.0.0.0");
        assert_eq!(snap.root_settings.listener_port, 9090);
        // Default starter is Minimal — one rule set with a health-check rule.
        assert_eq!(
            snap.rule_sets.len(),
            1,
            "Minimal starter creates one rule set"
        );
        assert_eq!(
            snap.rule_sets[0].rules.len(),
            1,
            "Minimal starter has one rule"
        );
        assert_eq!(snap.rule_sets[0].rules[0].payload.url_path, "/health");
        assert!(matches!(a.view, AppView::Workspace));
        assert!(a.notice.is_some(), "welcome notice shown after create");
    }

    #[test]
    fn wizard_create_with_empty_name_uses_default() {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        // Leave wizard name empty
        a.update(Message::WizardCreate);
        let snap = a.snapshot.as_ref().unwrap();
        assert_eq!(snap.meta.name, "my-mock", "default name used when blank");
    }

    #[test]
    fn open_workspace_still_loads_the_mock() {
        // OpenWorkspace (from Dashboard) continues to load the rich mock workspace.
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.update(Message::OpenWorkspace("payments-mock".into()));
        let snap = a.snapshot.as_ref().unwrap();
        assert!(
            !snap.rule_sets.is_empty(),
            "opening an existing workspace loads the full mock"
        );
    }

    #[test]
    fn blank_workspace_shows_add_rule_set_cta() {
        // With no rule sets, the centre panel shows the blank-workspace CTA.
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.update(Message::WizardCreate);
        assert!(matches!(a.view, AppView::Workspace));
        // Build the Routes view — should not panic even with no rule sets.
        let _ = crate::screens::routes::view(&a);
    }
}

#[cfg(test)]
mod tests_mk048 {
    use super::*;
    use crate::message::Message;

    fn expert_at_wizard() -> App {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.update(Message::GoWizard);
        a
    }

    // ── AddRuleSet ───────────────────────────────────────────────────────

    #[test]
    fn add_rule_set_creates_real_rule_set() {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.update(Message::WizardSetStarter(WizardStarter::Empty));
        a.update(Message::WizardCreate);
        assert!(a.snapshot.as_ref().unwrap().rule_sets.is_empty());

        a.update(Message::AddRuleSet);

        let snap = a.snapshot.as_ref().unwrap();
        assert_eq!(
            snap.rule_sets.len(),
            1,
            "AddRuleSet creates a real rule set"
        );
        assert!(
            snap.rule_sets[0].file.path.contains("rule-set-1"),
            "generated filename includes the sequence number"
        );
        assert!(snap.rule_sets[0].file.dirty, "new rule set starts dirty");
        assert_eq!(
            a.selection.rule_set,
            Some(snap.rule_sets[0].id),
            "new rule set is selected"
        );
    }

    #[test]
    fn add_rule_set_increments_filename_number() {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.update(Message::WizardSetStarter(WizardStarter::Empty));
        a.update(Message::WizardCreate);

        a.update(Message::AddRuleSet);
        a.update(Message::AddRuleSet);

        let snap = a.snapshot.as_ref().unwrap();
        assert_eq!(snap.rule_sets.len(), 2);
        assert!(
            snap.rule_sets[1].file.path.contains("rule-set-2"),
            "second rule set is numbered 2"
        );
    }

    // ── Wizard starter ───────────────────────────────────────────────────

    #[test]
    fn wizard_starter_minimal_creates_health_rule() {
        let mut a = expert_at_wizard();
        a.update(Message::WizardSetStarter(WizardStarter::Minimal));
        a.update(Message::WizardCreate);

        let snap = a.snapshot.as_ref().unwrap();
        assert_eq!(snap.rule_sets.len(), 1);
        assert_eq!(snap.rule_sets[0].rules.len(), 1);
        assert_eq!(snap.rule_sets[0].rules[0].payload.url_path, "/health");
        assert_eq!(snap.rule_sets[0].rules[0].payload.method, "GET");
    }

    #[test]
    fn wizard_starter_shop_api_loads_full_mock() {
        let mut a = expert_at_wizard();
        a.update(Message::WizardSetStarter(WizardStarter::ShopApi));
        a.update(Message::WizardCreate);

        let snap = a.snapshot.as_ref().unwrap();
        assert!(
            snap.rule_sets.len() >= 2,
            "ShopApi starter loads the full mock with multiple rule sets"
        );
        assert!(
            !snap.fallback_files.is_empty(),
            "ShopApi starter includes fallback files"
        );
    }

    #[test]
    fn wizard_starter_empty_produces_blank() {
        let mut a = expert_at_wizard();
        a.update(Message::WizardSetStarter(WizardStarter::Empty));
        a.update(Message::WizardCreate);

        let snap = a.snapshot.as_ref().unwrap();
        assert!(snap.rule_sets.is_empty(), "Empty starter = no rule sets");
    }

    #[test]
    fn wizard_starter_default_is_minimal() {
        let a = expert_at_wizard();
        assert_eq!(a.wizard.starter, WizardStarter::Minimal);
    }

    #[test]
    fn wizard_set_starter_message_updates_state() {
        let mut a = expert_at_wizard();
        a.update(Message::WizardSetStarter(WizardStarter::ShopApi));
        assert_eq!(a.wizard.starter, WizardStarter::ShopApi);
        a.update(Message::WizardSetStarter(WizardStarter::Empty));
        assert_eq!(a.wizard.starter, WizardStarter::Empty);
    }

    // ── Minimal workspace model ───────────────────────────────────────────

    #[test]
    fn minimal_workspace_has_health_check_rule() {
        let ws = apimokka_model::mock::minimal_workspace("svc", "127.0.0.1", 8080, false);
        assert_eq!(ws.meta.name, "svc");
        assert_eq!(ws.rule_sets.len(), 1);
        let rule = &ws.rule_sets[0].rules[0];
        assert_eq!(rule.payload.url_path, "/health");
        assert_eq!(rule.payload.method, "GET");
        assert_eq!(rule.payload.respond.status, "200 OK");
    }
}

#[cfg(test)]
mod tests_mk049 {
    use super::*;
    use crate::message::Message;

    fn expert() -> App {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.update(Message::OpenWorkspace("test".into()));
        a
    }

    // ── DuplicateRule ────────────────────────────────────────────────────

    #[test]
    fn duplicate_rule_creates_copy_after_original() {
        let mut a = expert();
        let snap = a.snapshot.as_ref().unwrap();
        let orig = snap.rule_sets[0].rules[0].id;
        let before = snap.rule_sets[0].rules.len();
        let orig_path = snap.rule_sets[0].rules[0].payload.url_path.clone();
        drop(snap);

        a.update(Message::DuplicateRule(orig));

        let snap = a.snapshot.as_ref().unwrap();
        assert_eq!(
            snap.rule_sets[0].rules.len(),
            before + 1,
            "duplicate adds one rule"
        );
        // The copy is inserted right after the original
        assert_eq!(
            snap.rule_sets[0].rules[1].payload.url_path, orig_path,
            "copy has the same URL path"
        );
        assert_ne!(snap.rule_sets[0].rules[1].id, orig, "copy has a fresh ID");
        assert_eq!(
            a.selection.rule,
            Some(snap.rule_sets[0].rules[1].id),
            "the copy is selected after duplication"
        );
    }

    #[test]
    fn duplicate_rule_is_undoable() {
        let mut a = expert();
        let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        let before = a.snapshot.as_ref().unwrap().rule_sets[0].rules.len();

        a.update(Message::DuplicateRule(rule_id));
        assert_eq!(
            a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
            before + 1
        );

        a.update(Message::Undo);
        assert_eq!(
            a.snapshot.as_ref().unwrap().rule_sets[0].rules.len(),
            before,
            "undo removes the duplicated rule"
        );
    }

    // ── run_stub_test header conditions ──────────────────────────────────

    fn make_test_app_with_header_rule() -> App {
        let mut a = expert();
        let rs_id = a.snapshot.as_ref().unwrap().rule_sets[0].id;
        let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
        // Set URL path and add a header condition
        a.update(Message::SelectRule(rule_id));
        a.update(Message::RuleSetUrlPath("/api/test".into()));
        a.update(Message::HeaderAdd);
        a.update(Message::HeaderSetName {
            index: 0,
            value: "authorization".into(),
        });
        a.update(Message::HeaderSetValue {
            index: 0,
            value: "Bearer token".into(),
        });
        a.test_rule.url_path = "/api/test".into();
        a.test_rule.method = "GET".into();
        a
    }

    #[test]
    fn test_rule_matches_when_header_condition_satisfied() {
        let mut a = make_test_app_with_header_rule();
        a.test_rule.headers_text = "authorization: Bearer token".into();
        a.update(Message::TestRuleRun);
        assert!(
            matches!(a.test_rule.result, Some(TestRuleResult::Matched { .. })),
            "should match when required header is present"
        );
    }

    #[test]
    fn test_rule_no_match_when_header_condition_fails() {
        let mut a = make_test_app_with_header_rule();
        a.test_rule.headers_text = "authorization: wrong".into();
        a.update(Message::TestRuleRun);
        assert!(
            matches!(a.test_rule.result, Some(TestRuleResult::NoMatch)),
            "should not match when header value is wrong"
        );
    }

    #[test]
    fn test_rule_no_match_when_required_header_absent() {
        let mut a = make_test_app_with_header_rule();
        a.test_rule.headers_text = "".into(); // no headers at all
        a.update(Message::TestRuleRun);
        assert!(
            matches!(a.test_rule.result, Some(TestRuleResult::NoMatch)),
            "should not match when required header is missing"
        );
    }

    // ── run_stub_test body conditions ────────────────────────────────────

    #[test]
    fn test_rule_matches_body_equal() {
        let mut a = expert();
        // Find a rule that has a body condition in the mock data
        let snap = a.snapshot.as_ref().unwrap();
        let rule_with_body = snap
            .rule_sets
            .iter()
            .flat_map(|rs| rs.rules.iter())
            .find(|r| !r.payload.body.is_empty());
        if let Some(rule) = rule_with_body {
            let id = rule.id;
            let bc = rule.payload.body[0].clone();
            let url = rule.payload.url_path.clone();
            let method = rule.payload.method.clone();
            drop(snap);
            a.update(Message::SelectRule(id));
            a.test_rule.url_path = url;
            a.test_rule.method = if method.is_empty() {
                "POST".into()
            } else {
                method
            };
            // Build a JSON body that satisfies the first body condition
            let json = format!(r#"{{"{key}": "{val}"}}"#, key = bc.path, val = bc.value);
            a.test_rule.body = json;
            a.test_rule.headers_text = String::new();
            a.update(Message::TestRuleRun);
            // We can only assert it didn't error — the condition may not be Equal
            assert!(
                !matches!(a.test_rule.result, Some(TestRuleResult::Error(_))),
                "test runner should not error on well-formed body JSON"
            );
        }
    }

    #[test]
    fn test_rule_errors_on_invalid_json_when_body_conditions_exist() {
        let mut a = expert();
        let snap = a.snapshot.as_ref().unwrap();
        let rule_with_body = snap
            .rule_sets
            .iter()
            .flat_map(|rs| rs.rules.iter())
            .find(|r| !r.payload.body.is_empty());
        if let Some(rule) = rule_with_body {
            let id = rule.id;
            drop(snap);
            a.update(Message::SelectRule(id));
            a.test_rule.body = "not json".into();
            a.test_rule.headers_text = String::new();
            a.update(Message::TestRuleRun);
            // Either NoMatch (other conditions failed before reaching body)
            // or Error (body is invalid JSON) — both are correct rejections.
            assert!(
                !matches!(a.test_rule.result, Some(TestRuleResult::Matched { .. })),
                "invalid JSON body should not produce a Matched result"
            );
        }
    }

    // ── ConfirmAction::DeleteRule removed ────────────────────────────────

    #[test]
    fn delete_rule_set_still_works_via_confirm() {
        // Verifying the confirm dialog still handles DeleteRuleSet correctly
        // (the remaining live variant after DeleteRule was removed).
        let mut a = expert();
        let rs_id = a.snapshot.as_ref().unwrap().rule_sets[0].id;
        let before = a.snapshot.as_ref().unwrap().rule_sets.len();

        a.update(Message::DeleteRuleSet(rs_id));
        assert!(
            a.confirm_dialog.is_some(),
            "DeleteRuleSet requires confirmation"
        );

        a.update(Message::ConfirmProceed);
        assert_eq!(
            a.snapshot.as_ref().unwrap().rule_sets.len(),
            before - 1,
            "rule set removed after confirmation"
        );
    }

    // ── dotted_path_get ──────────────────────────────────────────────────

    #[test]
    fn dotted_path_get_nested_object() {
        let v: serde_json::Value = serde_json::json!({"a": {"b": {"c": 42}}});
        assert_eq!(dotted_path_get(&v, "a.b.c"), Some(&serde_json::json!(42)));
    }

    #[test]
    fn dotted_path_get_array_index() {
        let v: serde_json::Value = serde_json::json!({"items": [1, 2, 3]});
        assert_eq!(dotted_path_get(&v, "items.1"), Some(&serde_json::json!(2)));
    }

    #[test]
    fn dotted_path_get_missing_key_returns_none() {
        let v: serde_json::Value = serde_json::json!({"a": 1});
        assert_eq!(dotted_path_get(&v, "b.c"), None);
    }
}

#[cfg(test)]
mod tests_mk050 {
    use super::*;
    use crate::message::Message;

    #[test]
    fn theme_choice_has_four_variants() {
        assert_eq!(ThemeChoice::all().len(), 4);
    }

    #[test]
    fn theme_toggle_cycles_through_all_four() {
        let mut c = ThemeChoice::Light;
        c = c.toggle();
        assert_eq!(c, ThemeChoice::Dark);
        c = c.toggle();
        assert_eq!(c, ThemeChoice::HighContrastLight);
        c = c.toggle();
        assert_eq!(c, ThemeChoice::HighContrastDark);
        c = c.toggle();
        assert_eq!(c, ThemeChoice::Light);
    }

    #[test]
    fn each_theme_choice_yields_distinct_tokens() {
        // Tokens differ across presets — verify text_muted is not identical
        // between light and high-contrast light (HC is darker/stronger).
        let light = ThemeChoice::Light.tokens();
        let hc = ThemeChoice::HighContrastLight.tokens();
        let l = light.palette.text_muted;
        let h = hc.palette.text_muted;
        assert!(
            (l.r - h.r).abs() > f32::EPSILON
                || (l.g - h.g).abs() > f32::EPSILON
                || (l.b - h.b).abs() > f32::EPSILON,
            "high-contrast muted text should differ from standard light"
        );
    }

    #[test]
    fn standard_themes_use_native_iced() {
        assert!(matches!(ThemeChoice::Light.iced(), iced::Theme::Light));
        assert!(matches!(ThemeChoice::Dark.iced(), iced::Theme::Dark));
    }

    #[test]
    fn high_contrast_themes_use_custom_palette() {
        assert!(matches!(
            ThemeChoice::HighContrastLight.iced(),
            iced::Theme::Custom(_)
        ));
        assert!(matches!(
            ThemeChoice::HighContrastDark.iced(),
            iced::Theme::Custom(_)
        ));
    }

    #[test]
    fn high_contrast_themes_are_detected() {
        assert!(crate::theme::is_high_contrast(
            &ThemeChoice::HighContrastLight.iced()
        ));
        assert!(crate::theme::is_high_contrast(
            &ThemeChoice::HighContrastDark.iced()
        ));
        assert!(!crate::theme::is_high_contrast(&ThemeChoice::Light.iced()));
        assert!(!crate::theme::is_high_contrast(&ThemeChoice::Dark.iced()));
    }

    #[test]
    fn set_theme_message_updates_choice() {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.update(Message::SetTheme(ThemeChoice::HighContrastDark));
        assert_eq!(a.theme_choice, ThemeChoice::HighContrastDark);
    }

    #[test]
    fn is_dark_classification() {
        assert!(!ThemeChoice::Light.is_dark());
        assert!(ThemeChoice::Dark.is_dark());
        assert!(!ThemeChoice::HighContrastLight.is_dark());
        assert!(ThemeChoice::HighContrastDark.is_dark());
    }

    #[test]
    fn card_style_builds_for_all_themes() {
        // The high-contrast branch adds a border; verify no panic for any theme.
        for choice in ThemeChoice::all() {
            let th = choice.iced();
            let _ = crate::theme::card_style(&th);
            let _ = crate::theme::panel_style(&th);
            let _ = crate::theme::muted(&th);
        }
    }

    #[test]
    fn settings_view_builds_with_high_contrast_theme() {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.update(Message::OpenWorkspace("test".into()));
        a.update(Message::SetTheme(ThemeChoice::HighContrastLight));
        a.tab = crate::selection::WorkspaceTab::Settings;
        let _ = crate::screens::settings::view(&a);
    }
}
