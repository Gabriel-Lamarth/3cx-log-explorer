#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    fs::File,
    io::BufReader,
    path::Path,
    sync::Mutex,
};
use tauri::State;
use threecx_log_explorer::{
    build_correlation_graph, build_timeline, inspect_archive, parse_reader, read_archive_log,
    write_html_report, CorrelationKey, LogEvent, LogLevel, RuleEngine,
};

const DEFAULT_RULES: &str = include_str!("../../../rules/default.lua");

struct AppState {
    events: Vec<LogEvent>,
    source: String,
    active_rules: String,
}
struct Shared(Mutex<AppState>);

#[derive(Serialize, Clone)]
struct EventDto {
    timestamp: String,
    thread_id: String,
    level: String,
    source: Option<String>,
    message: String,
    start_line: usize,
    end_line: usize,
    flow_id: Option<String>,
    call_id: Option<String>,
    component: Option<String>,
}
impl From<&LogEvent> for EventDto {
    fn from(event: &LogEvent) -> Self {
        Self {
            timestamp: event.timestamp.clone(),
            thread_id: event.thread_id.clone(),
            level: event.level.to_string(),
            source: event.source.clone(),
            message: event.message.clone(),
            start_line: event.start_line,
            end_line: event.end_line,
            flow_id: event.flow_id.clone(),
            call_id: event.call_id.clone(),
            component: event.component.clone(),
        }
    }
}

#[derive(Serialize)]
struct Stats {
    source: String,
    total: usize,
    errors: usize,
    warnings: usize,
    trace: usize,
    debug: usize,
    info: usize,
    calls: usize,
    flows: usize,
    components: usize,
    events_without_ids: usize,
    levels: Vec<LevelCount>,
}
#[derive(Serialize)]
struct LevelCount {
    level: String,
    count: usize,
}
#[derive(Serialize)]
struct NodeDto {
    kind: String,
    id: String,
    event_count: usize,
    error_count: usize,
    first_timestamp: String,
    last_timestamp: String,
}
#[derive(Serialize)]
struct EdgeDto {
    from_kind: String,
    from_id: String,
    to_kind: String,
    to_id: String,
    cooccurrences: usize,
    error_cooccurrences: usize,
}
#[derive(Serialize, Clone)]
struct FindingDto {
    rule_id: String,
    title: String,
    severity: String,
    description: String,
    recommendation: String,
    timestamp: String,
    start_line: usize,
    call_id: Option<String>,
    flow_id: Option<String>,
    component: Option<String>,
}
#[derive(Serialize)]
struct LoadResult {
    source: String,
    events: Vec<EventDto>,
    stats: Stats,
    nodes: Vec<NodeDto>,
    edges: Vec<EdgeDto>,
    findings: Vec<FindingDto>,
    zip_logs: Vec<String>,
}

fn dto(event: &LogEvent) -> EventDto {
    EventDto::from(event)
}

fn unique_values<F>(events: &[LogEvent], selector: F) -> usize
where
    F: for<'a> Fn(&'a LogEvent) -> Option<&'a str>,
{
    let mut values: HashSet<&str> = HashSet::new();
    for event in events {
        if let Some(value) = selector(event) {
            if !value.is_empty() {
                values.insert(value);
            }
        }
    }
    values.len()
}

fn combine_rules(extra: &str) -> String {
    // Each script is evaluated in its own Lua function scope. Extra rules extend
    // the defaults instead of silently replacing the built-in diagnostics.
    format!(
        "local defaults = (function()\n{DEFAULT_RULES}\nend)()\n\
         local extras = (function()\n{extra}\nend)()\n\
         local seen = {{}}\n\
         for _, item in ipairs(defaults) do seen[item.id] = true end\n\
         for _, item in ipairs(extras) do\n\
             if not seen[item.id] then table.insert(defaults, item); seen[item.id] = true end\n\
         end\n\
         return defaults"
    )
}

