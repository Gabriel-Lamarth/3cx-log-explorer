# Identificadores automáticos na versão web

A interface consulta `GET /api/identifiers?kind=call|flow&limit=50&offset=0&q=` para listar identificadores distintos já detectados pelo parser durante a indexação. A resposta inclui quantidade de eventos e intervalo temporal observado.

- `kind=call` lista CallIDs; `kind=flow` lista FlowIDs.
- `q` filtra por trecho do identificador.
- A paginação é feita no servidor, em lotes de até 500 registros por chamada (a interface usa 50).
- A seleção de um item preenche o filtro correspondente e solicita pesquisa, linha do tempo e diagnóstico.
- A pesquisa por palavras-chave continua usando SQLite FTS5; filtros de tempo permanecem no endpoint `/api/search`.

A lista é derivada dos identificadores reconhecidos pelo parser e preservados no índice. Linhas descartadas pela política compacta ou formatos que o parser não reconhece não podem contribuir para a lista. Para incluir mais identificadores, revise a extração no parser e/ou a política de indexação.
