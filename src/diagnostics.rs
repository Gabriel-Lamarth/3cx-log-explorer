//! Sandboxed Lua diagnostic rules for normalized 3CX log events.

use crate::model::{LogEvent, LogLevel};
use mlua::{Function, Lua, LuaOptions, StdLib, Table};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Finding {
    pub rule_id: String,
    pub title: String,
    pub severity: String,
    pub description: String,
    pub recommendation: String,
    pub timestamp: String,
    pub start_line: usize,
    pub call_id: Option<String>,
    pub flow_id: Option<String>,
    pub component: Option<String>,
}

/// Executes user-selected Lua rules with safe standard libraries only.
/// The Lua script must return an array of rule tables with `match(event)` and
/// finding metadata (`id`, `title`, `severity`, `description`, `recommendation`).
#[derive(Clone, Debug, serde::Serialize)]
pub struct RuleSummary { pub id: String, pub title: String, pub severity: String, pub description: String, pub recommendation: String }

pub struct RuleEngine {
    lua: Lua,
    rules: Vec<CompiledRule>,
}

struct CompiledRule {
    id: String,
    title: String,
    severity: String,
    description: String,
    recommendation: String,
    matcher: Function,
}

impl RuleEngine {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let script = std::fs::read_to_string(path)?;
        Self::from_script(&script).map_err(Into::into)
    }

    pub fn from_script(script: &str) -> mlua::Result<Self> {
        // Create only the libraries used by rules; file/process/module/debug APIs
        // are never exposed. Metadata and function handles are compiled once.
        let lua = Lua::new_with(
            StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8,
            LuaOptions::default(),
        )?;
        let globals = lua.globals();
        for name in ["io", "os", "package", "debug", "dofile", "loadfile", "loadstring", "load", "require", "collectgarbage", "print"] {
            globals.set(name, mlua::Value::Nil)?;
        }

        let table: Table = lua.load(script).set_name("3CX diagnostic rules").eval()?;
        let mut rules = Vec::new();
        for value in table.sequence_values::<Table>() {
            let rule = value?;
            rules.push(CompiledRule {
                id: rule.get("id")?,
                title: rule.get("title")?,
                severity: rule.get("severity")?,
                description: rule.get("description")?,
                recommendation: rule.get("recommendation")?,
                matcher: rule.get("match")?,
            });
        }
        Ok(Self { lua, rules })
    }

    pub fn summaries(&self) -> Vec<RuleSummary> {
        self.rules.iter().map(|r| RuleSummary { id:r.id.clone(), title:r.title.clone(), severity:r.severity.clone(), description:r.description.clone(), recommendation:r.recommendation.clone() }).collect()
    }

    pub fn diagnose(&self, event: &LogEvent) -> mlua::Result<Vec<Finding>> {
        let event_table = self.lua.create_table()?;
        event_table.set("timestamp", event.timestamp.as_str())?;
        event_table.set("thread_id", event.thread_id.as_str())?;
        event_table.set("level", event.level.as_str())?;
        event_table.set("source", event.source.as_deref().unwrap_or(""))?;
        event_table.set("message", event.message.as_str())?;
        event_table.set("flow_id", event.flow_id.as_deref().unwrap_or(""))?;
        event_table.set("call_id", event.call_id.as_deref().unwrap_or(""))?;
        event_table.set("component", event.component.as_deref().unwrap_or(""))?;
        event_table.set("start_line", event.start_line)?;
        event_table.set("end_line", event.end_line)?;

        let mut findings = Vec::new();
        for rule in &self.rules {
            if rule.matcher.call::<bool>(event_table.clone())? {
                findings.push(Finding {
                    rule_id: rule.id.clone(),
                    title: rule.title.clone(),
                    severity: rule.severity.clone(),
                    description: rule.description.clone(),
                    recommendation: rule.recommendation.clone(),
                    timestamp: event.timestamp.clone(),
                    start_line: event.start_line,
                    call_id: event.call_id.clone(),
                    flow_id: event.flow_id.clone(),
                    component: event.component.clone(),
                });
            }
        }
        Ok(findings)
    }
}

impl Finding {
    pub fn print(&self) {
        println!("[{}] {} ({})", self.severity.to_uppercase(), self.title, self.rule_id);
        println!("  Linha {} | {}", self.start_line, self.timestamp);
        println!("  {}", self.description);
        println!("  Recomendação: {}", self.recommendation);
        if let Some(call_id) = &self.call_id { println!("  Chamada: {call_id}"); }
        if let Some(flow_id) = &self.flow_id { println!("  Fluxo: {flow_id}"); }
        if let Some(component) = &self.component { println!("  Componente: {component}"); }
    }
}

/// True for explicit error-level records; diagnostic rules can also detect
/// suspicious patterns hidden in Trace/Info events.
pub fn is_explicit_error(event: &LogEvent) -> bool {
    matches!(&event.level, LogLevel::Error | LogLevel::Fatal)
}
