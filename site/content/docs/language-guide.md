---
id: language-guide
title: "Language guide"
slug: "/docs/language-guide"
description: "A tour of the language: declare what a deal is — the parties, the money, the timing, the basis — and let the engine derive the cash."
generated: none
---

# Language guide

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

To value cash beyond the modeled horizon — a terminal value struck off a
forward year — extend the grid without extending the cash:

```cfdl
time calendar monthly from 2026-01 for 120 project 12
```

## Entities

An entity is something that holds cash: a property, a borrower, a project, a
fund.

```cfdl
entity asset tower : CRE.Asset.RealProperty
entity party acme  : CRE.Party.Tenant
```

The first word is the entity's **family** — `asset` for something that produces
or consumes cash, `party` for someone who contracts, owns or lends. The second
is its name, and what follows the colon is its **type**, checked against the
active pack's vocabulary. A model with no pack still has the base types:
`Asset.Real`, `Asset.Financial`, `Asset.Intangible` and `Party`.

Every stream belongs to an entity, so cash totals per entity as well as per
model. Where a model declares hierarchy with `part of`, a parent's total
includes its children.

An asset's type may carry a **lifecycle** — a closed set of states it moves
through, with events that move it between them, and contracts and streams that
switch on where it is. See [the object model](/docs/object-model).

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

A curve is a dated series — a forward price, an index — declared once and read
by date:

```cfdl
curve power_price linear {
  2026-01: 42.10
  2027-01: 44.75
  2028-01: 46.20
}
```

```cfdl
amount = 1200 * curve_value("power_price", time.date)
```

The mode says how a value between two stated points is found: `linear`
interpolates, `step` holds the last point forward.

`obs.*` is a different thing — observations supplied at run time rather than
declared in the model.

See [Curves](/docs/guides/curves).

## Fields that move

Some quantities depend on the period before: a loan balance, a survival factor,
a reserve. Those belong to the thing they describe, so they are declared on it.

```cfdl
entity asset loan : Credit.Asset.Tranche {
  seniority = 1

  balance init 1000000
          next prev * (1 - 0.01)
}
```

`init` is the value at period zero; `next` computes each later period, with
`prev` bound to the field's own previous value. Both take an expression
directly, with no `=`, the way `schedule` and `active when` do. A field with no
`next` simply holds.

Read it by naming the thing:

```cfdl
amount = asset.loan.balance * 0.005
```

`prev.asset.loan.balance` reads the close before this one — which is how a debt
schedule charges interest on the average of a period's opening and closing
balance without declaring the quantity twice.

A field is **not cash**: it never reaches the model's total. It exists so a
stream, waterfall or event can read it.

Writing the recurrence directly is often the only way to match a published
figure exactly, because a source that escalates an already-rounded number each
year is not computing a power of its base.

**A field needs a typed entity.** An untyped `entity asset thing` takes no
block, so a value that moves needs something with a declared type to belong to.

## Waterfalls

Some cash is not earned, it is **allocated**. A securitization pays its tranches
in strict order; a fund returns capital before it pays carry. A `waterfall`
declares that order over a pot:

```cfdl
waterfall deal.distribution on entity asset.trust {
  schedule every month from 2026-01 to 2030-12
  from available

  pay servicing to party.servicer    = 12500.0
  pay senior    to asset.class_a     = 6250.0
  pay residual  to party.certificate = remaining
}
```

Every step is `pay <name> to <payee> = <expr>`, and a step takes what it asks
for or what is left, whichever is smaller. `remaining` is what survives the
steps above; `paid.<step>` and `owed.<step>` read what an earlier step did.

A waterfall runs after the period's fields and streams, so it shares out money
that already exists. Its steps publish as series, so a waterfall declared later
can draw on an earlier one's payment as its own pot — a fund's carry becoming a
management company's. See [Waterfalls](/docs/guides/waterfalls).

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
| [Examples](/docs/examples) | Eight short lessons, then worked deals checked against published sources |
| [How-to guides](/docs/guides/schedules-and-calendars) | Schedules, packs, scenarios, curves, metrics |
| [Reference](/docs/reference) | Expressions, contracts, metrics, statements, diagnostics |
| [Specification](/docs/specification) | The normative definition, for implementers |
