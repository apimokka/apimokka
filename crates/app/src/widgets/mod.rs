//! Shared widget helpers using MK-022 design tokens.

use apimokka_model::Severity;
use iced::widget::{Space, button, container, row, text};
use iced::{Color, Element, Length, Padding};

use crate::message::Message;
use crate::theme::{self, pad, size, space};

// ── Divider ──────────────────────────────────────────────────────────────────

pub fn divider<'a>() -> Element<'a, Message> {
    container(Space::new().height(Length::Fixed(1.0)))
        .width(Length::Fill)
        .style(theme::hairline_style)
        .into()
}

// ── Empty state ───────────────────────────────────────────────────────────────

pub fn empty_state<'a>(msg: &'a str) -> Element<'a, Message> {
    container(
        text(msg)
            .size(size::BODY)
            .color(Color::from_rgb(0.52, 0.52, 0.52)),
    )
    .padding(Padding::from([space::S5, space::S6]))
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .into()
}

// ── Dirty dot ─────────────────────────────────────────────────────────────────

pub fn dirty_dot<'a>() -> Element<'a, Message> {
    text("●")
        .size(size::CAPTION)
        .color(Color::from_rgb(0.90, 0.55, 0.10))
        .into()
}

// ── Severity badge ─────────────────────────────────────────────────────────────

pub fn severity_glyph(sev: Severity) -> &'static str {
    match sev {
        Severity::Error => "✕",
        Severity::Warning => "⚠",
        Severity::Info => "ℹ",
    }
}

#[allow(dead_code)]
pub fn severity_badge<'a>(t: &iced::Theme, sev: Severity, msg: &'a str) -> Element<'a, Message> {
    let color = theme::severity_color(t, sev);
    container(
        row![
            text(severity_glyph(sev)).size(size::CAPTION).color(color),
            text(msg).size(size::CAPTION).color(color),
        ]
        .spacing(space::S1),
    )
    .padding(Padding::from(pad::CHIP))
    .into()
}

// ── Button helpers (MK-022 §4.6) ─────────────────────────────────────────────

pub fn primary_btn<'a>(label: &'a str, msg: Message) -> Element<'a, Message> {
    button(text(label).size(size::BODY))
        .on_press(msg)
        .padding(Padding::from(pad::BUTTON_PRIMARY))
        .style(iced::widget::button::primary)
        .into()
}

pub fn secondary_btn<'a>(label: &'a str, msg: Message) -> Element<'a, Message> {
    button(text(label).size(size::BODY))
        .on_press(msg)
        .padding(Padding::from(pad::BUTTON))
        .style(iced::widget::button::secondary)
        .into()
}

pub fn ghost_btn<'a>(label: &'a str, msg: Message) -> Element<'a, Message> {
    button(text(label).size(size::BODY))
        .on_press(msg)
        .padding(Padding::from(pad::BUTTON))
        .style(iced::widget::button::text)
        .into()
}

pub fn danger_btn<'a>(label: &'a str, msg: Message) -> Element<'a, Message> {
    button(text(label).size(size::BODY))
        .on_press(msg)
        .padding(Padding::from(pad::BUTTON))
        .style(iced::widget::button::danger)
        .into()
}

pub fn icon_btn<'a>(glyph: &'a str, msg: Message) -> Element<'a, Message> {
    button(text(glyph).size(size::BODY))
        .on_press(msg)
        .padding(Padding::from([space::S1, space::S2]))
        .into()
}

// ── Labelled field ─────────────────────────────────────────────────────────────

pub fn field<'a>(label: &'a str, control: Element<'a, Message>) -> Element<'a, Message> {
    iced::widget::column![
        text(label)
            .size(size::CAPTION)
            .color(Color::from_rgb(0.52, 0.52, 0.52)),
        control,
    ]
    .spacing(space::S1)
    .into()
}

// ── MK-039: info hint (ⓘ tooltip) ─────────────────────────────────────────────

/// A small ⓘ affordance that reveals a concept hint on hover. The hint teaches
/// a domain gotcha in technical language; it is opt-in so the default view
/// stays uncluttered ("less is more").
pub fn info_hint<'a>(theme: &iced::Theme, hint: &'a str) -> Element<'a, Message> {
    let marker = text("\u{24D8}") // ⓘ
        .size(size::CAPTION)
        .color(theme::muted(theme));

    let bubble = container(text(hint).size(size::CAPTION))
        .padding(Padding::from([space::S2, space::S3]))
        .max_width(280.0)
        .style(theme::card_style);

    iced::widget::tooltip(marker, bubble, iced::widget::tooltip::Position::Top)
        .gap(space::S1)
        .into()
}

/// A label with a trailing ⓘ hint, for field headings.
pub fn label_with_hint<'a>(
    theme: &iced::Theme,
    label: &'a str,
    hint: &'a str,
) -> Element<'a, Message> {
    row![
        text(label).size(size::BODY_STRONG),
        Space::new().width(Length::Fixed(space::S1)),
        info_hint(theme, hint),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}

// ── MK-039: primary action with a visible blocked reason ──────────────────────

/// Renders a primary action. When `ready` carries a message the button is
/// active; when it is `None` the button is disabled AND `reason` is shown
/// beside it, so no dead control ever appears without an explanation.
pub fn action_with_reason<'a>(
    theme: &iced::Theme,
    label: &'a str,
    ready: Option<Message>,
    reason: &'a str,
) -> Element<'a, Message> {
    let mut b = button(text(label).size(size::BODY))
        .padding(Padding::from(pad::BUTTON_PRIMARY))
        .height(Length::Fixed(theme::touch::COMFORTABLE));
    let blocked = ready.is_none();
    if let Some(msg) = ready {
        b = b.on_press(msg).style(iced::widget::button::primary);
    }
    if blocked {
        row![
            b,
            Space::new().width(Length::Fixed(space::S3)),
            text(reason).size(size::CAPTION).color(theme::muted(theme)),
        ]
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        b.into()
    }
}

// ── MK-042: two-column field row for trace detail ─────────────────────────────

/// A compact `label  value` row for the match detail panel.
pub fn field_row<'a>(label: &'a str, value: &'a str) -> Element<'a, Message> {
    row![
        text(label)
            .size(size::CAPTION)
            .color(iced::Color::from_rgb(0.55, 0.55, 0.55))
            .width(Length::Fixed(100.0)),
        text(value).size(size::CAPTION).width(Length::Fill),
    ]
    .spacing(space::S2)
    .align_y(iced::Alignment::Center)
    .into()
}
