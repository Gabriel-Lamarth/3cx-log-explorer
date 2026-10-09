use std::io::Write;
use tempfile::tempdir;
use threecx_log_explorer::{inspect_archive, read_archive_log};
use zip::write::FileOptions;
use zip::ZipWriter;

#[test]
fn lists_logs_and_streams_them_without_extraction() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("supportinfo.zip");
    {
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        zip.start_file("Logs/3CXCallFlow.log", FileOptions::default()).unwrap();
        zip.write_all(b"2026-10-08 11:21:08.028|0028| Info| [_3CX.HttpClient] Received '200 OK'\n").unwrap();
        zip.start_file("DbTables/example.csv", FileOptions::default()).unwrap();
        zip.write_all(b"not a log\n").unwrap();
        zip.finish().unwrap();
    }

    let summary = inspect_archive(&zip_path).unwrap();
    assert_eq!(summary.entry_count, 2);
    assert_eq!(summary.logs.len(), 1);
    assert_eq!(summary.logs[0].name, "Logs/3CXCallFlow.log");

    let mut messages = Vec::new();
    read_archive_log(&zip_path, "Logs/3CXCallFlow.log", |event| messages.push(event.message)).unwrap();
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("200 OK"));
}

#[test]
fn rejects_path_traversal_requests() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("supportinfo.zip");
    {
        let file = std::fs::File::create(&zip_path).unwrap();
        ZipWriter::new(file).finish().unwrap();
    }
    let result = read_archive_log(&zip_path, "../outside.log", |_| {});
    assert!(result.is_err());
}
