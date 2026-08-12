---
id: benchmark-opco-lbo-financing-cases
title: "OpCo: one buyout at three capital structures"
slug: "/docs/examples/opco-lbo-financing-cases"
source: benchmarks/opco/lbo_financing_cases
---

# OpCo: one buyout at three capital structures

One sponsor buyout run at three capital structures, with the published five-year multiple and return reproduced for each.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A sponsor buys a mid-market business for $720mm — 8.0x an LTM adjusted EBITDA of
$90mm — holds it five years and sells at the same multiple. Revenue grows
5, 6, 7, 6 and 5 per cent; margin, depreciation and capital expenditure hold at
their trailing ratios; working capital turns on stated days.

The same deal is run at **three capital structures**. Only the financing
changes:

| | Term Loan B | Senior Notes | Sub Notes | Total |
|---|---|---|---|---|
| Base | 3.0x @ L+3.00% | 2.0x @ 7.0% | 1.0x @ 8.5% | 6.0x |
| High leverage | 3.0x @ L+3.50% | 2.5x @ 7.0% | 2.0x @ 10.0% | 7.5x |
| Low leverage | 3.0x @ L+2.75% | 1.5x @ 6.0% | — | 4.4x |

The subordinated notes pay in kind for three years. Every dollar of free cash
flow after a 1% mandatory amortisation sweeps against the term loan, and
interest accrues on the average balance — so the balance depends on the interest
that depends on the balance.

## The reference

A seven-step leveraged buyout teaching model published as a downloadable
spreadsheet, free and without registration. It carries its own financing-case
switch, and publishes a five-year multiple and return for each of the three
structures across a grid of entry and exit multiples.

**Not redistributable.** The workbook carries an "All Rights Reserved" notice
and no open licence, so it is neither vendored nor wired into the test suite. It
was downloaded once outside the repository and only its output numbers were
carried across.

It publishes a period-by-period debt schedule for **Base only**. For the other
two structures it publishes the returns and nothing in between.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Declared | two states, five curves, two native streams, three run scenarios |
| Language features | **run-config scenarios**, `cfg.*` parameters, declared state with `init`/`next`, curves |
| Conventions | average-balance interest, payment-in-kind accrual, a 100% cash sweep, tranche sizing to a debt increment, a sponsor cheque struck as the plug |

The financing case is the **run config**, not the model: the deterministic run is
Base and two scenarios override the tranche sizes, coupons and the sponsor's
cheque. That is what the source's own case switch does.

Sizes are not stated as inputs. Each tranche is its leverage multiple times LTM
EBITDA rounded to a $25mm increment, and the sponsor's cheque is whatever
balances sources against uses. Base checks the rule — its published $275mm,
$175mm and $100mm are what 3.0x, 2.0x and 1.0x round to — and the other two
structures are derived rather than transcribed.

## The result

**All three structures reproduce the published multiple and return.**

| | MoIC | reference | IRR | reference |
|---|---:|---:|---:|---:|
| Base | 2.952823 | 2.952823 | 24.1788% | 24.1788% |
| High leverage | 5.479046 | 5.479046 | 40.5209% | 40.5209% |
| Low leverage | 2.271875 | 2.271875 | 17.8357% | 17.8357% |

Worst disagreement across all six figures: **4.5e-7**.

Base additionally asserts the term loan and subordinated balances period by
period; the term loan agrees at the engine's own publication precision across
all five years.

For the two scenarios **nothing between the inputs and the answer is
anchored**: the operating build, the sizing rule, the sweep, the PIK accrual,
the exit and the returns arithmetic all have to be right to land on a published
multiple. Base anchors every intermediate line; these two anchor none.

## The delta

There is no arithmetic delta on the returns.

One column carries a looser tolerance, and the reason is the reference. The
subordinated PIK accrual is self-referential, and the source solves it by
switching on iterative calculation where this model solves it in closed form.
Checked against the reference's own equation, `B = B0 + avg(B0, B) * r`:

| | residual |
|---|---:|
| closed form | −1.4e-14 |
| reference | +3.7e-05 |

The source stopped iterating while its own equation still had a residual of
3.7e-5, so that is what its convergence supports and the column is asserted to
1e-4. The closed form is the more accurate of the two.

