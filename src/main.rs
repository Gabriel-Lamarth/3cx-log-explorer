use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::BufReader;

use threecx_log_explorer::{inspect_archive, parse_reader, read_archive_log, EventIndex, SearchFilters, LogEvent, LogLevel, RuleEngine, build_timeline, CorrelationKey, build_correlation_graph, CorrelationGraph, write_html_report};

fn main() {
    if let Err(message) = run() {
        eprintln!("Erro: {message}\n");
        print_usage();
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("index") if args.len() == 4 => {
            let mut index = EventIndex::open(&args[3])?;
            let count = index.index_file(&args[2])?;
            println!("Indexação concluída: {count} eventos em {}", args[3]);
        }
        Some("index-zip") if args.len() >= 4 && args.len() <= 5 => {
            let archive_path = &args[2];
            let db_path = &args[3];
            let entry_name = if args.len() == 5 {
                args[4].clone()
            } else {
                let summary = inspect_archive(archive_path).map_err(|e| e.to_string())?;
                summary.logs.iter()
                    .find(|log| log.name.rsplit('/').next() == Some("3CXCallFlow.log"))
                    .or_else(|| summary.logs.iter().find(|log| log.name.to_ascii_lowercase().contains("callflow")))
                    .map(|log| log.name.clone())
                    .ok_or_else(|| "Nenhum log CallFlow encontrado. Informe o caminho interno do log.".to_string())?
            };
            let mut index = EventIndex::open(db_path)?;
            let count = index.index_archive_log(archive_path, &entry_name)?;
            println!("Indexação concluída: {count} eventos de {}::{} em {}", archive_path, entry_name, db_path);
        }
        Some("stats") if args.len() == 3 => {
            let index = EventIndex::open(&args[2])?;
            let (sources, events, levels) = index.stats()?;
            println!("Fontes indexadas: {sources}\nEventos: {events}\n");
            for (level, count) in levels { println!("  {level:<10} {count}"); }
        }
        Some("search") if args.len() >= 3 => {
            let filters = parse_search_filters(&args[3..])?;
            let index = EventIndex::open(&args[2])?;
            let results = index.search(&filters)?;
            println!("Resultados: {}\n", results.len());
            for event in results {
                println!("{} [{}] linha {} | {}", event.timestamp, event.level, event.start_line, event.message.lines().next().unwrap_or_default());
                if let Some(call) = event.call_id { println!("  CallID: {call}"); }
                if let Some(flow) = event.flow_id { println!("  FlowID: {flow}"); }
                if let Some(component) = event.component { println!("  Componente: {component}"); }
                println!("  Fonte: {}", event.indexed_source);
            }
        }
        Some("timeline") if args.len() == 5 => {
            let key = parse_correlation_key(&args[3])?;
            let file = File::open(&args[2]).map_err(|e| format!("não foi possível abrir '{}': {e}", args[2]))?;
            let mut events = Vec::new();
            parse_reader(BufReader::new(file), |event| events.push(event)).map_err(|e| e.to_string())?;
            print_timeline(&events, key, &args[4], &args[2]);
        }
        Some("timeline-zip") if args.len() >= 5 && args.len() <= 6 => {
            let archive_path = &args[2];
            let (entry_name, key_arg, id) = if args.len() == 6 {
                (args[3].clone(), args[4].clone(), args[5].clone())
            } else {
                let summary = inspect_archive(archive_path).map_err(|e| e.to_string())?;
                let entry = summary.logs.iter()
                    .find(|log| log.name.rsplit('/').next() == Some("3CXCallFlow.log"))
                    .or_else(|| summary.logs.iter().find(|log| log.name.to_ascii_lowercase().contains("callflow")))
                    .map(|log| log.name.clone())
                    .ok_or_else(|| "Nenhum log CallFlow encontrado. Informe o caminho interno do log.".to_string())?;
                (entry, args[3].clone(), args[4].clone())
            };
            let key = parse_correlation_key(&key_arg)?;
            let mut events = Vec::new();
            read_archive_log(archive_path, &entry_name, |event| events.push(event)).map_err(|e| e.to_string())?;
            print_timeline(&events, key, &id, &format!("{} :: {}", archive_path, entry_name));
        }
        Some("correlate") if args.len() == 3 => {
            let file = File::open(&args[2]).map_err(|e| format!("não foi possível abrir '{}': {e}", args[2]))?;
            let mut events = Vec::new();
            parse_reader(BufReader::new(file), |event| events.push(event)).map_err(|e| e.to_string())?;
            print_correlation_graph(&build_correlation_graph(&events), &args[2]);
        }
        Some("correlate-zip") if args.len() == 3 || args.len() == 4 => {
            let archive_path = &args[2];
            let summary = inspect_archive(archive_path).map_err(|e| e.to_string())?;
            let entry_name = if args.len() == 4 {
                args[3].clone()
            } else {
                summary.logs.iter()
                    .find(|log| log.name.rsplit('/').next() == Some("3CXCallFlow.log"))
                    .or_else(|| summary.logs.iter().find(|log| log.name.to_ascii_lowercase().contains("callflow")))
                    .map(|log| log.name.clone())
                    .ok_or_else(|| "Nenhum log CallFlow encontrado. Informe o caminho interno do log.".to_string())?
            };
            let mut events = Vec::new();
            read_archive_log(archive_path, &entry_name, |event| events.push(event)).map_err(|e| e.to_string())?;
            print_correlation_graph(&build_correlation_graph(&events), &format!("{} :: {}", archive_path, entry_name));
        }
        Some("report") if args.len() == 4 => {
            let file = File::open(&args[2]).map_err(|e| format!("não foi possível abrir '{}': {e}", args[2]))?;
            let mut events = Vec::new();
            parse_reader(BufReader::new(file), |event| events.push(event)).map_err(|e| e.to_string())?;
            write_html_report(&events, &args[2], &args[3]).map_err(|e| format!("não foi possível gravar '{}': {e}", args[3]))?;
            println!("Relatório HTML criado: {} ({} eventos)", args[3], events.len());
        }
        Some("report-zip") if args.len() == 4 || args.len() == 5 => {
            let archive_path = &args[2];
            let summary = inspect_archive(archive_path).map_err(|e| e.to_string())?;
            let (entry_name, output) = if args.len() == 5 {
                (args[3].clone(), args[4].clone())
            } else {
                let entry = summary.logs.iter()
                    .find(|log| log.name.rsplit('/').next() == Some("3CXCallFlow.log"))
                    .or_else(|| summary.logs.iter().find(|log| log.name.to_ascii_lowercase().contains("callflow")))
                    .map(|log| log.name.clone())
                    .ok_or_else(|| "Nenhum log CallFlow encontrado. Informe o caminho interno do log.".to_string())?;
                (entry, args[3].clone())
            };
            let mut events = Vec::new();
            read_archive_log(archive_path, &entry_name, |event| events.push(event)).map_err(|e| e.to_string())?;
            let source = format!("{} :: {}", archive_path, entry_name);
            write_html_report(&events, &source, &output).map_err(|e| format!("não foi possível gravar '{}': {e}", output))?;
            println!("Relatório HTML criado: {} ({} eventos)", output, events.len());
        }
        Some("inspect-zip") if args.len() == 3 => {
            let summary = inspect_archive(&args[2]).map_err(|e| e.to_string())?;
            println!("Entradas: {}", summary.entry_count);
            println!("Tamanho descompactado total: {} bytes", summary.total_uncompressed_bytes);
            println!("Logs encontrados: {}", summary.logs.len());
            for log in summary.logs {
                println!("{:>12} bytes  {}", log.uncompressed_bytes, log.name);
            }
        }
        Some("analyze") if args.len() == 3 => {
            let file = File::open(&args[2]).map_err(|e| e.to_string())?;
            let mut summary = Summary::default();
            parse_reader(BufReader::new(file), |event| summary.add(event))
                .map_err(|e| e.to_string())?;
            summary.print(&args[2]);
        }
        Some("diagnose") if args.len() == 3 || args.len() == 4 => {
            let file = File::open(&args[2]).map_err(|e| e.to_string())?;
            let mut events = Vec::new();
            parse_reader(BufReader::new(file), |event| events.push(event))
                .map_err(|e| e.to_string())?;
            let rules_path = args.get(3).map(String::as_str).unwrap_or("rules/default.lua");
            print_diagnostics(&events, rules_path)?;
        }
        Some("diagnose-zip") if (args.len() >= 3 && args.len() <= 5) => {
            let archive_path = &args[2];
            let summary = inspect_archive(archive_path).map_err(|e| e.to_string())?;
            let entry_name = if args.len() >= 4 {
                args[3].clone()
            } else {
                summary.logs.iter()
                    .find(|log| log.name.rsplit('/').next() == Some("3CXCallFlow.log"))
                    .or_else(|| summary.logs.iter().find(|log| log.name.to_ascii_lowercase().contains("callflow")))
                    .map(|log| log.name.clone())
                    .ok_or_else(|| "Nenhum log CallFlow encontrado. Informe o caminho interno do log.".to_string())?
            };
            let rules_path = args.get(4).map(String::as_str).unwrap_or("rules/default.lua");
            let mut events = Vec::new();
            read_archive_log(archive_path, &entry_name, |event| events.push(event))
                .map_err(|e| e.to_string())?;
            println!("Arquivo: {} :: {}", archive_path, entry_name);
            print_diagnostics(&events, rules_path)?;
        }
        Some("analyze-zip") if args.len() == 3 || args.len() == 4 => {
            let archive_path = &args[2];
            let entry_name = if args.len() == 4 {
                args[3].clone()
            } else {
                let summary = inspect_archive(archive_path).map_err(|e| e.to_string())?;
                summary.logs.iter()
                    .find(|log| log.name.rsplit('/').next() == Some("3CXCallFlow.log"))
                    .or_else(|| summary.logs.iter().find(|log| log.name.to_ascii_lowercase().contains("callflow")))
                    .map(|log| log.name.clone())
                    .ok_or_else(|| "Nenhum log CallFlow encontrado. Informe o caminho interno do log.".to_string())?
            };
            let mut summary = Summary::default();
            read_archive_log(archive_path, &entry_name, |event| summary.add(event))
                .map_err(|e| e.to_string())?;
            summary.print(&format!("{} :: {}", archive_path, entry_name));
        }
        _ => return Err("Comando ou argumentos inválidos.".to_string()),
    }
    Ok(())
}

