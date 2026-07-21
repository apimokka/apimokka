//! Shell composition (MK-027, v0.6 revision).
//!
//! Layout change from v0.6.0: the side rail is replaced by a horizontal
//! tab bar rendered at the top of the body. This frees the full window
//! width for the Routes three-column layout.
//!
//! Snora AppLayout structure:
//!   header  — top bar (app identity, status chips, actions)
//!   body    — tab bar + screen content stacked vertically
//!   sheet   — bottom drawer (when open)
//!   dialog  — overlay stack (confirm > palette > test-rule > path-assistant)

use iced::Element;
use iced::widget::column;
use snora::{AppLayout, Dialog, LayoutDirection, Sheet, SheetEdge, SheetSize};

use crate::app::App;
use crate::message::Message;
use crate::screens;
use crate::selection::WorkspaceTab;
use crate::shell;

pub fn view(app: &App) -> Element<'_, Message> {
    let header = shell::top_bar::view(app);
    let tab_bar = shell::tab_bar::view(app);

    // Screen content
    let screen: Element<Message> = match app.tab {
        WorkspaceTab::Routes => screens::routes::view(app),
        WorkspaceTab::Trace => screens::trace::view(app),
        WorkspaceTab::Settings => screens::settings::view(app),
    };

    // MK-039 feedback banner: friendly error > undo > success notice.
    // Sits between the tab bar and the screen so it is always visible
    // without obscuring content.
    let body: Element<Message> = match feedback_banner(app) {
        Some(banner) => column![tab_bar, banner, screen].into(),
        None => column![tab_bar, screen].into(),
    };

    let mut layout = AppLayout::new(body)
        .header(header)
        .direction(LayoutDirection::Ltr);

    // Workspace menu (snora header_menu)
    if app.workspace_menu_open {
        let menu = screens::workspace_menu::view(app);
        layout = layout
            .header_menu(menu)
            .on_close_menus(Message::CloseWorkspaceMenu);
    }

    // Bottom drawer
    if app.drawer.is_some() {
        let drawer_el = shell::bottom_drawer::view(app);
        let sheet = Sheet::new(drawer_el)
            .at(SheetEdge::Bottom)
            .with_size(SheetSize::Ratio(0.32));
        layout = layout.sheet(sheet).on_close_modals(Message::CloseDrawer);
    }

    // Dialog priority order (MK-021):
    // 0. First-run audience mode picker (MK-040) — highest priority and NOT
    //    dismissible: the user must choose before using the app. No
    //    on_close_modals sink, so Esc / backdrop cannot close it.
    if app.audience_mode.is_none() {
        return snora::render(layout.dialog(Dialog::new(screens::mode_picker::view(app))));
    }
    // 1. Confirm dialog
    if app.confirm_dialog.is_some() {
        return snora::render(
            layout
                .dialog(Dialog::new(screens::confirm_dialog::view(app)))
                .on_close_modals(Message::ConfirmCancel),
        );
    }
    // 2. Command palette
    if app.command_palette.open {
        return snora::render(
            layout
                .dialog(Dialog::new(screens::command_palette::view(app)))
                .on_close_modals(Message::ToggleCommandPalette),
        );
    }
    // 3. Test rule
    if app.test_rule.open {
        return snora::render(
            layout
                .dialog(Dialog::new(screens::test_rule::view(app)))
                .on_close_modals(Message::TestRuleClose),
        );
    }
    // 4. Dotted-path assistant
    if app.path_assistant.open {
        return snora::render(
            layout
                .dialog(Dialog::new(screens::dotted_path::view(app)))
                .on_close_modals(Message::PathAssistantClose),
        );
    }

    snora::render(layout)
}

