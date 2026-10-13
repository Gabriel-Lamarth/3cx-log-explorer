use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Query, State},
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::{Path, PathBuf}, sync::{Arc, Mutex}};
use crate::{EventIndex, SearchFilters, inspect_archive, LogEvent, LogLevel, RuleEngine};

#[derive(Clone)] struct AppState { db: Arc<Mutex<EventIndex>>, data_dir: PathBuf, shared_dir: PathBuf, users: Arc<Vec<(String,String)>> }
#[derive(Serialize)] struct ApiMessage { ok: bool, message: String }
#[derive(Serialize)] struct Stats { sources: usize, events: usize, levels: Vec<(String,usize)> }
#[derive(Serialize)] struct EventResult { timestamp:String, level:String, message:String, raw:String, source:String, start_line:usize, end_line:usize, call_id:Option<String>, flow_id:Option<String>, component:Option<String>, call_ref:Option<String> }
#[derive(Deserialize)] struct SearchQuery { call_id:Option<String>, flow_id:Option<String>, call_ref:Option<String>, text:Option<String>, level:Option<String>, from:Option<String>, to:Option<String>, limit:Option<usize>, offset:Option<usize> }

pub async fn run() -> Result<(), String> {
    let data_dir = PathBuf::from(env::var("DATA_DIR").unwrap_or_else(|_| "/data".into()));
    let shared_dir = PathBuf::from(env::var("SHARED_LOGS_DIR").unwrap_or_else(|_| "/shared-logs".into()));
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(data_dir.join("uploads")).map_err(|e| e.to_string())?;
    let users = parse_users(&env::var("APP_USERS").map_err(|_| "Defina APP_USERS com usuario:senha,usuario2:senha2".to_string())?)?;
    let index = EventIndex::open(data_dir.join("index.sqlite"))?;
    let state = AppState { db: Arc::new(Mutex::new(index)), data_dir, shared_dir, users: Arc::new(users) };
    let protected = Router::new()
        .route("/", get(home))
        .route("/api/search", get(search))
        .route("/api/context", get(event_context))
        .route("/api/stats", get(stats))
        .route("/api/timeline", get(timeline))
        .route("/api/calls", get(calls))
        .route("/api/identifiers", get(identifiers))
        .route("/api/diagnostics", get(diagnostics))
        .route("/api/rules", get(rules_catalog))
        .route("/api/upload", post(upload))
        .route("/api/scan-shared", post(scan_shared))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth));
    let app = Router::new().merge(protected).with_state(state);
    let addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| format!("bind {addr}: {e}"))?;
    println!("3CX Log Explorer web listening on {addr}");
    axum::serve(listener, app).await.map_err(|e| e.to_string())
}

fn parse_users(raw: &str) -> Result<Vec<(String,String)>,String> {
    let users: Vec<_> = raw.split(',').filter_map(|item| { let (u,p)=item.trim().split_once(':')?; if u.is_empty() || p.is_empty(){None}else{Some((u.to_string(),p.to_string()))} }).collect();
    if users.is_empty() { return Err("APP_USERS inválido; use usuario:senha,usuario2:senha2".into()); }
    if users.iter().any(|(_,p)| p.len() < 12) { return Err("Use senhas com pelo menos 12 caracteres em APP_USERS".into()); }
    Ok(users)
}

async fn auth(State(state): State<AppState>, req: Request<Body>, next: Next) -> Response {
    let valid = req.headers().get(header::AUTHORIZATION).and_then(|h| h.to_str().ok()).and_then(|h| h.strip_prefix("Basic ")).and_then(|encoded| STANDARD.decode(encoded).ok()).and_then(|bytes| String::from_utf8(bytes).ok()).and_then(|pair| pair.split_once(':').map(|(u,p)| (u.to_string(),p.to_string()))).map(|(u,p)| state.users.iter().any(|(name,password)| *name == u && *password == p)).unwrap_or(false);
    if valid { next.run(req).await } else { (StatusCode::UNAUTHORIZED, [(header::WWW_AUTHENTICATE, "Basic realm=\"3CX Log Explorer\", charset=\"UTF-8\"")], "Autenticação necessária").into_response() }
}

