//! MK-032 / MK-044 — Bottom drawer: validation panel and save-diff panel.
//!
//! MK-044 makes both panels actionable:
//! - Validation: groups by rule set, click-to-navigate, proper empty state.
//! - Save diff: shows rule summaries and fallback-file change indicators.

use crate::app::{
    App, FallbackSaveReport, GlobalSaveCompletion, ProgressTrust, SaveIntegrity,
    WorkspaceSaveProgress,
};
use crate::message::Message;
use crate::selection::DrawerMode;
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Color, Element, Length, Padding};

pub fn view(app: &App) -> Element<'_, Message> {
    let mode = match app.drawer {
        Some(m) => m,
        None => return Space::new().into(),
    };
    let body: Element<Message> = match mode {
        DrawerMode::Validation => validation_content(app),
        DrawerMode::SaveDiff => save_diff_content(app),
    };
    container(column![
        drawer_header(app, mode),
        widgets::divider(),
        scrollable(body).height(Length::Fill),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::panel_style)
    .into()
}

fn drawer_header<'a>(app: &'a App, mode: DrawerMode) -> Element<'a, Message> {
    let title = match mode {
        DrawerMode::Validation => app.t(Key::DrawerValidationTitle),
        DrawerMode::SaveDiff => app.t(Key::DrawerSaveDiffTitle),
    };
    container(
        row![
            text(title).size(size::SECTION).width(Length::Fill),
            button(text("✕").size(size::BODY))
                .on_press(Message::CloseDrawer)
                .padding(Padding::from([space::S1, space::S2]))
                .style(iced::widget::button::text),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([space::S3, space::S4]))
    .into()
}

// ── Validation panel ──────────────────────────────────────────────────────────

fn validation_content(app: &App) -> Element<'_, Message> {
    if app.snapshot.is_none() {
        return widgets::empty_state(app.t(Key::DrawerValidationOk));
    }

    let rows = durable_diagnostic_rows(app);
    if rows.is_empty() {
        return container(
            column![
                text("✓")
                    .size(size::TITLE)
                    .color(Color::from_rgb(0.1, 0.65, 0.1)),
                text(app.t(Key::DrawerValidationOk)).size(size::BODY),
            ]
            .spacing(space::S2)
            .align_x(iced::Alignment::Center),
        )
        .padding(Padding::from([space::S6, space::S5]))
        .width(Length::Fill)
        .into();
    }

    let mut col = column![]
        .spacing(space::S2)
        .padding(Padding::from([space::S3, space::S4]));

    for diagnostic in rows {
        let target = diagnostic.target;
        let severity_label = match diagnostic.severity {
            apimokka_model::Severity::Error => app.t(Key::DrawerValidationErrors),
            apimokka_model::Severity::Warning => app.t(Key::DrawerValidationWarnings),
            apimokka_model::Severity::Info => app.t(Key::DrawerValidationInfo),
        };
        let heading = row![
            text(widgets::severity_glyph(diagnostic.severity))
                .size(size::BODY)
                .color(theme::severity_color(&app.theme(), diagnostic.severity)),
            text(severity_label).size(size::CAPTION),
            text(diagnostic.scope).size(size::BODY).width(Length::Fill),
            text(if target.is_some() {
                app.t(Key::DrawerOpenDiagnostic)
            } else {
                ""
            })
            .size(size::CAPTION)
            .color(theme::muted(&app.theme())),
            text(if target.is_some() { "→" } else { "" })
                .size(size::CAPTION)
                .color(theme::muted(&app.theme())),
        ]
        .spacing(space::S2)
        .align_y(Alignment::Center);
        let heading: Element<'_, Message> = if let Some(target) = target {
            button(heading)
                .on_press(Message::JumpToDiagnostic(target))
                .padding(0)
                .style(iced::widget::button::text)
                .width(Length::Fill)
                .into()
        } else {
            heading.into()
        };
        let mut detail = column![
            heading,
            text(diagnostic.message)
                .size(size::CAPTION)
                .color(theme::muted(&app.theme()))
                .width(Length::Fill),
        ]
        .spacing(space::S1);
        if app.show_problem_details
            && let Some(location) = diagnostic.location
        {
            detail = detail.push(
                text(location)
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
            );
        }
        col = col.push(
            container(detail)
                .padding(Padding::from([space::S2, space::S3]))
                .style(theme::card_style)
                .width(Length::Fill),
        );
    }

    col.into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DurableDiagnosticRow {
    pub target: Option<apimokka_model::NodeId>,
    pub severity: apimokka_model::Severity,
    pub scope: String,
    pub message: String,
    pub location: Option<String>,
}

