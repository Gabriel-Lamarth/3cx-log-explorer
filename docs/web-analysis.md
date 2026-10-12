# Análise avançada na versão web

## Recursos restaurados

- Linha do tempo/fluxo por CallID ou FlowID, ordenada por timestamp e linha original.
- Diagnóstico automático com as regras Lua incluídas em `rules/default.lua`.
- Catálogo consultável das regras pré-configuradas, com severidade, explicação e recomendação.
- Importação de **todos** os arquivos `.log` encontrados em um SupportInfo `.zip`, em vez de analisar somente `3CXCallFlow.log`.
- Os endpoints de análise usam o banco SQLite e não precisam carregar o arquivo ZIP inteiro em memória.

## Endpoints autenticados

- `GET /api/timeline?call_id=<id>` ou `GET /api/timeline?flow_id=<id>`: eventos correlacionados em ordem cronológica; limite padrão de 10.000 eventos.
- `GET /api/diagnostics?call_id=<id>` ou `GET /api/diagnostics?flow_id=<id>`: executa as regras sobre eventos do identificador informado. Sem identificador, varre até 5.000 eventos por padrão.
- `GET /api/rules`: catálogo das regras padrão.

## Precisão e limitações

As regras são heurísticas e indicam evidências, não uma causa-raiz garantida. A indexação compacta remove eventos Trace/Debug rotineiros; por isso, o fluxo mostra eventos relevantes preservados, não necessariamente cada linha original do log. O arquivo de origem continua preservado em `DATA_DIR/uploads` ou no diretório compartilhado.

Para chamadas com muitos eventos, informe o CallID/FlowID para evitar uma varredura ampla. A análise de SupportInfo agora indexa todas as entradas `.log`; isso pode aumentar o tamanho do SQLite, mas melhora a correlação entre logs de SIP, serviços e call flow. Faça backup do volume `/data` antes de atualizações importantes.
