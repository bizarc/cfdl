#!/bin/bash
# CFDL agent-eval FULL baseline for the three finalists: all 112 public tasks
# (70 repair + 42 transcribe) per model, serial. An overnight run — expect
# roughly 3-6 hours per model. Requires OPENROUTER_API_KEY in the environment.
#
#   cd ~/Documents/cfdl
#   export OPENROUTER_API_KEY="..."
#   nohup bash tools/agent-eval/baseline-finalists.sh > eval-results/baseline.log 2>&1 &
#
# Safe to close the terminal after launching. Progress:
#   tail -f eval-results/baseline.log
set -u
[ -n "${OPENROUTER_API_KEY:-}" ] || { echo "export OPENROUTER_API_KEY first"; exit 1; }
cd "$(dirname "$0")/../.." || exit 1
mkdir -p eval-results

# Keep the Mac awake while this runs (releases automatically when done).
caffeinate -i -w $$ &

MODELS=(
  "z-ai/glm-5.3-flash"
  "x-ai/grok-4.6"
  "openai/gpt-5.6-sol"
)
for M in "${MODELS[@]}"; do
  SLUG=$(echo "$M" | tr '/:' '--')
  echo "=== $M ($(date))"
  CFDL_EVAL_MODEL="$M" .venv/bin/python tools/agent-eval/runner.py \
    --tier all \
    --agent 'cmd:.venv/bin/python tools/agent-eval/agents/openrouter.py' \
    --out "eval-results/baseline-$SLUG.json" || echo "!!! $M exited nonzero; continuing"
done
echo "--- baseline complete ($(date)); results in eval-results/"
