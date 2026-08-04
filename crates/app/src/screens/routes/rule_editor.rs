//! Rule editor — WHEN / RESPOND columns and their card builders.
//!
//! Boundary decision: single-responsibility — `rule_editor()` plus its nine
//! card-builder helpers, all exclusively used within this file (confirmed
//! crate-wide), are one screen's construction vocabulary. Deferred to this
//! step by slice 4's review.

use super::trace_activity::trace_activity_section;
use crate::app::App;
use crate::message::Message;
use crate::theme::{self, pad, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use apimokka_model::{BodyOp, HeaderOp, UrlPathOp, respond::RespondMode, settings::Strategy};
use iced::widget::{
    Space, button, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Alignment, Color, Element, Length, Padding};

pub(super) fn rule_editor<'a>(
    app: &'a App,
    rule: &'a apimokka_model::snapshot::RuleView,
    p: &'a apimokka_model::RulePayload,
) -> Element<'a, Message> {
    let t = |k| app.t(k);
    let rule_id = rule.id;
    let response_delay = app
        .snapshot
        .as_ref()
        .and_then(|session| session.rule_draft(rule_id))
        .map(|draft| draft.response_delay.as_str());
    let rule_prototype = app
        .snapshot
        .as_ref()
        .and_then(|session| session.prototype.rule_extras.get(&rule_id));

    // MK-043: active strategy drives conditional per-rule field visibility.
    let active_strategy = app
        .snapshot
        .as_ref()
        .map(|s| s.root_settings.strategy)
        .unwrap_or(Strategy::FirstMatch);

    // ── Validation issues strip ────────────────────────────────────────────
    // Shown above the action header when the rule has issues. Uses the
    // non-empty validation.issues from the mock data (e.g. missing weight).
    let validation_strip: Option<Element<Message>> = if !rule.validation.issues.is_empty() {
        let msgs: Vec<Element<Message>> = rule
            .validation
            .issues
            .iter()
            .map(|issue| {
                row![
                    text("⚠")
                        .size(size::CAPTION)
                        .color(Color::from_rgb(0.85, 0.45, 0.0)),
                    text(issue.message.as_str()).size(size::CAPTION),
                ]
                .spacing(space::S2)
                .into()
            })
            .collect();
        Some(
            container(
                column![
                    text(t(Key::RuleEditorValidationWarning))
                        .size(size::CAPTION)
                        .color(theme::muted(&app.theme())),
                    column(msgs).spacing(space::S1),
                ]
                .spacing(space::S1),
            )
            .padding(Padding::from([space::S2, space::S5]))
            .width(Length::Fill)
            .style(theme::banner_style)
            .into(),
        )
    } else {
        None
    };

    // ── Rule action header (ABOVE WHEN/RESPOND) ───────────────────────────
    // Test rule is the primary action. It is gated when the rule has no match
    // criteria at all — and the reason is shown rather than a dead button.
    let has_when = !p.url_path.is_empty()
        || !p.method.is_empty()
        || !p.headers.is_empty()
        || !p.body.is_empty();
    let test_ready = if has_when {
        Some(Message::TestRuleOpen)
    } else {
        None
    };

    let action_header = container(
        row![
            text(rule.summary()).size(size::BODY).width(Length::Fill),
            // Test rule — primary; disabled with a reason when WHEN is empty.
            widgets::action_with_reason(
                &app.theme(),
                t(Key::TestRuleTitle),
                test_ready,
                t(Key::DisabledNeedUrlPath),
            ),
            button(text(t(Key::BtnDuplicate)).size(size::CAPTION))
                .on_press(Message::DuplicateRule(rule_id))
                .padding(Padding::from(pad::BUTTON))
                .style(iced::widget::button::text),
            button(text("▲").size(size::CAPTION))
                .on_press(Message::MoveRuleUp(rule_id))
                .padding(Padding::from(pad::BUTTON))
                .style(iced::widget::button::text),
            button(text("▼").size(size::CAPTION))
                .on_press(Message::MoveRuleDown(rule_id))
                .padding(Padding::from(pad::BUTTON))
                .style(iced::widget::button::text),
            widgets::danger_btn(t(Key::BtnDelete), Message::DeleteRule(rule_id)),
        ]
        .spacing(space::S2)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([space::S2, space::S5]))
    .width(Length::Fill)
    .style(theme::panel_style);

    // ── WHEN column (FillPortion 3 — more fields than RESPOND) ───────────
    // MK-041: In Guided mode, headers and body conditions start collapsed
    // behind a "More matching criteria" toggle to reduce initial visual load.
    // Expert mode shows all four cards directly (no change from v0.9.0).
    let when_col = {
        let mut cards: Vec<Element<Message>> = vec![
            section_head(t(Key::WhenLabel)),
            url_path_card(app, p),
            method_card(app, &p.method),
        ];

        if app.shows_scaffolding() {
            // Guided: show headers + body only when expanded.
            let header_count = p.headers.len();
            let body_count = p.body.len();
            let active_hidden = header_count + body_count;

            if app.rule_when_more {
                // Expanded: show both advanced cards then a "Fewer" toggle.
                cards.push(headers_card(app, p));
                cards.push(body_card(app, p));
                cards.push(
                    button(
                        row![
                            text("▾")
                                .size(size::CAPTION)
                                .color(theme::muted(&app.theme())),
                            text(t(Key::LayoutFewerWhen))
                                .size(size::CAPTION)
                                .color(theme::muted(&app.theme())),
                        ]
                        .spacing(space::S2)
                        .align_y(Alignment::Center),
                    )
                    .on_press(Message::ToggleRuleWhenMore)
                    .padding(Padding::from([space::S2, space::S3]))
                    .style(iced::widget::button::text)
                    .into(),
                );
            } else {
                // Collapsed: show the "More" toggle + active-condition badge.
                let badge: Element<Message> = if active_hidden > 0 {
                    // Build a count string: "1 header · 2 body active"
                    let mut parts = Vec::new();
                    if header_count > 0 {
                        parts.push(format!("{} {}", header_count, t(Key::LayoutActiveHeader)));
                    }
                    if body_count > 0 {
                        parts.push(format!("{} {}", body_count, t(Key::LayoutActiveBody)));
                    }
                    let count_str = parts.join(" · ") + " active";
                    text(count_str)
                        .size(size::CAPTION)
                        .color(theme::muted(&app.theme()))
                        .into()
                } else {
                    Space::new().width(Length::Fixed(0.0)).into()
                };

                cards.push(
                    row![
                        button(
                            row![
                                text("▸")
                                    .size(size::CAPTION)
                                    .color(theme::muted(&app.theme())),
                                text(t(Key::LayoutMoreWhen))
                                    .size(size::CAPTION)
                                    .color(theme::muted(&app.theme())),
                            ]
                            .spacing(space::S2)
                            .align_y(Alignment::Center),
                        )
                        .on_press(Message::ToggleRuleWhenMore)
                        .padding(Padding::from([space::S2, space::S3]))
                        .style(iced::widget::button::text),
                        badge,
                    ]
                    .spacing(space::S3)
                    .align_y(Alignment::Center)
                    .into(),
                );
            }
        } else {
            // Expert: always show all four cards.
            cards.push(headers_card(app, p));
            cards.push(body_card(app, p));
        }

        cards.push(Space::new().height(space::S4).into());

        container(
            scrollable(
                column(cards)
                    .spacing(space::S3)
                    .padding(Padding::from([space::S4, space::S3])),
            )
            .height(Length::Fill),
        )
        .width(Length::FillPortion(3))
        .height(Length::Fill)
    };

    // ── RESPOND column (FillPortion 2 — fewer fields) ─────────────────────
    // MK-043: when strategy is WeightedRandom or Priority, a per-rule numeric
    // field appears below the respond card. In Guided mode it follows the
    // rule_when_more toggle (advanced field, hidden by default).
    let per_rule_field: Option<Element<Message>> = if active_strategy.needs_per_rule_field()
        && (!app.shows_scaffolding() || app.rule_when_more)
    {
        // Build each variant directly so the compiler has unambiguous types.
        let (label_key, hint_key, field_el): (Key, Key, Element<Message>) = match active_strategy {
            Strategy::WeightedRandom => {
                let current = rule_prototype
                    .and_then(|prototype| prototype.weight)
                    .map(|w| w.to_string())
                    .unwrap_or_default();
                let inp = text_input("", &current)
                    .on_input(Message::RuleWeightChanged)
                    .size(size::BODY)
                    .padding(Padding::from([space::S2, space::S3]))
                    .width(Length::Fixed(100.0));
                (Key::RuleWeightLabel, Key::RuleWeightHint, inp.into())
            }
            Strategy::Priority => {
                let current = rule_prototype
                    .and_then(|prototype| prototype.priority)
                    .map(|pr| pr.to_string())
                    .unwrap_or_default();
                let inp = text_input("", &current)
                    .on_input(Message::RulePriorityChanged)
                    .size(size::BODY)
                    .padding(Padding::from([space::S2, space::S3]))
                    .width(Length::Fixed(100.0));
                (Key::RulePriorityLabel, Key::RulePriorityHint, inp.into())
            }
            _ => unreachable!(),
        };
        Some(
            container(
                column![
                    widgets::label_with_hint(&app.theme(), t(label_key), t(hint_key)),
                    field_el,
                ]
                .spacing(space::S2),
            )
            .padding(Padding::from(pad::CARD))
            .style(theme::card_style)
            .width(Length::Fill)
            .into(),
        )
    } else {
        None
    };

    let respond_col = container(
        scrollable({
            let mut col = column![
                section_head(t(Key::RespondLabel)),
                respond_card(app, p, response_delay),
            ];
            if let Some(prf) = per_rule_field {
                col = col.push(prf);
            }
            col = col.push(Space::new().height(space::S4));
            col.spacing(space::S3)
                .padding(Padding::from([space::S4, space::S3]))
        })
        .height(Length::Fill),
    )
    .width(Length::FillPortion(2))
    .height(Length::Fill);

    // Arrow divider — centred vertically
    let arrow: Element<Message> = container(
        column![
            Space::new().height(Length::Fill),
            text("→")
                .size(size::TITLE)
                .color(theme::muted(&app.theme())),
            Space::new().height(Length::Fill),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fixed(44.0))
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .into();

    let editor_row = row![when_col, arrow, respond_col].height(Length::Fill);

    // ── Recent trace activity (jump-links to Trace tab) ───────────────────
    let trace_section = trace_activity_section(app, rule);

    let mut outer = column![];
    if let Some(strip) = validation_strip {
        outer = outer.push(strip);
    }
    outer = outer
        .push(action_header)
        .push(widgets::divider())
        .push(editor_row)
        .push(widgets::divider())
        .push(trace_section);
    outer.into()
}

