Patch: corrigir extração do FlowID

Arquivo alterado: src/parser.rs

O parser agora remove o prefixo `FlowID=` ou `FlowID ` quando ele faz parte do token
extraído, mantendo apenas o identificador, por exemplo:
  Entrada: FlowID=_7001_7.Callflow.2
  Saída:   _7001_7.Callflow.2

Como aplicar (a partir da raiz do repositório):
  git apply fix-flowid-prefix.patch

Depois rode:
  cargo test --test parser_tests
  cargo test

O patch foi gerado a partir da versão do código-fonte anexada nesta conversa. Se seu
checkout tiver alterações diferentes, `git apply` pode solicitar resolução manual.
