# CFDL — Feature Backlog

Status: informative. Things worth building that are **not** defects.

Bugs do not belong here; they get fixed or they get a failing test. What
belongs here is capability the language or a pack does not yet have, where the
gap was found by trying to model something real and hitting a wall. Each entry
therefore says what could not be expressed, and what forced the discovery —
a backlog item with no provenance is a guess.

Ordered within each section by how much it unblocks, not by effort.

---

## 1. CRE pack

### 1.1 Occupancy-varying operating expenses

`cre.property_opex` takes `opex_year` and `escalation`, so opex is a geometric
series. Real buildings split it: a portion is fixed and a portion moves with
occupancy.

MIT OCW 11.431J PS1 states it as 81% fixed / 19% variable, which is what
produces $135,161 of 2001 opex rather than $144,300 — and the variable share
then moves again in 2004 when a suite goes dark. The published answer depends
on it.

Blocks: `benchmarks/cre/mit_rentleg_plaza` models opex and recoveries as native
streams. Everything else in that model now runs through pack contracts.

Shape: an `occupancy` input and a `pct_fixed` term, with opex as
`opex_year * (pct_fixed + (1 - pct_fixed) * occupancy)`. The hard part is that
occupancy is itself derived from the rent roll, so this may want to be a
cross-stream (phase-2) rule rather than a term.

### 1.2 An expense stop that resets to a computed value

`cre.lease_unit.expense_stop_year` is a literal. A lease signed mid-hold
conventionally sets its stop at the *then-current* opex per SF, which is a
figure the model computes rather than one the analyst states.

MIT fn 5 does exactly this: the replacement Suite 100 lease takes its stop from
actual 2004 opex, which is why its 2004 reimbursement is exactly zero.

Blocks: the same benchmark. Also forces a duplicated opex formula — see 3.1,
which is the underlying cause.

Shape: a term that names a period rather than an amount, resolved after the
opex stream exists. Depends on 3.1.

### 1.3 Abatements as a first-class NOI line

`domain.cre.noi` sums base rent, recoveries, percentage rent, ops revenue, and
subtracts ops expense, vacancy and property opex. Free rent has no line.

The pack's own `cre.lease_unit` folds free rent into base rent, so it never
surfaces. Institutional pro formas report Abatements as its own deduction from
potential gross revenue — MIT's does. Today you can report it as a line **or**
have it counted in NOI, not both.

Shape: add an abatement stream family to the metric's denominator, and have
`cre.lease_unit` emit the deduction separately rather than netting it.

---

## 2. Language and engine

### 2.1 A stream may not read another period's value

Phase-1 streams cannot look at each other at all; phase-2 streams can read
phase-1 through `series_sum`/`series_avg` but not each other, which is what
makes cycles impossible by construction. The cost is that any rule needing
"the value of X in period k" must either duplicate X's formula or become
phase-2 and give up reading other derived streams.

Found twice: the base-year expense stop above (1.2), and
`benchmarks/cre/mit_rentleg_plaza`, which carried a duplicated opex formula
until the projection-tail fix removed the need.

Shape: a bounded backward reference — a stream may read a *strictly earlier*
period of another stream — which is acyclic by construction and would cover
both cases. Note this is close to the per-period persistent state the pack
roadmap identifies as the gate on roughly two thirds of its candidate packs, so
it is worth designing the two together rather than separately.

### 2.2 Per-period persistent state

No accumulator, no carryforward, no balance that a period can add to and a
later period draw down. Cash sweeps, revolver draws, FF&E reserves, escrow
accounts, NOL carryforwards and construction-interest capitalisation all need
it, and `packs/opco/lowering/rules.toml` says so in its header.

Not discovered by this work — it is a known absence — but recorded here because
2.1 is a strictly smaller version of it and the two should share a design.

---

## 3. Cross-pack

### 3.1 Day count beyond the four supported bases

`{{model.accrual_divisor}}` handles `30/360`, `30e/360`, `act/360` and
`act/365`. `act/act` is not supported: it needs the days in the *year* the
period falls in, which the expression environment does not expose
(`time.days_in_period` is the period, not the year).

Low urgency — the four cover most instruments — but `act/act` is the government
bond convention and will be wanted if a sovereign or municipal pack appears.

### 3.2 An acquisition or disposal in a period other than the term's

`schedule_kind = "on_date"` places a one-shot flow in the period containing its
date, and `schedule_at_period_end` now says where in that period it sits. What
cannot be expressed is a flow whose *period* differs from the contract term —
a sale agreed in one period and settling in another. Payment terms
(`net <n>`) cover this for recurring flows but not for `on_date`, which rejects
them outright, correctly, as having no accrual period.

No live case forced this; it is a symmetry gap noticed while fixing the
disposal discounting.

---

## Where these came from

Sections 1 and 2.1 were found building `benchmarks/cre/mit_rentleg_plaza`
against MIT OpenCourseWare 11.431J Problem Set 1 — the first CFDL benchmark
checked against a published third-party figure rather than an in-house
reference. That is the argument for building more of them: an external number
finds gaps that two of your own implementations agreeing never will. See
`research/CFDL_pack_roadmap_and_model_sourcing.md` for the catalogue.
