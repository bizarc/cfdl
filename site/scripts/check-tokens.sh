#!/usr/bin/env bash
# Design-system guard: components must consume semantic tokens, never raw
# color values. tokens.css is the single place a hex literal may appear.
set -euo pipefail

cd "$(dirname "$0")/.."

violations=$(
  grep -rnE '#[0-9a-fA-F]{3,8}\b' \
    --include='*.tsx' --include='*.ts' --include='*.css' \
    app components lib 2>/dev/null \
    | grep -v 'app/tokens.css' \
    | grep -v '^\s*\*' \
    || true
)

if [[ -n "$violations" ]]; then
  echo "Raw color literals found outside app/tokens.css:" >&2
  echo "$violations" >&2
  echo >&2
  echo "Add a semantic token in app/tokens.css and use it instead." >&2
  exit 1
fi

echo "check-tokens: OK (no raw color literals outside tokens.css)"
