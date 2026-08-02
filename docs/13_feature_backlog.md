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

Blocks: the same benchmark. Also forces a duplicated opex formula — see 5.1,
which is the underlying cause.

Shape: a term that names a period rather than an amount, resolved after the
opex stream exists. Depends on 5.1.

### 1.3 Abatements as a first-class NOI line

`domain.cre.noi` sums base rent, recoveries, percentage rent, ops revenue, and
subtracts ops expense, vacancy and property opex. Free rent has no line.

The pack's own `cre.lease_unit` folds free rent into base rent, so it never
surfaces. Institutional pro formas report Abatements as its own deduction from
potential gross revenue — MIT's does. Today you can report it as a line **or**
have it counted in NOI, not both.

Shape: add an abatement stream family to the metric's denominator, and have
`cre.lease_unit` emit the deduction separately rather than netting it.

### 1.4 Coverage ratios are lifetime aggregates, not per-period tests

`domain.cre.dscr` divides total NOI by total debt service over the whole hold.
That is not what a debt service coverage ratio is. A lender tests coverage
**every year**, and a covenant breaches in a single year — a lifetime ratio of
1.4 can contain a year at 0.9 and report nothing.

`benchmarks/cre/hud_home_multifamily` is exactly that shape: coverage declines
from 1.576 to 1.289 across the hold as 2.5% expense growth outruns 2.0% rent
growth, and the source publishes the ratio at four separate years to sixteen
significant figures precisely because the path matters. We reproduce all four by
hand, and cannot assert any of them.

Shape: a per-period metric kind, so `dscr` yields a series and the aggregate
becomes one reduction of it (min, mean, or the covenant test "never below x").
The same applies to `domain.energy.dscr` and `domain.opco.fcf_to_debt_service`.
Note this is a metrics-layer change, not an engine one — the series it needs are
all already computed.

### 1.5 A property may have only one operating expense line

`cre.property_opex` emits `cre.property.opex` with no `{{contract.dot_suffix}}`,
so a model may declare exactly one. Every real pro forma splits management,
maintenance, utilities and taxes/insurance, and reports them separately because
that is how they are underwritten and covenanted.

Found building `benchmarks/cre/hud_home_multifamily`, whose source publishes all
four lines and which therefore has to carry them as one aggregate native stream.

Shape: add the suffix to the rule, and widen `domain.cre.noi`'s exact-name
selector to the `.*` prefix match it already uses for the rent families. Small,
and it is the difference between a toy pro forma and a real one.

### 1.6 Vacancy cannot track a growing rent roll

`cre.vacancy_loss` takes a constant `potential_gross_year` and multiplies it by
a rate. But potential gross rent grows — with escalation, with rollover, with
the end of a rent restriction — and the rule cannot see any of it, so vacancy
loss stays flat while the rent it is a percentage *of* rises.

Found the same way. In that deal vacancy also has to step 46% at the
affordability cliff, which no constant can do.

Shape: this is really 5.1 in miniature — the rule needs to read another stream.
Either the term accepts a stream reference, or vacancy becomes a phase-2 rule
reading the rent families through `series_sum`.

### 1.7 A rent restriction that expires

Affordable housing is rent-capped for an affordability period and reverts to
market afterwards. It is the defining mechanic of the asset class, and the HUD
source models it by carrying two rent tracks side by side and switching.

CFDL expresses it today as a hand-written `if(time.t < n, restricted, market)`
across two geometric series — workable, but it is a pack primitive, not a
one-off.

Shape: `cre.restricted_rent` with a `restriction_years` term and a market track,
or a `reverts_after` term on `cre.lease_unit`. Note the HUD template's own
switch fires a year before its stated period, which is the kind of convention a
pack rule should settle once rather than leaving to each modeller.

---

## 2. Credit pack

### 2.1 Age-varying prepayment and default curves — RESOLVED

`cpr` and `cdr` are single constants per contract. A hazard that varies with
loan age cannot be expressed, so the standard prepayment model — 0.2% CPR in
month 1 rising 0.2%/month to 6.0% at month 30, times a speed — is out of reach,
and so is its default counterpart.

The hazard itself is not the problem: `min(speed / 100 * 0.2 * max(1,
min(month, 30)), 100)` is closed-form and is already asserted in
`tools/analytic-checks.py`. The balance is. Every pool factor in the pack is
`pow(k, p)`, which is only valid for constant `k`; under a ramp the survival
factor is a cumulative product with no elementary closed form, and the
expression language has no `exp`/`ln` to sum logs instead.

**RESOLVED.** Declared state variables made the balance a running product, and
three contract terms select the published curves:

| term | curve |
|---|---|
| `psa_speed` | CPR rises 0.2%/month to 6.0% at month 30, flat after |
| `sda_speed` | CDR rises 0.02%/month to 0.60% at 30, flat to 60, to 0.03% at 120 |
| `abs_speed` | a constant fraction of ORIGINAL balance each month |

All default to `0`, which selects the flat `cpr`/`cdr` path, so no existing
model moved. The proposed calc builtin was not needed — the hazard shapes are
one-liners; only the balance was ever the problem.

Three external cases, all previously blocked on this:

| case | result |
|---|---|
| `benchmarks/credit/auto_abs_speed_050` | 0.0048 pp |
| `benchmarks/credit/auto_abs_speed_150` | 0.0036 pp |
| `benchmarks/credit/mbs_pool_ramped` | within 0.51, the source's rounding floor |

**Correction to what this item said while unblocked:** it claimed the remaining
work was "a pack change, not a language one". That was wrong twice.

1. A state advanced once per MODEL period while `{{time.elapsed_periods}}`
   counts a rule's PAYMENT periods. On a daily book paying monthly that is 365
   steps a year against 12. Fixed by giving states their own `schedule`
   (`docs/14_state_and_recurrence.md` §8) — a language change.
