---
id: benchmark-opco-saas-sbc-convention-fork
title: "OpCo: SaaS DCF and the stock-compensation fork"
slug: "/docs/examples/opco-saas-sbc-convention-fork"
source: benchmarks/opco/saas_sbc_convention_fork
---

# OpCo: SaaS DCF and the stock-compensation fork

A subscription software business valued on discounted cash flow, with stock-based compensation carried as its own line so the same model states value before and after it.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A sponsor take-private of a subscription-software business. Stock-based
compensation is the most contested convention in software valuation, and this
case exists because the source discloses free cash flow **both ways** — before
and after it — for the same company on the same page. The gap is not a rounding
difference: $331mm before, $198mm after, in the first year alone. Two thirds of
first-year cash flow turns on the convention.

## The reference

Banker's discussion materials filed as an exhibit to a going-private
transaction. It discloses the cash flow build-up line by line, the
stock-compensation line, the post-compensation series, the discount rate range,
the terminal method, the discounting convention, the valuation date, and a 3×4
grid of implied enterprise values.

**Not redistributable.** The filer retains copyright, so figures are asserted
against. The exhibit uses code names for both parties, so the case describes the
analysis rather than the company.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Declared | ten native streams |
| Language features | stock compensation modelled as its own stream, so both conventions come from one model |
| Conventions | mid-period discounting, a nine-month stub, a terminal multiple struck on a pre-compensation base |

Compensation is a separate stream on the same date as the flow it burdens, so
the post-compensation series is *derived* rather than restated and cannot drift
from the pre-compensation one.

## The result

`model.npv` = **7,096** against the filing's published 7,096, at 13.5% and 16.0×.

All twelve cells of the disclosed grid reconcile, worst **±0.50 on ~7,000** —
0.007%, inside the filing's own whole-million rounding.

## The delta

The filing **mixes the two conventions**, and reproducing the grid is what
established it: the explicit-period flows are post-compensation while the
terminal multiple is applied to a pre-compensation base. Nothing in the document
says so. It is defensible — the multiple was calibrated on peers' pre-compensation
cash flow — but it means one model has to carry both definitions at once.

One input is not disclosed. The filing states first-year cash flow as a full year
and notes the valuation includes only the last three quarters, without publishing
the split. That figure is solved from the grid: one unknown against twelve
published values, leaving eleven degrees of freedom. At the solved value all
twelve land within ±0.50. It comes out at 68.4% of the year rather than 75%,
which is the expected direction for an annual-prepaid subscription business where
the first quarter carries the cash.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.135}}
// A sponsor take-private DCF of a subscription-software business, reconciled
// against the analysis disclosed in a public merger filing.
//
// WHAT THIS CASE IS FOR: stock-based compensation is the most contested
// convention in software valuation, and this filing is unusual in disclosing
// unlevered free cash flow BOTH WAYS — before SBC and after it — for the same
// company on the same page. The gap is not a rounding difference: $331mm
// before, $198mm after, in the first year alone.
//
// So SBC is modelled as ITS OWN STREAM rather than netted into the cash flow
// line. That makes the fork structural: the pre-SBC series and the SBC
// deduction are separately stated, separately asserted, and the post-SBC
// series is their sum rather than a second hand-entered series that could
// drift from the first. Deactivating the SBC streams gives the pre-SBC
// valuation from the same model.
//
// AND THE FILING MIXES THE TWO CONVENTIONS. The explicit-period flows are
// POST-SBC; the terminal value multiple is applied to a PRE-SBC base. That is
// not an error — the exit multiple was calibrated on peers' pre-SBC unlevered
// FCF, so applying it to a pre-SBC base is the consistent choice — but it does
// mean one model has to carry both definitions at once. Reproducing the
// published grid is what establishes it; see NOTES.md.
//
// TIMING. The valuation date is 31 March 2024 and the fiscal year ends
// 31 December, so the first forecast period is a NINE-MONTH STUB (Q2-Q4 2024)
// and the full years that follow sit at 1.25, 2.25, 3.25 and 4.25 years out.
// CFDL calendars are uniform, so a monthly grid is used and each fiscal year's
// cash is placed on the date that carries its convention:
//
//     stub Q2-Q4 2024   mid of month 4    4.5/12  = 0.375
//     FY2025            end of month 14  15.0/12  = 1.25
//     FY2026            end of month 26  27.0/12  = 2.25
//     FY2027            end of month 38  39.0/12  = 3.25
//     FY2028            end of month 50  51.0/12  = 4.25
//     terminal value    end of month 56  57.0/12  = 4.75
//
// The stub is the only flow needing `mid` — 0.375 is not a month boundary. The
// mid-period convention applies to the FLOWS and NOT to the terminal value: a
// terminal value is a price struck at a point in time, so it is discounted
// whole. This is the same convention stack banker_dcf_conventions found, on a
// different filing by a different bank — which is the first independent
// confirmation of it.

