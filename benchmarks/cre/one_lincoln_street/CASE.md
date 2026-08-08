## The case

A ground-up office development in Boston: a 36-storey building funded quarter by
quarter across a 2000–2003 construction period. Equity goes in first against a
$110,738,000 commitment; once that is exhausted the construction facility draws
the balance, and interest capitalises into the loan through the build.

## The reference

A real, named transaction taught as a case study, with its funding exhibits
published. The exhibit gives a sixteen-quarter draw schedule, an 8% rate and the
equity commitment.

**Redistributable.** CC BY-NC-SA 4.0, so the source PDF is committed under
`reference/` and a reader can mark every figure against it directly.

Every number asserted is the exhibit's, derived by the model from those three
published drivers.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Declared | one curve, one state, three native streams |
| Language features | a curve read per period, declared state as a running total |
| Conventions | equity-first funding, a facility that draws only once equity depletes, capitalised construction interest |

The case runs on native streams and a declared state rather than a pack
contract: `cre.construction_stub` takes a flat draw and cannot express an
equity-first waterfall that depletes mid-quarter.

## The result

Equity contribution and construction draw reproduce **exactly to the dollar**
across all sixteen quarters. Capitalised interest reconciles to the exhibit's
stated $16,310,570 of accrued construction interest.

Asserted: three stream columns quarter by quarter, plus the interest total.

## The delta

The interest line carries a wider tolerance than the funding lines because the
exhibit rounds its quarterly interest to the dollar while compounding on
unrounded balances. The funding lines, which the exhibit states exactly, match
exactly.
