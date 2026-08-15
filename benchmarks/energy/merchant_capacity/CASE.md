## The case

A 100 MW merchant renewable project — no contracted offtake, so it sells energy
at market prices — with a separate flat capacity payment for being available. It
claims the production tax credit over its first ten years and depreciates on the
five-year MACRS schedule. Because the production credit and the investment credit
are mutually exclusive, there is no basis reduction here: depreciation runs on
the full $100m.

## The reference

A national laboratory's open-source project-finance model, run for the merchant
and production-credit configuration.

**Not vendored.** The tool was run once outside the repository and only its
output numbers were carried across.

## What it exercises

| | |
|---|---|
| Pack | `energy` |
| Contract types | `energy.merchant`, `energy.capacity`, `energy.ptc`, `energy.om`, `energy.debt_service`, `energy.macrs_shield`, `energy.capex` |
| Language features | pack contracts; term units on the credit rate |
| Conventions | merchant pricing with escalation, a flat capacity payment, a ten-year production credit with an inflation adjustment, MACRS on full basis |

## The result

Every asserted line agrees with the reference.

The claim is narrower than its companion's. `energy.merchant` and `energy.ptc`
are the same expression as `energy.ppa` with different term names, and
`energy.capacity` is a single division, so agreement shows the terms reach the
right places and the contracts compose rather than that a new formula is
correct.

## The delta

None on the asserted lines.

One mechanic here is new. The production credit is a **staircase**: the
inflation-adjusted rate is published rounded to the nearest tenth of a cent per
kilowatt-hour, so it steps once a year and holds. Carried continuously it is
wrong by up to 1.8% in a single year and about −0.3% over the ten-year window,
and the error alternates sign rather than drifting.
