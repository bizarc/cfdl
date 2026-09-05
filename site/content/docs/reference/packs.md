---
id: reference-packs
title: "Pack contracts"
slug: "/docs/reference/packs"
description: "The contracts each pack ships, the terms they accept, and the streams they expand into."
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
    rent = 25000
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
| [`credit`](/docs/packs/credit) | Loan pools and structured credit — amortization, prepayment, defaults, recoveries |
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
| `energy.ppa` | `availability`, `degradation`, `escalation`, `price`, `quantity` | `energy.ppa.revenue[.suffix]` |
| `energy.merchant` | `availability`, `degradation`, `escalation`, `price`, `quantity` | `energy.merchant.revenue[.suffix]` |
| `energy.storage_arbitrage` | `degradation`, `price`, `quantity` | `energy.storage.margin[.suffix]` |
| `energy.capacity` | `price` | `energy.capacity.revenue[.suffix]` |
| `energy.om` | `escalation`, `fee_year` | `energy.om.expense[.suffix]` |
| `energy.itc` | `amount` | `energy.itc.credit[.suffix]` |
| `energy.capex` | `amount` | `energy.capex.outlay[.suffix]` |
| `energy.debt_service` | `funded_at_close`, `interest_rate`, `principal` | `energy.debt.proceeds[.suffix]`, `energy.debt.interest[.suffix]`, `energy.debt.principal[.suffix]` |
| `energy.ptc` | `amount`, `availability`, `degradation`, `escalation`, `quantity`, `round_step` | `energy.ptc.credit[.suffix]` |
| `energy.macrs_shield` | `basis`, `life`, `tax_rate` | `energy.macrs.shield[.suffix]` |

### `cre`

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

### `credit`

| Contract | Terms it reads | Streams it emits |
|---|---|---|
| `credit.loan` | `abs_speed`, `age_months`, `cdr`, `cpr`, `index_curve`, `interest_rate`, `margin`, `payment_frequency`, `prepay_penalty_rate`, `principal`, `psa_speed`, `rate_cap`, `rate_floor`, `sda_speed`, `servicing_fee`, `severity` | `credit.loan.interest[.suffix]`, `credit.loan.sched_principal[.suffix]`, `credit.loan.prepay[.suffix]`, `credit.loan.recoveries[.suffix]`, `credit.loan.servicing[.suffix]`, `credit.loan.penalty[.suffix]`, `credit.loan.bullet[.suffix]` |
| `credit.purchase` | `price` | `credit.purchase.price[.suffix]` |
| `credit.participation` | `share` | `credit.participation.interest[.suffix]`, `credit.participation.principal[.suffix]` |
| `credit.note` | `coupon`, `face`, `payment_frequency`, `principal_account` |  |

### `opco`

| Contract | Terms it reads | Streams it emits |
|---|---|---|
| `opco.revenue_line` | `amount`, `amount_year`, `growth_rate` | `opco.revenue.recurring[.suffix]` |
| `opco.opex_line` | `amount`, `amount_year`, `growth_rate` | `opco.opex.recurring[.suffix]` |
| `opco.working_capital` | `amount` | `opco.working_capital.adjustment[.suffix]` |
| `opco.exit_multiple` | `base`, `multiple` | `opco.exit.value` |
| `opco.working_capital_policy` | `ap_days`, `ar_days`, `inv_days`, `release_at_end` | `opco.working_capital.adjustment[.suffix]` |
| `opco.capex_line` | `amount`, `amount_year`, `growth_rate`, `pct_of_revenue` | `opco.capex.line[.suffix]` |
| `opco.term_debt` | `funded_at_close`, `interest_rate`, `principal` | `opco.debt.proceeds[.suffix]`, `opco.debt.interest[.suffix]`, `opco.debt.principal[.suffix]` |
| `opco.cash_taxes` | `da_growth`, `da_monthly`, `da_year`, `tax_rate` | `opco.taxes.cash[.suffix]` |
| `opco.exit_ebitda` | `multiple`, `selling_costs` | `opco.exit.value`, `opco.exit.selling_costs` |
| `opco.acquisition` | `price` | `opco.acquisition.price[.suffix]` |
| `opco.exit_perpetuity` | `base`, `discount_rate`, `growth_rate`, `selling_costs` | `opco.exit.value[.suffix]`, `opco.exit.selling_costs[.suffix]` |

<!-- /cfdl:generated pack-contracts -->

## Related

- [Statements](/docs/reference/statements) — how the streams a contract emits
  become a pro forma.
- [Contracts and packs](/docs/guides/contracts-and-packs) — a walkthrough.
- [Pack interface](/docs/specification/pack-interface) — authoring a pack.
