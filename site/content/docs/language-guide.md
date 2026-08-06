---
id: language-guide
title: "Language Guide"
slug: "/docs/language-guide"
generated: none
---

# Language Guide

CFDL is a language for saying what a deal *is*. You declare the terms — who the
parties are, what money moves, when, and on what basis — and the engine derives
the schedule, the cash flows and the metrics.

There is no calculation to write and no order of operations to get right. The
same model always produces the same numbers.

## The shape of a model

Every model declares four things:

```cfdl
version 0.1
model "first-look"
time calendar monthly from 2026-01 for 12

entity asset tower

stream tower.rent on entity asset.tower inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 10000
}
```

A **version**, a **name**, a **timeline**, and at least one **entity** and one
**stream**. That model runs, and reports 120,000 of cash across twelve months.

Try it in the [playground](/playground) — nothing to install.

## Time

The timeline is the grid every amount is evaluated on.

```cfdl
time calendar monthly from 2026-01 for 120
```

`calendar` may be `annual`, `quarterly`, `monthly` or `daily`; `from` is the
first period and `for` the count.

**Choose the grain at which something varies.** If a mortgage's interest changes
every month, the model is monthly. Reporting it annually is a separate question,
answered later by a [statement](/docs/reference/statements) rather than by the
timeline. A schedule finer than the calendar is rejected rather than silently
collapsed, because occurrences inside one period cannot be told apart.

To value cash beyond the modelled horizon — a terminal value struck off a
forward year — extend the grid without extending the cash:

```cfdl
time calendar monthly from 2026-01 for 120 project 12
```

## Entities

An entity is something that holds cash: a property, a borrower, a project, a
fund.

```cfdl
entity asset tower
entity legal borrower
```

The first word is the entity's type, the second its name. Every stream belongs
to one, so cash totals per entity as well as per model.

## Streams

A **stream** is one economic line item over time — the atom everything else is
built from. It says who, which way, in what currency, on what schedule, and how
much.

```cfdl
stream tower.opex on entity asset.tower outflow currency USD {
  schedule every month from 2026-01 to 2035-12
  amount = 4200
}
```

`inflow` and `outflow` set the sign. Values are stored signed — an outflow is
negative — so a period's cash is the sum of its streams and nothing else.

### Schedules

```cfdl
schedule on 2026-03
schedule every month from 2026-01 to 2030-12
schedule every quarter from 2026-01 to 2030-12
schedule every month on day 15 from 2026-01 to 2030-12
```

`on day 15` places the cash *within* its period. Where cash sits inside a period
changes what it is worth, so CFDL asks rather than assuming.

### Amounts that vary

`amount` is an [expression](/docs/reference/expressions) evaluated once per
period. `time.t` is the period index, counting from zero:

```cfdl
amount = 25000 * pow(1.03, time.t / 12.0)
```

That is rent escalating 3% a year on a monthly grid.

### Turning a stream on and off

```cfdl
stream tower.percentage_rent on entity asset.tower inflow currency USD {
  schedule every month from 2026-01 to 2035-12
  active when time.t >= 24
  amount = 1500
}
```

## Assumptions

`assume` names a value once, so the model reads as terms rather than numbers:

```cfdl
assume vacancy_rate = 0.05

stream tower.vacancy on entity asset.tower outflow currency USD {
  schedule every month from 2026-01 to 2035-12
  amount = 10000 * inputs.vacancy_rate
}
```

An assumption can be a **distribution** rather than a number, which is what
turns one model into a range of outcomes:

```cfdl
assume exit_cap ~ Normal(mean=0.065, stdev=0.005, clip=[0.05, 0.08])
```

A deterministic run uses the mean; a Monte Carlo run samples it, seeded, so the
same run reproduces exactly. See
[Stochastic modeling](/docs/stochastic-modeling).

## Curves

A curve is a dated series — a forward price, an index — read through `obs.*`:

```cfdl
curve power_price linear {
  2026-01: 42.10
  2027-01: 44.75
  2028-01: 46.20
}
```

The mode says how values between the stated points are found.

See [Curves](/docs/guides/curves).

## State

Some quantities depend on the period before: a loan balance, a survival factor.
A `state` says so directly.

```cfdl
state balance {
  init = 1000000
  next = prev * (1 - 0.01)
}
```

`init` is period zero; `next` computes each later period from `prev`. A state is
**not cash** — it never reaches the model's total. It exists so a stream can
read it:

```cfdl
amount = state.balance * 0.005
```

Writing the recurrence directly is often the only way to match a published
figure exactly, because a source that escalates an already-rounded number each
year is not computing a power of its base.

## Contracts and packs

Everything so far builds streams by hand. A **pack** lets you declare business
terms instead, and expands them into the streams those terms imply.

```cfdl
use pack "cre" version "0.1.0"

contract cre.lease {
  term 2026-07..2031-12
  terms {
    base_rent = 25000
  }
}
```

That is a lease, not a stream — and the pack turns it into the streams a lease
produces, each already classified so it lands on the right line of a
[statement](/docs/reference/statements).

Declare a contract more than once by giving it a suffix, so the pieces stay
separable in the results:

```
contract cre.lease.suite_200 { ... }
```

Four packs ship today: CRE, credit, energy and operating companies. See
[pack contracts](/docs/reference/packs) for every contract and the terms it
reads.

**Use contracts for what the pack understands and streams for everything else.**
The two mix freely in one model, and a hand-written stream can declare its own
`category` so it is counted in the same subtotals.

## Splitting a model up

Past a certain size, one file stops helping. Any model can be split across a
directory, which the compiler reads whole:

```
deal/
  time.cfdl
  structure.cfdl
  contracts.cfdl
```

Declaration order does not matter. See
[Multi-file models](/docs/guides/multi-file-models).

## What you get back

A run produces a results document containing:

- **`deterministic.series`** — every stream per period, plus `model.net_cash_flow`
- **`deterministic.metrics`** — NPV, IRR, MOIC, payback, weighted average life
- **`statements`** — the pro forma, when the pack declares one
- **`monte_carlo`** — percentiles and trial summaries, when trials were asked for

Read it in the playground, through the [Python SDK](/docs/python-sdk), or as
JSON. See [Reading results](/docs/guides/reading-results).

## Where to go next

| | |
|---|---|
| [Examples](/docs/examples) | Five short lessons, then worked deals reconciled against published sources |
| [Guides](/docs/guides/schedules-and-calendars) | Schedules, packs, scenarios, curves, metrics |
| [Reference](/docs/reference) | Expressions, contracts, metrics, statements, diagnostics |
| [Specification](/docs/specification) | The normative definition, for implementers |
