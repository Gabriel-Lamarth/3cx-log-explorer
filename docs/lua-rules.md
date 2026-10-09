# Regras de diagnóstico em Lua

O 3CX Log Explorer usa Lua para permitir adicionar diagnósticos sem recompilar o núcleo Rust. `rules/default.lua` contém as regras distribuídas com o projeto; `rules/custom-template.lua` é um ponto de partida para regras locais.

## Como uma regra funciona

O arquivo Lua deve retornar uma lista (array) de tabelas. Cada tabela descreve um achado e contém uma função `match(event)` que retorna `true` quando a regra corresponde ao evento.

```lua
return {
  {
    id = "empresa-api-limite-requisicoes",
    title = "API — limite de requisições atingido",
    severity = "warning", -- info, warning ou error
    description = "A resposta indica que o limite de requisições foi atingido.",
    recommendation = "Verifique rate limit, frequência de chamadas e política de retry.",
    match = function(event)
      local message = string.lower(event.message or "")
      return string.find(message, "429", 1, true) ~= nil
        and string.find(message, "rate limit", 1, true) ~= nil
    end,
  },
}
```

Use um `id` estável e único, preferencialmente com prefixo próprio. Os campos obrigatórios são `id`, `title`, `severity`, `description`, `recommendation` e `match`. A severidade deve ser `info`, `warning` ou `error` para manter a apresentação consistente na interface.

## Campos disponíveis em `event`

| Campo | Significado |
|---|---|
| `timestamp` | Data/hora textual do evento |
| `thread_id` | Identificador de thread extraído do log |
| `level` | Nível original, como Trace, Debug, Info, Warning, Error ou Fatal |
| `source` | Origem, se reconhecida; string vazia quando ausente |
| `message` | Mensagem do evento, incluindo continuação multilinha |
| `flow_id` | FlowID extraído; string vazia quando ausente |
| `call_id` | CallID extraído; string vazia quando ausente |
| `component` | Componente extraído; string vazia quando ausente |
| `start_line` | Primeira linha física do registro |
| `end_line` | Última linha física do registro |

Os campos opcionais ausentes são expostos como strings vazias, não como `nil`.

## Como o motor combina as regras

- As regras padrão são sempre incluídas.
- As regras adicionais são anexadas às padrão.
- Se uma regra adicional repetir o `id` de uma regra padrão (ou de uma regra adicional já adicionada), a repetida é ignorada. Para evitar confusão, dê IDs novos às regras próprias.
- O motor prepara o runtime e compila/carrega os metadados das regras uma vez por execução do diagnóstico; a função `match` é avaliada para cada evento.
- Um evento pode disparar várias regras. O resultado não é automaticamente deduplicado por chamada: cada achado aponta para o evento que o acionou.

## Sandbox e limites

O runtime disponibiliza apenas bibliotecas padrão selecionadas (`table`, `string`, `math` e `utf8`). APIs como `io`, `os`, `package`, `debug`, `dofile`, `loadfile`, `require` e funções de carregamento dinâmico são removidas ou não disponibilizadas. O código da regra não recebe API de rede do aplicativo.

Isso reduz a superfície de acesso, mas não equivale a uma garantia formal de segurança contra qualquer script hostil. Carregue somente regras que você entende e controla. A API atual não oferece orçamento de instruções por regra; uma regra com loop infinito pode travar a análise. Evite loops sem limite e lógica pesada.

## Boas práticas

1. Faça correspondências específicas, em vez de procurar palavras genéricas como `error` em todos os eventos.
2. Use `string.find(text, needle, 1, true)` para buscar texto literal, sem padrões Lua acidentais.
3. Normalize com `string.lower(event.message or "")` se a caixa não for importante.
4. Prefira exigir dois sinais em conjunto para reduzir falsos positivos.
5. Não escreva senhas, tokens, números telefônicos ou conteúdo sensível em `title`, `description` ou `recommendation`.
6. Trate regras como heurísticas: uma mensagem pode descrever uma falha recuperada, um retry esperado ou um código usado em outro contexto.
7. Teste a regra em logs anonimizados e compare os achados com o contexto da chamada antes de concluir a causa raiz.

## Testar pela CLI

```bash
cargo run -- diagnose /caminho/3CXCallFlow.log /caminho/minhas-regras.lua
cargo run -- diagnose-zip /caminho/SupportInfo.zip
```

Na interface desktop, use a opção de carregar regras Lua. Um arquivo inválido deve ser corrigido antes de ativá-lo; mantenha cópia das regras personalizadas fora do repositório se contiverem lógica interna da empresa.
