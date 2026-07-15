//! Horizontal tab bar. Each tab occupies 1/N of the full width (equal columns).
//! Text is explicitly centered within each cell.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length, Padding};
use apimokka_i18n::Key;

use crate::app::App;
use crate::message::Message;
use crate::selection::WorkspaceTab;
use crate::theme::{self, size, space};
use crate::widgets;

pub fn view(app: &App) -> Element<'_, Message> {
    let tabs = [
        (WorkspaceTab::Routes,   Key::NavRoutes),
        (WorkspaceTab::Trace,    Key::NavTrace),
        (WorkspaceTab::Settings, Key::NavSettings),
    ];

    let tab_items: Vec<Element<Message>> = tabs.iter().map(|(tab, key)| {
        let active = app.tab == *tab;
        let label  = app.t(*key);

        // 3 px accent strip below active tab
        let indicator: Element<Message> = if active {
            container(Space::new().height(Length::Fixed(3.0)))
                .width(Length::Fill)
                .style(theme::accent_strip_style)
                .into()
        } else {
            Space::new().height(Length::Fixed(3.0)).into()
        };

        // Button: text_color adapts to active/inactive; width fills cell;
        // content is centered (iced button centers its children by default,
        // and center_x below makes the column also centered).
        let btn = button(
            container(text(label).size(size::BODY))
                .align_x(iced::alignment::Horizontal::Center)
                .width(Length::Fill),
        )
        .on_press(Message::SwitchTab(*tab))
        .padding(Padding::from([space::S3, space::S5]))
        .width(Length::Fill)
        .style(if active { theme::seg_active } else { theme::seg_inactive });

        container(
            column![btn, indicator].align_x(Alignment::Center),
        )
        .width(Length::FillPortion(1))
        .into()
    }).collect();

    container(
        column![
            row(tab_items),
            widgets::divider(),
        ]
        .spacing(0),
    )
    .width(Length::Fill)
    .style(theme::panel_style)
    .into()
}

