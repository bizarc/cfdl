---
id: benchmark-energy-merchant-capacity
title: "Energy: merchant generator with capacity revenue"
slug: "/docs/examples/energy-merchant-capacity"
source: benchmarks/energy/merchant_capacity
---

# Energy: merchant generator with capacity revenue

A merchant generator earning both energy and capacity revenue, exposed to price rather than to a contracted offtake.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A 100 MW merchant renewable project — no contracted offtake, so it sells energy
at market prices — with a separate flat capacity payment for being available. It
claims the production tax credit over its first ten years and depreciates on the
five-year MACRS schedule. Because the production credit and the investment credit
are mutually exclusive, there is no basis reduction here: depreciation runs on
the full $100mm.

## The reference

A national laboratory's open-source project-finance model, run for the merchant
and production-credit configuration.

**Not vendored.** The tool was run once outside the repository and only its
output numbers were carried across.

## What it exercises

| | |
|---|---|
| Pack | `energy` |
| Contract types | `energy.merchant`, `energy.capacity`, `energy.ptc`, `energy.om`, `energy.debt_service`, `energy.macrs_shield`, `energy.capex` |
| Language features | pack contracts; term units on the credit rate |
| Conventions | merchant pricing with escalation, a flat capacity payment, a ten-year production credit with an inflation adjustment, MACRS on full basis |

## The result

Every asserted line agrees with the reference.

The case makes a narrower claim than its companion: `energy.merchant` and
`energy.ptc` are the same expression as `energy.ppa` with different term names,
and `energy.capacity` is a single division — so agreement shows the terms reach
the right places and the contracts compose, not that a new formula is correct.

## The delta

None on the asserted lines.

One mechanic here is genuinely new, and it is the reason the case exists. The
production credit is a **staircase**: the inflation-adjusted rate is published
rounded to the nearest tenth of a cent per kilowatt-hour, so it steps once a year
and holds. Carrying it continuously was wrong by up to 1.8% in a single year and
about −0.3% over the ten-year window — and the error alternates sign rather than
drifting, which is why it survived as long as it did. It looked like noise.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.064}}
// 100 MW-AC merchant renewable project with a capacity contract and the
// production tax credit, reconciled against the national laboratory's
// open-source project-finance model.
//
// COMPANION TO utility_pv_singleowner, AND A NARROWER CLAIM. That case
// validated the arithmetic; this one validates the WIRING. `energy.merchant`
// and `energy.ptc` are the same expression as `energy.ppa` with different term
// names, and `energy.capacity` is a single division — so agreement here shows
// the terms reach the right places and the contracts compose, not that a new
// formula is correct. NOTES.md says so; do not read it as more than that.
//
// THE ONE GENUINELY NEW THING is the production tax credit's STAIRCASE. The
// inflation-adjusted credit is published rounded to the nearest 0.1 cent per
// kWh, so it steps once a year and holds. The pack carried it continuously
// until now — wrong by up to 1.8% in a single year, alternating sign, which is
// why it read as noise. This case asserts the rounded path at a non-zero
// escalation, which is precisely the case that used to fail.
//
// Investment credit and production credit are mutually exclusive, so unlike the
// PV case there is no basis reduction here: MACRS runs on the full $100m.

version 0.1
model "merchant-capacity"
use pack "energy" version "0.1.0"

time calendar annual from 2025-01 for 26

entity asset wind : Energy.Asset.GenerationFacility

contract energy.capex on entity asset.wind {
  term 2025-01..2025-01
  terms { amount = 100000000 }
}

// Merchant energy: $45/MWh escalating 2%/yr, 0.5%/yr degradation.
contract energy.merchant on entity asset.wind {
  term 2026-01..2050-01
  terms {
    mwh_year = 250000
    price = 45
    price_escalation = 0.02
    degradation = 0.005
  }
}

// A flat capacity contract — no escalation, which is what the rule supports.
contract energy.capacity on entity asset.wind {
  term 2026-01..2050-01
  terms { payment_year = 4000000 }
}

// Production tax credit: $27.50/MWh base, 2.5%/yr inflation adjustment, ten
// years statutory. round_step = 0.10 is the rule's default and is the
// statutory 0.1 c/kWh tick stated on this rule's $/MWh basis.
contract energy.ptc on entity asset.wind {
  term 2026-01..2035-01
  terms {
    mwh_year = 250000 "MWh/yr"
    credit_per_mwh = 27.50 "USD/MWh"
    escalation = 0.025
    degradation = 0.005
  }
}

contract energy.om on entity asset.wind {
  term 2026-01..2050-01
  terms {
    om_year = 1500000
    escalation = 0.02
  }
}

contract energy.debt_service on entity asset.wind {
  term 2026-01..2043-01
  terms {
    rate = 0.06
    term_months = 216
    principal = 60000000
  }
}

// Full basis: no investment credit was taken, so nothing reduces it.
contract energy.macrs_shield on entity asset.wind {
  term 2026-01..2050-01
  terms {
    basis = 100000000
    tax_rate = 0.21
    life = 5
  }
}
```

## Run configuration

```json
{"deterministic":{"annual_discount_rate":0.064}}
```

## Verified results

Checked period by period: **6 series** across **14 periods**, each within ±0.001 of the reference.

- `energy.merchant.revenue`
- `energy.capacity.revenue`
- `energy.ptc.credit`
- `energy.om.expense`
- `energy.debt.service`
- `energy.macrs.shield`

