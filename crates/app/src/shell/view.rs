//! Shell composition (MK-027, v0.6 revision).
//!
//! Layout change from v0.6.0: the side rail is replaced by a horizontal
//! tab bar rendered at the top of the body. This frees the full window
//! width for the Routes three-column layout.
//!
//! Snora AppLayout structure:
//!   header  — top bar (app identity, status chips, actions)
//!   body    — tab bar + screen content stacked vertically
//!   sheet   — bottom drawer, when it is the topmost overlay
//!   dialog / header_menu — whichever overlay `App::topmost_overlay` says
//!     is on top (task 034 / D-22: exactly one of the two, never both —
//!     see `view`'s own comment for why)

use iced::Element;
use iced::widget::column;
use snora::{AppLayout, Dialog, LayoutDirection, Sheet, SheetEdge, SheetSize};

use crate::app::{App, Overlay};
use crate::message::Message;
use crate::screens;
use crate::selection::WorkspaceTab;
use crate::shell;

pub fn view(app: &App) -> Element<'_, Message> {
    let tokens = app.theme_choice.tokens();
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

    let layout = AppLayout::new(body)
        .header(header)
        .direction(LayoutDirection::Ltr);

    // Task 034 (D-22): exactly one overlay construct is attached per
    // frame — whichever `App::topmost_overlay` says is on top — never a
    // `Dialog`/`header_menu` alongside a `Sheet` the way the old code
    // unconditionally attached the workspace menu and the drawer before
    // picking a dialog. The drawer and the workspace menu stay open in
    // `App`'s own state either way; they simply are not drawn on a frame
    // where something ranked above them is. Without this, `snora`'s own
    // fixed layer order (`header_menu` below `dialog` below `sheet`) would
    // make the *last*-attached construct visually dominant regardless of
    // which this app actually considers "on top" — precisely how the
    // drawer ended up visible over the path assistant, or could end up
    // hiding a confirm dialog raised over it, before this fix.
    // `Overlay::ModePicker` has no arm here deliberately (task 035, review
    // note): `App::view` returns `screens::mode_picker::view` before
    // `shell::view::view` is ever called at all (`self.audience_mode.is_none()`
    // short-circuits first), so this function can never observe it — an
    // arm for it would be dead code, not a safety net.
    match app.topmost_overlay() {
        Some(Overlay::ModePicker) => unreachable!(
            "App::view returns the mode picker before shell::view::view is ever called"
        ),
        Some(Overlay::ConfirmDialog) => snora::design::render(
            layout
                .dialog(Dialog::new(screens::confirm_dialog::view(app)))
                .on_close_modals(Message::ConfirmCancel),
            &tokens,
        ),
        Some(Overlay::CommandPalette) => snora::design::render(
            layout
                .dialog(Dialog::new(screens::command_palette::view(app)))
                .on_close_modals(Message::ToggleCommandPalette),
            &tokens,
        ),
        Some(Overlay::TestRule) => snora::design::render(
            layout
                .dialog(Dialog::new(screens::test_rule::view(app)))
                .on_close_modals(Message::TestRuleClose),
            &tokens,
        ),
        Some(Overlay::PathAssistant) => snora::design::render(
            layout
                .dialog(Dialog::new(screens::dotted_path::view(app)))
                .on_close_modals(Message::PathAssistantClose),
            &tokens,
        ),
        Some(Overlay::WorkspaceMenu) => snora::design::render(
            layout
                .header_menu(screens::workspace_menu::view(app))
                .on_close_menus(Message::CloseWorkspaceMenu),
            &tokens,
        ),
        Some(Overlay::Drawer) => {
            let sheet = Sheet::new(shell::bottom_drawer::view(app))
                .at(SheetEdge::Bottom)
                .with_size(SheetSize::Ratio(0.32));
            snora::design::render(
                layout.sheet(sheet).on_close_modals(Message::CloseDrawer),
                &tokens,
            )
        }
        None => snora::design::render(layout, &tokens),
    }
}

