---
id: benchmark-opco-lbo-circular-interest
title: "OpCo: LBO debt schedule with average-balance interest"
slug: "/docs/examples/opco-lbo-circular-interest"
source: benchmarks/opco/lbo_circular_interest
---

# OpCo: LBO debt schedule with average-balance interest

A leveraged buyout's debt schedule, where interest accrues on the average balance and every dollar of free cash flow sweeps against the term loan.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A sponsor buys a mid-market business for $720mm — 8.0x an LTM adjusted EBITDA of
$90mm — funded with a $275mm term loan B, $175mm of senior notes, $100mm of
subordinated notes that pay in kind for three years, a 5% management rollover and
$158.9mm of sponsor equity. The model runs the four-year hold: a 35% tax rate, a
$5mm minimum cash balance, 1% mandatory term loan amortisation, and every
remaining dollar of free cash flow sweeping against the term loan.

The case is the debt schedule. Interest accrues on the **average** of each
period's opening and closing balance, which is the standard convention and the
reason an LBO is usually said to need an iterative solver: interest depends on
the closing balance, the closing balance depends on how much cash swept the debt
down, and that cash is net of interest.

## The reference

A seven-step leveraged buyout teaching model published as a downloadable
spreadsheet, free and without registration. It solves the same schedule **by
iteration** — it ships a `CIRC` switch that turns on the spreadsheet's iterative
calculation.

It publishes a complete cash flow table: every balance, every interest line and
every cash figure, as cached values in the workbook, so the comparison is period
by period rather than against a single answer.

**Not redistributable.** The workbook carries an "All Rights Reserved" notice and
no open licence, so it is neither vendored nor wired into the test suite. It was
downloaded once outside the repository and only its output numbers were carried
across.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Declared | five curves, four states, five native streams |
| Language features | declared state with `init`/`next`, curves read by `curve_value`, native streams |
| Conventions | average-balance interest, payment-in-kind accrual, a floating rate off a published path, a 100% cash sweep |

The four states carry the debt balances: the term loan and the subordinated notes,
each with its opening value, so a stream can see both ends of a period.

## The result

**Exact.** Against the reference's own unrounded cached values, the closed form
agrees to **2.8e-14** — machine epsilon — across all sixteen balance and interest
figures.

| year | term loan balance | reference | term loan interest | reference |
|---|---:|---:|---:|---:|
| 2017 | 238.517440443 | 238.517440443 | 8.986555208 | 8.986555208 |
| 2018 | 199.519287769 | 199.519287769 | 8.979752928 | 8.979752928 |
| 2019 | 156.762561123 | 156.762561123 | 8.016341600 | 8.016341600 |
| 2020 | 120.484780576 | 120.484780576 | 7.139119049 | 7.139119049 |

Asserted: the term loan and subordinated note balances, four interest lines and
the repayment, across four years — 33 figures in total.

## The delta

There is no arithmetic delta. The largest figure anywhere in the case is
**4.5e-7**, on the final year's repayment line, and it is the engine's own
publication precision rather than a disagreement: results carry money to six
decimal places, so half of that is the tightest any case here can assert.

The loop is **linear** in the closing balance — every step affine in it, with
no products of unknowns and no thresholds — so collecting terms solves it in one
substitution, which is what the model's `next` clause does. That holds because
no constraint binds in this deal: the revolver is never drawn, the term loan
never fully repays, and minimum cash is exactly met. A deal that hit any of
those would be piecewise linear, which is a different problem.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.1}}
// A sponsor leveraged buyout's debt schedule, reconciled against a published
// teaching model that solves the same schedule BY ITERATION.
//
// WHAT THIS CASE IS FOR. "Circular interest" is the standard reason given for
// why an LBO needs an iterative solver: interest depends on the average debt
// balance, the average balance depends on how much cash swept the debt down,
// and the cash available to sweep depends on interest. Excel closes the loop
// by enabling iterative calculation; the reference model ships a literal
// `CIRC` switch in its assumptions block to turn it on and off.
//
// THE LOOP IS LINEAR, SO IT HAS AN EXACT ALGEBRAIC SOLUTION. Write the ending
// balance as the unknown and it appears on both sides only to the first power:
//
//     B(t)        = B(t-1) - LFCF(t)
//     LFCF(t)     = (1 - tax) * (EBIT(t) - interest(t)) + C(t)
//     interest(t) = rate(t) * (B(t-1) + B(t)) / 2 + K(t)
//
// Collect B(t), with k = (1 - tax) * rate(t) / 2:
//
//     B(t) = [ B(t-1) * (1 + k) - (1 - tax) * (EBIT(t) - K(t)) - C(t) ]
//            / (1 - k)
//
// That is the `next` expression on `tlb_balance`. It reproduces the reference's
// iterated answer to 2.8e-14 across every balance and every interest figure.
// Excel iterates because iterating is easier to wire into a spreadsheet than
// doing the algebra, NOT because the problem requires it. See NOTES.md.
//
// K(t) is the interest that does NOT depend on the swept balance — the
// commitment fee on the undrawn revolver, fixed-rate senior notes, the PIK
// subordinated coupon, amortised financing fees, less interest earned on the
// minimum cash balance. All of it is known before B(t) is, which is exactly
// what makes collecting B(t) legitimate.
//
// C(t) is the non-EBIT cash flow: D&A back, non-cash interest back, the
// working capital movement, less capital expenditure.
//
// PERIOD 0 IS THE TRANSACTION YEAR and carries only the funded balances, so
// every `init` below is a genuine input — the amount actually drawn — rather
// than a first-year answer copied back in. Periods 1-4 are the hold years.

