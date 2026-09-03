# Merchant storage dispatch — work in progress

Not a benchmark case yet. `docs/20` §5.4 asks for one and `docs/13` §7.75 gates
it on "a dispatch reference that runs". The reference now runs. What is not yet
settled is whether it can be **driven correctly**, which is the open work.

Kept here rather than in `benchmarks/` because there is no defensible
`expected.csv`: asserting CFDL against its own output would be the suite marking
its own homework, and the reference is not yet trustworthy enough to anchor to.

## The asset

20 MW / 80 MWh standalone battery, SOC window 15–95% (64 MWh usable), merchant,
front of meter, one stated synthetic price year (seed 20260902).

## What the CFDL model does

Core CFDL. No pack, no `energy.storage_dispatch` contract — that contract does
not exist (`docs/27` §9 stage 4) and the model does not wait for it. The claim
being made is the stronger one of `docs/13` §7.3: the language expresses the
asset with no domain vocabulary.

- A **model-declared lifecycle** carries the operating decision on its edges.
  A model may not state edges on a *pack* machine (`E1357`), but its own machine
  takes guards directly, so no named events are needed. The states are IEEE Std
  762's as the energy pack reads them: a unit available but not synchronised is
  in `reserve_shutdown`, and economic curtailment is exactly that.
- **Two price axes, two constructs**, because they are indexed differently
  (`docs/13` §7.1 — a duration curve is not a date-indexed curve). A `curve`
  carries the day's LEVEL; a `quantile` carries the intraday SHAPE, each hour as
  a ratio to its own day's mean. Both are statistics of the same hourly series
  the reference consumes; neither contains a dispatch decision.
- **Depth** — how much of a cycle is worth running — is `quantile_of`
  inverting the marginal cost into a share of hours above it, capped by the
  share the asset can physically reach. This is what the quantile primitive was
  built for (`docs/27` §1).
- Round-trip efficiency is COMPUTED from the conversion chain, not read back
  from the reference. Days run and MWh are `metric`s: outputs, not assumptions,
  which is the circularity `docs/13` §7.1 names.

## Where it stands against the reference

Matched assumptions, gross arbitrage margin, no hurdle on either side:

| | CFDL | SAM | |
|---|---|---|---|
| days run | 365 | 302 | |
| MWh out | 23,360 | 14,193 | x1.65 volume |
| capture $/MWh | 58.20 | 47.77 | |
| charge $/MWh | 24.14 | 30.66 | |
| margin $/MWh | 31.35 | 13.68 | x2.29 unit margin |
| margin | 732,474 | 194,243 | x3.78 |

CFDL is an **upper bound**: the most a duration-matched asset could earn if each
day were independent and perfectly foreseen. The gap decomposes cleanly into
volume and unit margin, and the second is chronology — SAM carries state of
charge across midnight and sees 24 hours ahead.

Depth responds correctly to the wear assumption and is inert below about
$24/MWh, because a 3.2-hour asset reaches only 13.3% of the day's hours and the
physical cap binds before the economics do:

| cycle cost | days run | revenue | margin |
|---|---|---|---|
| $0/MWh | 365 | 1,299,636 | 758,464 |
| $12/MWh | 365 | 1,282,704 | 750,345 |
| $25/MWh | 365 | 853,790 | 535,502 |
| $35/MWh | 365 | 517,142 | 353,479 |
| $45/MWh | 364 | 365,294 | 264,680 |

## THE OPEN WORK: driving SAM correctly

Two inputs were found to be **silently ignored**, each of which produced a
plausible, wrong reference:

1. **The merchant price path does nothing.** `forecast_price_signal_model = 1`
   with `mp_energy_market_revenue` is the obvious route for a merchant battery.
   The dispatch ignores it and runs a fixed, price-blind schedule — identical
   output for the real prices, a FLAT $32 series, and the time-reversed series,
   losing money on two of the three. It reported 12,854 MWh and $175,513 of
   margin and none of it was a dispatch. What the optimiser actually reads is
   the PPA path: `forecast_price_signal_model = 0`, `ppa_price_input = (1.0,)`,
   `ppa_escalation = 0.0`, `ppa_multiplier_model = 1`, and the hourly prices in
   `dispatch_factors_ts`.
2. **`batt_cycle_cost` does nothing.** With `batt_cycle_cost_choice = 1`, the
   dispatch is identical at 0.0, 0.005, 0.01, 0.02 and 0.05 $/cycle-kWh. So the
   degradation hurdle could not be matched and both sides were set to zero.

Neither was caught by the output looking wrong. The flat-price test is what
caught the first: a series with no arbitrage in it that the battery traded
anyway. **Any future reference run should include that test.**

Also unresolved, and each a candidate reason the comparison is not yet a fair
one:

- `batt_look_ahead_hours` materially changes the answer (net value rises
  monotonically from 12h to 48h). The reference is only a fixed target against a
  declared horizon; 24h is stated, not derived.
- SAM's average depth is 73% of a full cycle over 302 active days. Whether that
  is SOC continuity, the look-ahead window, or something else is not known.
- Whether the `Battery` compute module standalone is the right vehicle at all,
  or whether merchant dispatch is only correct when chained to a financial
  model. `Merchantplant.from_existing` fails on an unassigned
  `system_pre_curtailment_kwac` and was not pursued.

Until those are understood, the distance between the two models cannot be
attributed to CFDL.

## What the documentation says, and where it disagrees with behaviour

From the PySAM `Battery` reference and NREL's battery-storage material:

- **The PPA/TOD path is the documented one.** "When modeling a battery system
  with one of the PPA financial models, SAM treats the battery as a
  front-of-meter application and can dispatch the battery to respond to a PPA
  price that varies with time using time of delivery (TOD) multipliers."
  `dispatch_factors_ts` is documented as REQUIRED when
  `forecast_price_signal_model = 0` and `ppa_multiplier_model = 1`. That is the
  configuration that works here, so the empirical finding and the documentation
  agree.
- **The merchant path is documented as required for model 1** —
  `mp_energy_market_revenue` is "Required if forecast_price_signal_model=1",
  a two-column array of MW and $/MW. It was supplied in exactly that shape and
  the dispatch ignored it. The most likely explanation is that it is consumed by
  the FINANCIAL module rather than the battery's own optimiser, so the
  standalone `Battery` compute module never sees a price — which would make the
  chained `Merchantplant` configuration mandatory for merchant dispatch, not
  merely conventional. NOT YET CONFIRMED.
- **Cycle cost is documented as a dispatch penalty.** `batt_cycle_cost_choice`
  selects "SAM's built-in degradation cost model" (0) or a custom penalty (1),
  and `batt_cycle_cost` is "$/cycle-kWh". Neither value changes the dispatch
  here. Also NOT EXPLAINED.

The authoritative description of the algorithm itself is NREL/TP-6A20-68614,
*An Overview of the Automated Dispatch Controller Algorithms in the System
Advisor Model*, and the 2020 paper *A Model for Evaluating the Configuration and
Dispatch of PV Plus Battery Power Plants*, which is cited as containing the
front-of-meter automated dispatch implementation. Neither was reachable from
this environment; both should be read before the reference is trusted, because
they are where the look-ahead semantics and the treatment of state of charge at
the window boundary are defined — and those are the two candidates for SAM's
73% average cycle depth.

## Reproducing

`python reference.py` regenerates `reference.json` (PySAM required; not a
project dependency — it establishes anchors and nothing in the repo imports it).
`model_inputs.json` carries the daily level curve and the intraday shape the
model declares.