#[derive(Default)]
struct Summary {
    events: usize,
    levels: BTreeMap<String, usize>,
    errors: Vec<LogEvent>,
    first_timestamp: Option<String>,
    last_timestamp: Option<String>,
}

impl Summary {
    fn add(&mut self, event: LogEvent) {
        self.events += 1;
        *self.levels.entry(event.level.to_string()).or_default() += 1;
        if self.first_timestamp.is_none() {
            self.first_timestamp = Some(event.timestamp.clone());
        }
        self.last_timestamp = Some(event.timestamp.clone());
        if matches!(&event.level, LogLevel::Error | LogLevel::Fatal) && self.errors.len() < 10 {
            self.errors.push(event);
        }
    }

    fn print(&self, source: &str) {
        println!("Arquivo: {source}");
        println!("Eventos: {}", self.events);
        if let (Some(first), Some(last)) = (&self.first_timestamp, &self.last_timestamp) {
            println!("Intervalo registrado: {first} — {last}");
        }
        println!("\nEventos por nível:");
        for (level, count) in &self.levels {
            println!("  {level:<10} {count}");
        }
        if self.errors.is_empty() {
            println!("\nNenhum evento Error/Fatal explícito encontrado.");
        } else {
            println!("\nPrimeiros erros/fatais (máximo 10):");
            for event in &self.errors {
                let message = event.message.lines().next().unwrap_or_default();
                println!("  linha {} [{}] {}", event.start_line, event.timestamp, message);
                if let Some(flow) = &event.flow_id { println!("    fluxo: {flow}"); }
                if let Some(component) = &event.component { println!("    componente: {component}"); }
                if let Some(call_id) = &event.call_id { println!("    chamada: {call_id}"); }
            }
        }
    }
}

