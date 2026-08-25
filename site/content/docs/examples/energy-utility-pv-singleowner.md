---
id: benchmark-energy-utility-pv-singleowner
title: "Energy: utility-scale PV, single owner"
slug: "/docs/examples/energy-utility-pv-singleowner"
description: "A utility-scale photovoltaic project in a single-owner structure, carrying its own tax position rather than allocating to an investor."
source: benchmarks/energy/utility_pv_singleowner
---

# Energy: utility-scale PV, single owner

A utility-scale photovoltaic project in a single-owner structure, carrying its own tax position rather than allocating to an investor.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A 100 MW-AC utility-scale photovoltaic project in a single-owner structure,
generating 250 GWh in its first year. It sells under a 25-year power purchase
agreement at $45/MWh escalating 2% a year, against 0.5% annual module
degradation. $60m of debt amortizes over 18 years at 6%. A 30% investment tax
credit lands in the first operating year, and the project depreciates on the
five-year MACRS schedule, on a basis reduced by half the credit.

Single owner means the project carries its own tax position rather than
allocating it to an investor.

## The reference

A national laboratory's open-source project-finance model, the standard tool
for this structure. Being open source, a disagreement can be traced to a
specific formula.

**Not vendored.** The tool was run once outside the repository and only its
output numbers were carried across, so nothing about it is a build dependency.

## What it exercises

| | |
|---|---|
| Pack | `energy` |
| Contract types | `energy.ppa`, `energy.om`, `energy.debt_service`, `energy.itc`, `energy.macrs_shield`, `energy.capex` |
| Language features | pack contracts across a full capital structure; term units |
| Conventions | production degradation, price escalation, level-pay debt, an investment tax credit, MACRS with a basis reduction |

More of the energy pack's contract surface than any other case.

## The result

Every asserted line agrees, worst **9.1e-7 dollars** across all 26 periods and
all four escalating streams.

Asserted: six stream columns at anchor periods — the MACRS table through its
final year and the zero after it, the debt tenor and its cliff at periods 18 and
19, and the compounding at the end of the hold.

## The delta

The residual is float noise, not convention. Anchors rather than every period
because escalation and degradation compound: a convention error shows up in
every period after the first and grows, so the anchors bracket where it would
appear.

The reference states its operations and maintenance escalation as a *real* rate
carried on top of an inflation assumption, while the pack's escalation term is
nominal. The case runs at zero inflation, where the two coincide exactly.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.064}}
version 0.1
model "utility-pv-singleowner"
use pack "energy" version "0.1.0"

// 100 MW-AC utility-scale PV, single-owner project finance, reconciled against
// the national laboratory's open-source project-finance model.
//
// Period 0 is the construction year and carries only capex, so periods 1..25
// line up index-for-index with the reference's annual operating years. That
// alignment is the point: no cadence translation sits between the two models,
// so a divergence is a convention difference and nothing else.
//
// Every escalation here is NOMINAL, which is what the pack's `escalation` term
// means. The reference states O&M escalation as a REAL rate carried on top of
// an inflation assumption, so this case is run at zero inflation, where the two
// coincide exactly. See NOTES.md — the conversion is additive, not compounded.
time calendar annual from 2025-01 for 26

entity asset pv : Energy.Asset.GenerationFacility

// THE DEAL'S TWO TAX INPUTS, STATED ONCE. Installed cost and the credit rate
// drive the capex, the credit and the depreciable basis, so changing either
// propagates instead of leaving a hand-computed figure behind.
assume installed_cost = 100000000
assume itc_rate       = 0.30

// 100 MW-AC / 250 GWh year one — a 28.5% net capacity factor.
contract energy.capex on entity asset.pv {
  term 2025-01..2025-01
  terms { amount = inputs.installed_cost }
}

// $45/MWh (4.5 c/kWh) escalating 2%/yr; 0.5%/yr module degradation.
contract energy.ppa on entity asset.pv {
  term 2026-01..2050-01
  terms {
    mwh_year = 250000 "MWh/yr"
    ppa_price = 45 "USD/MWh"
    escalation = 0.02
    degradation = 0.005
  }
}

// $15/kW-yr fixed O&M on 100,000 kW, escalating 2%/yr.
contract energy.om on entity asset.pv {
  term 2026-01..2050-01
  terms {
    om_year = 1500000
    escalation = 0.02
  }
}

// 60% debt at 6% over 18 years, level annual payments.
// funded_at_close = 0: the reference's cash flow starts post-financing —
// it nets operations against debt service and never books the draw — so the
// proceeds the contract funds by default are excluded to state what the
// source states.
contract energy.debt_service on entity asset.pv {
  term 2026-01..2043-01
  terms {
    rate = 0.06
    term_months = 216
    principal = 60000000
    funded_at_close = 0
  }
}

// The ITC on the full installed cost, taken in the first operating year.
contract energy.itc on entity asset.pv {
  term 2026-01..2026-01
  terms { credit = inputs.installed_cost * inputs.itc_rate }
}

// 5-year MACRS on the REDUCED basis: taking the investment credit removes
// half of it from what may be depreciated, so 100m becomes 85m here.
//
// The reduction is STATED, not pre-computed. The pack takes `basis` as an
// input rather than deriving it, because basis adjustments are jurisdictional
// and a wrong default is worse than none — but that is a reason for the model
// to say which adjustment applies, not a reason to paste in the answer. A
// hardcoded 85,000,000 goes stale the moment the cost or the credit rate
// moves, and nothing objects: the full basis overstates the shield by 17.6%.
contract energy.macrs_shield on entity asset.pv {
  term 2026-01..2050-01
  terms {
    basis = inputs.installed_cost * (1 - 0.5 * inputs.itc_rate)
    tax_rate = 0.21
    life = 5
  }
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.064
  }
}
```

## Verified results

Checked period by period: **6 series** across **13 periods** — **59 values** in all, each within ±0.01 of the reference.

- `energy.ppa.revenue`
- `energy.om.expense`
- `domain.energy.debt_service_periodic`
- `energy.macrs.shield`
- `energy.itc.credit`
- `energy.capex.outlay`

