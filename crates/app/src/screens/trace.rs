//! MK-029 / MK-042 — Trace screen (S-11) + Match detail panel (S-12).
//!
//! MK-042 completes the screen:
//! - Live filter on url_path, method, and outcome label.
//! - Outcome-aware detail panel (Matched / Fallback / Miss / Error).
//! - Dropped-count warning.

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length, Padding};
use apimokka_i18n::Key;
use apimokka_model::{MatchTraceEvent, TraceOutcome};

use crate::app::App;
use crate::message::Message;
use crate::theme::{self, size, space};
use crate::widgets;

pub fn view(app: &App) -> Element<'_, Message> {
    let events = filtered_events(app);
    let event_list = event_stream(app, &events);
    let detail     = match_detail(app);
    row![event_list, detail].height(Length::Fill).into()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Events that pass the live filter (case-insensitive substring on path /
/// method / outcome label). Empty filter = all events.
fn filtered_events<'a>(app: &'a App) -> Vec<&'a MatchTraceEvent> {
    let q = app.trace_filter.to_lowercase();
    app.trace.iter()
        .filter(|ev| {
            if q.is_empty() { return true; }
            ev.request.url_path.to_lowercase().contains(&q)
                || ev.request.method.to_lowercase().contains(&q)
                || ev.outcome.label().contains(q.as_str())
        })
        .collect()
}

// ── Event stream ──────────────────────────────────────────────────────────────

fn event_stream<'a>(app: &'a App, events: &[&'a MatchTraceEvent]) -> Element<'a, Message> {
    let filter_bar = container(
        row![
            text_input(app.t(Key::TraceFilterPath), &app.trace_filter)
                .on_input(Message::TraceFilterChanged)
                .size(size::BODY)
                .padding(Padding::from([space::S2, space::S3]))
                .width(Length::Fill),
            Space::new().width(space::S2),
            button(text(if app.trace_paused {
                    app.t(Key::TraceResume)
                } else {
                    app.t(Key::TracePause)
                }).size(size::BODY))
                .on_press(Message::TracePauseToggle)
                .padding(Padding::from([space::S2, space::S3])),
            button(text(app.t(Key::TraceClear)).size(size::BODY))
                .on_press(Message::TraceClear)
                .padding(Padding::from([space::S2, space::S3])),
        ]
        .spacing(space::S2)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([space::S3, space::S4]));

    let rows: Vec<Element<Message>> = if events.is_empty() {
        vec![widgets::empty_state(app.t(Key::TraceEmptyMessage))]
    } else {
        events.iter().rev().map(|ev| {
            let sel  = app.selected_trace == Some(ev.event_id);
            let eid  = ev.event_id;
            let style: fn(&iced::Theme) -> iced::widget::container::Style =
                if sel { theme::card_selected_style } else { theme::card_style };

            button(
                container(
                    row![
                        text(ev.outcome.glyph()).size(size::BODY),
                        column![
                            row![
                                text(ev.request.method.as_str()).size(size::BODY),
                                text(ev.request.url_path.as_str()).size(size::BODY)
                                    .width(Length::Fill),
                            ]
                            .spacing(space::S2),
                            row![
                                text(ev.time.as_str()).size(size::CAPTION)
                                    .color(theme::muted(&app.theme())),
                                Space::new().width(Length::Fill),
                                text(format!("{}ms", ev.duration_ms)).size(size::CAPTION)
                                    .color(theme::muted(&app.theme())),
                            ],
                        ]
                        .spacing(space::S1)
                        .width(Length::Fill),
                    ]
                    .spacing(space::S3)
                    .align_y(Alignment::Center),
                )
                .padding(Padding::from([space::S3, space::S4]))
                .style(style)
                .width(Length::Fill),
            )
            .on_press(Message::SelectTraceEvent(eid))
            .padding(0)
            .style(theme::naked)
            .width(Length::Fill)
            .into()
        })
        .collect()
    };

    column![
        container(
            text(app.t(Key::TraceTitle)).size(size::TITLE),
        )
        .padding(Padding::from([space::S4, space::S4])),
        filter_bar,
        widgets::divider(),
        scrollable(
            column(rows)
                .spacing(space::S2)
                .padding(Padding::from([space::S2, space::S4])),
        )
        .height(Length::Fill),
    ]
    .width(Length::Fill)
    .into()
}

// ── Match detail panel ────────────────────────────────────────────────────────

fn match_detail(app: &App) -> Element<'_, Message> {
    let empty = || {
        container(widgets::empty_state(app.t(Key::DetailTitle)))
            .width(Length::Fixed(380.0))
            .height(Length::Fill)
            .style(theme::panel_style)
            .into()
    };

    let Some(eid) = app.selected_trace else { return empty(); };
    let Some(ev)  = app.trace.iter().find(|e| e.event_id == eid) else { return empty(); };

    let mut col = column![
        text(app.t(Key::DetailTitle)).size(size::SECTION),
        widgets::divider(),
        // ── Request summary ──────────────────────────────────────────────
        text(app.t(Key::DetailRequest)).size(size::BODY_STRONG),
        text(format!("{} {}", ev.request.method, ev.request.url_path)).size(size::BODY),
    ]
    .spacing(space::S2)
    .padding(Padding::from([space::S4, space::S4]));

    for (k, v) in &ev.request.headers {
        col = col.push(
            text(format!("{k}: {v}")).size(size::CAPTION)
                .color(theme::muted(&app.theme())),
        );
    }
    if let Some(body) = &ev.request.body_preview {
        col = col.push(
            text(body.as_str()).size(size::CAPTION)
                .font(iced::Font::MONOSPACE),
        );
    }

    // ── Outcome — variant-specific content ──────────────────────────────
    col = col.push(widgets::divider());
    col = col.push(text(app.t(Key::DetailOutcome)).size(size::BODY_STRONG));

    match &ev.outcome {
        TraceOutcome::Matched { rule_set_index, rule_index } => {
            col = outcome_matched(app, col, *rule_set_index, *rule_index);
        }
        TraceOutcome::Fallback { file_path, status } => {
            col = outcome_fallback(app, col, file_path, status);
        }
        TraceOutcome::Miss { status } => {
            col = outcome_miss(app, col, status, &ev.request.url_path);
        }
        TraceOutcome::Error { kind, message } => {
            col = outcome_error(app, col, kind, message);
        }
    }

    // ── Dropped-count warning ────────────────────────────────────────────
    if ev.dropped_count > 0 {
        col = col.push(widgets::divider());
        col = col.push(
            container(
                text(format!("{} {}", ev.dropped_count, app.t(Key::DetailDroppedWarning)))
                    .size(size::CAPTION),
            )
            .padding(Padding::from([space::S2, space::S3]))
            .style(theme::banner_style)
            .width(Length::Fill),
        );
    }

    // ── Actions ──────────────────────────────────────────────────────────
    col = col.push(widgets::divider());
    col = col.push(
        widgets::secondary_btn(app.t(Key::BtnReplayAsTestInput), Message::ReplayAsTestInput(eid)),
    );

    container(scrollable(col).height(Length::Fill))
        .width(Length::Fixed(380.0))
        .height(Length::Fill)
        .style(theme::panel_style)
        .into()
}

