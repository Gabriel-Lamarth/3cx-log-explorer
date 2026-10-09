use std::fs::File;
use std::io::{self, BufReader};
use std::path::{Component, Path};
use zip::ZipArchive;

use crate::model::LogEvent;
use crate::parser::parse_reader;

const MAX_ENTRY_UNCOMPRESSED_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_TOTAL_UNCOMPRESSED_BYTES: u64 = 8 * 1024 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveLog {
    pub name: String,
    pub uncompressed_bytes: u64,
    pub compressed_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveSummary {
    pub entry_count: usize,
    pub total_uncompressed_bytes: u64,
    pub logs: Vec<ArchiveLog>,
}

#[derive(Debug)]
pub enum ArchiveError {
    Io(io::Error),
    Zip(zip::result::ZipError),
    UnsafeArchive(String),
    EntryNotFound(String),
}

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Zip(error) => write!(f, "ZIP error: {error}"),
            Self::UnsafeArchive(message) => write!(f, "Archive rejected: {message}"),
            Self::EntryNotFound(name) => write!(f, "Log entry not found in archive: {name}"),
        }
    }
}

impl std::error::Error for ArchiveError {}

impl From<io::Error> for ArchiveError {
    fn from(value: io::Error) -> Self { Self::Io(value) }
}

impl From<zip::result::ZipError> for ArchiveError {
    fn from(value: zip::result::ZipError) -> Self { Self::Zip(value) }
}

/// Inspect ZIP contents without extracting any files to disk.
pub fn inspect_archive(path: impl AsRef<Path>) -> Result<ArchiveSummary, ArchiveError> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut logs = Vec::new();
    let mut total_uncompressed_bytes = 0u64;
    let entry_count = archive.len();

    for index in 0..entry_count {
        let entry = archive.by_index(index)?;
        let name = entry.name().to_string();
        let size = entry.size();

        if !is_safe_archive_path(&name) {
            return Err(ArchiveError::UnsafeArchive(name));
        }
        if size > MAX_ENTRY_UNCOMPRESSED_BYTES {
            return Err(ArchiveError::UnsafeArchive(format!(
                "entry exceeds 1 GiB uncompressed limit: {name}"
            )));
        }
        total_uncompressed_bytes = total_uncompressed_bytes.saturating_add(size);
        if total_uncompressed_bytes > MAX_TOTAL_UNCOMPRESSED_BYTES {
            return Err(ArchiveError::UnsafeArchive(
                "archive exceeds 8 GiB total uncompressed limit".to_string(),
            ));
        }

        if !entry.is_dir() && name.to_ascii_lowercase().ends_with(".log") {
            logs.push(ArchiveLog {
                name,
                uncompressed_bytes: size,
                compressed_bytes: entry.compressed_size(),
            });
        }
    }

    logs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(ArchiveSummary { entry_count, total_uncompressed_bytes, logs })
}

/// Stream a selected log entry directly from ZIP; no extraction is performed.
pub fn read_archive_log<F>(
    path: impl AsRef<Path>,
    entry_name: &str,
    on_event: F,
) -> Result<(), ArchiveError>
where
    F: FnMut(LogEvent),
{
    if !is_safe_archive_path(entry_name) {
        return Err(ArchiveError::UnsafeArchive(entry_name.to_string()));
    }
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let entry = archive.by_name(entry_name)
        .map_err(|_| ArchiveError::EntryNotFound(entry_name.to_string()))?;
    if entry.size() > MAX_ENTRY_UNCOMPRESSED_BYTES {
        return Err(ArchiveError::UnsafeArchive(format!(
            "entry exceeds 1 GiB uncompressed limit: {entry_name}"
        )));
    }
    parse_reader(BufReader::new(entry), on_event).map_err(ArchiveError::Io)
}

fn is_safe_archive_path(name: &str) -> bool {
    if name.is_empty() || name.starts_with('/') || name.starts_with('\\') || name.contains('\\') {
        return false;
    }
    Path::new(name).components().all(|component| {
        matches!(component, Component::Normal(_) | Component::CurDir)
    })
}
