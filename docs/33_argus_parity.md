# Argus parity — the real items

Status: informative, 2026-08-28. Not published; repository-only, like the
backlog.

Argus Enterprise is the reference application for institutional CRE cash flow
projection. This document records what separates CFDL from it **at the level
of modeling mechanics** — the language and engine, not the application. The
UI, the lease-abstraction workflow, importers and the report library are out
of scope by decision: the agent substrate (`docs/32`) and surfaces built on
the results contract are the answer to those, and they consume the language
as it is.

Every claim about CFDL below was verified by probing the current build
pack-free, reading the pack sources, or citing a benchmark — not by reading
feature lists. The Argus side is domain knowledge; no benchmark reconciles
against an Argus run (see "The benchmark this document wants," below).

---

## What is already at parity or ahead

Recorded so the items below are read at their true size — the deficit is
narrow, not broad.

- **Lease-by-lease modeling** is real: per-tenant `cre.lease_unit`
  instances, escalation on lease anniversaries, free rent as its own
  deduction line, recoveries with an expense stop, gross-up and pro-rata
  share, base-year as the stop set to year-0 grossed-up opex.
- **Rollover economics** follow the industry expected-value convention:
  probability-blended renewal/re-let rent, downtime phasing, turnover costs
  split between the scenarios and timed to expiry and occupancy.
- **The rollover cycle itself is core-language.** A `lifecycle` re-fires
  edges without limit, `schedule … from state_enter(entity, state) for n
  periods` re-anchors on every re-entry, and per-cycle costs re-fire at each
  re-let. Probed pack-free: a leased/downtime machine cycled for 36 months,
  every transition journaled, TI/LC re-firing at each new lease.
- **Rent step schedules, recovery caps, vacancy netting** are expressible in
  core today and are pack-ergonomics items, not language gaps. Probed: a
  step `curve` is an arbitrary rent-step table; a field recurrence carries a
  cumulative recovery cap (the probe's cap bound exactly at its limit, with
  a correct partial period); netting general vacancy against modeled
  downtime is arithmetic.
- **Ahead of Argus:** percentage rent as an expectation over a sales
  distribution (`cre.percentage_rent_expected` — the point-estimate form
  pays 0.00 on any breakpoint above expected sales, however wide the
  distribution); grain/day-count/roll-convention/holiday-calendar time
  machinery; the journal as a causal audit trail; text source under version
  control; byte-comparable runs; per-assumption Monte Carlo.
- **Debt sizing needs no solver.** Every sizing met so far is closed-form,
  sequential once the wiring is untangled, or affine — the pattern is
  recorded once in `docs/26` ("A sized loan does not need a solver").

---

## Item 1 — lifecycle entry actions

**The one construct-level gap this comparison found.**

