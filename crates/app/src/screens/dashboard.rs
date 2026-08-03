//! MK-025 — S-01 Dashboard.
use crate::app::App;
use crate::message::Message;
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use apimokka_model::mock;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Padding};

pub fn view(app: &App) -> Element<'_, Message> {
    let header = row![
        text(app.t(Key::DashTitle))
            .size(size::TITLE)
            .width(Length::Fill),
        widgets::primary_btn(app.t(Key::BtnCreateWorkspace), Message::GoWizard),
    ]
    .align_y(Alignment::Center);

    let search = text_input(app.t(Key::DashSearchPlaceholder), &app.dash_search)
        .on_input(Message::DashSearch)
        .size(size::BODY)
        .padding(Padding::from([space::S2, space::S3]))
        .width(Length::Fill);

    let q = app.dash_search.to_lowercase();
    let workspaces: Vec<_> = mock::recent_workspaces()
        .into_iter()
        .filter(|ws| {
            q.is_empty()
                || ws.name.to_lowercase().contains(&q)
                || ws.path.to_lowercase().contains(&q)
        })
        .collect();

    let rows: Vec<Element<Message>> = workspaces
        .into_iter()
        .map(|ws| workspace_row(app, ws.name, ws.path, ws.last_opened, ws.pinned))
        .collect();

    let content = if rows.is_empty() {
        widgets::empty_state(app.t(Key::DashNoWorkspacesFound))
    } else {
        column(rows).spacing(space::S2).into()
    };

    let page = column![
        header,
        search,
        Space::new().height(space::S4),
        scrollable(content).height(Length::Fill),
    ]
    .spacing(space::S4)
    .padding(Padding::from([space::S6, space::S6]));

    container(page)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn workspace_row(
    app: &App,
    name: String,
    path: String,
    last: String,
    pinned: bool,
) -> Element<'static, Message> {
    let display_name = name.clone();
    let _open_msg = Message::OpenWorkspace(name.clone());
    let row_msg = Message::OpenWorkspace(name.clone());

    button(
        container(
            iced::widget::row![
                iced::widget::column![
                    iced::widget::text(display_name).size(size::BODY),
                    iced::widget::text(path)
                        .size(size::CAPTION)
                        .color(theme::muted(&app.theme())),
                    iced::widget::text(format!("{}: {}", app.t(Key::DashLastOpened), last))
                        .size(size::CAPTION)
                        .color(theme::muted(&app.theme())),
                ]
                .spacing(space::S1)
                .width(iced::Length::Fill),
                iced::widget::text(if pinned { "📌" } else { "" }).size(size::BODY),
            ]
            .spacing(space::S3)
            .align_y(iced::Alignment::Center),
        )
        .padding(Padding::from([space::S3, space::S4]))
        .style(theme::card_style)
        .width(iced::Length::Fill),
    )
    .on_press(row_msg)
    .padding(0)
    .style(theme::naked)
    .style(theme::naked)
    .width(iced::Length::Fill)
    .into()
}
