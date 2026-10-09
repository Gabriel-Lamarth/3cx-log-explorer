# Arquitetura técnica — etapa 1

## Princípios

1. **Local-first:** o conteúdo do SupportInfo não sai da máquina.
2. **Streaming:** evitar extrair o ZIP inteiro e evitar carregar o log inteiro em memória durante a leitura normal.
3. **Rastreabilidade:** cada evento guarda arquivo e linhas de origem (o arquivo será adicionado ao contexto de análise/indexação na próxima etapa).
4. **Parser tolerante:** campos opcionais podem estar ausentes e níveis desconhecidos são preservados.
5. **Diagnóstico explicável:** regras futuras deverão devolver a regra acionada, evidência textual e recomendação; nenhuma regra deve declarar causa raiz como certeza sem evidência suficiente.

## Caminho de dados

1. `inspect_archive` lista entradas `.log` e valida nomes e limites de tamanho.
2. `read_archive_log` abre a entrada selecionada dentro do ZIP sem extraí-la.
3. `parse_reader` lê uma linha por vez.
4. Linhas com timestamp 3CX iniciam um novo evento; linhas subsequentes sem esse formato são anexadas ao evento anterior.
5. `LogEvent` guarda os campos normalizados e o texto original para que o visualizador possa mostrar o contexto sem perda.

## Próxima fronteira de módulos

- `index`: índices em SQLite, filtros e consultas.
- `correlation`: associação de eventos por CallID, flow ID, componente, thread e tempo.
- `diagnostics`: regras Lua, achados e pontuação de confiança.
- `flow`: grafo de execução observado e exportação Mermaid/Graphviz.
- `ui`: aplicação Tauri; a UI não deverá reimplementar parsing ou diagnóstico.

## Segurança a expandir

- Limites de tamanho compactado/descompactado, contagem de entradas e tempo de processamento.
- Não extrair caminhos arbitrários do ZIP.
- Não executar scripts Lua encontrados dentro do ZIP; carregar somente regras locais confiáveis.
- Sanitizar HTML e texto ao exibir logs na interface.
- Anonimizar segredos, telefones, IDs e URLs antes de exportar relatórios.


## Etapa 2 — regras Lua

`diagnostics.rs` carrega um arquivo de regras em um runtime Lua com bibliotecas seguras. As regras recebem um evento normalizado e podem gerar um ou mais `Finding`s. O resultado inclui a localização original e identificadores de chamada/fluxo quando detectados. A CLI oferece `diagnose` e `diagnose-zip`. O runtime é local e não exige Lua instalado à parte (`mlua` usa Lua 5.4 vendorizado).

As regras atuais são heurísticas para HTTP 502 e erro de chave primária duplicada no MySQL. Próxima etapa: indexação SQLite e filtros combináveis para que a GUI não precise manter todos os eventos em memória.


## Etapa 3 — indexação local

`index.rs` usa SQLite para armazenar fontes importadas e eventos normalizados. A importação de uma fonte é transacional: ao reindexar a mesma fonte, os eventos antigos são removidos e substituídos apenas se a leitura terminar sem erro. Índices B-tree aceleram filtros por timestamp, severidade, CallID, FlowID, componente e fonte. A busca aceita filtros combináveis, trecho textual, limites temporais e paginação. O banco pode conter dados sensíveis e deve permanecer local.
