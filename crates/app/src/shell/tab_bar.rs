//! Horizontal tab bar. Each tab occupies 1/N of the full width (equal columns).
//! Text is explicitly centered within each cell.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Element, Length, Padding};

use crate::app::App;
use crate::message::Message;
use crate::regions::{Region, TAB_BAR_ORDER};
use crate::theme::{self, size, space};
use crate::widgets;

pub fn view(app: &App) -> Element<'_, Message> {
    let tab_items: Vec<Element<Message>> = TAB_BAR_ORDER
        .iter()
        .enumerate()
        .map(|(pos, (tab, key))| {
            let active = app.tab == *tab;
            let label = app.t(*key);
            // Task 027 §3.2: "active" (this tab is displayed) and "has the
            // cursor" (the keyboard is pointing at it) are different states,
            // drawn independently — the cursor border must be visible
            // whether or not it sits on the active tab, and must not render
            // at all once the tab bar no longer holds focus.
            let has_cursor =
                app.focused_region == Some(Region::TabBar) && app.region_cursor == Some(pos);

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
            .style(if active {
                theme::seg_active
            } else {
                theme::seg_inactive
            });

            // Task 027: the keyboard cursor, drawn as an outer border around
            // the whole cell so it reads as its own state rather than
            // merging with `seg_active`'s fill (§3.2).
            container(
                container(column![btn, indicator].align_x(Alignment::Center)).style(
                    if has_cursor {
                        theme::cursor_style
                    } else {
                        |_: &iced::Theme| iced::widget::container::Style::default()
                    },
                ),
            )
            .width(Length::FillPortion(1))
            .into()
        })
        .collect();

    container(column![row(tab_items), widgets::divider(),].spacing(0))
        .width(Length::Fill)
        .style(theme::panel_style)
        .into()
}
