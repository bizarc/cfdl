# SAM parity — the real items

Status: informative, 2026-09-01. Not published; repository-only, like the
backlog.

NREL's System Advisor Model is the reference application for renewable-energy
project finance. This document records what separates CFDL from it **at the
level of financial modeling mechanics** — the language, the engine and the
energy pack.

**The scope boundary first, because for SAM it is the whole comparison's
frame.** SAM is two layers: a physical simulation (8760-hour production from
weather files, irradiance, the module/inverter loss stack, wind resource) and
a financial model that consumes the production. The physical layer is out of
scope **by design**, not by deferral: CFDL consumes production as an input —
`quantity` (MWh/yr), degradation, availability — exactly as a project-finance analyst
consumes a P50 from an independent engineer. That is this document's analogue
of `docs/33`'s UI paragraph, with one difference: the UI decision defers work
to `docs/32`, while the physical layer is work CFDL should never do. True
hourly dispatch optimization sits on the same side of the line
(`docs/13` §7.1: "that is an optimizer, not a declarative cash-flow model").

Every claim about CFDL below was verified by probing the current build,
reading the pack sources, or citing a benchmark — not by reading feature
lists. Unlike the Argus and Intex documents, the reference side here is
checkable too — see "The benchmark this document wants," which for once
exists.

---

## What is already at parity or ahead

Recorded so the items below are read at their true size — energy is the
suite's best-validated pack (`docs/13` §7.3: 10/10 rules exercised, 9/10
externally validated).

- **The single-owner PPA structure end to end**: a 25-year PPA with a fixed
  escalator against annual degradation, project debt, a 30% ITC and
  five-year MACRS on a basis reduced by half the credit —
  `benchmarks/energy/utility_pv_singleowner`, reconciled against SAM itself
  (the NOTES carry the PySAM harness that produced the reference).
- **Merchant plus capacity**: uncontracted energy at market prices beside a
  flat capacity payment, PTC over the first ten years, MACRS on the full
  basis because PTC and ITC are mutually exclusive —
  `benchmarks/energy/merchant_capacity`.
- **O&M, capex and project debt** as pack contracts, the debt lowering to
  the whole instrument — proceeds at closing, interest and principal legs
  summing to the level payment exactly (`packs/energy/README.md`,
  `energy.debt_service`).
- **The statutory tax credits as statutes, not approximations**: the PTC
  computes the published rounding staircase via `round_to` rather than
  carrying the rate continuously — an error worth up to 1.8% in any one year
  that "survives every reconciliation except against a source that rounds"
  (`packs/energy/README.md`); exercised in
  `benchmarks/energy/wind_ptc_macrs`.
- **The tax-equity flip with a DERIVED flip date** — the flip is a test on
  the investor's after-tax return, not a date, and the model derives when it
  lands (`benchmarks/energy/tax_equity_flip`), with an account-built twin
  reconciling within tolerance on 50 of 50 cells
  (`benchmarks/energy/tax_equity_flip_account`, `docs/13` §7.76).
- **A three-way reconciliation**: CFDL against NREL's CREST spreadsheet AND
  its independent Python port, five opex lines that do not share an
  escalator, an abating PILOT, a revenue royalty
  (`benchmarks/energy/crest_solar_cost_based`).
- **Solar-plus-storage under contract** —
  `benchmarks/energy/solar_ppa_microgrid` (the storage *dispatch* question
  is Item 1, below).
- **An availability lifecycle SAM does not have**: the facility machine is
  eleven states on the NERC GADS / IEEE 762 taxonomy — in-service, reserve
  shutdown, the three outage classes, mothballed, retired, abandonment edges
  (`packs/energy/ontology/types.toml`; the restructure argued in
  `docs/36` §3.1).
- **All grains**: the same deal produces the same annual figures on daily,
  monthly, quarterly or annual calendars, and a gate asserts it
  (`packs/energy/README.md`).
- **Ahead of SAM**, the same list as the other parity documents and it bears
  repeating once: the journal as a causal audit trail, text source under
  version control, byte-comparable runs, per-assumption Monte Carlo with
  full metric distributions (`docs/13` §7.87).

