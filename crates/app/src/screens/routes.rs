//! MK-028 — Routes workbench. Three columns: sidebar / rule editor / right column.
//!
//! Split by boundary (RFC MK-057): sidebar, rule-set configuration, rule
//! editor, fallback editor, script viewer, and trace activity each live in
//! their own sibling module. This file keeps only the top-level layout and
//! the selection-priority dispatch that wires them together.

mod fallback_editor;
mod rule_editor;
mod rule_set_config;
mod script_viewer;
mod sidebar;
mod trace_activity;

use crate::app::App;
use crate::message::Message;
use crate::theme::space;
use crate::widgets;
use apimokka_i18n::Key;
use iced::widget::{column, container, row};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let sidebar = sidebar::left_sidebar(app);
    let centre = centre_panel(app);

    row![sidebar, centre].height(Length::Fill).into()
}

fn centre_panel(app: &App) -> Element<'_, Message> {
    let snap = match &app.snapshot {
        Some(s) => s,
        None => {
            return container(widgets::empty_state(app.t(Key::EmptyNoRuleSelected)))
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }
    };

    // Priority 1: rule selected → rule editor
    if let Some(rule_id) = app.selection.rule
        && let Some((_, rule)) = snap.find_rule(rule_id)
    {
        let payload = snap
            .rule_draft(rule_id)
            .map(|draft| &draft.payload)
            .unwrap_or(&rule.payload);
        return rule_editor::rule_editor(app, rule, payload);
    }

    // Priority 2: fallback file selected → JSON file editor
    // (must be above rule-set-config: SelectFileRoute clears rule but not rule_set)
    if let Some(path) = &app.selection.file_route
        && let Some(file) = snap
            .fallback_files
            .iter()
            .find(|f| &f.name == path || &f.path == path)
    {
        return fallback_editor::fallback_file_editor(app, file);
    }

    // Priority 3: middleware script selected → read-only viewer
    if let Some(path) = &app.selection.script
        && let Some(script) = snap.middleware_scripts.iter().find(|s| &s.path == path)
    {
        return script_viewer::script_viewer(app, script);
    }

    // Priority 4: rule set activated (no rule/file/script) → rule set configuration
    if let (Some(rs_id), None) = (app.selection.rule_set, app.selection.rule)
        && let Some(rs) = snap.rule_sets.iter().find(|rs| rs.id == rs_id)
    {
        return rule_set_config::rule_set_config(app, rs);
    }

    // Empty state — distinguish blank workspace (no rule sets) from "nothing selected"
    let has_rule_sets = !snap.rule_sets.is_empty();
    container(
        column![
            widgets::empty_state(if has_rule_sets {
                app.t(Key::EmptyNoRuleSelected)
            } else {
                app.t(Key::EmptyBlankWorkspace)
            }),
            container(if has_rule_sets {
                widgets::primary_btn(app.t(Key::EmptyNoRuleSelectedCta), {
                    if let Some(rs_id) = app.selection.rule_set {
                        Message::AddRule(rs_id)
                    } else if let Some(s) = &app.snapshot {
                        s.rule_sets
                            .first()
                            .map(|rs| Message::AddRule(rs.id))
                            .unwrap_or(Message::Noop)
                    } else {
                        Message::Noop
                    }
                })
            } else {
                widgets::primary_btn(app.t(Key::BtnAddRuleSet), Message::AddRuleSet)
            })
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
        ]
        .spacing(space::S3),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_y(Length::Fill)
    .into()
}
