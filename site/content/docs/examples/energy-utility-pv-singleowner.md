---
id: benchmark-energy-utility-pv-singleowner
title: "energy: utility pv singleowner"
slug: "/docs/examples/energy-utility-pv-singleowner"
source: benchmarks/energy/utility_pv_singleowner
---

# energy: utility pv singleowner

A utility-scale photovoltaic project in a single-owner structure, carrying its own tax position rather than allocating to an investor.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

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

// 100 MW-AC / 250 GWh year one — a 28.5% net capacity factor.
contract energy.capex on entity asset.pv {
  term 2025-01..2025-01
  terms { amount = 100000000 }
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
contract energy.debt_service on entity asset.pv {
  term 2026-01..2043-01
  terms {
    rate = 0.06
    term_months = 216
    principal = 60000000
  }
}

// 30% ITC on the full installed cost, taken in the first operating year.
contract energy.itc on entity asset.pv {
  term 2026-01..2026-01
  terms { credit = 30000000 }
}

// 5-year MACRS. The basis is 85m, not 100m: taking the ITC reduces the
// depreciable basis by half the credit (100m - 0.5 * 30m). The pack takes
// `basis` as an input rather than deriving it, so the reduction is stated
// here — see NOTES.md.
contract energy.macrs_shield on entity asset.pv {
  term 2026-01..2050-01
  terms {
    basis = 85000000
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

| Metric | Value | Tolerance |
|---|---:|---:|
