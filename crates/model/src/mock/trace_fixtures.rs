//! Canned trace events (RFC MK-053 / external design § 36.2).

use crate::trace::{MatchTraceEvent, RequestSummary, TraceOutcome};

/// Four canned trace events covering matched / fallback / miss / error
/// (external design § 36.2). Ordered newest first because the trace
/// panel auto-scrolls to the top.
pub fn sample_trace_events() -> Vec<MatchTraceEvent> {
    vec![
        MatchTraceEvent {
            event_id: 4,
            time: "12:04:18.331".into(),
            duration_ms: 6,
            request: RequestSummary {
                method: "POST".into(),
                url_path: "/api/orders".into(),
                headers: vec![("content-type".into(), "application/json".into())],
                body_preview: Some(r#"{ "action": "create" }"#.into()),
            },
            outcome: TraceOutcome::Error {
                kind: "RespondFile".into(),
                message: "responses/order-created.json: permission denied".into(),
            },
            dropped_count: 0,
        },
        MatchTraceEvent {
            event_id: 3,
            time: "12:04:16.771".into(),
            duration_ms: 2,
            request: RequestSummary {
                method: "GET".into(),
                url_path: "/api/unknown".into(),
                headers: vec![("accept".into(), "*/*".into())],
                body_preview: None,
            },
            outcome: TraceOutcome::Miss {
                status: "404 Not Found".into(),
            },
            dropped_count: 0,
        },
        MatchTraceEvent {
            event_id: 2,
            time: "12:04:14.008".into(),
            duration_ms: 1,
            request: RequestSummary {
                method: "GET".into(),
                url_path: "/health".into(),
                headers: vec![("user-agent".into(), "curl/8.5".into())],
                body_preview: None,
            },
            outcome: TraceOutcome::Fallback {
                file_path: "responses/health.json".into(),
                status: "200 OK".into(),
            },
            dropped_count: 3,
        },
        MatchTraceEvent {
            event_id: 1,
            time: "12:04:13.152".into(),
            duration_ms: 4,
            request: RequestSummary {
                method: "POST".into(),
                url_path: "/api/orders".into(),
                headers: vec![
                    ("content-type".into(), "application/json".into()),
                    ("x-trace-id".into(), "abc-123".into()),
                ],
                body_preview: Some(r#"{ "action": "create", "items": [{ "qty": 2 }] }"#.into()),
            },
            outcome: TraceOutcome::Matched {
                rule_set_index: 0,
                rule_index: 2,
            },
            dropped_count: 0,
        },
    ]
}

#[cfg(test)]
#[path = "trace_fixtures/tests.rs"]
mod tests;
