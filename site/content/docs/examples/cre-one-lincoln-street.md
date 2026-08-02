---
id: benchmark-cre-one-lincoln-street
title: "cre: one lincoln street"
slug: "/docs/examples/cre-one-lincoln-street"
source: benchmarks/cre/one_lincoln_street
---

# cre: one lincoln street

One Lincoln Street, Boston — construction period funding and interest, quarter by quarter across the 2000-2003 build. A real, named transaction taught as a case with its exhibits published, and freely redistributable under CC BY-NC-SA 4.0 — so the source PDF is committed under reference/ and a reader can mark every figure directly. EXTERNAL, ALL OF IT. Every number in expected.csv is the exhibit's, and every one is DERIVED by the model from three published drivers: a sixteen-quarter draw schedule, an 8% rate, and a $110,738,000 equity commitment. Nothing is fitted and nothing is fed back in.   equity.contribution         worst 0.00   exact to the dollar   loan.construction_draw      worst 0.00   exact to the dollar   loan.construction_interest  worst  480   see below period_tolerance = 500 — the exhibit rounds interest to whole thousands, so half a thousand is the tightest bound it can support. The two funding columns sit at zero regardless. Confirmed binding: at 400 the case fails. domain.cre.debt_service carries a wider tolerance for a stated reason. The exhibit's own total, 16,312,000, is the sum of its ROUNDED quarterly figures; the engine sums the exact ones and gets 16,310,570. The 1,430 gap is nine quarters of rounding, not a modelling difference.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The model

```cfdl
// One Lincoln Street, Boston — the construction period interest schedule.
//
// A real, named transaction: a 36-storey, ~1 million SF office development in
// Boston's Financial District, taught as a case with its exhibits published.
// This model reconciles Exhibit 7, the construction period funding and
// interest schedule, quarter by quarter across the whole 2000-2003 build.
//
// WHY THIS EXHIBIT RATHER THAN THE OPERATING PRO FORMA. Exhibit 5 publishes an
// eleven-year stabilised pro forma, but the lease-level assumptions that drive
// its rent, absorption and reimbursement lines are not published — so most of
// it could only be asserted by feeding in the answers. Exhibit 7 publishes the
// DRIVERS (a sixteen-quarter draw schedule, an 8% rate, a $110,738,000 equity
// commitment) and every line they produce. That is the difference this
// programme keeps running into, and it is why this case takes Exhibit 7.
//
// THE MECHANIC IS A DEPLETING COMMITMENT. Equity funds the project first and
// the construction loan takes over only once the equity is exhausted — which
// happens mid-quarter in 2001.4, splitting that quarter 10,522,000 / 18,908,000.
// Cumulative required funding is a running total, so it is a declared state;
// everything else follows from it in closed form:
//
//   cum(t)        running total of required funding through t
//   debt draw     min(required, max(0, cum - equity))
//   equity draw   required - debt draw
//   opening debt  max(0, cum - required - equity)
//   interest      (opening debt + debt draw / 2) * rate / 4
//
// The halved draw is the exhibit's own stated convention: funding is "assumed
// to occur ratably throughout the quarter", so a quarter's own draw earns half
// a quarter of interest.
//
// Interest is PAID, not capitalised — the exhibit's closing balance is opening
// plus draw with no interest added, and the interest is funded from the equity
// budget as a stated line. Modelling it as capitalised would compound it.

version 0.1
model "one-lincoln-street"
time calendar quarterly from 2000-01 for 16

entity asset tower

assume equity_commitment  = 110738000     // Exhibit 7, net construction period equity
assume construction_rate  = 0.08          // 8.00%, compounded quarterly

// Exhibit 6's quarterly funding requirement, totalling $285,145,000.
curve required_funding step {
  2000-01: 4983000
  2000-04: 9279000
  2000-07: 8403000
  2000-10: 14115000
  2001-01: 19022000
  2001-04: 21209000
  2001-07: 23205000
  2001-10: 29430000
  2002-01: 22171000
  2002-04: 22146000
  2002-07: 16843000
  2002-10: 12119000
  2003-01: 38719000
  2003-04: 15085000
  2003-07: 9970000
  2003-10: 18446000
}

// Cumulative required funding through and including this quarter.
state cum_required {
  init curve_value("required_funding", time.date)
  next prev + curve_value("required_funding", time.date)
}

// Equity funds first, until the commitment is exhausted.
stream equity.contribution on entity asset.tower inflow currency USD {
  schedule every quarter from 2000-01 to 2003-10
  amount = curve_value("required_funding", time.date)
           - min(curve_value("required_funding", time.date),
                 max(0, state.cum_required - inputs.equity_commitment))
}

// The construction loan takes the balance.
stream loan.construction_draw on entity asset.tower inflow currency USD {
  schedule every quarter from 2000-01 to 2003-10
  amount = min(curve_value("required_funding", time.date),
               max(0, state.cum_required - inputs.equity_commitment))
}

// Interest on the opening balance plus half of this quarter's draw.
// Named to match what domain.cre.debt_service reads.
stream loan.construction_interest on entity asset.tower outflow currency USD {
  schedule every quarter from 2000-01 to 2003-10
  amount = (max(0, state.cum_required - curve_value("required_funding", time.date) - inputs.equity_commitment)
            + min(curve_value("required_funding", time.date),
                  max(0, state.cum_required - inputs.equity_commitment)) / 2)
           * inputs.construction_rate / 4
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.08
  }
}
```

## Verified results

| Metric | Value | Tolerance |
|---|---:|---:|
| `domain.cre.debt_service` | 16,310,570 | ±1500 |
