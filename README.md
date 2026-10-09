# 3CX Log Explorer

Ferramenta desktop local para investigar logs 3CX e arquivos `SupportInfo.zip`. O núcleo é Rust; a interface usa Tauri 2 + Vite. Os dados são processados localmente, sem envio para serviços externos.

> **Estado do código:** inclui CLI, parser streaming, leitura de logs dentro de ZIP, indexação SQLite, linha do tempo, grafo de correlações, relatório HTML offline e interface desktop. O código-fonte e as verificações estáticas foram revisados neste pacote, mas a compilação Rust/Tauri deve ser confirmada no computador de destino; este ambiente de geração não tem `cargo`/`rustc`.

## Independência, marca e publicação

Este é um projeto independente, não afiliado, aprovado ou distribuído pela 3CX. “3CX” é usado apenas para identificar o formato/ecossistema de logs compatível. O nome e as marcas 3CX pertencem aos seus respectivos titulares; não use logos ou materiais proprietários da 3CX nem apresente este projeto como produto oficial. O software deste repositório é licenciado sob MIT, sujeita aos avisos de licenças das dependências.

Antes de publicar, revise o histórico Git e remova logs reais, `SupportInfo.zip`, bancos de dados, relatórios, tokens, credenciais, números telefônicos e dados de clientes. O `.gitignore` evita vários arquivos locais comuns, mas não substitui uma revisão do conteúdo e do histórico. O código do analisador foi escrito para este projeto; não inclua código, binários, documentação ou outros materiais proprietários da 3CX sem autorização. Esta nota não é aconselhamento jurídico; consulte os termos aplicáveis e um profissional se for distribuir comercialmente.

## Objetivos da primeira etapa

- Ler registros no formato `timestamp|thread|level|message` observado no `3CXCallFlow.log`.
- Preservar mensagens de múltiplas linhas, como stack traces e corpos de resposta HTTP.
- Extrair metadados quando presentes: serviço, ID de chamada, ID do fluxo e componente.
- Listar os logs de um ZIP sem extrair o conteúdo para o disco.
- Processar uma entrada de log diretamente do ZIP.
- Ter testes unitários e testes opcionais contra os arquivos de exemplo fornecidos.

## Arquitetura

```text
SupportInfo.zip
    └── archive.rs: inventário do ZIP + limites de segurança
          └── parser.rs: leitura streaming dos registros
                └── model.rs: LogEvent / LogLevel
                      ├── diagnostics.rs: runtime Lua em sandbox
                      ├── index.rs: banco SQLite e pesquisa
                      ├── timeline.rs: correlação cronológica por ID
                      ├── correlation.rs: grafo de coocorrência CallID/FlowID/componente
                      ├── report.rs: relatório HTML interativo autocontido
                      ├── CLI (src/main.rs)
                      ├── desktop/ (Tauri 2 + Vite)
                      └── testes (tests/)
```

### Módulos

