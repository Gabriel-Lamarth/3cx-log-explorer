-- Copie este arquivo e adicione suas regras próprias.
-- O arquivo deve retornar uma lista Lua; ele será combinado com as regras padrão.
-- Campos disponíveis em event: timestamp, thread_id, level, source, message,
-- flow_id, call_id, component, start_line, end_line.
-- Bibliotecas permitidas: table, string, math e utf8. Sem io/os/package/debug.

return {
    {
        id = "empresa-exemplo-codigo-interno",
        title = "Código de erro específico do ambiente",
        severity = "warning", -- info, warning ou error
        description = "A mensagem contém um código de erro conhecido neste ambiente.",
        recommendation = "Consulte o procedimento interno e correlacione CallID/FlowID.",
        match = function(event)
            local message = string.lower(event.message or "")
            return string.find(message, "meu_codigo_de_erro", 1, true) ~= nil
        end,
    },
}
