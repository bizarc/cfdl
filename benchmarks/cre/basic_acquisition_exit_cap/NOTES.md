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

## model.moic disagrees with the published equity multiple, and is wrong

The engine reports 1.905005; the source publishes 1.851981. Reconstructed:

```
engine    sum(positive period NCF) / sum(negative period NCF)
          2,542,954.5294 / 1,334,880.8333 = 1.905005
workbook  distributions / contribution
          2,626,032.0294 / 1,417,958.3333 = 1.851981
```

`model.moic` partitions PERIODS by the sign of net cash flow. Period 0 carries
the acquisition at its open and the first year's cash at its close, so they net
inside it and the denominator falls from 1,417,958.33 to 1,334,880.83 — the
first year's income is silently treated as a reduction in capital invested.

This is not specific to this deal. Any acquisition model on an annual grain
puts the purchase and the first year's operations in one period, so any of them
gets an inflated multiple. NPV and IRR are unaffected: both discount a flow by
where it sits, not by the sign of the period containing it, which is why they
are exact here while the multiple is not.

Raised as a capability gap at `docs/13` §7.84.

## Missing cre contracts

Three lines had to be native streams:

- **acquisition price** — `cre` has no acquisition contract, though `opco` does
  (`opco.acquisition`).
- **capital reserve** — `cre` has no capex contract at all.
- **selling costs on a capped exit** — `cre.exit` and `cre.exit_forward` each
  lower a selling-cost leg; `cre.exit_cap` lowers only the sale.

The first two are candidate contracts. The third is an asymmetry inside a
roster that otherwise has the leg, and is the cheapest of the three to close.

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