pub(crate) fn durable_diagnostic_rows(app: &App) -> Vec<DurableDiagnosticRow> {
    let Some(snapshot) = app.snapshot.as_ref() else {
        return Vec::new();
    };
    let mut rows = snapshot
        .diagnostics
        .iter()
        .map(|diagnostic| DurableDiagnosticRow {
            target: diagnostic.node_id,
            severity: diagnostic.severity,
            scope: app.t(Key::DrawerValidationWorkspace).to_owned(),
            message: diagnostic.message.clone(),
            location: None,
        })
        .collect::<Vec<_>>();
    for rule_set in &snapshot.rule_sets {
        rows.extend(
            rule_set
                .validation
                .issues
                .iter()
                .map(|issue| DurableDiagnosticRow {
                    target: Some(issue.node_id.unwrap_or(rule_set.id.0)),
                    severity: issue.severity,
                    scope: rule_set.file.path.clone(),
                    message: issue.message.clone(),
                    location: issue.location.clone(),
                }),
        );
        for rule in &rule_set.rules {
            rows.extend(
                rule.validation
                    .issues
                    .iter()
                    .map(|issue| DurableDiagnosticRow {
                        target: Some(issue.node_id.unwrap_or(rule.id)),
                        severity: issue.severity,
                        scope: format!("{} · {}", rule_set.file.path, rule.summary()),
                        message: issue.message.clone(),
                        location: issue.location.clone(),
                    }),
            );
        }
    }
    rows
}

// ── Save-diff panel ───────────────────────────────────────────────────────────

