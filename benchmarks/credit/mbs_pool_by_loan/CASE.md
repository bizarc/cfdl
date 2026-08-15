## The case

A $100m agency mortgage pool — 8% weighted average coupon, 360-month term, 20%
loss severity, twelve-month recovery lag, prepaying at a flat 1% single monthly
mortality against a 1% monthly default rate.

It is the same pool as the mortgage pool conventions case, at a different
grain. There it is one pool. Here it is **four loans of $40m, $30m, $20m and
$10m that belong to a pool**, and the pool itself holds no contract. Every
figure asserted against the pool is an aggregate.

## The reference

The industry's own standard formulas for analysing mortgage-backed securities —
the document that defines CPR, SMM, PSA and SDA — and the complete 176-month
cash flow schedule it publishes for this pool.

**Not redistributable.** The publisher forbids reproduction in any form, so the
source is neither vendored nor quoted; its figures are carried as anchor values
and cited as facts.

The reference publishes four columns: interest, scheduled amortization,
voluntary prepayments and principal recoveries. The pool's cash in a period is
their sum, so the anchors here are the published figures added together. Addition
is the only step taken.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Declared | five typed assets, one of them a parent; four contract instances |
| Language features | **`part of` hierarchy**, typed entity fields, per-instance contract suffixes |
| Conventions | level-pay amortization, SMM on the gross balance, MDR, a lagged recovery |

Two aggregates are asserted, computed by unrelated code:

- `entity.asset.pool.net_cash_flow` — the **hierarchy rollup**, aggregating the
  children a `part of` relation names rather than a matching name prefix.
- `domain.credit.gross_collections` — the **category subtotal**, the pack folding
  four contract instances into one domain line.

Both must reproduce the same published schedule. A defect in either shows as a
divergence between them.

## The result

**25 anchor months on both columns, across a 372-period grid.** Every one agrees
with the published schedule within the tolerance the source's rounding allows.

The rollup is also exact against the single-pool model: over all 372 periods,
`entity.asset.pool.net_cash_flow` here and `model.net_cash_flow` there agree to
**zero** — not within a tolerance, exactly. Splitting $100m into four unequal
loans changes nothing about the pool's cash.

## The delta

Largest residual anywhere: **1.76 dollars**, against a tolerance of 2.01.

It is the source's rounding, not arithmetic. Each published figure is given to
the whole dollar and up to four are added, so two dollars bounds the difference
before any model is run.