fn section_head(label: &str) -> Element<'_, Message> {
    text(label).size(size::SECTION).into()
}

fn card<'a>(title: &'a str, body: Element<'a, Message>) -> Element<'a, Message> {
    container(
        column![
            text(title).size(size::BODY_STRONG),
            Space::new().height(space::S2),
            body,
        ]
        .spacing(0),
    )
    .padding(Padding::from(pad::CARD))
    .style(theme::card_style)
    .width(Length::Fill)
    .into()
}

/// MK-039: a card whose heading carries an ⓘ concept hint. The hint is opt-in
/// (revealed on hover) so the default view stays uncluttered.
fn card_with_hint<'a>(
    app: &'a App,
    title: &'a str,
    hint: &'a str,
    body: Element<'a, Message>,
) -> Element<'a, Message> {
    // MK-040: Guided mode expands the hint inline as a plain gloss under the
    // heading; Expert mode shows only the ⓘ marker (hint on hover). The hint
    // text is identical in both — only its visibility differs.
    let heading: Element<Message> = if app.shows_scaffolding() {
        column![
            text(title).size(size::BODY_STRONG),
            text(hint)
                .size(size::CAPTION)
                .color(theme::muted(&app.theme())),
        ]
        .spacing(space::S1)
        .into()
    } else {
        widgets::label_with_hint(&app.theme(), title, hint)
    };

    container(column![heading, Space::new().height(space::S2), body,].spacing(0))
        .padding(Padding::from(pad::CARD))
        .style(theme::card_style)
        .width(Length::Fill)
        .into()
}

