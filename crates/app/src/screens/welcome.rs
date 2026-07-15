//! MK-025 — S-00 Welcome.
use crate::app::App;
use crate::message::Message;
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Element, Length, Padding};

pub fn view(app: &App) -> Element<'_, Message> {
    let hero = column![
        text(app.t(Key::AppName)).size(size::DISPLAY),
        text(app.t(Key::WelcomeHeroTagline))
            .size(size::BODY)
            .color(theme::muted(&app.theme())),
    ]
    .spacing(space::S2)
    .align_x(Alignment::Center);

    let open_btn = widgets::primary_btn(app.t(Key::WelcomeOpenWorkspace), Message::GoDashboard);
    let create_btn = widgets::secondary_btn(app.t(Key::WelcomeCreateWorkspace), Message::GoWizard);
    let actions = row![open_btn, create_btn]
        .spacing(space::S3)
        .align_y(Alignment::Center);

    // Request-handling diagram
    let diagram = container(
        column![
            text(app.t(Key::WelcomeHowTitle)).size(size::SECTION),
            Space::new().height(space::S3),
            pipeline_step("⚙", app.t(Key::WelcomeHowMiddleware)),
            arrow_down(),
            pipeline_step("⎘", app.t(Key::WelcomeHowRuleSets)),
            arrow_down(),
            pipeline_step("📄", app.t(Key::WelcomeHowFallback)),
        ]
        .spacing(space::S1)
        .align_x(Alignment::Center),
    )
    .padding(Padding::from([space::S4, space::S6]))
    .style(theme::card_style);

    let no_recents = text(app.t(Key::WelcomeNoRecents))
        .size(size::BODY)
        .color(theme::muted(&app.theme()));

    let page = column![
        Space::new().height(space::S6),
        hero,
        Space::new().height(space::S5),
        actions,
        Space::new().height(space::S6),
        diagram,
        Space::new().height(space::S4),
        no_recents,
    ]
    .align_x(Alignment::Center)
    .spacing(0)
    .width(Length::Fixed(680.0));

    container(page)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn pipeline_step<'a>(icon: &'a str, label: &'a str) -> Element<'a, Message> {
    container(
        row![text(icon).size(size::TITLE), text(label).size(size::BODY),]
            .spacing(space::S3)
            .align_y(Alignment::Center),
    )
    .padding(Padding::from([space::S3, space::S5]))
    .width(Length::Fixed(280.0))
    .style(theme::card_style)
    .into()
}

fn arrow_down<'a>() -> Element<'a, Message> {
    container(
        text("↓")
            .size(size::SECTION)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
    )
    .padding(Padding::from([space::S1, 0.0]))
    .align_x(iced::alignment::Horizontal::Center)
    .width(Length::Fixed(280.0))
    .into()
}