async fn home() -> Html<&'static str> { Html(include_str!("../web/index.html")) }
async fn stats(State(state):State<AppState>) -> Result<Json<Stats>,(StatusCode,String)> {
    let db=state.db.lock().map_err(internal)?; let (sources,events,levels)=db.stats().map_err(internal)?; Ok(Json(Stats{sources,events,levels}))
}

#[derive(Deserialize)] struct ContextQuery { source: String, start_line: usize, end_line: Option<usize>, call_id: Option<String>, flow_id: Option<String>, call_ref: Option<String> }
#[derive(Serialize)] struct EventContext { previous: Vec<EventResult>, current: Option<EventResult>, next: Vec<EventResult> }
async fn event_context(State(state):State<AppState>, Query(q):Query<ContextQuery>) -> Result<Json<EventContext>,(StatusCode,String)> {
    let end_line = q.end_line.unwrap_or(q.start_line).max(q.start_line);
    let filters = SearchFilters { source: Some(q.source.clone()), call_id: nonempty(q.call_id), flow_id: nonempty(q.flow_id), call_ref: nonempty(q.call_ref), limit: 10000, offset: 0, ..Default::default() };
    let db = state.db.lock().map_err(internal)?;
    let mut rows = db.search(&filters).map_err(internal)?;
    rows.sort_by(|a,b| a.start_line.cmp(&b.start_line));
    let pivot = rows.iter().position(|e| e.start_line == q.start_line);
    let Some(pivot) = pivot else { return Ok(Json(EventContext { previous: Vec::new(), current: None, next: Vec::new() })); };
    let previous = rows[..pivot].iter().rev().take(2).cloned().collect::<Vec<_>>().into_iter().rev().map(to_result).collect();
    let current = Some(to_result(rows[pivot].clone()));
    let next = rows.iter().skip(pivot+1).filter(|e| e.start_line > end_line).take(2).cloned().map(to_result).collect();
    Ok(Json(EventContext { previous, current, next }))
}

async fn search(State(state):State<AppState>, Query(q):Query<SearchQuery>) -> Result<Json<Vec<EventResult>>,(StatusCode,String)> {
    let filters=SearchFilters{call_id:nonempty(q.call_id),flow_id:nonempty(q.flow_id),call_ref:nonempty(q.call_ref),text:nonempty(q.text),level:nonempty(q.level),from:nonempty(q.from),to:nonempty(q.to),limit:q.limit.unwrap_or(100).min(500),offset:q.offset.unwrap_or(0),..Default::default()};
    let db=state.db.lock().map_err(internal)?; let events=db.search(&filters).map_err(internal)?;
    Ok(Json(events.into_iter().map(to_result).collect()))
}

#[derive(Serialize)] struct CallRefResult { id: String, events: usize, first_timestamp: String, last_timestamp: String }
#[derive(Deserialize)] struct CallsQuery { from: Option<String>, to: Option<String>, text: Option<String>, limit: Option<usize> }
async fn calls(State(state):State<AppState>, Query(q):Query<CallsQuery>) -> Result<Json<Vec<CallRefResult>>,(StatusCode,String)> {
    let db=state.db.lock().map_err(internal)?;
    let rows=db.list_call_refs(nonempty(q.from).as_deref(),nonempty(q.to).as_deref(),nonempty(q.text).as_deref(),q.limit.unwrap_or(200)).map_err(internal)?;
    Ok(Json(rows.into_iter().map(|(id,events,first_timestamp,last_timestamp)| CallRefResult{id,events,first_timestamp,last_timestamp}).collect()))
}

#[derive(Deserialize)] struct IdentifiersQuery { kind: String, limit: Option<usize> }
async fn identifiers(State(state):State<AppState>, Query(q):Query<IdentifiersQuery>) -> Result<Json<Vec<String>>,(StatusCode,String)> {
    let db=state.db.lock().map_err(internal)?;
    Ok(Json(db.list_identifiers(&q.kind, q.limit.unwrap_or(2000)).map_err(internal)?))
}

#[derive(Serialize)] struct TimelineResponse { key: String, id: String, total: usize, errors: usize, first_timestamp: String, last_timestamp: String, events: Vec<EventResult> }
#[derive(Serialize)] struct DiagnosticsResponse { scanned_events: usize, findings: Vec<crate::Finding> }
#[derive(Deserialize)] struct TimelineQuery { call_id: Option<String>, flow_id: Option<String>, call_ref: Option<String>, from: Option<String>, to: Option<String>, text: Option<String>, limit: Option<usize> }

