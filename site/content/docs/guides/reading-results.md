---
id: guide-reading-results
title: Reading results and IR
slug: /docs/guides/reading-results
description: "The two JSON documents a run produces — the canonical IR and the results — and how to read each."
generated: none
---

# Reading results and IR

Two JSON documents come out of the pipeline; both have published schemas
and stable, additive-intent shapes.

## Results anatomy

```json
{
  "results_version": "0.5",
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
- Per-period stream series and annual rollups accompany the metrics, with
  each declared account's balance as the non-cash series `account.<name>`,
  a transition log for lifecycle and event state changes, and a journal of
  every causal act with what became of it; the
  [Python SDK](/docs/python-sdk) exposes them as `results.cashflows()` (wide,
  PeriodIndex) and `results.annual()`.

## IR anatomy

The IR is the canonical compiled model: entities, streams (with lowered
schedule + expression slots, `lang: "cfdl"`), curves, assumptions, run
declarations — deterministically ordered with stable IDs. Useful habits:

- **Commit it**: IR diffs show exactly what a model change did.
- **Inspect pack lowering**: contracts appear as the streams they expanded
  into.

## Schemas

Machine-readable schemas are published here:
[IR schema](/docs/specification/ir-schema) ·
[Results schema](/docs/specification/results-schema). Both freeze as v1 at
launch with an additive-only policy after that.

## Reference links

- [Run-config reference](/docs/reference/run-config)
- [Python SDK](/docs/python-sdk)