2. Two convention defects were found only by the external references, after
   every identity already passed:
   - all three ramps are indexed from loan ORIGINATION, not from the deal's
     closing, so a seasoned pool starts part-way up the curve. `age_months`
     carries it. Worth 20 percentage points of note balance at 1.50% ABS.
   - the lagged pool factor the recoveries rules read was consuming the hazard
     one lag too late. Invisible under a flat hazard; 7.6% on recoveries by
     month 60 under a ramp.

Both are the same shape: the ramp's form and the running product were correct,
and where each read lands on the curve was not. No in-house reference would have
caught either.

The original proposal follows, kept for provenance.

**Original note.** This is not item 5.2 (per-period state) and should not be
bundled with it. A PSA ramp is deterministic in loan age — nothing accumulates —
so the natural fix is a calc builtin holding the schedule, exactly the
`macrs_rate` pattern: a published table behind a function. Substituting one call
for `pow(k, p)` would make every existing pool rule work under a ramp.

**The Absolute Prepayment Model is the same item.** ABS speed is a constant
number of ORIGINAL units prepaying each month, so with the denominator fixed at
the original count and the pool shrinking, the implied SMM *rises* over the
life. Different convention, identical blocker: `k` is not constant, so
`pow(k, p)` is not the balance. One builtin holding a per-period survival
schedule would serve PSA, SDA and ABS alike.

Found building `benchmarks/credit/mbs_pool_conventions`. The constant-hazard
case (1% SMM / 1% MDR) reproduces to the reference figure; the ramped variant on
the same pool (150% PSA, 100% SDA) needs this. Confirmed again by
`benchmarks/credit/auto_abs_wal`, which can take only the zero-speed column of a
published seven-speed grid for exactly this reason.

### 2.2 Actual-day-count amortisation on a pool

**Resolved for the payment.** `amortization_day_count` now strikes the level-pay
payment while `day_count` accrues interest, so an Actual-convention loan holds a
constant payment with interest varying by month length. What remains is the pool
factor: `S(p)` is built from a single periodic rate, so a pool that *amortises*
on an Actual basis (rather than merely accruing on one) still needs a per-period
divisor inside the closed form, which has no elementary form.

Today `amortization_day_count = "act/360"` on a pool is accepted and computes a
month-length-varying schedule — correct for a single loan, an approximation for
a pool whose factor assumes constancy. Worth a validation gate, or an explicit
rejection, before an Actual-amortising pool benchmark lands.

Found closing the recovery gap in `benchmarks/credit/mbs_pool_conventions`; the
recovery basis itself is fixed and asserted there.

### 2.3 SMM and MDR as direct terms

*Partly relieved by 2.1: a pool at a published ramp now states `psa_speed` /
`sda_speed` / `abs_speed` directly and never touches `cpr`/`cdr`. What remains
is the flat case, where a hand-computed annual equivalent is still required.*

The pack accepts only annual `cpr`/`cdr`. Practitioners quote monthly SMM and
MDR — the published reference schedules are specified that way — so a 1% SMM pool
has to be entered as `cpr = 1 - 0.99^12 = 0.11361512828387077`, computed by
hand and unrecognisable to a reader.

Shape: accept `smm`/`mdr` alongside `cpr`/`cdr`, mutually exclusive with them
(the `terms_mutually_exclusive` check kind already exists), converting on the
way in. Note the conversion is cadence-dependent, so it belongs with
`{{model.periods_per_year}}` rather than a literal 12.

---

### 2.4 Sequential-pay note classes

The credit pack models collateral. It has no liability stack, so it cannot say
what a Class A-2 or a Class D receives, and every published ABS exhibit states
its answers per class.

`benchmarks/credit/auto_abs_wal` gets around this because Class A-1 had already
retired, leaving A-2 taking 100% of pool principal — so its pay-down *is* the
collateral's, scaled by one constant. That is luck, and it does not extend to
the other five classes in the same exhibit.

Shape: an ordered waterfall over available funds, with subordination, an
overcollateralisation target that excess spread turbos toward, and a reserve
account. This is the pack roadmap's waterfall item rather than a small addition,
and it is the single thing standing between this pack and mainstream consumer
ABS. Also wants the optional clean-up call, which every such exhibit reports
both with and without.

---

## 3. OpCo pack

### 3.1 A stub first period

`time calendar <c> from <d> for <n>` produces `n` periods of one length. A
valuation dated off a fiscal-year boundary — which is every live deal, because
valuation dates are negotiated and fiscal years are not — has a **stub** first
period, and there is no way to say so.

Reproducing a disclosed banker DCF made this concrete: valuation date 30
September, fiscal year ending 30 June, so the first forecast period is nine
months and the full years after it sit at 1.25, 2.25, 3.25 and 4.25 years out
rather than 1, 2, 3, 4. `benchmarks/opco/banker_dcf_conventions` works around it
by dropping to a monthly grid and placing each fiscal year's cash on the date
that carries its convention. Every exponent lands exactly, which is luck — the
offsets happen to be month boundaries except the stub's, which happens to be a
month midpoint. A valuation dated mid-month has no such out.

Shape: a leading partial period the calendar knows the length of, so period
lengths are `[stub, p, p, …]` and discounting, escalation and `elapsed_years`
all read it. Note the schedule grammar already has `stub short_front` /
`long_front` for *schedules*; this is the same idea for the *calendar*, and the
two should probably share a spelling.

### 3.2 A one-shot cannot settle at its period's end from surface syntax

`schedule on <date>` discounts from the period's open. A pack lowering rule can
move it to the close with `schedule_at_period_end` — added when the CRE
reversion turned out to be discounted a period short — but no surface syntax
exposes it, so a hand-written model cannot say it.

The workaround is a single-occurrence `every`: `schedule every month from
2025-12 to 2025-12` is an ordinary annuity, so it falls at its period's end.
That produces the right answer and reads like a workaround.

Shape: `schedule on <date> at end`, beside the `mid` modifier that already
works there. Small, and it closes an asymmetry between what packs can express
and what models can.

Found the same way. Related to 3.1 — both are about a flow whose position is
not one of the three the calendar offers.

### 3.3 A settlement lag's sub-period residual is dropped from discounting