/// MK-039 feedback banner. Priority: friendly error > undo > success notice.
/// Returns None when there is nothing to show.
fn feedback_banner(app: &App) -> Option<Element<'_, Message>> {
    use crate::theme::{self, size, space};
    use apimokka_i18n::Key;
    use iced::widget::{Space, button, container, row, text};
    use iced::{Alignment, Length, Padding};

    // 1. Friendly error (highest priority)
    if let Some(p) = &app.last_problem {
        // Build the detail column: plain detail always; technical detail either
        // inline (Expert, or expanded) or behind a Show/Hide toggle (Guided).
        let mut detail_col = iced::widget::column![
            text(p.title.as_str()).size(size::BODY_STRONG),
            text(p.detail.as_str())
                .size(size::CAPTION)
                .color(theme::muted(&app.theme())),
        ]
        .spacing(space::S1)
        .width(Length::Fill);

        if let Some(tech) = &p.technical_detail {
            // Expert sees it inline by default; Guided collapses behind a toggle.
            let expanded = app.show_problem_details;
            if expanded {
                detail_col = detail_col.push(
                    text(tech.as_str())
                        .size(size::CAPTION)
                        .font(iced::Font::MONOSPACE)
                        .color(theme::muted(&app.theme())),
                );
            }
            // A toggle is offered in either mode so the user can flip it.
            let toggle_label = if expanded {
                app.t(Key::ErrorHideDetails)
            } else {
                app.t(Key::ErrorShowDetails)
            };
            detail_col = detail_col.push(
                button(text(toggle_label).size(size::CAPTION))
                    .on_press(Message::ToggleProblemDetails)
                    .padding(Padding::from([2.0, space::S1]))
                    .style(iced::widget::button::text),
            );
        }

        let mut r = row![text("!").size(size::BODY), detail_col]
            .spacing(space::S3)
            .align_y(Alignment::Center);

        if let Some(label) = &p.action_label {
            r = r.push(
                button(text(label.as_str()).size(size::CAPTION))
                    .on_press(Message::ProblemAction)
                    .padding(Padding::from([space::S1, space::S3])),
            );
        }
        r = r.push(
            button(text("✕").size(size::CAPTION))
                .on_press(Message::DismissProblem)
                .padding(Padding::from([space::S1, space::S2]))
                .style(iced::widget::button::text),
        );

        return Some(
            container(r)
                .padding(Padding::from([space::S2, space::S5]))
                .width(Length::Fill)
                .style(theme::banner_style)
                .into(),
        );
    }

    // 2. Undo (after a reversible action) — driven by the top of the stack
    if let Some(cmd) = app.undo_stack().last() {
        let label = app.t(cmd.banner_key());
        let r = row![
            text(label).size(size::BODY).width(Length::Fill),
            button(
                row![
                    text(app.t(Key::UndoLabel)).size(size::CAPTION),
                    text(" ⌘Z")
                        .size(size::CAPTION)
                        .color(theme::muted(&app.theme())),
                ]
                .spacing(2),
            )
            .on_press(Message::Undo)
            .padding(Padding::from([space::S1, space::S3])),
            button(text("✕").size(size::CAPTION))
                .on_press(Message::DismissNotice)
                .padding(Padding::from([space::S1, space::S2]))
                .style(iced::widget::button::text),
        ]
        .spacing(space::S3)
        .align_y(Alignment::Center);

        return Some(
            container(r)
                .padding(Padding::from([space::S2, space::S5]))
                .width(Length::Fill)
                .style(theme::chip_style)
                .into(),
        );
    }

    // 3. Success / info notice (lowest priority)
    if let Some(notice) = &app.notice {
        let r = row![
            text("✓").size(size::BODY),
            text(notice.as_str()).size(size::BODY).width(Length::Fill),
            Space::new().width(Length::Fixed(0.0)),
            button(text("✕").size(size::CAPTION))
                .on_press(Message::DismissNotice)
                .padding(Padding::from([space::S1, space::S2]))
                .style(iced::widget::button::text),
        ]
        .spacing(space::S3)
        .align_y(Alignment::Center);

        return Some(
            container(r)
                .padding(Padding::from([space::S2, space::S5]))
                .width(Length::Fill)
                .style(theme::chip_style)
                .into(),
        );
    }

    None
}
