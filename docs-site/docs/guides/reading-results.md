---
id: guide-reading-results
title: Reading Results & IR
slug: /guides/reading-results
---

# Reading Results & IR

Two JSON documents come out of the pipeline; both have published schemas
and stable, additive-intent shapes.

## Results anatomy

```json
{
  "results_version": "0.2",
  "model_hash": "…",
  "engine": { "name": "cfdl-engine", "version": "…" },
  "warnings": [],
  "deterministic": { "status": "ok", "metrics": { … }, … },
  "scenarios": { "<name>": { … } },
  "monte_carlo": { "status": "ok", "trials": 500, "seed": 42, "metrics": { … } }
}
```

- **Provenance first**: `model_hash` ties results to the exact IR;
  `engine.version` to the exact engine. Store both next to any number you
  publish.
- `deterministic.metrics` — flat map of core (and, with `--pack`, domain)
  metrics; money metrics carry `amount` + `currency`.
- `monte_carlo.metrics` — per-metric summaries: mean, stdev, min/max,
  percentiles p01–p99.
- Per-period stream series and annual rollups accompany the metrics; the
  [Python SDK](/python-sdk) exposes them as `results.cashflows()` (wide,
  PeriodIndex) and `results.annual()`.

## IR anatomy

The IR is the canonical compiled model: entities, streams (with lowered
schedule + expression slots, `lang: "cfdl"`), curves, assumptions, run
declarations — deterministically ordered with stable IDs. Useful habits:

- **Commit it**: IR diffs show exactly what a model change did.
- **Inspect pack lowering**: contracts appear as the streams they expanded
  into.

## Schemas

Machine-readable schemas ship with the repo and the docs site:
[IR schema](/language-reference/ir-schema) ·
[Results schema](/language-reference/results-schema). Both freeze as v1 at
launch with an additive-only policy after that.

## Reference links

- [Run-Config Reference](/reference/run-config)
- [Python SDK](/python-sdk)
