---
id: guide-metrics
title: Metrics
slug: /docs/guides/metrics
description: "The figures a run reports: the engine's, the active pack's, and the ones the model declares itself."
generated: none
---

# Metrics

A metric's prefix says who minted it: `model.*` is the engine's, `domain.*`
is the active pack's, and `metric.*` is the model's own — a figure declared
with `metric <name> = <expr>`.

## Core metrics (every run)

- `model.npv` — at `run.annual_discount_rate`
- `model.irr` — when the cash flow pattern supports it (solver + tolerance
  documented in the spec)
- `model.moic`, `model.payback_periods` / `model.payback_years`,
  `model.wal_years`, `model.total`
- Per-entity and per-stream totals: `entity.<name>.total`,
  `stream.<name>.total`
- Run facts: `run.annual_discount_rate`, `run.periods_per_year`

## Domain metrics (per pack)

Packs declare metric sets computed when you run with `--pack <name>`
(CLI), `pack=` (Python), or `pack` (API): e.g.
`domain.energy.tax_benefits`, `domain.credit.wal_years`,
`domain.credit.collections_multiple`, CRE NOI/exit metrics, OpCo
EBITDA/exit proceeds. Each pack guide lists its set.

```bash
cfdl run ir.json --packs packs --pack credit --out results.json
```

## Declared metrics (per model)

A model may name the figure it solved for — a number neither the engine nor
a pack mints:

```cfdl
metric gross_revenue = series_sum("ops.revenue", 0, 4)
metric total_cost    = series_sum("ops.cost", 0, 4)
metric margin        = metric.gross_revenue + metric.total_cost
```

A declared metric is evaluated once, at the horizon, over the finished
projection — a fold over completed results, never a recurrence that feeds
back into the walk. It may fold any series the model publishes — a stream by
name, a waterfall step, `entity.<symbol>.net_cash_flow`, `account.<name>`,
an entity field's series, a money subtotal, `model.net_cash_flow` — and read
`inputs.*`, `cfg.*`, the engine's `model.*` metrics, and `metric.<name>` for
any metric declared above it. Metrics compose in declaration order, the rule
waterfalls follow, so `margin` above reads the two before it; a forward or
circular reference is refused (`E1354`). Folding a name the model does not
publish is refused too (`E1365`), not read as zero.

Two folds exist in a metric and nowhere else — a participant's realized
return, read from the party's own account:

```cfdl
metric lp_irr  = irr(party.lp)
metric lp_moic = moic(party.lp)
```

Outside a metric both are refused (`E1355`): a stream amount cannot ask for
a return on cash the stream has not produced yet.

Every declared metric is published as `metric.<name>` in
`deterministic.metrics`, in every scenario summary, and in every Monte Carlo
trial.

## Under Monte Carlo

Every metric — the engine's, the pack's, and the model's declared ones —
gets a distribution summary in the results' `monte_carlo.metrics` block:
mean, stdev, min/max, percentiles (p01–p99), and `trials`, the count of
trials that published that name.

## In the Python SDK

`results.metrics()` returns a flat Series; `results.metrics_frame()` adds
currency and source lineage (`core` vs `domain:<pack>`).

## Reference links

- [Metrics reference](/docs/reference/metrics)
- [Results schema](/docs/specification/results-schema)
- [Domain packs](/docs/packs)
