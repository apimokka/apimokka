//! MK-025 — S-01 Dashboard.
use crate::app::App;
use crate::message::Message;
use crate::regions::{Cursor, Region, dashboard_filtered_workspaces};
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use iced::widget::{
    Space, button, column, container, mouse_area, row, scrollable, text, text_input,
};
use iced::{Alignment, Element, Length, Padding};

/// Task 028 §3.1: the `Id` `App::update_and_dispatch` focuses when
/// `DashboardSearch` becomes the focused region, and unfocuses when it stops
/// being one.
pub const SEARCH_INPUT_ID: &str = "mk023-dashboard-search";

/// Task 028 §3.3: the `Id` `App::update_and_dispatch` scrolls, via the same
/// linear relative-offset approximation D-5 established for the command
/// palette's results list.
pub const WORKSPACES_SCROLLABLE_ID: &str = "mk023-dashboard-workspaces";

pub fn view(app: &App) -> Element<'_, Message> {
    let create_has_cursor = app.focused_region == Some(Region::DashboardCreate)
        && app.region_cursor == Some(Cursor::Index(0));
    let header = row![
        text(app.t(Key::DashTitle))
            .size(size::TITLE)
            .width(Length::Fill),
        // `space::S1` gives `cursor_style`'s border room to show — without
        // it the button's own opaque background (rendered on top, as its
        // child) paints straight over the wrapping container's border.
        container(widgets::primary_btn(
            app.t(Key::BtnCreateWorkspace),
            Message::GoWizard
        ))
        .padding(space::S1)
        .style(if create_has_cursor {
            theme::cursor_style
        } else {
            |_: &iced::Theme| iced::widget::container::Style::default()
        }),
    ]
    .align_y(Alignment::Center);

    // §3.1: the field's own focus ring (`theme::text_input_style`) is its
    // cursor indicator — no extra wrapper is drawn around it, unlike the
    // button regions, since a focused native `text_input` already renders
    // one.
    // D-18 (task 032): `on_release`, not `on_press` — a focused `text_input`
    // captures `ButtonPressed` itself (positioning its native cursor), but
    // not `ButtonReleased`, so `on_release` is the one mouse-area hook that
    // still fires for a click landing on the field, confirmed at
    // `iced_widget::text_input`'s own source rather than assumed.
    let search: Element<Message> = mouse_area(
        text_input(app.t(Key::DashSearchPlaceholder), &app.dash_search)
            .id(SEARCH_INPUT_ID)
            .on_input(Message::DashSearch)
            .style(theme::text_input_style)
            .size(size::BODY)
            .padding(Padding::from([space::S2, space::S3]))
            .width(Length::Fill),
    )
    .on_release(Message::RegionClicked(Region::DashboardSearch))
    .into();

    // The one source this view and `Region::DashboardWorkspaces`'s cursor
    // logic both read (the D-9 lesson) — see `regions.rs`.
    let workspaces = dashboard_filtered_workspaces(app);

    let rows: Vec<Element<Message>> = workspaces
        .into_iter()
        .map(|ws| {
            let has_cursor = app.focused_region == Some(Region::DashboardWorkspaces)
                && app.region_cursor == Some(Cursor::WorkspaceName(ws.name.clone()));
            workspace_row(app, ws.name, ws.path, ws.last_opened, ws.pinned, has_cursor)
        })
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
        scrollable(content)
            .id(WORKSPACES_SCROLLABLE_ID)
            .height(Length::Fill),
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
    has_cursor: bool,
) -> Element<'static, Message> {
    let display_name = name.clone();
    let row_msg = Message::OpenWorkspace(name.clone());

    let btn = button(
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
    .width(iced::Length::Fill);

    // Same `space::S1` gap as the header's Create button — `card_style`'s
    // own opaque surface would otherwise paint over the cursor border.
    container(btn)
        .padding(space::S1)
        .style(if has_cursor {
            theme::cursor_style
        } else {
            |_: &iced::Theme| iced::widget::container::Style::default()
        })
        .width(iced::Length::Fill)
        .into()
}
