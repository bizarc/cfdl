---
id: reference-run-config
title: Run-config reference
slug: /docs/reference/run-config
description: "The run-config JSON: discount rate, as-of date, parameter overrides, named scenarios, and Monte Carlo settings."
generated: none
---

# Run-config reference

The run-config JSON controls how compiled IR is evaluated — discount rate,
as-of date, parameter overrides, named scenarios, and Monte Carlo. Pass it
with `cfdl run --config run.json`, `cfdl_sdk.run(config=...)`, or the API
server's `config` field.

## Full shape

```json
{
  "deterministic": {
    "annual_discount_rate": 0.10,
    "valuation_grain": "annual",
    "arithmetic": "decimal",
    "as_of": "2026-01-01",
    "parameters": {
      "inputs.base_rent": 1000.0,
      "cfg.exit_multiple": 8.0
    }
  },
  "scenarios": {
    "stress": {
      "annual_discount_rate": 0.12,
      "parameters": { "inputs.base_rent": 800.0 }
    }
  },
  "monte_carlo": {
    "trial_count": 1000,
    "seed": 12345,
    "distributions": {
      "inputs.exit_multiple": { "kind": "normal", "mean": 8.0, "stdev": 1.0, "clip": [5.0, 12.0] },
      "inputs.growth": { "kind": "uniform", "min": 0.01, "max": 0.05 }
    }
  }
}
```

Every section is optional; an empty config runs deterministically with the
CLI's `--rate` fallback (default 0.0). The config structs reject unknown
fields, so a misspelled key is a hard error, not a silent default. The
authoritative shape is the
[run schema](/schemas/CFDL_v0_1_Run.schema.json); this page restates it.

## `deterministic`

- `annual_discount_rate` — the discount rate for NPV (annual; the engine
  publishes `run.annual_discount_rate` and `run.periods_per_year` back into
  the metrics).
- `valuation_grain` — `"period"` (default when omitted): each flow discounts
  at its own fractional year on the model grid. `"annual"`: cash sums into
  calendar years and each year discounts once — the convention published
  sources use, and the setting to reach for when reconciling against one.
- `arithmetic` — `"decimal"` (default when omitted): what every published
  number uses. `"excel_compat"` reproduces a spreadsheet's float64
  artifacts, for reconciling against a workbook rather than for producing
  an answer.
- `as_of` — valuation as-of date (`YYYY-MM-DD`).
- `parameters` — override map applied before evaluation.

### Parameter keys

Four key shapes are recognized; anything else is ignored:

- `inputs.<name>` — an assumption. **Prefer this one**: a contract term
  declared as `inputs.<name>` is then drivable by scenarios and Monte Carlo
  alike without touching the model.
- `stream.<name>:amount` — replaces a stream's amount for every period,
  bypassing its expression.
- `cfg.<path>` — a config value read as `cfg.<path>` in expressions.
- `obs.<path>` — an observable read as `obs.<path>` in expressions.

## `scenarios`

Named variants of the deterministic run. Each scenario may override the
discount rate, as-of date, and parameters; results carry one entry per
scenario alongside the base deterministic result, publishing the same
metric map the deterministic block does.

## `monte_carlo`

- `trial_count` and `seed` are required — there are no implicit seeds.
- `distributions` adds or overrides distributions at run time without
  touching the model, keyed by the same shapes as `parameters` and
  overriding an in-language `assume x ~ Dist(...)` of the same name.
  The kinds match the language's: `fixed { value }`,
  `normal { mean, stdev }`, `uniform { min, max }`,
  `log_normal { mu, sigma }`, `triangular { min, mode, max }`. Every kind
  accepts `clip: [lo, hi]`, applied after the draw exactly as the
  language's `clip=[...]` is. `stddev` is a deprecated spelling of `stdev`;
  the language's spelling is `stdev`.

Each assumption draws from its own deterministic stream derived from the
seed — results are byte-reproducible, and adding one assumption never
changes another's draws.

## Precedence

Run-config values override CLI fallbacks (`--rate`, `--as-of`). Scenario
values override `deterministic` values for that scenario only.

## Where results land

`deterministic.metrics`, one scenario summary per scenario, and
`monte_carlo.metrics` with mean/stdev/min/max, percentiles p01–p99, and
`trials` — the count of trials that published that name — for every metric,
model-declared ones included. See the
[Results schema](/docs/specification/results-schema).
