#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/desktop"
if ! command -v npm >/dev/null 2>&1; then
  echo "Erro: npm não encontrado. Instale nodejs e npm pelo pacman." >&2
  exit 1
fi
npm install
npm run app:dev
