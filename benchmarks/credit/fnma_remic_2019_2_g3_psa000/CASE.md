## The case

No prepayments at all — and not the deal's own loans. For this column the supplement swaps the collateral for its stated alternative: new loans with 360-month original and remaining terms at 7.50%, so the pool amortises on pure schedule for thirty years and the strip to the 5.00% pass-through is 2.50% rather than 0.451%. The class takes until January 2049 to retire, and the table publishes a value in every one of the thirty years.

The deal is Security Group 3 of Fannie Mae REMIC Trust 2019-2: a $148,372,434
pass-through with the coupon stripped between a principal class and a notional
interest-only class. `fnma_remic_2019_2_g3` ships the 198% pricing speed and
carries the deal's full description; this case moves the prepayment assumption
to 0% PSA and asserts the decrement column the supplement publishes for it.

## The reference

The same table as the base case: the Prospectus Supplement dated
24 January 2019, page S-14, which publishes for Classes AB and IO the percent
of original balance outstanding after each January's distribution at seven
prepayment speeds, with a weighted average life for each. This case takes the
0% PSA column. See the base case's `SOURCE.md`.

## What it exercises

The same model as the base case with one term changed, and for 0% PSA the collateral itself: the supplement prepares this column on new 7.50% loans with 360-month original and remaining terms, so the case also exercises a 2.50% servicing and guaranty strip against a thirty-year schedule. What the seven
cases prove together is stronger than any one alone: a convention error in the
prepayment curve, the seasoning ramp or the payment timing that hides under
one column's whole-percent rounding has to hide under all seven columns and
seven published weighted average lives simultaneously.

## The result

**180 asserted values**, every one within the half-percent floor the
table's whole-percent rounding sets. Worst balance disagreement
**0.489 percentage points** against the 0.5 floor.

| | |
|---|---|
| Weighted average life | **20.2290**, published **20.2** |
| Residual to Classes R and RL | **0.0000000000**, every period |
| Principal returned to AB | 148,372,434.00 against an original of 148,372,434 |

The weighted average life is asserted at ±0.07: 0.05 is the print floor of a
figure published to one decimal, and ~0.015 is the axis — the engine measures
on its month-end axis while the deal distributes on the 25th measured from
late-January settlement, a bias uniform across all seven published speeds.

## The delta

The strip identity — 3.25% to AB plus 5.00% of the notional balance
reconstructing the 5.00% pass-through — holds to ten decimal places at this
speed as at every other, which is what makes the residual assertion exact
while the balances carry the table's rounding.

Everything structural — the no-losses guarantee, the compositional boundary
that keeps Groups 1 and 2 out, the one-line waterfall — is as the base case
states it.
