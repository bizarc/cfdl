---
id: benchmark-opco-dcf-exit-multiple-nwc
title: "OpCo: working capital and a terminal exit multiple"
slug: "/docs/examples/opco-dcf-exit-multiple-nwc"
description: "An operating company discounted cash flow with an explicit working-capital line and a terminal value struck as a multiple of trailing EBITDA."
source: benchmarks/opco/dcf_exit_multiple_nwc
---

# OpCo: working capital and a terminal exit multiple

An operating company discounted cash flow with an explicit working-capital line and a terminal value struck as a multiple of trailing EBITDA.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A practitioner discounted cash flow of a mid-market operating company, valued
at 31 December 2017 over a five-year forecast. Revenue grows from 121 to 154,
EBITDA margin holds near 36–37%, and the terminal year holds revenue, EBITDA,
capital expenditure and working capital flat while depreciation steps to 95% of
terminal capex.

What distinguishes it is that the template computes its terminal value **both
ways** — perpetuity growth and a multiple of trailing EBITDA — and publishes a
full three-by-three output grid for each, so the two methods can be read against
one another. This case asserts the exit-multiple arm.

The two methods do not discount alike, and the difference is in the formulas
rather than the prose: the exit-multiple terminal is discounted over 5.0 years
and the perpetuity terminal over 4.5. A terminal value is a price struck at a
point in time, and `opco.exit_multiple` places a disposal at the end of the
holding period, so the exit-multiple arm is the one the contract can spell.

## The reference

A published DCF template from a practitioner site, downloadable without
registration, carrying its inputs as typed constants and its outputs as live
formulas. It states the working-capital line and the exit multiple explicitly
rather than deriving either, which is why this source and not another.

**Not redistributable.** The publisher reserves copyright, so the figures are
asserted against and the workbook is not vendored — the same posture as
`banker_dcf_conventions`. `NOTES.md` records the URL, size and SHA-256 so a
reader can fetch the identical file.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Contract types | `opco.revenue_line`, `opco.opex_line`, `opco.cash_taxes`, `opco.capex_line`, `opco.working_capital`, `opco.exit_multiple` |
| Declared | 21 contracts, one entity, no curves, no native streams |
| Language features | a contract term restricted to a single period, to place an annual flow on a monthly grid |
| Conventions | mid-year discounting of flows, a terminal value discounted whole, depreciation carried as a tax term rather than a cash line |

It closes the last two unexercised types on the `opco` roster:
`opco.working_capital` — the fixed variant, as against the DSO/DPO
`working_capital_policy` that `lbo_buyout` covers — and `opco.exit_multiple`,
the stated-base form, as against the stream-derived `opco.exit_ebitda`.

Depreciation never becomes a flow. The published build subtracts it to reach
EBIT, taxes that, then adds it straight back, so it touches cash only through
the tax — which is what `opco.cash_taxes` carries `da_monthly` for. Nothing
here books a depreciation stream, and the model needs no native stream at all.

The whole model is built from pack contracts, which makes the pack's lowering
the thing under test rather than only the engine's arithmetic.

## The result

`model.npv` = **338.366157** against the template's published 338.3661574792812,
at 10.0% WACC and a 7.0x LTM EBITDA multiple — the low-rate, low-multiple cell
of the published grid.

Every line of the cash flow build reproduces exactly: revenue, cash operating
expense, cash taxes, capital expenditure and the increase in net working
capital, each asserted per period against the template's own figure, and the
terminal value at 401.396044.

All nine cells of the exit-multiple grid reconcile, the worst by 4.8e-7. The
other eight need a different discount rate or multiple per run, so the asserted
cell is one of them and the rest are in `NOTES.md`.

## The delta

None. The residual on every cell is within ±5e-7, which is the results
document's six-decimal rounding rather than a disagreement — unlike
`banker_dcf_conventions`, whose source rounded to whole millions and whose
tolerance had to be set by that.

