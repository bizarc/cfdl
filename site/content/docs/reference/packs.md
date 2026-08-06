---
id: reference-packs
title: "Pack contracts"
slug: "/docs/reference/packs"
generated: regions
---

# Pack contracts

A **contract** is how a model states business terms instead of building streams
by hand. Declare a lease and the pack expands it into the streams that lease
implies — rent, escalations, recoveries, the abatement — each already classified
so it lands in the right line of a statement.

```
use pack "cre" version "0.1.0"

contract cre.lease {
  term 2026-07..2031-12
  terms {
    base_rent = 25000
  }
}
```

## What you get back

Each contract lowers to one or more streams with fixed names, so a model can
refer to them, a benchmark can assert them, and a statement can group them. The
names are stable: they are part of the pack's interface, not an implementation
detail.

A term you do not supply is either defaulted by the pack or reported as a
missing-term error at compile time. It is never silently zero.

## Choosing a pack

| Pack | For |
|---|---|
| [`cre`](/docs/packs/cre) | Commercial real estate — leases, rollover, operating expenses, permanent debt, exit |
| [`credit`](/docs/packs/credit) | Loan pools and structured credit — amortisation, prepayment, defaults, recoveries |
| [`energy`](/docs/packs/energy) | Generation projects — PPAs, merchant revenue, O&M, tax credits, project debt |
| [`opco`](/docs/packs/opco) | Operating companies — revenue and cost lines, working capital, leverage, exit |

## Every contract

Generated from each pack's lowering rules, so a contract cannot be listed here
without existing, or exist without being listed. Terms shown are those a model
supplies; a pack also reads the contract's own `term` range.

`[.suffix]` in a stream name means the contract can be declared more than once.
Give it a suffix — `contract cre.lease.suite_200` — and the streams it emits
carry that suffix, so two leases stay separable in the results and on a
statement.

<!-- cfdl:generated pack-contracts -->
### `energy`

| Contract | Terms it reads | Streams it emits |
|---|---|---|
| `energy.ppa` | `availability`, `degradation`, `escalation`, `mwh_year`, `ppa_price` | `energy.ppa.revenue[.suffix]` |
| `energy.merchant` | `availability`, `degradation`, `mwh_year`, `price`, `price_escalation` | `energy.merchant.revenue[.suffix]` |
| `energy.storage_arbitrage` | `degradation`, `mwh_cycled_year`, `spread` | `energy.storage.margin[.suffix]` |
| `energy.capacity` | `payment_year` | `energy.capacity.revenue[.suffix]` |
| `energy.om` | `escalation`, `om_year` | `energy.om.expense[.suffix]` |
| `energy.itc` | `credit` | `energy.itc.credit[.suffix]` |
| `energy.capex` | `amount` | `energy.capex.outlay[.suffix]` |
| `energy.debt_service` | `principal`, `rate` | `energy.debt.service[.suffix]` |
| `energy.ptc` | `availability`, `credit_per_mwh`, `degradation`, `escalation`, `mwh_year`, `round_step` | `energy.ptc.credit[.suffix]` |
| `energy.macrs_shield` | `basis`, `life`, `tax_rate` | `energy.macrs.shield[.suffix]` |

### `cre`

| Contract | Terms it reads | Streams it emits |
|---|---|---|
| `cre.construction_stub` | `amount` | `cre.construction.draws` |
| `cre.lease` | `base_rent`, `base_rent_year` | `cre.lease.base_rent` |
| `cre.ops_revenue` | `amount` | `cre.ops.revenue` |
| `cre.ops_expense` | `amount` | `cre.ops.expense` |
| `cre.exit_cap` | `exit_cap`, `noi_value` | `cre.exit.sale` |
| `cre.lease_unit` | `escalation`, `expense_stop_year`, `gross_up_factor`, `lc_total`, `opex_escalation`, `opex_year`, `pro_rata_share`, `rent_year`, `ti_total` | `cre.unit.base_rent.[suffix]`, `cre.unit.abatement.[suffix]`, `cre.unit.recoveries.[suffix]`, `cre.unit.ti_lc.[suffix]` |
| `cre.rollover` | `market_escalation`, `market_rent_year`, `new_ti_lc`, `renewal_probability`, `renewal_rent_year`, `renewal_ti_lc` | `cre.rollover.rent.[suffix]`, `cre.rollover.ti_lc.[suffix]` |
| `cre.vacancy_loss` | `potential_gross_year`, `rate` | `cre.vacancy.loss` |
| `cre.property_opex` | `escalation`, `opex_year` | `cre.property.opex[.suffix]` |
| `cre.exit` | `exit_cap`, `noi_forward_year`, `selling_costs` | `cre.exit.proceeds` |
| `cre.percentage_rent` | `breakpoint_year`, `overage_pct`, `sales_growth`, `sales_year` | `cre.pct_rent[.suffix]` |
| `cre.exit_forward` | `exit_cap`, `selling_costs` | `cre.exit.proceeds` |
| `cre.permanent_debt` | `balloon_at_maturity`, `payment_frequency`, `principal`, `rate` | `loan.permanent_debt_service` |

