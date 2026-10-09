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

Esta é uma primeira implementação funcional proposta para testes, não uma migração concluída de todas as telas Tauri. A interface web cobre importação, pesquisa e estatísticas. Timeline visual, grafo, regras Lua e exportação de relatório ainda precisam ser integrados à interface web. Teste com cópias anonimizadas de logs antes de utilizar em produção.