fn url_path_card<'a>(app: &'a App, p: &'a apimokka_model::RulePayload) -> Element<'a, Message> {
    let ops = UrlPathOp::all().to_vec();

    card_with_hint(
        app,
        app.t(Key::UrlPathCardTitle),
        app.t(Key::HintUrlOp),
        column![
            row![
                text_input(app.t(Key::UrlPathField), &p.url_path)
                    .on_input(Message::RuleSetUrlPath)
                    .size(size::BODY)
                    .padding(Padding::from([space::S2, space::S3]))
                    .width(Length::Fill),
                pick_list(ops, p.url_path_op, Message::RuleSetUrlPathOp)
                    .text_size(size::CAPTION)
                    .padding(Padding::from([space::S2, space::S3]))
                    .width(Length::Fixed(140.0)),
            ]
            .spacing(space::S2)
            .align_y(Alignment::Center),
            text(app.t(Key::UrlPathHint))
                .size(size::CAPTION)
                .color(theme::muted(&app.theme())),
        ]
        .spacing(space::S2)
        .into(),
    )
}

fn method_card<'a>(app: &'a App, method: &'a str) -> Element<'a, Message> {
    let methods = ["Any", "GET", "POST", "PUT", "PATCH", "DELETE"];
    let btns: Vec<Element<Message>> = methods
        .iter()
        .map(|m| {
            let active = if *m == "Any" {
                method.is_empty()
            } else {
                method == *m
            };
            let msg = if *m == "Any" {
                Message::RuleSetMethod(String::new())
            } else {
                Message::RuleSetMethod(m.to_string())
            };
            let label = if *m == "Any" {
                app.t(Key::MethodAny)
            } else {
                *m
            };
            button(text(label).size(size::CAPTION))
                .on_press(msg)
                .padding(Padding::from([space::S2, space::S3 + 2.0]))
                .style(if active {
                    theme::seg_active
                } else {
                    theme::seg_inactive
                })
                .into()
        })
        .collect();

    card(
        app.t(Key::MethodCardTitle),
        row(btns).spacing(space::S1).into(),
    )
}

