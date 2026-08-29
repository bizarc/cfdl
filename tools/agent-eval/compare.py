#!/usr/bin/env python3
"""Build the model comparison table from baseline scorecards.

    .venv/bin/python tools/agent-eval/compare.py [eval-results]

Ranks by transcribe match rate (the authoring signal), then mean partial
credit, and prints observed spend beside each — the score-versus-cost view
the model-partner decision needs.
"""
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "eval-results")
rows = []
for f in sorted(root.glob("baseline-*.json")):
    d = json.loads(f.read_text(encoding="utf-8"))
    s = d["summary"]
    rep, tra = s.get("repair", {}), s.get("transcribe", {})
    cost = sum(t.get("cost_usd", 0.0) for t in s.values())
    rows.append({
        "model": f.stem.replace("baseline-", ""),
        "repair": f"{rep.get('passed',0)}/{rep.get('tasks',0)}",
        "repair_pct": rep.get("passed", 0) / max(rep.get("tasks", 1), 1),
        "transcribe": f"{tra.get('passed',0)}/{tra.get('tasks',0)}",
        "transcribe_pct": tra.get("passed", 0) / max(tra.get("tasks", 1), 1),
        "partial": tra.get("mean_partial", 0.0),
        "cost": cost,
        "results": d["results"],
    })
rows.sort(key=lambda r: (-r["transcribe_pct"], -r["partial"]))

if not rows:
    print(f"no baseline scorecards in {root}/")
    raise SystemExit(1)

print(f"{'model':32s} {'repair':>9s} {'transcribe':>11s} {'partial':>8s} {'spend':>9s} {'$/match':>9s}")
for r in rows:
    matches = max(int(r["transcribe"].split("/")[0]), 1)
    print(f"{r['model']:32s} {r['repair']:>9s} {r['transcribe']:>11s} "
          f"{r['partial']:8.3f} {'$%.2f' % r['cost']:>9s} {'$%.2f' % (r['cost']/matches):>9s}")

# Per-domain transcribe splits: where a model is strong is a pack question.
print("\nTranscribe by domain (matched / attempted):")
domains = sorted({r["id"].split("/")[0] for row in rows for r in row["results"]
                  if r["tier"] == "transcribe" and "/" in r["id"]})
print(f"{'model':32s} " + " ".join(f"{d:>10s}" for d in domains))
for row in rows:
    cells = []
    for d in domains:
        items = [r for r in row["results"]
                 if r["tier"] == "transcribe" and r["id"].startswith(d + "/")]
        got = sum(1 for r in items if r["score"]["matches"])
        cells.append(f"{got}/{len(items)}")
    print(f"{row['model']:32s} " + " ".join(f"{c:>10s}" for c in cells))
