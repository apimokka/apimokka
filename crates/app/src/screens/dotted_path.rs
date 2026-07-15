//! MK-034 — O-04 Dotted-path assistant.
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length, Padding};
use apimokka_i18n::Key;
use crate::app::App;
use crate::message::Message;
use crate::theme::{self, size, space};
use crate::widgets;

pub fn view(app: &App) -> Element<'_, Message> {
    let pa = &app.path_assistant;

    // Parse sample JSON into a simple leaf list
    let leaves = if pa.json_input.is_empty() {
        vec![]
    } else {
        parse_leaves(&pa.json_input)
    };

    let tree_rows: Vec<Element<Message>> = if leaves.is_empty() {
        if pa.json_input.is_empty() {
            vec![text(app.t(Key::DottedPathEmpty)).size(size::CAPTION)
                .color(theme::muted(&app.theme())).into()]
        } else {
            vec![text(app.t(Key::DottedPathJsonError)).size(size::CAPTION)
                .color(iced::Color::from_rgb(0.85, 0.0, 0.0)).into()]
        }
    } else {
        // Collect into owned pairs to avoid borrowing `leaves` across the closure.
        let owned: Vec<(String, String)> = leaves.into_iter().collect();
        owned.into_iter().map(|(path, val)| {
            let path_clone = path.clone();
            row![
                text(path).size(size::CAPTION).width(Length::Fill),
                text(val).size(size::CAPTION)
                    .color(theme::muted(&app.theme()))
                    .width(Length::Fixed(100.0)),
                button(text(app.t(Key::BtnUse)).size(size::CAPTION))
                    .on_press(Message::PathAssistantSelectPath(path_clone))
                    .padding(Padding::from([space::S1, space::S3])),
            ]
            .spacing(space::S3).align_y(Alignment::Center).into()
        }).collect()
    };

    let jsonpath_warn: Element<Message> = if pa.selected_path.starts_with("$.") {
        text(app.t(Key::DottedPathJsonpathHint)).size(size::CAPTION)
            .color(iced::Color::from_rgb(0.85, 0.45, 0.0)).into()
    } else {
        Space::new().height(0.0).into()
    };

    container(
        column![
            row![
                text(app.t(Key::DottedPathTitle)).size(size::SECTION).width(Length::Fill),
                button(text("✕").size(size::BODY))
                    .on_press(Message::PathAssistantClose)
                    .padding(Padding::from([space::S1, space::S2])),
            ]
            .align_y(Alignment::Center),
            widgets::divider(),
            widgets::field(app.t(Key::DottedPathPasteLabel),
                text_input("{\"user\":{\"id\":123}}", &pa.json_input)
                    .on_input(Message::PathAssistantSetJson)
                    .size(size::BODY)
                    .padding(Padding::from([space::S2, space::S3]))
                    .width(Length::Fill)
                    .into()),
            text(app.t(Key::DottedPathTreeLabel)).size(size::BODY),
            container(
                scrollable(column(tree_rows).spacing(space::S1))
                    .height(Length::Fixed(160.0)),
            )
            .padding(Padding::from([space::S2, 0.0])),
            {
                let selected_el: Element<Message> = if !pa.selected_path.is_empty() {
                    column![
                        row![
                            text(app.t(Key::DottedPathSelectedLabel)).size(size::CAPTION),
                            text(pa.selected_path.as_str()).size(size::CAPTION),
                        ].spacing(space::S2),
                        jsonpath_warn,
                    ]
                    .spacing(space::S1)
                    .into()
                } else {
                    Space::new().height(Length::Fixed(0.0)).into()
                };
                selected_el
            },
            widgets::divider(),
            row![
                widgets::ghost_btn(app.t(Key::BtnCancel), Message::PathAssistantClose),
                Space::new().width(Length::Fill),
                widgets::primary_btn(app.t(Key::BtnInsertPath), Message::PathAssistantInsert),
            ]
            .spacing(space::S3).align_y(Alignment::Center),
        ]
        .spacing(space::S3)
        .padding(space::S5)
        .width(Length::Fixed(480.0)),
    )
    .style(theme::dialog_style)
    .into()
}

/// Very simple JSON leaf extractor — extracts dotted paths and their values.
/// Handles flat objects and one level of nesting; good enough for the mockup.
fn parse_leaves(json: &str) -> Vec<(String, String)> {
    let json = json.trim();
    if !json.starts_with('{') { return vec![]; }
    let inner = json.trim_start_matches('{').trim_end_matches('}');
    let mut out = vec![];
    for part in inner.split(',') {
        let part = part.trim();
        if let Some(colon) = part.find(':') {
            let key = part[..colon].trim().trim_matches('"');
            let val = part[colon+1..].trim();
            if val.starts_with('{') {
                // One-level nested object
                let inner2 = val.trim_matches('{').trim_matches('}');
                for part2 in inner2.split(',') {
                    let part2 = part2.trim();
                    if let Some(c2) = part2.find(':') {
                        let k2 = part2[..c2].trim().trim_matches('"');
                        let v2 = part2[c2+1..].trim().trim_matches('"');
                        out.push((format!("{key}.{k2}"), v2.to_string()));
                    }
                }
            } else {
                out.push((key.to_string(), val.trim_matches('"').to_string()));
            }
        }
    }
    out
}