fn print_diagnostics(events: &[LogEvent], rules_path: &str) -> Result<(), String> {
    let engine = RuleEngine::from_file(rules_path)
        .map_err(|error| format!("não foi possível carregar as regras Lua '{rules_path}': {error}"))?;
    let mut findings = Vec::new();
    let mut rule_errors = 0usize;
    let explicit_errors = events.iter()
        .filter(|event| matches!(&event.level, LogLevel::Error | LogLevel::Fatal))
        .count();

    for event in events {
        match engine.diagnose(event) {
            Ok(mut matches) => findings.append(&mut matches),
            Err(error) => {
                rule_errors += 1;
                eprintln!("Aviso: falha ao executar regras na linha {}: {error}", event.start_line);
            }
        }
    }

    println!("Eventos analisados: {}", events.len());
    println!("Eventos Error/Fatal explícitos: {explicit_errors}");
    println!("Diagnósticos encontrados: {}", findings.len());
    if rule_errors > 0 {
        println!("Eventos com erro de execução das regras: {rule_errors}");
    }
    if findings.is_empty() {
        println!("Nenhuma regra correspondeu aos eventos analisados.");
    } else {
        println!();
        for finding in &findings {
            finding.print();
            println!();
        }
    }
    Ok(())
}

fn parse_correlation_key(value: &str) -> Result<CorrelationKey, String> {
    match value {
        "--call" => Ok(CorrelationKey::CallId),
        "--flow" => Ok(CorrelationKey::FlowId),
        _ => Err(format!("identificador inválido '{value}'; use --call ou --flow")),
    }
}

