---
id: benchmark-cre-one-lincoln-street-contract
title: "CRE: office development, through the pack contract"
slug: "/docs/examples/cre-one-lincoln-street-contract"
description: "The same published construction schedule as the native case, declared as one cre.construction_loan contract — equity first, the facility behind it, interest on the drawn balance."
source: benchmarks/cre/one_lincoln_street_contract
---

# CRE: office development, through the pack contract

The same published construction schedule as the native case, declared as one cre.construction_loan contract — equity first, the facility behind it, interest on the drawn balance.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

The same ground-up office development as the One Lincoln Street case beside
it: 36 storeys in Boston's Financial District, funded quarter by quarter across
the 2000–2003 build, with equity drawing first against a $110,738,000 commitment
and a construction facility taking the balance once it is exhausted.

What differs is how it is said. The other case builds the funding waterfall from
primitives. This one declares a single `cre.construction_loan` contract.

## The reference

MIT OpenCourseWare 11.431J Case Assignment 3, Exhibit 7 — the same source, the
same three published drivers: a sixteen-quarter draw schedule, an 8.00% rate
compounded quarterly, and the net equity commitment.

**Redistributable.** CC BY-NC-SA 4.0. The PDF is committed once, under the
native case's `reference/`, and both cases read the same figures from it.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Declared | one curve, one contract |
| Contracts | `cre.construction_loan` |
| Language features | a pack rule declaring a field, reading a model curve by name, and re-deriving an opening balance rather than carrying one |
| Conventions | equity-first funding, a facility drawing only once the commitment depletes, interest on the drawn balance with ratable draw timing |

The draw schedule stays a `curve` in the model rather than becoming a term. A
development's funding profile is per-deal data — sixteen published quarters
here, an S-curve or a contractor's schedule on the next deal — and all three are
the same object. What the contract adds is the funding convention.

The curve is stated ANNUALIZED — each point is the exhibit's quarterly figure
times four — and the contract divides by periods-per-year, the same convention
every annual quantity in the pack follows. On this quarterly model that divides
straight back to the published number. It matters because a curve is a level: a
step curve returns its last point on every date, so a schedule stated as
per-period totals would be correct here and would fund three times the money if
the same deal were run monthly.

## The result

**Zero difference from the native case, in all 48 cells**, and `model.total`,
`model.npv` and `model.wal_years` agree to the last decimal. Against the exhibit
itself the residuals are therefore the native case's: equity contribution and
loan draw exact to the dollar in all sixteen quarters, interest within the
exhibit's own rounding.

The quarter that proves the contract is 2001.4, where the commitment runs out
mid-quarter and $29,430,000 of required funding splits into $10,522,000 of
equity and $18,908,000 of debt. That split is stated nowhere; it falls out of
the cumulative field crossing the commitment inside a period.

## The delta

**Nothing between the two cases.** Where they ever differ, the contract is
wrong: the language case is the reference, because it was validated against the
exhibit first and depends on no domain vocabulary.

**Against the exhibit**, the deltas are inherited and unchanged — the exhibit
rounds interest to whole thousands, and its stated debt-service total of
16,312,000 is the sum of those rounded quarterlies against the engine's
16,310,570 of exact ones.

**Interest is paid, not capitalized**, as the exhibit funds it from the equity
budget as a separate line. A capitalizing facility compounds and is a different
recurrence; the contract does not model it.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.08}}
// One Lincoln Street, Boston — the same construction period schedule as the
// primitive-built case of the same name, expressed as a PACK CONTRACT.
//
// The two cases are a matched pair and the point is that they agree. The other
// one builds the funding waterfall from primitives — a curve, a field and three
// native streams — and proves the LANGUAGE can express a depleting equity
// commitment with no domain vocabulary at all. This one declares
// `cre.construction_loan` and proves the PACK lowers to exactly that, against
// the same published exhibit and the same numbers.
//
// That ordering matters. A contract that reproduces what the language already
// validated is a convenience for whoever writes the next development model; a
// contract validated only against itself would be the pack marking its own
// homework. Neither case replaces the other, and if they ever disagree the
// contract is wrong.
//
// THE DRAW SCHEDULE STAYS A CURVE. It is per-deal data — sixteen published
// quarters here — so the contract names it rather than parameterizing its
// shape. Everything the contract adds is the funding CONVENTION: equity first,
// the facility behind it, interest on the drawn balance.

version 0.1
model "one-lincoln-street-contract"
use pack "cre" version "0.1.0"
time calendar quarterly from 2000-01 for 16

entity asset tower : CRE.Asset.RealProperty

// Exhibit 6's quarterly funding requirement, totalling $285,145,000 — stated
// as an ANNUALIZED rate, which is how a CRE contract reads a curve. Each point
// is the exhibit's quarterly figure x 4, so on this quarterly model the
// contract divides straight back to the published number. Stating per-period
// totals instead would be correct here and wrong the moment the same schedule
// were run monthly, because a curve is a level: it returns its last point on
// every date, so a quarterly figure would repeat three times a quarter.
curve required_funding step {
  2000-01: 19932000
  2000-04: 37116000
  2000-07: 33612000
  2000-10: 56460000
  2001-01: 76088000
  2001-04: 84836000
  2001-07: 92820000
  2001-10: 117720000
  2002-01: 88684000
  2002-04: 88584000
  2002-07: 67372000
  2002-10: 48476000
  2003-01: 154876000
  2003-04: 60340000
  2003-07: 39880000
  2003-10: 73784000
}

// Exhibit 7's three stated drivers, and nothing else. The equity/debt split,
// the opening balances and the interest are all derived by the pack.
contract cre.construction_loan on entity asset.tower {
  term 2000-01..2003-10
  terms {
    draw_curve        = "required_funding"
    equity_commitment = 110738000
    rate              = 0.08
    // "Funding is assumed to occur ratably throughout the quarter", so a
    // quarter's own draw earns half a quarter of interest.
    draw_accrual_fraction = 0.5
  }
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

Checked period by period: **3 series** across **16 periods** — **48 values** in all, each within ±500.0 of the reference.

- `cre.construction.equity_draw`
- `cre.construction.loan_draw`
- `cre.construction.interest`

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `domain.cre.debt_service` | 16,310,570 | ±1500 |