fn run_diagnostics(events: &[LogEvent], script: &str) -> Result<Vec<FindingDto>, String> {
    let engine = RuleEngine::from_script(script)
        .map_err(|error| format!("Não foi possível carregar as regras Lua: {error}"))?;
    let mut findings = Vec::new();
    for event in events {
        let event_findings = engine
            .diagnose(event)
            .map_err(|error| format!("Falha ao executar regra Lua: {error}"))?;
        findings.extend(event_findings.into_iter().map(|finding| FindingDto {
            rule_id: finding.rule_id,
            title: finding.title,
            severity: finding.severity,
            description: finding.description,
            recommendation: finding.recommendation,
            timestamp: finding.timestamp,
            start_line: finding.start_line,
            call_id: finding.call_id,
            flow_id: finding.flow_id,
            component: finding.component,
        }));
    }
    Ok(findings)
}

fn analyse(
    events: &[LogEvent],
    source: &str,
    zip_logs: Vec<String>,
    rules_script: &str,
) -> Result<LoadResult, String> {
    let graph = build_correlation_graph(events);
    let mut counts = BTreeMap::<String, usize>::new();
    for event in events {
        *counts.entry(event.level.to_string()).or_default() += 1;
    }

    let findings = run_diagnostics(events, rules_script)?;
    let stats = Stats {
        source: source.to_string(),
        total: events.len(),
        errors: events
            .iter()
            .filter(|event| matches!(&event.level, LogLevel::Error | LogLevel::Fatal))
            .count(),
        warnings: events
            .iter()
            .filter(|event| matches!(&event.level, LogLevel::Warning))
            .count(),
        trace: *counts.get("Trace").unwrap_or(&0),
        debug: *counts.get("Debug").unwrap_or(&0),
        info: *counts.get("Info").unwrap_or(&0),
        calls: unique_values(events, |event| event.call_id.as_deref()),
        flows: unique_values(events, |event| event.flow_id.as_deref()),
        components: unique_values(events, |event| event.component.as_deref()),
        events_without_ids: graph.events_without_identifiers,
        levels: counts
            .into_iter()
            .map(|(level, count)| LevelCount { level, count })
            .collect(),
    };
    let nodes = graph
        .nodes
        .into_iter()
        .map(|node| NodeDto {
            kind: node.kind.label().to_string(),
            id: node.id,
            event_count: node.event_count,
            error_count: node.error_count,
            first_timestamp: node.first_timestamp,
            last_timestamp: node.last_timestamp,
        })
        .collect();
    let edges = graph
        .edges
        .into_iter()
        .map(|edge| EdgeDto {
            from_kind: edge.key.from_kind.label().to_string(),
            from_id: edge.key.from_id,
            to_kind: edge.key.to_kind.label().to_string(),
            to_id: edge.key.to_id,
            cooccurrences: edge.cooccurrences,
            error_cooccurrences: edge.error_cooccurrences,
        })
        .collect();

    Ok(LoadResult {
        source: source.to_string(),
        events: events.iter().map(dto).collect(),
        stats,
        nodes,
        edges,
        findings,
        zip_logs,
    })
}

#[tauri::command]
fn inspect_zip(path: String) -> Result<Vec<String>, String> {
    let summary = inspect_archive(&path).map_err(|error| error.to_string())?;
    Ok(summary.logs.into_iter().map(|log| log.name).collect())
}

#[tauri::command]
fn load_log(
    path: String,
    entry: Option<String>,
    state: State<'_, Shared>,
) -> Result<LoadResult, String> {
    let mut events = Vec::new();
    let mut zip_logs = Vec::new();
    let source: String;
    let path_ref = Path::new(&path);

    if path_ref
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
    {
        let summary = inspect_archive(&path).map_err(|error| error.to_string())?;
        zip_logs = summary.logs.iter().map(|log| log.name.clone()).collect();
        let chosen = entry
            .or_else(|| {
                summary
                    .logs
                    .iter()
                    .find(|log| log.name.rsplit('/').next() == Some("3CXCallFlow.log"))
                    .or_else(|| {
                        summary
                            .logs
                            .iter()
                            .find(|log| log.name.to_ascii_lowercase().contains("callflow"))
                    })
                    .map(|log| log.name.clone())
            })
            .ok_or_else(|| "Nenhum arquivo .log encontrado no ZIP.".to_string())?;
        read_archive_log(&path, &chosen, |event| events.push(event))
            .map_err(|error| error.to_string())?;
        source = format!("{} :: {}", path, chosen);
    } else {
        let file = File::open(&path)
            .map_err(|error| format!("Não foi possível abrir o arquivo: {error}"))?;
        parse_reader(BufReader::new(file), |event| events.push(event))
            .map_err(|error| error.to_string())?;
        // `path` is already a String; to_string_lossy belongs to Path/OsStr.
        source = path.clone();
    }

    let active_rules = state
        .0
        .lock()
        .map_err(|_| "Estado interno indisponível".to_string())?
        .active_rules
        .clone();
    let result = analyse(&events, &source, zip_logs, &active_rules)?;
    let mut inner = state
        .0
        .lock()
        .map_err(|_| "Estado interno indisponível".to_string())?;
    inner.events = events;
    inner.source = source;
    Ok(result)
}

