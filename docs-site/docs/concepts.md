---
id: concepts
title: How CFDL Works
slug: /concepts
---

# How CFDL Works

CFDL (Cash Flow Domain Language) is a small, declarative language for
modeling cash flows. You declare *what* the deal is — entities, contracts,
streams, assumptions, time — and the engine derives the period-by-period
cash flows and metrics. There is no imperative code path to debug: the same
model always produces the same results, byte for byte.

## The pipeline

Every surface (CLI, Python, playground, API server) runs the same pipeline:

```
model.cfdl ──compile──▶ IR JSON ──run──▶ Results JSON
             (lex → parse → resolve →      (schedule evaluation,
              validate → lower)             scenarios, Monte Carlo,
                                            metrics)
```

1. **Compile** — your `.cfdl` sources are lexed and parsed with full source
   spans, imports and symbols are resolved, the model is validated
   (structure, types, schedules, pack terms), and the result is lowered into
   a canonical **IR** (intermediate representation) JSON document. Same
   sources + same pack version + same compiler version ⇒ identical IR.
2. **Run** — the engine evaluates every stream over the model's time grid,
   applies scenarios and Monte Carlo trials from the run config, and emits a
   **Results** JSON document: per-stream cash flows, core metrics
   (NPV/IRR/MOIC/payback/WAL), pack domain metrics, and Monte Carlo
   summaries with percentiles.

Both documents have published JSON Schemas — see the
[IR schema](language-reference/ir-schema) and
[Results schema](language-reference/results-schema).

## What's in a model

- **Time** — one calendar declaration (`time calendar monthly from 2026-01
  for 72`) defines the period grid everything else lands on.
- **Entities** — the things that own cash flows (`entity legal borrower`,
  `entity real_estate property`).
- **Streams** — dated cash flow series with a schedule and an amount
  expression. The lowest-level building block.
- **Contracts** — pack-templated bundles of streams declared with business
  terms (`contract cre.lease { terms { base_rent = 25000 } }`). The compiler
  expands them into streams using the pack's rules.
- **Assumptions** — named inputs, fixed (`assume rate = 0.10`) or stochastic
  (`assume growth ~ Normal(mean=0.02, stdev=0.01)`), referenced from
  expressions as `inputs.<name>`.
- **Expressions** — bare, Excel-familiar formulas
  (`base_rent * pow(1 + escalation, years)`) with decimal money math and a
  financial function library (`pmt`, `npv`, `year_frac`, `eomonth`, …). See
  the [expression environment](language-reference/expression-environment).

## Packs

[Domain packs](/packs) (energy, CRE, credit, opco) supply contract
templates, term validations, and industry metrics, so models read like term
sheets instead of formula collections. Every pack is gated by a
[benchmark parity suite](/benchmarks) against independent reference models.

## Determinism and reproducibility

- Compilation is deterministic: IR arrays are canonically ordered and IDs
  derive from stable keys.
- Monte Carlo is deterministic: every run declares an explicit seed, and
  each assumption gets its own seeded draw stream — adding an assumption
  never reshuffles another's draws. Runs are byte-reproducible across
  machines and across surfaces (the CLI, Python SDK, browser playground,
  and API server embed the same compiler and engine).
- Results carry the model hash and engine version, so any output can be
  traced back to exactly what produced it.

## Where each surface fits

| Surface | Use it when… | Setup |
|---|---|---|
| [Playground](/playground) | trying the language, sharing a snippet | none — runs in your browser |
| CLI | files-and-git workflows, CI, golden testing | [Install the CLI](install/cli) |
| [Python SDK](python-sdk) | notebooks, pandas analysis, charts | [Install for Python](install/python) |
| [API server](api-server) | integrating CFDL into another product | [Run the server](install/api-server) |
| VS Code | authoring models with diagnostics + hover | [VS Code and LSP](install/vscode) |

Next: [Getting Started](getting-started) walks you through your first model.
