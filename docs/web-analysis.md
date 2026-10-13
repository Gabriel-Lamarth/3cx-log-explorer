# Referências de execução Main

A versão web reconhece referências no formato `_8510_44.Main.624` no conteúdo de eventos do CallFlow e indexa-as em `events.call_ref`, separadas de `CallID` e `FlowID`.

## O que foi adicionado

- Índice SQLite para `call_ref` e migração aditiva de bancos existentes.
- Endpoint `GET /api/calls` com contagem e intervalo temporal, com filtros opcionais `from`, `to` e `text`.
- Filtro exato `call_ref` em `GET /api/search`, `GET /api/timeline` e `GET /api/diagnostics`.
- Cronologia ordenada por timestamp/ordem de origem e fluxograma sequencial inferido dos eventos.
- Diagnósticos carregam o identificador da execução para abrir os eventos dessa mesma execução.
- Persistência do texto bruto do evento (`raw`) para exibir a linha/multilinha original do arquivo indexado.

## Como usar

1. Faça backup do volume `/data` antes de atualizar.
2. Implante a versão atualizada.
3. Reprocesse/reindexe os logs de CallFlow que já estavam indexados: registros antigos têm `call_ref` nulo porque a coluna nova não existia.
4. Abra a seção **Execuções / chamadas Main** e selecione a referência desejada.
5. Use os filtros de período e termo para restringir a lista e a análise. Ao selecionar uma referência, a cronologia, o fluxograma, os diagnósticos e a busca de eventos usam `call_ref` exato.

## Limitações

O fluxograma representa a ordem cronológica e os componentes detectados nos eventos indexados; não infere ramificações sem relações explícitas no log. O índice compacto pode omitir eventos rotineiros antigos que foram descartados anteriormente; para reconstruir a sequência mais completa, reindexe os arquivos de origem com a política de retenção apropriada. O visualizador mostra o texto bruto preservado pelo parser, não relê o arquivo original em tempo real.
