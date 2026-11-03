//! MK-034 / MK-052 — fail-closed Test Rule dialog.
use crate::app::App;
use crate::match_test::{
    ConditionIdentity, ConditionOutcome, DiagnosticScope, EvaluationError, TestRuleOutcome,
    UnsupportedReason,
};
use crate::message::Message;
use crate::regions::{Cursor, Region, TestRuleField, test_rule_field_id};
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use iced::widget::{Space, button, column, container, mouse_area, row, text, text_input};
use iced::{Alignment, Element, Length, Padding, Theme};

/// Same shape as `rule_editor.rs`'s own `cursor_wrap` — this dialog's
/// one-item regions (task 034).
fn cursor_wrap<'a>(
    app: &App,
    field: TestRuleField,
    el: Element<'a, Message>,
) -> Element<'a, Message> {
    let has_cursor = app.focused_region == Some(Region::TestRule(field))
        && app.region_cursor == Some(Cursor::Index(0));
    container(el)
        .padding(space::S1)
        .style(if has_cursor {
            theme::cursor_style
        } else {
            |_: &iced::Theme| iced::widget::container::Style::default()
        })
        .into()
}

/// Same shape as `rule_editor.rs`'s own `cursor_wrap_group` — the 5 method
/// buttons.
fn cursor_wrap_group<'a>(
    app: &App,
    field: TestRuleField,
    index: usize,
    el: Element<'a, Message>,
) -> Element<'a, Message> {
    let has_cursor = app.focused_region == Some(Region::TestRule(field))
        && app.region_cursor == Some(Cursor::Index(index));
    container(el)
        .padding(space::S1)
        .style(if has_cursor {
            theme::cursor_style
        } else {
            |_: &iced::Theme| iced::widget::container::Style::default()
        })
        .into()
}

