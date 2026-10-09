# Implantação web no EasyPanel

## Arquitetura

A versão web adiciona o binário `threecx-log-explorer-web`, reaproveitando o parser Rust e o indexador SQLite. A interface é servida pelo mesmo processo na porta 8080. A autenticação usa HTTP Basic com contas definidas em `APP_USERS`; publique atrás do proxy HTTPS do EasyPanel e não exponha a porta diretamente à internet.

## Variáveis de ambiente

- `APP_USERS`: lista `usuario:senha,usuario2:senha2`. Senhas devem ter pelo menos 12 caracteres. Não use vírgulas nas senhas. Para ambientes maiores, considere autenticação centralizada/proxy SSO antes de expor externamente.
- `BIND_ADDR`: padrão `0.0.0.0:8080`.
- `DATA_DIR`: padrão `/data`; guarda SQLite e uploads originais.
- `SHARED_LOGS_DIR`: padrão `/shared-logs`; pasta lida pela função de varredura.

## EasyPanel

1. Envie este repositório ao GitHub sem incluir logs reais, SupportInfo, banco de dados ou `.env`.
2. No EasyPanel, crie um serviço do tipo App a partir do repositório e use o `Dockerfile` da raiz.
3. Configure a porta interna `8080` e associe um domínio com HTTPS.
4. Defina `APP_USERS` como variável secreta no painel. Crie contas individuais e use senhas longas.
5. Monte um volume persistente em `/data`.
6. Monte a pasta de logs do host ou volume compartilhado em `/shared-logs`, preferencialmente como somente leitura. O caminho precisa existir dentro do container e ser legível pelo usuário UID 10001.
7. Faça deploy e autentique-se no navegador.

Não use o `docker-compose.yml` como configuração literal dentro do EasyPanel sem adaptar os volumes: `./shared-logs` refere-se ao diretório do projeto/host disponível ao Compose, não a uma pasta arbitrária do host.

## Docker Compose local

1. Copie `.env.example` para `.env` e substitua a senha.
2. Crie `shared-logs/` e coloque lá cópias dos logs que deseja processar.
3. Execute `docker compose up --build -d`.
4. Acesse `http://localhost:8080` na máquina local. Para uso real em rede, configure TLS no proxy reverso.

## Entrada dos logs

- Upload: envie `.log` ou `.zip`. Para ZIPs, o serviço prioriza `3CXCallFlow.log`, depois um log cujo nome contenha `callflow`, e por fim o primeiro log disponível.
- Pasta compartilhada: clique em **Processar pasta compartilhada**. A primeira versão varre somente o primeiro nível da pasta, não recursivamente, e reconhece `.log` e `.zip`.

## Compactação dos eventos

A indexação web usa uma política heurística. Erros e avisos são mantidos; eventos de Trace/Debug são mantidos se corresponderem a palavras-chave relacionadas a falhas ou eventos de chamada. Outros níveis são mantidos. Isso pode reduzir significativamente o ruído, mas pode descartar uma linha útil que não corresponda aos padrões configurados. Os arquivos originais são preservados em `/data/uploads` (uploads) ou no volume compartilhado original.

## Pesquisa

- `CallID` e `FlowID` usam igualdade exata e índices SQLite.
- Texto usa SQLite FTS5: termos separados são combinados com AND. É busca textual tokenizada, não busca por substring arbitrária.
- Cada página retorna no máximo 500 eventos na interface.

## Persistência, backup e privacidade

Faça backup do volume `/data` para preservar o índice e os uploads. Os arquivos de origem podem conter números de telefone, IPs, ramais, credenciais e dados de clientes; limite acesso ao volume e não envie logs para issues públicas. A autenticação HTTP Basic não substitui HTTPS. Para múltiplos usuários, use contas individuais e não compartilhe credenciais.