/// Wraps a non-workspace screen's bare `Element` with the confirm dialog,
/// if one is raised — task 035 (D-22's view dimension). `ConfirmRequest`
/// fires from Dashboard's and the Wizard's own row/button activations too
/// (switching workspace, creating one), not only from inside a workspace,
/// but `App::view` previously routed only `AppView::Workspace` through
/// any overlay-rendering code at all — an undrawable confirmation was a
/// correctness defect for pointer users on its own, before it was ever a
/// keyboard one. A thin host — `AppLayout` with no header, no tab bar,
/// nothing Welcome/Dashboard/Wizard did not already have — rather than
/// growing a dialog layer on each of the three screens separately, which
/// is why this lives here beside `shell::view::view`'s own overlay
/// composition rather than in `App::view` itself.
pub fn with_confirm_dialog_if_raised<'a>(
    app: &'a App,
    screen: Element<'a, Message>,
) -> Element<'a, Message> {
    if app.confirm_dialog.is_none() {
        return screen;
    }
    let tokens = app.theme_choice.tokens();
    snora::design::render(
        AppLayout::new(screen)
            .dialog(Dialog::new(screens::confirm_dialog::view(app)))
            .on_close_modals(Message::ConfirmCancel),
        &tokens,
    )
}

/// Which of the three mutually-exclusive banners is currently showing —
/// the one source both `feedback_banner` (below) and
/// `regions::feedback_banner_fields` (task 034) branch on, instead of two
/// readings of the same three `if`s that could drift apart (the same D-9
/// lesson `accelerator.rs`/`palette_commands.rs` are already the precedent
/// for).
pub(crate) enum ActiveBanner<'a> {
    Error(&'a apimokka_model::FriendlyProblem),
    Undo,
    Notice,
}

pub(crate) fn active_banner(app: &App) -> Option<ActiveBanner<'_>> {
    if let Some(p) = &app.last_problem {
        Some(ActiveBanner::Error(p))
    } else if !app.undo_stack().is_empty() {
        Some(ActiveBanner::Undo)
    } else if app.notice.is_some() {
        Some(ActiveBanner::Notice)
    } else {
        None
    }
}

/// Same shape as `rule_editor.rs`'s own `cursor_wrap` (task 034).
///
/// D-21 round 2 (task 036, F2): the cursor ring's colour is now a parameter
/// rather than always `theme::cursor_style`, because `cursor_style`'s
/// primary-coloured ring measures 1.13–2.33:1 against `banner_style`'s
/// `warning` fill — below the 3:1 non-text floor in every preset. The error
/// banner's three fields (`DetailsToggle`, `Action`, and `Dismiss` when the
/// error banner is active) pass `theme::banner_cursor_style`; the undo and
/// success banners, on `chip_style`'s neutral fill, keep the generic
/// `theme::cursor_style` — `Dismiss` is shared by all three banners (see
/// its own doc comment), so the choice has to be made by the caller, which
/// already knows which banner is on screen, not by the field alone.
fn feedback_banner_cursor_wrap<'a>(
    app: &App,
    field: crate::regions::FeedbackBannerField,
    cursor_style_fn: fn(&iced::Theme) -> iced::widget::container::Style,
    el: Element<'a, Message>,
) -> Element<'a, Message> {
    let has_cursor = app.focused_region == Some(crate::regions::Region::FeedbackBanner(field))
        && app.region_cursor == Some(crate::regions::Cursor::Index(0));
    iced::widget::container(el)
        .padding(crate::theme::space::S1)
        .style(if has_cursor {
            cursor_style_fn
        } else {
            |_: &iced::Theme| iced::widget::container::Style::default()
        })
        .into()
}

