//! MK-027 — Top bar.
//!
//! Follows the "less is more" principle: only workspace identity, server/save
//! status, and server action buttons. Theme, locale, and command palette are
//! in Settings (they are rare settings, not constant workflow controls).
//! ⌘K keyboard shortcut still works via the keyboard subscription.

use crate::app::App;
use crate::message::Message;
use crate::theme::{self, pad, size, space};
use apimokka_i18n::Key;
use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Element, Length, Padding};

/// MK-035 server state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum ServerState {
    #[default]
    Stopped,
    Starting,
    Running,
    ReloadPending,
    RestartRequired,
    Error,
}
impl ServerState {
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Stopped => "■",
            Self::Starting => "◔",
            Self::Running => "●",
            Self::ReloadPending => "↻",
            Self::RestartRequired => "⏻",
            Self::Error => "!",
        }
    }
    pub fn label(self, app: &App) -> &'static str {
        match self {
            Self::Stopped => app.t(Key::StatusStopped),
            Self::Starting => app.t(Key::StatusStarting),
            Self::Running => app.t(Key::StatusRunning),
            Self::ReloadPending => app.t(Key::StatusReloadPending),
            Self::RestartRequired => app.t(Key::StatusRestartRequired),
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
        let ws_btn = button(
            row![
                text(snap.meta.name.as_str()).size(size::SECTION),
                text(chevron)
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
            ]
            .spacing(space::S1)
            .align_y(Alignment::Center),
        )
        .on_press(Message::ToggleWorkspaceMenu)
        .padding(Padding::from([space::S1, space::S2]))
        .style(iced::widget::button::text);
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

    // ── Action buttons ────────────────────────────────────────────────────
    let save_btn = action_btn(app.t(Key::BtnSaveAll), Message::Save, app.dirty_count > 0);
    let reload_btn = action_btn(
        app.t(Key::BtnReload),
        Message::ReloadConfig,
        app.server_state == ServerState::ReloadPending,
    );
    let restart_btn = action_btn(
        app.t(Key::BtnRestart),
        Message::RestartServer,
        app.server_state == ServerState::RestartRequired,
    );
    let srv_label = match app.server_state {
        ServerState::Running | ServerState::ReloadPending | ServerState::RestartRequired => {
            app.t(Key::BtnStopServer)
        }
        _ => app.t(Key::BtnStartServer),
    };
    let server_btn = action_btn(srv_label, Message::StartStopServer, true);

    let bar = row![
        app_label,
        ws_section,
        Space::new().width(space::S4),
        server_chip,
        save_chip,
        Space::new().width(Length::Fill),
        save_btn,
        reload_btn,
        restart_btn,
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
        row![
            text(glyph).size(size::CAPTION),
            text(label).size(size::CAPTION),
        ]
        .spacing(space::S1)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([4.0, 10.0]))
    .style(theme::chip_style)
    .into()
}

fn action_btn<'a>(label: &'a str, msg: Message, enabled: bool) -> Element<'a, Message> {
    let b = button(text(label).size(size::CAPTION)).padding(Padding::from(pad::BUTTON));
    if enabled {
        b.on_press(msg).into()
    } else {
        b.into()
    }
}
