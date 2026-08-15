# Match the method

The published fee schedule rounds to the dollar each year and compounds on the rounded value. The starter's `pow()` compounds on unrounded values — close, and wrong in the way that fails a reconciliation.

1. Run the starter. Note the total.
2. Add a rule field `fee_schedule` on the entity: `init 100000`, `next round_to(prev * 1.03, 1)`.
3. Read the field from the fee stream as `asset.co.fee_schedule`.
4. Run again. Compare the two totals.

The totals differ by about a dollar over six years. On the thirty-year schedules where this pattern lives, the difference is real money. The difference *is* "matching the method": in a reconciliation it is the residual you would otherwise chase for an afternoon.
