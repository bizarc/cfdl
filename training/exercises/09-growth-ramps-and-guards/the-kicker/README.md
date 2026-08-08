# The percentage-rent kicker

Add the landlord's kicker: 7% of revenue above the 40,000 breakeven, only in months where revenue exceeds it. Use an `active when` guard for the condition and keep the economics in the amount; read revenue with a single-period `series_sum`.

Predict before running: with revenue ramping from 0 to 52,000 over twelve months, in which month does the guard first come true? (Solve `52000 × t/12 > 40000` for t.) Check the series afterward — the kicker's first nonzero month is your answer, and its amount that month should be 7% of the small excess, not 7% of revenue.