fn headers_card<'a>(app: &'a App, p: &'a apimokka_model::RulePayload) -> Element<'a, Message> {
    let mut rows: Vec<Element<Message>> = p
        .headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let show_val = !h.op.value_irrelevant();
            row![
                text_input(app.t(Key::HeaderColumnName), &h.name)
                    .on_input(move |v| Message::HeaderSetName { index: i, value: v })
                    .size(size::CAPTION)
                    .padding(Padding::from([space::S2, space::S2]))
                    .width(Length::Fixed(110.0)),
                pick_list(HeaderOp::all().to_vec(), Some(h.op), move |op| {
                    Message::HeaderSetOp { index: i, op }
                })
                .text_size(size::CAPTION)
                .padding(Padding::from([space::S2, space::S2]))
                .width(Length::Fixed(110.0)),
                {
                    let val_el: Element<Message> = if show_val {
                        text_input(app.t(Key::HeaderColumnValue), &h.value)
                            .on_input(move |v| Message::HeaderSetValue { index: i, value: v })
                            .size(size::CAPTION)
                            .padding(Padding::from([space::S2, space::S2]))
                            .width(Length::Fill)
                            .into()
                    } else {
                        Space::new().width(Length::Fill).into()
                    };
                    val_el
                },
                widgets::icon_btn("✕", Message::HeaderRemove(i)),
            ]
            .spacing(space::S1)
            .align_y(Alignment::Center)
            .into()
        })
        .collect();

    rows.push(
        button(text(format!("+ {}", app.t(Key::BtnAddHeader))).size(size::CAPTION))
            .on_press(Message::HeaderAdd)
            .padding(Padding::from([space::S2, space::S3]))
            .into(),
    );

    card_with_hint(
        app,
        app.t(Key::HeadersCardTitle),
        app.t(Key::HintHeaderOp),
        column(rows).spacing(space::S2).into(),
    )
}

