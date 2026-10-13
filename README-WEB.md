# 3CX Log Explorer — versão web

Versão web experimental para implantação em container (EasyPanel), com upload de logs, leitura de pasta compartilhada, autenticação, filtros por CallID/FlowID e pesquisa de texto via SQLite FTS5.

Veja o [guia de implantação no EasyPanel](docs/deployment-easypanel.md).

## Desenvolvimento local

Requer Rust stable e Cargo. Execute:

```bash
APP_USERS='analista:uma-senha-com-mais-de-12-caracteres' cargo run --release --bin threecx-log-explorer-web
```

Abra `http://localhost:8080`. Por padrão, o banco fica em `/data`; em desenvolvimento, defina `DATA_DIR=./data SHARED_LOGS_DIR=./shared-logs`.

## Importante

Esta versão inclui a lista de referências de execução `*.Main.*`, cronologia e fluxograma sequencial por referência, diagnósticos filtrados e exibição do texto bruto indexado. O fluxograma é uma sequência temporal dos eventos detectados, não uma reconstrução semântica de ramificações. Consulte `docs/web-analysis.md` para detalhes e para reindexar fontes já existentes. Teste com cópias anonimizadas de logs antes de utilizar em produção.
