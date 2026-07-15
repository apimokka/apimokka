//! MK-030 — S-13 Settings.
use crate::app::App;
use crate::message::Message;
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::{Key, Locale};
use iced::widget::{Space, button, checkbox, column, container, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length, Padding};

pub fn view(app: &App) -> Element<'_, Message> {
    let snap = match &app.snapshot {
        Some(s) => s,
        None => return widgets::empty_state("No workspace open."),
    };
    let s = &snap.root_settings;

    // ── Appearance section controls ───────────────────────────────────────
    let theme_btns: Element<Message> = {
        use crate::app::ThemeChoice;
        // MK-050: four-option theme picker (Light / Dark / HC Light / HC Dark).
        let mut row_el = row![].spacing(space::S1);
        for choice in ThemeChoice::all() {
            let selected = app.theme_choice == choice;
            row_el = row_el.push(
                button(text(app.t(choice.label_key())).size(size::CAPTION))
                    .on_press_maybe(if selected {
                        None
                    } else {
                        Some(Message::SetTheme(choice))
                    })
                    .padding(Padding::from([space::S2, space::S3]))
                    .style(if selected {
                        theme::seg_active
                    } else {
                        theme::seg_inactive
                    }),
            );
        }
        row_el.wrap().into()
    };

    let guidance_btns: Element<Message> = {
        use apimokka_model::AudienceMode;
        let guided = matches!(app.audience_mode, Some(AudienceMode::Guided));
        let expert = matches!(app.audience_mode, Some(AudienceMode::Expert));
        row![
            button(text(app.t(Key::ModeGuidedTitle)).size(size::BODY))
                .on_press(Message::ChooseAudienceMode(AudienceMode::Guided))
                .padding(Padding::from([space::S2, space::S4]))
                .style(if guided {
                    theme::seg_active
                } else {
                    theme::seg_inactive
                }),
            button(text(app.t(Key::ModeExpertTitle)).size(size::BODY))
                .on_press(Message::ChooseAudienceMode(AudienceMode::Expert))
                .padding(Padding::from([space::S2, space::S4]))
                .style(if expert {
                    theme::seg_active
                } else {
                    theme::seg_inactive
                }),
        ]
        .spacing(space::S1)
        .into()
    };

    // ── Build page column imperatively so mode-aware sections can be pushed ─
    let mut page: iced::widget::Column<Message> = column![]
        .spacing(space::S4)
        .padding(Padding::from([space::S5, space::S6]));

    page = page.push(text(app.t(Key::SettingsTitle)).size(size::TITLE));

    // Always visible: Appearance + Server
    page = page.push(section(
        app,
        Key::SettingsSectionAppearance,
        Key::SettingsImpactSaveOnly,
        column![
            widgets::field(app.t(Key::SettingsTheme), theme_btns),
            widgets::field(app.t(Key::SettingsAudienceMode), guidance_btns),
            widgets::field(
                app.t(Key::NavSettings),
                pick_list(
                    Locale::all().to_vec(),
                    Some(app.locale),
                    Message::ChangeLocale
                )
                .text_size(size::BODY)
                .padding(Padding::from([space::S2, space::S3]))
                .width(Length::Fixed(100.0))
                .into()
            ),
            row![
                text(app.t(Key::SettingsKeyboardSection))
                    .size(size::BODY)
                    .width(Length::Fill),
                text(app.t(Key::SettingsPaletteShortcut))
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(space::S3)
        .into(),
    ));

    page = page.push(section(
        app,
        Key::SettingsSectionServer,
        Key::SettingsImpactRestart,
        column![
            row![
                widgets::field(
                    app.t(Key::SettingsHost),
                    text_input("127.0.0.1", &s.listener_ip)
                        .on_input(Message::SettingsSetHost)
                        .size(size::BODY)
                        .padding(Padding::from([space::S2, space::S3]))
                        .width(Length::Fill)
                        .into()
                ),
                Space::new().width(space::S3),
                widgets::field(
                    app.t(Key::SettingsPort),
                    text_input("8080", &s.listener_port.to_string())
                        .on_input(Message::SettingsSetPort)
                        .size(size::BODY)
                        .padding(Padding::from([space::S2, space::S3]))
                        .width(Length::Fixed(100.0))
                        .into()
                ),
            ]
            .align_y(Alignment::End),
            checkbox(s.tls_enabled)
                .label(app.t(Key::SettingsTls))
                .on_toggle(Message::SettingsSetTls)
                .size(size::BODY),
        ]
        .spacing(space::S3)
        .into(),
    ));

    // MK-041: Logs + Trace — always visible in Expert, gated in Guided.
    if app.shows_scaffolding() {
        let (chevron, label) = if app.settings_advanced_more {
            ("▾", app.t(Key::LayoutFewerSettings))
        } else {
            ("▸", app.t(Key::LayoutMoreSettings))
        };
        let toggle_btn = button(
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
        .on_press(Message::ToggleSettingsAdvancedMore)
        .padding(Padding::from([space::S2, space::S3]))
        .style(iced::widget::button::text);

        if app.settings_advanced_more {
            page = push_logs_trace_sections(app, s, page);
            page = page.push(toggle_btn);
        } else {
            page = page.push(toggle_btn);
        }
    } else {
        page = push_logs_trace_sections(app, s, page);
    }

    page = page.push(Space::new().height(space::S4));
    iced::widget::scrollable(page).height(Length::Fill).into()
}

fn section<'a>(
    app: &'a App,
    heading: Key,
    impact: Key,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    column![
        row![
            text(app.t(heading)).size(size::SECTION).width(Length::Fill),
            container(text(app.t(impact)).size(size::CAPTION))
                .padding(Padding::from([2.0, 8.0]))
                .style(theme::chip_style),
        ]
        .align_y(Alignment::Center),
        container(body)
            .padding(Padding::from([space::S4, space::S5]))
            .style(theme::card_style)
            .width(Length::Fill),
    ]
    .spacing(space::S2)
    .into()
}

/// MK-041: push the Logs and Trace sections onto `page`. Extracted as a
/// free function so the borrow checker can see that both callers (Expert
/// always, Guided when expanded) build the same sections.
fn push_logs_trace_sections<'a>(
    app: &'a App,
    s: &'a apimokka_model::RootSettings,
    mut page: iced::widget::Column<'a, Message>,
) -> iced::widget::Column<'a, Message> {
    use crate::theme::{size, space};
    use crate::widgets;
    use apimokka_i18n::Key;
    use iced::widget::{Space, checkbox, column, text_input};
    use iced::{Length, Padding};

    page = page.push(section(
        app,
        Key::SettingsSectionLogs,
        Key::SettingsImpactReload,
        widgets::field(
            app.t(Key::SettingsLogFile),
            text_input("", &s.log_file)
                .size(size::BODY)
                .padding(Padding::from([space::S2, space::S3]))
                .width(Length::Fill)
                .into(),
        ),
    ));
    page = page.push(section(
        app,
        Key::SettingsSectionTrace,
        Key::SettingsImpactReload,
        column![
            checkbox(s.trace_enabled)
                .label(app.t(Key::SettingsTraceEnable))
                .on_toggle(crate::message::Message::SettingsSetTraceEnabled)
                .size(size::BODY),
            widgets::field(
                app.t(Key::SettingsTraceQueueSize),
                text_input("1024", &s.trace_queue_size.to_string())
                    .size(size::BODY)
                    .padding(Padding::from([space::S2, space::S3]))
                    .width(Length::Fixed(120.0))
                    .into()
            ),
            Space::new().height(0.0),
        ]
        .spacing(space::S3)
        .into(),
    ));
    page
}
