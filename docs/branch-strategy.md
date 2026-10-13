# Estratégia de branches: Desktop e Web

O repositório mantém o núcleo Rust compartilhado (`src/`, `Cargo.toml` e testes), mas cada linha de desenvolvimento tem uma branch própria:

- `desktop`: aplicação Tauri em `desktop/`, empacotamento Windows/AppImage e ajustes da interface nativa.
- `web`: aplicação web e sua implantação. Esta branch deve conter os arquivos e workflows da versão web, sem misturar alterações exclusivas da interface Tauri.

## Separar as branches sem perder o estado atual

Antes de executar, confirme que as alterações locais estão commitadas e que o remoto `origin` aponta para o repositório correto.

```bash
git fetch origin
git switch main
git pull --ff-only origin main
git switch -c desktop
git push -u origin desktop
```

A branch `desktop` parte do estado atual de `main`. Para preparar `web`, parta da branch/commit que já contém a versão web funcional — não crie `web` a partir de `desktop` se isso trouxer arquivos ou workflows errados:

```bash
git switch <branch-ou-commit-com-a-versao-web>
git switch -c web
git push -u origin web
```

Se a versão web funcional estiver atualmente em `main`, crie `web` antes de substituir ou reorganizar `main`:

```bash
git switch main
git switch -c web
git push -u origin web
```

## Regras para manter a separação

1. Desenvolva interface Tauri e instaladores apenas em `desktop`.
2. Desenvolva frontend/servidor web e implantação EasyPanel apenas em `web`.
3. Leve correções do núcleo compartilhado (`src/` e testes) entre branches com `cherry-pick` ou PR, após testar em ambas.
4. Configure workflows com filtros de branch explícitos. O workflow de release desktop deve rodar em tags `v*` ou manualmente; o deploy web deve rodar apenas na branch `web`.
5. Não apague nem renomeie `main` até confirmar que as duas branches existem no remoto e que cada uma contém a aplicação esperada.

**Observação:** o ZIP usado para esta revisão contém o núcleo Rust e o app desktop, mas não inclui o código da aplicação web. Portanto, a branch `web` precisa ser criada a partir do commit que já contém a versão web funcional no seu repositório.
