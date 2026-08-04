//! Recent trace activity strip shown at the foot of the rule editor.

use crate::app::App;
use crate::message::Message;
use crate::theme::{self, size, space};
use apimokka_i18n::Key;
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length, Padding};

/// Compact strip of recent trace events matching this rule.
pub(super) fn trace_activity_section<'a>(
    app: &'a App,
    rule: &'a apimokka_model::snapshot::RuleView,
) -> Element<'a, Message> {
    let recent = recent_matching_events(app, rule);

    let header = row![
        text(app.t(Key::RoutesRecentTraceActivity))
            .size(size::BODY)
            .color(theme::muted(&app.theme()))
            .width(Length::Fill),
        button(text(app.t(Key::RoutesViewAllInTrace)).size(size::CAPTION))
            .on_press(Message::ViewAllInTrace)
            .padding(Padding::from([space::S1, space::S2]))
            .style(iced::widget::button::text),
    ]
    .align_y(Alignment::Center);

    let body: Element<Message> = if recent.is_empty() {
        text(app.t(Key::RoutesNoRecentMatches))
            .size(size::CAPTION)
            .color(theme::muted(&app.theme()))
            .into()
    } else {
        let rows: Vec<Element<Message>> = recent
            .iter()
            .map(|ev| {
                let eid = ev.event_id;
                row![
                    text(ev.outcome.glyph()).size(size::BODY),
                    text(ev.request.method.as_str()).size(size::CAPTION),
                    text(ev.request.url_path.as_str())
                        .size(size::CAPTION)
                        .color(theme::muted(&app.theme()))
                        .width(Length::Fill),
                    text(format!(
                        "{}{}",
                        ev.duration_ms,
                        app.t(Key::RespondDelayUnit)
                    ))
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
                    text(ev.time.as_str())
                        .size(size::CAPTION)
                        .color(theme::muted(&app.theme())),
                    button(text(app.t(Key::RoutesJumpToTraceEvent)).size(size::CAPTION))
                        .on_press(Message::JumpToTraceEvent(eid))
                        .padding(Padding::from([space::S1, space::S2]))
                        .style(iced::widget::button::text),
                ]
                .spacing(space::S3)
                .align_y(Alignment::Center)
                .into()
            })
            .collect();
        column(rows).spacing(space::S1).into()
    };

    container(column![header, body].spacing(space::S2))
        .padding(Padding::from([space::S3, space::S5]))
        .width(Length::Fill)
        .into()
}

fn recent_matching_events<'a>(
    app: &'a App,
    rule: &apimokka_model::snapshot::RuleView,
) -> Vec<&'a apimokka_model::MatchTraceEvent> {
    // MK-042: primary strategy — match by the rule_set_index / rule_index that
    // the engine reports. We find this rule's position in the snapshot so we
    // can compare against the trace outcome directly.
    let rule_position: Option<(usize, usize)> = app.snapshot.as_ref().and_then(|snap| {
        snap.rule_sets.iter().enumerate().find_map(|(rs_idx, rs)| {
            rs.rules
                .iter()
                .position(|r| r.id == rule.id)
                .map(|r_idx| (rs_idx, r_idx))
        })
    });

    let url_path = &rule.payload.url_path;

    app.trace
        .iter()
        .rev()
        .filter(|ev| {
            match &ev.outcome {
                apimokka_model::TraceOutcome::Matched {
                    rule_set_index,
                    rule_index,
                } => {
                    // Primary: exact index match (engine-reported).
                    if let Some((rs, r)) = rule_position {
                        return *rule_set_index == rs && *rule_index == r;
                    }
                    // Fallback: index unavailable, use URL path heuristic.
                    if url_path.is_empty() {
                        return true;
                    }
                    ev.request.url_path == *url_path
                        || ev.request.url_path.starts_with(url_path.as_str())
                }
                // Non-Matched outcomes never belong to a specific rule.
                _ => false,
            }
        })
        .take(3)
        .collect()
}