#[tauri::command]
fn load_extra_rules(path: String, state: State<'_, Shared>) -> Result<Vec<FindingDto>, String> {
    let extra = fs::read_to_string(&path)
        .map_err(|error| format!("Não foi possível ler o arquivo de regras: {error}"))?;
    let combined = combine_rules(&extra);

    // Validate before replacing active rules. If the file is invalid, the
    // previous rules remain active and the user gets a clear error.
    let mut inner = state
        .0
        .lock()
        .map_err(|_| "Estado interno indisponível".to_string())?;
    let findings = run_diagnostics(&inner.events, &combined)?;
    inner.active_rules = combined;
    Ok(findings)
}


#[tauri::command]
fn preview_rules(script: String, state: State<'_, Shared>) -> Result<Vec<FindingDto>, String> {
    let inner = state.0.lock().map_err(|_| "Estado interno indisponível".to_string())?;
    if inner.events.is_empty() { return Err("Abra um log antes de testar uma regra.".into()); }
    let combined = combine_rules(&script);
    run_diagnostics(&inner.events, &combined)
}

#[tauri::command]
fn save_rule_script(path: String, content: String) -> Result<String, String> {
    if !path.to_ascii_lowercase().ends_with(".lua") {
        return Err("O arquivo da regra deve usar a extensão .lua".into());
    }
    if content.len() > 1024 * 1024 { return Err("O script excede o limite de 1 MiB.".into()); }
    // Validate the script and its rule metadata before writing it to disk.
    RuleEngine::from_script(&content)
        .map_err(|error| format!("Script Lua inválido: {error}"))?;
    fs::write(&path, content).map_err(|error| format!("Não foi possível salvar a regra: {error}"))?;
    Ok(path)
}

#[tauri::command]
fn get_event_raw(line: usize, state: State<'_, Shared>) -> Result<String, String> {
    let inner = state
        .0
        .lock()
        .map_err(|_| "Estado interno indisponível".to_string())?;
    inner
        .events
        .iter()
        .find(|event| event.start_line == line)
        .map(|event| event.raw.clone())
        .ok_or_else(|| format!("Não foi encontrado evento iniciando na linha {line}."))
}

#[tauri::command]
fn get_timeline(
    kind: String,
    id: String,
    state: State<'_, Shared>,
) -> Result<Vec<EventDto>, String> {
    let inner = state
        .0
        .lock()
        .map_err(|_| "Estado interno indisponível".to_string())?;
    let key = match kind.as_str() {
        "CallID" => CorrelationKey::CallId,
        "FlowID" => CorrelationKey::FlowId,
        _ => return Err("Tipo de correlação inválido".into()),
    };
    Ok(build_timeline(&inner.events, key, &id)
        .events
        .iter()
        .map(dto)
        .collect())
}

#[tauri::command]
fn export_report(output: String, state: State<'_, Shared>) -> Result<String, String> {
    let inner = state
        .0
        .lock()
        .map_err(|_| "Estado interno indisponível".to_string())?;
    if inner.events.is_empty() {
        return Err("Abra um log antes de exportar.".into());
    }
    write_html_report(&inner.events, &inner.source, &output)
        .map_err(|error| error.to_string())?;
    Ok(output)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Shared(Mutex::new(AppState {
            events: Vec::new(),
            source: String::new(),
            active_rules: DEFAULT_RULES.to_string(),
        })))
        .invoke_handler(tauri::generate_handler![
            inspect_zip,
            load_log,
            load_extra_rules,
            preview_rules,
            save_rule_script,
            get_event_raw,
            get_timeline,
            export_report
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar 3CX Log Explorer");
}
