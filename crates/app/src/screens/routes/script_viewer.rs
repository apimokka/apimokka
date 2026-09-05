//! Middleware script viewer — read-only.

use crate::app::App;
use crate::message::Message;
use crate::theme::{self, pad, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Element, Length, Padding};

pub(super) fn script_viewer<'a>(
    app: &'a App,
    script: &'a apimokka_model::node::ConfigFileView,
) -> Element<'a, Message> {
    let name = script.path.rsplit('/').next().unwrap_or(&script.path);

    // Placeholder content — in production this would read the file from disk.
    let content = format!(
        "fn before_request(req) {{\n    // Edit this file in your preferred editor.\n    // Middleware scripts run before rule matching.\n    req\n}}\n\n// Path: {}",
        script.path
    );

    let header = column![
        row![
            text(name).size(size::SECTION).width(Length::Fill),
            container(text(app.t(Key::ScriptsReadOnlyBadge)).size(size::LABEL),)
                .padding(Padding::from([2.0, space::S2]))
                .style(theme::chip_style),
        ]
        .align_y(Alignment::Center),
        text(script.path.as_str())
            .size(size::CAPTION)
            .color(theme::muted(&app.theme())),
        text(app.t(Key::ScriptsEmptyExplanation))
            .size(size::BODY_SMALL)
            .line_height(theme::line_height::body_small())
            .color(theme::muted(&app.theme())),
    ]
    .spacing(space::S2);

    let code_card = container(
        scrollable(text(content).size(size::MONO).font(iced::Font::MONOSPACE)).height(Length::Fill),
    )
    .padding(Padding::from(pad::CARD))
    .style(theme::card_style)
    .width(Length::Fill)
    .height(Length::Fill);

    container(
        column![header, widgets::divider(), code_card,]
            .spacing(space::S4)
            .padding(Padding::from([space::S4, space::S5])),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
