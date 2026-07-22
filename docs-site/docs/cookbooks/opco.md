---
id: cookbook-opco
title: "OpCo pack guide"
slug: "/cookbooks/opco"
---

> This page is generated from `packs/opco/README.md`.
> Source: https://github.com/bizarc/cfdl/blob/main/packs/opco/README.md

Operating-company / LBO pack: recurring operating lines, policy-driven
working capital, capex, scheduled term debt, cash taxes, and entry/exit —
benchmarked in `benchmarks/opco/` against an independent month-by-month
reference. All lowering is template-driven; the legacy
hardcoded compile path (`apply_opco_contract_terms`) is retired.

## Activation

```cfdl
use pack "opco" version "0.1.0"
```

## Contract types

All contracts accept instance suffixes (`opco.revenue_line.saas`,
`opco.revenue_line.services`, ...) which suffix the lowered stream names.
Growth is annual-compound stepped continuously on the model clock:
`value(t) = amount * (1 + growth_rate)^(time.t / 12)`.

### Operating lines

- `opco.revenue_line` — `amount` (monthly), optional `growth_rate`.
  Stream `opco.revenue.recurring`.
- `opco.opex_line` — same terms. Stream `opco.opex.recurring` (outflow).
- `opco.working_capital` — fixed monthly WC outflow (`amount`).
- `opco.working_capital_policy` — DSO/DPO/DIO-driven:
  `WC(t) = annualized revenue * ar_days/365 + annualized opex * (inv_days - ap_days)/365`
  from the modeled streams (phase-2 series lookups). Books the full initial
  WC in the first period, the period-over-period change afterwards, and
  releases the ending balance in the final period when `release_at_end = 1`.
  Terms: `ar_days`, `ap_days`, `inv_days` (all default 0), `release_at_end`.
- `opco.capex_line` — fixed `amount` (+ `growth_rate`) plus
  `pct_of_revenue` of the modeled revenue streams. Stream `opco.capex`.

### Financing

- `opco.term_debt` — scheduled term loan: `principal`, `rate`,
  `io_months` (default 0), `amort_months`; optional `funded_at_close`
  (default 1) controls the proceeds inflow at `term_start`. After the IO
  period the loan amortizes level-pay over `amort_months`; the remaining
  balance pays as a balloon at the contract's `term_end`. Streams
  `opco.debt.proceeds`, `opco.debt.interest`, `opco.debt.principal`.
  **Cash sweeps and revolvers need per-period persistent state and ship
  with Workstream H3.**
- `opco.acquisition` — purchase `price` paid at `term_start`
  (the equity check when paired with debt proceeds at the same date).

### Taxes

- `opco.cash_taxes` — `tax_rate` on `max(0, EBITDA - D&A - interest)` per
  period. EBITDA and interest come from the modeled streams; D&A is a
  declared deduction (`da_monthly`, optional `da_growth`), not a cash
  stream. **No NOL carryforwards** (losses floor at zero tax per period;
  carryforwards need H3-style state). Stream `opco.taxes`.

### Exit

- `opco.exit_multiple` — `base_value * exit_multiple` at the contract's
  `term_start` (the legacy `exit_period` term is inert).
- `opco.exit_ebitda` — `exit_multiple` × trailing-12-month EBITDA derived
  from the modeled streams, net of `selling_costs`, at `term_start`.

## Metrics

`domain.opco.revenue`, `.ebitda`, `.ebitda_margin`, `.capex`,
`.working_capital` (net investment; releases net out), `.taxes`,
`.debt_service`, `.fcf` (EBITDA − capex − WC − cash taxes; note taxes
deduct interest, so this is FCF after the interest tax shield),
`.fcf_to_debt_service`.

## Diagnostics (E7xxx)

- `E7001_OPCO_LINE_MISSING_AMOUNT`, `E7002_OPCO_LINE_INVALID_SCHEDULE`,
  `E7003_OPCO_LINE_INVALID_GROWTH`
- `E7010_OPCO_WC_MISSING_AMOUNT_OR_RULE`, `E7011_OPCO_WC_INVALID_SCHEDULE`
- `E7020_OPCO_EXIT_MISSING_MULTIPLE`, `E7021_OPCO_EXIT_INVALID_MULTIPLE`,
  `E7022_OPCO_EXIT_MISSING_BASE_VALUE`, `E7023_OPCO_EXIT_INVALID_SCHEDULE`
- `E7024_OPCO_EXIT_EBITDA_INVALID_MULTIPLE`
- `E7030_OPCO_DEBT_INVALID_AMORT`, `E7031_OPCO_DEBT_INVALID_RATE`
- Missing templated terms surface as `E5006_MISSING_CONTRACT_TERM`.

## Not in v0.1 (planned waterfall & capital-stack work)

- Cash-flow sweeps, revolver draws/paydowns, PIK toggles (need H3 state).
- NOL carryforwards.
- Waterfall distributions to the capital stack (Workstream H).

## Provenance and determinism

Generated streams carry source contract file/span and
`generated_by.pack/rule_id`; rule ordering, diagnostics ordering, IDs and
results are deterministic under identical inputs.