- `model.rs`: estrutura normalizada de um evento, preservando a mensagem e o intervalo de linhas de origem.
- `parser.rs`: parser streaming; linhas sem timestamp reconhecido são anexadas ao evento anterior para preservar stack traces e respostas multilinha.
- `archive.rs`: inventaria arquivos `.log`, evita extração automática e rejeita caminhos inseguros. Há limites iniciais de tamanho descompactado por entrada e por arquivo ZIP.
- `diagnostics.rs`: executa regras Lua com um conjunto mínimo de bibliotecas (`table`, `string`, `math`, `utf8`) e remove explicitamente APIs de arquivos, processos, carregamento de módulos e depuração (`io`, `os`, `package`, `dofile`, `loadfile`, `require`, `debug`). Cada correspondência gera um diagnóstico com severidade, descrição, recomendação e contexto do evento.
- `index.rs`: persiste eventos normalizados em SQLite, com índices para timestamp, nível, chamada, fluxo, componente e fonte; reindexar uma fonte substitui os registros antigos dessa mesma fonte.
- `timeline.rs`: filtra por correspondência exata de `CallID` ou `FlowID`, ordena por timestamp e linha de origem e resume níveis, componentes e erros/fatais observados. Não associa eventos sem identificador por suposição.
- `correlation.rs`: cria nós para `CallID`, `FlowID` e componente, e relações apenas quando dois identificadores são extraídos do mesmo evento. Agrega contagens e intervalos de tempo; coocorrência não é tratada como prova de causalidade.
- `report.rs`: gera um HTML autocontido com grafo SVG clicável, tabela de eventos, filtros por texto/nível/CallID/FlowID/componente e estatísticas. Não carrega fontes, scripts ou dados de serviços externos.
- `main.rs`: CLI para inspeção, análise e diagnóstico de logs comuns ou dentro do ZIP. `desktop/src-tauri/src/main.rs` integra a interface com o núcleo Rust.
- `rules/default.lua`: conjunto inicial de heurísticas para HTTP, rede/DNS/TLS, SIP, mídia/RTP, SDP, STUN/ICE, banco de dados, permissões, recursos do sistema e serviços auxiliares. `rules/custom-template.lua` é um modelo para regras extras.

## Executar a interface desktop no Arch Linux

Instale as dependências nativas do Tauri/WebKitGTK:

```bash
sudo pacman -Syu
sudo pacman -S --needed base-devel rust nodejs npm curl wget file openssl appmenu-gtk-module gtk3 libayatana-appindicator librsvg libsoup3 webkit2gtk-4.1 pkgconf patchelf
```

Depois, na raiz do repositório:

```bash
cd desktop
npm install
npm run app:dev
```

Para compilar somente o AppImage no Arch Linux, use `../scripts/build-desktop-arch.sh` a partir da raiz do projeto, ou execute `npm run app:build -- --bundles appimage` dentro de `desktop`. Os artefatos ficam em `desktop/src-tauri/target/release/bundle/appimage/`. O workflow de publicação gera apenas o instalador `.exe` para Windows e o `.AppImage` para Linux; os outros formatos não são publicados.

## Gerar o executável Windows (.exe)

O repositório inclui o workflow `.github/workflows/build-artifacts.yml`. Para gerar o instalador `.exe` em um runner Windows e o AppImage em um runner Linux do GitHub Actions:

1. Envie o repositório ao GitHub.
2. Abra **Actions → Build Windows executable → Run workflow**.
3. Aguarde o job terminar e baixe o artefato `3cx-log-explorer-windows-...`.

O artefato do Windows contém o instalador NSIS (`*-setup.exe`); o workflow não publica outros formatos de pacote. Para publicar os arquivos automaticamente numa Release, crie e envie uma tag, por exemplo `v0.2.1`; confirme antes que o código compila e que o conteúdo publicado não inclui logs ou dados confidenciais. O workflow compila em Windows; não é necessário tentar gerar o `.exe` diretamente no Arch Linux.

## Solução de problemas

### `window.__TAURI_INTERNALS__ is undefined`

A interface foi aberta fora do processo desktop do Tauri, geralmente porque foi executado `npm run dev` e a página foi aberta no navegador. Para usar os seletores de arquivos, comandos Rust e exportação, execute `npm run app:dev` dentro da pasta `desktop`. `npm run dev` sozinho serve apenas a interface web e não disponibiliza as APIs nativas. A interface agora exibe uma mensagem explicativa nesse caso.

### `format string ... no arguments were given` na montagem das regras Lua

As chaves literais da tabela Lua precisam ser escapadas no texto passado a `format!` do Rust. O código usa `{{}}` no formato Rust, que é renderizado como `{}` no script Lua final.

## Requisitos

- Rust estável e Cargo.
- Windows 10/11 ou uma distribuição Linux atual.
- Não é necessário instalar Lua separadamente: o runtime Lua é compilado junto com o projeto.

## Compilar e testar

```bash
cargo test
cargo build --release
```