fn print_timeline(events: &[LogEvent], key: CorrelationKey, id: &str, source: &str) {
    let timeline = build_timeline(events, key, id);
    println!("Fonte: {source}");
    println!("Linha do tempo por {}: {}", key.label(), id);
    println!("Eventos correlacionados: {}", timeline.events.len());
    if timeline.events.is_empty() {
        println!("Nenhum evento encontrado com esse identificador.");
        return;
    }
    println!("Intervalo observado: {} — {}", timeline.first_timestamp, timeline.last_timestamp);
    println!("Erros/fatais: {}", timeline.error_count());
    println!("\nEventos por nível:");
    for (level, count) in &timeline.level_counts { println!("  {level:<10} {count}"); }
    if !timeline.components.is_empty() {
        println!("\nComponentes observados:");
        for (component, count) in &timeline.components { println!("  {component} ({count})"); }
    }
    println!("\nLinha do tempo:");
    for event in &timeline.events {
        let message = event.message.lines().next().unwrap_or_default();
        println!("{} [{}] linha {} | {}", event.timestamp, event.level, event.start_line, message);
        if let Some(flow) = &event.flow_id { println!("  FlowID: {flow}"); }
        if let Some(call) = &event.call_id { println!("  CallID: {call}"); }
        if let Some(component) = &event.component { println!("  Componente: {component}"); }
        if let Some(source_name) = &event.source { println!("  Origem: {source_name}"); }
    }
}

