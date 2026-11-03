//! MK-034 — O-05 Confirm dialog.
use crate::app::App;
use crate::message::Message;
use crate::regions::{Cursor, Region};
use crate::theme::{self, size, space};
use crate::widgets;
use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Element, Length};

/// Task 033: `ConfirmDialog` is a bare, two-item region (Cancel at 0,
/// Proceed at 1 — `ConfirmRequest` sets the cursor to `Cancel` the moment
/// the dialog opens, task 026 §4.2's safety default). Same shape as
/// `settings.rs`'s own `cursor_wrap_group`, without a field type to carry.
fn cursor_wrap<'a>(app: &App, index: usize, el: Element<'a, Message>) -> Element<'a, Message> {
    let has_cursor = app.focused_region == Some(Region::ConfirmDialog)
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
    let Some(d) = &app.confirm_dialog else {
        return Space::new().into();
    };
    container(
        column![
            text(app.t(d.title)).size(size::SECTION),
            Space::new().height(space::S2),
            // RFC MK-059 decision 5: `body`'s line-height (1.4) is the one
            // unambiguous readability gain -- applied here since a confirm
            // dialog's body is exactly the wrapping-prose case it targets.
            text(app.t(d.body))
                .size(size::BODY)
                .line_height(theme::line_height::body())
                .color(theme::muted(&app.theme())),
            Space::new().height(space::S5),
            row![
                cursor_wrap(
                    app,
                    0,
                    widgets::ghost_btn(
                        app.t(apimokka_i18n::Key::BtnCancel),
                        Message::ConfirmCancel
                    ),
                ),
                Space::new().width(Length::Fill),
                cursor_wrap(
                    app,
                    1,
                    widgets::danger_btn(
                        app.t(apimokka_i18n::Key::ConfirmProceed),
                        Message::ConfirmProceed
                    ),
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