O projeto usa a crate `zip` para ler arquivos ZIP. Na primeira execução, Cargo precisará resolver as dependências do `Cargo.toml`.

## Usar a CLI

Listar os logs no pacote de suporte:

```bash
cargo run -- inspect-zip /caminho/SupportInfo.zip
```

Analisar um log que já foi extraído:

```bash
cargo run -- analyze /caminho/3CXCallFlow.log
```

Analisar o log CallFlow diretamente dentro do ZIP:

```bash
cargo run -- analyze-zip /caminho/SupportInfo.zip
```

Ou indicar explicitamente a entrada interna:

```bash
cargo run -- analyze-zip /caminho/SupportInfo.zip Logs/3CXCallFlow.log
```


Gerar um resumo de correlações explícitas entre chamadas, fluxos e componentes:

```bash
cargo run -- correlate /caminho/3CXCallFlow.log
cargo run -- correlate-zip /caminho/SupportInfo.zip
```

O relatório considera uma relação observada somente quando os dois identificadores aparecem no mesmo registro. Eventos próximos no tempo não são ligados automaticamente.

### Gerar relatório HTML interativo

O relatório abre em um navegador comum, funciona offline e não envia os logs para nenhum serviço. Clique em um nó do grafo para filtrar os registros associados; também é possível pesquisar texto, nível, `CallID`, `FlowID` ou componente.

```bash
# A partir de um arquivo .log
cargo run -- report /caminho/3CXCallFlow.log ./relatorio.html

# A partir do 3CXCallFlow.log encontrado dentro do ZIP
cargo run -- report-zip /caminho/SupportInfo.zip ./relatorio.html

# Informando caminho interno e saída
cargo run -- report-zip /caminho/SupportInfo.zip Logs/3CXCallFlow.log ./relatorio.html
```

O HTML contém dados do log; compartilhe-o apenas depois de revisar e, se necessário, anonimizar as informações.

Diagnosticar um log local usando as regras Lua padrão:

```bash
cargo run -- diagnose /caminho/3CXCallFlow.log
```

Usar um arquivo de regras personalizado:

```bash
cargo run -- diagnose /caminho/3CXCallFlow.log /caminho/minhas-regras.lua
```

Diagnosticar `3CXCallFlow.log` diretamente dentro do pacote de suporte:

```bash
cargo run -- diagnose-zip /caminho/SupportInfo.zip
```

Se necessário, informe a entrada interna e, opcionalmente, o arquivo de regras:

```bash
cargo run -- diagnose-zip /caminho/SupportInfo.zip Logs/3CXCallFlow.log rules/default.lua
```


### Indexação e pesquisa local (SQLite)

Criar/atualizar um banco a partir de um arquivo `.log`:

```bash
cargo run -- index /caminho/3CXCallFlow.log ./analise.sqlite
```

Indexar o CallFlow diretamente do pacote ZIP, sem extraí-lo:

```bash
cargo run -- index-zip /caminho/SupportInfo.zip ./analise.sqlite
```

Para escolher uma entrada específica dentro do ZIP:

```bash
cargo run -- index-zip /caminho/SupportInfo.zip ./analise.sqlite Logs/3CXCallFlow.log
```

Ver estatísticas do banco:

```bash
cargo run -- stats ./analise.sqlite
```

Pesquisar com filtros combináveis:

```bash
cargo run -- search ./analise.sqlite --call "ID-DA-CHAMADA" --level Error
cargo run -- search ./analise.sqlite --flow "_7001_7.Callflow.2" --component "discadorv2_tmslab"
cargo run -- search ./analise.sqlite --text "502 BadGateway" --from "2026-10-08 00:00:00.000" --to "2026-10-08 23:59:59.999" --limit 50
```

Filtros disponíveis: `--call`, `--flow`, `--component`, `--level`, `--text`, `--from`, `--to`, `--source`, `--limit` e `--offset`. Os filtros textuais de identificadores e nível são correspondências exatas; `--text` procura trecho no conteúdo da mensagem. Os limites temporais usam o formato do log (`YYYY-MM-DD HH:MM:SS.mmm`). Por padrão, são exibidos até 100 resultados, com limite máximo de 10.000 por consulta.