The tolerance is 1e-5 on the metric and 1e-3 per period, both set by the
rounding and neither by the arithmetic.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.1}}
// A practitioner DCF template that computes its terminal value BOTH ways —
// perpetuity growth and an LTM exit multiple — and publishes a 3x3 grid for
// each. This case asserts the exit-multiple arm.
//
// WHY THIS SOURCE. Two opco contract types had no case:
// `opco.working_capital` (the fixed variant, not the DSO/DPO policy) and
// `opco.exit_multiple` (a stated base times a stated multiple, not the
// stream-derived `opco.exit_ebitda`). Both need a source that PUBLISHES the
// two figures rather than deriving them. Damodaran's engine — the other
// candidate — cannot supply either: it folds working capital into
// reinvestment through a sales-to-capital ratio, and its terminal value is
// Gordon growth by doctrine. This template states an `Increase in NWC` line
// year by year and applies an explicit LTM EBITDA multiple.
//
// WHY THE EXIT-MULTIPLE ARM AND NOT PERPETUITY. The template discounts its
// two terminal values differently, and the difference is in the formulas
// rather than the prose: the exit-multiple terminal uses discount period 5.0
// and the perpetuity terminal uses 4.5. `opco.exit_multiple` lowers to
// `on_date` with `schedule_placement = "end"` — the full holding period, 5.0.
// So the exit-multiple arm is the one the contract can spell, and the
// perpetuity arm is not reachable through it. That is the same position
// packs/opco/lowering/rules.toml already takes: a terminal value is a price
// struck at a point in time and discounts whole.
//
// TIMING. The valuation date is 31 December 2017 and flows are discounted
// MID-YEAR: 0.5, 1.5, 2.5, 3.5, 4.5. CFDL calendars are uniform, so a monthly
// grid carries the convention and each year's cash is placed as a one-shot on
// the month whose close lands on the right exponent. A single-period `every`
// is an ordinary annuity, so it falls at the period's END:
//
//     FY2018   end of month index  5    6/12 = 0.5
//     FY2019   end of month index 17   18/12 = 1.5
//     FY2020   end of month index 29   30/12 = 2.5
//     FY2021   end of month index 41   42/12 = 3.5
//     FY2022   end of month index 53   54/12 = 4.5
//     terminal end of month index 59   60/12 = 5.0
//
// No flow needs `mid` — every exponent is a month boundary here, unlike
// benchmarks/opco/banker_dcf_conventions where the stub was not.
//
// D&A IS NOT A CASH LINE. The published build subtracts D&A to reach EBIT,
// taxes that, then adds D&A straight back — so it touches cash only through
// the tax. `opco.cash_taxes` carries it as `da_monthly` for exactly this
// reason, which is why nothing below books a depreciation flow.
//
// WHAT IS ASSERTED. The center-left cell of the published exit-multiple grid:
// 10.0% WACC, 7.0x LTM EBITDA, enterprise value 338.3661574792812. The other
// eight cells need a different discount rate or multiple per run and live in
// NOTES.md, the same shape as banker_dcf_conventions.

version 0.1
model "dcf-exit-multiple-nwc"
use pack "opco" version "0.1.0"
time calendar monthly from 2018-01 for 60

entity asset firm : OpCo.Asset.Enterprise

// --------------------------------------------------------------------- FY2018
// Cash opex is revenue less Adj. EBITDA, both published; the template does not
// print the subtotal, so it is stated here as the difference and nothing else.
contract opco.revenue_line.fy2018 on entity asset.firm {
  term 2018-06..2018-06
  terms { amount = 121.0 }
}
contract opco.opex_line.fy2018 on entity asset.firm {
  term 2018-06..2018-06
  terms { amount = 78.0 }
}
contract opco.cash_taxes.fy2018 on entity asset.firm {
  term 2018-06..2018-06
  terms {
    tax_rate   = 0.38
    da_monthly = 6.0
  }
}
contract opco.capex_line.fy2018 on entity asset.firm {
  term 2018-06..2018-06
  terms { amount = 7.0 }
}
contract opco.working_capital.fy2018 on entity asset.firm {
  term 2018-06..2018-06
  terms { amount = 3.0 }
}

// --------------------------------------------------------------------- FY2019
contract opco.revenue_line.fy2019 on entity asset.firm {
  term 2019-06..2019-06
  terms { amount = 129.47 }
}
contract opco.opex_line.fy2019 on entity asset.firm {
  term 2019-06..2019-06
  terms { amount = 82.47 }
}
contract opco.cash_taxes.fy2019 on entity asset.firm {
  term 2019-06..2019-06
  terms {
    tax_rate   = 0.38
    da_monthly = 6.300000000000001
  }
}
contract opco.capex_line.fy2019 on entity asset.firm {
  term 2019-06..2019-06
  terms { amount = 7.3500000000000005 }
}
contract opco.working_capital.fy2019 on entity asset.firm {
  term 2019-06..2019-06
  terms { amount = 3.2099999999999995 }
}

