# 3CX Log Explorer

**Aplicativo desktop para análise, investigação e diagnóstico de logs do 3CX.**

O 3CX Log Explorer é uma ferramenta para facilitar a investigação de problemas em ambientes de telefonia 3CX. Ele permite importar arquivos de log, pesquisar eventos, acompanhar o fluxo de sinalização SIP, identificar possíveis problemas por meio de regras de diagnóstico e gerar relatórios para análise posterior.

A aplicação utiliza Rust para processamento dos dados, Lua como mecanismo de regras de diagnóstico e SQLite para indexação e pesquisa local.

> **Importante:** os diagnósticos são baseados em padrões e heurísticas. Um alerta indica uma possível condição problemática, não necessariamente a causa definitiva de uma falha.

## Sumário

* [Principais funcionalidades](#principais-funcionalidades)
* [Download e instalação](#download-e-instalação)
* [Primeiros passos](#primeiros-passos)
* [Importação de logs](#importação-de-logs)
* [Pesquisa e análise de eventos](#pesquisa-e-análise-de-eventos)
* [Timeline de eventos](#timeline-de-eventos)
* [Correlação e grafo de eventos](#correlação-e-grafo-de-eventos)
* [Análise do fluxo SIP](#análise-do-fluxo-sip)
* [Diagnósticos automáticos](#diagnósticos-automáticos)
* [Construtor visual de regras](#construtor-visual-de-regras)
* [Relatórios HTML](#relatórios-html)
* [Regras Lua personalizadas](#regras-lua-personalizadas)
* [Privacidade e segurança](#privacidade-e-segurança)
* [Solução de problemas](#solução-de-problemas)
* [Limitações conhecidas](#limitações-conhecidas)

## Principais funcionalidades

| Funcionalidade      | Descrição                                                                     |
| ------------------- | ----------------------------------------------------------------------------- |
| Importação de logs  | Carrega arquivos de log e arquivos compactados suportados.                    |
| Indexação local     | Utiliza SQLite para facilitar pesquisas nos dados importados.                 |
| Pesquisa de eventos | Ajuda a localizar mensagens e ocorrências relevantes.                         |
| Timeline            | Organiza eventos cronologicamente.                                            |
| Correlação          | Ajuda a relacionar eventos que podem pertencer ao mesmo fluxo de comunicação. |
| Grafo de eventos    | Apresenta relações entre eventos e entidades identificadas.                   |
| Análise SIP         | Filtra métodos e respostas SIP para investigar chamadas.                      |
| Diagnósticos        | Executa regras heurísticas para apontar possíveis problemas.                  |
| Regras visuais      | Permite configurar regras de diagnóstico pela interface.                      |
| Regras Lua          | Permite criar verificações personalizadas.                                    |
| Relatórios HTML     | Gera uma representação dos resultados para consulta e compartilhamento.       |

## Download e instalação

Os binários são disponibilizados na página de Releases do GitHub:

**[Acessar downloads do 3CX Log Explorer](https://github.com/Gabriel-Lamarth/3cx-log-explorer/releases)**

### Windows

1. Acesse a página de Releases.
2. Baixe o instalador `.exe` correspondente à versão desejada.
3. Execute o instalador e siga as instruções apresentadas.
4. Abra o 3CX Log Explorer pelo atalho criado ou pelo menu de aplicativos.

Caso o Windows exiba um aviso de segurança para um executável não reconhecido, confira a origem do download antes de prosseguir. Não ignore alertas sem verificar a procedência do arquivo.

### Linux

1. Acesse a página de Releases.
2. Baixe o arquivo `.AppImage`.
3. Abra um terminal no diretório em que o arquivo foi baixado.
4. Conceda permissão de execução:

```bash
chmod +x 3cx-log-explorer.AppImage
```

5. Execute o aplicativo:

```bash
./3cx-log-explorer.AppImage
```

Substitua `3cx-log-explorer.AppImage` pelo nome real do arquivo baixado.

Se o aplicativo não abrir, consulte a seção [Solução de problemas](#solução-de-problemas).

## Primeiros passos

O fluxo recomendado para uma primeira análise é:

1. Obtenha os logs relacionados ao problema investigado.
2. Abra o 3CX Log Explorer.
3. Importe os arquivos disponíveis.
4. Pesquise o horário aproximado da ocorrência, o identificador da chamada ou outro dado conhecido.
5. Examine os eventos próximos ao problema.
6. Utilize a timeline e a correlação para compreender a sequência dos acontecimentos.
7. Se a investigação envolver telefonia SIP, consulte o fluxo SIP.
8. Execute ou examine os diagnósticos automáticos.
9. Exporte um relatório para registrar os resultados.

Para obter resultados melhores, utilize logs que cubram o período anterior, durante e posterior à falha. Um arquivo que contenha somente parte da comunicação pode não ser suficiente para determinar a causa.

## Importação de logs

A importação é o ponto de partida para a análise.

### Como utilizar

1. Abra a função de importação de arquivos na interface.
2. Selecione os logs que deseja investigar.
3. Quando disponível, selecione o arquivo compactado de suporte, como um `SupportInfo.zip`.
4. Aguarde o processamento e a indexação dos dados.
5. Abra os eventos importados para iniciar a investigação.

O aplicativo pode processar arquivos compactados e arquivos de log compatíveis com seus leitores. A compatibilidade depende do formato e do conteúdo dos arquivos.

### Boas práticas

* Prefira um conjunto de logs relacionado ao incidente.
* Evite misturar registros de períodos sem relação com a investigação.
* Preserve os arquivos originais para poder repetir a análise.
* Confira se o período importado realmente inclui o horário do problema.
* Não compartilhe arquivos de suporte publicamente sem remover informações sensíveis.

Se não houver eventos relevantes, verifique o formato dos arquivos e se o pacote importado contém os registros esperados.

## Pesquisa e análise de eventos

A pesquisa permite localizar ocorrências específicas sem precisar examinar manualmente todo o conteúdo dos logs.

### Como utilizar

1. Importe os arquivos.
2. Abra a área de pesquisa ou listagem de eventos.
3. Informe um termo relevante para a investigação.
4. Examine os resultados encontrados.
5. Abra um evento para verificar seus detalhes e contexto.

Exemplos de termos que podem ser úteis, dependendo do conteúdo dos logs:

* Número de ramal.
* Endereço IP.
* Identificador de chamada.
* Mensagens ou códigos SIP.
* Termos associados a erros.
* Horários ou outros identificadores presentes nos registros.

### Interpretando os resultados

Não analise uma mensagem isoladamente quando o problema envolver uma sequência de eventos. Uma mensagem aparentemente normal pode anteceder uma falha, e um erro pode ser apenas uma consequência de um evento anterior.

Sempre que possível, examine também os registros imediatamente anteriores e posteriores à ocorrência.

## Timeline de eventos

A timeline organiza os eventos por ordem temporal e facilita a reconstrução do que aconteceu durante uma chamada ou incidente.

### Como utilizar

1. Selecione os eventos ou a chamada que deseja investigar.
2. Abra a visualização temporal.
3. Identifique o primeiro evento relevante.
4. Acompanhe a sequência até o momento da falha ou do encerramento.
5. Compare os horários e a ordem dos eventos.

### Quando utilizar

A timeline é especialmente útil para investigar:

* Chamadas que não completam.
* Encerramentos inesperados.
* Atrasos entre etapas de sinalização.
* Sequências de erros.
* Eventos que ocorrem antes ou depois de uma transferência.

A timeline depende da qualidade dos timestamps disponíveis. Registros ausentes, relógios desalinhados ou formatos de data inconsistentes podem prejudicar a interpretação.

## Correlação e grafo de eventos

A correlação procura relacionar eventos que compartilham informações relevantes. O grafo apresenta essas relações visualmente.

Dependendo dos dados disponíveis, as relações podem envolver identificadores de chamada, endereços, participantes ou outros elementos reconhecidos durante o processamento.

### Como utilizar

1. Localize um evento importante na pesquisa.
2. Abra a visualização de correlação ou o grafo.
3. Examine os eventos e entidades conectados.
4. Siga as relações relevantes para descobrir possíveis etapas anteriores ou posteriores.
5. Volte aos registros originais para confirmar cada hipótese.

### Interpretando o grafo

Uma conexão representa uma relação identificada pelo aplicativo, não necessariamente uma relação causal.

Por exemplo, dois eventos associados ao mesmo identificador podem fazer parte da mesma chamada, mas isso não significa que um tenha provocado o outro.

Use o grafo para orientar a investigação e confirme as conclusões consultando os registros de origem.

## Análise do fluxo SIP

A análise SIP auxilia na investigação da sinalização envolvida no estabelecimento, manutenção e encerramento de chamadas.

A ferramenta disponibiliza filtros para métodos SIP, incluindo:

* `INVITE`
* `ACK`
* `CANCEL`
* `BYE`

Também contempla categorias de respostas SIP, que ajudam a separar respostas de sucesso, respostas provisórias e respostas de erro, conforme os dados identificados.

### Como utilizar

1. Abra a visualização de fluxo SIP.
2. Selecione a chamada ou o conjunto de eventos que deseja analisar.
3. Aplique os filtros de método ou categoria de resposta disponíveis.
4. Examine as mensagens na ordem em que ocorreram.
5. Verifique os códigos de resposta e as mensagens imediatamente anteriores e posteriores.
6. Relacione os resultados com a timeline e os diagnósticos.

### O que observar

**`INVITE`**

É utilizado para iniciar uma sessão ou convidar um participante. Investigue as respostas recebidas e se o fluxo prossegue conforme esperado.

**`ACK`**

Confirma determinadas respostas finais associadas ao estabelecimento de uma sessão. A presença ou ausência precisa ser interpretada considerando o contexto do diálogo SIP.

**`CANCEL`**

Solicita o cancelamento de uma tentativa de estabelecimento ainda em andamento. Verifique quem iniciou o cancelamento e o que ocorreu imediatamente antes.

**`BYE`**

É utilizado para encerrar uma sessão estabelecida. Observe a sequência de eventos para entender quando e em que contexto a chamada terminou.

### Respostas SIP

As respostas SIP fornecem pistas sobre o resultado de uma operação. Entre os exemplos conhecidos estão:

* `200 OK`: indica sucesso para a operação correspondente.
* `401 Unauthorized` e `407 Proxy Authentication Required`: podem fazer parte de um desafio de autenticação esperado.
* `404 Not Found`: pode indicar que o recurso solicitado não foi encontrado.
* `408 Request Timeout`: indica que uma solicitação expirou por tempo.
* `486 Busy Here`: indica que o destino está ocupado.
* `5xx`: respostas associadas a falhas de servidor, cuja causa depende do código específico e do contexto.

Um código isolado não é suficiente para diagnosticar uma falha. Por exemplo, uma resposta de autenticação pode ser seguida por uma nova solicitação válida. Examine a sequência completa antes de concluir que houve um erro.

## Diagnósticos automáticos

O aplicativo inclui um conjunto de regras heurísticas de diagnóstico para apontar padrões que podem indicar problemas.

A versão documentada do projeto inclui 79 padrões de diagnóstico. A quantidade e o comportamento das regras podem mudar entre versões.

### Como utilizar

1. Importe os logs.
2. Acesse a área de diagnósticos.
3. Execute a análise, caso a interface solicite uma ação explícita.
4. Examine os resultados apresentados.
5. Abra os eventos relacionados ao diagnóstico.
6. Compare o alerta com a timeline e o fluxo SIP.
7. Registre a conclusão somente depois de verificar as evidências.

### Como interpretar os resultados

Um diagnóstico pode indicar um padrão suspeito, uma resposta relevante ou uma sequência que merece atenção. Ele não substitui a investigação técnica.

Para cada alerta, procure responder:

* Qual evento foi identificado?
* Em que horário ocorreu?
* Quais registros vieram antes e depois?
* O padrão é esperado naquele cenário?
* Existem evidências independentes que confirmam a hipótese?

### Falsos positivos e falsos negativos

* **Falso positivo:** uma regra aponta um problema que não representa uma falha real naquele cenário.
* **Falso negativo:** uma falha real não é identificada pelas regras existentes.

Por isso, os diagnósticos devem ser tratados como auxílio à investigação, e não como vereditos definitivos.

## Construtor visual de regras

O construtor visual permite criar regras personalizadas sem precisar escrever todo o código Lua manualmente.

Ele é útil quando você deseja reconhecer um padrão específico de falha ou adaptar a análise a um ambiente particular.

### Como utilizar

1. Abra o construtor visual de regras.
2. Crie uma regra.
3. Configure os critérios e as condições disponíveis.
4. Defina a identificação ou descrição da regra.
5. Gere o código Lua correspondente.
6. Utilize a função de teste, quando disponível, para verificar o comportamento.
7. Salve a regra.
8. Execute a análise e examine os resultados.

### Recomendações para criar boas regras

* Comece com uma condição simples.
* Utilize informações que realmente existam nos logs.
* Evite condições excessivamente genéricas.
* Teste a regra com exemplos positivos e negativos.
* Confira se o alerta apresenta contexto suficiente para ser investigado.
* Evite interpretar a mera ausência de uma mensagem como prova de falha sem considerar a integridade dos dados.

O comportamento exato depende dos campos e operadores disponíveis na versão instalada.

## Relatórios HTML

O aplicativo oferece geração de relatórios em HTML para registrar ou consultar os resultados de uma investigação.

### Como utilizar

1. Realize a análise desejada.
2. Revise os eventos, diagnósticos e informações relevantes.
3. Abra a função de exportação ou geração de relatório.
4. Gere o relatório HTML.
5. Abra o arquivo resultante em um navegador.

O relatório pode ser utilizado para documentação, revisão técnica e compartilhamento de conclusões.

Antes de encaminhar um relatório, confira se ele contém números de telefone, ramais, endereços IP, identificadores de chamadas ou outros dados que não devem ser divulgados.

A disponibilidade de cada seção do relatório depende das informações efetivamente processadas pelo aplicativo.

## Regras Lua personalizadas

O mecanismo Lua permite estender as verificações do aplicativo com regras específicas.

Esse recurso é destinado a usuários que conhecem os padrões dos logs e desejam implementar critérios de diagnóstico próprios.

### Fluxo recomendado

1. Consulte a documentação de regras Lua incluída no repositório.
2. Examine uma regra de exemplo.
3. Defina o padrão que deseja identificar.
4. Implemente a regra utilizando a interface disponibilizada pelo mecanismo de diagnóstico.
5. Teste o código com registros conhecidos.
6. Salve a regra no formato esperado pelo aplicativo.
7. Execute a análise e confira os resultados.

### Cuidados

* Utilize apenas APIs e campos suportados pelo mecanismo de regras.
* Valide o comportamento antes de utilizar regras em análises importantes.
* Não coloque senhas, tokens ou outras credenciais no código.
* Evite regras que produzam alertas excessivos ou difíceis de interpretar.

Para detalhes sobre a estrutura das regras, consulte a documentação técnica [`docs/lua-rules.md`](docs/lua-rules.md).

## Privacidade e segurança

O 3CX Log Explorer foi projetado para trabalhar com dados locais, utilizando processamento local e indexação em SQLite. Isso não significa que todo dado derivado seja automaticamente seguro para compartilhar.

### Boas práticas

* Trabalhe com cópias dos logs quando possível.
* Não publique arquivos `SupportInfo.zip` ou logs reais em repositórios públicos.
* Não compartilhe bancos SQLite, relatórios ou capturas de tela sem verificar seu conteúdo.
* Remova informações sensíveis antes de abrir uma issue ou enviar arquivos para terceiros.
* Proteja os diretórios e arquivos gerados durante a análise.

O `.gitignore` do projeto deve ser mantido para ajudar a evitar o envio acidental de logs, bancos de dados, dumps, relatórios e credenciais ao Git.

## Solução de problemas

### O aplicativo não abre no Linux

Confira se o arquivo tem permissão de execução:

```bash
chmod +x 3cx-log-explorer.AppImage
```

Tente executá-lo pelo terminal para visualizar possíveis mensagens de erro:

```bash
./3cx-log-explorer.AppImage
```

Dependendo do sistema, podem ser necessárias bibliotecas de runtime compatíveis com o empacotamento do aplicativo.

### Nenhum evento foi encontrado

Verifique:

* Se os arquivos foram importados corretamente.
* Se o formato dos logs é compatível.
* Se os arquivos contêm dados do período investigado.
* Se os filtros utilizados são muito restritivos.
* Se o termo pesquisado aparece realmente nos registros.

### O diagnóstico não identificou o problema

Nem toda falha possui uma regra correspondente. Utilize a pesquisa, a timeline, a correlação e a análise SIP para continuar a investigação. Considere criar uma regra personalizada quando identificar um padrão repetível.

### Os resultados parecem incorretos

Confira a ordem temporal, os identificadores utilizados e o conteúdo original dos eventos. Uma relação visual ou um alerta heurístico deve ser confirmado com os registros de origem.

### A geração do relatório falhou

Confira se o diretório de destino permite gravação e se há espaço em disco. Se o problema persistir, registre a mensagem de erro e as etapas que levaram à falha.

## Limitações conhecidas

* A precisão dos resultados depende da qualidade, da completude e da compatibilidade dos logs.
* As regras heurísticas podem produzir falsos positivos e falsos negativos.
* A correlação depende das informações que o parser consegue reconhecer.
* Nem todos os formatos de log ou variantes do 3CX necessariamente são suportados.
* O aplicativo é uma ferramenta de investigação e não substitui a validação técnica dos resultados.
* O comportamento de cada função pode variar conforme a versão.

## Documentação adicional

Consulte os documentos disponíveis no repositório:

* [`docs/visual-rule-builder.md`](docs/visual-rule-builder.md) — construtor visual de regras.
* [`docs/sip-flow.md`](docs/sip-flow.md) — análise do fluxo SIP.
* [`docs/diagnostics-catalog.md`](docs/diagnostics-catalog.md) — catálogo de diagnósticos.
* [`docs/lua-rules.md`](docs/lua-rules.md) — criação de regras Lua.
* [`docs/architecture.md`](docs/architecture.md) — arquitetura interna do projeto.

## Contribuições e problemas

Sugestões, relatos de bugs e melhorias são bem-vindos.

Antes de abrir uma issue, descreva:

1. A versão do aplicativo e o sistema operacional.
2. O que você estava tentando fazer.
3. Os passos necessários para reproduzir o problema.
4. O comportamento esperado e o comportamento observado.
5. A mensagem de erro, se houver.

**Não anexe logs reais, arquivos de suporte ou informações confidenciais a issues públicas.** Se precisar demonstrar um problema de parsing, utilize um exemplo anonimizado.

## Licença

Consulte o arquivo [`LICENSE`](LICENSE) do repositório para conhecer as condições de uso e distribuição.
