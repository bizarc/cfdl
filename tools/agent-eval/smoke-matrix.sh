#!/bin/bash
# CFDL agent-eval smoke matrix: 3 repair tasks per model via OpenRouter.
# Run from anywhere: bash tools/agent-eval/smoke-matrix.sh
# Requires OPENROUTER_API_KEY in the environment. Results land in
# eval-results/ at the repository root (gitignored).
set -u
[ -n "${OPENROUTER_API_KEY:-}" ] || { echo "export OPENROUTER_API_KEY first"; exit 1; }
cd "$(dirname "$0")/../.." || exit 1
mkdir -p eval-results
MODELS=(
  "inclusionai/ling-3.0-flash-fin:free"
  "google/gemini-3.7-flash"
  "z-ai/glm-5.3-flash"
  "moonshotai/kimi-k3"
  "x-ai/grok-4.6"
  "openai/gpt-5.6-sol"
)
for M in "${MODELS[@]}"; do
  SLUG=$(echo "$M" | tr '/:' '--')
  echo "=== $M"
  CFDL_EVAL_MODEL="$M" python3 tools/agent-eval/runner.py \
    --tier repair --cases missing_time,dup_stream,unknown_time_read \
    --agent 'cmd:python3 tools/agent-eval/agents/openrouter.py' \
    --out "eval-results/smoke-$SLUG.json"
done
echo "--- smoke matrix done; results in eval-results/"