async fn timeline(State(state):State<AppState>, Query(q):Query<TimelineQuery>) -> Result<Json<TimelineResponse>,(StatusCode,String)> {
    let (key,id,filters) = if let Some(id)=nonempty(q.call_ref) { ("Execução",id.clone(),SearchFilters{call_ref:Some(id),from:nonempty(q.from),to:nonempty(q.to),text:nonempty(q.text),limit:q.limit.unwrap_or(10000).min(10000),offset:0,..Default::default()}) }
        else if let Some(id)=nonempty(q.call_id) { ("CallID",id.clone(),SearchFilters{call_id:Some(id),limit:q.limit.unwrap_or(10000).min(10000),offset:0,..Default::default()}) }
        else if let Some(id)=nonempty(q.flow_id) { ("FlowID",id.clone(),SearchFilters{flow_id:Some(id),limit:q.limit.unwrap_or(10000).min(10000),offset:0,..Default::default()}) }
        else { return Err((StatusCode::BAD_REQUEST,"Informe call_id ou flow_id".into())); };
    let db=state.db.lock().map_err(internal)?; let mut rows=db.search(&filters).map_err(internal)?;
    rows.sort_by(|a,b|a.timestamp.cmp(&b.timestamp).then(a.start_line.cmp(&b.start_line)));
    let events:Vec<EventResult>=rows.into_iter().map(to_result).collect();
    let errors=events.iter().filter(|e| matches!(e.level.to_ascii_lowercase().as_str(),"error"|"fatal"|"critical"|"warning"|"warn")).count();
    let first_timestamp=events.first().map(|e|e.timestamp.clone()).unwrap_or_default(); let last_timestamp=events.last().map(|e|e.timestamp.clone()).unwrap_or_default(); let total=events.len();
    Ok(Json(TimelineResponse{key:key.into(),id,total,errors,first_timestamp,last_timestamp,events}))
}

async fn diagnostics(State(state):State<AppState>, Query(q):Query<TimelineQuery>) -> Result<Json<DiagnosticsResponse>,(StatusCode,String)> {
    let filters=SearchFilters {
        call_ref: nonempty(q.call_ref), call_id: nonempty(q.call_id), flow_id: nonempty(q.flow_id),
        from: nonempty(q.from), to: nonempty(q.to), text: nonempty(q.text),
        limit:q.limit.unwrap_or(5000).min(10000), offset:0, ..Default::default()
    };
    let rows={ let db=state.db.lock().map_err(internal)?; db.search(&filters).map_err(internal)? };
    let engine=RuleEngine::from_script(include_str!("../rules/default.lua")).map_err(internal)?;
    let mut findings=Vec::new(); let scanned=rows.len();
    for row in rows { let event=to_log_event(row); findings.extend(engine.diagnose(&event).map_err(internal)?); }
    findings.sort_by(|a,b| severity_rank(&a.severity).cmp(&severity_rank(&b.severity)).then(a.timestamp.cmp(&b.timestamp)));
    findings.dedup_by(|a,b| a.rule_id==b.rule_id && a.start_line==b.start_line && a.call_id==b.call_id && a.flow_id==b.flow_id && a.call_ref==b.call_ref);
    Ok(Json(DiagnosticsResponse{scanned_events:scanned,findings}))
}

async fn rules_catalog() -> Result<Json<Vec<crate::diagnostics::RuleSummary>>,(StatusCode,String)> {
    let engine=RuleEngine::from_script(include_str!("../rules/default.lua")).map_err(internal)?;
    Ok(Json(engine.summaries()))
}
fn severity_rank(s:&str)->u8 { match s.to_ascii_lowercase().as_str() { "critical"|"fatal"=>0,"error"=>1,"warning"|"warn"=>2,_=>3 } }
fn to_result(e:crate::IndexedEvent)->EventResult { EventResult{timestamp:e.timestamp,level:e.level,message:e.message,raw:e.raw,source:e.indexed_source,start_line:e.start_line,end_line:e.end_line,call_id:e.call_id,flow_id:e.flow_id,component:e.component,call_ref:e.call_ref} }
fn to_log_event(e:crate::IndexedEvent)->LogEvent { LogEvent{timestamp:e.timestamp,thread_id:e.thread_id,level:LogLevel::parse(&e.level),source:e.source_name,message:e.message.clone(),raw:e.raw,start_line:e.start_line,end_line:e.end_line,flow_id:e.flow_id,call_id:e.call_id,component:e.component,call_ref:e.call_ref} }

