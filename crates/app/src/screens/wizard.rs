//! MK-026 — S-02 Wizard.
use crate::app::App;
use crate::message::Message;
use crate::regions::{Region, WizardField, wizard_field_id};
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use iced::widget::{
    Space, button, checkbox, column, container, mouse_area, row, scrollable, text, text_input,
};
use iced::{Alignment, Element, Length, Padding};

/// Task 032 (D-20): the `Id` `App::update_and_dispatch` scrolls when the
/// Wizard's keyboard cursor moves — same D-5 approximation the
/// sidebar/Dashboard/palette scrollables already use.
pub const SCROLLABLE_ID: &str = "mk023-wizard";

/// `cursor_style`'s border is drawn at the wrapping container's own bounds;
/// without a gap an opaque sibling paints over it — the fix task 028 §6
/// found, reused here for the Wizard's one-item regions (task 031).
fn cursor_wrap<'a>(
    app: &App,
    field: WizardField,
    el: Element<'a, Message>,
) -> Element<'a, Message> {
    let has_cursor = app.focused_region == Some(Region::Wizard(field))
        && app.region_cursor == Some(crate::regions::Cursor::Index(0));
    container(el)
        .padding(space::S1)
        .style(if has_cursor {
            theme::cursor_style
        } else {
            |_: &iced::Theme| iced::widget::container::Style::default()
        })
        .into()
}

/// Same as `cursor_wrap`, for one item of the `Starter` group at position
/// `index` within it.
fn cursor_wrap_group<'a>(
    app: &App,
    field: WizardField,
    index: usize,
    el: Element<'a, Message>,
) -> Element<'a, Message> {
    let has_cursor = app.focused_region == Some(Region::Wizard(field))
        && app.region_cursor == Some(crate::regions::Cursor::Index(index));
    container(el)
        .padding(space::S1)
        .style(if has_cursor {
            theme::cursor_style
        } else {
            |_: &iced::Theme| iced::widget::container::Style::default()
        })
        .into()
}

