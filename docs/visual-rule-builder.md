# Criador visual de regras de diagnóstico

O 3CX Log Explorer inclui a tela **Criar regra visual** para criar regras Lua sem escrever código manualmente.

## Campos

- **ID único:** identificador estável, composto por letras, números, ponto, hífen e sublinhado. Use prefixos próprios, como `empresa-`, para evitar conflito com regras internas.
- **Nome:** título mostrado nos achados.
- **Severidade:** `info`, `warning` ou `error`. Isso classifica o achado, não altera o nível original do evento.
- **Correspondência OU:** basta um dos termos encontrar-se na mensagem.
- **Correspondência E:** todos os termos devem aparecer na mensagem.
- **Nível opcional:** restringe a regra ao nível original do log.
- **Componente opcional:** exige que o componente contenha o texto informado.
- **Descrição e recomendação:** orientam a investigação; evite declarar causa-raiz sem evidência.

A correspondência é textual, literal e ignora diferenças entre maiúsculas e minúsculas. Os termos são interpretados como texto comum, não como regex.

## Fluxo recomendado

1. Abra um log sintético ou devidamente anonimizado.
2. Preencha ID, título, termos, descrição e recomendação.
3. Clique **Gerar script Lua**.
4. Clique **Testar no log carregado** para validar sintaxe e ver quantos achados seriam produzidos. O teste inclui as regras padrão e não ativa nem salva a regra.
5. Salve o arquivo `.lua`.
6. Para usá-lo na análise atual, clique **Importar Lua** e selecione o arquivo salvo.
7. Confira amostras positivas e negativas antes de usar a regra em produção.

## Formato Lua

O script deve retornar uma lista de tabelas. Cada regra precisa ter `id`, `title`, `severity`, `description`, `recommendation` e `match(event)`, retornando booleano. Consulte [`lua-rules.md`](lua-rules.md) e `rules/custom-template.lua`.

## Limitações e segurança

- IDs duplicados não substituem regras padrão.
- A prévia usa todos os eventos carregados; em logs grandes, o teste pode levar algum tempo.
- Lua é código executável. O sandbox remove bibliotecas usuais de arquivos, rede, processos e módulos, mas ainda não há limite de instruções por regra. Não carregue scripts de origem desconhecida: loops infinitos podem travar a análise.
- Não inclua tokens, senhas, números de clientes ou conteúdo privado nas descrições e nos arquivos de regra publicados.
