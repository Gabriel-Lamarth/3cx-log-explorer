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
use crate::{EventIndex, SearchFilters, inspect_archive};

#[derive(Clone)] struct AppState { db: Arc<Mutex<EventIndex>>, data_dir: PathBuf, shared_dir: PathBuf, users: Arc<Vec<(String,String)>> }
#[derive(Serialize)] struct ApiMessage { ok: bool, message: String }
#[derive(Serialize)] struct Stats { sources: usize, events: usize, levels: Vec<(String,usize)> }
#[derive(Serialize)] struct EventResult { timestamp:String, level:String, message:String, source:String, start_line:usize, end_line:usize, call_id:Option<String>, flow_id:Option<String>, component:Option<String> }
#[derive(Deserialize)] struct SearchQuery { call_id:Option<String>, flow_id:Option<String>, text:Option<String>, level:Option<String>, from:Option<String>, to:Option<String>, limit:Option<usize>, offset:Option<usize> }

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
        .route("/api/stats", get(stats))
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
async fn search(State(state):State<AppState>, Query(q):Query<SearchQuery>) -> Result<Json<Vec<EventResult>>,(StatusCode,String)> {
    let filters=SearchFilters{call_id:nonempty(q.call_id),flow_id:nonempty(q.flow_id),text:nonempty(q.text),level:nonempty(q.level),from:nonempty(q.from),to:nonempty(q.to),limit:q.limit.unwrap_or(100).min(500),offset:q.offset.unwrap_or(0),..Default::default()};
    let db=state.db.lock().map_err(internal)?; let events=db.search(&filters).map_err(internal)?;
    Ok(Json(events.into_iter().map(|e|EventResult{timestamp:e.timestamp,level:e.level,message:e.message,source:e.indexed_source,start_line:e.start_line,end_line:e.end_line,call_id:e.call_id,flow_id:e.flow_id,component:e.component}).collect()))
}
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
        let n=if lower.ends_with(".log") { db.index_file_with_policy(&dest,true).map_err(internal)? } else {
            let summary=inspect_archive(&dest).map_err(internal)?;
            let selected=summary.logs.iter().find(|l|l.name.rsplit('/').next()==Some("3CXCallFlow.log")).or_else(||summary.logs.iter().find(|l|l.name.to_ascii_lowercase().contains("callflow"))).or_else(||summary.logs.first()).ok_or((StatusCode::BAD_REQUEST,"Nenhum arquivo .log encontrado no ZIP".into()))?;
            db.index_archive_log_with_policy(&dest,&selected.name,true).map_err(internal)?
        };
        indexed+=n; files+=1;
    }
    Ok(Json(ApiMessage{ok:true,message:format!("{files} arquivo(s) processado(s); {indexed} eventos relevantes indexados. Arquivos originais preservados em uploads.")}))
}

async fn scan_shared(State(state):State<AppState>) -> Result<Json<ApiMessage>,(StatusCode,String)> {
    if !state.shared_dir.exists() { return Err((StatusCode::NOT_FOUND,format!("Pasta compartilhada não encontrada: {}",state.shared_dir.display()))); }
    let entries=fs::read_dir(&state.shared_dir).map_err(internal)?; let mut files=0usize; let mut events=0usize;
    for entry in entries { let entry=entry.map_err(internal)?; let path=entry.path(); if !path.is_file(){continue;} let name=path.file_name().and_then(|n|n.to_str()).unwrap_or("").to_ascii_lowercase();
        if name.ends_with(".log") { let mut db=state.db.lock().map_err(internal)?; events+=db.index_file_with_policy(&path,true).map_err(internal)?; files+=1; }
        else if name.ends_with(".zip") { let summary=inspect_archive(&path).map_err(internal)?; if let Some(log)=summary.logs.iter().find(|l|l.name.rsplit('/').next()==Some("3CXCallFlow.log")).or_else(||summary.logs.iter().find(|l|l.name.to_ascii_lowercase().contains("callflow"))) { let mut db=state.db.lock().map_err(internal)?; events+=db.index_archive_log_with_policy(&path,&log.name,true).map_err(internal)?; files+=1; } }
    }
    Ok(Json(ApiMessage{ok:true,message:format!("{files} arquivo(s) processado(s); {events} eventos relevantes indexados.")}))
}
fn chrono_stamp()->String { use std::time::{SystemTime,UNIX_EPOCH}; SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().to_string() }
