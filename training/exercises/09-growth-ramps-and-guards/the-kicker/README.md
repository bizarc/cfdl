# The percentage-rent kicker

Add the landlord's kicker: 7% of revenue above the 40,000 breakeven, only in months where revenue exceeds it.

1. Put the condition in an `active when` guard.
2. Keep the economics in the amount.
3. Read revenue with a single-period `series_sum`.

Predict the guard's first true month before you run. Revenue ramps from 0 to 52,000 over twelve months, so solve 52000 × t/12 > 40000 for t.

After the run, check the series. The kicker's first nonzero month is your answer. The amount that month is 7% of the small excess, not 7% of revenue.
