//! Left sidebar — rule-set accordion, fallback files, middleware scripts.
//!
//! Task 029 (MK-023 slice 4): rendered from `regions::sidebar_rows(app)`,
//! the same flattened row list `Region::Sidebar`'s cursor logic reads — not
//! a parallel reimplementation of the same iteration, per the D-9 lesson
//! this programme keeps re-applying. The dividers and the "Rule sets" label
//! are purely decorative and not rows in that list; they are inserted here,
//! around the one `SidebarRow::FallbackHeader`/`SidebarRow::MiddlewareHeader`
//! each appear exactly once.

use crate::app::App;
use crate::message::Message;
use crate::regions::{Cursor, Region, SidebarRow, sidebar_rows};
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Color, Element, Length, Padding};

pub(super) fn left_sidebar(app: &App) -> Element<'_, Message> {
    let t = |k| app.t(k);
    let Some(snap) = &app.snapshot else {
        return container(widgets::empty_state(t(Key::EmptyNoWorkspaceOpen)))
            .width(Length::Fixed(280.0))
            .height(Length::Fill)
            .style(theme::panel_style)
            .into();
    };

    let mut col = column![]
        .spacing(space::S1)
        .padding(Padding::from([space::S3, space::S2]));

    col = col.push(
        text(t(Key::RoutesRuleSets))
            .size(size::LABEL)
            .color(theme::muted(&app.theme())),
    );

    // `FallbackHeader` and `MiddlewareHeader` each appear exactly once in
    // `sidebar_rows`' output, open or not — the divider ahead of each is
    // rendered right alongside its header, not anchored to whichever row
    // happens to precede it (which depends on the *other* section's open
    // state and would otherwise have to be tracked separately).
    for row_kind in sidebar_rows(app) {
        match &row_kind {
            SidebarRow::RuleSetHeader(id) => {
                if let Some(rs) = snap.find_rule_set(*id) {
                    let is_open = app.rule_set_open == Some(*id);
                    col = col.push(cursor_wrap(
                        app,
                        &row_kind,
                        rule_set_header_row(app, rs, is_open),
                    ));
                }
            }
            SidebarRow::Rule(id) => {
                if let Some((_, rule)) = snap.find_rule(*id) {
                    col = col.push(cursor_wrap(app, &row_kind, rule_row(app, rule)));
                }
            }
            SidebarRow::AddRule(rs_id) => {
                col = col.push(cursor_wrap(app, &row_kind, add_rule_row(app, *rs_id)));
            }
            SidebarRow::AddRuleSet => {
                col = col.push(cursor_wrap(app, &row_kind, add_rule_set_row(app)));
            }
            SidebarRow::FallbackHeader => {
                col = col.push(widgets::divider());
                col = col.push(cursor_wrap(
                    app,
                    &row_kind,
                    fallback_header_row(app, snap.fallback_files.len(), app.fallback_section_open),
                ));
            }
            SidebarRow::FallbackFile(path) => {
                if let Some(f) = snap.fallback_files.iter().find(|f| &f.path == path) {
                    col = col.push(cursor_wrap(app, &row_kind, fallback_file_row(app, f)));
                }
            }
            SidebarRow::AddFallbackFile => {
                col = col.push(add_fallback_file_row(app));
            }
            SidebarRow::MiddlewareHeader => {
                col = col.push(widgets::divider());
                col = col.push(cursor_wrap(
                    app,
                    &row_kind,
                    middleware_header_row(
                        app,
                        snap.middleware_scripts.len(),
                        app.middleware_section_open,
                    ),
                ));
            }
            SidebarRow::MiddlewareScript(path) => {
                if let Some(s) = snap.middleware_scripts.iter().find(|s| &s.path == path) {
                    col = col.push(cursor_wrap(app, &row_kind, middleware_script_row(app, s)));
                }
            }
            SidebarRow::AddScript => {
                col = col.push(add_script_row(app));
            }
        }
    }

    container(
        scrollable(col)
            .id(super::SIDEBAR_SCROLLABLE_ID)
            .height(Length::Fill),
    )
    .width(Length::Fixed(280.0))
    .height(Length::Fill)
    .style(theme::panel_style)
    .into()
}

