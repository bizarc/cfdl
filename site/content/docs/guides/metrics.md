---
id: guide-metrics
title: Metrics
slug: /docs/guides/metrics
generated: none
---

# Metrics

Metrics are computed by the engine — models declare inputs and behavior,
not metric formulas.

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

## Under Monte Carlo

Every metric gets a distribution summary — mean, stdev, min/max, and
percentiles (p01–p99) — in the results' `monte_carlo.metrics` block.

## In the Python SDK

`results.metrics()` returns a flat Series; `results.metrics_frame()` adds
currency and source lineage (`core` vs `domain:<pack>`).

## Reference links

- [Results schema](/docs/specification/results-schema)
- [Domain Packs](/docs/packs)
