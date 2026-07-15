//! MK-028 — Routes workbench. Three columns: sidebar / rule editor / right column.
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input, Space};
use iced::{Alignment, Color, Element, Length, Padding};
use apimokka_i18n::Key;
use apimokka_model::{BodyOp, HeaderOp, UrlPathOp, respond::RespondMode, settings::Strategy, snapshot::RuleSetView};
use crate::app::App;
use crate::message::Message;
use crate::theme::{self, pad, size, space};
use crate::widgets;

pub fn view(app: &App) -> Element<'_, Message> {
    let sidebar = left_sidebar(app);
    let centre  = centre_panel(app);

    row![sidebar, centre]
        .height(Length::Fill)
        .into()
}

// ── Left sidebar ──────────────────────────────────────────────────────────────

fn left_sidebar(app: &App) -> Element<'_, Message> {
    let t = |k| app.t(k);
    let snap = match &app.snapshot {
        Some(s) => s,
        None => return container(widgets::empty_state("No workspace open."))
            .width(Length::Fixed(280.0)).height(Length::Fill)
            .style(theme::panel_style).into(),
    };

    let mut col = column![].spacing(space::S1)
        .padding(Padding::from([space::S3, space::S2]));

    // ── Rule sets (accordion: only one open at a time) ────────────────────
    col = col.push(
        text(t(Key::RoutesRuleSets)).size(size::CAPTION)
            .color(theme::muted(&app.theme()))
    );
    for rs in &snap.rule_sets {
        let is_open = app.rule_set_open == Some(rs.id);
        col = col.push(rule_set_group(app, rs, is_open));
    }
    col = col.push(
        button(text(format!("+ {}", t(Key::BtnAddRuleSet))).size(size::CAPTION))
            .on_press(Message::AddRuleSet)
            .padding(Padding::from([space::S1, space::S3]))
            .style(iced::widget::button::text)
            .width(Length::Fill),
    );

    // ── Fallback files (collapsed by default) ─────────────────────────────
    col = col.push(widgets::divider());
    let fb_open = app.fallback_section_open;
    let fb_chevron = if fb_open { "▾" } else { "▸" };
    let fb_count   = snap.fallback_files.len();
    col = col.push(
        button(
            row![
                text(fb_chevron).size(size::CAPTION).color(theme::muted(&app.theme())),
                text(t(Key::RoutesFallbackFiles)).size(size::CAPTION)
                    .color(theme::muted(&app.theme()))
                    .width(Length::Fill),
                text(format!("({})", fb_count)).size(size::CAPTION)
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
            let sel    = app.selection.file_route.as_deref() == Some(f.path.as_str());
            let hint   = f.route_hint.as_deref().unwrap_or("");
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
                                text("{ }").size(size::CAPTION)
                                    .color(theme::muted(&app.theme())),
                                text(f.name.as_str()).size(size::BODY).width(Length::Fill),
                                dirty_el,
                            ]
                            .spacing(space::S2).align_y(Alignment::Center),
                            text(hint).size(size::CAPTION)
                                .color(theme::muted(&app.theme())),
                        ]
                        .spacing(2.0),
                    )
                    .padding(Padding::from([space::S2, space::S3]))
                    .style(if sel { theme::card_selected_style } else { theme::card_style })
                    .width(Length::Fill),
                )
                .on_press(Message::SelectFileRoute(f.path.clone()))
                .padding(0).style(theme::naked).width(Length::Fill),
            );
        }
        col = col.push(
            button(text(format!("+ {}", t(Key::BtnAddFallbackFile))).size(size::CAPTION))
                .on_press(Message::Noop)
                .padding(Padding::from([space::S1, space::S3]))
                .style(iced::widget::button::text)
                .width(Length::Fill),
        );
    }

    // ── Middleware scripts (collapsed by default) ─────────────────────────
    col = col.push(widgets::divider());
    let mw_open    = app.middleware_section_open;
    let mw_chevron = if mw_open { "▾" } else { "▸" };
    let mw_count   = snap.middleware_scripts.len();
    col = col.push(
        button(
            row![
                text(mw_chevron).size(size::CAPTION).color(theme::muted(&app.theme())),
                text(t(Key::RoutesMiddleware)).size(size::CAPTION)
                    .color(theme::muted(&app.theme()))
                    .width(Length::Fill),
                text(format!("({})", mw_count)).size(size::CAPTION)
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
            let name     = s.path.rsplit('/').next().unwrap_or(&s.path);
            let path_str = s.path.clone();
            let sel      = app.selection.script.as_deref() == Some(&path_str);
            col = col.push(
                button(
                    container(text(name).size(size::BODY))
                        .padding(Padding::from([space::S2, space::S3]))
                        .style(if sel { theme::card_selected_style } else { theme::card_style })
                        .width(Length::Fill),
                )
                .on_press(Message::SelectScript(path_str))
                .padding(0).style(theme::naked).width(Length::Fill),
            );
        }
        col = col.push(
            button(text("+ Add .rhai").size(size::CAPTION))
                .on_press(Message::Noop)  // stub
                .padding(Padding::from([space::S1, space::S3]))
                .style(iced::widget::button::text)
                .width(Length::Fill),
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
                text(chevron).size(size::CAPTION).color(theme::muted(&app.theme())),
                text(file_name).size(size::BODY).width(Length::Fill),
                text(format!("({})", rule_count))
                    .size(size::CAPTION).color(theme::muted(&app.theme())),
                dirty_el,
            ]
            .spacing(space::S2)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([space::S2, space::S3]))
        .style(if rs_selected { theme::card_parent_selected_style } else { theme::card_style })
        .width(Length::Fill),
    )
    .on_press(Message::SelectRuleSet(rs.id))
    .padding(0).style(theme::naked).width(Length::Fill);

    if !is_open {
        // Collapsed: only show the header row
        return column![rs_row].spacing(0).into();
    }

    // Expanded: show rules
    let rule_rows: Vec<Element<Message>> = rs.rules.iter().map(|rule| {
        let rule_selected = app.selection.rule == Some(rule.id);
        let has_issues    = !rule.validation.issues.is_empty();
        let status_glyph: Element<Message> = if has_issues {
            text("⚠").size(size::CAPTION).color(Color::from_rgb(0.85, 0.45, 0.0)).into()
        } else if rule.matched_by_latest_trace {
            text("✓").size(size::CAPTION).color(Color::from_rgb(0.10, 0.65, 0.10)).into()
        } else {
            Space::new().width(0.0).into()
        };

        let summary = rule.summary();
        button(
            container(
                row![
                    text("⠿").size(size::CAPTION).color(theme::muted(&app.theme())),
                    text(summary).size(size::CAPTION).width(Length::Fill),
                    status_glyph,
                ]
                .spacing(space::S2)
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([space::S1 + 2.0, space::S3]))
            .style(if rule_selected { theme::card_selected_style } else { theme::card_style })
            .width(Length::Fill),
        )
        .on_press(Message::SelectRule(rule.id))
        .padding(0).style(theme::naked).width(Length::Fill)
        .into()
    }).collect();

    let add_rule_row = button(
        row![
            Space::new().width(Length::Fixed(space::S5)),
            text(format!("+ {}", app.t(Key::BtnAddRule))).size(size::CAPTION),
        ],
    )
    .on_press(Message::AddRule(rs.id))
    .padding(Padding::from([space::S1, space::S3]))
    .style(iced::widget::button::text)
    .width(Length::Fill);

    let mut col = column![rs_row, Space::new().height(space::S1)].spacing(space::S1);
    for r in rule_rows { col = col.push(r); }
    col = col.push(add_rule_row);
    col.into()
}

