//! Persistent SQLite index for fast local searching of parsed 3CX events.
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection};

use crate::archive::read_archive_log;
use crate::model::LogEvent;
use crate::parser::parse_reader;

#[derive(Clone, Debug, Default)]
pub struct SearchFilters {
    pub call_id: Option<String>,
    pub flow_id: Option<String>,
    pub component: Option<String>,
    pub level: Option<String>,
    pub text: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub source: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Clone, Debug)]
pub struct IndexedEvent {
    pub timestamp: String,
    pub thread_id: String,
    pub level: String,
    pub source_name: Option<String>,
    pub message: String,
    pub start_line: usize,
    pub end_line: usize,
    pub flow_id: Option<String>,
    pub call_id: Option<String>,
    pub component: Option<String>,
    pub indexed_source: String,
}

pub struct EventIndex {
    connection: Connection,
}

impl EventIndex {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(|e| e.to_string())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS sources (
                 id INTEGER PRIMARY KEY,
                 source TEXT NOT NULL UNIQUE,
                 indexed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 event_count INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE IF NOT EXISTS events (
                 id INTEGER PRIMARY KEY,
                 source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
                 timestamp TEXT NOT NULL,
                 thread_id TEXT NOT NULL,
                 level TEXT NOT NULL,
                 source_name TEXT,
                 message TEXT NOT NULL,
                 raw TEXT NOT NULL,
                 start_line INTEGER NOT NULL,
                 end_line INTEGER NOT NULL,
                 flow_id TEXT,
                 call_id TEXT,
                 component TEXT
             );
             CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
             CREATE INDEX IF NOT EXISTS idx_events_level ON events(level);
             CREATE INDEX IF NOT EXISTS idx_events_call_id ON events(call_id);
             CREATE INDEX IF NOT EXISTS idx_events_flow_id ON events(flow_id);
             CREATE INDEX IF NOT EXISTS idx_events_component ON events(component);
             CREATE INDEX IF NOT EXISTS idx_events_source_id ON events(source_id);
             CREATE VIRTUAL TABLE IF NOT EXISTS event_fts USING fts5(message, content='events', content_rowid='id');"
        ).map_err(|e| e.to_string())?;
        Ok(Self { connection })
    }

    pub fn index_file(&mut self, path: impl AsRef<Path>) -> Result<usize, String> {
        self.index_file_with_policy(path, false)
    }

    /// Index a log while optionally skipping routine low-value Trace/Debug events.
    /// Original files are never modified; events with identifiers or error-like content are retained.
    pub fn index_file_with_policy(&mut self, path: impl AsRef<Path>, compact: bool) -> Result<usize, String> {
        let path = path.as_ref();
        let source = path.to_string_lossy().into_owned();
        let file = File::open(path).map_err(|e| format!("não foi possível abrir '{}': {e}", path.display()))?;
        self.index_stream(&source, compact, |emit| {
            parse_reader(BufReader::new(file), |event| emit(event)).map_err(|e| e.to_string())
        })
    }

    pub fn index_archive_log(
        &mut self,
        archive_path: impl AsRef<Path>,
        entry_name: &str,
    ) -> Result<usize, String> {
        let archive_path = archive_path.as_ref();
        let source = format!("{}::{}", archive_path.display(), entry_name);
        self.index_stream(&source, false, |emit| {
            read_archive_log(archive_path, entry_name, |event| emit(event))
                .map_err(|e| e.to_string())
        })
    }

    pub fn index_archive_log_with_policy(&mut self, archive_path: impl AsRef<Path>, entry_name: &str, compact: bool) -> Result<usize, String> {
        let archive_path = archive_path.as_ref();
        let source = format!("{}::{}", archive_path.display(), entry_name);
        self.index_stream(&source, compact, |emit| {
            read_archive_log(archive_path, entry_name, |event| emit(event)).map_err(|e| e.to_string())
        })
    }

    fn index_stream<F>(&mut self, source: &str, compact: bool, parser: F) -> Result<usize, String>
    where
        F: FnOnce(&mut dyn FnMut(LogEvent)) -> Result<(), String>,
    {
        let transaction = self.connection.transaction().map_err(|e| e.to_string())?;
        transaction.execute(
            "INSERT INTO sources(source, indexed_at, event_count) VALUES (?1, CURRENT_TIMESTAMP, 0)
             ON CONFLICT(source) DO UPDATE SET indexed_at = CURRENT_TIMESTAMP, event_count = 0",
            params![source],
        ).map_err(|e| e.to_string())?;
        let source_id: i64 = transaction.query_row(
            "SELECT id FROM sources WHERE source = ?1", params![source], |row| row.get(0)
        ).map_err(|e| e.to_string())?;
        transaction.execute("DELETE FROM event_fts WHERE rowid IN (SELECT id FROM events WHERE source_id = ?1)", params![source_id]).map_err(|e| e.to_string())?;
        transaction.execute("DELETE FROM events WHERE source_id = ?1", params![source_id])
            .map_err(|e| e.to_string())?;

        let mut count = 0usize;
        let mut insertion_error: Option<String> = None;
        {
            let mut statement = transaction.prepare(
                "INSERT INTO events (
                    source_id, timestamp, thread_id, level, source_name, message, raw,
                    start_line, end_line, flow_id, call_id, component
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"
            ).map_err(|e| e.to_string())?;
            let mut fts_statement = transaction.prepare("INSERT INTO event_fts(rowid, message) VALUES (?1, ?2)").map_err(|e| e.to_string())?;
            let mut insert_event = |event: LogEvent| {
                if insertion_error.is_some() || (compact && !keep_compact_event(&event)) { return; }
                let result = statement.execute(params![
                    source_id,
                    event.timestamp,
                    event.thread_id,
                    event.level.as_str(),
                    event.source,
                    event.message,
                    "", // Original bytes remain in the source file; avoid storing the same text twice.
                    event.start_line as i64,
                    event.end_line as i64,
                    event.flow_id,
                    event.call_id,
                    event.component,
                ]);
                match result {
                    Ok(_) => {
                        let id = transaction.last_insert_rowid();
                        if let Err(error) = fts_statement.execute(params![id, event.message]) {
                            insertion_error = Some(error.to_string());
                        } else { count += 1; }
                    },
                    Err(error) => insertion_error = Some(error.to_string()),
                }
            };
            parser(&mut insert_event)?;
        }
        if let Some(error) = insertion_error {
            return Err(format!("falha ao gravar evento no SQLite: {error}"));
        }
        transaction.execute("UPDATE sources SET event_count = ?1 WHERE id = ?2", params![count as i64, source_id])
            .map_err(|e| e.to_string())?;
        transaction.commit().map_err(|e| e.to_string())?;
        Ok(count)
    }

    pub fn search(&self, filters: &SearchFilters) -> Result<Vec<IndexedEvent>, String> {
        let mut sql = String::from(
            "SELECT e.timestamp, e.thread_id, e.level, e.source_name, e.message,
                    e.start_line, e.end_line, e.flow_id, e.call_id, e.component, s.source
             FROM events e JOIN sources s ON s.id = e.source_id WHERE 1=1"
        );
        let mut values: Vec<Value> = Vec::new();
        push_filter(&mut sql, &mut values, "e.call_id", filters.call_id.as_deref());
        push_filter(&mut sql, &mut values, "e.flow_id", filters.flow_id.as_deref());
        push_filter(&mut sql, &mut values, "e.component", filters.component.as_deref());
        push_filter(&mut sql, &mut values, "e.level", filters.level.as_deref());
        push_filter(&mut sql, &mut values, "s.source", filters.source.as_deref());
        if let Some(text) = filters.text.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            let query = text.split_whitespace().map(|word| format!("\"{}\"", word.replace('\"', ""))).collect::<Vec<_>>().join(" AND ");
            sql.push_str(" AND e.id IN (SELECT rowid FROM event_fts WHERE message MATCH ?)");
            values.push(Value::Text(query));
        }
        if let Some(from) = filters.from.as_deref() {
            sql.push_str(" AND e.timestamp >= ?");
            values.push(Value::Text(from.to_string()));
        }
        if let Some(to) = filters.to.as_deref() {
            sql.push_str(" AND e.timestamp <= ?");
            values.push(Value::Text(to.to_string()));
        }
        let limit = if filters.limit == 0 { 100 } else { filters.limit.min(10_000) };
        sql.push_str(" ORDER BY e.timestamp ASC, e.id ASC LIMIT ? OFFSET ?");
        values.push(Value::Integer(limit as i64));
        values.push(Value::Integer(filters.offset as i64));

        let mut statement = self.connection.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = statement.query_map(params_from_iter(values.iter()), |row| {
            Ok(IndexedEvent {
                timestamp: row.get(0)?,
                thread_id: row.get(1)?,
                level: row.get(2)?,
                source_name: row.get(3)?,
                message: row.get(4)?,
                start_line: row.get::<_, i64>(5)? as usize,
                end_line: row.get::<_, i64>(6)? as usize,
                flow_id: row.get(7)?,
                call_id: row.get(8)?,
                component: row.get(9)?,
                indexed_source: row.get(10)?,
            })
        }).map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    pub fn stats(&self) -> Result<(usize, usize, Vec<(String, usize)>), String> {
        let sources: i64 = self.connection.query_row("SELECT COUNT(*) FROM sources", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let events: i64 = self.connection.query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let mut statement = self.connection.prepare(
            "SELECT level, COUNT(*) FROM events GROUP BY level ORDER BY level"
        ).map_err(|e| e.to_string())?;
        let rows = statement.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize)))
            .map_err(|e| e.to_string())?;
        let levels = rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
        Ok((sources as usize, events as usize, levels))
    }
}

