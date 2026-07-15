//! MK-034 — O-03 Test Rule dialog.
use crate::app::App;
use crate::message::{Message, TestRuleResult};
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use iced::widget::{Space, button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length, Padding};

pub fn view(app: &App) -> Element<'_, Message> {
    let tr = &app.test_rule;

    let result_el: Element<Message> = match &tr.result {
        None => text(app.t(Key::TestRuleResultHint))
            .size(size::CAPTION)
            .color(theme::muted(&app.theme()))
            .into(),
        Some(TestRuleResult::Matched { summary }) => container(
            row![
                text(app.t(Key::TestRuleMatched)).size(size::BODY),
                text(summary.as_str())
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
            ]
            .spacing(space::S3)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([space::S3, space::S4]))
        .style(theme::card_selected_style)
        .width(Length::Fill)
        .into(),
        Some(TestRuleResult::NoMatch) => {
            container(text(app.t(Key::TestRuleNoMatch)).size(size::BODY))
                .padding(Padding::from([space::S3, space::S4]))
                .style(theme::chip_style)
                .width(Length::Fill)
                .into()
        }
        Some(TestRuleResult::Error(msg)) => container(
            row![
                text(app.t(Key::TestRuleError)).size(size::BODY),
                text(msg.as_str()).size(size::CAPTION),
            ]
            .spacing(space::S3),
        )
        .padding(Padding::from([space::S3, space::S4]))
        .style(theme::banner_style)
        .width(Length::Fill)
        .into(),
    };

    let methods = ["GET", "POST", "PUT", "PATCH", "DELETE"];
    let method_btns: Vec<Element<Message>> = methods
        .iter()
        .map(|m| {
            let active = tr.method.to_uppercase() == *m;
            button(text(*m).size(size::CAPTION))
                .on_press(Message::TestRuleSetMethod(m.to_string()))
                .padding(Padding::from([space::S2, space::S3]))
                .style(if active {
                    theme::seg_active
                } else {
                    theme::seg_inactive
                })
                .into()
        })
        .collect();

    container(
        column![
            row![
                text(app.t(Key::TestRuleTitle))
                    .size(size::SECTION)
                    .width(Length::Fill),
                button(text("✕").size(size::BODY))
                    .on_press(Message::TestRuleClose)
                    .padding(Padding::from([space::S1, space::S2])),
            ]
            .align_y(Alignment::Center),
            text(app.t(Key::TestRuleHint))
                .size(size::CAPTION)
                .color(theme::muted(&app.theme())),
            Space::new().height(space::S2),
            widgets::divider(),
            widgets::field(
                app.t(Key::TestRuleMethod),
                row(method_btns).spacing(space::S1).into()
            ),
            widgets::field(
                app.t(Key::TestRulePath),
                text_input("/api/orders", &tr.url_path)
                    .on_input(Message::TestRuleSetPath)
                    .size(size::BODY)
                    .padding(Padding::from([space::S2, space::S3]))
                    .width(Length::Fill)
                    .into()
            ),
            widgets::field(
                app.t(Key::TestRuleHeaders),
                text_input("content-type: application/json", &tr.headers_text)
                    .on_input(Message::TestRuleSetHeaders)
                    .size(size::CAPTION)
                    .padding(Padding::from([space::S2, space::S3]))
                    .width(Length::Fill)
                    .into()
            ),
            widgets::field(
                app.t(Key::TestRuleBody),
                text_input("{}", &tr.body)
                    .on_input(Message::TestRuleSetBody)
                    .size(size::CAPTION)
                    .padding(Padding::from([space::S2, space::S3]))
                    .width(Length::Fill)
                    .into()
            ),
            result_el,
            widgets::divider(),
            row![
                Space::new().width(Length::Fill),
                widgets::ghost_btn(app.t(Key::BtnClose), Message::TestRuleClose),
                widgets::primary_btn(app.t(Key::BtnRunTest), Message::TestRuleRun),
            ]
            .spacing(space::S3)
            .align_y(Alignment::Center),
        ]
        .spacing(space::S3)
        .padding(space::S5)
        .width(Length::Fixed(540.0)),
    )
    .style(theme::dialog_style)
    .into()
}
