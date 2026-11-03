//! MK-034 — O-01 Workspace menu (snora header_menu slot).
use crate::app::App;
use crate::message::Message;
use crate::regions::{Cursor, Region, WorkspaceMenuField};
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use apimokka_model::mock;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Element, Length, Padding};

/// Same shape as `rule_editor.rs`'s own `cursor_wrap`, for `Open`/`Create`.
fn cursor_wrap<'a>(
    app: &App,
    field: WorkspaceMenuField,
    el: Element<'a, Message>,
) -> Element<'a, Message> {
    let has_cursor = app.focused_region == Some(Region::WorkspaceMenu(field))
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

/// Same shape as `rule_editor.rs`'s own `cursor_wrap_group`, for the
/// recent-workspace rows.
fn cursor_wrap_group<'a>(
    app: &App,
    field: WorkspaceMenuField,
    index: usize,
    el: Element<'a, Message>,
) -> Element<'a, Message> {
    let has_cursor = app.focused_region == Some(Region::WorkspaceMenu(field))
        && app.region_cursor == Some(Cursor::Index(index));
    container(el)
        .padding(space::S1)
        .style(if has_cursor {
            theme::cursor_style
        } else {
            |_: &iced::Theme| iced::widget::container::Style::default()
        })
        .into()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let ws_name = app
        .snapshot
        .as_ref()
        .map(|s| s.identity.name.as_str())
        .unwrap_or("—");
    let ws_path = app
        .snapshot
        .as_ref()
        .map(|s| s.identity.path.as_str())
        .unwrap_or("");

    let current = container(
        column![
            text(app.t(Key::WorkspaceMenuCurrent))
                .size(size::LABEL)
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
        .enumerate()
        .map(|(i, ws)| {
            let name = ws.name.clone();
            let is_cur = app
                .snapshot
                .as_ref()
                .map(|s| s.identity.name == ws.name)
                .unwrap_or(false);
            let btn = button(
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
            .width(Length::Fill);
            cursor_wrap_group(app, WorkspaceMenuField::Recent, i, btn.into())
        })
        .collect();

    let footer = container(
        row![
            cursor_wrap(
                app,
                WorkspaceMenuField::Open,
                button(text(app.t(Key::WorkspaceMenuOpen)).size(size::BODY))
                    .on_press(Message::GoDashboard)
                    .padding(Padding::from([space::S2, space::S4]))
                    .into(),
            ),
            cursor_wrap(
                app,
                WorkspaceMenuField::Create,
                button(text(app.t(Key::WorkspaceMenuCreate)).size(size::BODY))
                    .on_press(Message::GoWizard)
                    .padding(Padding::from([space::S2, space::S4]))
                    .into(),
            ),
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
