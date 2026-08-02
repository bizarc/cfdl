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

### 2.1 Age-varying prepayment and default curves

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

**This is not item 5.2 (per-period state) and should not be bundled with it.** A
PSA ramp is deterministic in loan age — nothing accumulates — so the natural
fix is a calc builtin holding the schedule, exactly the `macrs_rate` pattern:
a published table behind a function. Substituting one call for `pow(k, p)`
would make every existing pool rule work under a ramp.

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

### 5.1 A stream may not read another period's value

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

### 7.2 `round_to` is half of the recurrence problem

*Belongs with the energy pack (section 4), and closes the first half of 4.1.*

`round_to(x, step)` now exists and the production tax credit's statutory
staircase is expressed and asserted, so **4.1 is done for the ramp case**.

What remains is the general case, and it is a different item. The HUD
multifamily workbook escalates expenses as a **recurrence** — each year is last
year's already-rounded figure times the trend — which needs a stream to read its
own prior period. That is 5.1, and until it exists the recurrence is
inexpressible no matter how good the rounding builtin is.

Recorded so nobody reads 4.1 as fully closed. Found in
`benchmarks/cre/hud_home_multifamily`, where two of four expense sub-lines
reproduce exactly under the recurrence and under no closed form.

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
| `cre.permanent_debt` | The largest single gap in any pack. There is no debt contract at all, so every CRE benchmark hand-writes a mortgage, and `domain.cre.dscr` works only because it reads the native stream *names* `loan.permanent_debt_service` and `loan.construction_interest` by convention. Debt service coverage is the headline CRE metric and the thing producing it is not a primitive. Wants an interest-only period and DSCR-based sizing. |
| `cre.construction_loan` | The same gap on the construction side. |
| `cre.restricted_rent` | HUD — rent capped for an affordability period and reverting to a market track. The defining mechanic of affordable housing, currently a hand-written conditional. |
| `cre.abatement` | MIT — free rent as its own deduction from potential gross revenue. Today it can be reported as a line or counted in NOI, not both (1.3). |
| `cre.replacement_reserve` | HUD — a capital reserve, separately published and semantically distinct from operating expense. |

With 1.5, 1.6 and 1.7, these are what would let a real CRE deal be expressed in
pack contracts instead of native streams — which is the actual fix for 7.3 on
the CRE side, and needs no new source.

**OpCo — no terminal value a valuation practitioner would recognise.**

| candidate | forced by |
|---|---|
| `opco.exit_perpetuity` | Damodaran — a growing perpetuity is *the* intrinsic-valuation terminal. `opco.exit_multiple` is a run-rate multiple and `opco.exit_ebitda` is TTM; neither is this, so the largest component of value in a DCF cannot be expressed. |
| `opco.exit_forward_multiple` | The banker DCF — a forward (NTM) multiple struck at a point before model end. |
| `opco.depreciation` | No D&A contract exists, yet `opco_cash_taxes` consumes `da_monthly` as a bare term with no rule producing it. |
| `opco.equity_bridge` | Both opco sources — debt, cash, minority interests and non-operating assets between enterprise and equity value. Done outside the model today. |
| `opco.share_count` | Both — a share count that dilutes over time, so per-share value is expressible at all. |
| `opco.revolver`, `opco.cash_sweep`, `opco.nol_carryforward` | Every LBO source. All three need per-period state (5.2) and should be designed with it rather than before it. |

**Elsewhere.** `energy.storage_dispatch`, a curve-integrated storage rule so
arbitrage is priced against a duration curve rather than a scalar spread (7.1).
Credit's three uncovered contract types need a source, not a new contract.

### 7.6 Diagnostic codes are not unique

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

Shape: a uniqueness check over every pack's `validations.toml` and the engine's
own codes, wired into `make ci` — the same move that turned the IR and results
schemas from documentation into gates. Then renumber the duplicates in one
deliberate change with a note in the changelog.

### 7.7 Two thirds of pack validations never run

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

Shape, in order:
1. a gate that fails CI when a validation names a contract that any shipped
   model uses in suffixed form without `match = "instance"` — same move as the
   schema gates;
2. fix the thirty-three, reviewing what each one starts rejecting;
3. consider whether `instance` should be the *default*, since exact-only is
   almost never what an author means, and make `exact` the opt-in.
