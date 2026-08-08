# Divide or compound

The starter converts 8 CPR to a monthly rate by dividing by twelve. Fix both streams: a `factor` field on the pool declining by `cpr_to_smm(0.08)` per month, read by interest and servicing.

Anchor before running: after 12 months the factor must be exactly 0.92 — that is what CPR *means* — where the divided version leaves 0.9229. The gap looks tiny; it compounds across every month's interest, and on a real pool it is the difference between matching the servicer tape and writing a memo about why you don't. Same lesson as the discount-rate conversion in the reading-results chapter: annual rates meet monthly grids geometrically unless a document says otherwise.