/// `cursor_style`'s border is drawn at the wrapping container's own bounds;
/// without a gap, a row's own opaque `card_style`/`card_selected_style` (or
/// a plain `button::text` row's background on hover) paints over it, the
/// same defect task 028 §6 found and fixed for Welcome and Dashboard.
/// `space::S1` gives it room. Kept as a small local helper rather than a
/// `theme::` export — task 028's review preferred the reasoning repeated at
/// each of three call sites over a named abstraction; with the sidebar's
/// eight row kinds all needing it, one local function keeps that same
/// reasoning written once rather than copied eight times, without hiding it
/// behind a cross-module name.
fn cursor_wrap<'a>(app: &App, row: &SidebarRow, el: Element<'a, Message>) -> Element<'a, Message> {
    let has_cursor = app.focused_region == Some(Region::Sidebar)
        && app.region_cursor == Some(Cursor::Sidebar(row.clone()));
    container(el)
        .padding(space::S1)
        .style(if has_cursor {
            theme::cursor_style
        } else {
            |_: &iced::Theme| iced::widget::container::Style::default()
        })
        .into()
}

fn rule_set_header_row<'a>(
    app: &'a App,
    rs: &'a apimokka_model::snapshot::RuleSetView,
    is_open: bool,
) -> Element<'a, Message> {
    let rs_selected = app.selection.rule_set == Some(rs.id);
    let file_name: &str = rs.file.path.rsplit('/').next().unwrap_or(&rs.file.path);
    let rule_count = rs.rules.len();
    let chevron = if is_open { "▾" } else { "▸" };

    let dirty_el: Element<Message> = if rs.file.dirty {
        widgets::dirty_dot()
    } else {
        Space::new().width(0.0).into()
    };

    button(
        container(
            row![
                text(chevron)
                    .size(size::LABEL)
                    .color(theme::muted(&app.theme())),
                text(file_name).size(size::BODY).width(Length::Fill),
                text(format!("({})", rule_count))
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
                dirty_el,
            ]
            .spacing(space::S2)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([space::S2, space::S3]))
        .style(if rs_selected {
            theme::card_parent_selected_style
        } else {
            theme::card_style
        })
        .width(Length::Fill),
    )
    .on_press(Message::SelectRuleSet(rs.id))
    .padding(0)
    .style(theme::naked)
    .width(Length::Fill)
    .into()
}

fn rule_row<'a>(
    app: &'a App,
    rule: &'a apimokka_model::snapshot::RuleView,
) -> Element<'a, Message> {
    let rule_selected = app.selection.rule == Some(rule.id);
    let has_issues = !rule.validation.issues.is_empty();
    let status_glyph: Element<Message> = if has_issues {
        text("⚠")
            .size(size::LABEL)
            .color(Color::from_rgb(0.85, 0.45, 0.0))
            .into()
    } else if rule.matched_by_latest_trace {
        text("✓")
            .size(size::LABEL)
            .color(Color::from_rgb(0.10, 0.65, 0.10))
            .into()
    } else {
        Space::new().width(0.0).into()
    };

    let summary = rule.summary();
    button(
        container(
            row![
                text("⠿")
                    .size(size::LABEL)
                    .color(theme::muted(&app.theme())),
                text(summary).size(size::CAPTION).width(Length::Fill),
                status_glyph,
            ]
            .spacing(space::S2)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([space::S1 + 2.0, space::S3]))
        .style(if rule_selected {
            theme::card_selected_style
        } else {
            theme::card_style
        })
        .width(Length::Fill),
    )
    .on_press(Message::SelectRule(rule.id))
    .padding(0)
    .style(theme::naked)
    .width(Length::Fill)
    .into()
}

fn add_rule_row(app: &App, rs_id: apimokka_model::RuleSetId) -> Element<'_, Message> {
    button(row![
        Space::new().width(Length::Fixed(space::S5)),
        text(format!("+ {}", app.t(Key::BtnAddRule))).size(size::LABEL),
    ])
    .on_press(Message::AddRule(rs_id))
    .padding(Padding::from([space::S1, space::S3]))
    .style(iced::widget::button::text)
    .width(Length::Fill)
    .into()
}

fn add_rule_set_row(app: &App) -> Element<'_, Message> {
    button(text(format!("+ {}", app.t(Key::BtnAddRuleSet))).size(size::LABEL))
        .on_press(Message::AddRuleSet)
        .padding(Padding::from([space::S1, space::S3]))
        .style(iced::widget::button::text)
        .width(Length::Fill)
        .into()
}

fn fallback_header_row(app: &App, count: usize, open: bool) -> Element<'_, Message> {
    let chevron = if open { "▾" } else { "▸" };
    button(
        row![
            text(chevron)
                .size(size::LABEL)
                .color(theme::muted(&app.theme())),
            text(app.t(Key::RoutesFallbackFiles))
                .size(size::LABEL)
                .color(theme::muted(&app.theme()))
                .width(Length::Fill),
            // RFC MK-059 decision 4: counts are a genuine caption, not a
            // label — left at CAPTION deliberately, unlike the chevron and
            // header text either side of it.
            text(format!("({})", count))
                .size(size::CAPTION)
                .color(theme::muted(&app.theme())),
        ]
        .spacing(space::S2)
        .align_y(Alignment::Center),
    )
    .on_press(Message::ToggleFallbackSection)
    .padding(Padding::from([space::S1, space::S3]))
    .style(iced::widget::button::text)
    .width(Length::Fill)
    .into()
}

