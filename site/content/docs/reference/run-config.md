---
id: reference-run-config
title: Run-Config Reference
slug: /docs/reference/run-config
generated: none
---

# Run-Config Reference

The run-config JSON controls how compiled IR is evaluated — discount rate,
as-of date, parameter overrides, named scenarios, and Monte Carlo. Pass it
with `cfdl run --config run.json`, `cfdl_sdk.run(config=...)`, or the API
server's `config` field.

## Full shape

```json
{
  "deterministic": {
    "annual_discount_rate": 0.10,
    "as_of": "2026-01-01",
    "parameters": {
      "stream.cre.lease.base_rent:amount": 1000.0
    }
  },
  "scenarios": {
    "stress": {
      "annual_discount_rate": 0.12,
      "parameters": {
        "stream.cre.lease.base_rent:amount": 800.0
      }
    }
  },
  "monte_carlo": {
    "trial_count": 1000,
    "seed": 12345,
    "distributions": {
      "cfg.exit_multiple": { "kind": "normal", "mean": 8.0, "stddev": 1.0 },
      "cfg.growth": { "kind": "uniform", "min": 0.01, "max": 0.05 }
    }
  }
}
```

Every section is optional; an empty config runs deterministically with the
CLI's `--rate` fallback (default 0.0).

## `deterministic`

- `annual_discount_rate` — the discount rate for NPV (annual; the engine
  publishes `run.annual_discount_rate` and `run.periods_per_year` back into
  the metrics).
- `as_of` — valuation as-of date (`YYYY-MM-DD`).
- `parameters` — override map applied before evaluation.

### Parameter keys

- `stream.<dotted_stream_name>:amount` — override a stream's amount.
- `cfg.<path>` — set a value readable in expressions via `cfg.<path>`.

## `scenarios`

Named variants of the deterministic run. Each scenario may override the
discount rate, as-of date, and parameters; results carry one entry per
scenario alongside the base deterministic result.

## `monte_carlo`

- `trial_count` and `seed` are required — there are no implicit seeds.
- `distributions` adds or overrides distributions at run time without
  touching the model. Supported `kind`s here: `fixed { value }`,
  `normal { mean, stddev }`, `uniform { min, max }`.
- In-language `assume <x> ~ …` distributions (which also include
  `LogNormal` and `Triangular`) are sampled per-trial automatically; the
  run-config layer is for overrides and experiment knobs.

Each assumption draws from its own deterministic stream derived from the
seed — results are byte-reproducible, and adding one assumption never
changes another's draws.

## Precedence

Run-config values override CLI fallbacks (`--rate`, `--as-of`). Scenario
values override `deterministic` values for that scenario only.

## Where results land

`deterministic.metrics`, one `scenarios.<name>` block per scenario, and
`monte_carlo.metrics` with mean/stdev/min/max and percentiles per metric —
see the [Results schema](/docs/language-reference/results-schema).
