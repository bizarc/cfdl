---
id: pack-opco
title: "OpCo"
slug: "/docs/packs/opco"
generated: regions
---

# OpCo

Operating companies: revenue and cost lines, working capital, leverage and an exit.

## What it models

A business valued on its cash flow. Revenue and operating cost growing at declared rates, cash taxes, movements in working capital, capital spend, debt drawn and repaid, and an exit at a multiple or a perpetuity.

## Contracts

Declare a contract and the pack expands it into the streams those terms imply,
each classified so it lands on the right line of a
[statement](/docs/reference/statements).

<!-- cfdl:generated contracts-opco -->
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
<!-- /cfdl:generated contracts-opco -->

A contract can be declared more than once by giving it a suffix, so the pieces
stay separable in the results.

## Reporting

A free cash flow build-up from revenue through unlevered to levered cash flow, a sponsor view for a leveraged buyout, and a statement of cash flows by activity as ASC 230 and IAS 7 define it.

## Related

- [Statements](/docs/reference/statements) — the pro forma this pack produces
- [Metrics](/docs/reference/metrics) — what it reports over the whole model
- [Validation](/docs/benchmarks) — the reference models it is gated against