### Reconstruir a linha do tempo

Correlacione os eventos que contêm um `CallID` ou `FlowID` específico. Os resultados são ordenados pelo timestamp e, em caso de empate, pela linha de origem.

```bash
# Arquivo .log
cargo run -- timeline /caminho/3CXCallFlow.log --call "ID-DA-CHAMADA"
cargo run -- timeline /caminho/3CXCallFlow.log --flow "_7001_7.Callflow.2"

# ZIP de suporte; o programa escolhe o 3CXCallFlow.log quando disponível
cargo run -- timeline-zip /caminho/SupportInfo.zip --call "ID-DA-CHAMADA"

# ZIP com caminho interno explícito
cargo run -- timeline-zip /caminho/SupportInfo.zip Logs/3CXCallFlow.log --flow "_7001_7.Callflow.2"
```

A saída resume a quantidade de eventos, o intervalo observado, erros/fatais, níveis e componentes. Esse intervalo é o primeiro e o último timestamp encontrados para o identificador, não necessariamente a duração completa da chamada. Eventos sem o identificador escolhido ficam de fora; não são associados por proximidade temporal.

O banco SQLite contém dados potencialmente sensíveis dos logs. Mantenha-o local e não o publique junto ao código.

### Diagnósticos padrão e regras personalizadas

`rules/default.lua` inclui regras heurísticas para HTTP (401/403/404/429/500/502/503/504 e timeouts), conectividade (DNS, conexão recusada/interrompida, TLS), SIP (registro, autenticação, rota, timeout e erros de servidor), SDP/codec, RTP/SRTP, STUN/ICE/NAT, banco de dados (chave duplicada, deadlock, conexão e SQL), recursos do sistema, permissões, arquivos, configuração, WebSocket, SMTP, LDAP, licença, backup, rate limit e eventos `Fatal`/`Critical`. Nenhuma lista consegue antecipar todos os erros possíveis: as mensagens variam por versão, provedor e idioma, e alguns padrões são apenas indícios.

Na interface desktop, use **Regras Lua** para selecionar um arquivo adicional. Ele é combinado com a biblioteca padrão; não é necessário editar nem recompilar o aplicativo. O modelo está em `rules/custom-template.lua`. As regras extras substituem o conjunto extra anteriormente carregado, mas preservam as regras padrão. A regra deve retornar uma lista de tabelas Lua. Use identificadores `id` únicos para evitar confusão nos resultados.

**Importante:** scripts Lua personalizados são executados localmente. O ambiente remove APIs de arquivos, processos, carregamento de módulos e depuração, mas aceite apenas regras de fontes confiáveis. Uma regra mal escrita ou com loop infinito ainda pode atrasar a análise.

### Formato das regras Lua

O arquivo deve retornar uma lista de regras. Cada regra define `id`, `title`, `severity`, `description`, `recommendation` e uma função `match(event)` que retorna `true` quando o evento corresponde. O objeto `event` inclui `timestamp`, `thread_id`, `level`, `source`, `message`, `flow_id`, `call_id`, `component`, `start_line` e `end_line`. Veja `rules/default.lua` e `tests/diagnostics_tests.rs` como exemplos.

O runtime usa uma configuração de bibliotecas seguras: regras não recebem as APIs padrão de acesso a arquivos, processos, carregamento de módulos ou depuração. Mesmo assim, use regras de fontes confiáveis e trate os diagnósticos como hipóteses para investigação, não como prova automática da causa-raiz.

## Testes com os arquivos de exemplo

Os testes comuns usam uma pequena fixture baseada na estrutura real, com identificador de chamada e endpoint anonimizados:

```bash
cargo test
```

Para testar o parser contra o `3CXCallFlow.log` completo enviado, defina o caminho do arquivo antes de rodar a suíte. No Linux:

