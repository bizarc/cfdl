## The case

A 20 MW / 80 MWh merchant battery, front of meter, earning an intraday
arbitrage spread across one year at a daily grain.

The economics are simple to state and awkward to model. The asset charges when
power is cheap and discharges when it is dear, so its revenue depends on the
*dispersion* of prices within a day rather than on their level. What it can
reach depends on its duration: a four-hour battery discharges into roughly the
three dearest hours of a day, not the sixteen-hour on-peak block.

And the battery decides nothing. It is a constraint set — power, usable energy,
round-trip efficiency, and what a cycle costs in wear. Whether it runs on a
given day is an operating decision, and the case is about expressing that
decision declaratively rather than assuming its outcome.

## The reference

A **provably optimal dispatch**, solved as a linear program: maximise arbitrage
margin subject to power limits, the state-of-charge window, round-trip
efficiency, and a warranty cap of one equivalent full cycle a day. Each day is
solved independently, which is the best a daily-grain model can do.

An optimum rather than a tool, deliberately. A national laboratory's
project-finance model was tried first and its dispatch is documented as
"automated but suboptimal", performing "no optimization around the cost of
energy and power" — so agreement with it would be evidence of nothing, and
disagreement evidence of nothing either. A linear program's optimum is a proof.

The reference shares no code with the model and reaches its answer by a
different method: optimisation over 8,760 hours against closed-form arithmetic
over daily blocks. Both consume the same stated price series, and the price
year is synthetic and seeded, so any reader can regenerate it.

## What it exercises

| | |
|---|---|
| Pack | none — core language only |
| Declared | one lifecycle, two curves, two streams, two metrics |
| Language features | a model-declared lifecycle with guarded edges, `active in state`, `curve_value`, `series_count` |
| Conventions | IEEE Std 762 unit states, TBx block pricing at the asset's duration, round-trip loss taken wholly on charge |

The energy pack has no storage dispatch contract, and this case does not wait
for one. Its existing storage rule prices a battery as `mwh_cycled_year *
spread`, which asks the modeller to state the quantity a dispatch model exists
to compute. Written in core language instead, cycling is an **output**.

The run/idle decision is a guarded edge on a state machine, in the industry's
own vocabulary: IEEE Std 762 separates availability from dispatch, and a unit
that is available but not synchronised is in **reserve shutdown**. An idle day
is not a zero in an expression — it is a battery in reserve shutdown, and the
cash follows the state.

## The result

The model reproduces the optimal dispatch **exactly on volume and to 0.13% on
margin**:

| | model | optimum | |
|---|---|---|---|
| days dispatched | 365 | 365 | exact |
| MWh discharged | 23,360 | 23,360 | exact |
| revenue | 1,359,583 | 1,358,922 | 0.05% |
| cost | 627,109 | 627,377 | 0.04% |
| margin | 732,474 | 731,545 | 0.13% |

Asserted: both cash columns on every one of the 365 days, plus the annual
margin, the day count and the energy discharged. The median day agrees exactly.

## The delta

**The model is a slight upper bound, and the reason is precise.** It reads the
day's TBx block prices, which treat the dear and cheap slices as independent
and therefore ignore ordering *within* the day. A battery cannot discharge at
09:00 using energy it buys at 14:00, and the optimum respects that while the
blocks do not. The effect is bounded: the largest daily deviation is $71.16 on
revenue and $56.95 on cost, against daily revenues averaging $3,723.

That residual is a property of the daily grain, not an error in the arithmetic.
A second figure sizes what the grain itself costs: solving the same year as one
program, with charge carried across midnight, earns 766,648 against the
daily-independent 731,545 — **4.8% more**. Storage value that a daily model
cannot see, measured rather than asserted.