fn print_correlation_graph(graph: &CorrelationGraph, source: &str) {
    println!("Fonte: {source}");
    println!("Nós de correlação: {}", graph.nodes.len());
    println!("Relações observadas: {}", graph.edges.len());
    println!("Eventos sem CallID, FlowID ou componente extraído: {}", graph.events_without_identifiers);

    println!("\nEntidades:");
    for node in &graph.nodes {
        println!("  {}={} | eventos={} | erros/fatais={} | {} — {}",
            node.kind.label(), node.id, node.event_count, node.error_count,
            node.first_timestamp, node.last_timestamp);
    }
    println!("\nRelações (presença explícita no mesmo evento):");
    if graph.edges.is_empty() { println!("  Nenhuma relação explícita encontrada."); }
    for edge in &graph.edges {
        println!("  {}={} <-> {}={} | coocorrências={} | coocorrências com erro={} | {} — {}",
            edge.key.from_kind.label(), edge.key.from_id,
            edge.key.to_kind.label(), edge.key.to_id,
            edge.cooccurrences, edge.error_cooccurrences,
            edge.first_timestamp, edge.last_timestamp);
    }
    println!("\nNota: coocorrência indica apenas que os identificadores apareceram no mesmo registro; não prova causalidade.");
}

fn parse_search_filters(args: &[String]) -> Result<SearchFilters, String> {
    let mut filters = SearchFilters { limit: 100, ..SearchFilters::default() };
    let mut i = 0;
    while i < args.len() {
        if i + 1 >= args.len() { return Err(format!("faltou valor para {}", args[i])); }
        let flag = args[i].as_str();
        let value = args[i + 1].clone();
        match flag {
            "--call" => filters.call_id = Some(value),
            "--flow" => filters.flow_id = Some(value),
            "--component" => filters.component = Some(value),
            "--level" => filters.level = Some(value),
            "--text" => filters.text = Some(value),
            "--from" => filters.from = Some(value),
            "--to" => filters.to = Some(value),
            "--source" => filters.source = Some(value),
            "--limit" => filters.limit = value.parse().map_err(|_| "--limit precisa ser um número inteiro".to_string())?,
            "--offset" => filters.offset = value.parse().map_err(|_| "--offset precisa ser um número inteiro".to_string())?,
            _ => return Err(format!("filtro desconhecido: {flag}")),
        }
        i += 2;
    }
    Ok(filters)
}

fn print_usage() {
    eprintln!("Uso:");
    eprintln!("  threecx-log-explorer index <arquivo.log> <banco.sqlite>");
    eprintln!("  threecx-log-explorer index-zip <SupportInfo.zip> <banco.sqlite> [caminho/interno.log]");
    eprintln!("  threecx-log-explorer stats <banco.sqlite>");
    eprintln!("  threecx-log-explorer search <banco.sqlite> [--call ID] [--flow ID] [--component NOME] [--level NÍVEL] [--text TEXTO] [--from TIMESTAMP] [--to TIMESTAMP] [--limit N] [--offset N]");
    eprintln!("  threecx-log-explorer timeline <arquivo.log> (--call|--flow) ID");
    eprintln!("  threecx-log-explorer correlate <arquivo.log>");
    eprintln!("  threecx-log-explorer correlate-zip <SupportInfo.zip> [caminho/interno.log]");
    eprintln!("  threecx-log-explorer timeline-zip <SupportInfo.zip> [caminho/interno.log] (--call|--flow) ID");
    eprintln!("  threecx-log-explorer report <arquivo.log> <saida.html>");
    eprintln!("  threecx-log-explorer report-zip <SupportInfo.zip> [caminho/interno.log] <saida.html>");
    eprintln!("  threecx-log-explorer inspect-zip <SupportInfo.zip>");
    eprintln!("  threecx-log-explorer analyze <arquivo.log>");
    eprintln!("  threecx-log-explorer analyze-zip <SupportInfo.zip> [caminho/interno.log]");
    eprintln!("  threecx-log-explorer diagnose <arquivo.log> [regras.lua]");
    eprintln!("  threecx-log-explorer diagnose-zip <SupportInfo.zip> [caminho/interno.log] [regras.lua]");
}

