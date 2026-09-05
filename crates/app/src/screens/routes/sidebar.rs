//! Left sidebar — rule-set accordion, fallback files, middleware scripts.

use crate::app::App;
use crate::message::Message;
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use apimokka_model::snapshot::RuleSetView;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Color, Element, Length, Padding};

pub(super) fn left_sidebar(app: &App) -> Element<'_, Message> {
    let t = |k| app.t(k);
    let snap = match &app.snapshot {
        Some(s) => s,
        None => {
            return container(widgets::empty_state(t(Key::EmptyNoWorkspaceOpen)))
                .width(Length::Fixed(280.0))
                .height(Length::Fill)
                .style(theme::panel_style)
                .into();
        }
    };

    let mut col = column![]
        .spacing(space::S1)
        .padding(Padding::from([space::S3, space::S2]));

    // ── Rule sets (accordion: only one open at a time) ────────────────────
    col = col.push(
        text(t(Key::RoutesRuleSets))
            .size(size::LABEL)
            .color(theme::muted(&app.theme())),
    );
    for rs in &snap.rule_sets {
        let is_open = app.rule_set_open == Some(rs.id);
        col = col.push(rule_set_group(app, rs, is_open));
    }
    col = col.push(
        button(text(format!("+ {}", t(Key::BtnAddRuleSet))).size(size::LABEL))
            .on_press(Message::AddRuleSet)
            .padding(Padding::from([space::S1, space::S3]))
            .style(iced::widget::button::text)
            .width(Length::Fill),
    );

    // ── Fallback files (collapsed by default) ─────────────────────────────
    col = col.push(widgets::divider());
    let fb_open = app.fallback_section_open;
    let fb_chevron = if fb_open { "▾" } else { "▸" };
    let fb_count = snap.fallback_files.len();
    col = col.push(
        button(
            row![
                text(fb_chevron)
                    .size(size::LABEL)
                    .color(theme::muted(&app.theme())),
                text(t(Key::RoutesFallbackFiles))
                    .size(size::LABEL)
                    .color(theme::muted(&app.theme()))
                    .width(Length::Fill),
                // RFC MK-059 decision 4: counts are a genuine caption, not a
                // label — left at CAPTION deliberately, unlike the chevron
                // and header text either side of it.
                text(format!("({})", fb_count))
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
            ]
            .spacing(space::S2)
            .align_y(Alignment::Center),
        )
        .on_press(Message::ToggleFallbackSection)
        .padding(Padding::from([space::S1, space::S3]))
        .style(iced::widget::button::text)
        .width(Length::Fill),
    );

    if fb_open {
        for f in &snap.fallback_files {
            let sel = app.selection.file_route.as_deref() == Some(f.path.as_str());
            let hint = f.route_hint.as_deref().unwrap_or("");
            let fdirty = app.is_fallback_dirty(&f.path);
            let dirty_el: Element<Message> = if fdirty {
                widgets::dirty_dot()
            } else {
                Space::new().width(Length::Fixed(0.0)).into()
            };
            col = col.push(
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
                .width(Length::Fill),
            );
        }
        col = col.push(
            column![
                button(text(format!("+ {}", t(Key::BtnAddFallbackFile))).size(size::LABEL))
                    .padding(Padding::from([space::S1, space::S3]))
                    .style(iced::widget::button::text)
                    .width(Length::Fill),
                container(
                    text(t(Key::DisabledNoFileIo))
                        .size(size::CAPTION)
                        .color(theme::muted(&app.theme()))
                )
                .padding(Padding::from([0.0, space::S3])),
            ]
            .spacing(2.0),
        );
    }

    // ── Middleware scripts (collapsed by default) ─────────────────────────
    col = col.push(widgets::divider());
    let mw_open = app.middleware_section_open;
    let mw_chevron = if mw_open { "▾" } else { "▸" };
    let mw_count = snap.middleware_scripts.len();
    col = col.push(
        button(
            row![
                text(mw_chevron)
                    .size(size::LABEL)
                    .color(theme::muted(&app.theme())),
                text(t(Key::RoutesMiddleware))
                    .size(size::LABEL)
                    .color(theme::muted(&app.theme()))
                    .width(Length::Fill),
                // RFC MK-059 decision 4: counts are a genuine caption, not a
                // label — left at CAPTION deliberately, unlike the chevron
                // and header text either side of it.
                text(format!("({})", mw_count))
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
            ]
            .spacing(space::S2)
            .align_y(Alignment::Center),
        )
        .on_press(Message::ToggleMiddlewareSection)
        .padding(Padding::from([space::S1, space::S3]))
        .style(iced::widget::button::text)
        .width(Length::Fill),
    );

    if mw_open {
        for s in &snap.middleware_scripts {
            let name = s.path.rsplit('/').next().unwrap_or(&s.path);
            let path_str = s.path.clone();
            let sel = app.selection.script.as_deref() == Some(&path_str);
            col = col.push(
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
                .on_press(Message::SelectScript(path_str))
                .padding(0)
                .style(theme::naked)
                .width(Length::Fill),
            );
        }
        col = col.push(
            column![
                button(text(format!("+ {}", t(Key::BtnAddScript))).size(size::LABEL))
                    .padding(Padding::from([space::S1, space::S3]))
                    .style(iced::widget::button::text)
                    .width(Length::Fill),
                container(
                    text(t(Key::DisabledNoFileIo))
                        .size(size::CAPTION)
                        .color(theme::muted(&app.theme()))
                )
                .padding(Padding::from([0.0, space::S3])),
            ]
            .spacing(2.0),
        );
    }

    container(scrollable(col).height(Length::Fill))
        .width(Length::Fixed(280.0))
        .height(Length::Fill)
        .style(theme::panel_style)
        .into()
}

fn rule_set_group<'a>(app: &'a App, rs: &'a RuleSetView, is_open: bool) -> Element<'a, Message> {
    let rs_selected = app.selection.rule_set == Some(rs.id);
    let file_name: &str = rs.file.path.rsplit('/').next().unwrap_or(&rs.file.path);
    let rule_count = rs.rules.len();
    let chevron = if is_open { "▾" } else { "▸" };

    let dirty_el: Element<Message> = if rs.file.dirty {
        widgets::dirty_dot()
    } else {
        Space::new().width(0.0).into()
    };

    // Header: chevron + filename + rule count + dirty marker
    let rs_row = button(
        container(
            row![
                text(chevron)
                    .size(size::LABEL)
                    .color(theme::muted(&app.theme())),
                text(file_name).size(size::BODY).width(Length::Fill),
                // RFC MK-059 decision 4: a count is a genuine caption.
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
    .width(Length::Fill);

    if !is_open {
        // Collapsed: only show the header row
        return column![rs_row].spacing(0).into();
    }

    // Expanded: show rules
    let rule_rows: Vec<Element<Message>> = rs
        .rules
        .iter()
        .map(|rule| {
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
        })
        .collect();

    let add_rule_row = button(row![
        Space::new().width(Length::Fixed(space::S5)),
        text(format!("+ {}", app.t(Key::BtnAddRule))).size(size::LABEL),
    ])
    .on_press(Message::AddRule(rs.id))
    .padding(Padding::from([space::S1, space::S3]))
    .style(iced::widget::button::text)
    .width(Length::Fill);

    let mut col = column![rs_row, Space::new().height(space::S1)].spacing(space::S1);
    for r in rule_rows {
        col = col.push(r);
    }
    col = col.push(add_rule_row);
    col.into()
}
