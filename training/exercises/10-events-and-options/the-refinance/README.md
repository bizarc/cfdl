# One turning point, stated once

The starter pays both loans for the full two years — 15,700 a month of interest on a deal that refinances at month twelve. Add the lifecycle.

1. Guard each loan stream on `entity.status`.
2. Add the event that sets the status to `"refinanced"` at `time.t >= 12`.

Predict the saving before you run: twelve months of bridge (9,500) plus twelve of perm (6,200), against twenty-four months of both.

Then check the series:

- The bridge stops exactly when the perm starts.
- No month pays both loans. No month pays neither.

An off-by-one is the difference between the event firing *at* twelve and *after* twelve. The series tells you which claim you actually wrote.
