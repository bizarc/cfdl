# Merchant storage dispatch — the SAM investigation

**The case shipped**: `benchmarks/energy/merchant_storage_arbitrage`, against a
provably optimal linear program. This directory is the record of why SAM is
*not* that reference, kept because the traps are not obvious from its output and
the next person to reach for it should not rediscover them.

`reference.py` is the SAM harness as it stood when abandoned. It runs, and its
numbers are not a dispatch.

## SAM does not optimise, and documents that

NREL/TP-6A20-68614 (`../68614.pdf`) §1:

> Instead of performing a cost-based optimization, SAM provides options that
> provide automated but SUBOPTIMAL dispatch to achieve specified goals. Reasons
> for taking this approach include -- but not limited to -- the consideration
> that SAM is a public, commercial grade tool and must be accessible to users of
> varying technical ability

§5: "these heuristic algorithms do not do any optimization around the cost of
energy and power." §4.4 is titled *Controller Limitations*.

The source agrees. `shared/lib_battery_dispatch_automatic_fom.cpp` solves each
look-ahead window independently, with **no constraint linking end-of-window
state of charge to the next** — a rolling heuristic, not a global optimum. That
is why net value does not improve monotonically with `batt_look_ahead_hours`.

On the case's price year SAM reaches 194,243 against the optimum's 731,545, or
27%. Anchoring to that would assert a heuristic's shortfall as a target.

Note also that 68614 is the wrong document for a front-of-meter merchant asset:
it covers behind-the-meter PEAK SHAVING, and §4.4.2 states each strategy targets
a single value stream with "no consideration of energy costs". The references
that would cover this configuration are NREL/TP-6A20-64987 (*Economic Analysis
Case Studies of Battery Energy Storage with SAM*) and the 2020 paper *A Model
for Evaluating the Configuration and Dispatch of PV Plus Battery Power Plants*.
Neither was reachable: `nrel.gov` does not resolve from this environment, though
`osti.gov` and `github.com` do — which is how the source was read instead.

## Two inputs that are silently ignored

Each produced a reference that ran clean, returned plausible magnitudes, and was
fiction.

1. **The merchant price path.** `forecast_price_signal_model = 1` with
   `mp_energy_market_revenue` is the obvious route and is documented as required
   for that model. The dispatch never sees it: identical output for the real
   prices, a FLAT $32 series and the time-reversed series — 12,854 MWh and
   $175,513 of "margin", losing money on two of the three. The working path is
   the PPA one: `forecast_price_signal_model = 0`, `ppa_price_input = (1.0,)`,
   `ppa_escalation = 0.0`, `ppa_multiplier_model = 1`, hourly prices in
   `dispatch_factors_ts`. `forecast_price_signal::setup` in `ssc/common.cpp`
   also computes `nrows / analysis_period`, so the merchant matrix must carry
   `8760 x analysis_period` rows — correcting that changes the answer and STILL
   does not make the dispatch price-responsive. Unresolved.
2. **`batt_cycle_cost`.** Identical dispatch at 0.0, 0.005, 0.01, 0.02 and 0.05
   $/cycle-kWh, with `batt_cycle_cost_choice = 1`. A degradation hurdle cannot
   be matched between models.

## The test that caught it

A **flat price series**. A dispatch model given a year with no arbitrage in it
should refuse to trade; SAM traded it identically to the real year and lost
$46,238. Nothing else — not magnitudes, not the shape of the schedule, not the
look-ahead sweep — revealed the problem. Any future reference claiming to
dispatch on price should face that test before its numbers are believed.