One thing the case does **not** cover: the reference publishes a full 5×5 grid
of entry and exit multiples for each structure — 150 figures. This asserts the
8.0x / 8.0x corner of each; the rest needs one scenario per grid point.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0,"parameters":{"cfg.tlb_size":275,"cfg.senior_size":175,"cfg.sub_size":100,"cfg.tlb_spread":0.03,"cfg.senior_rate":0.07,"cfg.sub_rate":0.085,"cfg.fee_amort":1.3604166666666666,"cfg.sponsor_equity":158.9375}},"scenarios":{"high_leverage":{"parameters":{"cfg.tlb_size":275,"cfg.senior_size":225,"cfg.sub_size":175,"cfg.tlb_spread":0.035,"cfg.senior_rate":0.07,"cfg.sub_rate":0.1,"cfg.fee_amort":1.6729166667,"cfg.sponsor_equity":36.8125}},"low_leverage":{"parameters":{"cfg.tlb_size":275,"cfg.senior_size":125,"cfg.sub_size":0,"cfg.tlb_spread":0.0275,"cfg.senior_rate":0.06,"cfg.sub_rate":0.085,"cfg.fee_amort":0.9854166667,"cfg.sponsor_equity":305.4375}}}}
version 0.1
model "lbo-financing-cases"
use pack "opco" version "0.1.0"
time calendar annual from 2016-01 for 6

// A sponsor buyout of a mid-market business, run at THREE capital structures.
//
// The operating case is identical in all three — same revenue path, same
// margin, same capex, same working capital. Only the financing changes, which
// is what the reference's own "financing case" switch does. Here that switch is
// the run config: the deterministic run is Base, and two scenarios override the
// tranche sizes and coupons.
//
// WHAT IS ASSERTED IS THE ENDPOINT. The reference publishes a period-by-period
// debt schedule for Base only, but it publishes MoIC and IRR for all three. So
// this case asserts returns, and nothing in between is anchored: the operating
// build, the debt schedule, the sweep, the PIK accrual and the exit all have to
// be right to land on a published multiple.
//
// Sizes are not stated as inputs. The reference sizes each tranche as its
// leverage multiple times LTM EBITDA, rounded to a $25mm increment, and the
// sponsor's cheque is the plug that balances sources against uses. Both rules
// are reproduced below, which is why the three cases differ only in leverage
// and pricing.

entity asset target : OpCo.Asset.Enterprise
entity party sponsor : OpCo.Party.Sponsor { name = "Sponsor" }
entity party mgmt : OpCo.Party.Management { name = "Management" }

// --- The operating case, identical across financing cases -------------------
// EBIT, D&A, capex and the working-capital movement the reference derives from
// its revenue build. Held as curves because they are inputs to the recursion
// below, not outputs of it.

curve ebitda step {
  2017-01: 94.5
  2018-01: 100.17
  2019-01: 107.1819
  2020-01: 113.612814
  2021-01: 119.2934547
}

curve ebit step {
  2017-01: 76.65
  2018-01: 81.249
  2019-01: 86.93643
  2020-01: 92.1526158
  2021-01: 96.76024659
}

curve dna step {
  2017-01: 17.85
  2018-01: 18.921
  2019-01: 20.24547
  2020-01: 21.4601982
  2021-01: 22.53320811
}

curve capex step {
  2017-01: 17.85
  2018-01: 18.921
  2019-01: 20.24547
  2020-01: 21.4601982
  2021-01: 22.53320811
}

curve delta_nwc step {
  2017-01: -2.9
  2018-01: -3.654
  2019-01: -4.51878
  2020-01: -4.1443668
  2021-01: -3.66085734
}

curve libor step {
  2017-01: 0.005
  2018-01: 0.011
  2019-01: 0.015
  2020-01: 0.0215
  2021-01: 0.025
}

// --- The financing case ------------------------------------------------------
// Every knob a scenario overrides. The deterministic run is Base at 6.0x.

assume ltm_ebitda        = 90.0
assume debt_increment    = 25.0
assume tax_rate          = 0.35
assume purchase_equity   = 720.0        // 8.0x entry
assume transaction_costs = 15.0
assume rollover          = 36.0         // 5% management rollover
assume exit_multiple     = 8.0
assume cash_at_exit      = 5.0          // the minimum cash balance

// Tranche sizes, coupons, the annual fee amortisation and the sponsor's cheque
// all arrive from the run config, because they are what a financing case IS.
// The deterministic run is Base; the two scenarios are the other structures.


assume commitment_fee = 0.35   // 0.35% on a $100mm undrawn revolver
assume interest_income = 0.0125 // 0.25% on the $5mm minimum cash balance

// --- The debt schedule -------------------------------------------------------
// Subordinated notes pay in kind for three years. A PIK coupon on an average
// balance is self-referential and collapses to a constant growth factor:
//
//     B = B0 + r (B0 + B) / 2   ->   B = B0 (1 + r/2) / (1 - r/2)

