use std::fs;
use threecx_log_explorer::{parse_text, LogLevel};

#[test]
fn parses_realistic_3cx_records_and_metadata() {
    let fixture = include_str!("fixtures/callflow_sample.log");
    let events = parse_text(fixture);
    assert_eq!(events.len(), 5, "stack trace lines must remain attached to the error event");

    assert_eq!(events[0].timestamp, "2026-10-08 11:21:08.028");
    assert_eq!(events[0].thread_id, "0028");
    assert_eq!(events[0].level, LogLevel::Info);
    assert_eq!(events[0].source.as_deref(), Some("_3CX.HttpClient"));
    assert!(events[0].message.contains("502 BadGateway"));

    assert_eq!(events[1].flow_id.as_deref(), Some("_7001_7.Callflow.2"));
    assert_eq!(events[1].component.as_deref(), Some("discadorv2_tmslab"));

    let error = events.iter().find(|event| event.level == LogLevel::Error).unwrap();
    assert_eq!(error.call_id.as_deref(), Some("REDACTED-CALL-ID"));
    assert_eq!(error.component.as_deref(), Some("PS_SMT"));
    assert!(error.message.contains("Duplicate entry '0'"));
    assert!(error.message.contains("ServerSession.ReceiveReplyAsync"));
    assert_eq!(error.start_line, 5);
    assert_eq!(error.end_line, 7);
}

#[test]
fn supports_unknown_levels_and_records_without_source() {
    let events = parse_text("2026-10-08 11:21:08.028|0001|Notice| bare message\n");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, LogLevel::Other("Notice".to_string()));
    assert_eq!(events[0].source, None);
    assert_eq!(events[0].message, "bare message");
}

#[test]
fn parses_full_uploaded_callflow_log_when_path_is_supplied() {
    let Ok(path) = std::env::var("THREECX_REAL_LOG") else {
        return;
    };
    let contents = fs::read_to_string(path).expect("THREECX_REAL_LOG should point to a readable log");
    let events = parse_text(&contents);
    assert_eq!(events.len(), 2952, "record count based on the supplied 3CXCallFlow.log");
    assert_eq!(events.iter().filter(|e| e.level == LogLevel::Trace).count(), 2552);
    assert_eq!(events.iter().filter(|e| e.level == LogLevel::Debug).count(), 36);
    assert_eq!(events.iter().filter(|e| e.level == LogLevel::Info).count(), 363);
    let errors: Vec<_> = events.iter().filter(|e| e.level == LogLevel::Error).collect();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("Duplicate entry '0'"));
    assert!(events.iter().any(|e| e.message.contains("502 BadGateway")));
}
