---
id: reference-metrics
title: "Metrics"
slug: "/docs/reference/metrics"
generated: regions
---

# Metrics

A run reports a set of scalars beside its cash flows. Model metrics are
computed for every model; domain metrics come from the active pack and are
absent without one.

## Model metrics

`model.npv` discounts the model's net cash flow at the run's rate.
`model.irr` is the rate at which that NPV is zero, and is undefined for a
series that never changes sign. `model.moic` is the multiple of invested
capital, `model.payback_years` when cumulative cash first turns positive, and
`model.wal_years` the weighted average life — when principal actually comes
back, which a maturity date does not tell you.

`model.total` is the undiscounted sum, and `model.net_cash_flow` the per-period
series everything else is derived from.

## Metrics and subtotals are not the same thing

A metric is one number for the whole model. A subtotal is a series — one value
per period — published under `domain.<pack>.<name>` in `deterministic.series`.

Where both exist under the same name, the metric is the reduction of the
subtotal: `domain.cre.noi` in the metrics block is the sum over periods of
`domain.cre.noi` in the series block. That is deliberate — one definition,
reduced, rather than two definitions that drift.

A lifetime figure hides the path. A coverage ratio of 1.4 over a hold can
contain a year at 0.9, which is why coverage is published per period and read
from a [statement](/docs/reference/statements).

## Every domain metric

Generated from each pack's declarations.

<!-- cfdl:generated pack-metrics -->
### `energy`

| Metric | Kind | Definition |
|---|---|---|
| `domain.energy.revenue` | money | sum(numerator_streams) |
| `domain.energy.opex` | money | -sum(numerator_streams) |
| `domain.energy.ebitda` | money | sum(numerator_streams) + sum(denominator_streams) |
| `domain.energy.debt_service` | money | -sum(numerator_streams) |
| `domain.energy.dscr` | number | domain.energy.ebitda / domain.energy.debt_service |
| `domain.energy.tax_benefits` | money | sum(numerator_streams) |

### `cre`

| Metric | Kind | Definition |
|---|---|---|
| `domain.cre.noi` | money | sum over periods of domain.cre.noi |
| `domain.cre.debt_service` | money | sum over periods of domain.cre.debt_service |
| `domain.cre.dscr` | number | domain.cre.noi / domain.cre.debt_service |
| `domain.cre.leasing_costs` | money | sum over periods of domain.cre.leasing_costs |

### `credit`

| Metric | Kind | Definition |
|---|---|---|
| `domain.credit.interest` | money | sum(numerator_streams) |
| `domain.credit.principal` | money | sum(numerator_streams) |
| `domain.credit.recoveries` | money | sum(numerator_streams) |
| `domain.credit.penalties` | money | sum(numerator_streams) |
| `domain.credit.servicing` | money | -sum(numerator_streams) |
| `domain.credit.wal_years` | number | wal_years(numerator_streams) |
| `domain.credit.collections` | money | sum(numerator_streams) |
| `domain.credit.purchase` | money | -sum(numerator_streams) |
| `domain.credit.collections_multiple` | number | domain.credit.collections / domain.credit.purchase |

### `opco`

| Metric | Kind | Definition |
|---|---|---|
| `domain.opco.revenue` | money | sum(numerator_streams) |
| `domain.opco.ebitda` | money | sum(numerator_streams) + sum(denominator_streams) |
| `domain.opco.ebitda_margin` | number | domain.opco.ebitda / domain.opco.revenue |
| `domain.opco.capex` | money | -sum(numerator_streams) |
| `domain.opco.working_capital` | money | -sum(numerator_streams) |
| `domain.opco.taxes` | money | -sum(numerator_streams) |
| `domain.opco.debt_service` | money | -sum(numerator_streams) |
| `domain.opco.fcf` | money | sum(numerator_streams) + sum(denominator_streams) |
| `domain.opco.fcf_to_debt_service` | number | domain.opco.fcf / domain.opco.debt_service |

<!-- /cfdl:generated pack-metrics -->

## Related

- [Metrics guide](/docs/guides/metrics) — choosing and interpreting them.
- [Statements](/docs/reference/statements) — the per-period view.
- [Results schema](/docs/specification/results-schema) — where each metric sits
  in the output document.