```bash
THREECX_REAL_LOG="/caminho/3CXCallFlow.log" cargo test parses_full_uploaded_callflow_log_when_path_is_supplied -- --exact
```

No PowerShell:

```powershell
$env:THREECX_REAL_LOG = "C:\caminho\3CXCallFlow.log"
cargo test parses_full_uploaded_callflow_log_when_path_is_supplied -- --exact
```

Para verificar o log que está dentro do ZIP, Linux:

```bash
THREECX_SUPPORT_ZIP="/caminho/SupportInfo.zip" cargo test --test real_log_tests
```

PowerShell:

```powershell
$env:THREECX_SUPPORT_ZIP = "C:\caminho\SupportInfo.zip"
cargo test --test real_log_tests
```

Os testes de integração pulam a verificação do arquivo real se a variável de ambiente não estiver definida.

## Observações sobre o parser

- O `thread_id` não é tratado como identificador único de chamada.
- O parser só preenche `call_id`, `flow_id` e `component` quando encontra padrões reconhecidos; esses campos podem ficar ausentes.
- A severidade `Error` é um sinal explícito, mas respostas HTTP não-2xx e falhas descritas em mensagens `Trace`/`Info` também podem ser relevantes. As regras de diagnóstico serão responsáveis por destacar esses padrões.
- Um log de execução permite reconstruir caminhos observados, mas não necessariamente todo o fluxo original nem todos os ramos possíveis.
- A inspeção do ZIP não extrai arquivos. Os limites de tamanho são salvaguardas iniciais, não substituem futuras proteções contra arquivos malformados.

## Privacidade

Os logs de suporte podem conter telefones, endereços, IDs de chamadas, URLs, nomes de hosts, SQL e configurações operacionais. A ferramenta é planejada para processamento local. Não publique logs reais em repositórios. A fixture do repositório usa valores anonimizados.

## Próximas etapas

1. Validar parser e leitura do ZIP com mais versões e tipos de log 3CX.
2. Correlacionar diagnósticos com eventos próximos e construir a timeline.
3. Criar UI Tauri para Windows e Linux.
4. Refinar a visualização de caminhos de chamadas e exportar subconjuntos selecionados.

## Aplicação desktop (Linux e Windows)

O projeto inclui uma interface desktop Tauri 2 em `desktop/`, conectada ao núcleo Rust existente. Ela trabalha localmente e oferece:

- abertura de arquivos `.log`/`.txt` e pacotes `SupportInfo.zip`;
- resumo de eventos, níveis, chamadas, fluxos e componentes;
- explorador de eventos com busca textual e filtros por severidade, CallID, FlowID e componente;
- visualização do registro completo, incluindo continuação multilinha;
- diagnósticos baseados nas regras Lua locais (`rules/default.lua`);
- entidades e relações explícitas de correlação;
- linha do tempo cronológica de CallID ou FlowID;
- exportação de relatório HTML independente e offline.

### Pré-requisitos

- Rust estável e Cargo (`rustup` recomendado);
- Node.js 20 ou superior e npm;
- dependências de compilação do Tauri 2 para seu sistema operacional. No Linux, instale também as bibliotecas de desenvolvimento WebKitGTK/GTK e ferramentas de compilação exigidas pela documentação oficial do Tauri para sua distribuição.

### Executar em modo de desenvolvimento

No terminal, a partir da raiz do repositório:

```bash
cd desktop
npm install
npm run app:dev
```

### Gerar instalador/pacote

```bash
cd desktop
npm install
npm run app:build
```

Os artefatos são gerados pelo Tauri em `desktop/src-tauri/target/release/bundle/`. O tipo de pacote disponível depende do sistema operacional e das bibliotecas instaladas. Para gerar builds Linux e Windows, compile em cada sistema ou configure uma pipeline de CI apropriada.

### Exportar um relatório HTML sem abrir a interface desktop

O CLI continua disponível:

