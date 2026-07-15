//! MK-034 — O-01 Workspace menu (snora header_menu slot).
use crate::app::App;
use crate::message::Message;
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use apimokka_model::mock;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Element, Length, Padding};

pub fn view(app: &App) -> Element<'_, Message> {
    let ws_name = app
        .snapshot
        .as_ref()
        .map(|s| s.meta.name.as_str())
        .unwrap_or("—");
    let ws_path = app
        .snapshot
        .as_ref()
        .map(|s| s.meta.path.as_str())
        .unwrap_or("");

    let current = container(
        column![
            text(app.t(Key::WorkspaceMenuCurrent))
                .size(size::CAPTION)
                .color(theme::muted(&app.theme())),
            text(ws_name).size(size::BODY),
            text(ws_path)
                .size(size::CAPTION)
                .color(theme::muted(&app.theme())),
        ]
        .spacing(space::S1),
    )
    .padding(Padding::from([space::S3, space::S4]));

    let recent_rows: Vec<Element<Message>> = mock::recent_workspaces()
        .into_iter()
        .map(|ws| {
            let name = ws.name.clone();
            let is_cur = app
                .snapshot
                .as_ref()
                .map(|s| s.meta.name == ws.name)
                .unwrap_or(false);
            button(
                container(
                    column![
                        text(ws.name).size(size::BODY),
                        text(format!("{} · {}", ws.path, ws.last_opened))
                            .size(size::CAPTION)
                            .color(theme::muted(&app.theme())),
                    ]
                    .spacing(2.0),
                )
                .padding(Padding::from([space::S2, space::S4]))
                .style(if is_cur {
                    theme::card_selected_style
                } else {
                    theme::card_style
                })
                .width(Length::Fill),
            )
            .on_press(Message::OpenWorkspace(name))
            .padding(0)
            .style(theme::naked)
            .style(theme::naked)
            .width(Length::Fill)
            .into()
        })
        .collect();

    let footer = container(
        row![
            button(text(app.t(Key::WorkspaceMenuOpen)).size(size::BODY))
                .on_press(Message::GoDashboard)
                .padding(Padding::from([space::S2, space::S4])),
            button(text(app.t(Key::WorkspaceMenuCreate)).size(size::BODY))
                .on_press(Message::GoWizard)
                .padding(Padding::from([space::S2, space::S4])),
            Space::new().width(Length::Fill),
        ]
        .spacing(space::S2),
    )
    .padding(Padding::from([space::S2, space::S3]))
    .style(theme::panel_style);

    container(
        column![
            current,
            widgets::divider(),
            scrollable(
                column(recent_rows)
                    .spacing(space::S1)
                    .padding(Padding::from([space::S1, space::S2]))
            )
            .height(Length::Shrink),
            widgets::divider(),
            footer,
        ]
        .spacing(0)
        .width(Length::Fixed(360.0)),
    )
    .style(theme::card_style)
    .into()
}