**What could not be expressed:** an action performed on the Nth entry into a
state. In one sentence: **the repeatable construct cannot act, and the
acting construct cannot repeat.** An edge is `from -> to [when <expr>]` and
nothing else (`crates/cfdl-parser/src/lib.rs`, `LifecycleEdge { from, to,
guard }` — no action list; the engine's transition loop journals and moves
`status`, and writes nothing else). The only construct that carries actions
— `set entity …` — is the event, and an event latches by design: the engine
skips a fired event forever (`crates/cfdl-engine/src/state.rs`,
`event_fired`), and `docs/01` §13 keeps it that way deliberately ("a regime
that returns is the machine's job"). Repetition was moved to the machine —
a guarded edge, self-edges included, fires as many times as re-entry
re-arms it (`docs/28` §6) — but arrival performs no actions.

**What forced the discovery:** chained rollover, probed pack-free. Two
per-visit quantities had no spelling:

- **A duration-in-state counter.** "Re-lease after 3 months of downtime"
  needs `months_in_downtime` to return to zero at each new vacancy. A field
  recurrence counts up forever; the conditional form
  (`next if(prev.<entity>.status == "leased", …)`) fails at run with
  `E5002 — prev.<entity>.status is not declared`. (A *fixed* duration can be
  spelled as a chain of states — `downtime_1 -> downtime_2 -> leased` — but
  a duration that is a term, an input, or per-instance data cannot.)
- **Re-striking rent on re-let.** Each new lease pays the market rent
  prevailing *when it began*, held for its term. Where re-let dates are
  known in advance, a step `curve` with knots at the cycle starts is the
  strike schedule and nothing is missing — that is the deterministic Argus
  case, and it works today. Where the transition date is **endogenous** (a
  default event, a downtime whose end depends on modeled state), no
  calendar-keyed curve can know when the cycle began.

**The shape:** an action block on an edge (equivalently: on state entry),
performing a **lookup at the transition's own instant** and writing entity
fields:

```cfdl
downtime -> leased when <expr> {
  set in_place_rent   = curve_value("cre.market_rent", time.date)
  set months_in_state = 0
}
```

The field then just holds (`next prev.<…>`), streams read the field, and the
recurrence never reads state — cycles stay impossible by the same argument
as today. The write machinery exists on the event side (`set entity …`,
validated against the declared edge relation); the ask is attaching the same
action list to the edge, in the model grammar and in a pack's `types.toml`
transitions alike. This closes duration counters, per-cycle burn-off
(free rent per new lease), and endogenous-date re-striking in one construct.

**CLOSED 2026-08-30** by backlog §7.79 (`docs/34`, phases 1-5). Both halves
shipped: a transition carries actions (`on enter <state>` for the state, an
action block on an edge for the path), and the named event fires on each
occurrence rather than latching. The duration counter this item could not
spell is now `on enter <state> { set months_in_state = 0 }`, and the
endogenous re-strike is an edge action reading the curve at the transition's
own instant. Verified against `cre.unit`: an event moves the unit, the pack's
entry action resets the counter, and a re-let three months after going vacant
runs — the probe that found the gap became the fixture that closes it.

Originally scoped in `docs/34_events_and_the_machine.md`; backlog §7.79. Sits directly
on M2's machine work (`docs/13` §7.78); adjacent to §7.73's state-gating. Note the probe's failure mode is itself §7.38-shaped: the
state-reading recurrence compiles clean and dies at run.

## Item 2 — the discount curve reaches the valuation plane

**Backlog §7.4, made explicit.** The language side is done — `curve` already
expresses a sparse rate schedule (step = flat-forward, linear =
calendar-day interpolation). The gap is confined to the valuation plane:
`RunConfig.discount_rate` is a single `f64`, turned into one
`per_period_rate` and handed to `npv_with_offsets`; no discounted figure
ever consults a curve.

**The shape:** the run configuration (or the model) names a curve; the
valuation plane looks up the prevailing annualized rate per period date and
compounds discount factors cumulatively —

    DF(t) = DF(t-1) / (1 + r(date_t) / ppy)

— rather than exponentiating one rate. Two conventions to state, not solve:
the curve holds prevailing annualized rates (flat-forward, matching step
curve semantics), not zero-coupon spots; and `model.irr` remains a scalar
solve by construction and is documented as such (`irr_with_offsets` — §7.4
flags this). One interaction to state: the annual-grain exponent question
(§7.69) — the cumulative DF product sidesteps it, which is an argument for
that form.

**Why it matters here:** Argus supports term-varying rates; construction
discounts at a different rate than stabilized operations; Damodaran's
converging cost of capital is the opco twin. The standing cost is already in
the suite: `benchmarks/opco/damodaran_fcff` asserts the entire cash-flow
build and **no discounted figure at all**, because flat-rate discounting
would produce a number that is not a check.

## Item 3 — the market-leasing-assumption bundle (pack design item)

Argus defines a market profile once — market rent, downtime, renewal
probability, TI/LC on renewal and re-let — and applies it to many suites.
The sharing mechanism exists in CFDL today: the `reference` family is
declared in every pack's ontology (`cre.market_rent`, `cre.cap_rate`, …),
and contract terms accept expressions, so fifty rollover instances can read
one curve (`renewal_rent_year = curve_value("office_a_market", time.date)`).

What is missing is only the **named bundle**: a way for a rollover contract
to say `market = office_class_a` instead of restating five terms per
instance. The ontology's field machinery already supports typed,
multi-field declarations; this is a pack/ontology surface design, not a
language construct. No backlog entry until a case forces the shape.

---

## Non-items, recorded so they are not rediscovered

| candidate gap | resolution |
|---|---|
| Rent step tables | a step `curve` is exactly this; pack ergonomics only (`cre.lease_unit` takes a scalar `rent_year`) |
| Recovery caps (cumulative, collared, ratcheted) | field recurrence; probed, cap binds exactly |
| General vacancy net of modeled downtime | arithmetic in the amount expression |
| Repeating rollover cycle, re-anchored windows, per-cycle TI/LC | shipped core (M1 phase 5); probed |
| Deterministic re-strike at market | step `curve` with knots at known cycle starts |
| DSCR/LTV/sculpted debt sizing | no solver needed — `docs/26`, "A sized loan does not need a solver" |
| UI, abstraction workflow, importers, report library | out of scope; `docs/32` |

## The benchmark this document wants

An Argus tie would be the highest-value CRE benchmark in the programme — it
is the reference CRE practitioners already trust. It cannot be a public
case: Argus output is not redistributable and producing it needs a licensed
seat. It belongs in the **private held-out case set** that `docs/32` Phase 3
already contemplates, alongside engagement-derived cases from `docs/31` W2.