Not found by the DCF, but exposed while deciding how `mid` should interact with
payment terms. `net <n>` is resolved on the calendar — billing date, plus the
lag, rolled for business days — and then the cash is moved into whichever
**period** the result lands in. The lag is therefore honoured to whole periods
and its remainder is dropped from the discounting.

On a monthly model with `net 30` that is exact. With `net 45`, the cash lands
one period later and the extra fifteen days are not discounted for. The error
is small and one-directional, and it applies to every schedule carrying payment
terms, not only to the ones that also want a position.

`mid` with `net` is rejected outright today
(`E2109_SCHEDULE_CONFLICTING_PLACEMENT`) rather than composed, because
composing them means answering this. Doing it properly means carrying a
fractional residual out of the bucketing step and adding it to the stream's
offset — which the offset mechanism already supports, since it is a float.

---

## 4. Energy pack

### 4.1 A rounding builtin, and the production credit that needs one

`energy_ptc_credit` carries the escalated credit rate as a continuous quantity.
The statutory credit is published **rounded to the nearest 0.1 cent per kWh**
after each year's inflation adjustment, so the real schedule is a staircase and
the pack computes the ramp underneath it.

The error alternates sign rather than drifting — reconciled over a 10-year
window it runs from -1.79% in year 1 to +1.18% in year 5, netting -0.30% over
the window. Small in aggregate, up to 1.8% in any single year, and a debt sizing
struck off one year's coverage will feel it.

The blocker is the language, not the pack: there is no `round_to(x, step)` in
the expression environment, so the staircase cannot be written.

**A second source now asks for the same builtin, and for more.** The HUD
multifamily template escalates expenses as a *recurrence* — each year is last
year's already-rounded figure times the trend — and two of its four expense
lines reproduce exactly under that rule and under no closed form. Expressing it
needs `round_to` **and** a backward period reference (5.1), because the input to
each year's rounding is the previous year's output. That combination is the
general case; the production credit above is the special case where the
recurrence happens to have a closed form. That builtin is
the item; the rule change is one call once it exists. It would also serve
tariff blocks, tranche denominations and any other quantity quoted to a tick.

Found reconciling `benchmarks/energy/utility_pv_singleowner` against an external
project-finance model. `benchmarks/energy/wind_ptc_macrs` asserts the unrounded
figure against an in-house generator, so both sides carried the same omission
and had always agreed.

### 4.2 A derived depreciable basis

`energy.macrs_shield` takes `basis` as an input. Taking an investment credit
conventionally reduces the depreciable basis by half the credit, so a model
claiming a 30% ITC on $100m must enter $85m by hand; entering $100m overstates
the shield by 17.6% and nothing objects.

Deliberate today — basis adjustments are jurisdictional and there are several,
and a wrong default is worse than no default. But the commonest one is
mechanical, and the pack already has both the credit and the cost in scope.

Shape: an optional `itc_basis_reduction` term on `energy.macrs_shield`, or a
cross-contract rule that reads `energy.itc`. The latter is phase-2 machinery for
a one-line arithmetic adjustment, so probably the former.

Found the same way. Documented in `packs/energy/README.md` in the meantime.

---

## 5. Language and engine

### 5.1 A stream may not read another period's value — RESOLVED

**Declared state variables shipped.** `docs/14_state_and_recurrence.md` is the
design; the construct is language-level and needs no pack:

```cfdl
state opex_index { init 1.0  next round_to(prev * 1.025, 1) }
```

`init` is mandatory, `next` sees only `prev`, `prev.<name>`, `time.*`,
`inputs.*` and curves — never a same-period value — so "cycles are impossible by
construction" survives rather than being traded for cycle detection. States are
published as `state.<name>` in results and never enter cash.

Two independent published sources confirm it, which is why this is marked
resolved rather than merely built:

| case | before | after |
|---|---|---|
| `benchmarks/opco/damodaran_fcff` | revenue −2.4% at year 10, years 6–10 unasserted | **all ten years exact** |
| `benchmarks/cre/hud_home_multifamily` | 12.26 residual, `period_tolerance = 13` | **exact, tolerance 0.5** |

Pack rules may declare a state too (`state_name`/`state_init`/`state_next`), so
the three opco growth rules compound through a running product without any model
being edited. Blast radius across 110 goldens: one 1.4e-13 relative shift and a
signed zero.

What it does NOT solve is unchanged and is 5.2: same-period cross-stream
dependency — a cash sweep needs cash remaining *after* this period's debt
service, and no backward-only construct reaches that.

The original statement of the problem follows, kept for provenance.

**Original design note.** It proposes a
declared state variable — named, mandatorily seeded, updated once per period,
readable by streams — whose update expression evaluates in an environment that
*excludes* same-period values. That keeps "cycles are impossible by
construction" intact rather than trading it for cycle detection, and it is the
design Lustre/SCADE, HDL registers, Analytica and Anaplan all converge on.

The entry below is the original statement of the problem, kept for provenance.


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

### 5.2 Per-period persistent state

**Scope boundary now settled** (`docs/14_state_and_recurrence.md` §5): the
backward-only state variable in 5.1 does *not* reach this. A cash sweep needs
same-period information — how much cash remains after this period's debt
service — which is an instantaneous dependency. The right shape here is an
**ordered allocation pass**: a waterfall is an author-declared priority over a
pot, not a dependency graph to be solved, so it needs no cycle detection either.
Design it separately; do not relax the stream reference rules to get it.


No accumulator, no carryforward, no balance that a period can add to and a
later period draw down. Cash sweeps, revolver draws, FF&E reserves, escrow
accounts, NOL carryforwards and construction-interest capitalisation all need
it, and `packs/opco/lowering/rules.toml` says so in its header.

Not discovered by this work — it is a known absence — but recorded here because
5.1 is a strictly smaller version of it and the two should share a design.

---

## 6. Cross-pack

### 6.1 Day count beyond the four supported bases

`{{model.accrual_divisor}}` handles `30/360`, `30e/360`, `act/360` and
`act/365`. `act/act` is not supported: it needs the days in the *year* the
period falls in, which the expression environment does not expose
(`time.days_in_period` is the period, not the year).

