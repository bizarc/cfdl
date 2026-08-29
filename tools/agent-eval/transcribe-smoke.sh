#!/bin/bash
# CFDL agent-eval transcribe smoke: 3 authoring tasks per model via OpenRouter.
# Requires OPENROUTER_API_KEY in the environment.
set -u
[ -n "${OPENROUTER_API_KEY:-}" ] || { echo "export OPENROUTER_API_KEY first"; exit 1; }
cd "$(dirname "$0")/../.." || exit 1
mkdir -p eval-results
# Paid, fast routes first; the free-tier model last — its provider queue can
# stall for long stretches and should not gate the signal.
MODELS=(
  "z-ai/glm-5.3-flash"
  "google/gemini-3.7-flash"
  "x-ai/grok-4.6"
  "openai/gpt-5.6-sol"
  "moonshotai/kimi-k3"
  "inclusionai/ling-3.0-flash-fin:free"
)
for M in "${MODELS[@]}"; do
  SLUG=$(echo "$M" | tr '/:' '--')
  echo "=== $M"
  CFDL_EVAL_MODEL="$M" .venv/bin/python tools/agent-eval/runner.py \
    --tier transcribe --cases credit/level_pay_pool,credit/io_bullet_loan,cre/retail_strip \
    --agent 'cmd:.venv/bin/python tools/agent-eval/agents/openrouter.py' \
    --out "eval-results/tsmoke-$SLUG.json"
done
echo "--- transcribe smoke done; results in eval-results/"
