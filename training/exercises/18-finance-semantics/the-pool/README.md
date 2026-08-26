# Divide or compound

The starter converts 8 CPR to a monthly rate by dividing by twelve. Fix both streams.

1. Add a `factor` field on the pool that declines by `cpr_to_smm(0.08)` per month.
2. Read the factor from both interest and servicing.

Anchor before you run. After 12 months the factor must be exactly 0.92, because that is what CPR *means*. The divided version leaves 0.9229. The gap looks tiny. It compounds across every month's interest. On a real pool it is the difference between matching the servicer tape and writing a memo about why you do not. Annual rates meet monthly grids geometrically unless a document says otherwise — the same lesson as the discount-rate conversion in the reading-results chapter.
