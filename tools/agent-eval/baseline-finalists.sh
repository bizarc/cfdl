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

# Say it on the terminal even when stdout and stderr are redirected to a log.
# Without this, a backgrounded run that refuses to start (no key, bad path)
# prints its reason into the log and shows the user an empty prompt — which
# looks exactly like a command that silently did nothing.
announce() {
  printf '%s\n' "$*" >&2
  # The tty write is best-effort: a cron or sandboxed run has no terminal,
  # and its failure must not become noise in the log.
  { printf '%s\n' "$*" > /dev/tty; } 2>/dev/null
  return 0
}

if [ -z "${OPENROUTER_API_KEY:-}" ]; then
  announce "agent-eval: OPENROUTER_API_KEY is not set in this shell — nothing was run."
  announce "  export OPENROUTER_API_KEY=\"your-key\"   then run this again"
  exit 1
fi
cd "$(dirname "$0")/../.." || exit 1
mkdir -p eval-results

# Keep the Mac awake for the duration; released when this script exits.
caffeinate -i -w $$ &

if [ "$#" -gt 0 ]; then
  MODELS=("$@")
else
  # Cheapest first. Chosen on MEASURED spend, not list price: the smoke runs
  # showed grok-4.6 costing 5x gpt-5.6-sol per task ($0.68 vs $0.138) while
  # scoring below it, because a slower loop re-sends a growing conversation —
  # a per-token price says nothing about what a task costs. grok and kimi are
  # cut on that evidence; name them as arguments to run them anyway.
  MODELS=(
    "z-ai/glm-5.3-flash"
    "openai/gpt-5.6-sol"
    "google/gemini-3.7-flash"
  )
fi

announce "agent-eval: running ${#MODELS[@]} model(s), 112 tasks each. Progress: tail -f eval-results/baseline.log"

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
announce "agent-eval: baseline complete."
echo "--- baseline complete ($(date)); compare with:"
echo "    .venv/bin/python tools/agent-eval/compare.py"
