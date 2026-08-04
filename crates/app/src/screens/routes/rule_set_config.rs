//! Rule-set configuration panel — file header, rule list, strategy picker.

use crate::app::App;
use crate::message::Message;
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use apimokka_model::settings::Strategy;
use apimokka_model::snapshot::RuleSetView;
use iced::widget::{Space, button, column, container, pick_list, row, scrollable, text};
use iced::{Alignment, Element, Length, Padding};

pub(super) fn rule_set_config<'a>(app: &'a App, rs: &'a RuleSetView) -> Element<'a, Message> {
    let t = |k| app.t(k);
    let file_name = rs.file.path.rsplit('/').next().unwrap_or(&rs.file.path);

    let header = container(
        row![
            column![
                text(file_name).size(size::SECTION),
                text(rs.file.path.as_str())
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
                text(format!(
                    "{} {}",
                    rs.rules.len(),
                    t(Key::RoutesRuleCountNoun)
                ))
                .size(size::CAPTION)
                .color(theme::muted(&app.theme())),
            ]
            .spacing(space::S1)
            .width(Length::Fill),
            widgets::danger_btn(t(Key::BtnDelete), Message::DeleteRuleSet(rs.id)),
        ]
        .spacing(space::S3)
        .align_y(Alignment::Start),
    )
    .padding(Padding::from([space::S4, space::S5]))
    .width(Length::Fill);

    let rule_rows: Vec<Element<Message>> = rs
        .rules
        .iter()
        .map(|r| {
            let summary = r.summary();
            let rule_id = r.id;
            button(
                container(
                    row![
                        text("⠿")
                            .size(size::CAPTION)
                            .color(theme::muted(&app.theme())),
                        text(summary).size(size::BODY).width(Length::Fill),
                    ]
                    .spacing(space::S2)
                    .align_y(Alignment::Center),
                )
                .padding(Padding::from([space::S2, space::S3]))
                .style(theme::card_style)
                .width(Length::Fill),
            )
            .on_press(Message::SelectRule(rule_id))
            .padding(0)
            .style(theme::naked)
            .width(Length::Fill)
            .into()
        })
        .collect();

    let rules_body: Element<Message> = if rule_rows.is_empty() {
        widgets::empty_state(t(Key::EmptyRuleSetNoRules))
    } else {
        column(rule_rows).spacing(space::S1).into()
    };

    // MK-043: strategy section — always visible in Expert, collapsible in Guided.
    let active_strategy = app
        .snapshot
        .as_ref()
        .map(|s| s.root_settings.strategy)
        .unwrap_or(Strategy::FirstMatch);

    let strategy_section: Element<Message> = {
        let dropdown = pick_list(
            Strategy::all().to_vec(),
            Some(active_strategy),
            Message::RuleSetSetStrategy,
        )
        .text_size(size::BODY)
        .padding(Padding::from([space::S2, space::S3]));

        let help_text = text(active_strategy.help())
            .size(size::CAPTION)
            .color(theme::muted(&app.theme()));

        let heading: Element<Message> = if app.shows_scaffolding() {
            // Guided: ⓘ hint expanded inline
            column![
                text(t(Key::RuleSetConfigStrategy)).size(size::BODY_STRONG),
                text(active_strategy.help())
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
            ]
            .spacing(space::S1)
            .into()
        } else {
            widgets::label_with_hint(
                &app.theme(),
                t(Key::RuleSetConfigStrategy),
                t(Key::HintStrategy),
            )
        };

        let inner = column![
            heading,
            row![dropdown, Space::new().width(space::S3), help_text].align_y(Alignment::Center),
        ]
        .spacing(space::S2);

        if app.shows_scaffolding() {
            // Guided: strategy section collapsed by default
            let (chevron, label) = if app.rule_set_config_more {
                ("▾", t(Key::RuleSetConfigFewerOptions))
            } else {
                ("▸", t(Key::RuleSetConfigMoreOptions))
            };
            let toggle = button(
                row![
                    text(chevron)
                        .size(size::CAPTION)
                        .color(theme::muted(&app.theme())),
                    text(label)
                        .size(size::CAPTION)
                        .color(theme::muted(&app.theme())),
                ]
                .spacing(space::S2)
                .align_y(Alignment::Center),
            )
            .on_press(Message::ToggleRuleSetConfigMore)
            .padding(Padding::from([space::S2, space::S3]))
            .style(iced::widget::button::text);

            if app.rule_set_config_more {
                column![inner, toggle].spacing(space::S2).into()
            } else {
                toggle.into()
            }
        } else {
            inner.into()
        }
    };

    container(
        column![
            header,
            widgets::divider(),
            scrollable(
                column![
                    rules_body,
                    Space::new().height(space::S3),
                    widgets::divider(),
                    strategy_section,
                    Space::new().height(space::S3),
                    container(widgets::primary_btn(
                        t(Key::BtnAddRule),
                        Message::AddRule(rs.id)
                    ),)
                    .padding(Padding::from([0.0, space::S5])),
                ]
                .spacing(0)
                .padding(Padding::from([space::S3, space::S5])),
            )
            .height(Length::Fill),
        ]
        .spacing(0),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
