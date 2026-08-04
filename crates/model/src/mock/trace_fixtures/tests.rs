use super::*;

#[test]
fn sample_trace_has_distinct_event_ids() {
    let events = sample_trace_events();
    assert!(!events.is_empty());
    let mut ids: Vec<u64> = events.iter().map(|e| e.event_id).collect();
    let before = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(before, ids.len(), "trace event ids must be unique");
}