fn fallback_file_row<'a>(
    app: &'a App,
    f: &'a apimokka_model::node::FileNodeView,
) -> Element<'a, Message> {
    let sel = app.selection.file_route.as_deref() == Some(f.path.as_str());
    let hint = f.route_hint.as_deref().unwrap_or("");
    let fdirty = app.is_fallback_dirty(&f.path);
    let dirty_el: Element<Message> = if fdirty {
        widgets::dirty_dot()
    } else {
        Space::new().width(Length::Fixed(0.0)).into()
    };
    button(
        container(
            column![
                row![
                    text("{ }")
                        .size(size::LABEL)
                        .color(theme::muted(&app.theme())),
                    text(f.name.as_str()).size(size::BODY).width(Length::Fill),
                    dirty_el,
                ]
                .spacing(space::S2)
                .align_y(Alignment::Center),
                text(hint)
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
            ]
            .spacing(2.0),
        )
        .padding(Padding::from([space::S2, space::S3]))
        .style(if sel {
            theme::card_selected_style
        } else {
            theme::card_style
        })
        .width(Length::Fill),
    )
    .on_press(Message::SelectFileRoute(f.path.clone()))
    .padding(0)
    .style(theme::naked)
    .width(Length::Fill)
    .into()
}

fn add_fallback_file_row(app: &App) -> Element<'_, Message> {
    // No `.on_press`: file creation has no I/O in this mockup
    // (`DisabledNoFileIo` below explains why) — inert by design, and
    // `Region::activation_message` returns `None` for this row to match.
    column![
        button(text(format!("+ {}", app.t(Key::BtnAddFallbackFile))).size(size::LABEL))
            .padding(Padding::from([space::S1, space::S3]))
            .style(iced::widget::button::text)
            .width(Length::Fill),
        container(
            text(app.t(Key::DisabledNoFileIo))
                .size(size::CAPTION)
                .color(theme::muted(&app.theme()))
        )
        .padding(Padding::from([0.0, space::S3])),
    ]
    .spacing(2.0)
    .into()
}

fn middleware_header_row(app: &App, count: usize, open: bool) -> Element<'_, Message> {
    let chevron = if open { "▾" } else { "▸" };
    button(
        row![
            text(chevron)
                .size(size::LABEL)
                .color(theme::muted(&app.theme())),
            text(app.t(Key::RoutesMiddleware))
                .size(size::LABEL)
                .color(theme::muted(&app.theme()))
                .width(Length::Fill),
            text(format!("({})", count))
                .size(size::CAPTION)
                .color(theme::muted(&app.theme())),
        ]
        .spacing(space::S2)
        .align_y(Alignment::Center),
    )
    .on_press(Message::ToggleMiddlewareSection)
    .padding(Padding::from([space::S1, space::S3]))
    .style(iced::widget::button::text)
    .width(Length::Fill)
    .into()
}

fn middleware_script_row<'a>(
    app: &'a App,
    s: &'a apimokka_model::node::ConfigFileView,
) -> Element<'a, Message> {
    let name = s.path.rsplit('/').next().unwrap_or(&s.path);
    let sel = app.selection.script.as_deref() == Some(s.path.as_str());
    button(
        container(text(name).size(size::BODY))
            .padding(Padding::from([space::S2, space::S3]))
            .style(if sel {
                theme::card_selected_style
            } else {
                theme::card_style
            })
            .width(Length::Fill),
    )
    .on_press(Message::SelectScript(s.path.clone()))
    .padding(0)
    .style(theme::naked)
    .width(Length::Fill)
    .into()
}

fn add_script_row(app: &App) -> Element<'_, Message> {
    // No `.on_press` — same inert-by-design reasoning as `add_fallback_file_row`.
    column![
        button(text(format!("+ {}", app.t(Key::BtnAddScript))).size(size::LABEL))
            .padding(Padding::from([space::S1, space::S3]))
            .style(iced::widget::button::text)
            .width(Length::Fill),
        container(
            text(app.t(Key::DisabledNoFileIo))
                .size(size::CAPTION)
                .color(theme::muted(&app.theme()))
        )
        .padding(Padding::from([0.0, space::S3])),
    ]
    .spacing(2.0)
    .into()
}
