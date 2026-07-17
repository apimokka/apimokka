use super::*;
use crate::message::Message;
use apimokka_model::{MatchTraceEvent, RequestSummary, TraceOutcome};

fn with_trace() -> App {
    let mut a = App::new().0;
    a.update(Message::ChooseAudienceMode(
        apimokka_model::AudienceMode::Expert,
    ));
    a.update(Message::OpenWorkspace("test".into()));
    a.trace = apimokka_model::mock::sample_trace_events();
    a
}

// ── Filter ──────────────────────────────────────────────────────────

#[test]
fn filter_empty_shows_all() {
    let a = with_trace();
    let q = a.trace_filter.clone();
    assert!(q.is_empty());
    // All events pass an empty filter (checked via trace count equality).
    let filtered: Vec<_> = a
        .trace
        .iter()
        .filter(|ev| {
            ev.request.url_path.to_lowercase().contains(&q)
                || ev.request.method.to_lowercase().contains(&q)
                || ev.outcome.label().contains(q.as_str())
                || q.is_empty()
        })
        .collect();
    assert_eq!(filtered.len(), a.trace.len());
}

#[test]
fn filter_by_path_narrows_list() {
    let mut a = with_trace();
    a.update(Message::TraceFilterChanged("/api/orders".into()));
    assert_eq!(a.trace_filter, "/api/orders");
    let filtered: Vec<_> = a
        .trace
        .iter()
        .filter(|ev| ev.request.url_path.contains("/api/orders"))
        .collect();
    // sample data has 2 events on /api/orders
    assert!(!filtered.is_empty());
    assert!(filtered.len() < a.trace.len());
}

// ── Jump actions ─────────────────────────────────────────────────────

#[test]
fn jump_to_rule_switches_tab_and_selects() {
    let mut a = with_trace();
    a.tab = crate::selection::WorkspaceTab::Trace;
    let rule_id = a.snapshot.as_ref().unwrap().rule_sets[0].rules[0].id;
    a.update(Message::JumpToRule(rule_id));
    assert_eq!(a.tab, crate::selection::WorkspaceTab::Routes);
    assert_eq!(a.selection.rule, Some(rule_id));
}

#[test]
fn jump_to_file_switches_tab_and_selects() {
    let mut a = with_trace();
    a.tab = crate::selection::WorkspaceTab::Trace;
    let path = a.snapshot.as_ref().unwrap().fallback_files[0].path.clone();
    a.update(Message::JumpToFile(path.clone()));
    assert_eq!(a.tab, crate::selection::WorkspaceTab::Routes);
    assert_eq!(a.selection.file_route.as_deref(), Some(path.as_str()));
}

// ── Rule editor trace strip ──────────────────────────────────────────

#[test]
fn trace_strip_matches_by_rule_index() {
    // The Matched event in sample data has rule_set_index=0, rule_index=2.
    // The strip for rule at [0][2] should find it; rule at [0][0] should not.
    let a = with_trace();
    let snap = a.snapshot.as_ref().unwrap();

    let rule_matches = |rule: &apimokka_model::snapshot::RuleView| {
        let rule_position: Option<(usize, usize)> =
            snap.rule_sets.iter().enumerate().find_map(|(rs_idx, rs)| {
                rs.rules
                    .iter()
                    .position(|r| r.id == rule.id)
                    .map(|r_idx| (rs_idx, r_idx))
            });
        a.trace.iter().any(|ev| {
            matches!(&ev.outcome,
                TraceOutcome::Matched { rule_set_index, rule_index }
                    if rule_position == Some((*rule_set_index, *rule_index))
            )
        })
    };

    let rule_2 = &snap.rule_sets[0].rules[2];
    let rule_0 = &snap.rule_sets[0].rules[0];
    assert!(
        rule_matches(rule_2),
        "rule at index 2 should have a matched trace event"
    );
    assert!(
        !rule_matches(rule_0),
        "rule at index 0 has no matched trace event"
    );
}

// ── Trace view builds with each outcome ──────────────────────────────

#[test]
fn trace_view_builds_for_each_outcome() {
    use apimokka_model::TraceOutcome;
    let outcomes = vec![
        TraceOutcome::Matched {
            rule_set_index: 0,
            rule_index: 0,
        },
        TraceOutcome::Fallback {
            file_path: "responses/health.json".into(),
            status: "200 OK".into(),
        },
        TraceOutcome::Miss {
            status: "404 Not Found".into(),
        },
        TraceOutcome::Error {
            kind: "RespondFile".into(),
            message: "permission denied".into(),
        },
    ];
    for (i, outcome) in outcomes.into_iter().enumerate() {
        let mut a = App::new().0;
        a.update(Message::ChooseAudienceMode(
            apimokka_model::AudienceMode::Expert,
        ));
        a.trace = vec![MatchTraceEvent {
            event_id: i as u64,
            time: "12:00:00.000".into(),
            duration_ms: 1,
            request: RequestSummary {
                method: "GET".into(),
                url_path: "/test".into(),
                headers: vec![],
                body_preview: None,
            },
            outcome,
            dropped_count: if i == 2 { 5 } else { 0 }, // test dropped_count warning
        }];
        a.selected_trace = Some(i as u64);
        let _ = crate::screens::trace::view(&a);
    }
}
