# Match the method

The published fee schedule rounds to the dollar each year and compounds on the rounded value. The starter's `pow()` compounds on unrounded values — close, and wrong, in the way that fails a reconciliation.

Replace the formula with the recurrence: a `state` with `init 100000` and `next round_to(prev * 1.03, 1)`, read by the fee stream as `state.fee_schedule`.

Before running the solution, run the starter and note the total. The recurrence's total differs — by about a dollar over six years here, and by real money over the thirty-year schedules where this pattern lives. That difference *is* "matching the method": in a reconciliation it is the residual you would otherwise chase for an afternoon.
