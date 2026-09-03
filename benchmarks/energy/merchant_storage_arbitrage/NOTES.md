# merchant_storage_arbitrage — maintainer's notes

## Why the reference is an LP and not a tool

The first reference attempted was NREL's SAM, through PySAM. It was abandoned,
and the reasons are worth keeping because they are not obvious from its output.

**SAM does not optimise, and says so.** NREL/TP-6A20-68614 §1: "Instead of
performing a cost-based optimization, SAM provides options that provide
automated but SUBOPTIMAL dispatch to achieve specified goals", with the stated
reasons being accessibility and licensing rather than anything technical. §5
repeats it — "these heuristic algorithms do not do any optimization around the
cost of energy and power" — and §4.4 is titled *Controller Limitations*. The
source agrees: `lib_battery_dispatch_automatic_fom.cpp` solves each look-ahead
window independently with no constraint linking end-of-window state of charge to
the next.

So a model computing the per-day optimum SHOULD exceed it. On this price year
SAM reaches 194,243 against the optimum's 731,545 — 27%. Anchoring a case to
that would assert a heuristic's shortfall as if it were a target.

**Two of its inputs were silently ignored**, and each produced a plausible,
wrong reference that ran clean and returned sensible magnitudes:

- `forecast_price_signal_model = 1` with `mp_energy_market_revenue` — the
  obvious merchant route, and documented as required for that model — does not
  reach the dispatch. The battery ran a fixed, price-blind schedule: identical
  output for the real prices, a FLAT $32 series, and the time-reversed series,
  losing money on two of the three. It reported 12,854 MWh and $175,513 of
  margin, none of it a dispatch. The path that works is the PPA one, with the
  hourly prices in `dispatch_factors_ts`. (`forecast_price_signal::setup` also
  requires that matrix to carry `8760 x analysis_period` rows, not one year —
  correcting that changes the answer and still does not make the dispatch
  price-responsive.)
- `batt_cycle_cost` does nothing. Identical dispatch at 0.0 through 0.05
  $/cycle-kWh.

Neither was caught by the output looking wrong. What caught the first was
feeding the model a **flat price series it should have refused to trade and did
not**. Any future reference that claims to dispatch on price should face that
test first.

## Why the model is a slight upper bound

0.13% above the optimum on margin, and the cause is structural rather than
numerical. The model reads the day's TBx blocks — the mean of the dearest hours
it can reach and the cheapest it can draw from — and those are computed as two
independent slices of the sorted day. Sorting discards ORDER, so the blocks
permit a combination the clock does not: discharging at 09:00 on energy bought
at 14:00.

The optimum respects ordering, so it is always the smaller number. The gap is
bounded by how often the day's cheapest hours fall after its dearest, which on
this price shape is seldom — the median day agrees exactly, and the worst is
$71.16 on a daily revenue averaging $3,723.

Left as it is rather than corrected. Fixing it needs intraday ordering, which
is below the daily grain, and the residual is smaller than the thing it would
cost to model.

## What the grain itself costs

Solving the same year as ONE program, with charge carried across midnight,
earns 766,648 against the daily-independent 731,545 — 4.8% more. That is
storage value a daily-grain model cannot see at all, and it is the chronology
error `docs/13` §7.1 predicts, measured.

Worth stating plainly: this case does NOT validate `energy.storage_arbitrage`.
It is core-spelled and never declares that contract. What it shows is that the
pack rule's SHAPE is wrong — `mwh_cycled_year` is an input to it and the primary
output of any dispatch model, which is the circularity §7.1 records — and what a
replacement would have to do instead. The pack rule remains exercised but
unvalidated (`docs/13` §7.3).

## The price year

Synthetic and seeded (20260902), not a market download. Summer-peaking,
evening-peaked, day-to-day variation, twelve scarcity days. It is a model input
like a PSA speed: both sides consume it, and the case's claim is about the
distance between two models on identical inputs, not about any particular
market. A real ISO year would exercise negative prices and genuine scarcity
pricing, neither of which this series has, and would be a strictly better input
if one can be vendored.
