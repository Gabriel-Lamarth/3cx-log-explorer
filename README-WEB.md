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

## Visualização unificada de eventos

A tela web mantém filtros por Execução Main (`call_ref`), CallID, FlowID, palavra-chave, nível e intervalo de tempo. O resultado filtrado alimenta a lista de eventos, a cronologia e o fluxograma, todos em ordem cronológica. Eventos Error/Fatal/Critical ficam destacados em vermelho.

Clique em um evento para abrir o visualizador central com a entrada completa indexada e até duas entradas lógicas anteriores e seguintes da mesma fonte. O endpoint autenticado `/api/context` recupera esse contexto a partir do índice SQLite. Como a indexação compacta preserva eventos relevantes, essas entradas vizinhas são os eventos preservados no índice, não necessariamente cada linha física ignorada pelo modo compacto.

Os diagnósticos recebem os mesmos identificadores, palavra-chave e intervalo de tempo informados nos filtros. Após atualizar o código, reindexe os logs se quiser garantir que o índice corresponde à versão atual do parser.