fn push_filter(sql: &mut String, values: &mut Vec<Value>, column: &str, value: Option<&str>) {
    if let Some(value) = value {
        sql.push_str(" AND ");
        sql.push_str(column);
        sql.push_str(" = ?");
        values.push(Value::Text(value.to_string()));
    }
}

/// Keep severe, call-related, and diagnostic-looking events when compact indexing is enabled.
fn keep_compact_event(event: &LogEvent) -> bool {
    let level = event.level.as_str().to_ascii_lowercase();
    if matches!(level.as_str(), "error" | "fatal" | "critical" | "warn" | "warning") { return true; }
    let text = format!("{} {}", event.message, event.raw).to_ascii_lowercase();
    let important = ["exception", "failed", "failure", "error", "timeout", "timed out", "unauthorized", "forbidden", "rejected", "disconnect", "terminated", "cancel", "unreachable", "refused", "not found", "503", "500", "408", "486", "401", "407", "authentication", "denied", "crash", "panic", "invite", "bye", "ack", "connectionstatus", "connected", "disconnected", "call terminated", "call has been removed", "switching executing mode", "oncallterminated", "transfer", "route to", "route", "session", "sip/2.0"];
    if important.iter().any(|needle| text.contains(needle)) { return true; }
    if matches!(level.as_str(), "trace" | "debug") { return false; }
    event.call_id.is_some() || event.flow_id.is_some() || !matches!(level.as_str(), "trace" | "debug")
}
