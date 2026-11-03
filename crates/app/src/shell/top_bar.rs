//! MK-027 — Top bar.
//!
//! Follows the "less is more" principle: only workspace identity, server/save
//! status, and server action buttons. Theme, locale, and command palette are
//! in Settings (they are rare settings, not constant workflow controls).
//! ⌘K keyboard shortcut still works via the keyboard subscription.

use crate::app::{App, RuntimeAction};
use crate::message::Message;
use crate::regions::{Cursor, Region, TopBarField};
use crate::theme::{self, pad, size, space};
use apimokka_i18n::Key;
use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Element, Length, Padding};

/// Same shape as `rule_editor.rs`'s own `cursor_wrap` (task 034). All four
/// of this bar's controls are one-item regions.
fn cursor_wrap<'a>(
    app: &App,
    field: TopBarField,
    el: Element<'a, Message>,
) -> Element<'a, Message> {
    let has_cursor = app.focused_region == Some(Region::TopBar(field))
        && app.region_cursor == Some(Cursor::Index(0));
    container(el)
        .padding(space::S1)
        .style(if has_cursor {
            theme::cursor_style
        } else {
            |_: &iced::Theme| iced::widget::container::Style::default()
        })
        .into()
}

/// The Reload/Restart action, if the current runtime phase has one
/// available right now — the one source both `view` (below) and
/// `regions::top_bar_fields`/`activation_message` read, so a disabled or
/// absent phase can never be reachable from one and not the other.
pub(crate) fn reload_or_restart(app: &App) -> Option<(Key, Message)> {
    match app.runtime_phase() {
        apimokka_model::RuntimeEffect::Reload
            if app.runtime_request_available(RuntimeAction::Reload) =>
        {
            Some((Key::BtnReload, Message::ReloadConfig))
        }
        apimokka_model::RuntimeEffect::Restart
            if app.runtime_request_available(RuntimeAction::Restart) =>
        {
            Some((Key::BtnRestart, Message::RestartServer))
        }
        _ => None,
    }
}

/// The Start/Stop server action's own message, gated the same way
/// `view`'s own `server_btn` is — `None` when it is not actionable right
/// now, the same "present always, inert when not applicable" shape the
/// fallback editor's `Revert`/`Save` already established.
pub(crate) fn start_stop_message(app: &App) -> Option<Message> {
    let server_action = if app.server_state == ServerState::Running {
        RuntimeAction::Stop
    } else {
        RuntimeAction::Start
    };
    app.runtime_request_available(server_action)
        .then_some(Message::StartStopServer)
}

/// MK-035 server state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum ServerState {
    #[default]
    Stopped,
    Starting,
    Running,
    Error,
}
impl ServerState {
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Stopped => "■",
            Self::Starting => "◔",
            Self::Running => "●",
            Self::Error => "!",
        }
    }
    pub fn label(self, app: &App) -> &'static str {
        match self {
            Self::Stopped => app.t(Key::StatusStopped),
            Self::Starting => app.t(Key::StatusStarting),
            Self::Running => app.t(Key::StatusRunning),
            Self::Error => app.t(Key::StatusError),
        }
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    // ── Identity: app name (label) + workspace name (button) ─────────────
    // These are separate: app name is static identity; workspace name opens
    // the workspace switcher menu.
    let app_label = text(app.t(Key::AppName)).size(size::SECTION);

    let ws_section: Element<Message> = if let Some(snap) = &app.snapshot {
        let chevron = if app.workspace_menu_open {
            "▲"
        } else {
            "▼"
        };
        let ws_btn = cursor_wrap(
            app,
            TopBarField::Switcher,
            button(
                row![
                    text(snap.identity.name.as_str()).size(size::SECTION),
                    text(chevron)
                        .size(size::LABEL)
                        .color(theme::muted(&app.theme())),
                ]
                .spacing(space::S1)
                .align_y(Alignment::Center),
            )
            .on_press(Message::ToggleWorkspaceMenu)
            .padding(Padding::from([space::S1, space::S2]))
            .style(iced::widget::button::text)
            .into(),
        );
        row![
            text("·")
                .size(size::SECTION)
                .color(theme::muted(&app.theme())),
            ws_btn,
        ]
        .spacing(space::S2)
        .align_y(Alignment::Center)
        .into()
    } else {
        Space::new().width(Length::Fixed(0.0)).into()
    };

    // ── Status chips ──────────────────────────────────────────────────────
    let save_label = if app.dirty_count > 0 {
        format!("{} ({})", app.t(Key::StatusUnsaved), app.dirty_count)
    } else {
        app.t(Key::StatusSaved).to_string()
    };
    let save_glyph = if app.dirty_count > 0 { "●" } else { "✓" };
    let server_label = app.server_state.label(app).to_string();
    let server_chip = chip(app.server_state.glyph().to_string(), server_label);
    let save_chip = chip(save_glyph.to_string(), save_label);
    let phase = app.runtime_phase();
    let phase_chip: Element<'_, Message> = match phase {
        apimokka_model::RuntimeEffect::None => Space::new().width(Length::Fixed(0.0)).into(),
        apimokka_model::RuntimeEffect::Reload => chip(
            "↻".into(),
            app.t(if app.server_state == ServerState::Stopped {
                Key::StatusReloadOnStart
            } else {
                Key::StatusReloadPending
            })
            .into(),
        ),
        apimokka_model::RuntimeEffect::Restart => chip(
            "⏻".into(),
            app.t(if app.server_state == ServerState::Stopped {
                Key::StatusRestartOnStart
            } else {
                Key::StatusRestartRequired
            })
            .into(),
        ),
    };

    // ── Action buttons ────────────────────────────────────────────────────
    let save_btn = cursor_wrap(
        app,
        TopBarField::SaveAll,
        action_btn(app.t(Key::BtnSaveAll), Message::Save, app.dirty_count > 0),
    );
    let config_btn: Element<'_, Message> = match reload_or_restart(app) {
        Some((key, msg)) => cursor_wrap(
            app,
            TopBarField::ReloadOrRestart,
            action_btn(app.t(key), msg, true),
        ),
        None => Space::new().width(Length::Fixed(0.0)).into(),
    };
    let srv_label = match app.server_state {
        ServerState::Running => app.t(Key::BtnStopServer),
        _ => app.t(Key::BtnStartServer),
    };
    let server_btn = cursor_wrap(
        app,
        TopBarField::StartStop,
        action_btn(
            srv_label,
            Message::StartStopServer,
            start_stop_message(app).is_some(),
        ),
    );

    let bar = row![
        app_label,
        ws_section,
        Space::new().width(space::S4),
        server_chip,
        phase_chip,
        save_chip,
        Space::new().width(Length::Fill),
        save_btn,
        config_btn,
        server_btn,
    ]
    .spacing(space::S2)
    .align_y(Alignment::Center);

    container(bar)
        .width(Length::Fill)
        .padding(Padding::from([space::S3, space::S5]))
        .style(theme::panel_style)
        .into()
}

fn chip(glyph: String, label: String) -> Element<'static, Message> {
    container(
        row![text(glyph).size(size::LABEL), text(label).size(size::LABEL),]
            .spacing(space::S1)
            .align_y(Alignment::Center),
    )
    .padding(Padding::from([4.0, 10.0]))
    .style(theme::chip_style)
    .into()
}

fn action_btn<'a>(label: &'a str, msg: Message, enabled: bool) -> Element<'a, Message> {
    let b = button(text(label).size(size::LABEL)).padding(Padding::from(pad::BUTTON));
    if enabled {
        b.on_press(msg).into()
    } else {
        b.into()
    }
}
