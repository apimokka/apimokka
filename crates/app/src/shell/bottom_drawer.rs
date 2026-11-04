//! MK-032 / MK-044 — Bottom drawer: validation panel and save-diff panel.
//!
//! MK-044 makes both panels actionable:
//! - Validation: groups by rule set, click-to-navigate, proper empty state.
//! - Save diff: shows rule summaries and fallback-file change indicators.
//!
//! Boundary decision: split — two parallel, independently-called panels
//! share only dispatch and two small helpers: validation
//! (`validation_content` + `durable_diagnostic_rows`, called from
//! `app/drawers.rs`) and save-diff (`save_diff_content` +
//! `last_save_report_lines`, called from `global_save_tests.rs`). A clean
//! two-way boundary at the same shape as `routes.rs`'s. Deferred to a
//! follow-up slice; recorded here rather than executed to keep this slice
//! to decisions only.

use crate::app::{
    App, FallbackSaveReport, GlobalSaveCompletion, ProgressTrust, SaveIntegrity,
    WorkspaceSaveProgress,
};
use crate::message::Message;
use crate::regions::{Cursor, DrawerField, Region};
use crate::selection::DrawerMode;
use crate::theme::{self, size, space};
use crate::widgets;
use apimokka_i18n::Key;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Color, Element, Length, Padding};

/// Task 033: same D-5 approximation as `routes::SIDEBAR_SCROLLABLE_ID` and
/// friends, for the drawer's one scrollable. `Close` sits in
/// `drawer_header`, above it, not inside it, so it is excluded from the
/// position list the same way Wizard excludes `Cancel`/`Create`.
pub(crate) const SCROLLABLE_ID: &str = "mk023-drawer";

/// Same shape as `rule_editor.rs`'s own `cursor_wrap` — every field here is
/// a one-item region (`Region::Drawer(_)`'s own `len()` is always 1; a
/// `JumpToDiagnostic(i)` and a `JumpToDiagnostic(j)` are two distinct
/// regions, not two positions within one).
fn cursor_wrap<'a>(
    app: &App,
    field: DrawerField,
    el: Element<'a, Message>,
) -> Element<'a, Message> {
    let has_cursor = app.focused_region == Some(Region::Drawer(field))
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

