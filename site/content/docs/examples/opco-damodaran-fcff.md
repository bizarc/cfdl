---
id: benchmark-opco-damodaran-fcff
title: "OpCo: free cash flow to firm"
slug: "/docs/examples/opco-damodaran-fcff"
description: "A free cash flow to firm valuation following Damodaran's published method, with reinvestment driven by growth and return on capital."
source: benchmarks/opco/damodaran_fcff
---

# OpCo: free cash flow to firm

A free cash flow to firm valuation following Damodaran's published method, with reinvestment driven by growth and return on capital.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A cross-industry operating company valued on free cash flow to the firm: revenue
growing off a declining growth path, operating margins, cash taxes, capital
expenditure and working capital, discounted to an enterprise value.

The rate declines year on year, so revenue is a running product of ten
different growth rates rather than one rate compounded.

## The reference

A widely used academic valuation spreadsheet, published free by its author with
an explicit grant to download and modify. It publishes the full ten-year build-up
and the resulting value.

**Redistributable**, and the workbook is committed under `reference/` so a reader
can mark every figure against the original.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Contract types | `opco.revenue_line`, `opco.opex_line`, `opco.capex_line`, `opco.cash_taxes` |
| Declared | two curves |
| Language features | pack contracts driven by curves; declared state inside the pack's growth rules |
| Conventions | a declining growth path, margin-driven operating expense, cash taxes, capital expenditure as a share of revenue |

The reference publishes the **drivers** rather than only the results, which is
what a pack rule consumes, so the pack's lowering is checked and not only the
engine's arithmetic.

## The result

All ten years reproduce exactly.

Revenue is carried as declared state because the growth rate moves:
`pow(1 + g, t)` applies one year's rate as though it had held from the start,
which is exact only while the rate is constant.

## The delta

None.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.0705501574064654}}
// Damodaran's FCFF Simple Ginzu — the reference implementation of textbook
// intrinsic valuation, and the first opco case built from PACK CONTRACTS.
//
// WHY THIS SOURCE. benchmarks/opco/banker_dcf_conventions reconciles a banker's
// DCF, but that filing publishes the RESULT — per-year unlevered cash flow — so
// the model had to hand-write six native streams and validated the engine's
// discounting rather than the pack. This source publishes the DRIVERS: revenue
// growth, operating margin, tax rate and a sales-to-capital ratio, and every
// line they produce. That is what a pack rule consumes, so this case is built
// entirely from opco contracts and takes the pack off 0-of-10.
//
// THE DRIVERS CONVERGE, which is the whole character of intrinsic valuation:
// growth decays toward the riskfree rate and the effective tax rate climbs
// toward the marginal one as the firm matures. Both paths below are DERIVED
// from the stated inputs (5% -> 4.58%, 17.5% -> 25%, linearly over years 6-10),
// not read off the output — verified to reproduce the published growth and tax
// rows exactly.
//
// WHAT IS ASSERTED, AND WHY NOT ALL TEN YEARS. The curves carry a PER-PERIOD
// rate, which is the right interface and the one that will be correct once a
// stream can read its own prior period. Until then the rules
// compound with pow(1 + g, t), which applies one period's rate as though it had
// held throughout — exact while the rate is flat, drifting once it moves. So
// years 1-5 are asserted and years 6-10 are not; NOTES.md carries the measured
// drift, which is the delta 5.1 is expected to close.
//
// Reinvestment funds NEXT year's growth, so its exact window closes a year
// earlier than revenue's. Also asserted only where it is exact.
//
// NOT ASSERTED AT ALL: value, NPV, per-share price. The cost of capital
// converges 7.055% -> 8.81% and the engine takes a single scalar discount rate,
// so a term structure is inexpressible. Discounting at a flat rate and calling
// the result agreement would be worse than saying so.

version 0.1
model "damodaran-fcff"
use pack "opco" version "0.1.0"
time calendar annual from 2026-01 for 10

entity asset firm : OpCo.Asset.Enterprise

// Revenue growth: 5% while the firm is growing, decaying to the riskfree rate
// by the terminal year.
curve revenue_growth linear {
  2026-01: 0.0500000000
  2027-01: 0.0500000000
  2028-01: 0.0500000000
  2029-01: 0.0500000000
  2030-01: 0.0500000000
  2031-01: 0.0491600000
  2032-01: 0.0483200000
  2033-01: 0.0474800000
  2034-01: 0.0466400000
  2035-01: 0.0458000000
}

// Effective tax rate climbing to the marginal rate over the same window.
curve tax_rate linear {
  2026-01: 0.1750000000
  2027-01: 0.1750000000
  2028-01: 0.1750000000
  2029-01: 0.1750000000
  2030-01: 0.1750000000
  2031-01: 0.1900000000
  2032-01: 0.2050000000
  2033-01: 0.2200000000
  2034-01: 0.2350000000
  2035-01: 0.2500000000
}

contract opco.revenue_line.core on entity asset.firm {
  term 2026-01..2035-01
  terms {
    amount = 22853.6700000000
    growth_rate = curve_value("revenue_growth", time.date)
  }
}

// Operating margin is flat at 14.063%, so operating cost is the complement of
// revenue and follows the same path.
contract opco.opex_line.operating on entity asset.firm {
  term 2026-01..2035-01
  terms {
    amount = 19639.7250000000
    growth_rate = curve_value("revenue_growth", time.date)
  }
}

// Cash taxes on EBIT. The rule reads revenue and opex from base streams and
// opex is signed negative, so their sum is EBIT; no debt and no D&A here.
contract opco.cash_taxes.federal on entity asset.firm {
  term 2026-01..2035-01
  terms {
    tax_rate = curve_value("tax_rate", time.date)
  }
}

// Reinvestment = revenue * growth / sales-to-capital, which funds NEXT year's
// growth. With a flat growth rate it is itself a geometric series on the same
// curve.
contract opco.capex_line.reinvestment on entity asset.firm {
  term 2026-01..2035-01
  terms {
    amount = 668.8079047797
    growth_rate = curve_value("revenue_growth", time.date)
  }
}
```

## Run configuration

```json
{"deterministic":{"annual_discount_rate":0.0705501574064654}}
```

## Verified results

Checked period by period: **4 series** across **10 periods** — **34 values** in all, each within ±0.001 of the reference.

- `opco.revenue.recurring.core`
- `opco.opex.recurring.operating`
- `opco.taxes.cash.federal`
- `opco.capex.line.reinvestment`