Low urgency — the four cover most instruments — but `act/act` is the government
bond convention and will be wanted if a sovereign or municipal pack appears.

### 6.2 excel_compat cannot be selected for a model run

`cfdl_expr::eval_with_mode` takes a `Mode`, and `Mode::ExcelCompat` evaluates
in IEEE-754 float64 to reproduce Excel's representation artifacts. But the
engine always calls the plain `eval`, which hardcodes `Mode::Decimal`, and
there is no CLI flag or run-config key. Nothing in the repo calls
`eval_with_mode` at all, so the capability is real at the library boundary and
unreachable above it.

Whether it would change anything is now measured rather than guessed:
`excel_compat_stability` in `crates/cfdl-calc/src/lib.rs` pins the credit
pack's arithmetic below 1e-12 relative across both modes. So this is not
urgent — it matters for a model that accumulates long sums or compares for
equality, which the packs do not.

Worth having when the first Excel-parity benchmark lands: the catalogue's
A.CRE and Finamodel workbooks are Excel, and "our decimal answer differs from
the spreadsheet in the fifteenth digit" is a question best answered by running
both ways rather than by argument.

**Now measured against a float64 reference.** Reconciling
`benchmarks/energy/utility_pv_singleowner` left a residual, and decomposing it
says what the mode is worth. Against the same reference, on the streams that are
a plain geometric series or a single `pmt`:

| stream | decimal (shipped) | float64 |
|---|---|---|
| `energy.om.expense` | 4.68e-7 | 4.66e-10 (1 ulp) |
| `energy.debt.service` | 2.54e-7 | 9.31e-10 (1 ulp) |
| `energy.ppa.revenue` | 9.15e-7 | 5.57e-7 |

So float64 lands on the reference to the last representable bit where the
reference's own arithmetic is short, and gains almost nothing on PPA revenue,
where the residual is accumulated error inside the reference's longer chain
rather than decimal-versus-float64 at all.

Two things follow. The mode would make a parity claim exact rather than merely
tight — worth having. And **closer is not more correct**: the decimal answer is
the exact one, and float64 agrees better precisely because it reproduces the
reference's rounding as well as its arithmetic. At 5e-7 on 2e6 that is 2e-13
relative, below any tolerance a case would declare. The caveat: this was
measured with an independent float64 reimplementation, so it indicates what the
mode would do rather than proving it — `pow` can differ by under an ulp between
libm implementations.

Found while validating the credit pack against an external reference and asking
whether Excel mode would move the numbers. It cannot be turned on to find out.

### 6.3 An acquisition or disposal in a period other than the term's

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

Items 1.4 through 1.7 were found building `benchmarks/cre/hud_home_multifamily`
against HUD's own populated underwriting Sample — the only source in the
programme that may be redistributed, and so the only one whose reference
workbook is committed beside the model.

Section 1's first three items and item 5.1 were found building `benchmarks/cre/mit_rentleg_plaza`
against MIT OpenCourseWare 11.431J Problem Set 1 — the first CFDL benchmark
checked against a published third-party figure rather than an in-house
reference. Section 2 came the same way, from `benchmarks/credit/mbs_pool_conventions`
against the published industry reference for MBS cash flows — which also found
three outright defects, in the prepayment base, the recovery basis and the
payment-striking divisor, all fixed rather than listed here.

Section 3 came from `benchmarks/opco/banker_dcf_conventions` against a
disclosed valuation in a public merger filing — the opco pack's first external
check. All nine cells of the banker's answer grid reproduce within $1.2mm on
$19bn. It also found two outright engine defects, fixed rather than listed
here: mid-period discounting had no spelling at all, and `on day <n>` divided
by a literal 30 on every calendar.

Section 4 came from `benchmarks/energy/utility_pv_singleowner` against a
national laboratory's open-source project-finance model — the energy pack's
first external check of any kind. Five rules reproduced it to within 1e-6
dollars on the first attempt; the two items above are what the reconciliation
found *around* the agreement.

**Section 7 is below this one, not above it.** New items are appended to the end
of the file rather than filed into the section they belong to, because the
numbers are positional and inserting one renumbers everything below it. Each
appended item names its home section.

That is the argument for building more of them: an external number finds gaps
that two of your own implementations agreeing never will. See
`research/CFDL_pack_roadmap_and_model_sourcing.md` for the catalogue.

---

## 7. Appended after the section numbering settled

New items go here rather than into the section they belong to. Backlog numbers
are positional, so inserting one renumbers everything below it and silently
breaks every `NOTES.md` reference and commit-message citation pointing past the
insertion — which has already happened once. Appending never renumbers. Each
item says which section it belongs with.

### 7.1 Storage revenue is a reduced form with an unquantified error

*Belongs with the energy pack (section 4).*

`energy.storage_arbitrage` is `mwh_cycled_year * spread * (1 - degradation)^y`.
The industry reference models a battery with a **dispatch optimiser** over an
hourly price series, so its revenue emerges from thousands of hourly
charge/discharge decisions. The two do not reduce to one another and no choice
of inputs makes them agree — fitting `spread` until they matched would be
calibration, not validation.

So this rule has **no external validation**, and
`benchmarks/energy/merchant_capacity` says so rather than quietly including it.
Energy is at 9 of 10 rules.

**Attempted, and this is what it showed.** A dispatch run was tried rather than
assumed: SAM's `Battwatts` model, 20 MW / 80 MWh behind a 100 MW PV plant,
diurnal generation and an evening-peak load.

- Behind the meter, it discharged **27.9 MWh across a whole year** from an
  80 MWh battery, and charged nothing. Not a bug — dispatch is driven entirely
  by the load and price context, and with 100 MW of PV against a 55 MW peak
  there was nothing for the battery to do.
- Reconfigured front-of-meter for merchant arbitrage, the native library
  **segfaulted** (exit 139).

