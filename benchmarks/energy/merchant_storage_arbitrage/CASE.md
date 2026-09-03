## The case

A 20 MW / 80 MWh merchant battery, front of meter, earning an intraday
arbitrage spread across one year at a daily grain.

The economics are simple to state and awkward to model. The battery charges
when power is cheap and discharges when power is expensive, so its revenue
depends on how far prices spread within a day rather than on their level. What
the battery can capture depends on its duration: a four-hour battery discharges
into roughly the three most expensive hours of a day, not into the sixteen-hour
on-peak block.

The battery itself decides nothing. It is a constraint set — power, usable
energy, round-trip efficiency, and what a cycle costs in wear. Whether the
battery runs on a given day is an operating decision, and this case is about
expressing that decision declaratively rather than assuming its outcome.

## The reference

A **provably optimal dispatch**, solved as a linear program: maximize arbitrage
margin subject to power limits, the state-of-charge window, round-trip
efficiency, and a warranty cap of one equivalent full cycle a day. Each day is
solved on its own, which is the best result a daily-grain model can reach.

The reference is an optimum rather than a tool, and that choice is deliberate.
A national laboratory's project-finance model was measured first. Its dispatch
is documented as "automated but suboptimal", performing "no optimization around
the cost of energy and power", and it reaches 27% of the optimum on this price
year. A linear program's optimum is a proof, so it is the stronger target.

The reference shares no code with the model and reaches its answer by a
different method: optimization over 8,760 hours, against closed-form arithmetic
over daily blocks. Both consume the same stated price series. The price year is
synthetic and seeded, so any reader can regenerate it.

## What it exercises

| | |
|---|---|
| Pack | none — core language only |
| Declared | one lifecycle, two curves, two streams, two metrics |
| Language features | a model-declared lifecycle with guarded edges, `active in state`, `curve_value`, `series_count` |
| Conventions | IEEE Std 762 unit states, TBx block pricing at the battery's duration, round-trip loss taken entirely on charge |

The case is written in the core language, with no pack and no contract, which is
the stronger claim: the language expresses this deal with no domain vocabulary
at all.

Cycling is an **output**. The count of days the battery ran, and the energy
those days moved, both follow from the operating policy. A dispatch model
computes the same two figures, which is what makes the comparison meaningful.

The run-or-idle decision is a guarded edge on a state machine, in the industry's
own vocabulary. IEEE Std 762 separates availability from dispatch, and a unit
that is available but not synchronized is in **reserve shutdown**. An idle day
is a battery in reserve shutdown, and the cash follows the state.

## The result

The model reproduces the optimal dispatch **exactly on volume, and to 0.13% on
margin**:

| | model | optimum | |
|---|---|---|---|
| days dispatched | 365 | 365 | exact |
| MWh discharged | 23,360 | 23,360 | exact |
| revenue | 1,359,583 | 1,358,922 | 0.05% |
| cost | 627,109 | 627,377 | 0.04% |
| margin | 732,474 | 731,545 | 0.13% |

The case asserts both cash columns on every one of the 365 days, together with
the annual margin, the day count, and the energy discharged. The median day
agrees exactly.

## The delta

**The model is a slight upper bound, and the reason is precise.** The model
reads the day's TBx block prices, and those blocks treat the expensive and
cheap slices as independent. Treating them as independent ignores the order of
hours within the day: a battery cannot discharge at 09:00 on energy it buys at
14:00. The optimum respects that order and the blocks do not. The effect is
bounded, and the largest daily deviation is $71.16 on revenue and $56.95 on
cost, against daily revenues averaging $3,723.

That residual is a property of the daily grain rather than an error in the
arithmetic. A second figure measures what the grain itself costs. Solving the
same year as one program, with charge carried across midnight, earns 766,648
against the daily-independent 731,545 — **4.8% more**. That difference is
storage value a daily model cannot capture, and this case measures it rather
than assuming it.
