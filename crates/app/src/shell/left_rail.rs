//! MK-027 — Left rail (4 destinations).
use crate::app::App;
use crate::message::Message;
use crate::selection::WorkspaceTab;
use crate::theme::{self, pad, size, space};
use apimokka_i18n::Key;
use iced::widget::{button, column, container, text};
use iced::{Element, Length, Padding};

#[allow(dead_code)]
pub fn view(app: &App) -> Element<'_, Message> {
    let items = [
        (WorkspaceTab::Routes, Key::NavRoutes, "⎘"),
        (WorkspaceTab::Trace, Key::NavTrace, "∿"),
        (WorkspaceTab::Settings, Key::NavSettings, "⊞"),
    ];

    let mut col = column![]
        .spacing(space::S1)
        .padding(Padding::from([space::S3, space::S2]));
    for (tab, label_key, _glyph) in items {
        let active = app.tab == tab;
        let label = app.t(label_key);
        let content = container(text(label).size(size::BODY))
            .padding(Padding::from(pad::RAIL_ITEM))
            .width(Length::Fill);

        let btn = button(if active {
            container(content)
                .style(theme::rail_selected_style)
                .width(Length::Fill)
        } else {
            container(content).width(Length::Fill)
        })
        .on_press(Message::SwitchTab(tab))
        .padding(0)
        .style(theme::naked)
        .style(theme::naked)
        .width(Length::Fill);

        col = col.push(btn);
    }

    container(col)
        .width(Length::Fixed(120.0))
        .height(Length::Fill)
        .style(theme::panel_style)
        .into()
}