fn nonempty(v:Option<String>)->Option<String>{v.map(|x|x.trim().to_string()).filter(|x|!x.is_empty())}
fn internal<E:ToString>(e:E)->(StatusCode,String){(StatusCode::INTERNAL_SERVER_ERROR,e.to_string())}

async fn upload(State(state):State<AppState>, mut multipart:Multipart) -> Result<Json<ApiMessage>,(StatusCode,String)> {
    let mut indexed=0usize; let mut files=0usize;
    while let Some(field)=multipart.next_field().await.map_err(internal)? {
        let Some(name)=field.file_name().map(str::to_string) else { continue; };
        let safe=Path::new(&name).file_name().and_then(|n|n.to_str()).unwrap_or("upload.log").to_string();
        let lower=safe.to_ascii_lowercase();
        if !(lower.ends_with(".log")||lower.ends_with(".zip")) { return Err((StatusCode::BAD_REQUEST,format!("Formato não suportado: {safe}. Envie .log ou .zip."))); }
        let dest=state.data_dir.join("uploads").join(format!("{}-{safe}",chrono_stamp()));
        let mut output=tokio::fs::File::create(&dest).await.map_err(internal)?;
        let mut written: u64=0;
        let mut field=field;
        while let Some(chunk)=field.chunk().await.map_err(internal)? {
            written=written.saturating_add(chunk.len() as u64);
            if written > 1024*1024*1024 { return Err((StatusCode::PAYLOAD_TOO_LARGE,"Arquivo excede o limite de 1 GiB".into())); }
            tokio::io::AsyncWriteExt::write_all(&mut output,&chunk).await.map_err(internal)?;
        }
        tokio::io::AsyncWriteExt::flush(&mut output).await.map_err(internal)?;
        drop(output);
        let mut db=state.db.lock().map_err(internal)?;
        if lower.ends_with(".log") { indexed+=db.index_file_with_policy(&dest,true).map_err(internal)?; }
        else {
            let summary=inspect_archive(&dest).map_err(internal)?;
            if summary.logs.is_empty() { return Err((StatusCode::BAD_REQUEST,format!("Nenhum arquivo .log encontrado no ZIP: {safe}"))); }
            // Index all logs in the SupportInfo archive, not only CallFlow, so diagnostics
            // can correlate PBX, SIP, services and call-flow messages across files.
            for log in &summary.logs { indexed+=db.index_archive_log_with_policy(&dest,&log.name,true).map_err(internal)?; }
        }
        files+=1;
    }
    Ok(Json(ApiMessage{ok:true,message:format!("{files} arquivo(s) processado(s); {indexed} eventos relevantes indexados. Arquivos originais preservados em uploads.")}))
}

async fn scan_shared(State(state):State<AppState>) -> Result<Json<ApiMessage>,(StatusCode,String)> {
    if !state.shared_dir.exists() { return Err((StatusCode::NOT_FOUND,format!("Pasta compartilhada não encontrada: {}",state.shared_dir.display()))); }
    let entries=fs::read_dir(&state.shared_dir).map_err(internal)?; let mut files=0usize; let mut events=0usize;
    for entry in entries { let entry=entry.map_err(internal)?; let path=entry.path(); if !path.is_file(){continue;} let name=path.file_name().and_then(|n|n.to_str()).unwrap_or("").to_ascii_lowercase();
        if name.ends_with(".log") { let mut db=state.db.lock().map_err(internal)?; events+=db.index_file_with_policy(&path,true).map_err(internal)?; files+=1; }
        else if name.ends_with(".zip") { let summary=inspect_archive(&path).map_err(internal)?; if !summary.logs.is_empty() { let mut db=state.db.lock().map_err(internal)?; for log in &summary.logs { events+=db.index_archive_log_with_policy(&path,&log.name,true).map_err(internal)?; } files+=1; } }
    }
    Ok(Json(ApiMessage{ok:true,message:format!("{files} arquivo(s) processado(s); {events} eventos relevantes indexados.")}))
}
fn chrono_stamp()->String { use std::time::{SystemTime,UNIX_EPOCH}; SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().to_string() }