pub fn view(app: &App) -> Element<'_, Message> {
    let mode = match app.drawer {
        Some(m) => m,
        None => return Space::new().into(),
    };
    let body: Element<Message> = match mode {
        DrawerMode::Validation => validation_content(app),
        DrawerMode::SaveDiff => save_diff_content(app),
    };
    let mut col = column![
        drawer_header(app, mode),
        widgets::divider(),
        scrollable(body).id(SCROLLABLE_ID).height(Length::Fill),
    ];
    // D-27 (task 036): `Discard`/`Save all` sit outside the scrollable, the
    // same way `Close` already sits in `drawer_header` above it, rather
    // than taking a scroll-snap position (offset 0.5 of 3) that does not
    // guarantee visibility — D-5's own recorded limit. Visibility no longer
    // depends on scroll position at all.
    if mode == DrawerMode::SaveDiff
        && let Some(actions) = save_diff_actions(app)
    {
        col = col.push(widgets::divider());
        col = col.push(actions);
    }
    container(col)
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
            cursor_wrap(
                app,
                DrawerField::Close,
                button(text("✕").size(size::BODY))
                    .on_press(Message::CloseDrawer)
                    .padding(Padding::from([space::S1, space::S2]))
                    .style(iced::widget::button::text)
                    .into(),
            ),
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

    // Task 033: `DrawerField::JumpToDiagnostic`'s own position is counted
    // only over rows with `target.is_some()` — the identical scope
    // `regions::drawer_jump_target_count` counts, so the two can never
    // disagree about which row position N points to.
    let mut jump_index = 0usize;
    for diagnostic in rows {
        let target = diagnostic.target;
        let severity_label = match diagnostic.severity {
            apimokka_model::Severity::Error => app.t(Key::DrawerValidationErrors),
            apimokka_model::Severity::Warning => app.t(Key::DrawerValidationWarnings),
            apimokka_model::Severity::Info => app.t(Key::DrawerValidationInfo),
        };
        let heading = row![
            text(diagnostic.severity.glyph())
                .size(size::BODY)
                .color(theme::severity_color(&app.theme(), diagnostic.severity)),
            text(severity_label).size(size::LABEL),
            text(diagnostic.scope).size(size::BODY).width(Length::Fill),
            // RFC MK-059 decision 4: this whole heading is wrapped in a
            // `button(...)` below when `target` is `Some` -- text inside a
            // button is a button label, not a caption.
            text(if target.is_some() {
                app.t(Key::DrawerOpenDiagnostic)
            } else {
                ""
            })
            .size(size::LABEL)
            .color(theme::muted(&app.theme())),
            text(if target.is_some() { "→" } else { "" })
                .size(size::LABEL)
                .color(theme::muted(&app.theme())),
        ]
        .spacing(space::S2)
        .align_y(Alignment::Center);
        let heading: Element<'_, Message> = if let Some(target) = target {
            let field = DrawerField::JumpToDiagnostic(jump_index);
            jump_index += 1;
            cursor_wrap(
                app,
                field,
                button(heading)
                    .on_press(Message::JumpToDiagnostic(target))
                    .padding(0)
                    .style(iced::widget::button::text)
                    .width(Length::Fill)
                    .into(),
            )
        } else {
            heading.into()
        };
        let mut detail = column![
            heading,
            // RFC MK-059 decision 4: a validation message is arbitrary-length
            // explanatory prose, not a caption -- promoted so it does not
            // land on the readability floor for the exact text a user needs
            // to read to fix a problem.
            text(diagnostic.message)
                .size(size::BODY_SMALL)
                .line_height(theme::line_height::body_small())
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

/// The count `save_diff_content`'s own "anything to show" branch reads,
/// factored out so `regions::drawer_fields` (task 033) can read the
/// identical number when deciding whether `Discard`/`SaveAll` are regions
/// right now — the D-9 lesson, one source rather than a second count that
/// could disagree with the view's own.
pub(crate) fn save_diff_total(app: &App) -> usize {
    let Some(snap) = &app.snapshot else {
        return 0;
    };
    let dirty_rule_sets = snap.rule_sets.iter().filter(|rs| rs.file.dirty).count();
    let dirty_fallback = app
        .fallback_drafts
        .keys()
        .filter(|p| app.is_fallback_dirty(p))
        .count();
    dirty_rule_sets + dirty_fallback
}

fn save_diff_content(app: &App) -> Element<'_, Message> {
    let Some(snap) = &app.snapshot else {
        return widgets::empty_state(app.t(Key::EmptyNoWorkspaceOpen));
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

    let total = save_diff_total(app);
    if total == 0 && app.last_save_report.is_none() {
        return container(
            column![
                text("✓")
                    .size(size::TITLE)
                    .color(Color::from_rgb(0.1, 0.65, 0.1)),
                text(app.t(Key::DrawerNoUnsavedChanges)).size(size::BODY),
            ]
            .spacing(space::S2)
            .align_x(iced::Alignment::Center),
        )
        .padding(Padding::from([space::S6, space::S5]))
        .width(Length::Fill)
        .into();
    }

    let count_text = format!(
        "{} {}",
        total,
        app.t(if total == 1 {
            Key::DrawerUnsavedCountSingular
        } else {
            Key::DrawerUnsavedCountPlural
        })
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
                    .size(size::BODY_SMALL)
                    .line_height(theme::line_height::body_small())
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
                    text(app.t(Key::DrawerNoUnsavedChanges)).size(size::BODY),
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
                            .size(size::LABEL)
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
                        .size(size::BODY_SMALL)
                        .line_height(theme::line_height::body_small())
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
                            .size(size::LABEL)
                            .color(theme::muted(&app.theme())),
                        text(name.to_string()).size(size::BODY).width(Length::Fill),
                    ]
                    .spacing(space::S2)
                    .align_y(Alignment::Center),
                    row![
                        Space::new().width(Length::Fixed(space::S4)),
                        text(app.t(Key::DrawerSaveDiffFallbackMod))
                            .size(size::BODY_SMALL)
                            .line_height(theme::line_height::body_small())
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

    col.into()
}

/// D-27 (task 036): `Discard`/`Save all`, factored out of `save_diff_content`
/// so `view` can place them outside the drawer's one scrollable — `None`
/// when there is nothing to act on, mirroring `save_diff_content`'s own
/// `total == 0` check (`save_diff_total` is the one source both read, the
/// same reasoning `drawer_fields` is already built on).
fn save_diff_actions(app: &App) -> Option<Element<'_, Message>> {
    if save_diff_total(app) == 0 {
        return None;
    }
    Some(
        container(
            row![
                cursor_wrap(
                    app,
                    DrawerField::Discard,
                    // Task 034 (D-24's confirmation half): routed through the
                    // confirm dialog — a confirmation already existed for this
                    // (`ConfirmAction::DiscardChanges`, title/body keys and
                    // all) and was dispatched by nothing; one click used to
                    // discard every unsaved change immediately.
                    widgets::ghost_btn(
                        app.t(Key::BtnDiscard),
                        Message::ConfirmRequest(crate::message::ConfirmAction::DiscardChanges),
                    ),
                ),
                Space::new().width(Length::Fill),
                cursor_wrap(
                    app,
                    DrawerField::SaveAll,
                    widgets::primary_btn(app.t(Key::BtnSaveAll), Message::SaveAll),
                ),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([space::S3, space::S4]))
        .into(),
    )
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
