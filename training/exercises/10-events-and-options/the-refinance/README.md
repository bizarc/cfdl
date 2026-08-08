# One turning point, stated once

The starter pays both loans for the full two years — 15,700 a month of interest on a deal that refinances at month twelve. Add the lifecycle: guard each loan on `entity.state.status`, and add the event that sets it to `"refinanced"` at `time.t >= 12`.

Predict the saving before you run: twelve months of bridge (9,500) plus twelve of perm (6,200), versus twenty-four of both. Then check the series — the bridge should stop exactly when the perm starts, with no overlap month and no gap month. Off-by-one here is the difference between the latch firing *at* twelve and *after* twelve; the series will tell you which claim you actually wrote.
