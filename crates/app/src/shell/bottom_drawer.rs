//! MK-032 / MK-044 — Bottom drawer: validation panel and save-diff panel.
//!
//! MK-044 makes both panels actionable:
//! - Validation: groups by rule set, click-to-navigate, proper empty state.
//! - Save diff: shows rule summaries and fallback-file change indicators.

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Color, Element, Length, Padding};
use apimokka_i18n::Key;
use crate::app::App;
use crate::message::Message;
use crate::selection::DrawerMode;
use crate::theme::{self, size, space};
use crate::widgets;

pub fn view(app: &App) -> Element<'_, Message> {
    let mode = match app.drawer { Some(m) => m, None => return Space::new().into() };
    let body: Element<Message> = match mode {
        DrawerMode::Validation => validation_content(app),
        DrawerMode::SaveDiff   => save_diff_content(app),
    };
    container(
        column![
            drawer_header(app, mode),
            widgets::divider(),
            scrollable(body).height(Length::Fill),
        ]
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::panel_style)
    .into()
}

fn drawer_header<'a>(app: &'a App, mode: DrawerMode) -> Element<'a, Message> {
    let title = match mode {
        DrawerMode::Validation => app.t(Key::DrawerValidationTitle),
        DrawerMode::SaveDiff   => app.t(Key::DrawerSaveDiffTitle),
    };
    container(row![
        text(title).size(size::SECTION).width(Length::Fill),
        button(text("✕").size(size::BODY)).on_press(Message::CloseDrawer)
            .padding(Padding::from([space::S1, space::S2]))
            .style(iced::widget::button::text),
    ].align_y(Alignment::Center))
    .padding(Padding::from([space::S3, space::S4]))
    .into()
}

// ── Validation panel ──────────────────────────────────────────────────────────

fn validation_content(app: &App) -> Element<'_, Message> {
    let Some(snap) = &app.snapshot else {
        return widgets::empty_state(app.t(Key::DrawerValidationOk));
    };

    let mut col = column![].spacing(space::S2)
        .padding(Padding::from([space::S3, space::S4]));

    // ── Workspace-level diagnostics (shown first, at workspace scope) ────
    if !snap.diagnostics.is_empty() {
        col = col.push(
            text(app.t(Key::DrawerValidationWorkspace))
                .size(size::BODY_STRONG),
        );
        for d in &snap.diagnostics {
            col = col.push(
                row![
                    text(widgets::severity_glyph(d.severity)).size(size::BODY),
                    text(d.message.as_str()).size(size::CAPTION)
                        .color(theme::muted(&app.theme()))
                        .width(Length::Fill),
                ]
                .spacing(space::S2)
                .align_y(Alignment::Center),
            );
        }
        col = col.push(widgets::divider());
    }

    // ── Per-rule-set groups ──────────────────────────────────────────────
    let mut any_issue = false;
    for rs in &snap.rule_sets {
        let file_name = rs.file.path.rsplit('/').next().unwrap_or(&rs.file.path);

        let rule_issues: Vec<_> = rs.rules.iter()
            .flat_map(|r| r.validation.issues.iter().map(move |iss| (r, iss)))
            .collect();

        if rule_issues.is_empty() {
            // Clean file — positive confirmation in muted text
            col = col.push(
                row![
                    text("✓").size(size::CAPTION)
                        .color(Color::from_rgb(0.1, 0.65, 0.1)),
                    text(app.t(Key::DrawerValidationFileOk)).size(size::CAPTION)
                        .color(theme::muted(&app.theme())),
                    text(file_name).size(size::CAPTION)
                        .color(theme::muted(&app.theme())),
                ]
                .spacing(space::S2)
                .align_y(Alignment::Center),
            );
        } else {
            any_issue = true;
            // File heading
            col = col.push(
                text(file_name).size(size::BODY_STRONG),
            );
            for (rule, iss) in &rule_issues {
                let rule_id  = rule.id;
                let summary  = rule.summary();
                let glyph    = widgets::severity_glyph(iss.severity);
                let glyph_color = match iss.severity {
                    apimokka_model::Severity::Error   => Color::from_rgb(0.85, 0.15, 0.15),
                    apimokka_model::Severity::Warning => Color::from_rgb(0.85, 0.45, 0.0),
                    apimokka_model::Severity::Info    => Color::from_rgb(0.2, 0.5, 0.9),
                };
                col = col.push(
                    container(
                        column![
                            // Rule summary (clickable to navigate)
                            button(
                                row![
                                    text(glyph).size(size::CAPTION).color(glyph_color),
                                    text(summary).size(size::BODY).width(Length::Fill),
                                    text(app.t(Key::DrawerJumpToRule)).size(size::CAPTION)
                                        .color(theme::muted(&app.theme())),
                                    text("→").size(size::CAPTION)
                                        .color(theme::muted(&app.theme())),
                                ]
                                .spacing(space::S2)
                                .align_y(Alignment::Center),
                            )
                            .on_press(Message::JumpToRule(rule_id))
                            .padding(0)
                            .style(iced::widget::button::text)
                            .width(Length::Fill),
                            // Issue message, indented
                            row![
                                Space::new().width(Length::Fixed(space::S5)),
                                text(iss.message.as_str()).size(size::CAPTION)
                                    .color(theme::muted(&app.theme()))
                                    .width(Length::Fill),
                            ],
                        ]
                        .spacing(space::S1),
                    )
                    .padding(Padding::from([space::S2, space::S3]))
                    .style(theme::card_style)
                    .width(Length::Fill),
                );
            }
        }
    }

    // ── All-clear state ──────────────────────────────────────────────────
    if !any_issue && snap.diagnostics.is_empty() {
        return container(
            column![
                text("✓").size(size::TITLE).color(Color::from_rgb(0.1, 0.65, 0.1)),
                text(app.t(Key::DrawerValidationOk)).size(size::BODY),
            ]
            .spacing(space::S2)
            .align_x(iced::Alignment::Center),
        )
        .padding(Padding::from([space::S6, space::S5]))
        .width(Length::Fill)
        .into();
    }

    col.into()
}

