---
id: concepts
title: How CFDL works
slug: /docs/concepts
description: "What CFDL is and how it works — you declare the deal, the engine derives the cash flows, and the same model always gives the same answer."
generated: none
---

# How CFDL works

CFDL (Cash Flow Domain Language) is a small, declarative language for
modeling cash flows. You declare *what* the deal is — entities, contracts,
streams, assumptions, time — and the engine derives the period-by-period
cash flows and metrics. There is no imperative code path to debug: the same
model always produces the same results, byte for byte.

## The pipeline

Every surface (CLI, Python, playground, API server) runs the same pipeline:

![The CFDL pipeline: model.cfdl is compiled to IR JSON, which the engine runs to produce Results JSON.](/diagrams/pipeline.svg)

1. **Compile** — your `.cfdl` sources are lexed and parsed with full source
   spans, imports and symbols are resolved, the model is validated
   (structure, types, schedules, pack terms), and the result is lowered into
   a canonical **IR** (intermediate representation) JSON document. Same
   sources + same pack version + same compiler version ⇒ identical IR.
2. **Run** — the engine evaluates every stream over the model's time grid,
   applies scenarios and Monte Carlo trials from the run configuration, and emits a
   **Results** JSON document: per-stream cash flows, core metrics
   (NPV/IRR/MOIC/payback/WAL), pack domain metrics, and Monte Carlo
   summaries with percentiles.

Both documents have published JSON Schemas — see the
[IR schema](/docs/specification/ir-schema) and
[Results schema](/docs/specification/results-schema).

## What a model contains

- **Time** — one calendar declaration (`time calendar monthly from 2026-01
  for 72`) defines the period grid everything else lands on.
- **Entities** — the things a model is about. An `asset` produces or consumes
  cash (`entity asset tower : CRE.Asset.RealProperty`); a `party` contracts,
  owns or lends (`entity party acme : CRE.Party.Tenant`). A type is checked
  against the active pack's vocabulary. See [the object
  model](/docs/object-model).
- **Lifecycles** — a declared finite state machine: enumerated states and
  guarded edges, evaluated each period the entity is in the edge's from-state.
  A unit goes delinquent because last period's rent came in short, and cures
  when it resumes — the topology walked as often as the deal's history walks
  it. A model declares one with a `lifecycle` block, or a pack declares it on
  the type.
- **Streams** — dated cash flow series with a schedule and an amount
  expression. The lowest-level building block.
- **Contracts** — pack-templated bundles of streams declared with business
  terms (`contract cre.lease { terms { rent = 25000 } }`). The compiler
  expands them into streams using the pack's rules.
- **Events and options** — an event is a condition and a one-time change
  (`event expiry when time.t >= 24 { set entity asset.suite.status = "vacant" }`);
  an option is a contract with an exercise condition and a payoff. A regime
  that returns is a lifecycle edge, not an event. Transitions from all three
  are recorded in results as a transition log.
- **Accounts** — declared cash locations whose balances accumulate across
  periods: a reserve funded to a target, proceeds waiting for a quarterly
  distribution date. A waterfall draws one with `from <account>`, a step pays
  into one, and logic reads its settled balance as `prev.<account>`.
- **Assumptions** — named inputs, fixed (`assume rate = 0.10`) or stochastic
  (`assume growth ~ Normal(mean=0.02, stdev=0.01)`), referenced from
  expressions as `inputs.<name>`.
- **Expressions** — bare, Excel-familiar formulas
  (`base_rent * pow(1 + escalation, years)`) with decimal money math and a
  financial function library (`pmt`, `year_frac`, `eomonth`, `macrs_rate`, …). See
  the [expression environment](/docs/specification/expression-environment).

## Packs

[Domain packs](/docs/packs) (energy, CRE, credit, opco) supply contract
templates, term validations, and industry metrics, so models read like term
sheets instead of formula collections. Every pack is gated by a
[benchmark parity suite](/docs/benchmarks) against independent reference models.

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
| CLI | files-and-git workflows, CI, golden testing | [Install the CLI](/docs/install/cli) |
| [Python SDK](/docs/python-sdk) | notebooks, pandas analysis, charts | [Install for Python](/docs/install/python) |
| [API server](/docs/api-server) | integrating CFDL into another product | [Run the server](/docs/install/api-server) |
| VS Code | authoring models with diagnostics + hover | [VS Code and LSP](/docs/install/vscode) |

Next: [Getting started](/docs/getting-started) walks you through your first model.
