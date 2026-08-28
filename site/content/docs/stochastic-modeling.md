---
id: stochastic-modeling
title: Stochastic modeling
slug: /docs/stochastic-modeling
description: "Get the deterministic number and the distribution around it from one model, with seeded draws that reproduce byte for byte."
generated: none
---

# Stochastic modeling

A CFDL model gives you the deterministic number and the distribution around
it from the same file. Draws are seeded, so a run reproduces byte for byte.

## Declaring uncertainty

Any assumption can be a distribution instead of a constant:

```cfdl
assume discount_rate = 0.10
assume rent_growth ~ Normal(mean=0.03, stdev=0.01, clip=[-0.02, 0.08])
```

Supported distributions: `Normal(mean, stdev, clip?)`,
`LogNormal(mu, sigma, clip?)`, `Uniform(min, max)`,
`Triangular(min, mode, max)`. Expressions reference stochastic values the
same way as constants, via `inputs.<name>`.

## Running Monte Carlo

```cfdl
run monte_carlo trials 20000 seed 42
```

Every Monte Carlo run declares an explicit seed. Each assumption gets its own
deterministic draw stream, so **adding a new assumption never reshuffles
another assumption's draws** — results are reproducible byte-for-byte across
machines and runs. The run configuration can override or add distributions without
touching the model.

## Scenario-consistent branching

Because draws are ordinary values, expressions can branch on them —
producing coherent, binary outcomes per trial rather than expected-value
blends:

```cfdl
// Per trial: either the tenant renews (renewal rent, no downtime)
// or the space rolls (market rent after downtime and re-lease costs).
amount = if(inputs.renewal_draw < 0.70, renewal_rent, market_rent)
```

An expected-value blend hides the bimodal shape of outcomes like lease
rollover; per-trial branching preserves it.

## Dispersion inside one period: quantiles

Distributions spread a value *across trials*. Some economics depend on the
spread *within a single period* — a battery earns the gap between a month's
most and least expensive hours; overage rent is an option on sales above a
breakpoint. A `quantile` declares that within-period distribution as a value
per cumulative share:

```cfdl
quantile prices linear {
  0.00:  11.0
  0.50:  28.0
  0.98: 340.0
  1.00: 512.0
}
```

Three functions read it: `quantile_mean("prices", 0.98, 1.0)` averages a
slice (the top 2% of hours), `quantile_at` reads one point, and
`quantile_of` inverts it — the share of hours below a threshold. A nonlinear
payoff evaluated at a point estimate is wrong even when the point estimate
is right; when the payoff bends, feed it the distribution it bends over. The
two compose: the quantile carries the within-period shape, and a distributed
assumption multiplying the read carries the across-trials uncertainty about
its level.

## What results carry

Monte Carlo results summarize each metric with `mean`, `stdev`, `min`/`max`,
and percentiles (`p01` through `p99`, including `p05`/`p25`/`p50`/`p75`/`p95`)
— see the [Results schema](/docs/specification/results-schema). The
[Python SDK](/docs/python-sdk) exposes them via `results.monte_carlo()`.