fn centre_panel(app: &App) -> Element<'_, Message> {
    let snap = match &app.snapshot {
        Some(s) => s,
        None => return container(widgets::empty_state(app.t(Key::EmptyNoRuleSelected)))
            .width(Length::Fill).height(Length::Fill).into(),
    };

    // Priority 1: rule selected → rule editor
    if let Some(rule_id) = app.selection.rule {
        if let Some((_, rule)) = snap.find_rule(rule_id) {
            return rule_editor(app, rule);
        }
    }

    // Priority 2: fallback file selected → JSON file editor
    // (must be above rule-set-config: SelectFileRoute clears rule but not rule_set)
    if let Some(path) = &app.selection.file_route {
        if let Some(file) = snap.fallback_files.iter().find(|f| &f.name == path || &f.path == path) {
            return fallback_file_editor(app, file);
        }
    }

    // Priority 3: middleware script selected → read-only viewer
    if let Some(path) = &app.selection.script {
        if let Some(script) = snap.middleware_scripts.iter().find(|s| &s.path == path) {
            return script_viewer(app, script);
        }
    }

    // Priority 4: rule set activated (no rule/file/script) → rule set configuration
    if let (Some(rs_id), None) = (app.selection.rule_set, app.selection.rule) {
        if let Some(rs) = snap.rule_sets.iter().find(|rs| rs.id == rs_id) {
            return rule_set_config(app, rs);
        }
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
            container(
                if has_rule_sets {
                    widgets::primary_btn(app.t(Key::EmptyNoRuleSelectedCta), {
                        if let Some(rs_id) = app.selection.rule_set {
                            Message::AddRule(rs_id)
                        } else if let Some(s) = &app.snapshot {
                            s.rule_sets.first().map(|rs| Message::AddRule(rs.id))
                                .unwrap_or(Message::Noop)
                        } else {
                            Message::Noop
                        }
                    })
                } else {
                    widgets::primary_btn(app.t(Key::BtnAddRuleSet), Message::AddRuleSet)
                }
            )
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

// ── Rule set configuration ────────────────────────────────────────────────────

fn rule_set_config<'a>(app: &'a App, rs: &'a RuleSetView) -> Element<'a, Message> {
    let t = |k| app.t(k);
    let file_name = rs.file.path.rsplit('/').next().unwrap_or(&rs.file.path);

    let header = container(
        row![
            column![
                text(file_name).size(size::SECTION),
                text(rs.file.path.as_str()).size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
                text(format!("{} rules", rs.rules.len())).size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
            ]
            .spacing(space::S1)
            .width(Length::Fill),
            widgets::danger_btn(t(Key::BtnDelete), Message::DeleteRuleSet(rs.id)),
        ]
        .spacing(space::S3)
        .align_y(Alignment::Start),
    )
    .padding(Padding::from([space::S4, space::S5]))
    .width(Length::Fill);

    let rule_rows: Vec<Element<Message>> = rs.rules.iter().map(|r| {
        let summary = r.summary();
        let rule_id = r.id;
        button(
            container(
                row![
                    text("⠿").size(size::CAPTION).color(theme::muted(&app.theme())),
                    text(summary).size(size::BODY).width(Length::Fill),
                ]
                .spacing(space::S2)
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([space::S2, space::S3]))
            .style(theme::card_style)
            .width(Length::Fill),
        )
        .on_press(Message::SelectRule(rule_id))
        .padding(0).style(theme::naked).width(Length::Fill)
        .into()
    }).collect();

    let rules_body: Element<Message> = if rule_rows.is_empty() {
        widgets::empty_state(t(Key::EmptyRuleSetNoRules))
    } else {
        column(rule_rows).spacing(space::S1).into()
    };

    // MK-043: strategy section — always visible in Expert, collapsible in Guided.
    let active_strategy = app.snapshot.as_ref()
        .map(|s| s.root_settings.strategy)
        .unwrap_or(Strategy::FirstMatch);

    let strategy_section: Element<Message> = {
        let dropdown = pick_list(
            Strategy::all().to_vec(),
            Some(active_strategy),
            Message::RuleSetSetStrategy,
        )
        .text_size(size::BODY)
        .padding(Padding::from([space::S2, space::S3]));

        let help_text = text(active_strategy.help()).size(size::CAPTION)
            .color(theme::muted(&app.theme()));

        let heading: Element<Message> = if app.shows_scaffolding() {
            // Guided: ⓘ hint expanded inline
            column![
                text(t(Key::RuleSetConfigStrategy)).size(size::BODY_STRONG),
                text(active_strategy.help()).size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
            ]
            .spacing(space::S1)
            .into()
        } else {
            widgets::label_with_hint(&app.theme(), t(Key::RuleSetConfigStrategy), t(Key::HintStrategy))
        };

        let inner = column![
            heading,
            row![dropdown, Space::new().width(space::S3), help_text]
                .align_y(Alignment::Center),
        ]
        .spacing(space::S2);

        if app.shows_scaffolding() {
            // Guided: strategy section collapsed by default
            let (chevron, label) = if app.rule_set_config_more {
                ("▾", t(Key::RuleSetConfigFewerOptions))
            } else {
                ("▸", t(Key::RuleSetConfigMoreOptions))
            };
            let toggle = button(
                row![
                    text(chevron).size(size::CAPTION).color(theme::muted(&app.theme())),
                    text(label).size(size::CAPTION).color(theme::muted(&app.theme())),
                ]
                .spacing(space::S2)
                .align_y(Alignment::Center),
            )
            .on_press(Message::ToggleRuleSetConfigMore)
            .padding(Padding::from([space::S2, space::S3]))
            .style(iced::widget::button::text);

            if app.rule_set_config_more {
                column![inner, toggle].spacing(space::S2).into()
            } else {
                toggle.into()
            }
        } else {
            inner.into()
        }
    };

    container(
        column![
            header,
            widgets::divider(),
            scrollable(
                column![
                    rules_body,
                    Space::new().height(space::S3),
                    widgets::divider(),
                    strategy_section,
                    Space::new().height(space::S3),
                    container(
                        widgets::primary_btn(t(Key::BtnAddRule), Message::AddRule(rs.id)),
                    )
                    .padding(Padding::from([0.0, space::S5])),
                ]
                .spacing(0)
                .padding(Padding::from([space::S3, space::S5])),
            )
            .height(Length::Fill),
        ]
        .spacing(0),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn rule_editor<'a>(app: &'a App, rule: &'a apimokka_model::snapshot::RuleView) -> Element<'a, Message> {
    let p = &rule.payload;
    let t = |k| app.t(k);
    let rule_id = rule.id;

    // MK-043: active strategy drives conditional per-rule field visibility.
    let active_strategy = app.snapshot.as_ref()
        .map(|s| s.root_settings.strategy)
        .unwrap_or(Strategy::FirstMatch);

    // ── Validation issues strip ────────────────────────────────────────────
    // Shown above the action header when the rule has issues. Uses the
    // non-empty validation.issues from the mock data (e.g. missing weight).
    let validation_strip: Option<Element<Message>> = if !rule.validation.issues.is_empty() {
        let msgs: Vec<Element<Message>> = rule.validation.issues.iter().map(|issue| {
            row![
                text("⚠").size(size::CAPTION).color(Color::from_rgb(0.85, 0.45, 0.0)),
                text(issue.message.as_str()).size(size::CAPTION),
            ]
            .spacing(space::S2)
            .into()
        }).collect();
        Some(
            container(
                column![
                    text(t(Key::RuleEditorValidationWarning)).size(size::CAPTION)
                        .color(theme::muted(&app.theme())),
                    column(msgs).spacing(space::S1),
                ]
                .spacing(space::S1),
            )
            .padding(Padding::from([space::S2, space::S5]))
            .width(Length::Fill)
            .style(theme::banner_style)
            .into()
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
    let test_ready = if has_when { Some(Message::TestRuleOpen) } else { None };

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
            section_head(t(Key::WhenLabel)).into(),
            url_path_card(app, p),
            method_card(app, &p.method),
        ];

        if app.shows_scaffolding() {
            // Guided: show headers + body only when expanded.
            let header_count = p.headers.len();
            let body_count   = p.body.len();
            let active_hidden = header_count + body_count;

            if app.rule_when_more {
                // Expanded: show both advanced cards then a "Fewer" toggle.
                cards.push(headers_card(app, p));
                cards.push(body_card(app, p));
                cards.push(
                    button(
                        row![
                            text("▾").size(size::CAPTION)
                                .color(theme::muted(&app.theme())),
                            text(t(Key::LayoutFewerWhen)).size(size::CAPTION)
                                .color(theme::muted(&app.theme())),
                        ]
                        .spacing(space::S2)
                        .align_y(Alignment::Center),
                    )
                    .on_press(Message::ToggleRuleWhenMore)
                    .padding(Padding::from([space::S2, space::S3]))
                    .style(iced::widget::button::text)
                    .into()
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
                    text(count_str).size(size::CAPTION)
                        .color(theme::muted(&app.theme()))
                        .into()
                } else {
                    Space::new().width(Length::Fixed(0.0)).into()
                };

                cards.push(
                    row![
                        button(
                            row![
                                text("▸").size(size::CAPTION)
                                    .color(theme::muted(&app.theme())),
                                text(t(Key::LayoutMoreWhen)).size(size::CAPTION)
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
                    .into()
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
    let per_rule_field: Option<Element<Message>> =
        if active_strategy.needs_per_rule_field()
            && (!app.shows_scaffolding() || app.rule_when_more)
        {
            // Build each variant directly so the compiler has unambiguous types.
            let (label_key, hint_key, field_el): (Key, Key, Element<Message>) =
                match active_strategy {
                    Strategy::WeightedRandom => {
                        let current = p.weight.map(|w| w.to_string()).unwrap_or_default();
                        let inp = text_input("", &current)
                            .on_input(Message::RuleWeightChanged)
                            .size(size::BODY)
                            .padding(Padding::from([space::S2, space::S3]))
                            .width(Length::Fixed(100.0));
                        (Key::RuleWeightLabel, Key::RuleWeightHint, inp.into())
                    }
                    Strategy::Priority => {
                        let current = p.priority.map(|pr| pr.to_string()).unwrap_or_default();
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
        scrollable(
            {
                let mut col = column![
                    section_head(t(Key::RespondLabel)),
                    respond_card(app, p),
                ];
                if let Some(prf) = per_rule_field {
                    col = col.push(prf);
                }
                col = col.push(Space::new().height(space::S4));
                col.spacing(space::S3)
                    .padding(Padding::from([space::S4, space::S3]))
            },
        )
        .height(Length::Fill),
    )
    .width(Length::FillPortion(2))
    .height(Length::Fill);

    // Arrow divider — centred vertically
    let arrow: Element<Message> = container(
        column![
            Space::new().height(Length::Fill),
            text("→").size(size::TITLE).color(theme::muted(&app.theme())),
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

fn fallback_file_editor<'a>(app: &'a App, file: &'a apimokka_model::node::FileNodeView) -> Element<'a, Message> {
    let t = |k| app.t(k);
    let route_hint = file.route_hint.as_deref().unwrap_or("/");
    let dirty = app.is_fallback_dirty(&file.path);
    let valid = app.fallback_json_valid(&file.path);
    let status = app.fallback_status_draft.get(&file.path)
        .map(|s| s.as_str())
        .unwrap_or("200 OK");

    // ── Header ────────────────────────────────────────────────────────────

    let header_col = column![
        row![
            text(file.name.as_str()).size(size::SECTION),
            Space::new().width(space::S2),
            // Dirty dot mirrors the sidebar marker (MK-038)
            {
                let dot: Element<Message> = if dirty {
                    widgets::dirty_dot()
                } else {
                    Space::new().width(Length::Fixed(0.0)).into()
                };
                dot
            },
            Space::new().width(Length::Fill),
        ]
        .align_y(Alignment::Center),
        container(
            row![
                text(t(Key::FallbackServesLabel)).size(size::BODY),
                Space::new().width(space::S1),
                text("GET").size(size::BODY),
                text(route_hint).size(size::BODY),
            ]
            .spacing(space::S2)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([space::S1 + 2.0, space::S3]))
        .style(theme::chip_style),
        text(t(Key::FallbackRouteExplanation))
            .size(size::CAPTION)
            .color(theme::muted(&app.theme())),
    ]
    .spacing(space::S2);

    // ── Content editor (multi-line, MK-038) ───────────────────────────────
    // JSON syntax highlighting via iced's `highlighter` feature (syntect).
    // Disabled theme: SolarizedDark/InspiredGitHub — pick in Settings (future).

    let editor: Element<Message> = if let Some(content) = app.fallback_drafts.get(&file.path) {
        iced::widget::text_editor(content)
            .on_action(Message::FallbackEditorAction)
            .size(size::MONO)
            .font(iced::Font::MONOSPACE)
            .height(Length::Fill)
            .highlight("json", iced::highlighter::Theme::InspiredGitHub)
            .into()
    } else {
        // Draft is created on selection; this branch is defensive only.
        widgets::empty_state(t(Key::FallbackEmptyHint))
    };

    let content_card = container(
        column![
            row![
                text(t(Key::FallbackContentLabel))
                    .size(size::BODY)
                    .width(Length::Fill),
                text(file.path.as_str())
                    .size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
            ]
            .align_y(Alignment::Center),
            editor,
        ]
        .spacing(space::S3),
    )
    .padding(Padding::from(pad::CARD))
    .style(theme::card_style)
    .width(Length::Fill)
    .height(Length::Fill);

    // ── Footer: validity badge · state hint · status · Revert · Save ──────

    let validity: Element<Message> = if valid {
        text(t(Key::FallbackJsonValid))
            .size(size::CAPTION)
            .color(Color::from_rgb(0.10, 0.60, 0.10))
            .into()
    } else {
        text(t(Key::FallbackJsonInvalid))
            .size(size::CAPTION)
            .color(Color::from_rgb(0.85, 0.45, 0.0))
            .into()
    };

    let state_hint = text(if dirty { t(Key::FallbackUnsavedHint) } else { t(Key::FallbackSavedHint) })
        .size(size::CAPTION)
        .color(theme::muted(&app.theme()));

    // Revert: ghost, only actionable when dirty (routes through confirm).
    let revert_btn: Element<Message> = {
        let b = button(text(t(Key::BtnRevert)).size(size::CAPTION))
            .padding(Padding::from(pad::BUTTON))
            .style(iced::widget::button::text);
        if dirty { b.on_press(Message::FallbackFileRevert).into() } else { b.into() }
    };

    // Save: primary, only actionable when dirty. Label includes the filename
    // so users know exactly which file they are saving.
    let save_label = format!("Save  {}", file.name);
    let save_btn: Element<Message> = {
        let b = button(text(save_label).size(size::BODY))
            .padding(Padding::from(pad::BUTTON_PRIMARY))
            .style(iced::widget::button::primary);
        if dirty { b.on_press(Message::FallbackFileSave).into() } else { b.into() }
    };

    let footer = container(
        row![
            column![validity, state_hint].spacing(space::S1),
            Space::new().width(Length::Fill),
            widgets::field(
                t(Key::FallbackStatusLabel),
                text_input("200 OK", status)
                    .on_input(Message::FallbackFileSetStatus)
                    .size(size::CAPTION)
                    .padding(Padding::from([space::S2, space::S3]))
                    .width(Length::Fixed(110.0))
                    .into(),
            ),
            button(text(t(Key::FallbackFormatJson)).size(size::CAPTION))
                .on_press(Message::FallbackFileFormat)
                .padding(Padding::from(pad::BUTTON))
                .style(iced::widget::button::text),
            revert_btn,
            save_btn,
        ]
        .spacing(space::S3)
        .align_y(Alignment::End),
    )
    .padding(Padding::from([space::S3, 0.0]));

    // ── Layout ────────────────────────────────────────────────────────────

    container(
        column![
            header_col,
            widgets::divider(),
            content_card,
            footer,
        ]
        .spacing(space::S4)
        .padding(Padding::from([space::S4, space::S5])),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn script_viewer<'a>(app: &'a App, script: &'a apimokka_model::node::ConfigFileView) -> Element<'a, Message> {
    let name = script.path.rsplit('/').next().unwrap_or(&script.path);

    // Placeholder content — in production this would read the file from disk.
    let content = format!(
        "fn before_request(req) {{\n    // Edit this file in your preferred editor.\n    // Middleware scripts run before rule matching.\n    req\n}}\n\n// Path: {}",
        script.path
    );

    let header = column![
        row![
            text(name).size(size::SECTION).width(Length::Fill),
            container(
                text("read-only").size(size::CAPTION),
            )
            .padding(Padding::from([2.0, space::S2]))
            .style(theme::chip_style),
        ]
        .align_y(Alignment::Center),
        text(script.path.as_str()).size(size::CAPTION)
            .color(theme::muted(&app.theme())),
        text("Middleware scripts run before rule matching and can transform requests.")
            .size(size::CAPTION)
            .color(theme::muted(&app.theme())),
    ]
    .spacing(space::S2);

    let code_card = container(
        scrollable(
            text(content).size(size::MONO)
                .font(iced::Font::MONOSPACE),
        )
        .height(Length::Fill),
    )
    .padding(Padding::from(pad::CARD))
    .style(theme::card_style)
    .width(Length::Fill)
    .height(Length::Fill);

    container(
        column![
            header,
            widgets::divider(),
            code_card,
        ]
        .spacing(space::S4)
        .padding(Padding::from([space::S4, space::S5])),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Compact strip of recent trace events matching this rule.
fn trace_activity_section<'a>(
    app: &'a App,
    rule: &'a apimokka_model::snapshot::RuleView,
) -> Element<'a, Message> {
    let recent = recent_matching_events(app, rule);

    let header = row![
        text("Recent trace activity").size(size::BODY)
            .color(theme::muted(&app.theme()))
            .width(Length::Fill),
        button(text("View all in Trace →").size(size::CAPTION))
            .on_press(Message::ViewAllInTrace)
            .padding(Padding::from([space::S1, space::S2]))
            .style(iced::widget::button::text),
    ]
    .align_y(Alignment::Center);

    let body: Element<Message> = if recent.is_empty() {
        text("No recent matches for this rule.")
            .size(size::CAPTION)
            .color(theme::muted(&app.theme()))
            .into()
    } else {
        let rows: Vec<Element<Message>> = recent.iter().map(|ev| {
            let eid = ev.event_id;
            row![
                text(ev.outcome.glyph()).size(size::BODY),
                text(ev.request.method.as_str()).size(size::CAPTION),
                text(ev.request.url_path.as_str()).size(size::CAPTION)
                    .color(theme::muted(&app.theme()))
                    .width(Length::Fill),
                text(format!("{}ms", ev.duration_ms)).size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
                text(ev.time.as_str()).size(size::CAPTION)
                    .color(theme::muted(&app.theme())),
                button(text("Jump →").size(size::CAPTION))
                    .on_press(Message::JumpToTraceEvent(eid))
                    .padding(Padding::from([space::S1, space::S2]))
                    .style(iced::widget::button::text),
            ]
            .spacing(space::S3)
            .align_y(Alignment::Center)
            .into()
        }).collect();
        column(rows).spacing(space::S1).into()
    };

    container(
        column![header, body].spacing(space::S2),
    )
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
            rs.rules.iter().position(|r| r.id == rule.id)
                .map(|r_idx| (rs_idx, r_idx))
        })
    });

    let url_path = &rule.payload.url_path;

    app.trace.iter().rev()
        .filter(|ev| {
            match &ev.outcome {
                apimokka_model::TraceOutcome::Matched { rule_set_index, rule_index } => {
                    // Primary: exact index match (engine-reported).
                    if let Some((rs, r)) = rule_position {
                        return *rule_set_index == rs && *rule_index == r;
                    }
                    // Fallback: index unavailable, use URL path heuristic.
                    if url_path.is_empty() { return true; }
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
            text(hint).size(size::CAPTION).color(theme::muted(&app.theme())),
        ]
        .spacing(space::S1)
        .into()
    } else {
        widgets::label_with_hint(&app.theme(), title, hint)
    };

    container(
        column![
            heading,
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
            text(app.t(Key::UrlPathHint)).size(size::CAPTION)
                .color(theme::muted(&app.theme())),
        ]
        .spacing(space::S2)
        .into(),
    )
}

fn method_card<'a>(app: &'a App, method: &'a str) -> Element<'a, Message> {
    let methods = ["Any", "GET", "POST", "PUT", "PATCH", "DELETE"];
    let btns: Vec<Element<Message>> = methods.iter().map(|m| {
        let active = if *m == "Any" { method.is_empty() } else { method == *m };
        let msg    = if *m == "Any" { Message::RuleSetMethod(String::new()) }
                     else           { Message::RuleSetMethod(m.to_string()) };
        button(text(*m).size(size::CAPTION))
            .on_press(msg)
            .padding(Padding::from([space::S2, space::S3 + 2.0]))
            .style(if active { theme::seg_active } else { theme::seg_inactive })
            .into()
    }).collect();

    card(app.t(Key::MethodCardTitle), row(btns).spacing(space::S1).into())
}

fn headers_card<'a>(app: &'a App, p: &'a apimokka_model::RulePayload) -> Element<'a, Message> {
    let mut rows: Vec<Element<Message>> = p.headers.iter().enumerate().map(|(i, h)| {
        let show_val = !h.op.value_irrelevant();
        row![
            text_input(app.t(Key::HeaderColumnName), &h.name)
                .on_input(move |v| Message::HeaderSetName { index: i, value: v })
                .size(size::CAPTION)
                .padding(Padding::from([space::S2, space::S2]))
                .width(Length::Fixed(110.0)),
            pick_list(HeaderOp::all().to_vec(), Some(h.op), move |op| Message::HeaderSetOp { index: i, op })
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
        .spacing(space::S1).align_y(Alignment::Center).into()
    }).collect();

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
    let mut rows: Vec<Element<Message>> = p.body.iter().enumerate().map(|(i, b)| {
        let show_val = b.op != BodyOp::Exists && b.op != BodyOp::Absent;
        let jsonpath_warn: Element<Message> = if b.path.starts_with("$.") {
            text(app.t(Key::BodyJsonpathWarn)).size(size::CAPTION)
                .color(Color::from_rgb(0.85, 0.45, 0.0)).into()
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
                pick_list(BodyOp::all().to_vec(), Some(b.op), move |op| Message::BodySetOp { index: i, op })
                    .text_size(size::CAPTION)
                    .padding(Padding::from([space::S2, space::S2]))
                    .width(Length::Fixed(120.0)),
                {
                    let bval: Element<Message> = if show_val {
                        text_input("value", &b.value)
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
            .spacing(space::S1).align_y(Alignment::Center),
            jsonpath_warn,
        ]
        .spacing(space::S1)
        .into()
    }).collect();

    if p.body.is_empty() {
        rows.push(
            text(app.t(Key::BodyDottedPathHint)).size(size::CAPTION)
                .color(theme::muted(&app.theme())).into()
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

fn respond_card<'a>(app: &'a App, p: &'a apimokka_model::RulePayload) -> Element<'a, Message> {
    let is_inline = p.respond.mode == RespondMode::InlineText;
    let mode_btns: Element<Message> = row![
        mode_tab(app.t(Key::RespondModeInline), is_inline, RespondMode::InlineText),
        mode_tab(app.t(Key::RespondModeFile), !is_inline, RespondMode::ServeFile),
    ]
    .spacing(space::S1)
    .into();

    let body_el: Element<Message> = if is_inline {
        text_input("Response body…", &p.respond.text)
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

    let delay_str = p.respond.delay_milliseconds.to_string();

    card(
        app.t(Key::RespondCardTitle),
        column![
            mode_btns,
            body_el,
            row![
                widgets::field(app.t(Key::RespondStatusLabel),
                    text_input("200 OK", &p.respond.status)
                        .on_input(Message::RespondSetStatus)
                        .size(size::CAPTION)
                        .padding(Padding::from([space::S2, space::S3]))
                        .width(Length::Fixed(110.0))
                        .into(),
                ),
                Space::new().width(space::S3),
                widgets::field(app.t(Key::RespondDelayLabel),
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
            text(app.t(Key::RespondMutexHint)).size(size::CAPTION)
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
        .style(if active { theme::seg_active } else { theme::seg_inactive })
        .into()
}
