//! Fallback file editor — MK-038 multi-line JSON editor with save/revert.

use crate::app::App;
use crate::message::Message;
use crate::regions::{Cursor, FallbackField, Region, fallback_field_id};
use crate::theme::{self, pad, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use iced::widget::{Space, button, column, container, mouse_area, row, text, text_input};
use iced::{Alignment, Color, Element, Length, Padding};

/// Same shape as `rule_editor.rs`'s own `cursor_wrap` — for this editor's
/// one-item button regions (`Format`/`Revert`/`Save`). `Content`/`Status`
/// are text fields and use the native focus ring instead, same precedent.
fn cursor_wrap<'a>(
    app: &App,
    field: FallbackField,
    el: Element<'a, Message>,
) -> Element<'a, Message> {
    let has_cursor = app.focused_region == Some(Region::FallbackEditor(field))
        && app.region_cursor == Some(Cursor::Index(0));
    container(el)
        .padding(space::S1)
        .style(if has_cursor {
            theme::cursor_style
        } else {
            |_: &iced::Theme| iced::widget::container::Style::default()
        })
        .into()
}

/// Same shape as `rule_editor.rs`'s own `click_focus` (D-18): a click on a
/// text-field region's own widget enters that region, via `on_release`.
fn click_focus<'a>(
    field: FallbackField,
    el: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    mouse_area(el)
        .on_release(Message::RegionClicked(Region::FallbackEditor(field)))
        .into()
}

pub(super) fn fallback_file_editor<'a>(
    app: &'a App,
    file: &'a apimokka_model::node::FileNodeView,
) -> Element<'a, Message> {
    let t = |k| app.t(k);
    let route_hint = file.route_hint.as_deref().unwrap_or("/");
    let dirty = app.is_fallback_dirty(&file.path);
    let valid = app.fallback_json_valid(&file.path);
    let status = app
        .fallback_status_draft
        .get(&file.path)
        .map(|s| s.as_str())
        .unwrap_or("200 OK");

    // ── Header ────────────────────────────────────────────────────────────

    let header_col = column![
        row![
            text(file.name.as_str()).size(size::SECTION),
            Space::new().width(space::S2),
            // Dirty dot mirrors the sidebar marker (MK-038)
            {
                let dot: Element<Message> = if dirty {
                    widgets::dirty_dot()
                } else {
                    Space::new().width(Length::Fixed(0.0)).into()
                };
                dot
            },
            Space::new().width(Length::Fill),
        ]
        .align_y(Alignment::Center),
        container(
            row![
                text(t(Key::FallbackServesLabel)).size(size::BODY),
                Space::new().width(space::S1),
                text("GET").size(size::BODY),
                text(route_hint).size(size::BODY),
            ]
            .spacing(space::S2)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([space::S1 + 2.0, space::S3]))
        .style(theme::chip_style),
        text(t(Key::FallbackRouteExplanation))
            .size(size::BODY_SMALL)
            .line_height(theme::line_height::body_small())
            .color(theme::muted(&app.theme())),
    ]
    .spacing(space::S2);

    // ── Content editor (multi-line, MK-038) ───────────────────────────────
    let editor: Element<Message> = if let Some(content) = app.fallback_drafts.get(&file.path) {
        click_focus(
            FallbackField::Content,
            iced::widget::text_editor(content)
                .id(fallback_field_id(FallbackField::Content).unwrap())
                .on_action(Message::FallbackEditorAction)
                .size(size::MONO)
                .font(iced::Font::MONOSPACE)
                .height(Length::Fill),
        )
    } else {
        // Draft is created on selection; this branch is defensive only.
        widgets::empty_state(t(Key::FallbackEmptyHint))
    };

    let content_card = container(
        column![
            row![
                text(t(Key::FallbackContentLabel))
                    .size(size::BODY)
                    .width(Length::Fill),
                text(file.path.as_str())
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
            ]
            .align_y(Alignment::Center),
            editor,
        ]
        .spacing(space::S3),
    )
    .padding(Padding::from(pad::CARD))
    .style(theme::card_style)
    .width(Length::Fill)
    .height(Length::Fill);

    // ── Footer: validity badge · state hint · status · Revert · Save ──────

    let validity: Element<Message> = if valid {
        text(t(Key::FallbackJsonValid))
            .size(size::BODY_SMALL)
            .line_height(theme::line_height::body_small())
            .color(Color::from_rgb(0.10, 0.60, 0.10))
            .into()
    } else {
        text(t(Key::FallbackJsonInvalid))
            .size(size::BODY_SMALL)
            .line_height(theme::line_height::body_small())
            .color(Color::from_rgb(0.85, 0.45, 0.0))
            .into()
    };

    let state_hint = text(if dirty {
        t(Key::FallbackUnsavedHint)
    } else {
        t(Key::FallbackSavedHint)
    })
    .size(size::BODY_SMALL)
    .line_height(theme::line_height::body_small())
    .color(theme::muted(&app.theme()));

    // Revert: ghost, only actionable when dirty (routes through confirm).
    let revert_btn: Element<Message> = {
        let b = button(text(t(Key::BtnRevert)).size(size::LABEL))
            .padding(Padding::from(pad::BUTTON))
            .style(iced::widget::button::text);
        if dirty {
            b.on_press(Message::FallbackFileRevert).into()
        } else {
            b.into()
        }
    };

    // Save: primary, only actionable when dirty. Label includes the filename
    // so users know exactly which file they are saving.
    let save_label = format!("{}  {}", t(Key::BtnSaveFilePrefix), file.name);
    let save_btn: Element<Message> = {
        let b = button(text(save_label).size(size::BODY))
            .padding(Padding::from(pad::BUTTON_PRIMARY))
            .style(iced::widget::button::primary);
        if dirty {
            b.on_press(Message::FallbackFileSave).into()
        } else {
            b.into()
        }
    };

    let footer = container(
        row![
            column![validity, state_hint].spacing(space::S1),
            Space::new().width(Length::Fill),
            widgets::field(
                t(Key::FallbackStatusLabel),
                click_focus(
                    FallbackField::Status,
                    text_input("200 OK", status)
                        .id(fallback_field_id(FallbackField::Status).unwrap())
                        .on_input(Message::FallbackFileSetStatus)
                        .style(theme::text_input_style)
                        .size(size::CAPTION)
                        .padding(Padding::from([space::S2, space::S3]))
                        .width(Length::Fixed(110.0)),
                ),
            ),
            cursor_wrap(
                app,
                FallbackField::Format,
                button(text(t(Key::FallbackFormatJson)).size(size::LABEL))
                    .on_press(Message::FallbackFileFormat)
                    .padding(Padding::from(pad::BUTTON))
                    .style(iced::widget::button::text)
                    .into(),
            ),
            cursor_wrap(app, FallbackField::Revert, revert_btn),
            cursor_wrap(app, FallbackField::Save, save_btn),
        ]
        .spacing(space::S3)
        .align_y(Alignment::End),
    )
    .padding(Padding::from([space::S3, 0.0]));

    // ── Layout ────────────────────────────────────────────────────────────

    container(
        column![header_col, widgets::divider(), content_card, footer,]
            .spacing(space::S4)
            .padding(Padding::from([space::S4, space::S5])),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
