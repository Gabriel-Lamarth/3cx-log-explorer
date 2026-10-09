//! Optional integration check against the full SupportInfo ZIP supplied by the user.
//! Run with: THREECX_SUPPORT_ZIP=/path/SupportInfo.zip cargo test --test real_log_tests
use threecx_log_explorer::{inspect_archive, read_archive_log, LogLevel};

#[test]
fn finds_callflow_log_in_uploaded_supportinfo_archive() {
    let Ok(path) = std::env::var("THREECX_SUPPORT_ZIP") else {
        eprintln!("Skipping real archive check: set THREECX_SUPPORT_ZIP to the supplied archive");
        return;
    };
    let summary = inspect_archive(&path).expect("SupportInfo ZIP should be readable");
    let entry = summary.logs.iter()
        .find(|log| log.name == "Logs/3CXCallFlow.log")
        .expect("expected Logs/3CXCallFlow.log in the supplied SupportInfo archive");
    assert!(entry.uncompressed_bytes > 0);

    let mut events = 0usize;
    let mut explicit_errors = 0usize;
    read_archive_log(path, &entry.name, |event| {
        events += 1;
        if event.level == LogLevel::Error { explicit_errors += 1; }
    }).expect("CallFlow log should parse from the archive");
    assert!(events > 0);
    assert_eq!(explicit_errors, 1, "baseline count for the supplied archive's CallFlow log");
}
