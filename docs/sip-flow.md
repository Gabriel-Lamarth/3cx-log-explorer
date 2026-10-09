# Fluxo de sinalização SIP

A tela **Fluxo SIP** reconstrói a sequência cronológica de eventos associados a um `CallID` interno ou `FlowID` extraído pelo parser. Ela também inclui registros cuja mensagem contenha literalmente o identificador informado, o que permite tentar localizar um cabeçalho SIP `Call-ID` copiado do log. Ela destaca métodos SIP conhecidos, códigos de resposta e mensagens que parecem indicar falha. O filtro permite visualizar todos os eventos, somente sinalização SIP, métodos específicos, respostas 1xx/2xx/4xx–6xx e erros.

## INVITE até BYE

Por padrão, a tela destaca o intervalo desde o primeiro evento identificado como `INVITE` até o primeiro `BYE` posterior, se ambos estiverem presentes. Se não houver BYE, o intervalo termina no fim dos eventos associados. Desmarque **Destacar intervalo INVITE → BYE** para ver a chamada completa, inclusive eventos anteriores e posteriores.

A interface pode reconhecer `INVITE`, `ACK`, `BYE`, `CANCEL`, `REGISTER`, `OPTIONS`, `PRACK`, `UPDATE`, `INFO`, `NOTIFY`, `SUBSCRIBE`, `REFER`, `MESSAGE`, além de respostas SIP 1xx–6xx quando a mensagem tem indicação SIP.

## Interpretação e limitações

- O `CallID` usado pela ferramenta pode ser o identificador interno do fluxo 3CX, não necessariamente o cabeçalho SIP `Call-ID`.
- Se a sinalização estiver dividida em vários arquivos ou IDs, escolha um `FlowID`, compare logs correlatos e pesquise o SIP Call-ID na tela Eventos.
- Um evento não identificado como SIP não significa que não seja relevante à chamada. O filtro **Todos os eventos** preserva contexto não-SIP associado ao ID.
- A reconstrução é uma heurística textual, não um decodificador completo de SIP/SDP. Não infira direção de pacotes ou causalidade sem evidência.
- Linhas perdidas, timestamps de máquinas diferentes e truncamento do pacote de suporte podem alterar a ordem aparente.
