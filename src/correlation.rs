//! Evidence-based relationships observed together in log events.
use std::collections::HashMap;

use crate::model::{LogEvent, LogLevel};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeKind {
    Call,
    Flow,
    Component,
}

impl NodeKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Call => "CallID",
            Self::Flow => "FlowID",
            Self::Component => "Component",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrelationNode {
    pub kind: NodeKind,
    pub id: String,
    pub event_count: usize,
    pub error_count: usize,
    pub first_timestamp: String,
    pub last_timestamp: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeKey {
    pub from_kind: NodeKind,
    pub from_id: String,
    pub to_kind: NodeKind,
    pub to_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrelationEdge {
    pub key: EdgeKey,
    /// Number of log events in which both endpoints were explicitly present.
    pub cooccurrences: usize,
    pub error_cooccurrences: usize,
    pub first_timestamp: String,
    pub last_timestamp: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CorrelationGraph {
    pub nodes: Vec<CorrelationNode>,
    pub edges: Vec<CorrelationEdge>,
    pub events_without_identifiers: usize,
}

#[derive(Default)]
struct NodeAccum { count: usize, errors: usize, first: String, last: String }
#[derive(Default)]
struct EdgeAccum { count: usize, errors: usize, first: String, last: String }

/// Build a graph from explicit co-occurrence within individual log events.
/// This deliberately does not infer relationships from nearby timestamps alone.
pub fn build_correlation_graph(events: &[LogEvent]) -> CorrelationGraph {
    // Hash maps avoid logarithmic tree lookups on large support logs. Results are
    // sorted before returning so the UI, reports, and tests remain deterministic.
    let mut nodes: HashMap<(NodeKind, String), NodeAccum> = HashMap::new();
    let mut edges: HashMap<EdgeKey, EdgeAccum> = HashMap::new();
    let mut events_without_identifiers = 0;

    for event in events {
        let mut present = Vec::with_capacity(3);
        if let Some(id) = nonempty(event.call_id.as_deref()) {
            present.push((NodeKind::Call, id.to_string()));
        }
        if let Some(id) = nonempty(event.flow_id.as_deref()) {
            present.push((NodeKind::Flow, id.to_string()));
        }
        if let Some(id) = nonempty(event.component.as_deref()) {
            present.push((NodeKind::Component, id.to_string()));
        }
        present.sort_unstable();
        present.dedup();

        if present.is_empty() {
            events_without_identifiers += 1;
        }
        let is_error = matches!(&event.level, LogLevel::Error | LogLevel::Fatal);
        for key in &present {
            let acc = nodes.entry(key.clone()).or_default();
            update_range(&mut acc.first, &mut acc.last, &event.timestamp);
            acc.count += 1;
            if is_error {
                acc.errors += 1;
            }
        }
        for i in 0..present.len() {
            for j in (i + 1)..present.len() {
                let key = EdgeKey {
                    from_kind: present[i].0.clone(),
                    from_id: present[i].1.clone(),
                    to_kind: present[j].0.clone(),
                    to_id: present[j].1.clone(),
                };
                let acc = edges.entry(key).or_default();
                update_range(&mut acc.first, &mut acc.last, &event.timestamp);
                acc.count += 1;
                if is_error {
                    acc.errors += 1;
                }
            }
        }
    }

    let mut nodes: Vec<_> = nodes
        .into_iter()
        .map(|((kind, id), acc)| CorrelationNode {
            kind,
            id,
            event_count: acc.count,
            error_count: acc.errors,
            first_timestamp: acc.first,
            last_timestamp: acc.last,
        })
        .collect();
    nodes.sort_by(|a, b| (&a.kind, &a.id).cmp(&(&b.kind, &b.id)));

    let mut edges: Vec<_> = edges
        .into_iter()
        .map(|(key, acc)| CorrelationEdge {
            key,
            cooccurrences: acc.count,
            error_cooccurrences: acc.errors,
            first_timestamp: acc.first,
            last_timestamp: acc.last,
        })
        .collect();
    edges.sort_by(|a, b| a.key.cmp(&b.key));

    CorrelationGraph {
        nodes,
        edges,
        events_without_identifiers,
    }
}

fn nonempty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn update_range(first: &mut String, last: &mut String, timestamp: &str) {
    if first.is_empty() || timestamp < first.as_str() { *first = timestamp.to_string(); }
    if last.is_empty() || timestamp > last.as_str() { *last = timestamp.to_string(); }
}