// ── Save-diff panel ───────────────────────────────────────────────────────────

fn save_diff_content(app: &App) -> Element<'_, Message> {
    let Some(snap) = &app.snapshot else {
        return widgets::empty_state("No workspace open.");
    };

    let dirty_rule_sets: Vec<_> = snap.rule_sets.iter()
        .filter(|rs| rs.file.dirty)
        .collect();

    let dirty_fallback_paths: Vec<&str> = {
        let mut v: Vec<&str> = app.fallback_drafts.keys()
            .filter(|p| app.is_fallback_dirty(p))
            .map(|p| p.as_str())
            .collect();
        v.sort();
        v
    };

    let total = dirty_rule_sets.len() + dirty_fallback_paths.len();
    if total == 0 {
        return container(
            column![
                text("✓").size(size::TITLE).color(Color::from_rgb(0.1, 0.65, 0.1)),
                text("No unsaved changes.").size(size::BODY),
            ]
            .spacing(space::S2)
            .align_x(iced::Alignment::Center),
        )
        .padding(Padding::from([space::S6, space::S5]))
        .width(Length::Fill)
        .into();
    }

    let count_text = format!("{} {} unsaved changes", total,
        if total == 1 { "file with" } else { "files with" });

    let mut col = column![
        text(count_text).size(size::BODY_STRONG),
        Space::new().height(space::S2),
    ]
    .spacing(space::S2)
    .padding(Padding::from([space::S3, space::S4]));

    // ── Dirty rule-set files ─────────────────────────────────────────────
    for rs in &dirty_rule_sets {
        let file_name = rs.file.path.rsplit('/').next().unwrap_or(&rs.file.path);
        let rule_count = rs.rules.len();

        // Build rule summary list (comma-joined, truncated)
        let summaries: Vec<String> = rs.rules.iter().map(|r| r.summary()).collect();
        let preview = if summaries.len() <= 3 {
            summaries.join(", ")
        } else {
            format!("{}, … +{}", summaries[..3].join(", "), summaries.len() - 3)
        };

        col = col.push(
            container(
                column![
                    row![
                        text("●").size(size::CAPTION)
                            .color(theme::muted(&app.theme())),
                        text(file_name).size(size::BODY).width(Length::Fill),
                    ]
                    .spacing(space::S2)
                    .align_y(Alignment::Center),
                    row![
                        Space::new().width(Length::Fixed(space::S4)),
                        text(format!("{} {} {}", rule_count, app.t(Key::DrawerSaveDiffChangedRules), preview))
                            .size(size::CAPTION)
                            .color(theme::muted(&app.theme()))
                            .width(Length::Fill),
                    ],
                ]
                .spacing(space::S1),
            )
            .padding(Padding::from([space::S2, space::S3]))
            .style(theme::card_style)
            .width(Length::Fill),
        );
    }

    // ── Dirty fallback files ─────────────────────────────────────────────
    for path in &dirty_fallback_paths {
        let name = path.rsplit('/').next().unwrap_or(path);
        col = col.push(
            container(
                column![
                    row![
                        text("●").size(size::CAPTION)
                            .color(theme::muted(&app.theme())),
                        text(name.to_string()).size(size::BODY).width(Length::Fill),
                    ]
                    .spacing(space::S2)
                    .align_y(Alignment::Center),
                    row![
                        Space::new().width(Length::Fixed(space::S4)),
                        text(app.t(Key::DrawerSaveDiffFallbackMod)).size(size::CAPTION)
                            .color(theme::muted(&app.theme())),
                    ],
                ]
                .spacing(space::S1),
            )
            .padding(Padding::from([space::S2, space::S3]))
            .style(theme::card_style)
            .width(Length::Fill),
        );
    }

    // ── Actions ──────────────────────────────────────────────────────────
    col = col.push(Space::new().height(space::S2));
    col = col.push(
        row![
            widgets::ghost_btn(app.t(Key::BtnDiscard), Message::DiscardChanges),
            Space::new().width(Length::Fill),
            widgets::primary_btn(app.t(Key::BtnSaveAll), Message::SaveAll),
        ]
        .align_y(Alignment::Center),
    );

    col.into()
}
