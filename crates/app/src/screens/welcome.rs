//! MK-025 — S-00 Welcome.
use crate::app::App;
use crate::message::Message;
use crate::regions::{Cursor, Region, WELCOME_ACTIONS};
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

    // Task 028 §2: rendered from the same table `regions::Region::WelcomeActions`
    // reads, so the visible actions and the keyboard's idea of them can never
    // disagree (the D-9 lesson). Position 0 (Open workspace) stays primary,
    // position 1 (Create new workspace) stays secondary — the same visual
    // distinction the screen always had.
    let action_items: Vec<Element<Message>> = WELCOME_ACTIONS
        .iter()
        .enumerate()
        .map(|(pos, &(key, message))| {
            let has_cursor = app.focused_region == Some(Region::WelcomeActions)
                && app.region_cursor == Some(Cursor::Index(pos));
            let btn = if pos == 0 {
                widgets::primary_btn(app.t(key), message())
            } else {
                widgets::secondary_btn(app.t(key), message())
            };
            // `cursor_style`'s border is drawn at the wrapping container's own
            // bounds; without a gap the primary/secondary button's own
            // opaque background (its own child, rendered on top) paints
            // straight over it. `space::S1` gives the border room to show,
            // the same reason `tab_bar::view`'s wrap has the indicator strip
            // beneath its button rather than butting the border against it.
            container(btn)
                .padding(space::S1)
                .style(if has_cursor {
                    theme::cursor_style
                } else {
                    |_: &iced::Theme| iced::widget::container::Style::default()
                })
                .into()
        })
        .collect();
    let actions = row(action_items)
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

    // RFC MK-059 decision 5: a full sentence, the wrapping-prose case
    // `body`'s line-height (1.4, the one unambiguous readability gain)
    // targets -- unlike the hero tagline above, which is a short single-line
    // label that never wraps and is left without it.
    let no_recents = text(app.t(Key::WelcomeNoRecents))
        .size(size::BODY)
        .line_height(theme::line_height::body())
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