```bash
cargo run -- report /caminho/3CXCallFlow.log ./relatorio.html
cargo run -- report-zip /caminho/SupportInfo.zip ./relatorio.html
cargo test
```

**Limitações conhecidas desta versão:** o diagnóstico usa as regras de exemplo do repositório; elas devem ser adaptadas ao ambiente e à versão do 3CX. A interface carrega os eventos do arquivo aberto em memória durante a sessão. O índice SQLite permanece disponível no núcleo/CLI, mas ainda não há uma tela de gerenciamento de índices na interface desktop. O projeto precisa ser compilado e validado no ambiente-alvo antes de distribuir binários.


## Fluxo SIP e criador visual de regras

A versão desktop inclui as telas **Fluxo SIP** e **Criar regra visual**. O fluxo SIP filtra por CallID/FlowID, identifica métodos e respostas comuns e destaca o intervalo INVITE → BYE quando disponível. A reconstrução é heurística e depende da qualidade e da correlação dos logs. Leia [`docs/sip-flow.md`](docs/sip-flow.md).

O criador visual gera scripts Lua a partir de termos literais, modo OU/E, nível, componente, severidade e texto de orientação. É possível testar a regra contra o log carregado e salvar um arquivo `.lua`; para ativá-lo, use **Importar Lua**. Leia [`docs/visual-rule-builder.md`](docs/visual-rule-builder.md) e [`docs/lua-rules.md`](docs/lua-rules.md).

### Diagnósticos padrão

Consulte o [catálogo de diagnósticos](docs/diagnostics-catalog.md) para ver os IDs e títulos de todas as regras padrão. A biblioteca `rules/default.lua` é heurística e inclui padrões relacionados a HTTP (400/401/403/404/429/5xx), timeouts, DNS, conexões recusadas/interrompidas, TLS/certificados, SIP (autenticação, registro, roteamento, troncos, timeouts, cancelamentos, retransmissões, respostas comuns), SDP/codecs/RTP/SRTP, STUN/ICE/NAT, banco de dados (chave duplicada, deadlock, conexão/pool/consulta), disco, memória, threads, serviços, configuração, WebSocket, SMTP, LDAP, licenças, backup, filas, gravação, voicemail, áudio unidirecional, fax/T.38 e relógio. Os padrões concretos ficam listados no próprio arquivo Lua; nem todo código SIP/HTTP é falha em todos os contextos.

## Gerar apenas EXE Windows e AppImage Linux

O workflow [`.github/workflows/build-artifacts.yml`](.github/workflows/build-artifacts.yml) compila somente o instalador NSIS `.exe` para Windows e o `.AppImage` para Linux. Ele roda manualmente pela aba **Actions** ou automaticamente ao enviar uma tag `v*`. Os artefatos ficam disponíveis na execução e, em tags, são anexados a uma GitHub Release.

Para lançar uma versão:

```bash
git tag v0.3.0
git push origin v0.3.0
```

Antes de lançar, atualize a versão em `desktop/package.json`, `desktop/src-tauri/tauri.conf.json` e `desktop/src-tauri/Cargo.toml` para manter os metadados alinhados. O AppImage é compilado em Ubuntu 24.04 no CI; teste o artefato em distribuições alvo antes de declarar suporte universal.

## Privacidade antes do push

O `.gitignore` exclui logs, pacotes SupportInfo, capturas de rede, dumps, relatórios, bancos locais e configurações/credenciais comuns. Isso reduz acidentes, mas **não substitui uma revisão do que já está rastreado pelo Git**. Antes de tornar público, execute:

```bash
git status --short
git ls-files | grep -Ei '(supportinfo|3cxcallflow|\.log$|\.zip$|\.pcapng?$|\.dmp$|\.sqlite3?$|(^|/)\.env)'
```

Se arquivos sensíveis já foram commitados, `.gitignore` não os remove do histórico. Remova-os do índice e, se já foram enviados, considere-os expostos; revogue segredos e avalie a limpeza do histórico. Use somente logs sintéticos/anonimizados em issues, testes e fixtures.
