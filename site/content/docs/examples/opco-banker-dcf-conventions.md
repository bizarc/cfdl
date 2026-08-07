---
id: benchmark-opco-banker-dcf-conventions
title: "opco: banker dcf conventions"
slug: "/docs/examples/opco-banker-dcf-conventions"
source: benchmarks/opco/banker_dcf_conventions
---

# opco: banker dcf conventions

An operating company discounted cash flow built to standard banking conventions, from revenue through unlevered free cash flow to enterprise value.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.10375}}
// A sell-side banker's DCF of an enterprise-software business, reconciled
// against the disclosed analysis in a public merger filing.
//
// The filing discloses everything needed to reproduce it: the unlevered free
// cash flow build-up line by line, the discount rate, the terminal multiple,
// the discounting convention, the dilution assumption, and the answer. That is
// unusual and it is why this case exists — the figures below are a third
// party's, not ours.
//
// TIMING IS THE WHOLE DIFFICULTY. The valuation date is 30 September 2024;
// the fiscal year ends 30 June. So the first forecast period is a NINE-MONTH
// STUB (Oct 24 - Jun 25) and the full years that follow sit at 1.25, 2.25,
// 3.25 and 4.25 years out, not 1, 2, 3, 4. CFDL calendars are uniform, so
// there is no stub period; a monthly grid is used instead and each fiscal
// year's cash placed as a one-shot flow on the date that carries its
// convention. Every exponent then lands exactly:
//
//     stub Q2-Q4 FY25   mid of month 4    4.5/12  = 0.375
//     FY26              end of month 14  15.0/12  = 1.25
//     FY27              end of month 26  27.0/12  = 2.25
//     FY28              end of month 38  39.0/12  = 3.25
//     FY29              end of month 50  51.0/12  = 4.25
//     terminal value    end of month 56  57.0/12  = 4.75
//
// Note the stub is the only flow needing `mid` — 0.375 is not a month
// boundary. The rest are, which is luck rather than design. See NOTES.md.
//
// Each flow is written as a one-occurrence `every month`, not as
// `schedule on <date>`. A one-shot discounts from its period's OPEN, and only
// a pack lowering rule can move it to the period's close; there is no surface
// syntax for that. A single-period `every` is an ordinary annuity, so it falls
// at the period's end, which is the placement these flows need. See NOTES.md.
//
// The mid-period convention applies to the FLOWS and NOT to the terminal
// value: a terminal value is a price struck at a point in time, so it is
// discounted whole. The filing's own figures confirm the asymmetry.

version 0.1
model "banker-dcf-conventions"
use pack "opco" version "0.1.0"
time calendar monthly from 2024-10 for 60

entity asset target : OpCo.Asset.Enterprise

// Cumulative dilution from equity awards, applied as a haircut on each flow.
// The filing states ~2% cumulative over the projection, ramping 0.5%/yr.
assume dilution_fy26 = 0.005
assume dilution_fy27 = 0.010
assume dilution_fy28 = 0.015
assume dilution_fy29 = 0.020
assume dilution_term = 0.020

// Terminal value: a forward (NTM) multiple on the terminal year's unlevered
// free cash flow, struck as of the start of that year. 25.0x is the midpoint
// of the disclosed 20.0x-30.0x range; $891mm is the disclosed FY30E UFCF.
assume terminal_ntm_multiple = 25.0
assume terminal_year_ufcf    = 891.0

// ---------------------------------------------------------------------------
// Unlevered free cash flow, as disclosed ($mm). One flow per fiscal period.
// ---------------------------------------------------------------------------

stream opco.ufcf.stub_fy25 on entity asset.target inflow currency USD {
  schedule every month mid from 2025-02 to 2025-02
  category operating.revenue.recurring
  amount = 367
}

stream opco.ufcf.fy26 on entity asset.target inflow currency USD {
  schedule every month from 2025-12 to 2025-12
  category operating.revenue.recurring
  amount = 481 * (1 - inputs.dilution_fy26)
}

stream opco.ufcf.fy27 on entity asset.target inflow currency USD {
  schedule every month from 2026-12 to 2026-12
  category operating.revenue.recurring
  amount = 502 * (1 - inputs.dilution_fy27)
}

stream opco.ufcf.fy28 on entity asset.target inflow currency USD {
  schedule every month from 2027-12 to 2027-12
  category operating.revenue.recurring
  amount = 639 * (1 - inputs.dilution_fy28)
}

stream opco.ufcf.fy29 on entity asset.target inflow currency USD {
  schedule every month from 2028-12 to 2028-12
  category operating.revenue.recurring
  amount = 736 * (1 - inputs.dilution_fy29)
}

stream opco.exit.value on entity asset.target inflow currency USD {
  schedule every month from 2029-06 to 2029-06
  category investing.exit
  amount = inputs.terminal_year_ufcf * inputs.terminal_ntm_multiple
           * (1 - inputs.dilution_term)
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.10375
  }
}
```

## Verified results

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | 15,764 | ±1 |