/// MK-039 feedback banner. Priority: friendly error > undo > success notice.
/// Returns None when there is nothing to show.
fn feedback_banner(app: &App) -> Option<Element<'_, Message>> {
    use crate::accelerator::{self, Accelerator};
    use crate::regions::FeedbackBannerField;
    use crate::theme::{self, size, space};
    use apimokka_i18n::Key;
    use iced::widget::{Space, button, container, row, text};
    use iced::{Alignment, Length, Padding};

    // 1. Friendly error (highest priority)
    if let Some(ActiveBanner::Error(p)) = active_banner(app) {
        // Build the detail column: plain detail always; technical detail either
        // inline (Expert, or expanded) or behind a Show/Hide toggle (Guided).
        // D-21 (task 036): `detail`/`technical_detail` took `theme::muted`,
        // guaranteed only against the three neutral surfaces, not against
        // this banner's `warning` fill. `theme::banner_text` is the role
        // snora does guarantee here — see its own doc comment for why both
        // of this banner's text roles share one colour.
        let mut detail_col = iced::widget::column![
            text(p.title.as_str()).size(size::BODY_STRONG),
            text(p.detail.as_str())
                .size(size::BODY_SMALL)
                .line_height(theme::line_height::body_small())
                .color(theme::banner_text(&app.theme())),
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
                        .color(theme::banner_text(&app.theme())),
                );
            }
            // A toggle is offered in either mode so the user can flip it.
            let toggle_label = if expanded {
                app.t(Key::ErrorHideDetails)
            } else {
                app.t(Key::ErrorShowDetails)
            };
            detail_col = detail_col.push(feedback_banner_cursor_wrap(
                app,
                FeedbackBannerField::DetailsToggle,
                theme::banner_cursor_style,
                button(text(toggle_label).size(size::LABEL))
                    .on_press(Message::ToggleProblemDetails)
                    .padding(Padding::from([2.0, space::S1]))
                    .style(theme::banner_text_button)
                    .into(),
            ));
        }

        let mut r = row![text("!").size(size::BODY), detail_col]
            .spacing(space::S3)
            .align_y(Alignment::Center);

        if let Some(label) = &p.action_label {
            r = r.push(feedback_banner_cursor_wrap(
                app,
                FeedbackBannerField::Action,
                theme::banner_cursor_style,
                button(text(label.as_str()).size(size::LABEL))
                    .on_press(Message::ProblemAction)
                    .padding(Padding::from([space::S1, space::S3]))
                    .into(),
            ));
        }
        r = r.push(feedback_banner_cursor_wrap(
            app,
            FeedbackBannerField::Dismiss,
            theme::banner_cursor_style,
            button(text("✕").size(size::LABEL))
                .on_press(Message::DismissProblem)
                .padding(Padding::from([space::S1, space::S2]))
                .style(theme::banner_text_button)
                .into(),
        ));

        return Some(
            container(r)
                .padding(Padding::from([space::S2, space::S5]))
                .width(Length::Fill)
                .style(theme::banner_style)
                .into(),
        );
    }

    // 2. Undo (after a reversible action) — driven by the top of the stack
    if matches!(active_banner(app), Some(ActiveBanner::Undo))
        && let Some(cmd) = app.undo_stack().last()
    {
        let label = app.t(cmd.banner_key());
        let r = row![
            text(label).size(size::BODY).width(Length::Fill),
            feedback_banner_cursor_wrap(
                app,
                FeedbackBannerField::Undo,
                theme::cursor_style,
                button(
                    row![
                        text(app.t(Key::UndoLabel)).size(size::LABEL),
                        // D-11: was a hard-coded " ⌘Z" -- advertised a macOS-only
                        // key on every platform. `accelerator::display` is the
                        // single source `screens/command_palette.rs` already
                        // reads for the same binding; this is the one site task
                        // 002 missed. Leading space kept to preserve the
                        // existing gap rather than restructuring with spacing.
                        text(format!(" {}", accelerator::display(Accelerator::Undo)))
                            .size(size::LABEL)
                            .color(theme::muted(&app.theme())),
                    ]
                    .spacing(2),
                )
                .on_press(Message::Undo)
                .padding(Padding::from([space::S1, space::S3]))
                .into(),
            ),
            feedback_banner_cursor_wrap(
                app,
                FeedbackBannerField::Dismiss,
                theme::cursor_style,
                button(text("✕").size(size::LABEL))
                    .on_press(Message::DismissNotice)
                    .padding(Padding::from([space::S1, space::S2]))
                    .style(iced::widget::button::text)
                    .into(),
            ),
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
    if matches!(active_banner(app), Some(ActiveBanner::Notice))
        && let Some(notice) = &app.notice
    {
        let r = row![
            text("✓").size(size::BODY),
            text(notice.as_str()).size(size::BODY).width(Length::Fill),
            Space::new().width(Length::Fixed(0.0)),
            feedback_banner_cursor_wrap(
                app,
                FeedbackBannerField::Dismiss,
                theme::cursor_style,
                button(text("✕").size(size::LABEL))
                    .on_press(Message::DismissNotice)
                    .padding(Padding::from([space::S1, space::S2]))
                    .style(iced::widget::button::text)
                    .into(),
            ),
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
