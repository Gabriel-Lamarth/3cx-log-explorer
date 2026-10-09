use threecx_log_explorer::{build_correlation_graph, parse_text, NodeKind};

const LOG: &str = concat!(
    "2026-10-08 11:21:08.000|0001|Info|start CallID call-a _7001_7.Callflow.2 [C:1.2]-Router\n",
    "2026-10-08 11:21:09.000|0002|Error|failure CallID call-a _7001_7.Callflow.2 [C:1.2]-Router\n",
    "2026-10-08 11:21:10.000|0003|Info|another call CallID call-b _7001_7.Callflow.2 [C:2.1]-SIP\n",
    "2026-10-08 11:21:11.000|0004|Trace|no IDs here\n",
);

#[test]
fn graph_aggregates_explicit_nodes_and_cooccurrences() {
    let events = parse_text(LOG);
    let graph = build_correlation_graph(&events);
    let call_a = graph.nodes.iter().find(|n| n.kind == NodeKind::Call && n.id == "call-a").unwrap();
    assert_eq!(call_a.event_count, 2);
    assert_eq!(call_a.error_count, 1);
    let call_flow = graph.edges.iter().find(|e| e.key.from_kind == NodeKind::Call && e.key.from_id == "call-a" && e.key.to_kind == NodeKind::Flow).unwrap();
    assert_eq!(call_flow.cooccurrences, 2);
    assert_eq!(call_flow.error_cooccurrences, 1);
    assert_eq!(graph.events_without_identifiers, 1);
}

#[test]
fn graph_does_not_infer_edges_from_nearby_events() {
    let events = parse_text("2026-10-08 11:21:08.000|0001|Info|CallID call-a\n2026-10-08 11:21:09.000|0002|Info|[C:1.2]-Router\n");
    let graph = build_correlation_graph(&events);
    assert!(graph.edges.is_empty());
}

#[test]
fn graph_orders_nodes_and_edges_deterministically() {
    let graph = build_correlation_graph(&parse_text(LOG));
    let mut nodes = graph.nodes.clone();
    nodes.sort_by(|a,b| (&a.kind, &a.id).cmp(&(&b.kind, &b.id)));
    assert_eq!(graph.nodes, nodes);
    let mut edges = graph.edges.clone();
    edges.sort_by(|a,b| a.key.cmp(&b.key));
    assert_eq!(graph.edges, edges);
}
