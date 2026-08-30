# Notes

Not published. Working record for this case.

## The source

| | |
|---|---|
| Model | Basic Real Estate Model, v1.2 |
| Publisher | Adventures in CRE (A.CRE) |
| Local copy | `research/CRE Case Examples/Basic-Real-Estate-Model-v1.2.xlsx` |
| Sheet | `Sheet1`, the only sheet |

`research/` is gitignored, so the workbook is local material and not repo
content. The publisher states no redistribution terms — the catalogue entry
records them as "not stated", which is not a grant — so it is not vendored
and the case asserts against it.

## Why this case exists

It takes `cre.revenue_line` and `cre.exit_cap` off the unexercised list. Both
needed a source with income stated at the property level rather than the lease
level, and a disposition struck as a stated NOI over a stated cap rate — which
is what a stabilized single-tenant teaching model is.

## What was read

Rows 6–9 for income, 12–18 for expenses and NOI, 23 for the reserve, 26 for
cash flow from operations, 43 for the unlevered cash flow, and 48–50 for the
returns. Purchase price is `U12` (`=G18/0.06`), the disposition is `P37:P39`
(`=P18/0.065`, less 2%).

Nothing is derived. Every figure asserted is the workbook's own, and the
escalation is stated in its assumptions block rather than inferred from the
rows — all ten years then reproduce from the year-one amounts.

## The two ends of the hold

`schedule on <date>` settles at the period's OPEN; a one-period `every` is an
ordinary annuity and settles at its CLOSE. That is the whole timing story:

| flow | placement | years |
|---|---|---|
| purchase | period 0 open | 0 |
| year 1 operations | period 0 close | 1 |
| year 10 operations | period 9 close | 10 |
| sale and selling costs | period 9 close | 10 |

Moving the purchase to the period close moves NPV by 92,764, which is the
mutation below.

## The equity multiple, and what model.moic is

The case asserts the published 1.851981 through a declared metric on the
valuation plane:

```cfdl
metric invested        = 0.0 - series_sum("cre.acquisition.purchase", 0, 9)
metric returned        = model.total + metric.invested
metric equity_multiple = metric.returned / metric.invested
```

Nothing in the language or the engine had to change for that. `metric` already
folds once at the horizon over the finished projection and may read series and
`model.*`, which is exactly where a multiple belongs — the model states what it
counts as invested capital rather than having the engine guess.

`model.moic` gives 1.905005 on the same deal. Its own comment describes it as a
ratio of cash in to cash out over the life; the code sums the model's
net-positive periods over its net-negative ones, which is a different quantity
whenever one period holds both. Here period 0 holds the purchase at its open
and the first year's cash at its close:

```
-1,417,958.33 + 83,077.50 = -1,334,880.83
2,542,954.53 / 1,334,880.83 = 1.905005
2,626,032.03 / 1,417,958.33 = 1.851981   <- the published figure
```

Because it partitions periods, the figure also moves with the grain: on a
monthly calendar the purchase would sit alone in month 0 and the same deal
would read differently. Noted at `docs/13` §7.84. It is an observation about
one engine metric, not a blocker — this case asserts the multiple it wanted.

## Three native streams, by design

The acquisition price, the capital reserve and the selling costs are native
streams. `cre` has no acquisition contract and does not need one: each of these
is a single dated amount with a category, which the core language already
states directly. Adding a pack contract would rebuild the same machinery inside
the pack for a concept the language expresses without it.

Worth recording that `cre` has the disposal half of the acquire/dispose pair
(`cre.exit`, `cre.exit_cap`, `cre.exit_forward`) and not the acquiring half,
because the disposal side carries real pack logic — a cap rate applied to a
stated or derived NOI — and a purchase price does not.

## Mutation testing

Each applied to a copy, compared against the baseline npv 90853.72729 and irr
0.078484. All eight caught.

| mutation | npv | irr |
|---|---|---|
| PGI escalation 0.03 → 0.035 | 107,152.84490 | 0.079977 |
| exit cap 0.065 → 0.060 | 163,200.20013 | 0.084794 |
| exit NOI −1,000 | 83,032.96895 | 0.077779 |
| vacancy rate 0.10 → 0.11 | 81,547.10866 | 0.077615 |
| purchase price −10,000 | 100,853.72729 | 0.079460 |
| purchase moved to period close | 183,617.35657 | 0.090582 |
| management fee 3,172.5 → 3,200 | 90,635.91281 | 0.078463 |
| capital reserve 2,000 → 2,100 | 90,061.67464 | 0.078410 |

The placement mutation is the one that matters: it is the largest single move
in the set, which is the right result for a case whose subject is where the
two ends of the hold sit.
