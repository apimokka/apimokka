//! MK-033 — Command palette dialog.
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length, Padding};
use apimokka_i18n::Key;
use crate::app::App;
use crate::message::Message;
use crate::selection::WorkspaceTab;
use crate::theme::{self, size, space};
use crate::widgets;

#[allow(dead_code)]
struct Cmd { label: Key, shortcut: Option<&'static str>, msg: Message }

pub fn view(app: &App) -> Element<'_, Message> {
    let cmds: &[(Key, Option<&'static str>, Message)] = &[
        (Key::PaletteCmdUndo,           Some("⌘Z"),  Message::Undo),
        (Key::PaletteCmdRedo,           Some("⌘⇧Z"), Message::Redo),
        (Key::PaletteCmdSave,           Some("⌘S"),  Message::Save),
        (Key::PaletteCmdAddRule,        None,        Message::AddRuleFromPalette),
        (Key::PaletteCmdAddRuleSet,     None,        Message::AddRuleSet),
        (Key::PaletteCmdTestRule,       None,        Message::TestRuleOpen),
        (Key::PaletteCmdToggleTrace,    None,        Message::ViewAllInTrace),
        (Key::PaletteCmdOpenValidation, None,       Message::OpenValidationDrawer),
        (Key::PaletteCmdOpenSaveDiff,   None,       Message::OpenSaveDiffDrawer),
        (Key::PaletteCmdStartServer,    None,       Message::StartStopServer),
        (Key::PaletteCmdReload,         Some("⌘R"), Message::ReloadConfig),
        (Key::PaletteCmdRestart,        None,       Message::RestartServer),
        (Key::PaletteCmdSwitchWorkspace,None,       Message::ToggleWorkspaceMenu),
        (Key::PaletteCmdSettings,       None,       Message::SwitchTab(WorkspaceTab::Settings)),
        (Key::PaletteCmdToggleTheme,    None,       Message::ToggleTheme),
        (Key::PaletteCmdGoRoutes,       None,       Message::SwitchTab(WorkspaceTab::Routes)),
        (Key::PaletteCmdGoTrace,        None,       Message::SwitchTab(WorkspaceTab::Trace)),
        (Key::PaletteCmdGoSettings,     None,       Message::SwitchTab(WorkspaceTab::Settings)),
    ];

    let q = app.command_palette.query.to_lowercase();
    let rows: Vec<Element<Message>> = cmds.iter()
        .filter(|(label_key, _, _)| {
            q.is_empty() || app.t(*label_key).to_lowercase().contains(&q)
        })
        .map(|(label_key, shortcut, msg)| {
            let shortcut_el: Element<Message> = if let Some(sc) = shortcut {
                container(text(*sc).size(size::CAPTION))
                    .padding(Padding::from([2.0, 8.0]))
                    .style(theme::chip_style)
                    .into()
            } else {
                Space::new().width(0.0).into()
            };
            button(
                container(
                    row![
                        text(app.t(*label_key)).size(size::BODY).width(Length::Fill),
                        shortcut_el,
                    ]
                    .align_y(Alignment::Center),
                )
                .padding(Padding::from([space::S3, space::S4]))
                .style(theme::card_style)
                .width(Length::Fill),
            )
            .on_press(msg.clone())
            .padding(0).style(theme::naked).style(theme::naked)
            .width(Length::Fill)
            .into()
        })
        .collect();

    let list: Element<Message> = if rows.is_empty() {
        widgets::empty_state(app.t(Key::PaletteNoMatch))
    } else {
        scrollable(column(rows).spacing(space::S1))
            .height(Length::Fixed(320.0))
            .into()
    };

    container(
        column![
            // Header with shortcut hints
            row![
                text(app.t(Key::PaletteTitle)).size(size::SECTION).width(Length::Fill),
                container(text("⌘K").size(size::CAPTION)).padding(Padding::from([2.0, 8.0])).style(theme::chip_style),
                container(text("Esc").size(size::CAPTION)).padding(Padding::from([2.0, 8.0])).style(theme::chip_style),
                Space::new().width(space::S2),
                button(text("✕").size(size::BODY))
                    .on_press(Message::ToggleCommandPalette)
                    .padding(Padding::from([space::S1, space::S2])),
            ]
            .spacing(space::S2)
            .align_y(Alignment::Center),
            Space::new().height(space::S2),
            text_input(app.t(Key::PaletteSearch), &app.command_palette.query)
                .on_input(Message::PaletteQuery)
                .size(size::BODY)
                .padding(Padding::from([space::S3, space::S4]))
                .width(Length::Fill),
            Space::new().height(space::S1),
            widgets::divider(),
            list,
        ]
        .spacing(space::S2)
        .padding(space::S5)
        .width(Length::Fixed(540.0)),
    )
    .style(theme::dialog_style)
    .into()
}
