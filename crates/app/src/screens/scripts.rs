//! MK-031 — S-14 Scripts (read-only middleware viewer).
use crate::app::App;
use crate::message::Message;
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length, Padding};

#[allow(dead_code)]
pub fn view(app: &App) -> Element<'_, Message> {
    let snap = match &app.snapshot {
        Some(s) => s,
        None => return widgets::empty_state(app.t(Key::EmptyNoWorkspaceOpen)),
    };

    if snap.middleware_scripts.is_empty() {
        return container(
            column![
                widgets::empty_state(app.t(Key::ScriptsEmptyMessage)),
                container(
                    text(app.t(Key::ScriptsEmptyExplanation))
                        .size(size::BODY_SMALL)
                        .line_height(theme::line_height::body_small())
                        .color(theme::muted(&app.theme())),
                )
                .padding(Padding::from([0.0, space::S6])),
            ]
            .spacing(space::S2),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }

    let list: Vec<Element<Message>> = snap
        .middleware_scripts
        .iter()
        .map(|s| {
            let name = s.path.rsplit('/').next().unwrap_or(&s.path);
            let path_str = s.path.clone();
            let sel = app.selection.script.as_deref() == Some(&path_str);
            button(
                container(text(name).size(size::BODY))
                    .padding(Padding::from([space::S3, space::S4]))
                    .style(if sel {
                        theme::card_selected_style
                    } else {
                        theme::card_style
                    })
                    .width(Length::Fill),
            )
            .on_press(Message::SelectScript(path_str))
            .padding(0)
            .style(theme::naked)
            .style(theme::naked)
            .width(Length::Fill)
            .into()
        })
        .collect();

    let sidebar = container(
        column![
            text(app.t(Key::ScriptsTitle))
                .size(size::SECTION)
                .width(Length::Fill),
            scrollable(column(list).spacing(space::S1)).height(Length::Fill),
        ]
        .spacing(space::S3)
        .padding(Padding::from([space::S4, space::S3])),
    )
    .width(Length::Fixed(240.0))
    .height(Length::Fill)
    .style(theme::panel_style);

    let viewer: Element<Message> = if let Some(path) = &app.selection.script {
        let content = format!(
            "-- {} --\n\nfn before_request(req) {{\n    // (script content shown here in production)\n    req\n}}\n",
            path.rsplit('/').next().unwrap_or(path.as_str())
        );
        container(scrollable(text(content).size(size::MONO)).height(Length::Fill))
            .padding(Padding::from([space::S5, space::S6]))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        container(widgets::empty_state(app.t(Key::ScriptsSelectToView)))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    row![sidebar, viewer].height(Length::Fill).into()
}