/// D-18 (task 032): a click on a text-field region's own widget enters that
/// region, via `on_release` (see `Message::RegionClicked`'s doc comment for
/// why `on_press` would not fire on a click landing on the field itself).
fn click_focus<'a>(
    field: WizardField,
    el: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    mouse_area(el)
        .on_release(Message::RegionClicked(Region::Wizard(field)))
        .into()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let w = &app.wizard;
    let t = |k| app.t(k);

    // Required fields
    let required = column![
        widgets::field(
            t(Key::WizardFieldName),
            cursor_wrap(
                app,
                WizardField::Name,
                click_focus(
                    WizardField::Name,
                    text_input("payments-mock", &w.name)
                        .id(wizard_field_id(WizardField::Name).unwrap())
                        .on_input(Message::WizardSetName)
                        .style(theme::text_input_style)
                        .size(size::BODY)
                        .padding(Padding::from([space::S2, space::S3]))
                        .width(Length::Fill),
                )
            )
        ),
        widgets::field(
            t(Key::WizardFieldFolder),
            cursor_wrap(
                app,
                WizardField::Folder,
                click_focus(
                    WizardField::Folder,
                    text_input("~/projects", &w.folder)
                        .id(wizard_field_id(WizardField::Folder).unwrap())
                        .on_input(Message::WizardSetFolder)
                        .style(theme::text_input_style)
                        .size(size::BODY)
                        .padding(Padding::from([space::S2, space::S3]))
                        .width(Length::Fill),
                )
            )
        ),
    ]
    .spacing(space::S3);

    // Server section
    let server = collapsible(
        app,
        WizardField::ServerToggle,
        t(Key::WizardSectionServer),
        t(Key::WizardSectionServerHint),
        0,
        column![
            row![
                widgets::field(
                    t(Key::WizardFieldHost),
                    cursor_wrap(
                        app,
                        WizardField::Host,
                        click_focus(
                            WizardField::Host,
                            text_input("127.0.0.1", &w.host)
                                .id(wizard_field_id(WizardField::Host).unwrap())
                                .on_input(Message::WizardSetHost)
                                .style(theme::text_input_style)
                                .size(size::BODY)
                                .padding(Padding::from([space::S2, space::S3]))
                                .width(Length::Fill),
                        )
                    )
                ),
                widgets::field(
                    t(Key::WizardFieldPort),
                    cursor_wrap(
                        app,
                        WizardField::Port,
                        click_focus(
                            WizardField::Port,
                            text_input("8080", &w.port)
                                .id(wizard_field_id(WizardField::Port).unwrap())
                                .on_input(Message::WizardSetPort)
                                .style(theme::text_input_style)
                                .size(size::BODY)
                                .padding(Padding::from([space::S2, space::S3]))
                                .width(Length::Fixed(100.0)),
                        )
                    )
                ),
            ]
            .spacing(space::S3)
            .align_y(Alignment::End),
            cursor_wrap(
                app,
                WizardField::Tls,
                checkbox(w.tls)
                    .label(t(Key::WizardFieldTls))
                    .on_toggle(Message::WizardSetTls)
                    .size(size::BODY)
                    .into()
            ),
        ]
        .spacing(space::S3)
        .into(),
    );

    // Starter section
    let starter = collapsible(
        app,
        WizardField::StarterToggle,
        t(Key::WizardSectionStarter),
        t(Key::WizardSectionStarterHint),
        1,
        column![
            cursor_wrap_group(
                app,
                WizardField::Starter,
                0,
                iced::widget::radio(
                    t(Key::WizardStarterMinimal),
                    crate::app::WizardStarter::Minimal,
                    Some(w.starter),
                    Message::WizardSetStarter,
                )
                .size(size::BODY)
                .into(),
            ),
            cursor_wrap_group(
                app,
                WizardField::Starter,
                1,
                iced::widget::radio(
                    t(Key::WizardStarterShopApi),
                    crate::app::WizardStarter::ShopApi,
                    Some(w.starter),
                    Message::WizardSetStarter,
                )
                .size(size::BODY)
                .into(),
            ),
            cursor_wrap_group(
                app,
                WizardField::Starter,
                2,
                iced::widget::radio(
                    t(Key::WizardStarterEmpty),
                    crate::app::WizardStarter::Empty,
                    Some(w.starter),
                    Message::WizardSetStarter,
                )
                .size(size::BODY)
                .into(),
            ),
        ]
        .spacing(space::S3)
        .into(),
    );

    // Trace section
    let trace = collapsible(
        app,
        WizardField::TraceToggle,
        t(Key::WizardSectionTrace),
        t(Key::WizardSectionTraceHint),
        2,
        column![
            cursor_wrap(
                app,
                WizardField::TraceEnable,
                checkbox(true)
                    .label(t(Key::WizardTraceEnable))
                    .size(size::BODY)
                    .into(),
            ),
            widgets::field(
                t(Key::WizardQueueSize),
                cursor_wrap(
                    app,
                    WizardField::QueueSize,
                    click_focus(
                        WizardField::QueueSize,
                        text_input("1024", &w.queue_size)
                            .id(wizard_field_id(WizardField::QueueSize).unwrap())
                            .on_input(Message::WizardSetQueueSize)
                            .style(theme::text_input_style)
                            .size(size::BODY)
                            .padding(Padding::from([space::S2, space::S3]))
                            .width(Length::Fixed(120.0)),
                    )
                )
            ),
        ]
        .spacing(space::S3)
        .into(),
    );

    let form = column![
        text(t(Key::WizardTitle)).size(size::TITLE),
        Space::new().height(space::S1),
        text(t(Key::WizardIntroHint))
            .size(size::BODY_SMALL)
            .line_height(theme::line_height::body_small())
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
            cursor_wrap(
                app,
                WizardField::Cancel,
                widgets::ghost_btn(t(Key::BtnCancel), Message::WizardCancel),
            ),
            Space::new().width(Length::Fill),
            cursor_wrap(
                app,
                WizardField::Create,
                widgets::primary_btn(t(Key::BtnCreate), Message::WizardCreate),
            ),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([space::S3, space::S6]))
    .style(theme::panel_style);

    let inner = column![
        scrollable(form).id(SCROLLABLE_ID).height(Length::Fill),
        action_bar,
    ]
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
    toggle_field: WizardField,
    heading: &'a str,
    hint: &'a str,
    index: usize,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    let open = app.wizard.section_open.get(index).copied().unwrap_or(false);
    let chevron = if open { "▾" } else { "▸" };
    // D-3 (M8 capture, found by the project owner, no case covering it):
    // this button had no `.style(...)` and no `.width(...)`, so it fell
    // back to iced's default primary-filled button (a light-blue block in
    // every preset, worst in `high_contrast_dark`) sized to its heading —
    // the longer hint text then overflowed past the fill's right edge, and
    // the three sections ended up different widths. `theme::naked` (the
    // same style `mode_picker.rs`'s cards use for an identical
    // button-wrapping-content shape) and a filled width fix both at once;
    // `theme::muted` on the hint was always correct for a plain background,
    // it only read as wrong because the fill was default-primary underneath it.
    let section_head = cursor_wrap(
        app,
        toggle_field,
        button(
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
        .padding(Padding::from([space::S2, 0.0]))
        .style(theme::naked)
        .width(Length::Fill)
        .into(),
    );

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