The first result is the important one, and it sharpens the item: `mwh_cycled_year`
is an **input** to our rule and the primary **output** of a dispatch model. The
quantity we ask the modeller to state is the thing the reference exists to
compute. So the two cannot be compared without first deciding the answer — which
is why "fit `spread` until they agree" is calibration and not validation.

A real comparison needs the full `Battery` module with a price-signal dispatch
choice and a generation chain, not `Battwatts`. Worth doing, but it is a
scoping exercise of its own rather than a benchmark.

The reduced form is not wrong; practitioners use exactly this shape at the
financing stage. What is missing is a bound on its error. Two ways forward, in
order of cost:

- **A price-duration curve input.** CFDL already has `curve` declarations (used
  for floating-rate credit). Integrating a spread against a duration curve is
  materially more faithful and needs no new engine capability.
- **State of charge**, which needs per-period persistent state (5.2) and would
  let cycling be modelled rather than assumed.

True hourly dispatch optimisation is out of scope and should stay there — that
is an optimiser, not a declarative cash-flow model.

### 7.2 `round_to` is half of the recurrence problem — RESOLVED

*Belongs with the energy pack (section 4), and closes the first half of 4.1.*

`round_to(x, step)` now exists and the production tax credit's statutory
staircase is expressed and asserted, so **4.1 is done for the ramp case**.

**RESOLVED.** The other half was 5.1, and it has landed. The HUD multifamily
workbook escalates expenses as a **recurrence** — each year is last year's
already-rounded figure times the trend — which is now written directly:

```cfdl
state opex_management { init inputs.opex_management  next round_to(prev * (1 + inputs.opex_trend), 1) }
```

Both expense lines in `benchmarks/cre/hud_home_multifamily` reproduce the
published figures exactly over 29 years, and the case's `period_tolerance` drops
13 → 0.5. So 4.1 is now closed for both the ramp case and the general one.

One thing that surfaced only by building it: the total expense line needs **four
states, one per published sub-line**. The workbook rounds each sub-line before
summing, and rounding the sum is different arithmetic — modelling the total as a
single rounded line still left 11.00 of the original 12.26.

### 7.3 The external cases route around the packs they should be validating

*Belongs with no single pack — it is about the validation programme.*

Measured across the six externally-reconciled benchmarks, counting pack contract
types exercised by at least one of them:

| pack | externally validated | not covered |
|---|---|---|
| energy | **9 / 10** | `storage_arbitrage` |
| credit | 1 / 4 | `pool_io_bullet`, `pool_float_io_bullet`, `purchase` |
| cre | 1 / 12 | everything but `exit_forward` |
| opco | **0 / 10** | everything |

And by construction: `hud_home_multifamily` is 0 pack contracts and 6 native
streams, `banker_dcf_conventions` 0 and 6, `mit_rentleg_plaza` 1 and 10. The
credit and energy cases are the opposite — `auto_abs_wal` is 43 contracts and
no native streams.

Each case documents why its pack rules did not fit — single-instance opex,
non-escalating vacancy, sources that publish per-year figures rather than
drivers. But the aggregate is circular: **the two packs with the weakest rule
coverage are exactly the ones whose benchmarks bypass the pack**, so for cre and
opco we are validating the engine, not the domain logic.

Two things follow, and they are separable:

- **Fix the rules the cases tripped over** (1.5, 1.6, 1.7) so a CRE deal can be
  expressed in pack contracts at all. That converts an existing case rather than
  needing a new source.
- **Choose sources that disclose drivers, not outputs.** A fairness opinion
  publishes the unlevered cash flow; a sponsor model publishes the growth rate,
  margin path and working-capital policy that produce it. Only the second can
  validate `opco.revenue_line`.

Recorded because the headline "four domains externally validated" is true and
does not mean what it sounds like.

### 7.4 A discount rate cannot vary over time

*Belongs with the language and engine (section 5).*

`RunConfig.discount_rate` is a single `f64`, turned into one `per_period_rate`
and handed to `npv_with_offsets`. Every discounted figure in a model uses it.

Intrinsic valuation converges the cost of capital as a firm matures —
Damodaran's model runs 7.055% for five years and 8.81% thereafter. Project
finance uses one rate through construction and another in operation. Neither is
exotic and neither is expressible.

The consequence is not cosmetic: `benchmarks/opco/damodaran_fcff` asserts the
entire cash-flow build and **no discounted figure at all** — not NPV, not
enterprise value, not the per-share price the source exists to produce.
Discounting at a flat rate would have produced a number and not a check.

Shape: a discount *curve* alongside the scalar, read per period. Note the offset
machinery in `npv_with_offsets` already handles per-STREAM variation; this is
per-PERIOD variation, a different axis, and it touches IRR too — `irr_with_offsets`
solves for a single rate by construction.

### 7.5 Candidate contracts, and the packs that need them

*Belongs with the CRE and OpCo packs (sections 1 and 3).*

Every entry below was forced by a source, not proposed from taste. Listed
together because the shape of the gap is the same in both packs: the contracts
that exist model an operating business well and stop at the point where a deal
gets financed or valued.

**CRE — the pack cannot borrow money.**

| candidate | forced by |
|---|---|
| ~~`cre.permanent_debt`~~ | **SHIPPED.** Interest-only period, level payment, balloon opt-in, one combined `loan.permanent_debt_service` stream so `domain.cre.debt_service` needs no change. DSCR-based sizing is a solve and stays out. |
| `cre.construction_loan` | The same gap on the construction side, and now with a source: `benchmarks/cre/one_lincoln_street` publishes a sixteen-quarter draw schedule against an equity commitment that depletes mid-quarter. `cre.construction_stub` takes a flat draw and cannot express an equity-first waterfall, which is why that case runs on native streams. |
| `cre.restricted_rent` | HUD — rent capped for an affordability period and reverting to a market track. The defining mechanic of affordable housing, currently a hand-written conditional. |
| `cre.abatement` | MIT — free rent as its own deduction from potential gross revenue. Today it can be reported as a line or counted in NOI, not both (1.3). |
| `cre.replacement_reserve` | HUD — a capital reserve, separately published and semantically distinct from operating expense. Also One Lincoln Street, whose operating pro forma carries a Capital Reserve line. |

