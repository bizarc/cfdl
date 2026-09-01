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
  "results_version": "0.12",
  "model_hash": "…",
  "ledger_hash": "…",
  "engine": { "name": "cfdl-engine", "version": "…" },
  "warnings": [],
  "inputs": { "resolved": { "rent_psf": 2.0, … } },
  "deterministic": { "status": "ok", "metrics": { … }, "series": { … } },
  "scenarios": { "status": "not_run", "summaries": [] },
  "monte_carlo": { "status": "ok", "trials": 500, "seed": 42,
                   "metrics": { … }, "trial_summaries": [ … ] },
  "statements": { "statements": [ { "id": "by_entity", "default": true, … } ] },
  "slices": [ { "id": "artist_a", "selection": { … }, "net": { … }, … } ],
  "graph": { "entities": [ { "symbol": "asset.co", "family": "asset", … } ] }
}
```

- **Provenance first**: `model_hash` identifies the model — a hash of the
  compiled IR *without* its `views` (slices and statements), so two users who
  look at identical results differently share a model hash. A declared
  `metric` is a figure the model claims, so it does move `model_hash`.
  `ledger_hash` covers what actually happened — the journal, the transitions
  and the series — and `engine.version` names the exact engine. The run
  configuration is in neither hash. Store all of it next to any number you
  publish.
- `inputs.resolved` — optional; every assumption at its resolved value.
- `deterministic.metrics` — flat map of core, domain (with `--pack`), and
  model-declared (`metric.*`) metrics; money metrics carry `amount` +
  `currency`.
- `deterministic.series` — every stream per period, each attributed to its
  owning entity and category, plus `model.net_cash_flow`, per-entity nets,
  and each declared account's balance as the non-cash series
  `account.<name>`. A transition log and a journal of every causal act — with
  an act's own consequences nested under it as `children` — accompany them.
- `statements` — every rendered statement; a model that declares none gets
  its entity hierarchy, marked `"default": true`.
- `slices` — each declared slice's selection, matched streams, net series,
  and figures.
- `graph` — the model's entity graph: symbols, families, types, `part of`.
- `monte_carlo` — each trial summary carries the full metric map, declared
  metrics included; `monte_carlo.metrics` summarizes each across trials with
  mean, stdev, min/max, percentiles p01–p99, and `trials`, the count of
  trials that published that name.
- The [Python SDK](/docs/python-sdk) exposes the series as
  `results.cashflows()` (wide, PeriodIndex) and `results.annual()`.

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
