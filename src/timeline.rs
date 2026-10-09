//! Chronological correlation of events belonging to one CallID or FlowID.
use std::collections::BTreeMap;

use crate::model::{LogEvent, LogLevel};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorrelationKey {
    CallId,
    FlowId,
}

impl CorrelationKey {
    pub fn label(self) -> &'static str {
        match self {
            Self::CallId => "CallID",
            Self::FlowId => "FlowID",
        }
    }

    fn value<'a>(self, event: &'a LogEvent) -> Option<&'a str> {
        match self {
            Self::CallId => event.call_id.as_deref(),
            Self::FlowId => event.flow_id.as_deref(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Timeline {
    pub key: CorrelationKey,
    pub id: String,
    pub events: Vec<LogEvent>,
    pub level_counts: BTreeMap<String, usize>,
    pub components: BTreeMap<String, usize>,
    pub first_timestamp: String,
    pub last_timestamp: String,
}

/// Select only exact identifier matches and sort them by timestamp, then source line.
/// Events without the requested identifier are intentionally not correlated.
pub fn build_timeline(events: &[LogEvent], key: CorrelationKey, id: &str) -> Timeline {
    let mut matched: Vec<LogEvent> = events.iter()
        .filter(|event| key.value(event) == Some(id))
        .cloned()
        .collect();
    matched.sort_by(|a, b| a.timestamp.cmp(&b.timestamp).then(a.start_line.cmp(&b.start_line)));

    let mut level_counts = BTreeMap::new();
    let mut components = BTreeMap::new();
    for event in &matched {
        *level_counts.entry(event.level.to_string()).or_insert(0) += 1;
        if let Some(component) = &event.component {
            *components.entry(component.clone()).or_insert(0) += 1;
        }
    }
    let first_timestamp = matched.first().map(|event| event.timestamp.clone()).unwrap_or_default();
    let last_timestamp = matched.last().map(|event| event.timestamp.clone()).unwrap_or_default();

    Timeline { key, id: id.to_string(), events: matched, level_counts, components, first_timestamp, last_timestamp }
}

impl Timeline {
    pub fn error_count(&self) -> usize {
        self.events.iter().filter(|event| matches!(&event.level, LogLevel::Error | LogLevel::Fatal)).count()
    }
}