With 1.5, 1.6 and 1.7, these are what would let a real CRE deal be expressed in
pack contracts instead of native streams — which is the actual fix for 7.3 on
the CRE side, and needs no new source.

**OpCo — no terminal value a valuation practitioner would recognise.**

| candidate | forced by |
|---|---|
| ~~`opco.exit_perpetuity`~~ | **SHIPPED**, and validated against a published nine-point growth sensitivity grid (`benchmarks/opco/gordon_growth_coned`). `discount_rate` is a contract term, which is faithful to the sources rather than a workaround: a terminal cost of capital is not the near-term one. A stream-derived variant is the follow-on. |
| `opco.exit_forward_multiple` | The banker DCF — a forward (NTM) multiple struck at a point before model end. |
| `opco.depreciation` | No D&A contract exists, yet `opco_cash_taxes` consumes `da_monthly` as a bare term with no rule producing it. |
| `opco.equity_bridge` | Both opco sources — debt, cash, minority interests and non-operating assets between enterprise and equity value. Done outside the model today. |
| `opco.share_count` | Both — a share count that dilutes over time, so per-share value is expressible at all. |
| `opco.revolver`, `opco.cash_sweep`, `opco.nol_carryforward` | Every LBO source. All three need per-period state (5.2) and should be designed with it rather than before it. |

**Elsewhere.** `energy.storage_dispatch`, a curve-integrated storage rule so
arbitrage is priced against a duration curve rather than a scalar spread (7.1).
Credit's three uncovered contract types need a source, not a new contract.

### 7.6 Diagnostic codes are not unique — RESOLVED

*Belongs with the language and engine (section 5).*

`packs/opco/validations.toml` uses `E7010` for **two** different checks
(`OPCO_LINE_AMBIGUOUS_AMOUNT` and `OPCO_WC_MISSING_AMOUNT_OR_RULE`) and `E7011`
for two more (`OPCO_TAXES_AMBIGUOUS_DA` and `OPCO_WC_INVALID_SCHEDULE`).
`docs/08_diagnostics.md` documents both meanings of both codes, in different
sections.

The numeric prefix is the stable identifier — it is what a user greps for, what
a support conversation quotes, and what a downstream tool would match on. Three
unrelated failures answering to one code makes all three unsearchable.

Found while adding a check and colliding with `E7010` a third time. The new one
was moved to `E7012`; the existing pair was left alone deliberately, because
renumbering a shipped diagnostic is a breaking change for anyone matching on it
and should be a decision rather than a side effect.

**Resolved.** All three renumbered — working capital to `E7013`/`E7014`, the
lease-unit escalation check to `E6033` — and `tools/check-pack-validations.py`
now fails CI when a code names two checks.

Worth recording how it went, because it justifies the gate more than the
original defect did: the first replacement code chosen was **also already
taken** (`E6031` belonged to `CRE_UNIT_INVALID_FREE_RENT`), creating a fourth
collision in the act of fixing the first three. Picking a free number by reading
the file is not reliable.

### 7.7 Two thirds of pack validations never run — RESOLVED

*Belongs with the language and engine (section 5). Highest-severity item on this
list — everything else here is a missing capability; this is a safety net with a
hole in it.*

A pack validation matches a contract by exact name unless it declares
`match = "instance"`. Contracts are routinely written in the suffixed form —
`credit.pool_level_pay.auto_a`, `cre.lease_unit.anchor`, `opco.revenue_line.core`
— and for those, a validation without that flag is **silently skipped**.

Measured across the shipped packs:

| pack | validations | declare `match = "instance"` | skipped on suffixed contracts |
|---|---|---|---|
| credit | 10 | 10 | 0 |
| cre | 14 | 5 | **9** |
| energy | 9 | 0 | **9** |
| opco | 15 | 1 | **14** |

**33 of 48.** Credit was done correctly and the other three packs were not, which
is why this has never been noticed: the pack with the most validation coverage is
also the only one where the validations fire.

Demonstrated, not inferred. `opco.revenue_line` stating no amount at all is
rejected with `E7001_OPCO_LINE_MISSING_AMOUNT`; `opco.revenue_line.core` stating
no amount compiles clean. Same model, same defect, one character of difference.
The same holds for the CRE and energy checks without the flag — including
`E5019` day-count validation and the CRE lease and exit-cap bounds.

Found while adding `E7012_OPCO_TAXES_MISSING_RATE` and testing that it actually
fires, which it did not until the flag was added. Writing a validation and never
confirming it rejects anything is how thirty-three of these got here.

**Not fixed in that change, deliberately.** Adding the flag everywhere is a
one-line edit per validation, but it turns thirty-three dormant checks live at
once, and any existing model that violates one would start failing. That needs
to be a change of its own where each newly-firing check is reviewed. A first
attempt at the blanket edit broke TOML parsing in nine places (the flag has to
follow the *close* of a multi-line `contracts` array), which is a fair warning
about doing it quickly.

**Resolved.** All 48 validations across all four packs now declare
`match = "instance"`, and `tools/check-pack-validations.py` requires every
validation to state its match mode explicitly — `exact` remains available, it
just has to be written. Defaulting was the trap.

Blast radius was measured before the change: only 8 of the 33 had any suffixed
usage in shipped models to trip on, all bounds or ambiguity checks. Prediction
was that no golden would move. **None did** — 108/108 still pass, so eight
previously-dormant checks are now live and every shipped model already satisfied
them.

Still open, as a deliberate follow-on: whether `Instance` should be the *default*
in `crates/cfdl-pack/src/lib.rs:119-124`, which would make all 48 declarations
redundant. Left alone for now because the explicit-declaration gate achieves the
same safety and makes the choice visible at each call site.

### 7.8 A stream cannot be non-cash, which blocks the log-sum technique

*Belongs with the language and engine (section 5). Small surface, and it is the
only thing standing between `exp`/`ln` and closing 2.1.*