version 0.1
model "saas-sbc-convention-fork"
use pack "opco" version "0.1.0"
time calendar monthly from 2024-04 for 60

entity asset target : OpCo.Asset.Enterprise

// Terminal value: an NTM unlevered-FCF multiple struck at 31 December 2028 on
// terminal-year (FY2029E) unlevered FCF. 16.0x sits inside the disclosed
// 15.0x-18.0x range; $696mm is the disclosed FY2029E figure, PRE-SBC.
assume terminal_ntm_multiple = 16.0
assume terminal_year_ufcf    = 696.0

// ---------------------------------------------------------------------------
// Unlevered free cash flow BEFORE stock-based compensation, as disclosed ($mm).
// ---------------------------------------------------------------------------

// THE ONE FIGURE THAT IS NOT DISCLOSED. The filing states FY2024E unlevered
// FCF as a full year and notes the analysis includes only Q2-Q4; it does not
// publish the quarterly split. $135.51mm is solved from the published grid —
// one unknown against twelve published enterprise values, which leaves eleven
// degrees of freedom, and every one of the twelve then lands within the
// filing's own whole-$mm rounding.
//
// It is 68.4% of the full year rather than 75%, and that is the expected
// direction: this is an annual-prepaid subscription business, so Q1 carries a
// disproportionate share of the year's cash collection. Stated post-SBC
// because the pre-SBC/SBC split of the stub is not recoverable either. See
// NOTES.md, "The one solved input".
stream opco.ufcf.stub_2024 on entity asset.target inflow currency USD {
  schedule every month mid from 2024-08 to 2024-08
  category operating.revenue.recurring
  amount = 135.51
}

stream opco.ufcf.fy2025 on entity asset.target inflow currency USD {
  schedule every month from 2025-06 to 2025-06
  category operating.revenue.recurring
  amount = 392
}

stream opco.ufcf.fy2026 on entity asset.target inflow currency USD {
  schedule every month from 2026-06 to 2026-06
  category operating.revenue.recurring
  amount = 448
}

stream opco.ufcf.fy2027 on entity asset.target inflow currency USD {
  schedule every month from 2027-06 to 2027-06
  category operating.revenue.recurring
  amount = 527
}

stream opco.ufcf.fy2028 on entity asset.target inflow currency USD {
  schedule every month from 2028-06 to 2028-06
  category operating.revenue.recurring
  amount = 616
}

// ---------------------------------------------------------------------------
// Stock-based compensation, as disclosed ($mm). Carried as a separate outflow
// on the same dates as the flow it burdens, so that pre-SBC + SBC = post-SBC
// arithmetically rather than by restatement. The published post-SBC series
// ($226 / $268 / $337 / $423) is what expected.csv checks these against.
//
// No stub line: the stub above is already net of SBC, because its pre/post
// split is not disclosed.
// ---------------------------------------------------------------------------

stream opco.sbc.fy2025 on entity asset.target outflow currency USD {
  schedule every month from 2025-06 to 2025-06
  category operating.expense.opex
  amount = 166
}

stream opco.sbc.fy2026 on entity asset.target outflow currency USD {
  schedule every month from 2026-06 to 2026-06
  category operating.expense.opex
  amount = 180
}

stream opco.sbc.fy2027 on entity asset.target outflow currency USD {
  schedule every month from 2027-06 to 2027-06
  category operating.expense.opex
  amount = 190
}

stream opco.sbc.fy2028 on entity asset.target outflow currency USD {
  schedule every month from 2028-06 to 2028-06
  category operating.expense.opex
  amount = 193
}

// The exit. Discounted whole, not mid-period — and struck on the PRE-SBC
// terminal-year figure, which is the convention fork this case exists to pin.
stream opco.exit.value on entity asset.target inflow currency USD {
  schedule every month from 2028-12 to 2028-12
  category investing.exit
  amount = inputs.terminal_year_ufcf * inputs.terminal_ntm_multiple
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.135
  }
}
```

## Verified results

Checked period by period: **11 series** across **8 periods**, each within ±0.001 of the reference.

- `opco.ufcf.stub_2024`
- `opco.ufcf.fy2025`
- `opco.ufcf.fy2026`
- `opco.ufcf.fy2027`
- `opco.ufcf.fy2028`
- `opco.sbc.fy2025`
- `opco.sbc.fy2026`
- `opco.sbc.fy2027`
- `opco.sbc.fy2028`
- `opco.exit.value`
- `model.net_cash_flow`

Summary metrics:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | 7,096 | ±1 |
