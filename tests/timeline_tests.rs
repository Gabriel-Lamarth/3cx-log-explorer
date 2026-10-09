use threecx_log_explorer::{build_timeline, parse_text, CorrelationKey};

const LOG: &str = concat!(
    "2026-10-08 11:21:09.000|0002|Error|failure CallID call-a _7001_7.Callflow.2\n",
    "2026-10-08 11:21:08.000|0001|Info|start CallID call-a _7001_7.Callflow.2\n",
    "2026-10-08 11:21:10.000|0003|Trace|other call CallID call-b _7001_7.Callflow.2\n",
    "2026-10-08 11:21:11.000|0004|Info|without ids\n",
);

#[test]
fn call_timeline_sorts_chronologically_and_keeps_only_exact_call_id() {
    let events = parse_text(LOG);
    let timeline = build_timeline(&events, CorrelationKey::CallId, "call-a");
    assert_eq!(timeline.events.len(), 2);
    assert_eq!(timeline.events[0].start_line, 2);
    assert_eq!(timeline.events[1].start_line, 1);
    assert_eq!(timeline.error_count(), 1);
    assert_eq!(timeline.first_timestamp, "2026-10-08 11:21:08.000");
    assert_eq!(timeline.last_timestamp, "2026-10-08 11:21:09.000");
}

#[test]
fn flow_timeline_groups_events_by_flow_id_independently_of_call_id() {
    let events = parse_text(LOG);
    let timeline = build_timeline(&events, CorrelationKey::FlowId, "_7001_7.Callflow.2");
    assert_eq!(timeline.events.len(), 3);
}

#[test]
fn empty_timeline_is_valid_and_has_no_timestamps() {
    let events = parse_text(LOG);
    let timeline = build_timeline(&events, CorrelationKey::CallId, "not-present");
    assert!(timeline.events.is_empty());
    assert!(timeline.first_timestamp.is_empty());
    assert_eq!(timeline.error_count(), 0);
}
