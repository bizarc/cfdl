#!/bin/bash
# CFDL agent-eval FULL baseline: all 112 public tasks (70 repair + 42
# transcribe) per model. Requires OPENROUTER_API_KEY in the environment.
#
#   cd ~/Documents/cfdl
#   export OPENROUTER_API_KEY="..."
#   nohup bash tools/agent-eval/baseline-finalists.sh > eval-results/baseline.log 2>&1 &
#
# Runs the three finalists by default, cheapest first, so the value bet is
# measured before a dollar goes to the premium models. Name models as
# arguments to run a subset:
#
#   bash tools/agent-eval/baseline-finalists.sh z-ai/glm-5.3-flash
#
# Cost controls: CFDL_EVAL_MAX_COST caps ONE task (default $1.50 — a task
# that spends more has lost the plot). Every scorecard records observed
# spend, and this script prints a per-model total as it goes.
set -u
[ -n "${OPENROUTER_API_KEY:-}" ] || { echo "export OPENROUTER_API_KEY first"; exit 1; }
cd "$(dirname "$0")/../.." || exit 1
mkdir -p eval-results

# Keep the Mac awake for the duration; released when this script exits.
caffeinate -i -w $$ &

if [ "$#" -gt 0 ]; then
  MODELS=("$@")
else
  MODELS=(
    "z-ai/glm-5.3-flash"
    "x-ai/grok-4.6"
    "openai/gpt-5.6-sol"
  )
fi

for M in "${MODELS[@]}"; do
  SLUG=$(echo "$M" | tr '/:' '--')
  OUT="eval-results/baseline-$SLUG.json"
  echo "=== $M  ($(date '+%H:%M:%S'))"
  CFDL_EVAL_MODEL="$M" .venv/bin/python tools/agent-eval/runner.py \
    --tier all \
    --agent 'cmd:.venv/bin/python tools/agent-eval/agents/openrouter.py' \
    --out "$OUT" || echo "!!! $M exited nonzero; continuing"
  if [ -f "$OUT" ]; then
    .venv/bin/python - "$OUT" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
total = sum(t.get("cost_usd", 0) for t in d["summary"].values())
print(f"    spend so far on this model: ${total:.2f}")
PY
  fi
done
echo "--- baseline complete ($(date)); compare with:"
echo "    .venv/bin/python tools/agent-eval/compare.py"
