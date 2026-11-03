//! MK-029 / MK-042 — Trace screen (S-11) + Match detail panel (S-12).
//!
//! MK-042 completes the screen:
//! - Live filter on url_path, method, and outcome label.
//! - Outcome-aware detail panel (Matched / Fallback / Miss / Error).
//! - Dropped-count warning.

use apimokka_i18n::Key;
use apimokka_model::{MatchTraceEvent, TraceOutcome};
use iced::widget::{
    Space, button, column, container, mouse_area, row, scrollable, text, text_input,
};
use iced::{Alignment, Element, Length, Padding};

use crate::app::App;
use crate::message::Message;
use crate::regions::{Cursor, Region, filtered_trace_events};
use crate::theme::{self, size, space};
use crate::widgets;

/// Task 030 §4/§5: the `Id` `App::update_and_dispatch` focuses when
/// `TraceFilter` becomes the focused region, and unfocuses when it stops
/// being one (via `Region::focus_id`).
pub const FILTER_INPUT_ID: &str = "mk023-trace-filter";

/// Task 032 (D-20): the `Id`s `App::update_and_dispatch` scrolls when the
/// event list's or the match-detail panel's keyboard cursor moves — same
/// D-5 approximation the sidebar/Dashboard/palette scrollables already use.
pub const EVENTS_SCROLLABLE_ID: &str = "mk023-trace-events";
pub const DETAIL_SCROLLABLE_ID: &str = "mk023-trace-detail";

pub fn view(app: &App) -> Element<'_, Message> {
    let events = filtered_trace_events(app);
    let event_list = event_stream(app, &events);
    let detail = match_detail(app);
    row![event_list, detail].height(Length::Fill).into()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// `cursor_style`'s border is drawn at the wrapping container's own bounds;
/// without a gap an opaque sibling (a button's own background, a selected
/// card's fill) paints over it — the fix task 028 §6 found, reused here.
fn cursor_wrap<'a>(has_cursor: bool, el: Element<'a, Message>) -> Element<'a, Message> {
    container(el)
        .padding(space::S1)
        .style(if has_cursor {
            theme::cursor_style
        } else {
            |_: &iced::Theme| iced::widget::container::Style::default()
        })
        .into()
}

// ── Event stream ──────────────────────────────────────────────────────────────

