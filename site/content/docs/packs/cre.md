---
id: pack-cre
title: "CRE"
slug: "/docs/packs/cre"
description: "The CRE pack: a lease-by-lease institutional DCF with escalating rent, expense recoveries above a stop, lease-up, and an exit on a cap rate."
generated: regions
---

# CRE

Commercial real estate: income-producing property held, operated and sold.

## What it models

A lease-by-lease institutional DCF. Rent escalating on its anniversary, expense recoveries above a stop, vacancy and free rent, rollover at expiry blended across renewal and re-letting, tenant improvements and leasing commissions below the NOI line, amortizing debt with an optional interest-only period, and a reversion struck at a cap rate off a forward year's income.

## Contracts

Declare a contract and the pack expands it into the streams those terms imply,
each classified so it lands on the right line of a
[statement](/docs/reference/statements).

<!-- cfdl:generated contracts-cre -->
| Contract | Terms it reads | Streams it emits |
|---|---|---|
| `cre.construction_stub` | `amount` | `cre.construction.draws` |
| `cre.lease` | `rent`, `rent_year` | `cre.lease.base_rent` |
| `cre.revenue_line` | `amount`, `amount_year`, `growth_rate` | `cre.revenue.line[.suffix]` |
| `cre.exit_cap` | `cap_rate`, `income` | `cre.exit.sale` |
| `cre.lease_unit` | `escalation`, `expense_stop_year`, `gross_up_factor`, `lc_total`, `opex_escalation`, `opex_year`, `pro_rata_share`, `rent_year`, `ti_total` | `cre.unit.base_rent.[suffix]`, `cre.unit.abatement.[suffix]`, `cre.unit.recoveries.[suffix]`, `cre.unit.ti_lc.[suffix]` |
| `cre.rollover` | `market_escalation`, `market_rent_year`, `new_ti_lc`, `renewal_probability`, `renewal_rent_year`, `renewal_ti_lc` | `cre.rollover.rent.[suffix]`, `cre.rollover.ti_lc.[suffix]` |
| `cre.vacancy_loss` | `potential_gross_year`, `rate` | `cre.vacancy.loss` |
| `cre.opex_line` | `amount`, `amount_year`, `growth_rate`, `occupancy`, `pct_fixed` | `cre.opex.line[.suffix]` |
| `cre.exit` | `cap_rate`, `income`, `selling_costs` | `cre.exit.selling_costs`, `cre.exit.proceeds` |
| `cre.percentage_rent_expected` | `breakpoint_year`, `overage_pct`, `sales_growth`, `sales_quantile` | `cre.pct_rent.overage[.suffix]` |
| `cre.percentage_rent` | `breakpoint_year`, `overage_pct`, `sales_growth`, `sales_year` | `cre.pct_rent.overage[.suffix]` |
| `cre.exit_forward` | `cap_rate`, `selling_costs` | `cre.exit.proceeds`, `cre.exit.selling_costs` |
| `cre.permanent_debt` | `balloon_at_maturity`, `funded_at_close`, `interest_rate`, `payment_frequency`, `principal` | `cre.debt.proceeds[.suffix]`, `cre.debt.interest[.suffix]`, `cre.debt.principal[.suffix]` |
| `cre.construction_loan` | `capitalize_interest`, `draw_accrual_fraction`, `draw_curve`, `equity_commitment`, `interest_rate` | `cre.construction.equity_draw[.suffix]`, `cre.construction.loan_draw[.suffix]`, `cre.construction.interest[.suffix]` |
<!-- /cfdl:generated contracts-cre -->

A contract can be declared more than once by giving it a suffix, so the pieces
stay separable in the results.

## Reporting

An operating pro forma at the model grid, and the same cash annually. Debt service coverage is reported per period rather than over the hold, because a lifetime ratio of 1.4 can contain a year at 0.9.

## Related

- [Statements](/docs/reference/statements) — the pro forma this pack produces
- [Metrics](/docs/reference/metrics) — what it reports over the whole model
- [Validation](/docs/benchmarks) — the reference models it is gated against