### `credit`

| Contract | Terms it reads | Streams it emits |
|---|---|---|
| `credit.pool_level_pay` | `abs_speed`, `age_months`, `balance`, `cdr`, `cpr`, `payment_frequency`, `prepay_penalty_rate`, `psa_speed`, `rate`, `sda_speed`, `servicing_fee`, `severity` | `credit.pool.interest[.suffix]`, `credit.pool.sched_principal[.suffix]`, `credit.pool.prepay[.suffix]`, `credit.pool.recoveries[.suffix]`, `credit.pool.servicing[.suffix]`, `credit.pool.penalty[.suffix]` |
| `credit.pool_io_bullet` | `abs_speed`, `age_months`, `balance`, `cdr`, `cpr`, `payment_frequency`, `prepay_penalty_rate`, `psa_speed`, `rate`, `sda_speed`, `servicing_fee`, `severity` | `credit.pool.interest[.suffix]`, `credit.pool.prepay[.suffix]`, `credit.pool.bullet[.suffix]`, `credit.pool.recoveries[.suffix]`, `credit.pool.servicing[.suffix]`, `credit.pool.penalty[.suffix]` |
| `credit.pool_float_io_bullet` | `abs_speed`, `age_months`, `balance`, `cdr`, `cpr`, `index_curve`, `margin`, `payment_frequency`, `prepay_penalty_rate`, `psa_speed`, `rate_cap`, `rate_floor`, `sda_speed`, `servicing_fee`, `severity` | `credit.pool.interest[.suffix]`, `credit.pool.prepay[.suffix]`, `credit.pool.bullet[.suffix]`, `credit.pool.recoveries[.suffix]`, `credit.pool.servicing[.suffix]`, `credit.pool.penalty[.suffix]` |
| `credit.purchase` | `price` | `credit.purchase.price[.suffix]` |

### `opco`

| Contract | Terms it reads | Streams it emits |
|---|---|---|
| `opco.revenue_line` | `amount`, `amount_year`, `growth_curve`, `growth_rate` | `opco.revenue.recurring[.suffix]` |
| `opco.opex_line` | `amount`, `amount_year`, `growth_curve`, `growth_rate` | `opco.opex.recurring[.suffix]` |
| `opco.working_capital` | `amount` | `opco.working_capital.adjustment[.suffix]` |
| `opco.exit_multiple` | `base_value`, `exit_multiple` | `opco.exit.value` |
| `opco.working_capital_policy` | `ap_days`, `ar_days`, `inv_days`, `release_at_end` | `opco.working_capital.adjustment[.suffix]` |
| `opco.capex_line` | `amount`, `amount_year`, `growth_curve`, `growth_rate`, `pct_of_revenue` | `opco.capex[.suffix]` |
| `opco.term_debt` | `funded_at_close`, `principal`, `rate` | `opco.debt.proceeds[.suffix]`, `opco.debt.interest[.suffix]`, `opco.debt.principal[.suffix]` |
| `opco.cash_taxes` | `da_growth`, `da_monthly`, `da_year`, `tax_rate`, `tax_rate_curve` | `opco.taxes[.suffix]` |
| `opco.exit_ebitda` | `exit_multiple`, `selling_costs` | `opco.exit.value` |
| `opco.acquisition` | `price` | `opco.acquisition.price[.suffix]` |
| `opco.exit_perpetuity` | `base_value`, `discount_rate`, `growth_rate`, `selling_costs` | `opco.exit.value[.suffix]` |

<!-- /cfdl:generated pack-contracts -->

## Related

- [Statements](/docs/reference/statements) — how the streams a contract emits
  become a pro forma.
- [Contracts & packs](/docs/guides/contracts-and-packs) — a walkthrough.
- [Pack interface](/docs/specification/pack-interface) — authoring a pack.