/// D-18's own `mouse_area::on_release` idiom, for the 3 text fields.
fn click_focus<'a>(
    field: TestRuleField,
    el: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    mouse_area(el)
        .on_release(Message::RegionClicked(Region::TestRule(field)))
        .into()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let tr = &app.test_rule;
    let limitations =
        crate::match_test::unsupported_conditions(app.selected_rule().map(|rule| &rule.payload));

    let result_el: Element<Message> = match &tr.result {
        None => text(app.t(Key::TestRuleResultHint))
            .size(size::BODY_SMALL)
            .line_height(theme::line_height::body_small())
            .color(theme::muted(&app.theme()))
            .into(),
        Some(result) => {
            let style: fn(&Theme) -> iced::widget::container::Style = match result.outcome {
                TestRuleOutcome::Matched => theme::card_selected_style,
                TestRuleOutcome::NoMatch => theme::chip_style,
                TestRuleOutcome::Unsupported | TestRuleOutcome::Error => theme::banner_style,
            };
            let mut details: Vec<Element<Message>> = vec![
                text(result_title(app, result.outcome))
                    .size(size::BODY)
                    .into(),
            ];
            for line in result_lines(app, result) {
                details.push(text(line).size(size::CAPTION).into());
            }
            container(column(details).spacing(space::S1))
                .padding(Padding::from([space::S3, space::S4]))
                .style(style)
                .width(Length::Fill)
                .into()
        }
    };

    let limitation_el: Element<Message> = if limitations.is_empty() {
        Space::new().height(0).into()
    } else {
        container(
            text(app.t(Key::TestRuleUnableVerify))
                .size(size::BODY_SMALL)
                .line_height(theme::line_height::body_small()),
        )
        .padding(Padding::from([space::S2, space::S3]))
        .style(theme::banner_style)
        .width(Length::Fill)
        .into()
    };

    let methods = ["GET", "POST", "PUT", "PATCH", "DELETE"];
    let method_btns: Vec<Element<Message>> = methods
        .iter()
        .enumerate()
        .map(|(i, method)| {
            let active = tr.method.to_uppercase() == *method;
            let btn = button(text(*method).size(size::LABEL))
                .on_press(Message::TestRuleSetMethod((*method).to_owned()))
                .padding(Padding::from([space::S2, space::S3]))
                .style(if active {
                    theme::seg_active
                } else {
                    theme::seg_inactive
                });
            cursor_wrap_group(app, TestRuleField::Method, i, btn.into())
        })
        .collect();

    container(
        column![
            row![
                text(app.t(Key::TestRuleTitle))
                    .size(size::SECTION)
                    .width(Length::Fill),
                cursor_wrap(
                    app,
                    TestRuleField::Dismiss,
                    button(text("✕").size(size::BODY))
                        .on_press(Message::TestRuleClose)
                        .padding(Padding::from([space::S1, space::S2]))
                        .into(),
                ),
            ]
            .align_y(Alignment::Center),
            text(app.t(Key::TestRuleHint))
                .size(size::BODY_SMALL)
                .line_height(theme::line_height::body_small())
                .color(theme::muted(&app.theme())),
            limitation_el,
            widgets::divider(),
            widgets::field(
                app.t(Key::TestRuleMethod),
                row(method_btns).spacing(space::S1).into()
            ),
            widgets::field(
                app.t(Key::TestRulePath),
                click_focus(
                    TestRuleField::Path,
                    text_input("/api/orders", &tr.url_path)
                        .id(test_rule_field_id(TestRuleField::Path).unwrap())
                        .on_input(Message::TestRuleSetPath)
                        .style(theme::text_input_style)
                        .size(size::BODY)
                        .padding(Padding::from([space::S2, space::S3]))
                        .width(Length::Fill),
                ),
            ),
            widgets::field(
                app.t(Key::TestRuleHeaders),
                click_focus(
                    TestRuleField::Headers,
                    text_input("content-type: application/json", &tr.headers_text)
                        .id(test_rule_field_id(TestRuleField::Headers).unwrap())
                        .on_input(Message::TestRuleSetHeaders)
                        .style(theme::text_input_style)
                        .size(size::CAPTION)
                        .padding(Padding::from([space::S2, space::S3]))
                        .width(Length::Fill),
                ),
            ),
            widgets::field(
                app.t(Key::TestRuleBody),
                click_focus(
                    TestRuleField::Body,
                    text_input("{}", &tr.body)
                        .id(test_rule_field_id(TestRuleField::Body).unwrap())
                        .on_input(Message::TestRuleSetBody)
                        .style(theme::text_input_style)
                        .size(size::CAPTION)
                        .padding(Padding::from([space::S2, space::S3]))
                        .width(Length::Fill),
                ),
            ),
            result_el,
            widgets::divider(),
            row![
                Space::new().width(Length::Fill),
                cursor_wrap(
                    app,
                    TestRuleField::Close,
                    widgets::ghost_btn(app.t(Key::BtnClose), Message::TestRuleClose),
                ),
                cursor_wrap(
                    app,
                    TestRuleField::Run,
                    widgets::primary_btn(app.t(Key::BtnRunTest), Message::TestRuleRun),
                ),
            ]
            .spacing(space::S3)
            .align_y(Alignment::Center),
        ]
        .spacing(space::S3)
        .padding(space::S5)
        .width(Length::Fixed(540.0)),
    )
    .style(theme::dialog_style)
    .into()
}

pub(crate) fn result_lines(app: &App, result: &crate::match_test::TestRuleResult) -> Vec<String> {
    let mut lines = Vec::with_capacity(result.diagnostics.len() + result.conditions.len());
    for diagnostic in &result.diagnostics {
        lines.push(format!(
            "{}: {}",
            diagnostic_label(app, &diagnostic.scope),
            error_label(app, &diagnostic.reason)
        ));
    }
    for condition in &result.conditions {
        lines.push(format!(
            "{}: {}",
            condition_label(app, &condition.condition),
            outcome_label(app, &condition.outcome)
        ));
    }
    lines
}

pub(crate) fn result_title(app: &App, outcome: TestRuleOutcome) -> &str {
    match outcome {
        TestRuleOutcome::Matched => app.t(Key::TestRuleMatched),
        TestRuleOutcome::NoMatch => app.t(Key::TestRuleNoMatch),
        TestRuleOutcome::Unsupported => app.t(Key::TestRuleUnsupported),
        TestRuleOutcome::Error => app.t(Key::TestRuleError),
    }
}