// --------------------------------------------------------------------- FY2020
contract opco.revenue_line.fy2020 on entity asset.firm {
  term 2020-06..2020-06
  terms { amount = 139.82760000000002 }
}
contract opco.opex_line.fy2020 on entity asset.firm {
  term 2020-06..2020-06
  terms { amount = 88.82760000000002 }
}
contract opco.cash_taxes.fy2020 on entity asset.firm {
  term 2020-06..2020-06
  terms {
    tax_rate   = 0.38
    da_monthly = 6.615000000000001
  }
}
contract opco.capex_line.fy2020 on entity asset.firm {
  term 2020-06..2020-06
  terms { amount = 7.717500000000001 }
}
contract opco.working_capital.fy2020 on entity asset.firm {
  term 2020-06..2020-06
  terms { amount = 3.4667999999999997 }
}

// --------------------------------------------------------------------- FY2021
contract opco.revenue_line.fy2021 on entity asset.firm {
  term 2021-06..2021-06
  terms { amount = 148.21725600000002 }
}
contract opco.opex_line.fy2021 on entity asset.firm {
  term 2021-06..2021-06
  terms { amount = 93.21725600000002 }
}
contract opco.cash_taxes.fy2021 on entity asset.firm {
  term 2021-06..2021-06
  terms {
    tax_rate   = 0.38
    da_monthly = 6.945750000000001
  }
}
contract opco.capex_line.fy2021 on entity asset.firm {
  term 2021-06..2021-06
  terms { amount = 8.103375000000002 }
}
contract opco.working_capital.fy2021 on entity asset.firm {
  term 2021-06..2021-06
  terms { amount = 3.6748079999999996 }
}

// --------------------------------------------------------------------- FY2022
contract opco.revenue_line.fy2022 on entity asset.firm {
  term 2022-06..2022-06
  terms { amount = 154.14594624000003 }
}
contract opco.opex_line.fy2022 on entity asset.firm {
  term 2022-06..2022-06
  terms { amount = 96.80365423872001 }
}
contract opco.cash_taxes.fy2022 on entity asset.firm {
  term 2022-06..2022-06
  terms {
    tax_rate   = 0.38
    da_monthly = 7.293037500000001
  }
}
contract opco.capex_line.fy2022 on entity asset.firm {
  term 2022-06..2022-06
  terms { amount = 8.508543750000001 }
}
contract opco.working_capital.fy2022 on entity asset.firm {
  term 2022-06..2022-06
  terms { amount = 3.82180032 }
}

// -------------------------------------------------------------------- terminal
// LTM EBITDA is the terminal column's Adj. EBITDA, which the template holds
// flat at FY2022. 7.0x is the low end of the published 7.0x/8.0x/9.0x row.
contract opco.exit_multiple.terminal on entity asset.firm {
  term 2022-12..2022-12
  terms {
    base_value    = 57.34229200128001
    exit_multiple = 7.0
  }
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.10
  }
}
```

## Verified results

Checked period by period: **26 series** across **60 periods** — **1560 values** in all, each within ±0.001 of the reference.

- `opco.revenue.recurring.fy2018`
- `opco.revenue.recurring.fy2019`
- `opco.revenue.recurring.fy2020`
- `opco.revenue.recurring.fy2021`
- `opco.revenue.recurring.fy2022`
- `opco.opex.recurring.fy2018`
- `opco.opex.recurring.fy2019`
- `opco.opex.recurring.fy2020`
- `opco.opex.recurring.fy2021`
- `opco.opex.recurring.fy2022`
- `opco.taxes.cash.fy2018`
- `opco.taxes.cash.fy2019`
- `opco.taxes.cash.fy2020`
- `opco.taxes.cash.fy2021`
- `opco.taxes.cash.fy2022`
- `opco.capex.line.fy2018`
- `opco.capex.line.fy2019`
- `opco.capex.line.fy2020`
- `opco.capex.line.fy2021`
- `opco.capex.line.fy2022`
- `opco.working_capital.adjustment.fy2018`
- `opco.working_capital.adjustment.fy2019`
- `opco.working_capital.adjustment.fy2020`
- `opco.working_capital.adjustment.fy2021`
- `opco.working_capital.adjustment.fy2022`
- `opco.exit.value`

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | 338.3661574792812 | ±0.00001 |
