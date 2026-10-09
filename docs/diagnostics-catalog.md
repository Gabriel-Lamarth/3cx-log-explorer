# Catálogo de diagnósticos padrão

A biblioteca atual contém **79 regras/padrões**. O catálogo descreve os identificadores e títulos definidos em `rules/default.lua`. A quantidade e os nomes podem mudar entre versões. O conjunto não é exaustivo: são heurísticas baseadas em texto, não uma validação formal do protocolo nem uma lista oficial da 3CX.

| ID | Diagnóstico |
|---|---|
| `http-502` | HTTP 502 — Bad Gateway |
| `http-400` | HTTP 400 — requisição inválida |
| `http-408` | HTTP 408 — tempo limite da requisição |
| `http-409` | HTTP 409 — conflito |
| `http-500` | HTTP 500 — erro interno |
| `http-503` | HTTP 503 — serviço indisponível |
| `http-504` | HTTP 504 — timeout do gateway |
| `http-429` | HTTP 429 — limite de requisições |
| `http-auth` | HTTP 401/403 — autenticação ou autorização |
| `authentication-failure` | Falha de autenticação |
| `http-not-found` | HTTP 404 — endpoint não encontrado |
| `http-timeout` | Timeout em comunicação HTTP |
| `network-refused` | Conexão recusada |
| `network-reset` | Conexão interrompida |
| `network-dns` | Falha de resolução DNS |
| `tls-certificate` | Falha de TLS/certificado |
| `sip-registration` | Falha de registro SIP |
| `sip-auth` | Falha de autenticação SIP |
| `sip-no-route` | Rota de saída não encontrada |
| `sip-unavailable` | Destino SIP indisponível |
| `sip-busy` | Destino ocupado ou chamada rejeitada |
| `sip-media-rejected` | SIP 488/415 — mídia não aceita |
| `sip-request-pending` | SIP 491 — requisição pendente |
| `sip-request-timeout` | Timeout SIP |
| `sip-server-error` | Erro de servidor SIP |
| `sip-sdp-codec` | Negociação SDP/codec com problema |
| `rtp-media` | Possível problema de mídia/RTP |
| `stun-ice` | Falha STUN/ICE/NAT traversal |
| `mysql-duplicate-key` | MySQL — chave duplicada |
| `database-deadlock` | Banco de dados — deadlock |
| `database-connection` | Falha de conexão com banco de dados |
| `database-query` | Falha de consulta SQL |
| `disk-space` | Possível falta de espaço em disco |
| `permission-denied` | Permissão negada |
| `file-missing` | Arquivo ou caminho não encontrado |
| `resource-memory` | Possível pressão de memória |
| `resource-thread` | Falha ao criar thread ou tarefa |
| `service-crash` | Falha ou encerramento de serviço |
| `config-invalid` | Configuração inválida |
| `websocket-disconnect` | Conexão WebSocket interrompida |
| `smtp-failure` | Falha no envio de e-mail/SMTP |
| `ldap-failure` | Falha de autenticação/consulta LDAP |
| `license-failure` | Problema de licença ou ativação |
| `backup-failure` | Falha de backup ou restauração |
| `rate-limit-retry` | Repetição excessiva ou rate limit |
| `certificate-time` | Possível relógio do sistema incorreto |
| `sip-not-found` | SIP 404 — destino não encontrado |
| `sip-auth-challenge` | SIP 401/407 — desafio de autenticação |
| `sip-temporarily-unavailable` | SIP 480 — destino temporariamente indisponível |
| `sip-address-incomplete` | SIP 484 — endereço incompleto |
| `sip-request-terminated` | SIP 487 — requisição terminada |
| `sip-auth-failed` | Autenticação SIP rejeitada |
| `sip-registration-lost` | Registro SIP perdido ou expirado |
| `sip-no-route` | SIP — nenhuma rota disponível |
| `sip-trunk-unavailable` | Tronco SIP indisponível |
| `sip-call-timeout` | Chamada SIP expirou sem conclusão |
| `sip-cancelled` | Chamada cancelada antes de atender |
| `sip-early-media` | Sinalização de mídia antecipada |
| `sip-session-refresh` | Falha de atualização de sessão SIP |
| `sip-retransmission` | Retransmissões SIP repetidas |
| `dns-resolution` | Falha de resolução DNS |
| `network-refused` | Conexão recusada pelo destino |
| `network-reset` | Conexão de rede interrompida |
| `tls-handshake` | Falha de handshake TLS |
| `certificate-chain` | Certificado TLS não confiável |
| `http-rate-limit` | HTTP 429 — limite de requisições |
| `http-auth` | HTTP 401/403 — autenticação ou autorização |
| `http-not-found` | HTTP 404 — recurso não encontrado |
| `http-service-unavailable` | HTTP 503/504 — serviço indisponível ou timeout |
| `queue-overflow` | Fila ou limite de chamadas atingido |
| `recording-failure` | Falha na gravação da chamada |
| `voicemail-failure` | Falha de correio de voz |
| `audio-one-way` | Possível áudio unidirecional |
| `codec-transcoding` | Falha de codec ou transcodificação |
| `fax-t38` | Falha de fax/T.38 |
| `websocket-handshake` | Falha no handshake WebSocket |
| `database-pool` | Pool de conexões esgotado |
| `clock-skew` | Possível divergência de relógio |
| `explicit-fatal` | Evento Fatal/Critical |

## Como interpretar

- Um achado significa que a mensagem correspondeu ao padrão configurado, não que a causa-raiz esteja comprovada.
- Respostas SIP como 401, 487 e 200 podem ser normais dependendo do contexto; examine a sequência completa.
- Vários padrões podem corresponder ao mesmo evento.
- Os IDs e mensagens usados na biblioteca são padrões de texto. Logs em outros idiomas, versões e formatos podem exigir regras personalizadas.
- Para cadastrar uma regra sem editar Lua manualmente, use **Criar regra visual** na interface desktop.