fn condition_label(app: &App, identity: &ConditionIdentity) -> String {
    match identity {
        ConditionIdentity::Method => app.t(Key::TestRuleMethod).to_owned(),
        ConditionIdentity::UrlPath => app.t(Key::TestRulePath).to_owned(),
        ConditionIdentity::Header { index, name } => {
            format!("{} {} ({name})", app.t(Key::TestRuleHeaders), index + 1)
        }
        ConditionIdentity::Body { index, path } => {
            format!("{} {} ({path})", app.t(Key::TestRuleBody), index + 1)
        }
    }
}

fn diagnostic_label(app: &App, scope: &DiagnosticScope) -> String {
    match scope {
        DiagnosticScope::Selection => app.t(Key::TestRuleScopeSelection).to_owned(),
        DiagnosticScope::RequestMethod => app.t(Key::TestRuleScopeRequestMethod).to_owned(),
        DiagnosticScope::RequestHeaderLine(line) => {
            format!("{} {line}", app.t(Key::TestRuleScopeHeaderLine))
        }
        DiagnosticScope::RequestBody => app.t(Key::TestRuleScopeRequestBody).to_owned(),
    }
}

fn outcome_label(app: &App, outcome: &ConditionOutcome) -> String {
    match outcome {
        ConditionOutcome::Passed => app.t(Key::TestRuleConditionPassed).to_owned(),
        ConditionOutcome::Failed => app.t(Key::TestRuleConditionFailed).to_owned(),
        ConditionOutcome::Unsupported { reason } => format!(
            "{} — {}",
            app.t(Key::TestRuleConditionUnsupported),
            unsupported_label(app, reason)
        ),
        ConditionOutcome::Error { reason } => format!(
            "{} — {}",
            app.t(Key::TestRuleConditionError),
            error_label(app, reason)
        ),
    }
}

fn unsupported_label(app: &App, reason: &UnsupportedReason) -> String {
    match reason {
        UnsupportedReason::ConfiguredMethod(method) => {
            format!("{}: {method}", app.t(Key::TestRuleReasonUnsupportedMethod))
        }
        UnsupportedReason::UrlOperator(operator) => format!(
            "{}: {} {operator}",
            app.t(Key::TestRuleReasonUnsupportedOperator),
            app.t(Key::TestRulePath)
        ),
        UnsupportedReason::HeaderOperator(operator) => format!(
            "{}: {} {operator}",
            app.t(Key::TestRuleReasonUnsupportedOperator),
            app.t(Key::TestRuleHeaders)
        ),
        UnsupportedReason::BodyOperator(operator) => format!(
            "{}: {} {operator}",
            app.t(Key::TestRuleReasonUnsupportedOperator),
            app.t(Key::TestRuleBody)
        ),
    }
}

fn error_label(app: &App, reason: &EvaluationError) -> String {
    let key = match reason {
        EvaluationError::NoRuleSelected => Key::TestRuleReasonNoSelection,
        EvaluationError::InvalidRequestMethod(_) | EvaluationError::InvalidConfiguredMethod(_) => {
            Key::TestRuleReasonInvalidMethod
        }
        EvaluationError::MissingHeaderColon
        | EvaluationError::InvalidHeaderName(_)
        | EvaluationError::InvalidHeaderValue
        | EvaluationError::HeaderValueNotText
        | EvaluationError::InvalidConfiguredHeaderName(_) => Key::TestRuleReasonInvalidHeader,
        EvaluationError::DuplicateHeader { .. } => Key::TestRuleReasonDuplicateHeader,
        EvaluationError::InvalidRequestBody => Key::TestRuleReasonInvalidBody,
        EvaluationError::InvalidConfiguredJson
        | EvaluationError::InvalidConfiguredNumber
        | EvaluationError::InvalidConfiguredInteger
        | EvaluationError::InvalidConfiguredLength => Key::TestRuleReasonInvalidConfig,
    };
    let label = app.t(key);
    match reason {
        EvaluationError::InvalidRequestMethod(value)
        | EvaluationError::InvalidConfiguredMethod(value)
        | EvaluationError::InvalidHeaderName(value)
        | EvaluationError::InvalidConfiguredHeaderName(value) => format!("{label}: {value}"),
        EvaluationError::DuplicateHeader { name, first_line } => {
            format!(
                "{label}: {name} ({} {first_line})",
                app.t(Key::TestRuleScopeHeaderLine)
            )
        }
        _ => label.to_owned(),
    }
}
