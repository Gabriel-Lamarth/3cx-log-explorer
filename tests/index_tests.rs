use std::fs;
use threecx_log_explorer::{EventIndex, SearchFilters};

const LOG: &str = concat!(
    "2026-10-08 11:21:08.028|0028|Info|HTTP response Received '502 BadGateway' CallID call-a\n",
    "2026-10-08 11:21:09.028|0028|Error|Duplicate entry '0' for key 'PS_SMT.PRIMARY' CallID call-a [C:1.2]-database - insert\n",
    "2026-10-08 11:21:10.028|0029|Trace|Normal event CallID call-b\n",
);

#[test]
fn indexes_events_and_filters_by_call_id_and_level() {
    let dir = tempfile::tempdir().unwrap();
    let log_path = dir.path().join("callflow.log");
    let db_path = dir.path().join("index.sqlite");
    fs::write(&log_path, LOG).unwrap();

    let mut index = EventIndex::open(&db_path).unwrap();
    assert_eq!(index.index_file(&log_path).unwrap(), 3);
    let (sources, total, levels) = index.stats().unwrap();
    assert_eq!((sources, total), (1, 3));
    assert!(levels.contains(&("Error".to_string(), 1)));

    let results = index.search(&SearchFilters {
        call_id: Some("call-a".to_string()),
        level: Some("Error".to_string()),
        ..SearchFilters::default()
    }).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].message.contains("Duplicate entry"));
    assert_eq!(results[0].component.as_deref(), Some("database"));
}

#[test]
fn reindexing_same_source_replaces_old_events_instead_of_duplicating() {
    let dir = tempfile::tempdir().unwrap();
    let log_path = dir.path().join("callflow.log");
    let db_path = dir.path().join("index.sqlite");
    fs::write(&log_path, LOG).unwrap();

    let mut index = EventIndex::open(&db_path).unwrap();
    index.index_file(&log_path).unwrap();
    fs::write(&log_path, "2026-10-08 11:21:08.028|0028|Info|updated\n").unwrap();
    assert_eq!(index.index_file(&log_path).unwrap(), 1);
    assert_eq!(index.stats().unwrap().1, 1);
}

#[test]
fn supports_text_search_and_pagination() {
    let dir = tempfile::tempdir().unwrap();
    let log_path = dir.path().join("callflow.log");
    let db_path = dir.path().join("index.sqlite");
    fs::write(&log_path, LOG).unwrap();
    let mut index = EventIndex::open(&db_path).unwrap();
    index.index_file(&log_path).unwrap();

    let results = index.search(&SearchFilters {
        text: Some("CallID".to_string()),
        limit: 1,
        offset: 1,
        ..SearchFilters::default()
    }).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].start_line, 2);
}