fn body_card<'a>(app: &'a App, p: &'a apimokka_model::RulePayload) -> Element<'a, Message> {
    let mut rows: Vec<Element<Message>> = p
        .body
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let show_val = b.op != BodyOp::Exists && b.op != BodyOp::Absent;
            let jsonpath_warn: Element<Message> = if b.path.starts_with("$.") {
                text(app.t(Key::BodyJsonpathWarn))
                    .size(size::CAPTION)
                    .color(Color::from_rgb(0.85, 0.45, 0.0))
                    .into()
            } else {
                Space::new().height(0.0).into()
            };
            column![
                row![
                    text_input("user.id", &b.path)
                        .on_input(move |v| Message::BodySetPath { index: i, value: v })
                        .size(size::CAPTION)
                        .padding(Padding::from([space::S2, space::S2]))
                        .width(Length::Fill),
                    button(text("…").size(size::CAPTION))
                        .on_press(Message::PathAssistantOpen(i))
                        .padding(Padding::from([space::S2, space::S2])),
                    pick_list(BodyOp::all().to_vec(), Some(b.op), move |op| {
                        Message::BodySetOp { index: i, op }
                    })
                    .text_size(size::CAPTION)
                    .padding(Padding::from([space::S2, space::S2]))
                    .width(Length::Fixed(120.0)),
                    {
                        let bval: Element<Message> = if show_val {
                            text_input(app.t(Key::BodyValuePlaceholder), &b.value)
                                .on_input(move |v| Message::BodySetValue { index: i, value: v })
                                .size(size::CAPTION)
                                .padding(Padding::from([space::S2, space::S2]))
                                .width(Length::Fill)
                                .into()
                        } else {
                            Space::new().width(Length::Fill).into()
                        };
                        bval
                    },
                    widgets::icon_btn("✕", Message::BodyRemove(i)),
                ]
                .spacing(space::S1)
                .align_y(Alignment::Center),
                jsonpath_warn,
            ]
            .spacing(space::S1)
            .into()
        })
        .collect();

    if p.body.is_empty() {
        rows.push(
            text(app.t(Key::BodyDottedPathHint))
                .size(size::CAPTION)
                .color(theme::muted(&app.theme()))
                .into(),
        );
    }
    rows.push(
        button(text(format!("+ {}", app.t(Key::BtnAddBodyCondition))).size(size::CAPTION))
            .on_press(Message::BodyAdd)
            .padding(Padding::from([space::S2, space::S3]))
            .into(),
    );

    card_with_hint(
        app,
        app.t(Key::BodyCardTitle),
        app.t(Key::HintBodyPath),
        column(rows).spacing(space::S2).into(),
    )
}

fn respond_card<'a>(
    app: &'a App,
    p: &'a apimokka_model::RulePayload,
    response_delay: Option<&'a str>,
) -> Element<'a, Message> {
    let is_inline = p.respond.mode == RespondMode::InlineText;
    let mode_btns: Element<Message> = row![
        mode_tab(
            app.t(Key::RespondModeInline),
            is_inline,
            RespondMode::InlineText
        ),
        mode_tab(
            app.t(Key::RespondModeFile),
            !is_inline,
            RespondMode::ServeFile
        ),
    ]
    .spacing(space::S1)
    .into();

    let body_el: Element<Message> = if is_inline {
        text_input(app.t(Key::RespondBodyPlaceholder), &p.respond.text)
            .on_input(Message::RespondSetText)
            .size(size::BODY)
            .padding(Padding::from([space::S2, space::S3]))
            .width(Length::Fill)
            .into()
    } else {
        text_input("path/to/response.json", &p.respond.file_path)
            .on_input(Message::RespondSetFilePath)
            .size(size::BODY)
            .padding(Padding::from([space::S2, space::S3]))
            .width(Length::Fill)
            .into()
    };

    let delay_str = response_delay
        .map(str::to_owned)
        .unwrap_or_else(|| p.respond.delay_milliseconds.to_string());

    card(
        app.t(Key::RespondCardTitle),
        column![
            mode_btns,
            body_el,
            row![
                widgets::field(
                    app.t(Key::RespondStatusLabel),
                    text_input("200 OK", &p.respond.status)
                        .on_input(Message::RespondSetStatus)
                        .size(size::CAPTION)
                        .padding(Padding::from([space::S2, space::S3]))
                        .width(Length::Fixed(110.0))
                        .into(),
                ),
                Space::new().width(space::S3),
                widgets::field(
                    app.t(Key::RespondDelayLabel),
                    row![
                        text_input("0", &delay_str)
                            .on_input(Message::RespondSetDelay)
                            .size(size::CAPTION)
                            .padding(Padding::from([space::S2, space::S3]))
                            .width(Length::Fixed(70.0)),
                        text(app.t(Key::RespondDelayUnit)).size(size::CAPTION),
                    ]
                    .spacing(space::S1)
                    .align_y(Alignment::Center)
                    .into(),
                ),
            ]
            .align_y(Alignment::End),
            text(app.t(Key::RespondMutexHint))
                .size(size::CAPTION)
                .color(theme::muted(&app.theme())),
        ]
        .spacing(space::S3)
        .into(),
    )
}

fn mode_tab(label: &str, active: bool, mode: RespondMode) -> Element<'_, Message> {
    button(text(label).size(size::CAPTION))
        .on_press(Message::RespondSetMode(mode))
        .padding(Padding::from([space::S2, space::S4]))
        .style(if active {
            theme::seg_active
        } else {
            theme::seg_inactive
        })
        .into()
}