// ── Outcome variant renderers ─────────────────────────────────────────────────

fn outcome_matched<'a>(
    app: &'a App,
    mut col: iced::widget::Column<'a, Message>,
    rs_idx: usize,
    rule_idx: usize,
) -> iced::widget::Column<'a, Message> {
    col = col.push(
        text(format!("{} ✓", app.t(Key::TraceMatchedLabel)))
            .size(size::BODY),
    );

    if let Some(snap) = &app.snapshot {
        if let Some(rs) = snap.rule_sets.get(rs_idx) {
            let rs_name = rs.file.path.rsplit('/').next().unwrap_or(&rs.file.path);
            col = col.push(
                widgets::field_row(app.t(Key::DetailMatchedRuleSet), rs_name),
            );

            if let Some(rule) = rs.rules.get(rule_idx) {
                let rule_summary = rule.summary();
                col = col.push(
                    row![
                        text(app.t(Key::DetailMatchedRule)).size(size::CAPTION)
                            .color(theme::muted(&app.theme()))
                            .width(Length::Fixed(100.0)),
                        text(rule_summary).size(size::CAPTION).width(Length::Fill),
                    ]
                    .spacing(space::S2)
                    .align_y(Alignment::Center),
                );
                let rule_id = rule.id;
                col = col.push(
                    button(text(app.t(Key::DetailJumpToRule)).size(size::BODY))
                        .on_press(Message::JumpToRule(rule_id))
                        .padding(Padding::from([space::S2, space::S3])),
                );
            }
        }
    }
    col
}

fn outcome_fallback<'a>(
    app: &'a App,
    mut col: iced::widget::Column<'a, Message>,
    file_path: &'a str,
    status: &'a str,
) -> iced::widget::Column<'a, Message> {
    col = col.push(
        text(format!("{} ↩", app.t(Key::TraceFallbackLabel))).size(size::BODY),
    );
    col = col.push(widgets::field_row(app.t(Key::DetailFallbackFile), file_path));
    col = col.push(widgets::field_row(app.t(Key::DetailFallbackStatus), status));
    col = col.push(text(app.t(Key::DetailFallbackExplanation)).size(size::CAPTION)
        .color(theme::muted(&app.theme())));

    let path_owned = file_path.to_string();
    col = col.push(
        button(text(app.t(Key::DetailJumpToFile)).size(size::BODY))
            .on_press(Message::JumpToFile(path_owned))
            .padding(Padding::from([space::S2, space::S3])),
    );
    col
}

fn outcome_miss<'a>(
    app: &'a App,
    mut col: iced::widget::Column<'a, Message>,
    status: &'a str,
    url_path: &'a str,
) -> iced::widget::Column<'a, Message> {
    col = col.push(
        text(format!("{} ◯", app.t(Key::TraceMissLabel))).size(size::BODY),
    );
    col = col.push(widgets::field_row(app.t(Key::DetailMissStatus), status));
    col = col.push(
        text(app.t(Key::DetailMissExplanation)).size(size::CAPTION)
            .color(theme::muted(&app.theme())),
    );
    // CTA: create a rule pre-populated with this URL path.
    let path_for_rule = url_path.to_string();
    col = col.push(
        button(text(app.t(Key::DetailMissCreateCta)).size(size::BODY))
            .on_press(Message::AddRuleForPath(path_for_rule))
            .padding(Padding::from([space::S2, space::S3])),
    );
    col
}

fn outcome_error<'a>(
    app: &'a App,
    mut col: iced::widget::Column<'a, Message>,
    kind: &'a str,
    message: &'a str,
) -> iced::widget::Column<'a, Message> {
    col = col.push(
        text(format!("{} !", app.t(Key::TraceErrorLabel))).size(size::BODY),
    );
    col = col.push(widgets::field_row(app.t(Key::DetailErrorKind), kind));
    col = col.push(
        text(app.t(Key::DetailErrorMessage)).size(size::CAPTION)
            .color(theme::muted(&app.theme())),
    );
    col = col.push(
        text(message).size(size::CAPTION)
            .font(iced::Font::MONOSPACE)
            .color(theme::muted(&app.theme())),
    );
    col
}