**And the structural advantage that makes this document different from
`docs/33` and `docs/38`:** SAM is BSD-3 licensed and `pip install nrel-pysam`
runs it headless. The benchmark the Argus document wants and cannot have —
a reconciliation against the reference application itself — already exists
for energy: four benchmark NOTES carry PySAM harnesses today
(`utility_pv_singleowner`, `merchant_capacity`, `tax_equity_flip`,
`tax_equity_flip_account`), and
`research/CFDL_pack_roadmap_and_model_sourcing.md` proposes the formal
version — a harness sweeping hundreds of parameter combinations and diffing
every annual cash flow array, converting energy validation from "we wrote a
second implementation" to "we agree with NREL."

---

## Item 1 — storage dispatch, and the state of charge

**The construct is buildable; the reference is missing.**
`energy.storage_arbitrage` is the pack's one externally-unvalidated rule —
`mwh_cycled_year * spread`, a dispersion functional evaluated at a point
(`docs/13` §7.1). The walk made the fix buildable: a state-of-charge balance
stepped per period turns `quantity` (the storage rule's MWh cycled) from an assumed input into an
output of dispatch against a price shape, which is exactly the circularity
that blocks validation today (`docs/13` §7.75 — the quantity we ask the
modeller to state is the thing the reference exists to compute).

**What forced the discovery** is also what gates it: §7.1 ran the comparison
rather than assuming it. SAM's `Battwatts` model behind the meter discharged
27.9 MWh in a year from an 80 MWh battery — nothing for the battery to do
under that load — and reconfigured front-of-meter for merchant arbitrage,
the native library segfaulted (exit 139). A real comparison needs the full
`Battery` module with price-signal dispatch, a scoping exercise of its own.
The construct no longer waits on the engine; the case waits on a source.
Backlog: `docs/13` §7.75, §7.1; the quantile half is `docs/27` §9.

## Item 2 — additive real-plus-inflation escalation

**What could not be expressed ergonomically:** SAM and its relatives state
escalation as a *real* rate on top of a separate inflation assumption, and at
least one widely used model combines the two **additively** — 2.5% inflation
plus 2.0% real is 4.5%/yr, not 4.55%. The pack's `escalation` is one nominal
rate applied whole. The workaround is exact — enter
`escalation = inflation + real` — and the pack README documents it with the
warning that the difference compounds if a reader assumes the other
convention (`packs/energy/README.md`; verified in the utility-scale PV
benchmark). **The shape:** pack ergonomics — separate `inflation` and
`real_escalation` terms with a stated combination rule — not a language
construct. Listed as pack-shape work in `docs/30` §2.

## Item 3 — bonus depreciation and multi-class basis

A project's basis splits across MACRS classes (5-, 7-, 15-, 20-year), and
bonus depreciation takes a stated fraction in year one; `energy.macrs_shield`
takes one basis and one life. Multiple contract instances cover the class
split today, but nothing expresses bonus depreciation's year-one take, and
no benchmark exercises a split basis. Recorded from the reference gaps in
`docs/30` §2 ("bonus depreciation and multi-class basis —
`crest_solar_cost_based` gaps; `utility_pv_singleowner` findings"). Pack
work, forced by sources already in the suite.

## Item 4 — the DSRA and the major-maintenance reserve

**What could not be modeled, and now can — minus the case.** Both PV
references zeroed reserves out as inexpressible, and CREST's EBITDA includes
~$4,606 of year-one interest earned on funded reserve accounts that CFDL
did not model (`benchmarks/energy/crest_solar_cost_based/NOTES.md`;
`benchmarks/energy/utility_pv_singleowner/NOTES.md`). The mechanism has since
shipped — the account, fund-to-target, interest on the prior balance
(`docs/28` §5.1; `docs/13` §7.76, whose credit-pack instance landed on
AmeriCredit) — so what remains is the energy reserve contract shape and its
case.

**And the ~$4,606 is not an anchor to fit.** §7.76 is explicit: it is a
single rounded year-one aggregate against three unknowns — balance, funding
rule, rate — the conventional structures do not fit it, the port's reserve
schedule was never carried into the repo, and the case deliberately has no
close period to fund from. Fitting a structure to one number would be
numerology against the suite's tightest external reconciliation. The route is
re-running the licensed port to recover the schedule — a sourcing step, and
open. Backlog: `docs/13` §7.76, part two.

## Item 5 — the DSCR covenant, benchmarked

The pack publishes `dscr_periodic` every period
(`packs/energy/statements.toml`), and until 2026-08-30 nothing could act on
it. The cash-trap-with-cure mechanism has shipped
(`fixtures/valid/dscr_cash_trap_cure_period`, `docs/13` §7.77) — breach traps
distributions in an account, consecutive good periods release. What remains
is §7.77's own remainder: an external reference, a published credit agreement
with a cash-trap schedule to reconcile against, and none is vendored. A
case-authoring ask with a sourcing problem, not a language gap. Backlog:
`docs/13` §7.77; `docs/20` §5.1.

## Item 6 — the curtailment regime, declared rather than improvised

The documented idiom is still the workaround: a bare field flipped by a price
test, with the stream reading the field (`docs/26`, "A regime that turns on
and off repeatedly" — written when events latched). The machinery has moved
under it: the machine can act and re-fire (`docs/13` §7.79), and the facility
lifecycle now names the regime properly — economic curtailment is
`reserve_shutdown`, available but not dispatched (`docs/36` §3.1, landed in
`packs/energy/ontology/types.toml`). What no model or benchmark yet does is
run curtailment THROUGH the declared machine — journaled transitions,
`active in state` gating, re-entry-safe — and retire the bare-field idiom
from the teaching docs. `docs/30` §2 names this as the walk-enabled item.

## Item 7 — outage derating, unexercised

The restructure added `available_capacity_fraction` — a derate is a
magnitude that coexists with being in service, so it is a field, not a state
(`packs/energy/ontology/types.toml`; the argument in `docs/36` §3.1, from
IEEE 762's treatment of derates). Nothing exercises it: no rule consumes the
field and no case asserts a derated period, so the availability metrics the
taxonomy exists to support (EAF, EFOR) are not yet computable from a model.
A pack-and-case item, forced the day an availability-payment deal with a
derating schedule is modeled.

---

## Non-items, recorded so they are not rediscovered

| candidate gap | resolution |
|---|---|
| The 8760 physical layer — weather, irradiance, the loss stack | the scope boundary, by design; CFDL consumes production as an input, as the intro argues |
| Hourly dispatch optimization | out of scope and staying there — an optimizer, not a declarative cash-flow model (`docs/13` §7.1) |
| Derived ITC basis | a deliberate boundary, documented with its idiom: basis adjustments are jurisdictional, so the pack takes `basis` as an input and the README shows the two-line model that derives it from `installed_cost` and `itc_rate` — "state the adjustment, not the answer" (`packs/energy/README.md`; the utility-scale PV benchmark is written this way) |
| DSCR-sculpted debt sizing | no solver needed — the pattern is `docs/26`, "A sized loan does not need a solver"; the README's roadmap line predates it |
| Leap-year daily totals paying 366/365 | the Act/365-Fixed convention behaving correctly, documented with the workaround (`packs/energy/README.md`) |

## The benchmark this document wants — and, uniquely, can have

`docs/33` ends wanting a benchmark it cannot ship publicly, and `docs/38`
inherits the same licensing bind. This document does not. SAM is BSD-3;
PySAM is a pip install; the reference runs headless in CI. The ask is
`research/CFDL_pack_roadmap_and_model_sourcing.md`'s next-step 2, stood up
formally: express the existing solar and wind benchmarks as `Singleowner`
configurations and diff annual arrays — then widen to a parameter sweep, so
the claim "we agree with NREL" is measured over hundreds of configurations
rather than the handful the NOTES already carry. Every open item above that
ships (the reserve, the covenant case, storage dispatch) should land with its
PySAM column where SAM computes one, and the two that SAM cannot check (the
declared machine, the derate field) are where CFDL is ahead, not behind.