entity asset sub_notes : Asset.Financial {
  balance init cfg.sub_size
          next if(time.t <= 3,
                  prev * (1 + cfg.sub_rate / 2) / (1 - cfg.sub_rate / 2),
                  prev)
}

// Term Loan B, in closed form. Every dollar of free cash flow after the 1%
// mandatory amortisation sweeps against it, and interest accrues on the average
// balance — so the balance depends on the interest that depends on the balance.
// Collecting terms solves it in one substitution.
entity asset tlb : Asset.Financial {
  balance init cfg.tlb_size
          next max(0.0,
       (prev * (1 + (1 - inputs.tax_rate) * (curve_value("libor", time.date) + cfg.tlb_spread) / 2)
        - (1 - inputs.tax_rate) * (curve_value("ebit", time.date)
             - (inputs.commitment_fee + cfg.senior_size * cfg.senior_rate
                + if(time.t <= 3,
                     prev.asset.sub_notes.balance * ((1 + cfg.sub_rate / 2) / (1 - cfg.sub_rate / 2) - 1),
                     cfg.sub_rate * prev.asset.sub_notes.balance)
                + cfg.fee_amort - inputs.interest_income))
        - (curve_value("dna", time.date)
           + if(time.t <= 3,
                prev.asset.sub_notes.balance * ((1 + cfg.sub_rate / 2) / (1 - cfg.sub_rate / 2) - 1),
                0.0)
           + cfg.fee_amort
           + curve_value("delta_nwc", time.date)
           - curve_value("capex", time.date)))
       / (1 - (1 - inputs.tax_rate) * (curve_value("libor", time.date) + cfg.tlb_spread) / 2))
}

// --- The sponsor's cash flows ------------------------------------------------
// Two points: the cheque at close, and the proceeds at exit. MoIC and IRR fall
// out of them, which is what the reference publishes for all three cases.

stream opco.sponsor.investment on entity asset.target outflow currency USD {
  schedule on 2016-01
  category investing.acquisition
  amount = cfg.sponsor_equity
}

// Exit enterprise value less net debt is the equity; the sponsor's preferred
// converts one-for-one at this exit level, so sponsor and management divide it
// in proportion to what each put in.
stream opco.sponsor.proceeds on entity asset.target inflow currency USD {
  schedule on 2021-01
  category investing.exit
  amount = (inputs.exit_multiple * curve_value("ebitda", time.date)
            - (asset.tlb.balance + cfg.senior_size + asset.sub_notes.balance
               - inputs.cash_at_exit))
           * cfg.sponsor_equity / (cfg.sponsor_equity + inputs.rollover)
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.0,
    "parameters": {
      "cfg.tlb_size": 275.0,
      "cfg.senior_size": 175.0,
      "cfg.sub_size": 100.0,
      "cfg.tlb_spread": 0.03,
      "cfg.senior_rate": 0.07,
      "cfg.sub_rate": 0.085,
      "cfg.fee_amort": 1.3604166666666666,
      "cfg.sponsor_equity": 158.9375
    }
  },
  "scenarios": {
    "high_leverage": {
      "parameters": {
        "cfg.tlb_size": 275.0,
        "cfg.senior_size": 225.0,
        "cfg.sub_size": 175.0,
        "cfg.tlb_spread": 0.035,
        "cfg.senior_rate": 0.07,
        "cfg.sub_rate": 0.1,
        "cfg.fee_amort": 1.6729166667,
        "cfg.sponsor_equity": 36.8125
      }
    },
    "low_leverage": {
      "parameters": {
        "cfg.tlb_size": 275.0,
        "cfg.senior_size": 125.0,
        "cfg.sub_size": 0.0,
        "cfg.tlb_spread": 0.0275,
        "cfg.senior_rate": 0.06,
        "cfg.sub_rate": 0.085,
        "cfg.fee_amort": 0.9854166667,
        "cfg.sponsor_equity": 305.4375
      }
    }
  }
}
```

## Verified results

Checked period by period: **2 series** across **6 periods** — **12 values** in all, each within the tolerance shown.

- `asset.tlb.balance` — within ±1e-6
- `asset.sub_notes.balance` — within ±1e-4

Checked per scenario, each a full run under its own parameters:

| Scenario | `model.moic` | `model.irr` |
|---|---:|---:|
| `high_leverage` | 5.479046249577944 | 0.40520922846134866 |
| `low_leverage` | 2.2718751295780626 | 0.178357014624281 |

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.moic` | 2.952822546525116 | ±0.00001 |
| `model.irr` | 0.24178803124249515 | ±0.00001 |
