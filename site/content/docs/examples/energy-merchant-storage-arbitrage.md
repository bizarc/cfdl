---
id: benchmark-energy-merchant-storage-arbitrage
title: "Energy: a merchant battery dispatched on the day-ahead spread"
slug: "/docs/examples/energy-merchant-storage-arbitrage"
description: "A 20 MW / 80 MWh merchant battery dispatched on the day-ahead spread, with the run/idle decision as a state machine in IEEE Std 762's vocabulary and cycling as an output rather than an assumption."
source: benchmarks/energy/merchant_storage_arbitrage
---

# Energy: a merchant battery dispatched on the day-ahead spread

A 20 MW / 80 MWh merchant battery dispatched on the day-ahead spread, with the run/idle decision as a state machine in IEEE Std 762's vocabulary and cycling as an output rather than an assumption.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

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

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0}}
version 0.1
model "merchant-storage-arbitrage"
time calendar daily from 2026-01-01 for 365

// A 20 MW / 80 MWh merchant battery earning an intraday arbitrage spread.
//
// THE BATTERY DECIDES NOTHING. It is a constraint set — power, usable energy,
// round-trip efficiency, and what a cycle costs in wear. Whether it runs on a
// given day is an operating decision, and that decision is declared here as a
// guarded edge on a state machine.
//
// THE STATES ARE THE INDUSTRY'S. IEEE Std 762, as NERC GADS operationalizes it,
// separates AVAILABILITY from DISPATCH: a unit that is available but not
// synchronized is in RESERVE SHUTDOWN, and economic curtailment is exactly
// that. An idle day is a battery in reserve shutdown, and the cash follows the
// state.
//
// THE MARKET INPUT IS THE DAY'S ACHIEVABLE SPREAD AT THIS BATTERY'S DURATION —
// the mean of the most expensive hours it can discharge into, against the
// cheapest it can charge from. That is the market's own battery product, the
// top-bottom or TBx spread, quoted at a duration because a one-hour and a
// four-hour battery capture different spreads from identical prices. An
// on-peak/off-peak block averages sixteen hours including mid-day, and
// understates what a four-hour battery captures by several times.

entity asset battery {
  lifecycle facility
}

entity party operator { name = "Operator" }

// ── the battery, from primitives ────────────────────────────────────────────
// Round-trip efficiency is COMPUTED from the conversion chain and the cell,
// rather than supplied as one figure.
assume power_mw    = 20.0
assume nameplate   = 80.0
assume soc_min     = 0.15
assume soc_max     = 0.95
assume eff_ac_dc   = 0.96
assume eff_dc_ac   = 0.96
assume eff_cell    = 0.9757
assume round_trip  = inputs.eff_ac_dc * inputs.eff_dc_ac * inputs.eff_cell

// What one cycle moves. Discharge is bounded by the usable window, and
// charging must put in more than comes out, by the round-trip loss.
assume usable_mwh  = inputs.nameplate * (inputs.soc_max - inputs.soc_min)
assume charge_mwh  = inputs.usable_mwh / inputs.round_trip

// The wear a cycle costs, per MWh discharged, and the hurdle a day must clear
// to be worth running. Zero on this deal, so that the model and the reference
// both maximize gross arbitrage margin. An underwriter states degradation
// here.
assume cycle_cost_mwh = 0.0

// ── the market, at this battery's duration ──────────────────────────────────
// The two daily curves are 365 points each, generated from the price year, and
// live in their own file so that the deal is legible beside the data.
import "prices.cfdl"

// ── the operating decision ──────────────────────────────────────────────────
// The machine is declared here and carries the guards on its edges. Each day
// the battery compares the margin a cycle would earn against the wear it would
// cost, and moves between service and reserve shutdown on the answer.
lifecycle facility {
  initial in_service
  state in_service, reserve_shutdown

  in_service -> reserve_shutdown when
    inputs.usable_mwh * curve_value("capture_price", time.date)
      - inputs.charge_mwh * curve_value("cost_price", time.date)
      <= inputs.cycle_cost_mwh * inputs.usable_mwh

  reserve_shutdown -> in_service when
    inputs.usable_mwh * curve_value("capture_price", time.date)
      - inputs.charge_mwh * curve_value("cost_price", time.date)
      > inputs.cycle_cost_mwh * inputs.usable_mwh
}

// ── the cash, which follows the state ───────────────────────────────────────
stream market.discharge on entity asset.battery inflow currency USD {
  schedule every day from 2026-01-01 to 2026-12-31
  amount = inputs.usable_mwh * curve_value("capture_price", time.date)
  active in state in_service
}

stream market.charge on entity asset.battery outflow currency USD {
  schedule every day from 2026-01-01 to 2026-12-31
  amount = inputs.charge_mwh * curve_value("cost_price", time.date)
  active in state in_service
}

// ── outputs, not assumptions ────────────────────────────────────────────────
// Cycling is what the operating policy PRODUCES: the count of days the battery
// ran, and the energy those days moved. A dispatch model computes the same two
// figures, which is what makes them comparable.
metric days_run = series_count("market.discharge", 0, 364)
metric mwh_out  = series_count("market.discharge", 0, 364) * inputs.usable_mwh
```

## Run configuration

```json
{
 "deterministic": {
  "annual_discount_rate": 0.0
 }
}
```

## Verified results

Checked period by period: **2 series** across **365 periods** — **730 values** in all, each within the tolerance shown.

- `market.discharge` — within ±80.0
- `market.charge` — within ±60.0

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.total` | 732,473.55 | ±1500 |
| `metric.days_run` | 365 | ±0 |
| `metric.mwh_out` | 23,360 | ±0.01 |
