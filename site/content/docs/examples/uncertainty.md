---
id: example-uncertainty
title: "Uncertainty and Monte Carlo"
slug: "/docs/examples/uncertainty"
---

> Generated from `examples/language_tutorial/uncertainty/`.

An assumption can be a **distribution** rather than a number, which is what
turns one model into a range of outcomes.

`assume growth ~ Normal(...)` is sampled per trial. A deterministic run uses the
distribution's mean, so the same model answers both questions without being
rewritten.

`run.json` asks for 500 trials with a fixed seed, so the run reproduces exactly.
The results carry percentiles alongside the deterministic figures.

## model.cfdl

```cfdl run={"deterministic":{"annual_discount_rate":0.1},"monte_carlo":{"trials":500,"seed":42}}
version 0.1
model "tutorial-uncertainty"
time calendar monthly from 2026-01 for 24

entity legal co

// A fixed assumption names a number once.
assume monthly_cost = 18000

// A stochastic assumption names a DISTRIBUTION. A deterministic run uses
// its mean; a Monte Carlo run samples it, seeded, so the same run
// reproduces exactly.
assume growth ~ Normal(mean=0.02, stdev=0.008, clip=[0.0, 0.05])

stream co.revenue on entity legal.co inflow currency USD {
  schedule every month from 2026-01 to 2027-12
  amount = 40000 * pow(1 + inputs.growth, time.t / 12.0)
}

stream co.cost on entity legal.co outflow currency USD {
  schedule every month from 2026-01 to 2027-12
  amount = inputs.monthly_cost
}
```
