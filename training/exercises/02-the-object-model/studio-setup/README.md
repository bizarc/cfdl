# A studio, from a blank grid

Only the time grid is written. Build the rest.

1. Declare the studio as an entity.
2. Add a two-month fitout phase.
3. Add a monthly rent cost of 1,850 for the full two years.
4. Add a one-time 12,000 setup cost. Schedule the cost with `phase_enter("fitout")`, not with a date.

Both streams are outflows, so net cash is negative by construction. The exercise practices the trio every model is made of: grid, cast, claims.

Check yourself: total cash out is 12,000 + 1,850 × 24 = 56,400.
