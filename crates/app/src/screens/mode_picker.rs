//! MK-040 / MK-046 — First-run audience mode picker.
//!
//! Shown full-screen when `App::audience_mode` is `None`. The user must
//! choose Guided or Expert before anything else is rendered. The choice is
//! reversible in Settings at any time.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length, Padding};
use apimokka_i18n::Key;
use apimokka_model::AudienceMode;

use crate::app::App;
use crate::message::Message;
use crate::theme::{self, pad, size, space};

pub fn view(app: &App) -> Element<'_, Message> {
    let card = |title: Key, desc: Key, mode: AudienceMode| -> Element<Message> {
        button(
            container(
                column![
                    text(app.t(title)).size(size::SECTION),
                    text(app.t(desc)).size(size::BODY)
                        .color(theme::muted(&app.theme())),
                ]
                .spacing(space::S2),
            )
            .padding(Padding::from(pad::CARD))
            .style(theme::card_style)
            .width(Length::Fill),
        )
        .on_press(Message::ChooseAudienceMode(mode))
        .padding(0)
        .style(theme::naked)
        .width(Length::Fill)
        .into()
    };

    let picker = container(
        column![
            text(app.t(Key::AppName)).size(size::DISPLAY)
                .color(theme::muted(&app.theme())),
            Space::new().height(space::S2),
            text(app.t(Key::ModePickerTitle)).size(size::TITLE),
            Space::new().height(space::S3),
            card(Key::ModeGuidedTitle, Key::ModeGuidedDesc, AudienceMode::Guided),
            Space::new().height(space::S2),
            card(Key::ModeExpertTitle, Key::ModeExpertDesc, AudienceMode::Expert),
            Space::new().height(space::S3),
            row![
                text(app.t(Key::ModePickerHint)).size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
            ],
        ]
        .spacing(0)
        .width(Length::Fixed(500.0)),
    )
    .padding(Padding::from(pad::CARD))
    .style(theme::dialog_style);

    // Center the picker card on a plain background
    container(picker)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
