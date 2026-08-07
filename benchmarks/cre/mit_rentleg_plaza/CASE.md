## The case

A 30,000 rentable square foot office building with two suites, acquired and held
for five years. The two suites sit at different expense stops, so recoveries
differ between them; the stop resets to a new base year when a suite re-lets.
Operating expenses vary with occupancy, rollover at expiry is
probability-weighted, market rent spikes once during the hold, and the building
is sold at ten times forward net operating income net of a 5% commission.

## The reference

Problem Set 1 from MIT OpenCourseWare's real estate finance and investment
course. It publishes the full pro forma table **and** the answer: a present
value at 12% of **$2,292,810**.

**Redistributable.** Released under CC BY-NC-SA 4.0, which is the only content
in the source catalogue with an unambiguous reuse grant.

Unusually, this source publishes both the working and the answer, so the case
checks every intermediate line as well as the result.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Contract types | `cre.exit_forward` |
| Declared | seven native streams |
| Language features | native streams alongside a pack contract |
| Conventions | two expense stops at different levels, a base-year stop reset on re-lease, occupancy-varying operating expenses, probability-weighted rollover, a forward-NOI reversion |

## The result

Every pro forma line reproduces, and so does the published answer:
`model.npv` = **2,292,810.18** against the problem set's $2,292,810.

Asserted: eight stream columns across the five-year table, plus the present
value and the undiscounted total.

## The delta

The 18 cents is the source's rounding, not the engine's — the problem set states
its answer to the dollar. Every per-period line agrees inside a one-cent
tolerance.