fn save_diff_content(app: &App) -> Element<'_, Message> {
    let Some(snap) = &app.snapshot else {
        return widgets::empty_state("No workspace open.");
    };

    let dirty_rule_sets: Vec<_> = snap.rule_sets.iter().filter(|rs| rs.file.dirty).collect();

    let dirty_fallback_paths: Vec<&str> = {
        let mut v: Vec<&str> = app
            .fallback_drafts
            .keys()
            .filter(|p| app.is_fallback_dirty(p))
            .map(|p| p.as_str())
            .collect();
        v.sort();
        v
    };

    let total = dirty_rule_sets.len() + dirty_fallback_paths.len();
    if total == 0 && app.last_save_report.is_none() {
        return container(
            column![
                text("✓")
                    .size(size::TITLE)
                    .color(Color::from_rgb(0.1, 0.65, 0.1)),
                text("No unsaved changes.").size(size::BODY),
            ]
            .spacing(space::S2)
            .align_x(iced::Alignment::Center),
        )
        .padding(Padding::from([space::S6, space::S5]))
        .width(Length::Fill)
        .into();
    }

    let count_text = format!(
        "{} {} unsaved changes",
        total,
        if total == 1 {
            "file with"
        } else {
            "files with"
        }
    );

    let mut col = column![]
        .spacing(space::S2)
        .padding(Padding::from([space::S3, space::S4]));

    if app.last_save_report.is_some() {
        col = col.push(text(app.t(Key::DrawerLastSaveAttempt)).size(size::BODY_STRONG));
        let mut report = column![].spacing(space::S1);
        for line in last_save_report_lines(app) {
            report = report.push(
                text(line)
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme()))
                    .width(Length::Fill),
            );
        }
        col = col.push(
            container(report)
                .padding(Padding::from([space::S2, space::S3]))
                .style(theme::card_style)
                .width(Length::Fill),
        );
        col = col.push(Space::new().height(space::S2));
        col = col.push(text(app.t(Key::DrawerCurrentUnsaved)).size(size::BODY_STRONG));
    }

    if total == 0 {
        col = col.push(
            container(
                column![
                    text("✓")
                        .size(size::TITLE)
                        .color(Color::from_rgb(0.1, 0.65, 0.1)),
                    text("No unsaved changes.").size(size::BODY),
                ]
                .spacing(space::S2)
                .align_x(iced::Alignment::Center),
            )
            .padding(Padding::from([space::S4, space::S5]))
            .width(Length::Fill),
        );
        return col.into();
    }

    col = col.push(text(count_text).size(size::BODY_STRONG));
    col = col.push(Space::new().height(space::S2));

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
                        text("●")
                            .size(size::CAPTION)
                            .color(theme::muted(&app.theme())),
                        text(file_name).size(size::BODY).width(Length::Fill),
                    ]
                    .spacing(space::S2)
                    .align_y(Alignment::Center),
                    row![
                        Space::new().width(Length::Fixed(space::S4)),
                        text(format!(
                            "{} {} {}",
                            rule_count,
                            app.t(Key::DrawerSaveDiffChangedRules),
                            preview
                        ))
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
                        text("●")
                            .size(size::CAPTION)
                            .color(theme::muted(&app.theme())),
                        text(name.to_string()).size(size::BODY).width(Length::Fill),
                    ]
                    .spacing(space::S2)
                    .align_y(Alignment::Center),
                    row![
                        Space::new().width(Length::Fixed(space::S4)),
                        text(app.t(Key::DrawerSaveDiffFallbackMod))
                            .size(size::CAPTION)
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

pub(crate) fn last_save_report_lines(app: &App) -> Vec<String> {
    let Some(report) = app.last_save_report.as_ref() else {
        return Vec::new();
    };
    let mut lines = vec![
        app.t(match report.completion() {
            GlobalSaveCompletion::Complete => Key::SaveCompletionComplete,
            GlobalSaveCompletion::Partial => Key::SaveCompletionPartial,
            GlobalSaveCompletion::Failed => Key::SaveCompletionFailed,
            GlobalSaveCompletion::Indeterminate => Key::SaveCompletionIndeterminate,
        })
        .to_owned(),
    ];

    let progress_trust = match &report.workspace.integrity {
        SaveIntegrity::Valid => ProgressTrust::Verified,
        SaveIntegrity::ContractFault {
            reason,
            progress_trust,
        } => {
            lines.push(format!("{}: {}", app.t(Key::SaveIntegrityFailure), reason));
            *progress_trust
        }
    };

    let join_paths = |paths: &[apimokka_model::WorkspaceRelativePath]| {
        if paths.is_empty() {
            app.t(Key::SaveNone).to_owned()
        } else {
            paths
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        }
    };
    if progress_trust == ProgressTrust::Verified {
        lines.push(format!(
            "{}: {}",
            app.t(Key::SaveVerifiedWritten),
            join_paths(report.workspace.progress.written_files())
        ));
    } else {
        lines.push(format!(
            "{}: {}",
            app.t(Key::SaveReportedWritten),
            join_paths(report.workspace.progress.written_files())
        ));
        let diffs = if report.workspace.progress.diffs().is_empty() {
            app.t(Key::SaveNone).to_owned()
        } else {
            report
                .workspace
                .progress
                .diffs()
                .iter()
                .map(|diff| format!("{} ({})", diff.path, runtime_effect_label(app, diff.effect)))
                .collect::<Vec<_>>()
                .join(", ")
        };
        lines.push(format!("{}: {diffs}", app.t(Key::SaveReportedDiffs)));
    }

    if let WorkspaceSaveProgress::Failed {
        failed_file, cause, ..
    } = &report.workspace.progress
    {
        lines.push(format!(
            "{}: {} — {}",
            app.t(if progress_trust == ProgressTrust::Verified {
                Key::SaveFailedFile
            } else {
                Key::SaveReportedFailure
            }),
            failed_file,
            cause.detail()
        ));
    }

    match &report.fallback {
        FallbackSaveReport::Completed { written_keys } => lines.push(format!(
            "{}: {}",
            app.t(Key::SaveFallbackWritten),
            join_strings(app, written_keys)
        )),
        FallbackSaveReport::NotEntered { remaining_keys, .. } => lines.push(format!(
            "{}: {}",
            app.t(Key::SaveRemainingScopes),
            join_strings(app, remaining_keys)
        )),
        FallbackSaveReport::Failed {
            written_keys,
            failure,
            remaining_keys,
        } => {
            lines.push(format!(
                "{}: {}",
                app.t(Key::SaveFallbackWritten),
                join_strings(app, written_keys)
            ));
            lines.push(format!(
                "{}: {} — {}",
                app.t(Key::SaveFailedFile),
                failure.key,
                failure.cause.detail()
            ));
            lines.push(format!(
                "{}: {}",
                app.t(Key::SaveRemainingScopes),
                join_strings(app, remaining_keys)
            ));
        }
    }

    lines.push(format!(
        "{}: {}={}, {}={}",
        app.t(Key::SaveAttemptPhases),
        app.t(Key::SaveUnsavedPhase),
        runtime_effect_label(app, report.workspace.unsaved_hint),
        app.t(Key::SavePendingPhase),
        runtime_effect_label(app, report.workspace.runtime_pending)
    ));
    lines
}

fn join_strings(app: &App, values: &[String]) -> String {
    if values.is_empty() {
        app.t(Key::SaveNone).to_owned()
    } else {
        values.join(", ")
    }
}

fn runtime_effect_label(app: &App, effect: apimokka_model::RuntimeEffect) -> &'static str {
    app.t(match effect {
        apimokka_model::RuntimeEffect::None => Key::SavePhaseNone,
        apimokka_model::RuntimeEffect::Reload => Key::SavePhaseReload,
        apimokka_model::RuntimeEffect::Restart => Key::SavePhaseRestart,
    })
}