`ln` and `exp` now exist so a cumulative **product** can be computed as a
cumulative **sum**:

    PROD(1 + r_i)  ==  exp(series_sum("ln_one_plus_r", 0, t))

with a helper stream carrying `ln(1 + r_t)`. **Verified end to end**: a probe
reproduces all ten years of a published forecast whose growth rate decays,
exactly, where `pow(1 + g, t)` drifts to −2.4% by year 10.

The blocker is that **every stream is a cash stream**. `record_stream`
(`crates/cfdl-engine/src/lib.rs`) adds each one to `model_series`, so the helper
lands in net cash flow and NPV. In the probe, year-1 net cash flow came to
22,853.7188 against a true 22,853.6700 — the extra 0.0488 is a dimensionless
logarithm being added to dollars.

So the technique cannot be used from a pack rule, which is where it is needed:
`credit` for the PSA/SDA/ABS ramps (2.1) and `opco` for a decaying growth path.

Shape, and the choice matters:

- **A non-cash stream kind** — `stream ... informational` or similar, computed
  and readable by `series_sum` but excluded from `model_series`, totals and NPV.
  General, and it also gives packs somewhere to put intermediate quantities that
  are not money, which several rules currently inline and recompute.
- **A `curve_sum` builtin** — aggregate a *curve* over a period window without a
  stream at all. Narrower: it serves rates that are already curves (the opco
  growth path) but not rates that are a formula of loan age (PSA, SDA), which
  is most of 2.1.

The first is the better answer for the same reason the delayed-reference design
was: it solves the general case rather than the instance in front of us. It is
also a genuine language surface addition, so it wants deciding rather than
assuming — note that a non-cash stream has knock-on questions for the results
schema, the domain metrics, and whether it appears in `series` at all.

**Update — largely superseded, and the knock-on questions are answered.** The
log-sum technique existed to reach a recurrence, and 5.1 now reaches it directly
and decimal-exactly, without escaping to `f64` and without a helper stream. Both
cases named above are closed: the opco growth path by rule-declared states, the
credit ramps expressible the same way.

A declared `state` also happens to be the non-cash quantity this item asked for,
and shipping it settled every question listed: it appears in `series` under a
`state.` prefix, as bare numbers rather than Money, and it is excluded from
`model_series`, totals, NPV, the annual rollup and every domain metric — with an
identity asserting exactly that.

What survives is narrower: a non-cash quantity that must aggregate a *stream's*
values over a window (`series_sum` over something that is not cash). A state
cannot do that, because `next` has no series access. Nothing currently needs it.

### 7.9 `opco.capex_line` cannot express a derived line

Found closing 5.1 against `benchmarks/opco/damodaran_fcff`, and worth separating
because the old drift table made it look like the same defect it is not.

Reinvestment is **derived** from another line: `revenue(t) * g(t+1) /
sales_to_capital`. It funds *next* year's growth, so its own growth factor is
`(1 + g_t) * g_{t+1} / g_t`, which leads the revenue growth path by one year:

| | yr 5 | yr 6 | yr 7 | yr 8 | yr 9 | yr 10 |
|---|---|---|---|---|---|---|
| reinvestment grows | 3.24% | 3.12% | 3.01% | 2.89% | 2.78% | 4.58% |
| revenue grows | 5.00% | 4.92% | 4.83% | 4.75% | 4.66% | 4.58% |

`opco.capex_line` is a self-growing line — a base times a rate path — so it
cannot express a quantity defined by another line's growth. No recurrence fixes
this; it is a contract shape gap. The benchmark therefore asserts reinvestment
for years 1–4 only, which is honest rather than fitted: deriving a
reinvestment-ratio curve by hand would pass and would hide the gap, exactly as a
cumulative-index curve would have hidden 5.1.

Shape: a reinvestment contract taking a revenue reference, a growth curve and a
sales-to-capital ratio, reading revenue through `series_sum` as
`opco.working_capital_policy` already does. That also makes the FCFF identity
(`EBIT(1-t) − reinvestment`) expressible from drivers rather than from a
hand-computed base.

### 7.10 A state's `next` cannot read a stream's history

Deliberate in v1 of `docs/14_state_and_recurrence.md` §3.1, and recorded so it is
a stated boundary rather than a silent gap.

`next` sees `prev`, `prev.<name>`, `time.*`, `inputs.*`, `cfg`, `obs` and curves.
It does **not** see stream series. The design permits series up to `t-1`, but
enforcing "up to `t-1`" would mean truncating the series map per period — an
O(n) copy per period, O(n²) overall — and, worse, would make the restriction a
runtime *check* rather than an *absence*, which is the property the whole design
exists to preserve.

Nothing currently needs it: every shape the backlog asks for is multiplicative
(survival factors, escalation indices, degradation, discount factors), additive
(accumulators) or a running maximum (high-water marks). What it would unlock is a
state that accumulates a stream — a reserve balance fed by actual collections, a
carryforward of realised losses.

Shape when needed: a borrowed truncated view rather than a copy, so the cost is a
slice and the restriction stays structural.

### 7.11 Engine diagnostic codes have no uniqueness gate

`tools/check-pack-validations.py` gates `packs/*/validations.toml`, where authors
add codes by hand and where four collisions occurred. It does not see codes the
**engine** emits, which live in Rust string literals.

That gap bit while adding the lowering diagnostics for 5.1: `E5010` and `E5011`
were picked by reading the file and both were already taken
(`E5010_TERM_UNKNOWN_INPUT`, `E5011_TERM_CLIP_OUT_OF_BOUNDS`). The same failure
as 7.6, one layer over, and by the same method — picking a free code by eye is
not reliable.

Mitigated: the gate now also checks numeric-prefix uniqueness across
`docs/08_diagnostics.md`, the published register where every engine code is
listed. Confirmed to bite.

What is still missing is the other half of the pair — a check that every code
emitted in non-test Rust actually appears in that register. Without it the doc
gate only fires for codes someone remembered to document. Extracting from Rust
needs to exclude the deliberately corrupted codes inside parser tests
(`E7001_WRONG_PACK` and friends), which is why it was not done in the same pass.

