//! MK-040 / MK-046 — First-run audience mode picker.
//!
//! Shown full-screen when `App::audience_mode` is `None`. The user must
//! choose Guided or Expert before anything else is rendered. The choice is
//! reversible in Settings at any time.
//!
//! MK-023's first-screen gap (task 014 §4): a keyboard-only user could not
//! reach this screen's cards at all before this table existed, since
//! `Message::ChooseAudienceMode` fired only from a button `.on_press`. `OPTIONS`
//! is the same table `view` (which card to render, and which one is
//! highlighted) and `App::update` (which `AudienceMode` arrow keys/Enter
//! select) both read — the same selected-row-plus-Enter idiom the command
//! palette uses, so the application has one keyboard interaction model
//! rather than two.

use apimokka_i18n::Key;
use apimokka_model::AudienceMode;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Element, Length, Padding};

use crate::app::App;
use crate::message::Message;
use crate::theme::{self, pad, size, space};

/// One selectable card, in on-screen order. `App::update` indexes into this
/// with the same positions `view` renders, so arrow-key navigation and
/// `Enter` can never select a different mode than the one visibly
/// highlighted.
pub const OPTIONS: &[(Key, Key, AudienceMode)] = &[
    (
        Key::ModeGuidedTitle,
        Key::ModeGuidedDesc,
        AudienceMode::Guided,
    ),
    (
        Key::ModeExpertTitle,
        Key::ModeExpertDesc,
        AudienceMode::Expert,
    ),
];

pub fn view(app: &App) -> Element<'_, Message> {
    let card = |pos: usize, title: Key, desc: Key, mode: AudienceMode| -> Element<Message> {
        let selected = app.mode_picker_selected == Some(pos);
        button(
            container(
                column![
                    text(app.t(title)).size(size::SECTION),
                    text(app.t(desc))
                        .size(size::BODY)
                        .color(theme::muted(&app.theme())),
                ]
                .spacing(space::S2),
            )
            .padding(Padding::from(pad::CARD))
            .style(if selected {
                theme::card_selected_style
            } else {
                theme::card_style
            })
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
            text(app.t(Key::AppName))
                .size(size::DISPLAY)
                .color(theme::muted(&app.theme())),
            Space::new().height(space::S2),
            text(app.t(Key::ModePickerTitle)).size(size::TITLE),
            Space::new().height(space::S3),
            card(0, OPTIONS[0].0, OPTIONS[0].1, OPTIONS[0].2),
            Space::new().height(space::S2),
            card(1, OPTIONS[1].0, OPTIONS[1].1, OPTIONS[1].2),
            Space::new().height(space::S3),
            row![
                text(app.t(Key::ModePickerHint))
                    .size(size::CAPTION)
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
