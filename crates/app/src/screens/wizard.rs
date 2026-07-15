//! MK-026 — S-02 Wizard.
use crate::app::App;
use crate::message::Message;
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use iced::widget::{Space, button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Padding};

pub fn view(app: &App) -> Element<'_, Message> {
    let w = &app.wizard;
    let t = |k| app.t(k);

    // Required fields
    let required = column![
        widgets::field(
            t(Key::WizardFieldName),
            text_input("payments-mock", &w.name)
                .on_input(Message::WizardSetName)
                .size(size::BODY)
                .padding(Padding::from([space::S2, space::S3]))
                .width(Length::Fill)
                .into()
        ),
        widgets::field(
            t(Key::WizardFieldFolder),
            text_input("~/projects", &w.folder)
                .on_input(Message::WizardSetFolder)
                .size(size::BODY)
                .padding(Padding::from([space::S2, space::S3]))
                .width(Length::Fill)
                .into()
        ),
    ]
    .spacing(space::S3);

    // Server section
    let server = collapsible(
        app,
        t(Key::WizardSectionServer),
        t(Key::WizardSectionServerHint),
        0,
        column![
            row![
                widgets::field(
                    t(Key::WizardFieldHost),
                    text_input("127.0.0.1", &w.host)
                        .on_input(Message::WizardSetHost)
                        .size(size::BODY)
                        .padding(Padding::from([space::S2, space::S3]))
                        .width(Length::Fill)
                        .into()
                ),
                widgets::field(
                    t(Key::WizardFieldPort),
                    text_input("8080", &w.port)
                        .on_input(Message::WizardSetPort)
                        .size(size::BODY)
                        .padding(Padding::from([space::S2, space::S3]))
                        .width(Length::Fixed(100.0))
                        .into()
                ),
            ]
            .spacing(space::S3)
            .align_y(Alignment::End),
            checkbox(w.tls)
                .label(t(Key::WizardFieldTls))
                .on_toggle(Message::WizardSetTls)
                .size(size::BODY),
        ]
        .spacing(space::S3)
        .into(),
    );

    // Starter section
    let starter = collapsible(
        app,
        t(Key::WizardSectionStarter),
        t(Key::WizardSectionStarterHint),
        1,
        column![
            iced::widget::radio(
                t(Key::WizardStarterMinimal),
                crate::app::WizardStarter::Minimal,
                Some(w.starter),
                Message::WizardSetStarter,
            )
            .size(size::BODY),
            iced::widget::radio(
                t(Key::WizardStarterShopApi),
                crate::app::WizardStarter::ShopApi,
                Some(w.starter),
                Message::WizardSetStarter,
            )
            .size(size::BODY),
            iced::widget::radio(
                t(Key::WizardStarterEmpty),
                crate::app::WizardStarter::Empty,
                Some(w.starter),
                Message::WizardSetStarter,
            )
            .size(size::BODY),
        ]
        .spacing(space::S3)
        .into(),
    );

    // Trace section
    let trace = collapsible(
        app,
        t(Key::WizardSectionTrace),
        t(Key::WizardSectionTraceHint),
        2,
        column![
            checkbox(true)
                .label(t(Key::WizardTraceEnable))
                .size(size::BODY),
            widgets::field(
                t(Key::WizardQueueSize),
                text_input("1024", &w.queue_size)
                    .on_input(Message::WizardSetQueueSize)
                    .size(size::BODY)
                    .padding(Padding::from([space::S2, space::S3]))
                    .width(Length::Fixed(120.0))
                    .into()
            ),
        ]
        .spacing(space::S3)
        .into(),
    );

    let form = column![
        text(t(Key::WizardTitle)).size(size::TITLE),
        Space::new().height(space::S1),
        text("Fill in your workspace name and folder. Advanced sections have sensible defaults.")
            .size(size::CAPTION)
            .color(theme::muted(&app.theme())),
        widgets::divider(),
        required,
        widgets::divider(),
        server,
        starter,
        trace,
    ]
    .spacing(space::S4)
    .padding(Padding::from([space::S5, space::S6]));

    let action_bar = container(
        row![
            widgets::ghost_btn(t(Key::BtnCancel), Message::WizardCancel),
            Space::new().width(Length::Fill),
            widgets::primary_btn(t(Key::BtnCreate), Message::WizardCreate),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([space::S3, space::S6]))
    .style(theme::panel_style);

    let inner = column![scrollable(form).height(Length::Fill), action_bar,]
        .width(Length::Fixed(680.0))
        .height(Length::Fixed(620.0));

    container(inner)
        .style(theme::dialog_style)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn collapsible<'a>(
    app: &'a App,
    heading: &'a str,
    hint: &'a str,
    index: usize,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    let open = app.wizard.section_open.get(index).copied().unwrap_or(false);
    let chevron = if open { "▾" } else { "▸" };
    let section_head = button(
        row![
            text(chevron).size(size::BODY),
            column![
                text(heading).size(size::SECTION),
                text(hint)
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
            ]
            .spacing(2.0),
        ]
        .spacing(space::S2)
        .align_y(Alignment::Center),
    )
    .on_press(Message::WizardToggleSection(index))
    .padding(Padding::from([space::S2, 0.0]));

    if open {
        column![
            section_head,
            container(body)
                .padding(Padding::from([space::S3, space::S4]))
                .style(theme::card_style)
                .width(Length::Fill),
        ]
        .spacing(space::S2)
        .into()
    } else {
        column![section_head].into()
    }
}
