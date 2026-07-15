//! MK-034 — O-05 Confirm dialog.
use crate::app::App;
use crate::message::Message;
use crate::theme::{self, size, space};
use crate::widgets;
use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let Some(d) = &app.confirm_dialog else {
        return Space::new().into();
    };
    container(
        column![
            text(app.t(d.title)).size(size::SECTION),
            Space::new().height(space::S2),
            text(app.t(d.body))
                .size(size::BODY)
                .color(theme::muted(&app.theme())),
            Space::new().height(space::S5),
            row![
                widgets::ghost_btn(app.t(apimokka_i18n::Key::BtnCancel), Message::ConfirmCancel),
                Space::new().width(Length::Fill),
                widgets::danger_btn(
                    app.t(apimokka_i18n::Key::ConfirmProceed),
                    Message::ConfirmProceed
                ),
            ]
            .spacing(space::S3)
            .align_y(Alignment::Center),
        ]
        .padding(space::S6)
        .width(Length::Fixed(440.0)),
    )
    .style(theme::dialog_style)
    .into()
}
