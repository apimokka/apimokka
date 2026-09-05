//! MK-033 — Command palette dialog.
use crate::accelerator::{self, Accelerator};
use crate::app::App;
use crate::message::Message;
use crate::palette_commands::{self, filtered_indices};
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Padding};

/// Stable id for the search field, so `update` can focus it on open
/// (MK-033 lines 38, 95, 118) without the id drifting from this view.
pub const SEARCH_INPUT_ID: &str = "mk033-palette-search";

pub fn view(app: &App) -> Element<'_, Message> {
    let filtered = filtered_indices(app, &app.command_palette.query);
    let rows: Vec<Element<Message>> = filtered
        .iter()
        .enumerate()
        .map(|(pos, &table_index)| {
            let cmd = &palette_commands::TABLE[table_index];
            let shortcut_el: Element<Message> = if let Some(sc) = cmd.shortcut {
                container(text(accelerator::display(sc)).size(size::CAPTION))
                    .padding(Padding::from([2.0, 8.0]))
                    .style(theme::chip_style)
                    .into()
            } else {
                Space::new().width(0.0).into()
            };
            let selected = Some(pos) == app.command_palette.selected;
            button(
                container(
                    row![
                        text(app.t(cmd.label)).size(size::BODY).width(Length::Fill),
                        shortcut_el,
                    ]
                    .align_y(Alignment::Center),
                )
                .padding(Padding::from([space::S3, space::S4]))
                .style(if selected {
                    theme::card_selected_style
                } else {
                    theme::card_style
                })
                .width(Length::Fill),
            )
            .on_press((cmd.message)())
            .padding(0)
            .style(theme::naked)
            .width(Length::Fill)
            .into()
        })
        .collect();

    let list: Element<Message> = if rows.is_empty() {
        widgets::empty_state(app.t(Key::PaletteNoMatch))
    } else {
        scrollable(column(rows).spacing(space::S1))
            .height(Length::Fixed(320.0))
            .into()
    };

    container(
        column![
            // Header with shortcut hints
            row![
                text(app.t(Key::PaletteTitle))
                    .size(size::SECTION)
                    .width(Length::Fill),
                container(text(accelerator::display(Accelerator::Palette)).size(size::CAPTION))
                    .padding(Padding::from([2.0, 8.0]))
                    .style(theme::chip_style),
                container(text("Esc").size(size::CAPTION))
                    .padding(Padding::from([2.0, 8.0]))
                    .style(theme::chip_style),
                Space::new().width(space::S2),
                button(text("✕").size(size::BODY))
                    .on_press(Message::ToggleCommandPalette)
                    .padding(Padding::from([space::S1, space::S2])),
            ]
            .spacing(space::S2)
            .align_y(Alignment::Center),
            Space::new().height(space::S2),
            text_input(app.t(Key::PaletteSearch), &app.command_palette.query)
                .id(SEARCH_INPUT_ID)
                .on_input(Message::PaletteQuery)
                .size(size::BODY)
                .padding(Padding::from([space::S3, space::S4]))
                .width(Length::Fill),
            Space::new().height(space::S1),
            widgets::divider(),
            list,
        ]
        .spacing(space::S2)
        .padding(space::S5)
        .width(Length::Fixed(540.0)),
    )
    .style(theme::dialog_style)
    .into()
}