### 7.12 A pool's amortisation state is not exposed

Found building `benchmarks/credit/auto_abs_speed_050` and `_150`. The published
figure in an ABS exhibit is *percent of a note class outstanding*, which is
derived from cumulative pool principal. There is no single stream carrying that,
and `tools/benchmark-runner.py` checks per-stream series and scalar metrics only,
so the reconciliation lives in `NOTES.md` and is not machine-checked. Both new
cases fall back to a `net_cash_flow` regression guard plus
`domain.credit.principal`.

The same limitation applies to `benchmarks/credit/auto_abs_wal`, and it is why
that case's external evidence has always been prose.

Shape: either a `domain.credit.pool_factor` per-period metric, or letting a
benchmark case assert a named expression over streams. The second is more
general and would serve the CRE debt-service-coverage case too.

### 7.13 District energy has no usable reference model

Scoped, not built. `research/CFDL_pack_roadmap_and_model_catalogue.xlsx` ranks
District Energy / Waste-to-Energy as a Tier 1 pack candidate with the gate
"None new — Energy pack extension (~65% reuse)", and names the Ed Bodmer project
finance collection as the first reference to build against.

That collection does not contain one. Measured across its thermal and
biomass/biogas pages:

| term | thermal page | biomass page |
|---|---|---|
| "district" | 0 | 0 |
| "cogeneration" | 0 | 0 |
| "combined heat" | 0 | 0 |
| "waste" | 0 | 3 (prose) |

The four downloadable thermal models are gas-fired IPPs (`IPP-Model.xlsm`,
`Gas-Plant-Example`, `Indonesia-Gas-Plant`, `NGCC-with-Merchant`). Their
structure — PPA and merchant revenue, O&M, senior debt, tax depreciation — is
what `benchmarks/energy/utility_pv_singleowner` and
`benchmarks/energy/merchant_capacity` already reconcile against a national
laboratory model, so they would add a second source for mechanics already
covered rather than the new ones the candidate needs (thermal load, fuel cost,
heat offtake).

They also lean on two things that are not expressible: debt sized to a target
coverage ratio ("sculpt" appears 41 times on the thermal page) and capitalised
construction interest resolved circularly ("circular", 28 times). Both are
solves. `docs/14_state_and_recurrence.md` §5 covers why an iterative construct
would need to be explicit, bounded and convergence-checked rather than implied.

What a district energy case actually needs is a source publishing a thermal
plant's drivers and the lines they produce — heat and power sold separately,
fuel cost as a driver, and a heat offtake contract. The catalogue's remaining
Tier 1 entries with no new gate are Telecom Towers (#9, A.CRE single-tenant NNN)
and Hospitality (#3/#20, A.CRE or Finamodel), both of which require an email
registration to download.

### 7.14 A CRE mortgage cannot pay monthly on an annual model, and MIP is not debt service

Found converting the CRE benchmarks onto `cre.permanent_debt`.
`benchmarks/cre/office_two_tenant` converted cleanly and reproduces its debt
service exactly. `benchmarks/cre/hud_home_multifamily` cannot, for two separate
reasons.

**Cadence.** The HUD workbook's first mortgage is $150,000 at 4.00% over 15
years paid MONTHLY, and its published annual line is the annualised monthly
payment, 13,314.38. The model runs on an annual calendar, and a rule paying
monthly there is `E2108_SCHEDULE_FINER_THAN_CALENDAR`. Striking the payment
annually instead is out by $177 a year.

This is the mirror of the case that motivated state schedules: there, a rule
paid *less* often than the calendar and the fix was to give the recurrence its
own clock. Here the rule pays *more* often than the calendar, and the period
grid genuinely cannot hold it — several payments would collapse into one. The
shape, if it is ever wanted, is a sub-period accrual that reports at the
calendar's grain, which is a larger change than a schedule.

**Mortgage insurance.** The published line is P+I+MIP; the residual is exactly
675.00, or 0.450% of the original principal, flat. MIP is not a payment on the
debt and `cre.permanent_debt` deliberately does not model it. An FHA/HUD-insured
multifamily loan is common enough that a `cre.mortgage_insurance` contract —
or a `mip_rate` term on a separate insurance line, not on the debt — would be
the honest shape. It affects `domain.cre.dscr`, since coverage is measured
against P+I+MIP.

Until then this case's debt stays a native stream, which is why CRE's pack-rule
coverage counts it as unconverted.

### 7.15 Adding a contract and a source did not move CRE's coverage

Measured after shipping `cre.permanent_debt` and
`benchmarks/cre/one_lincoln_street`. Counting pack contract types exercised by
at least one **externally reconciled** case:

| pack | before | after |
|---|---|---|
| energy | 9/10 | 9/10 |
| opco | 4/10 | **5/11** |
| credit | 1/4 | 1/4 |
| CRE | 1/12 | **1/13** |

CRE went *backwards as a ratio*, and the reason is worth stating rather than
smoothing over.

**The contract's only user is an in-house case.** `cre.permanent_debt` converted
`benchmarks/cre/office_two_tenant`, whose `case.toml` says plainly: *"Reference:
reference_gen.py (independent implementation). Status: pending practitioner
review."* Its figures are ours. `hud_home_multifamily` — which is external —
could not convert, for the two reasons in 7.14.

**The new source exercises no contract.** One Lincoln Street's funding waterfall
is an equity commitment that depletes mid-quarter, and `cre.construction_stub`
takes a flat draw, so the case runs on native streams and a declared state.

Neither is a defect; both are the same gap seen from two sides. What CRE needs
is a source that publishes the DRIVERS of an operating pro forma — a rent roll
with escalations and expense stops — rather than its results. One Lincoln
Street's Exhibit 5 is exactly that pro forma and was rejected for exactly that
reason: it publishes the lines, not the leases.

Every direct-download CRE candidate in the catalogue has now been checked. The
remaining ones (A.CRE, Finamodel, PropertyMetrics) require an email
registration, which is the actual blocker on CRE coverage — not the pack.
