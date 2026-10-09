//! Local-first parsing primitives for 3CX support logs.
//!
//! Phase 1 intentionally avoids extracting SupportInfo archives to disk. It can
//! inspect archive metadata and stream a selected log entry directly from ZIP.

pub mod archive;
pub mod diagnostics;
pub mod correlation;
pub mod index;
pub mod model;
pub mod parser;
pub mod report;
pub mod timeline;

pub use archive::{inspect_archive, read_archive_log, ArchiveLog, ArchiveSummary};
pub use diagnostics::{Finding, RuleEngine};
pub use correlation::{build_correlation_graph, CorrelationEdge, CorrelationGraph, CorrelationNode, EdgeKey, NodeKind};
pub use index::{EventIndex, IndexedEvent, SearchFilters};
pub use model::{LogEvent, LogLevel};
pub use parser::{parse_reader, parse_text};
pub use timeline::{build_timeline, CorrelationKey, Timeline};

pub use report::{render_html_report, write_html_report};
