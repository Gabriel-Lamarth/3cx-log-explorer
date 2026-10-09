//! Self-contained interactive HTML report for local exploration of 3CX logs.
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::{build_correlation_graph, LogEvent, LogLevel, NodeKind};

/// Build an offline HTML report with a correlation graph and filterable event table.
pub fn write_html_report<P: AsRef<Path>>(events: &[LogEvent], source: &str, output: P) -> std::io::Result<()> {
    fs::write(output, render_html_report(events, source))
}

/// Render a complete HTML document without external scripts, fonts, or network access.
pub fn render_html_report(events: &[LogEvent], source: &str) -> String {
    let graph = build_correlation_graph(events);
    let mut html = String::with_capacity(events.iter().map(|e| e.message.len()).sum::<usize>() + 16_000);
    html.push_str(r#"<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>3CX Log Explorer — Relatório</title>
<style>
:root{color-scheme:dark;--bg:#0b1020;--panel:#131b2e;--line:#2b3852;--text:#e8edf7;--muted:#9aa9c2;--accent:#72a7ff;--bad:#ff7b86;--good:#63d6ad}*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--text);font:14px/1.5 system-ui,-apple-system,Segoe UI,sans-serif}header{padding:28px clamp(16px,4vw,48px);border-bottom:1px solid var(--line);background:#0e1527}h1{margin:0 0 6px;font-size:25px}h2{font-size:18px;margin:0 0 14px}p,.muted{color:var(--muted)}main{padding:24px clamp(12px,3vw,36px);display:grid;gap:20px}.stats{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:12px}.stat,section{background:var(--panel);border:1px solid var(--line);border-radius:12px;padding:16px}.stat strong{display:block;font-size:25px}.stat span{color:var(--muted)}.scroll{overflow:auto;max-width:100%}svg{min-width:760px;width:100%;height:auto;background:#0e1527;border-radius:8px}.edge{stroke:#4b5d7d;stroke-width:1.5;opacity:.7}.node{cursor:pointer;stroke:#dce7ff;stroke-width:1.2}.node:hover{stroke:#fff;stroke-width:3}.node-label{fill:var(--text);font-size:11px;pointer-events:none}.legend{display:flex;gap:14px;flex-wrap:wrap;color:var(--muted);margin:10px 0}.dot{display:inline-block;width:10px;height:10px;border-radius:50%;margin-right:5px}.controls{display:flex;gap:8px;flex-wrap:wrap;margin-bottom:12px}.controls input,.controls select{background:#0c1323;color:var(--text);border:1px solid var(--line);border-radius:8px;padding:9px;min-width:150px}button{background:#223653;color:var(--text);border:1px solid #405779;border-radius:8px;padding:9px 12px;cursor:pointer}button:hover{background:#2b456b}table{width:100%;border-collapse:collapse;font-size:12px}th,td{text-align:left;padding:9px;border-bottom:1px solid var(--line);vertical-align:top}th{position:sticky;top:0;background:#18243b}td.message{min-width:300px;white-space:pre-wrap;overflow-wrap:anywhere;max-width:700px}tr.error td:first-child,tr.fatal td:first-child{color:var(--bad);font-weight:700}.pill{display:inline-block;border:1px solid var(--line);border-radius:999px;padding:2px 7px;color:var(--muted)}.node-list{display:flex;gap:8px;flex-wrap:wrap}.node-button{text-align:left;max-width:100%;overflow-wrap:anywhere}.note{border-left:3px solid var(--accent);padding:8px 12px;background:#101a2e;color:var(--muted)}footer{color:var(--muted);padding:8px 0 24px}@media(max-width:700px){header{padding:20px 14px}main{padding:12px}.stat,section{padding:12px}}
</style></head><body><header><h1>3CX Log Explorer</h1><div class="muted">Relatório interativo local · sem dependências externas</div><p>Fonte: <strong>"#);
    html.push_str(&escape_html(source));
    html.push_str(r#"</strong></p></header><main><div class="stats">"#);
    stat(&mut html, "Eventos", &events.len().to_string());
    stat(&mut html, "Entidades", &graph.nodes.len().to_string());
    stat(&mut html, "Relações explícitas", &graph.edges.len().to_string());
    stat(&mut html, "Eventos sem IDs extraídos", &graph.events_without_identifiers.to_string());
    stat(&mut html, "Erros/fatais", &events.iter().filter(|e| matches!(&e.level, LogLevel::Error | LogLevel::Fatal)).count().to_string());
    html.push_str(r#"</div><section><h2>Grafo de correlações</h2><p class="muted">Clique em um nó para filtrar os eventos associados. Uma aresta indica coocorrência explícita no mesmo registro, não causalidade.</p><div class="legend"><span><i class="dot" style="background:#6ca8ff"></i>CallID</span><span><i class="dot" style="background:#68d7b0"></i>FlowID</span><span><i class="dot" style="background:#f6c56b"></i>Componente</span><span>Espessura/contagem representa registros em comum.</span></div><div class="scroll"><svg id="graph" viewBox="0 0 1100 600" role="img" aria-label="Grafo de correlações"><g id="edges">"#);

    let mut positions: BTreeMap<(NodeKind, String), (f64, f64)> = BTreeMap::new();
    let mut columns: BTreeMap<NodeKind, Vec<_>> = BTreeMap::new();
    for node in &graph.nodes { columns.entry(node.kind.clone()).or_default().push(node); }
    let x_for = |kind: &NodeKind| match kind { NodeKind::Call => 180.0, NodeKind::Flow => 550.0, NodeKind::Component => 920.0 };
    let max_rows = columns.values().map(Vec::len).max().unwrap_or(1).max(1);
    let svg_height = (max_rows as f64 * 58.0 + 100.0).max(400.0).min(24000.0);
    // Use a taller viewBox for large graphs, and keep each kind in its own column.
    html = html.replace("viewBox=\"0 0 1100 600\"", &format!("viewBox=\"0 0 1100 {svg_height:.0}\""));
    for (kind, nodes) in &columns {
        for (i, node) in nodes.iter().enumerate() {
            let y = 55.0 + i as f64 * ((svg_height - 100.0) / nodes.len().max(1) as f64);
            positions.insert((kind.clone(), node.id.clone()), (x_for(kind), y));
        }
    }
    for edge in &graph.edges {
        if let (Some(&(x1,y1)), Some(&(x2,y2))) = (positions.get(&(edge.key.from_kind.clone(), edge.key.from_id.clone())), positions.get(&(edge.key.to_kind.clone(), edge.key.to_id.clone()))) {
            let width = (1.0 + (edge.cooccurrences as f64).ln()).min(7.0);
            html.push_str(&format!("<line class=\"edge\" x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke-width=\"{width:.1}\"><title>{} coocorrência(s); {} coocorrência(s) com erro/fatal</title></line>", edge.cooccurrences, edge.error_cooccurrences));
        }
    }
    html.push_str("</g><g id=\"nodes\">");
    for (kind, nodes) in &columns {
        for node in nodes {
            if let Some(&(x,y)) = positions.get(&(kind.clone(), node.id.clone())) {
                let color = match kind { NodeKind::Call => "#6ca8ff", NodeKind::Flow => "#68d7b0", NodeKind::Component => "#f6c56b" };
                let radius = (7.0 + (node.event_count as f64).ln()).min(15.0);
                let label = truncate(&node.id, 32);
                html.push_str(&format!("<g class=\"graph-node\" data-kind=\"{}\" data-id=\"{}\" tabindex=\"0\" role=\"button\" aria-label=\"Filtrar {} {}\"><circle class=\"node\" cx=\"{x}\" cy=\"{y}\" r=\"{radius:.1}\" fill=\"{color}\"/><text class=\"node-label\" x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}</text><title>{}: {} · {} eventos · {} erros/fatais</title></g>", kind_label(kind), escape_html(&node.id), kind_label(kind), escape_html(&node.id), x, y + radius + 13.0, escape_html(&label), kind_label(kind), escape_html(&node.id), node.event_count, node.error_count));
            }
        }
    }
    html.push_str(r#"</g></svg></div><p class="muted">Nós: CallID, FlowID e componente. Use a lista abaixo se preferir filtrar por entidade.</p><div class="node-list">"#);
    for node in &graph.nodes {
        html.push_str(&format!("<button class=\"node-button\" data-filter-kind=\"{}\" data-filter-id=\"{}\">{}: {} <span class=\"pill\">{} eventos</span></button>", kind_label(&node.kind), escape_html(&node.id), kind_label(&node.kind), escape_html(&node.id), node.event_count));
    }
    html.push_str(r#"</div></section><section><h2>Eventos</h2><div class="controls"><input id="query" type="search" placeholder="Buscar no texto..." aria-label="Buscar no texto"><select id="level"><option value="">Todos os níveis</option><option>Trace</option><option>Debug</option><option>Info</option><option>Warning</option><option>Error</option><option>Fatal</option></select><input id="call" placeholder="Filtrar CallID"><input id="flow" placeholder="Filtrar FlowID"><input id="component" placeholder="Filtrar componente"><button id="clear">Limpar filtros</button></div><p class="muted"><span id="visible-count">"#);
    html.push_str(&events.len().to_string());
    html.push_str(r#"</span> evento(s) visível(is).</p><div class="scroll"><table><thead><tr><th>Timestamp</th><th>Nível</th><th>Linha</th><th>CallID</th><th>FlowID</th><th>Componente</th><th>Mensagem</th></tr></thead><tbody id="events">"#);
    let mut sorted: Vec<&LogEvent> = events.iter().collect();
    sorted.sort_by(|a,b| a.timestamp.cmp(&b.timestamp).then(a.start_line.cmp(&b.start_line)));
    for event in sorted {
        let level = event.level.to_string();
        let call = event.call_id.as_deref().unwrap_or("");
        let flow = event.flow_id.as_deref().unwrap_or("");
        let component = event.component.as_deref().unwrap_or("");
        html.push_str(&format!("<tr class=\"{}\" data-level=\"{}\" data-call=\"{}\" data-flow=\"{}\" data-component=\"{}\"><td>{}</td><td>{}</td><td>{}-{} </td><td>{}</td><td>{}</td><td>{}</td><td class=\"message\">{}</td></tr>", escape_html(&level.to_ascii_lowercase()), escape_html(&level), escape_html(call), escape_html(flow), escape_html(component), escape_html(&event.timestamp), escape_html(&level), event.start_line, event.end_line, escape_html(call), escape_html(flow), escape_html(component), escape_html(&event.message)));
    }
    html.push_str(r#"</tbody></table></div></section><div class="note">Relatório gerado localmente. Revise os dados antes de compartilhar: logs podem conter IDs de chamadas, telefones, URLs e informações operacionais.</div><footer>3CX Log Explorer · HTML autocontido</footer></main><script>
const rows=[...document.querySelectorAll('#events tr')];const fields={query:document.querySelector('#query'),level:document.querySelector('#level'),call:document.querySelector('#call'),flow:document.querySelector('#flow'),component:document.querySelector('#component')};
function apply(){let n=0;for(const r of rows){const ok=(!fields.query.value||r.textContent.toLowerCase().includes(fields.query.value.toLowerCase()))&&(!fields.level.value||r.dataset.level.toLowerCase()===fields.level.value.toLowerCase())&&(!fields.call.value||r.dataset.call.toLowerCase().includes(fields.call.value.toLowerCase()))&&(!fields.flow.value||r.dataset.flow.toLowerCase().includes(fields.flow.value.toLowerCase()))&&(!fields.component.value||r.dataset.component.toLowerCase().includes(fields.component.value.toLowerCase()));r.hidden=!ok;if(ok)n++}document.querySelector('#visible-count').textContent=n}
Object.values(fields).forEach(f=>f.addEventListener('input',apply));function setFilter(kind,id){const map={CallID:'call',FlowID:'flow',Component:'component'};fields[map[kind]].value=id;apply();document.querySelector('#events').closest('section').scrollIntoView({behavior:'smooth'});}
document.querySelectorAll('[data-filter-kind]').forEach(b=>b.addEventListener('click',()=>setFilter(b.dataset.filterKind,b.dataset.filterId)));document.querySelectorAll('.graph-node').forEach(n=>{n.addEventListener('click',()=>setFilter(n.dataset.kind,n.dataset.id));n.addEventListener('keydown',e=>{if(e.key==='Enter'||e.key===' '){e.preventDefault();setFilter(n.dataset.kind,n.dataset.id)}})});document.querySelector('#clear').addEventListener('click',()=>{Object.values(fields).forEach(f=>f.value='');apply()});
</script></body></html>"#);
    html
}

fn stat(html: &mut String, label: &str, value: &str) {
    html.push_str(&format!("<div class=\"stat\"><strong>{}</strong><span>{}</span></div>", escape_html(value), escape_html(label)));
}
fn kind_label(kind: &NodeKind) -> &'static str { match kind { NodeKind::Call => "CallID", NodeKind::Flow => "FlowID", NodeKind::Component => "Component" } }
fn truncate(value: &str, max_chars: usize) -> String { let mut s: String = value.chars().take(max_chars).collect(); if value.chars().count() > max_chars { s.push('…'); } s }
fn escape_html(value: &str) -> String { value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;") }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_text;

    #[test]
    fn report_contains_graph_events_and_filter_controls() {
        let events = parse_text("2026-10-08 11:21:08.000|0001|Info|CallID call-a _flow-a [C:1.2]-Router\n");
        let html = render_html_report(&events, "fixture.log");
        assert!(html.contains("Grafo de correlações"));
        assert!(html.contains("call-a"));
        assert!(html.contains("id=\"events\""));
        assert!(html.contains("setFilter"));
    }

    #[test]
    fn report_escapes_untrusted_log_content_and_source() {
        let events = parse_text("2026-10-08 11:21:08.000|0001|Info|<script>alert('x')</script>\n");
        let html = render_html_report(&events, "<file&name>.log");
        assert!(html.contains("&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt;"));
        assert!(html.contains("&lt;file&amp;name&gt;.log"));
        assert!(!html.contains("<script>alert('x')</script>"));
    }
}