fn event_stream<'a>(app: &'a App, events: &[&'a MatchTraceEvent]) -> Element<'a, Message> {
    let filter_bar = container(
        row![
            // D-18 (task 032): `on_release`, same reasoning as
            // `dashboard.rs`'s search field.
            mouse_area(
                text_input(app.t(Key::TraceFilterPath), &app.trace_filter)
                    .id(FILTER_INPUT_ID)
                    .on_input(Message::TraceFilterChanged)
                    .style(theme::text_input_style)
                    .size(size::BODY)
                    .padding(Padding::from([space::S2, space::S3]))
                    .width(Length::Fill),
            )
            .on_release(Message::RegionClicked(Region::TraceFilter)),
            Space::new().width(space::S2),
            cursor_wrap(
                app.focused_region == Some(Region::TracePause),
                button(
                    text(if app.trace_paused {
                        app.t(Key::TraceResume)
                    } else {
                        app.t(Key::TracePause)
                    })
                    .size(size::BODY)
                )
                .on_press(Message::TracePauseToggle)
                .padding(Padding::from([space::S2, space::S3]))
                .into(),
            ),
            cursor_wrap(
                app.focused_region == Some(Region::TraceClear),
                button(text(app.t(Key::TraceClear)).size(size::BODY))
                    .on_press(Message::TraceClear)
                    .padding(Padding::from([space::S2, space::S3]))
                    .into(),
            ),
        ]
        .spacing(space::S2)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([space::S3, space::S4]));

    let rows: Vec<Element<Message>> = if events.is_empty() {
        vec![widgets::empty_state(app.t(Key::TraceEmptyMessage))]
    } else {
        events
            .iter()
            .rev()
            .map(|ev| {
                let sel = app.selected_trace == Some(ev.event_id);
                let eid = ev.event_id;
                let has_cursor = app.focused_region == Some(Region::TraceEvents)
                    && app.region_cursor == Some(Cursor::TraceEvent(eid));
                let style: fn(&iced::Theme) -> iced::widget::container::Style = if sel {
                    theme::card_selected_style
                } else {
                    theme::card_style
                };

                let row_el = button(
                    container(
                        row![
                            text(ev.outcome.glyph()).size(size::BODY),
                            column![
                                row![
                                    text(ev.request.method.as_str()).size(size::BODY),
                                    text(ev.request.url_path.as_str())
                                        .size(size::BODY)
                                        .width(Length::Fill),
                                ]
                                .spacing(space::S2),
                                row![
                                    text(ev.time.as_str())
                                        .size(size::CAPTION)
                                        .color(theme::muted(&app.theme())),
                                    Space::new().width(Length::Fill),
                                    text(format!("{}ms", ev.duration_ms))
                                        .size(size::CAPTION)
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
                .width(Length::Fill);

                cursor_wrap(has_cursor, row_el.into())
            })
            .collect()
    };

    column![
        container(text(app.t(Key::TraceTitle)).size(size::TITLE),)
            .padding(Padding::from([space::S4, space::S4])),
        filter_bar,
        widgets::divider(),
        scrollable(
            column(rows)
                .spacing(space::S2)
                .padding(Padding::from([space::S2, space::S4])),
        )
        .id(EVENTS_SCROLLABLE_ID)
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

    let Some(eid) = app.selected_trace else {
        return empty();
    };
    let Some(ev) = app.trace.iter().find(|e| e.event_id == eid) else {
        return empty();
    };

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
            text(format!("{k}: {v}"))
                .size(size::CAPTION)
                .color(theme::muted(&app.theme())),
        );
    }
    if let Some(body) = &ev.request.body_preview {
        col = col.push(
            text(body.as_str())
                .size(size::CAPTION)
                .font(iced::Font::MONOSPACE),
        );
    }

    // ── Outcome — variant-specific content ──────────────────────────────
    col = col.push(widgets::divider());
    col = col.push(text(app.t(Key::DetailOutcome)).size(size::BODY_STRONG));

    match &ev.outcome {
        TraceOutcome::Matched {
            rule_set_index,
            rule_index,
        } => {
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
                text(format!(
                    "{} {}",
                    ev.dropped_count,
                    app.t(Key::DetailDroppedWarning)
                ))
                .size(size::BODY_SMALL)
                .line_height(theme::line_height::body_small()),
            )
            .padding(Padding::from([space::S2, space::S3]))
            .style(theme::banner_style)
            .width(Length::Fill),
        );
    }

    // ── Actions ──────────────────────────────────────────────────────────
    col = col.push(widgets::divider());
    col = col.push(cursor_wrap(
        app.focused_region == Some(Region::TraceReplay),
        widgets::secondary_btn(
            app.t(Key::BtnReplayAsTestInput),
            Message::ReplayAsTestInput(eid),
        ),
    ));

    container(
        scrollable(col)
            .id(DETAIL_SCROLLABLE_ID)
            .height(Length::Fill),
    )
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
    col = col.push(text(format!("{} ✓", app.t(Key::TraceMatchedLabel))).size(size::BODY));

    if let Some(snap) = &app.snapshot
        && let Some(rs) = snap.rule_sets.get(rs_idx)
    {
        let rs_name = rs.file.path.rsplit('/').next().unwrap_or(&rs.file.path);
        col = col.push(widgets::field_row(
            app.t(Key::DetailMatchedRuleSet),
            rs_name,
        ));

        if let Some(rule) = rs.rules.get(rule_idx) {
            let rule_summary = rule.summary();
            col = col.push(
                row![
                    text(app.t(Key::DetailMatchedRule))
                        .size(size::CAPTION)
                        .color(theme::muted(&app.theme()))
                        .width(Length::Fixed(100.0)),
                    text(rule_summary).size(size::CAPTION).width(Length::Fill),
                ]
                .spacing(space::S2)
                .align_y(Alignment::Center),
            );
            let rule_id = rule.id;
            col = col.push(cursor_wrap(
                app.focused_region == Some(Region::TraceOutcomeAction),
                button(text(app.t(Key::DetailJumpToRule)).size(size::BODY))
                    .on_press(Message::JumpToRule(rule_id))
                    .padding(Padding::from([space::S2, space::S3]))
                    .into(),
            ));
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
    col = col.push(text(format!("{} ↩", app.t(Key::TraceFallbackLabel))).size(size::BODY));
    col = col.push(widgets::field_row(
        app.t(Key::DetailFallbackFile),
        file_path,
    ));
    col = col.push(widgets::field_row(app.t(Key::DetailFallbackStatus), status));
    col = col.push(
        text(app.t(Key::DetailFallbackExplanation))
            .size(size::BODY_SMALL)
            .line_height(theme::line_height::body_small())
            .color(theme::muted(&app.theme())),
    );

    let path_owned = file_path.to_string();
    col = col.push(cursor_wrap(
        app.focused_region == Some(Region::TraceOutcomeAction),
        button(text(app.t(Key::DetailJumpToFile)).size(size::BODY))
            .on_press(Message::JumpToFile(path_owned))
            .padding(Padding::from([space::S2, space::S3]))
            .into(),
    ));
    col
}

fn outcome_miss<'a>(
    app: &'a App,
    mut col: iced::widget::Column<'a, Message>,
    status: &'a str,
    url_path: &'a str,
) -> iced::widget::Column<'a, Message> {
    col = col.push(text(format!("{} ◯", app.t(Key::TraceMissLabel))).size(size::BODY));
    col = col.push(widgets::field_row(app.t(Key::DetailMissStatus), status));
    col = col.push(
        text(app.t(Key::DetailMissExplanation))
            .size(size::BODY_SMALL)
            .line_height(theme::line_height::body_small())
            .color(theme::muted(&app.theme())),
    );
    // CTA: create a rule pre-populated with this URL path.
    let path_for_rule = url_path.to_string();
    col = col.push(cursor_wrap(
        app.focused_region == Some(Region::TraceOutcomeAction),
        button(text(app.t(Key::DetailMissCreateCta)).size(size::BODY))
            .on_press(Message::AddRuleForPath(path_for_rule))
            .padding(Padding::from([space::S2, space::S3]))
            .into(),
    ));
    col
}

fn outcome_error<'a>(
    app: &'a App,
    mut col: iced::widget::Column<'a, Message>,
    kind: &'a str,
    message: &'a str,
) -> iced::widget::Column<'a, Message> {
    col = col.push(text(format!("{} !", app.t(Key::TraceErrorLabel))).size(size::BODY));
    col = col.push(widgets::field_row(app.t(Key::DetailErrorKind), kind));
    col = col.push(
        text(app.t(Key::DetailErrorMessage))
            .size(size::LABEL)
            .color(theme::muted(&app.theme())),
    );
    col = col.push(
        text(message)
            .size(size::CAPTION)
            .font(iced::Font::MONOSPACE)
            .color(theme::muted(&app.theme())),
    );
    col
}