version 0.1
model "lbo-circular-interest"
use pack "opco" version "0.1.0"
time calendar annual from 2016-01 for 5

entity asset target : OpCo.Asset.Enterprise

// Floating base rate, on the reference's own published path. The term loan
// reprices off it annually, which is why it is a curve and not a constant.
curve libor linear {
  2016-01: 0.005
  2017-01: 0.005
  2018-01: 0.011
  2019-01: 0.015
  2020-01: 0.0215
}

// Adjusted EBIT ($mm).
curve ebit linear {
  2016-01: 73.0
  2017-01: 76.65
  2018-01: 81.249
  2019-01: 86.93643
  2020-01: 92.1526158
}

// Depreciation and amortisation, and capital expenditure. Equal in every
// projected year — both run at 5.0% of sales — so they cancel out of free cash
// flow exactly. Carried as separate lines anyway: they cancel by coincidence
// of assumption, not by construction, and netting them would hide that.
curve dna linear {
  2016-01: 17.0
  2017-01: 17.85
  2018-01: 18.921
  2019-01: 20.24547
  2020-01: 21.4601982
}

curve capex linear {
  2016-01: 17.0
  2017-01: 17.85
  2018-01: 18.921
  2019-01: 20.24547
  2020-01: 21.4601982
}

// Increase in net working capital — a use of cash, carried negative.
curve delta_nwc linear {
  2016-01: 0.0
  2017-01: -2.9
  2018-01: -3.654
  2019-01: -4.51878
  2020-01: -4.1443668
}

// ---------------------------------------------------------------------------
// Subordinated notes: PIK for three years on the AVERAGE balance.
//
// That is a fixed point in the balance alone — nothing external enters — so it
// collapses to a constant growth factor rather than needing a solver:
//
//     B = B0 + r * (B0 + B) / 2   ->   B = B0 * (1 + r/2) / (1 - r/2)
//
// the bilinear transform. After three years the coupon turns cash and the
// balance holds flat, which is the `if` below.
// ---------------------------------------------------------------------------
entity asset sub_notes : Asset.Financial {
  balance init 100.0
          next if(time.t <= 3, prev * (1 + 0.085 / 2) / (1 - 0.085 / 2), prev)
}

// ---------------------------------------------------------------------------
// Term Loan B: the circular tranche, in closed form.
//
// The subordinated coupon appears inside this expression because it is part of
// K(t). It is restated from `prev.asset.sub_notes.balance` rather than read from the
// stream, because a stream is an output and this is an input to the recursion.
// ---------------------------------------------------------------------------
entity asset tlb : Asset.Financial {
  balance init 275.0
          next (prev * (1 + (1 - 0.35) * (curve_value("libor", time.date) + 0.03) / 2)
        - (1 - 0.35) * (curve_value("ebit", time.date)
             - (0.35 + 12.25
                + if(time.t <= 3,
                     prev.asset.sub_notes.balance * ((1 + 0.085 / 2) / (1 - 0.085 / 2) - 1),
                     0.085 * prev.asset.sub_notes.balance)
                + 1.3604166666666666 - 0.0125))
        - (curve_value("dna", time.date)
           + if(time.t <= 3,
                prev.asset.sub_notes.balance * ((1 + 0.085 / 2) / (1 - 0.085 / 2) - 1),
                0.0)
           + 1.3604166666666666
           + curve_value("delta_nwc", time.date)
           - curve_value("capex", time.date)))
       / (1 - (1 - 0.35) * (curve_value("libor", time.date) + 0.03) / 2)
}

// ---------------------------------------------------------------------------
// The reported lines. Each reads the states above; none restates a balance.
// ---------------------------------------------------------------------------

// Term loan interest on the average balance — the quantity the closed form
// exists to make computable without iterating.
stream opco.interest.term_loan on entity asset.target outflow currency USD {
  schedule every year from 2017-01 to 2020-01
  category financing.interest
  amount = (curve_value("libor", time.date) + 0.03)
           * (prev.asset.tlb.balance + asset.tlb.balance) / 2
}

// Subordinated coupon. While it is PIK it IS the balance increase, which is
// why it is written as a difference rather than recomputed — the two cannot
// disagree. Once PIK ends it is cash interest on a flat balance.
stream opco.interest.sub_notes on entity asset.target outflow currency USD {
  schedule every year from 2017-01 to 2020-01
  category financing.interest
  amount = if(time.t <= 3,
              asset.sub_notes.balance - prev.asset.sub_notes.balance,
              0.085 * asset.sub_notes.balance)
}

// $175 of senior notes at a fixed 7.0%.
stream opco.interest.senior_notes on entity asset.target outflow currency USD {
  schedule every year from 2017-01 to 2020-01
  category financing.interest
  amount = 12.25
}

// $100 of undrawn revolver commitment at 0.35%. Never drawn in this deal, so
// the line is the commitment fee alone.
stream opco.interest.undrawn_revolver on entity asset.target outflow currency USD {
  schedule every year from 2017-01 to 2020-01
  category financing.interest
  amount = 0.35
}

// Levered free cash flow, which in this structure is exactly the fall in the
// term loan balance: all of it sweeps, mandatory amortisation included.
stream opco.debt.repayment on entity asset.target outflow currency USD {
  schedule every year from 2017-01 to 2020-01
  category financing.debt_principal
  amount = prev.asset.tlb.balance - asset.tlb.balance
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

Checked period by period: **7 series** across **5 periods**, each within ±1e-6 of the reference.

- `asset.tlb.balance`
- `asset.sub_notes.balance`
- `opco.interest.term_loan`
- `opco.interest.sub_notes`
- `opco.interest.senior_notes`
- `opco.interest.undrawn_revolver`
- `opco.debt.repayment`

