use threecx_log_explorer::{parse_text, RuleEngine};

const RULES: &str = r#"
return {
  {
    id = "contains-warning",
    title = "Teste de regra",
    severity = "warning",
    description = "A mensagem contém o marcador.",
    recommendation = "Investigue o evento.",
    match = function(event)
      return string.find(event.message, "MARCADOR", 1, true) ~= nil
    end,
  }
}
"#;

#[test]
fn lua_rule_matches_message_and_preserves_event_context() {
    let events = parse_text("2026-01-02 03:04:05.123|42|Info|mensagem com MARCADOR CallID abc-123\n");
    let engine = RuleEngine::from_script(RULES).expect("valid Lua rules");
    let findings = engine.diagnose(&events[0]).expect("rule execution");

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id, "contains-warning");
    assert_eq!(findings[0].start_line, 1);
    assert_eq!(findings[0].call_id.as_deref(), Some("abc-123"));
}

#[test]
fn lua_rule_returns_no_finding_when_not_matched() {
    let events = parse_text("2026-01-02 03:04:05.123|42|Info|mensagem normal\n");
    let engine = RuleEngine::from_script(RULES).expect("valid Lua rules");
    assert!(engine.diagnose(&events[0]).expect("rule execution").is_empty());
}

#[test]
fn lua_sandbox_does_not_expose_filesystem_api() {
    let script = "return { { id='x', title='x', severity='error', description='x', recommendation='x', match=function(e) return io == nil and os == nil and package == nil and debug == nil and dofile == nil and loadfile == nil and load == nil and require == nil end } }";
    let events = parse_text("2026-01-02 03:04:05.123|42|Info|message\n");
    let engine = RuleEngine::from_script(script).expect("sandboxed Lua rules");
    let findings = engine.diagnose(&events[0]).expect("rule execution");
    assert_eq!(findings.len(), 1, "io/os/package should be absent in the sandbox");
}

#[test]
fn bundled_rules_detect_known_patterns() {
    let rules = include_str!("../rules/default.lua");
    let engine = RuleEngine::from_script(rules).expect("bundled rules should load");
    let events = parse_text(concat!(
        "2026-01-02 03:04:05.123|42|Trace|Received '502 BadGateway' from service\n",
        "2026-01-02 03:04:06.123|42|Error|MySQL error: Duplicate entry '0' for key 'PS_SMT.PRIMARY'\n"
    ));
    let first = engine.diagnose(&events[0]).expect("HTTP rule should run");
    let second = engine.diagnose(&events[1]).expect("MySQL rule should run");
    assert_eq!(first[0].rule_id, "http-502");
    assert_eq!(second[0].rule_id, "mysql-duplicate-key");
}

#[test]
fn bundled_rules_cover_common_network_database_and_fatal_patterns() {
    let rules = include_str!("../rules/default.lua");
    let engine = RuleEngine::from_script(rules).expect("default rules should load");
    let events = parse_text(concat!(
        "2026-01-02 03:04:05.123|42|Warning|TLS handshake failed: certificate verify failed\n",
        "2026-01-02 03:04:06.123|42|Error|Deadlock found when trying to get lock\n",
        "2026-01-02 03:04:07.123|42|Fatal|service terminated unexpectedly\n",
        "2026-01-02 03:04:08.123|42|Info|normal call progress\n"
    ));

    let tls = engine.diagnose(&events[0]).expect("TLS rules should run");
    let db = engine.diagnose(&events[1]).expect("database rules should run");
    let fatal = engine.diagnose(&events[2]).expect("fatal rule should run");
    let normal = engine.diagnose(&events[3]).expect("normal event should run");

    assert!(tls.iter().any(|finding| finding.rule_id == "tls-certificate"));
    assert!(db.iter().any(|finding| finding.rule_id == "database-deadlock"));
    assert!(fatal.iter().any(|finding| finding.rule_id == "explicit-fatal"));
    assert!(normal.is_empty());
}
