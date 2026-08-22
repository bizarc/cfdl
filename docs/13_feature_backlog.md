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

### 1.3 Abatements as a first-class NOI line — RESOLVED

**Resolved.** `cre.lease_unit` now emits `cre.unit.abatement.<id>` as its own
deduction and publishes base rent GROSS; `domain.cre.noi` carries the abatement
family in its denominator, so the two still net to the rent collected. Verified
as an exact decomposition: gross + abatement equals the previous net to 0.00e+00
on `cre_office_two_tenant`, and NPV, NOI and DSCR are unchanged.

A stream may also declare `category operating.deduction.abatement` directly,
which is how `benchmarks/cre/mit_rentleg_plaza` has its hand-written abatement
counted — the pack has no contract for that shape and a name-based selector
could never have reached it.


`domain.cre.noi` sums base rent, recoveries, percentage rent, ops revenue, and
subtracts ops expense, vacancy and property opex. Free rent has no line.

The pack's own `cre.lease_unit` folds free rent into base rent, so it never
surfaces. Institutional pro formas report Abatements as its own deduction from
potential gross revenue — MIT's does. Today you can report it as a line **or**
have it counted in NOI, not both.

Shape: add an abatement stream family to the metric's denominator, and have
`cre.lease_unit` emit the deduction separately rather than netting it.

### 1.4 Coverage ratios are lifetime aggregates, not per-period tests — RESOLVED

**Resolved.** `domain.cre.dscr` is a per-period series, declared in
`packs/cre/statements.toml` as a `ratio` subtotal over `domain.cre.noi` and
`domain.cre.debt_service`. `benchmarks/cre/hud_home_multifamily` asserts the
four published values this item is about, at every anchor year, as a column in
`expected.csv` — converting `NOTES.md`'s "we reproduce all four by hand, and
cannot assert any of them" into four machine-checked assertions against a public
HUD workbook.

The same shape now exists in the other packs: `domain.energy.dscr_periodic`
against CFADS, which is the covenant a project lender tests, and
`domain.opco.debt_service_coverage`.

**A ratio is recomputed at whatever grain it is reported at**, never averaged
from finer ratios. That is the trap this item implies and it is guarded by an
analytic check with a probe where the two answers differ by more than a factor
of two — verified by mutation to fail if the implementation ever averages.

Still open, and deliberately: the reduction over the series — "never below x" as
a covenant test — is not built. The series is what makes it expressible;
`min`/`mean` over a subtotal is a metrics-layer addition with no external user
yet.

#### Original entry: 1.4 Coverage ratios are lifetime aggregates

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

### 1.5 A property may have only one operating expense line — RESOLVED

**Resolved.** `cre_property_opex` takes `{{contract.dot_suffix}}` and
`domain.cre.noi` selects `cre.property.opex.*`. `benchmarks/cre/hud_home_multifamily`
now carries its four published sub-lines as four streams, and
**asserts all four independently** against the Sample workbook's Operating Pro
Forma rows 18–21 — where it previously asserted only their total. The states
were already per-sub-line for the rounding reason, so the split moved nothing.


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

**What is left, now that declared state has shipped.** The original entry said
"no accumulator, no carryforward, no balance that a period can add to and a
later period draw down", and listed six things needing it. That sentence is no
longer true and contradicted the paragraph above it — a state IS an accumulator,
and `next` reading `prev` IS a balance a period adds to and a later one draws
down. The list splits:

- **Still blocked**, because they need SAME-PERIOD information: cash sweeps and
  revolver draws. How much cash remains after this period's debt service is an
  instantaneous dependency, and no backward-only construct reaches it. This is
  the ordered allocation pass above, and it is what remains of 5.2.
- **Now expressible** as backward recurrences: FF&E reserves, escrow accounts,
  NOL carryforwards and construction-interest capitalisation.
  `benchmarks/opco/lbo_circular_interest` carries an average-balance interest
  schedule with no iteration, which is the shape all four take.

Not discovered by this work — it was a known absence — but recorded here because
5.1 was a strictly smaller version of it and the two shared a design.

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
| ~~`cre.construction_loan`~~ | **SHIPPED.** Equity-first funding behind a commitment, the facility taking the balance once it depletes, interest on the drawn balance. The draw schedule stays a model `curve` and the contract names it, because a funding profile is per-deal data rather than a term. `benchmarks/cre/one_lincoln_street_contract` reproduces the primitive-built case in all 48 cells with zero difference — the pair is the assertion, and if they disagree the contract is wrong. Capitalised interest is a follow-on: affine in the closing balance, so it collects rather than needing a solver. |
| `cre.restricted_rent` | HUD — rent capped for an affordability period and reverting to a market track. The defining mechanic of affordable housing, currently a hand-written conditional. |
| `cre.abatement` | MIT — free rent as its own deduction from potential gross revenue. Today it can be reported as a line or counted in NOI, not both (1.3). |
| `cre.replacement_reserve` | HUD — a capital reserve, separately published and semantically distinct from operating expense. Also One Lincoln Street, whose operating pro forma carries a Capital Reserve line. |

With 1.5, 1.6 and 1.7, these are what would let a real CRE deal be expressed in
pack contracts instead of native streams — which is the actual fix for 7.3 on
the CRE side, and needs no new source.

**A correction to how 7.3 framed this.** That entry treats a benchmark running
on native streams as a coverage failure. It is not, or not only. A case built
from primitives proves the LANGUAGE expresses the deal with no domain vocabulary
— which is the stronger claim, and the one a reader evaluating CFDL as a
language can check. A pack contract is an ergonomics layer for a practitioner
who should not have to derive an equity-first waterfall from scratch.

So the fix is not to CONVERT those cases. It is to add a contract twin beside
each, asserted against the primitive-built original rather than only against the
source: `one_lincoln_street` and `one_lincoln_street_contract` are the first
pair. A contract validated solely against its own source is the pack marking its
own homework.

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

### 7.12 A pool's amortisation state is not exposed — RESOLVED

**Not resolved.** An earlier plan recorded this as closed by the fold layer; it
was not, and the correction is worth keeping. The fold layer shipped for CRE
only, and `benchmarks/credit/auto_abs_speed_050` still asserts one column,
`net_cash_flow`, with the percent-outstanding reconciliation still in prose.

**What now exists.** `packs/credit/statements.toml` publishes
`domain.credit.principal_collections` per period — scheduled principal and
prepayments folded together. That is the input this item was missing, and
`tools/benchmark-runner.py` accepts a verbatim series key, so it is assertable.

**Resolved with a `cumulative` op.** Percent-outstanding is a STOCK derived from
a FLOW, which every other subtotal op structurally cannot express — they all
answer "what happened this period" and this asks "how much so far".

`packs/credit/statements.toml` now publishes the chain: principal paid to date,
the original balance from the pool purchase, the balance still outstanding, and
`domain.credit.pool_factor` as the ratio of the last two. Verified on
`benchmarks/credit/level_pay_pool`, which amortises 0.9871, 0.9742, 0.9615 down
to 0.0716 against a 24,750,000 pool.

Subtraction is expressed as a sum of a negated cumulative rather than as a new
op — adding a negative is the same arithmetic with one fewer concept in the
language.

A model with no purchase stream reports a null pool factor rather than a
misleading 1.0: `benchmarks/credit/auto_abs_speed_050` models the pool without
its acquisition, so the original balance is genuinely unknown there.

#### Original entry: 7.12 A pool's amortisation state is not exposed

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

### 7.14 HUD's mortgage is P+I+MIP, and MIP is not debt service — RESOLVED

**Resolved for the reporting half.** `benchmarks/cre/hud_home_multifamily` now
carries the two legs as separate lines, both grounded in the workbook's own
First Mortgage Sizing tab: mortgage insurance is the stated 0.450% of original
principal on the stated $150,000 loan (675.00 exactly, flat), and debt service
is the residual of the published "Calculated Monthly P+I+MIP Payment" of
1,165.7819. That reconstructs the 13,314.3827 this item recorded, and both
figures are now asserted rather than reconciled in prose.

`domain.cre.debt_service` carries `loan.mortgage_insurance` because coverage
here is measured against the whole published line — which is what this item
said, and what the DSCR the workbook publishes is computed from.

No expectation moved. The two legs sum to the pro forma's own 13,989, because
that cell is `=ROUND(...,0)` and so is computed rather than merely displayed;
the published DSCR is that rounded line divided into a rounded NOI. The model
applies the same round via `round_to` rather than restating the result, so the
0.38 the workbook discards sits on the P&I leg — the leg it rounded.

Still open: converting the case onto `cre.permanent_debt`. That contract
computes P&I from principal and rate, and reaching HUD's payment needs a
monthly schedule on an annual grid — which `E2108` forbids and which the grain
rule in `docs/01_language_spec.md` deliberately keeps forbidden. A pack
`cre.mortgage_insurance` contract is the remaining shape, and it is not added
here because nothing would use it yet (see 7.15 on shipping contracts with no
external user).

#### Original entry: 7.14 HUD's mortgage is P+I+MIP, and MIP is not debt service

Found converting the CRE benchmarks onto `cre.permanent_debt`.
`benchmarks/cre/office_two_tenant` converted cleanly.
`benchmarks/cre/hud_home_multifamily` did not.

**Corrected.** This item originally gave two reasons, and the first was wrong.
It claimed the payments could not be modelled because they are MONTHLY on an
ANNUAL calendar. Measured with `E2108` bypassed, `cre.permanent_debt` at
`payment_frequency = "month"` on HUD's annual model returns **13,314.3827** —
the workbook's published annual P&I, to the cent. The engine sums the twelve
monthly accruals into the year; it was the CHECK that blocked it, not the
arithmetic. That gap is now its own item, 7.16.

What actually remains is the second reason.

**The published line is not debt service.** It is P+I+MIP — the workbook says so
where it defines coverage. The residual is exact:

```
published line     13,989.38
P&I                13,314.38
mortgage insurance    675.00   = 0.450% of the original principal, flat
```

Mortgage insurance is not a payment on the debt, and `cre.permanent_debt` does
not invent one. Modelling it would mean either a `mip_rate` term on a debt
contract that has no business carrying it, or fitting principal and rate
backwards until the total landed on 13,989.38 — the number without the loan.

An FHA/HUD-insured multifamily loan is common enough that a
`cre.mortgage_insurance` contract, or an insurance line separate from the debt,
would be the honest shape. It affects `domain.cre.dscr`, since coverage there is
measured against P+I+MIP.

**Note the interaction with 7.16.** Even once occurrences are distinguishable,
this case would need care: the contract's balloon fires on
`{{time.periods_to_term_end}} == 0`, which is true for *every* occurrence in the
final period — twelve times on an annual model. HUD needs no balloon, so its
conversion is safe, but the contract is not generally safe at a sub-calendar
cadence.

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

### 7.16 Occurrences inside one model period cannot be told apart — ANSWERED

**Answered by the grain rule, not built.** `docs/01_language_spec.md` now states
it beside the `E2108` definition: model at the finest grain at which anything
varies; report at any coarser grain by folding. The case this item describes
becomes unconstructable rather than merely diagnosed, and `E2108` is the
enforcement.

`docs/15_streams_and_the_grid.md`, which proposed the opposite trade — retire
`E2108`, add a sub-period occurrence layer — is retired as **rejected**, with
the reasoning recorded in the document.

#### Original entry: 7.16 Occurrences inside one model period cannot be told apart

The real limitation behind `E2108_SCHEDULE_FINER_THAN_CALENDAR`, measured after
the check's own message turned out to be wrong.

**What the message said.** "Several payments would fall in one period and
collapse into one." That is false for the current engine, and
`docs/01_language_spec.md` always had it right — *"cannot be distinguished once
they land in the same bucket"*. The message and two doc pages had drifted; all
three are now corrected.

**What actually happens.** Measured on an annual model:

```
100 per month   -> [1200, 1200, 100]   twelve accruals SUMMED per year
time.t per month -> [   0,   12,   2]   twelve x time.t, not a sum over months
```

Three different things are being conflated, and only the third is a defect:

1. **Many contributions per period — supported.** `schedule_accruals` returns
   `Vec<Vec<usize>>`: a payment period holds many accruals and
   `values[pay_idx] += amount` accumulates them. This is already load-bearing —
   under net-30, February and March both settle in March.
2. **Many *reported* values per period — impossible, and correctly so.** A
   series is one number per model period (`vec![0.0; timeline.len()]`). The
   period is the reporting grain by definition; wanting twelve separately
   reported payments means wanting twelve periods.
3. **Many *distinct* values per period — not possible today.** This is the gap.

**Why.** The accrual list stores a model **period index**, not an occurrence.
At `crates/cfdl-engine/src/lib.rs:2133` the occurrence's date becomes
`period_index(timeline, start)` and both the date and the loop ordinal `k` are
discarded; `out[settled].push(accrual_idx)` pushes an integer. So twelve
monthly occurrences inside one annual period become twelve copies of the same
integer, and `build_expr_env(ir, …, idx, &timeline[idx], …)` builds twelve
identical environments. A constant amount is therefore exact, and anything
varying with `time.*` or `{{time.elapsed_periods}}` is computed once and
multiplied.

It has never been fixed because it has never been reachable: `E2108` enforces
schedule granularity at or coarser than the calendar, which makes the case
impossible to construct.

**Shape.** Carry the occurrence rather than its period — `Vec<Vec<Accrual>>`
with `{ period_idx, date, ordinal }` — and build the environment from the
occurrence's own date and ordinal. Reporting stays one summed value per period,
which is right. `apply_schedule_indices` already computes both fields and throws
them away, so the change is bounded.

Two things that must move with it, or the fix is worse than the gap:

- **`{{time.elapsed_periods}}` must count occurrences, not model periods**, or
  an amortisation schedule would still read the same month twelve times.
- **Last-period tests break.** `{{time.periods_to_term_end}} == 0` is the
  balloon idiom in `cre.permanent_debt` and `opco.term_debt`, and it is true for
  every occurrence in the final period. It needs to become a last-*occurrence*
  test.

Worth against: HUD's mortgage (7.14), and any instrument whose payment rhythm is
finer than the book it is carried on — which is most lending on a quarterly or
annual reporting grid.

### 7.17 Reporting is a language capability, and it is missing — LARGELY RESOLVED

**Built.** Classification, per-period subtotals, ratios, statements, reporting
grain and the completeness check all ship, in all four packs. `docs/07` §6.10 is
the authoring reference; §6.10 is superseded and says why.

What this item asked for, and where it landed:

- **counts of line items per period** — a statement row per period, at any
  declared grain, with drill-down to the contributing streams
- **classification** — `category` on the emitting lowering rule, roots closed by
  the language to `operating` / `investing` / `financing`
- **subtotals and ratios** — `[[subtotals]]`, per period, published as
  `domain.<pack>.<name>`
- **presentation** — `[[statements]]` with order, depth, labels and a display
  sign that never changes the arithmetic

**The half that remains is the one this item's title is about.** Reporting is a
LANGUAGE capability in its design — the category roots are the language's, and a
pack-less model can now classify its streams — but the DECLARATIONS still live
only in pack TOML. A model with no pack cannot declare a subtotal or a statement
of its own. That needs a surface in the language, and the syntax is undecided;
`docs/16` records the question.

#### Original entry: 7.17 Reporting is a language capability, and it is missing

*Belongs with the language and engine (section 5), and applies to every pack.*

A stated aim is parity with the tools practitioners already use — Argus in CRE,
and its equivalents elsewhere. Those tools are not valued for their discounting;
they are valued for the **statement** they produce: every line item, per period,
grouped, subtotalled, and traceable to the transactions behind it.

**Reporting is distinct from the DCF engine and should be built as such.** NPV
and IRR consume a netted cash flow; a statement consumes the line items and their
structure. Conflating them is why the capability has never been built — every
reporting need to date has been filed as a metric, and metrics answer a different
question.

#### What exists

Per-stream, per-period values. `benchmarks/cre/office_two_tenant` publishes
thirteen independent series — base rent per tenant, recoveries per tenant, opex,
vacancy, TI/LC, debt — each a full array over the grid, aggregating to
`model.net_cash_flow`. The line items are all there.

#### What is missing

1. **Per-period subtotals.** `domain.cre.noi` is `4,718,933.90` — one number for
   a ten-year hold, not a series. A statement's middle rows (EGI, NOI,
   before-tax cash flow) do not exist at any period. This is 1.4, which reports
   it from the coverage-ratio angle: `hud_home_multifamily` reproduces four
   published DSCR values by hand and can assert none of them.
2. **Statement structure.** `metrics.toml` declares flat named scalars. Nothing
   declares order, grouping, hierarchy, labels, or which lines roll into which
   subtotal. An Argus report is exactly that declaration.
3. **Drill-down.** A period total cannot be traced to the payments behind it,
   because the occurrences are never materialised. See
   `docs/15_streams_and_the_grid.md`.
4. **Reporting grain.** The grid, plus an annual rollup, and nothing else. A
   monthly model cannot publish a quarterly statement. Note the annual rollup
   does not close (1) either — it carries the line items and the bottom line and
   no intermediate subtotals, so it has the top and bottom of a statement and
   none of the middle.
5. **Counts.** Metric ops are `sum`, `negated_sum`, `ratio`, `wal_years`.
   Nothing counts occurrences, so "how many payments fell in this period" is
   unanswerable.

#### Existing items that are really reporting items

Filed separately, each from a case that hit it:

| item | why it is reporting |
|---|---|
| **1.3** abatements as a first-class NOI line | *"you can report it as a line OR have it counted in NOI, not both"* — a line's presentation and its arithmetic role are the same thing today |
| **1.4** coverage ratios are lifetime aggregates | per-period subtotals |
| **7.12** a pool's amortisation state is not exposed | percent-outstanding is a derived per-period series; the harness can only check streams and scalars |

#### Per-pack standards

This is not one report. CRE wants an Argus-shaped operating statement; credit
wants a distribution/waterfall report and factor history; opco wants a P&L and a
cash flow statement in the accounting sense; energy wants generation and revenue
by source. The structure belongs in the pack, beside `metrics.toml`, and the
*mechanism* belongs in the language.

#### Sequencing

Independent of the ledger and of each other:

- Per-period metrics (1) can land on their own and immediately let
  `hud_home_multifamily` assert four published ratios — an external validation
  win from a self-contained change.
- Statement structure (2) needs (1) underneath it.
- Drill-down (3) and counts (5) need the ledger.
- Grain (4) is additive once flows are dated.

### 7.18 Monte Carlo carries no transition log

*Belongs with language and engine (section 5).*

`deterministic.transitions` publishes the entity-state trajectory — period,
entity, field, from, to, and the firing event — which made transitions
assertable for the first time. Monte Carlo publishes none.

**A per-trial log is the wrong shape.** It would be trials x transitions of
output, and nobody reads ten thousand copies of the same sequence. The question
a stochastic run actually asks is *when* something happens, not whether it
happened in trial 4,127:

- the distribution over the period each event first fired, and
- the share of trials in which it fired at all.

Both are summaries over the per-trial logs the engine already builds and throws
away, so the work is in choosing the summary rather than in collecting the data.

Found while building the deterministic log (task: transition log). Recorded
rather than built because the useful artifact is a distribution, which is a
different feature from a trail — not a smaller version of one.

Two details already settled by the deterministic side and worth carrying over:
a transition is recorded even when the value does not change, because the
question is whether the event fired; and visibility is two rules, not one — an
event or option guard reads the state as the period opened, a stream reads it as
the period closed.

### 7.19 The lexer reserves words the canonical grammar admits

*Belongs with language and packs (section 5).*

The formal grammar allows any identifier where a name is expected. The lexer
does not, and the difference is 95 words.

```ebnf
entity_stmt     = "entity" IDENT IDENT ":" qname entity_block ;
entity_block    = "{" { kv_stmt } "}" ;
kv_stmt         = IDENT literal_or_expr ;
entity_field    = IDENT ( "=" literal | "init" expr [ "next" expr ] ) ;
```

`IDENT` is `[A-Za-z_][A-Za-z0-9_]*` (`docs/02` §1). No production excludes a
reserved word, and the grammar file states the case outright a few lines below
`entity_field`: **"`use = "office"` states a fact and holds."** That is the
canonical example of a field, and it does not compile.

```
entity asset tower : Asset.Real { use = "office" }
  ERROR[E0004_EXPECTED_TOKEN] Expected a field, 'part of', 'state' or '}' in entity block.

entity asset tower               { use = "office" }        (untyped)
  ERROR[E0004_EXPECTED_TOKEN] Expected token 'pack', found <punctuation>.
```

The second message is the mechanism showing through: `use` becomes a keyword
token, the parser abandons the block and starts looking for a `use pack`
statement, and reports a construct the author never wrote.

**Where it happens.** `crates/cfdl-lexer/src/lib.rs` converts a bare word to
`TokenKind::Keyword` before any position is known, so the exclusion applies
everywhere an identifier is expected. A word whose only grammatical role is
inside one clause blocks a name at the top of a file: `net` exists only for
`payment net 30`, and it is refused as an entity name, a field name and an
assumption name.

The one survival is accidental. `is_qname` is tested BEFORE the keyword table,
so a dotted name lexes as a single token and never meets it. `stream a.net`
compiles and `net = 1` does not — the same word, legal after a dot, illegal
alone. That is not a rule anyone chose; it falls out of the order of two
branches.

**Three measurements.**

*The set is larger than the specification says.* §18 documents 57 reserved
words; the lexer reserves 95. The 38 undocumented ones include `year`, `month`,
`net`, `state`, `active`, `in`, `none`, `mid`, `due` and `clip` — ordinary
words a financial model wants for a field.

*Fourteen are consumed by no production at all* — the seven weekday names, the
four stub policies, `direction`, `owner` and `tags`. They appear in the parser
only in `keyword_text`, which renders a keyword back to text for error
messages. Eleven belong to features `docs/10` records as REJECTED. `owner` and
`direction` are vestigial: direction is written as the bare words `inflow` and
`outflow`, and `owner` appears in no rule.

*Nothing enforces it where the names originate.* A pack's type registry
declares CFDL identifiers through a TOML door that never lexes them, so
`Credit.Asset.Loan.term` and `CRE.Asset.Unit.use` load clean and no model can
spell either. Neither is referenced by any rule, which is why the packs work.

**A second route exists and is worse than none.** An event may write such a
field — `set entity asset.tower.use = "retail"` compiles and publishes the
transition — but reading it fails `E1131_UNKNOWN_FIELD_READ`, because a read
requires a declaration and declaring is what the lexer blocks. The field is
write-only: it reaches the transition log, produces no series, and no
expression can reach it. `check_field_paths` builds its set of known names from
declared fields plus `status` and `state`, and never inspects an event's `set`
targets.

**Shape.** Accept a keyword token where the grammar admits `IDENT`, in the
naming positions: `kv_stmt`/`entity_field`, `assume_stmt`, `entity_stmt`,
`phase_stmt`, `curve_stmt`, `set_entity_stmt`. One token of lookahead
disambiguates the two clauses that share the entity block, because a field is
always followed by `=` or `init` while `state <name>` and `part of <ref>` are
not. Implemented that way the change is ADDITIVE: every spelling it accepts is
a compile error today, so no model that compiles now can change meaning, and
the goldens should be byte-identical. Implemented without the lookahead it
breaks `state` and `part of`, which is the one way to get it wrong.

Out of scope here: bare keywords in EXPRESSION position, which the expression
grammar governs separately; and the two cleanups this audit turned up — the 14
words reserved for nothing, and §18's 38 missing entries.

**The canonical grammar is behind the implementation in the other direction
too.** `entity_block = "{" { kv_stmt } "}"` omits `part of` and the `state`
clause, both of which the parser supports and the grammar's own comment
describes. Whichever way this is settled, the EBNF needs to state what an
entity block actually contains.

Found building `benchmarks/credit/mbs_pool_by_loan`, the first case to declare
typed fields on loan-level assets, and rewritten August 2026 after reading the
canonical grammar. The original item read this as a pack problem with two
possible repairs — widen the grammar, or refuse the collision at pack load. It
is neither: the grammar already admits the name, and refusing it at pack load
would constrain a pack's vocabulary by the core's keyword set, which is the
opposite of what `docs/07` §6.2 says a pack is for.
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
| ~~`cre.construction_loan`~~ | **SHIPPED.** Equity-first funding behind a commitment, the facility taking the balance once it depletes, interest on the drawn balance. The draw schedule stays a model `curve` and the contract names it, because a funding profile is per-deal data rather than a term. `benchmarks/cre/one_lincoln_street_contract` reproduces the primitive-built case in all 48 cells with zero difference — the pair is the assertion, and if they disagree the contract is wrong. Capitalised interest is a follow-on: affine in the closing balance, so it collects rather than needing a solver. |
| `cre.restricted_rent` | HUD — rent capped for an affordability period and reverting to a market track. The defining mechanic of affordable housing, currently a hand-written conditional. |
| `cre.abatement` | MIT — free rent as its own deduction from potential gross revenue. Today it can be reported as a line or counted in NOI, not both (1.3). |
| `cre.replacement_reserve` | HUD — a capital reserve, separately published and semantically distinct from operating expense. Also One Lincoln Street, whose operating pro forma carries a Capital Reserve line. |

With 1.5, 1.6 and 1.7, these are what would let a real CRE deal be expressed in
pack contracts instead of native streams — which is the actual fix for 7.3 on
the CRE side, and needs no new source.

**A correction to how 7.3 framed this.** That entry treats a benchmark running
on native streams as a coverage failure. It is not, or not only. A case built
from primitives proves the LANGUAGE expresses the deal with no domain vocabulary
— which is the stronger claim, and the one a reader evaluating CFDL as a
language can check. A pack contract is an ergonomics layer for a practitioner
who should not have to derive an equity-first waterfall from scratch.

So the fix is not to CONVERT those cases. It is to add a contract twin beside
each, asserted against the primitive-built original rather than only against the
source: `one_lincoln_street` and `one_lincoln_street_contract` are the first
pair. A contract validated solely against its own source is the pack marking its
own homework.

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

### 7.12 A pool's amortisation state is not exposed — RESOLVED

**Not resolved.** An earlier plan recorded this as closed by the fold layer; it
was not, and the correction is worth keeping. The fold layer shipped for CRE
only, and `benchmarks/credit/auto_abs_speed_050` still asserts one column,
`net_cash_flow`, with the percent-outstanding reconciliation still in prose.

**What now exists.** `packs/credit/statements.toml` publishes
`domain.credit.principal_collections` per period — scheduled principal and
prepayments folded together. That is the input this item was missing, and
`tools/benchmark-runner.py` accepts a verbatim series key, so it is assertable.

**Resolved with a `cumulative` op.** Percent-outstanding is a STOCK derived from
a FLOW, which every other subtotal op structurally cannot express — they all
answer "what happened this period" and this asks "how much so far".

`packs/credit/statements.toml` now publishes the chain: principal paid to date,
the original balance from the pool purchase, the balance still outstanding, and
`domain.credit.pool_factor` as the ratio of the last two. Verified on
`benchmarks/credit/level_pay_pool`, which amortises 0.9871, 0.9742, 0.9615 down
to 0.0716 against a 24,750,000 pool.

Subtraction is expressed as a sum of a negated cumulative rather than as a new
op — adding a negative is the same arithmetic with one fewer concept in the
language.

A model with no purchase stream reports a null pool factor rather than a
misleading 1.0: `benchmarks/credit/auto_abs_speed_050` models the pool without
its acquisition, so the original balance is genuinely unknown there.

#### Original entry: 7.12 A pool's amortisation state is not exposed

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

### 7.14 HUD's mortgage is P+I+MIP, and MIP is not debt service — RESOLVED

**Resolved for the reporting half.** `benchmarks/cre/hud_home_multifamily` now
carries the two legs as separate lines, both grounded in the workbook's own
First Mortgage Sizing tab: mortgage insurance is the stated 0.450% of original
principal on the stated $150,000 loan (675.00 exactly, flat), and debt service
is the residual of the published "Calculated Monthly P+I+MIP Payment" of
1,165.7819. That reconstructs the 13,314.3827 this item recorded, and both
figures are now asserted rather than reconciled in prose.

`domain.cre.debt_service` carries `loan.mortgage_insurance` because coverage
here is measured against the whole published line — which is what this item
said, and what the DSCR the workbook publishes is computed from.

No expectation moved. The two legs sum to the pro forma's own 13,989, because
that cell is `=ROUND(...,0)` and so is computed rather than merely displayed;
the published DSCR is that rounded line divided into a rounded NOI. The model
applies the same round via `round_to` rather than restating the result, so the
0.38 the workbook discards sits on the P&I leg — the leg it rounded.

Still open: converting the case onto `cre.permanent_debt`. That contract
computes P&I from principal and rate, and reaching HUD's payment needs a
monthly schedule on an annual grid — which `E2108` forbids and which the grain
rule in `docs/01_language_spec.md` deliberately keeps forbidden. A pack
`cre.mortgage_insurance` contract is the remaining shape, and it is not added
here because nothing would use it yet (see 7.15 on shipping contracts with no
external user).

#### Original entry: 7.14 HUD's mortgage is P+I+MIP, and MIP is not debt service

Found converting the CRE benchmarks onto `cre.permanent_debt`.
`benchmarks/cre/office_two_tenant` converted cleanly.
`benchmarks/cre/hud_home_multifamily` did not.

**Corrected.** This item originally gave two reasons, and the first was wrong.
It claimed the payments could not be modelled because they are MONTHLY on an
ANNUAL calendar. Measured with `E2108` bypassed, `cre.permanent_debt` at
`payment_frequency = "month"` on HUD's annual model returns **13,314.3827** —
the workbook's published annual P&I, to the cent. The engine sums the twelve
monthly accruals into the year; it was the CHECK that blocked it, not the
arithmetic. That gap is now its own item, 7.16.

What actually remains is the second reason.

**The published line is not debt service.** It is P+I+MIP — the workbook says so
where it defines coverage. The residual is exact:

```
published line     13,989.38
P&I                13,314.38
mortgage insurance    675.00   = 0.450% of the original principal, flat
```

Mortgage insurance is not a payment on the debt, and `cre.permanent_debt` does
not invent one. Modelling it would mean either a `mip_rate` term on a debt
contract that has no business carrying it, or fitting principal and rate
backwards until the total landed on 13,989.38 — the number without the loan.

An FHA/HUD-insured multifamily loan is common enough that a
`cre.mortgage_insurance` contract, or an insurance line separate from the debt,
would be the honest shape. It affects `domain.cre.dscr`, since coverage there is
measured against P+I+MIP.

**Note the interaction with 7.16.** Even once occurrences are distinguishable,
this case would need care: the contract's balloon fires on
`{{time.periods_to_term_end}} == 0`, which is true for *every* occurrence in the
final period — twelve times on an annual model. HUD needs no balloon, so its
conversion is safe, but the contract is not generally safe at a sub-calendar
cadence.

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

### 7.16 Occurrences inside one model period cannot be told apart — ANSWERED

**Answered by the grain rule, not built.** `docs/01_language_spec.md` now states
it beside the `E2108` definition: model at the finest grain at which anything
varies; report at any coarser grain by folding. The case this item describes
becomes unconstructable rather than merely diagnosed, and `E2108` is the
enforcement.

`docs/15_streams_and_the_grid.md`, which proposed the opposite trade — retire
`E2108`, add a sub-period occurrence layer — is retired as **rejected**, with
the reasoning recorded in the document.

#### Original entry: 7.16 Occurrences inside one model period cannot be told apart

The real limitation behind `E2108_SCHEDULE_FINER_THAN_CALENDAR`, measured after
the check's own message turned out to be wrong.

**What the message said.** "Several payments would fall in one period and
collapse into one." That is false for the current engine, and
`docs/01_language_spec.md` always had it right — *"cannot be distinguished once
they land in the same bucket"*. The message and two doc pages had drifted; all
three are now corrected.

**What actually happens.** Measured on an annual model:

```
100 per month   -> [1200, 1200, 100]   twelve accruals SUMMED per year
time.t per month -> [   0,   12,   2]   twelve x time.t, not a sum over months
```

Three different things are being conflated, and only the third is a defect:

1. **Many contributions per period — supported.** `schedule_accruals` returns
   `Vec<Vec<usize>>`: a payment period holds many accruals and
   `values[pay_idx] += amount` accumulates them. This is already load-bearing —
   under net-30, February and March both settle in March.
2. **Many *reported* values per period — impossible, and correctly so.** A
   series is one number per model period (`vec![0.0; timeline.len()]`). The
   period is the reporting grain by definition; wanting twelve separately
   reported payments means wanting twelve periods.
3. **Many *distinct* values per period — not possible today.** This is the gap.

**Why.** The accrual list stores a model **period index**, not an occurrence.
At `crates/cfdl-engine/src/lib.rs:2133` the occurrence's date becomes
`period_index(timeline, start)` and both the date and the loop ordinal `k` are
discarded; `out[settled].push(accrual_idx)` pushes an integer. So twelve
monthly occurrences inside one annual period become twelve copies of the same
integer, and `build_expr_env(ir, …, idx, &timeline[idx], …)` builds twelve
identical environments. A constant amount is therefore exact, and anything
varying with `time.*` or `{{time.elapsed_periods}}` is computed once and
multiplied.

It has never been fixed because it has never been reachable: `E2108` enforces
schedule granularity at or coarser than the calendar, which makes the case
impossible to construct.

**Shape.** Carry the occurrence rather than its period — `Vec<Vec<Accrual>>`
with `{ period_idx, date, ordinal }` — and build the environment from the
occurrence's own date and ordinal. Reporting stays one summed value per period,
which is right. `apply_schedule_indices` already computes both fields and throws
them away, so the change is bounded.

Two things that must move with it, or the fix is worse than the gap:

- **`{{time.elapsed_periods}}` must count occurrences, not model periods**, or
  an amortisation schedule would still read the same month twelve times.
- **Last-period tests break.** `{{time.periods_to_term_end}} == 0` is the
  balloon idiom in `cre.permanent_debt` and `opco.term_debt`, and it is true for
  every occurrence in the final period. It needs to become a last-*occurrence*
  test.

Worth against: HUD's mortgage (7.14), and any instrument whose payment rhythm is
finer than the book it is carried on — which is most lending on a quarterly or
annual reporting grid.

### 7.17 Reporting is a language capability, and it is missing — LARGELY RESOLVED

**Built.** Classification, per-period subtotals, ratios, statements, reporting
grain and the completeness check all ship, in all four packs. `docs/07` §6.10 is
the authoring reference; §6.10 is superseded and says why.

What this item asked for, and where it landed:

- **counts of line items per period** — a statement row per period, at any
  declared grain, with drill-down to the contributing streams
- **classification** — `category` on the emitting lowering rule, roots closed by
  the language to `operating` / `investing` / `financing`
- **subtotals and ratios** — `[[subtotals]]`, per period, published as
  `domain.<pack>.<name>`
- **presentation** — `[[statements]]` with order, depth, labels and a display
  sign that never changes the arithmetic

**The half that remains is the one this item's title is about.** Reporting is a
LANGUAGE capability in its design — the category roots are the language's, and a
pack-less model can now classify its streams — but the DECLARATIONS still live
only in pack TOML. A model with no pack cannot declare a subtotal or a statement
of its own. That needs a surface in the language, and the syntax is undecided;
`docs/16` records the question.

#### Original entry: 7.17 Reporting is a language capability, and it is missing

*Belongs with the language and engine (section 5), and applies to every pack.*

A stated aim is parity with the tools practitioners already use — Argus in CRE,
and its equivalents elsewhere. Those tools are not valued for their discounting;
they are valued for the **statement** they produce: every line item, per period,
grouped, subtotalled, and traceable to the transactions behind it.

**Reporting is distinct from the DCF engine and should be built as such.** NPV
and IRR consume a netted cash flow; a statement consumes the line items and their
structure. Conflating them is why the capability has never been built — every
reporting need to date has been filed as a metric, and metrics answer a different
question.

#### What exists

Per-stream, per-period values. `benchmarks/cre/office_two_tenant` publishes
thirteen independent series — base rent per tenant, recoveries per tenant, opex,
vacancy, TI/LC, debt — each a full array over the grid, aggregating to
`model.net_cash_flow`. The line items are all there.

#### What is missing

1. **Per-period subtotals.** `domain.cre.noi` is `4,718,933.90` — one number for
   a ten-year hold, not a series. A statement's middle rows (EGI, NOI,
   before-tax cash flow) do not exist at any period. This is 1.4, which reports
   it from the coverage-ratio angle: `hud_home_multifamily` reproduces four
   published DSCR values by hand and can assert none of them.
2. **Statement structure.** `metrics.toml` declares flat named scalars. Nothing
   declares order, grouping, hierarchy, labels, or which lines roll into which
   subtotal. An Argus report is exactly that declaration.
3. **Drill-down.** A period total cannot be traced to the payments behind it,
   because the occurrences are never materialised. See
   `docs/15_streams_and_the_grid.md`.
4. **Reporting grain.** The grid, plus an annual rollup, and nothing else. A
   monthly model cannot publish a quarterly statement. Note the annual rollup
   does not close (1) either — it carries the line items and the bottom line and
   no intermediate subtotals, so it has the top and bottom of a statement and
   none of the middle.
5. **Counts.** Metric ops are `sum`, `negated_sum`, `ratio`, `wal_years`.
   Nothing counts occurrences, so "how many payments fell in this period" is
   unanswerable.

#### Existing items that are really reporting items

Filed separately, each from a case that hit it:

| item | why it is reporting |
|---|---|
| **1.3** abatements as a first-class NOI line | *"you can report it as a line OR have it counted in NOI, not both"* — a line's presentation and its arithmetic role are the same thing today |
| **1.4** coverage ratios are lifetime aggregates | per-period subtotals |
| **7.12** a pool's amortisation state is not exposed | percent-outstanding is a derived per-period series; the harness can only check streams and scalars |

#### Per-pack standards

This is not one report. CRE wants an Argus-shaped operating statement; credit
wants a distribution/waterfall report and factor history; opco wants a P&L and a
cash flow statement in the accounting sense; energy wants generation and revenue
by source. The structure belongs in the pack, beside `metrics.toml`, and the
*mechanism* belongs in the language.

#### Sequencing

Independent of the ledger and of each other:

- Per-period metrics (1) can land on their own and immediately let
  `hud_home_multifamily` assert four published ratios — an external validation
  win from a self-contained change.
- Statement structure (2) needs (1) underneath it.
- Drill-down (3) and counts (5) need the ledger.
- Grain (4) is additive once flows are dated.

### 7.18 Monte Carlo carries no transition log

*Belongs with language and engine (section 5).*

`deterministic.transitions` publishes the entity-state trajectory — period,
entity, field, from, to, and the firing event — which made transitions
assertable for the first time. Monte Carlo publishes none.

**A per-trial log is the wrong shape.** It would be trials x transitions of
output, and nobody reads ten thousand copies of the same sequence. The question
a stochastic run actually asks is *when* something happens, not whether it
happened in trial 4,127:

- the distribution over the period each event first fired, and
- the share of trials in which it fired at all.

Both are summaries over the per-trial logs the engine already builds and throws
away, so the work is in choosing the summary rather than in collecting the data.

Found while building the deterministic log (task: transition log). Recorded
rather than built because the useful artifact is a distribution, which is a
different feature from a trail — not a smaller version of one.

Two details already settled by the deterministic side and worth carrying over:
a transition is recorded even when the value does not change, because the
question is whether the event fired; and visibility is two rules, not one — an
event or option guard reads the state as the period opened, a stream reads it as
the period closed.

### 7.19 An ontology field named after a keyword cannot be written

*Belongs with language and packs (section 5).*

`Credit.Asset.Loan` declares three fields — `original_balance`, `coupon` and
`term`. The third cannot be set by any model:

```
entity asset loan_a : Credit.Asset.Loan {
  term = 360          // ERROR[E0004_EXPECTED_TOKEN]
}
```

`term` is a keyword, so the lexer never offers it as an attribute name and the
entity block will not accept it. The ontology declares a field that is
unreachable from the language, and nothing says so — the pack loads, the type
validates, and the failure appears only when someone writes the attribute.

Two ways to close it, and they are not equivalent:

1. **Accept keyword-shaped identifiers in attribute position.** The parser knows
   it is reading an attribute name there, so the ambiguity is local. Larger
   change, and it makes the grammar's keyword set position-dependent.
2. **Reject the collision at pack load.** `validate_ontology_against_rules`
   already walks every field; adding a check against the keyword set turns this
   into a pack error with a name and a line, caught once by whoever writes the
   pack rather than repeatedly by whoever writes a model. Then rename the field.

The second is smaller and catches the next one at the right moment. It does cost
the pack a name it wanted, which is the argument for the first.

Found building `benchmarks/credit/mbs_pool_by_loan`, the first case to declare
typed attributes on loan-level assets. The case is unaffected — the term its
schedule uses is the contract's `term_months` — so it states the balance and
coupon and leaves `term` out.

### 7.20 `E1129` never sees a pack-lowered stream — RESOLVED

*Belongs with the language and engine (section 5).*

`check_prev_first_period` walks `resolve_output.source_statements` for
`Stmt::Stream`. A stream a PACK emitted is not a source statement — it exists
only after lowering — so the check runs on hand-written streams and on nothing
else.

The diagnostic it skips is real. A stream reading `prev.<entity>.<field>` in
the model's FIRST period is reading a close that does not exist: `prev_states`
is empty at `t = 0`, so the read warns and substitutes zero for that period
while every later period is correct. One wrong period in an otherwise right
series is the hardest shape to notice, and `status: ok`.

It is bounded — one period, and a warning is emitted — where the defect that
found it was every period. But it is the same class, and it is live for exactly
the contract that found it: a construction facility whose draw begins at model
period 0 and whose interest accrues on the average of opening and closing
balance.

The fix is placement rather than logic. The check needs the lowered streams, so
it belongs after lowering rather than beside the other resolve-stage checks; the
predicate (`reads_prev_field`, plus the schedule-starts-at-model-start test) is
unchanged, and already recognises the `prev.entity.` spelling a lowering rule
produces. What needs deciding is the message: a model author cannot "start the
stream one period later" when the pack owns the schedule, so a lowered stream
wants a wording that names the CONTRACT and its term instead.

Found fixing `prev.field.<name>` (`fixtures/valid/pack_rule_reads_prev_field`),
which is the accessor whose absence hid this: no shipped rule read `prev`, so
no lowered stream could have tripped the check even if it had run.

**Resolved.** `check_lowered_prev_first_period` runs on `lowered.streams`
immediately after lowering, with the same predicate and the same code. The
message names the contract, read from the stream-inputs provenance:

```
ERROR[E1129_PREV_IN_FIRST_PERIOD] Stream 'testpack.avg_balance_interest',
lowered from contract 'test.avg_balance_contract', reads a field's previous
period but runs from the model's first period, where there is none. Start the
contract's term one period after the model, or have the rule carry the opening
value as a field of its own.
```

`fixtures/invalid/pack_rule_prev_first_period` is the same contract as the
valid fixture with its term moved onto period 0, so the pair reads as one
statement: this term start compiles, that one does not. No existing golden
moved.

### 7.21 Two build-freshness guards hash packs into a binary that does not contain them

*Belongs with the tooling.*

`tools/py-stamp.py` hashes `packs/` into the extension's build stamp, and its
docstring states why: *"crates/cfdl-pack `include_str!`s every pack TOML at
compile time — editing a lowering rule changes the extension with no Rust source
change at all."* `python/tests/test_native_is_fresh.py` globs
`packs/**/*.toml` under a helper documented as *"most recently modified file
that is compiled into the native module."*

Both premises are false for the Python extension. The `include_str!` block is
`#[cfg(feature = "embedded-packs")]`, and `crates/cfdl-py/Cargo.toml` takes
`cfdl-pack` **without** that feature — only `cfdl-cli`, `cfdl-server` and
`cfdl-wasm` enable it. The SDK says so itself when asked to compile without a
pack directory:

```
E4004_MISSING_PACK: No pack directory was provided and this build has no
embedded packs.
```

The SDK reads packs from disk at run time, via `packs_dir`. A pack edit changes
what it produces without changing the binary at all.

The cost is a false alarm on one of the actions this repository takes most
often. Editing a lowering rule fails `py-check`, which demands `make
py-develop`; the rebuild is a no-op because cargo correctly sees no dirty input,
so the `.so` keeps its old mtime and `test_native_is_fresh` then fails anyway.
The way out is to touch a crate source to force a rebuild that changes nothing.
`check-notebooks-fresh` already reasons correctly about exactly this trade —
`benchmarks/` is deliberately not a directory-wide input there, for the same
reason.

**The notebook stamp's `packs/` entry is correct and should stay.** Pack data
changes what a notebook prints, because the SDK reads it at run time. That guard
is about rendered output; these two are about a compiled binary, and only the
first has packs in it.

**A second defect the first one hid.** `make py-develop` runs `pip install -e`
and then `py-stamp --write` unconditionally, so the stamp records the sources as
built whether or not a rebuild happened. Here that was harmless — the extension
did not need rebuilding — but it means a genuinely skipped rebuild would be
certified fresh, and the only reason it was caught at all is that the mtime test
disagreed with the hash test. Two guards on one property, reaching opposite
answers, with the wrong one silent.

Shape: drop `packs` from `STAMP_INPUTS` and from the test's globs, and derive
the stamp from the artefact rather than writing it on faith after an install.
Both guards should then stay quiet through a pack edit and speak on an engine
one.

Found shipping `cre.construction_loan`, which is a pack-only change and tripped
both guards twice.

### 7.22 A published weighted average life cannot be asserted

Belongs with section 4 (credit pack).

`domain.credit.wal_years` folds the pool's own streams —
`credit.pool.sched_principal.*`, `prepay.*`, `bullet.*`, `recoveries.*` — so it
answers "when does the collateral come back". A structured deal publishes the
question one level up: when does *each class* come back. There is no metric for
that, and a waterfall step's stream cannot be reached by one, because
`metrics.toml` names streams by pattern and a WAL needs the class's original
balance as well as its payments.

So a published weighted average life cannot be checked. Ginnie Mae REMIC Trust
2026-100 publishes 709 of them, one per class per prepayment speed; Fannie Mae
REMIC Trust 2019-2 publishes seven. In both cases the model reproduces them —
709 of 709 exactly for the first, all seven for the second — and in both cases
the only place to say so was `CASE.md`, in prose.

This is §7.12 one level up. That item was about a pool's amortisation state not
being exposed, which left `auto_abs_speed_050` reconciling its percent-outstanding
column in words; the fix was a cumulative subtotal, and the case now asserts it.
The same argument applies here: a figure the issuer publishes, that the model
gets right, that no gate would notice going wrong.

Shape: a per-class WAL wants two inputs the pack does not currently pair — a
payment stream and the original balance of the thing being paid. The class
already carries the second as `original_balance` on a `Credit.Asset.Tranche`, so
the metric is plausibly a fold over a stream *keyed to an entity*, rather than
over a stream pattern alone. That is a wider change than a new metric row, which
is why this is a backlog item and not a patch.

Found modelling Ginnie Mae 2026-100 and Fannie Mae 2019-2, where between them
716 published figures could be reproduced and none could be asserted.

### 7.23 A scenario asserts metrics, but not the per-period column that is the published artefact

Belongs with section 5 (harness and tooling).

`expected_scenarios.json` checks a scenario's **metrics**. `expected.csv` checks
per-period series, but only for the deterministic run. So a case can vary an
input across scenarios and assert what that does to a summary number, and cannot
assert what it does to a schedule.

For structured credit that is backwards. The published artefact *is* the
per-period column, tabulated at several prepayment speeds — five to seven of
them per class — and the summary number is the derived thing. Ginnie Mae
2026-100 publishes 58 such tables; the model reconciles every one, and a case can
assert one speed.

The existing route is one case directory per speed, as `auto_abs_speed_050` and
`auto_abs_speed_150` do. That works for two. It does not work for roughly 75
directories differing in a single term, and the duplication is not free: each
carries its own `CASE.md`, `SOURCE.md` and tolerances to keep in step, and the
site publishes each as a separate page.

Shape: let `expected.csv` carry a scenario column, or let a scenario name a CSV
of its own. Either makes the speed grid one case with N columns instead of N
cases with one, and neither changes what the engine computes — the scenario runs
already happen and their series are already produced, they are simply not
reachable from the harness.

Found modelling Ginnie Mae 2026-100, whose decrement tables publish 21,570 cells
across five to seven speeds per class, of which a single case can assert the
0%-, 100%- or 259%-PSA column but not all three.

### 7.24 A validated contract term cannot be parameterised — WRONG, see the correction

Belongs with section 5 (language and engine).

A contract term bound to a run-config parameter fails at compile time:
`psa_speed = cfg.psa` is `E9016_CREDIT_INVALID_PSA_SPEED`, because pack
validations evaluate terms when the model compiles and a parameter has no
value yet. So every input a pack validates — speed, rate, term, severity —
is a literal, and everything downstream of that inherits the restriction:

- a scenario cannot vary a pack deal's prepayment assumption, which is why
  the FNMA 2019-2 decrement table ships as seven case directories differing
  in one number (§7.23 is the harness half of that story; this is the
  compiler half);
- a Monte Carlo distribution cannot attach to one. Distributions reach
  `cfg.*` and `stream.<name>:amount`, and each trial is a full rerun — the
  machinery works — but the inputs a structured-credit practitioner most
  wants to simulate (CPR, CDR, severity, speed) are contract terms, and no
  distribution can touch them.

Shape: when a term is an expression over `cfg.*`, defer that term's pack
validation to run start and validate the RESOLVED value — per run, per
scenario, per trial — failing the run with the same diagnostic the compiler
would have raised. Terms that are literals keep compile-time validation;
nothing changes for existing models.

Found modelling FNMA 2019-2 at seven speeds, where the attempt to carry the
speed as `cfg.psa` was the first thing tried and the first thing refused.

#### Correction — `inputs.<name>` was never refused

Appended rather than edited in, because what was believed and what was
measured are both worth seeing.

The restriction is `cfg.*`, not parameterisation. A term deferred to
`inputs.<name>` compiles, lowers carrying the reference, and responds to a
run-config override — on the very term this entry names. Probed against the
credit pack, `psa_speed = inputs.psa`:

| run | prepayments |
|---|---:|
| deterministic, 198% PSA | 25,572.80 |
| scenario `psa000` | 0.00 |
| scenario `psa700` | 91,348.34 |

`E9016_CREDIT_INVALID_PSA_SPEED` does not fire, and it is not an oversight
that it does not: `docs/01_language_spec.md` §8.2.1 states it as policy — *"a
term whose value is an input reference is not range-checked at compile time,
since its value is not yet known; pack bounds still apply to literal terms."*
The channel was documented and open the whole time.

So both consequences this entry draws are false as written:

- **A scenario CAN vary a pack deal's prepayment assumption.** The three runs
  above are one model and one `run.json`.
- **A Monte Carlo distribution CAN attach to a validated contract term.**
  `assume psa ~ Normal(mean=1.98, stdev=0.4, clip=[0.5, 4.0])` over 200 seeded
  trials moves `model.npv` across 72,663.72 - 97,375.47, mean 86,696.38. The
  distribution reaches the term because a draw writes `inputs.<name>`, which is
  the same channel a scenario writes.

**What remains, and it is smaller.** Two real questions survive:

1. Should `cfg.*` work in a term as well? It is the run-config's other half,
   and a reader who reaches for it gets a diagnostic that says the value is
   invalid rather than that the channel is wrong. If the answer is no, the
   diagnostic should say so — `E9016` naming a bound is actively misleading
   when the term is `cfg.psa`.
2. A term deferred to `inputs.` is **never** bounds-checked, at compile time or
   at run start. §8.2.1 accepts that deliberately, but it means a scenario may
   push `psa_speed` to 40 and the run will price it. The original "Shape" above
   — validate the RESOLVED value per run, per scenario, per trial — is still
   the right fix, and it now applies to `inputs.` rather than to `cfg.`.

**The FNMA seven directories are still justified — by §7.23, not by this.**
The harness asserts metrics per scenario and not the per-period column, and
the decrement table IS a per-period column. That is the whole reason the case
ships seven ways, and this entry claimed the compiler half of a story that
turned out to have only a harness half.

Found while answering an external question about whether pack-validated terms
could carry scenario and Monte Carlo values, which is the same question this
entry answers "no" to.

### 7.25 A model cannot declare a metric

Belongs with section 5 (language and engine).

Metric keys are minted in exactly two places: the engine (`model.*`) and a
pack's `metrics.toml` (`domain.*`). A model that computes a deal-specific
figure — a class weighted average life on the deal's own axis, a crossover
date, an overcollateralisation ratio — has no way to name it. The workaround
is an entity field asserted per-period in `expected.csv`, which works (§7.22's
single-class cases use exactly that route for balances) but leaves the number
a case exists to check sitting unnamed in a CSV column rather than stated in
`expected_metrics.json` next to the published figure it reproduces.

This is the case-side complement of §7.22. That item asks the credit pack for
an entity-keyed WAL; this one asks the language to let a case declare the
number it solved for, whatever the pack thinks. "We solve for the case, not
for the pack" needs somewhere to put the answer.

Shape: a `metric` declaration — an expression over series and fields,
evaluated at the horizon, published into `deterministic.metrics` and into
every scenario summary. Landing in scenario summaries matters: combined with
§7.23 it lets a speed grid assert a derived figure per column, not only the
engine's built-ins.

Found modelling FNMA 2019-2, whose published WALs are asserted through the
pack's pool-level metric only because a single-class pass-through makes the
pool's WAL and the class's the same number. The next deal's classes will not
be so obliging.

### 7.26 Time-weighted metrics measure from the model start, on period fractions; a published WAL measures from settlement to stated payment days

Belongs with section 5 (language and engine).

`docs/12_payment_timing.md` names both limits itself: the axis origin is the
model start, not a settlement date, and precision is period fractions, not
calendar days. A prospectus weighted average life is defined on neither. The
FNMA REMIC prospectus (1 November 2018) computes it as principal reductions
weighted by "the number of years from the settlement date ... to the second
such distribution date", and the 2019-2 supplement's Pricing Assumptions fix
both anchors for its tables: settlement 30 January 2019, "each Distribution
Date occurs on the 25th day of a month".

The distinction is falsifiable, and was falsified. Recomputing the deal's
seven published WALs under four axes, only settlement-to-the-25th (actual/365)
reproduces all seven to the printed tenth. The discriminating column is 400%
PSA: printed 2.9; settlement-to-25th 2.9474; the engine's month-end axis
2.9608, and both month-end-from-settlement variants 2.956-2.962 — every
month-end reading rounds to 3.0. Amounts are unaffected throughout: P&I per
period is scheduled activity and carries no day. Only the time-weighting
moves.

Today the seven REMIC cases carry the gap as tolerance: ±0.07, decomposed in
each `case.toml` as 0.05 print floor plus ~0.015 axis. That is stated rather
than hidden, but it is the wrong long-term shape, because the axis differs by
program — Ginnie I pays the 15th, Ginnie II the 20th, FNMA REMICs the 25th —
while a single widened band is deaf to which one a deal used. A band derived
from one deal's axis can hide a same-sized convention error in the next
deal's, and nothing in the suite would notice: 2019-2 happened to publish a
column (400%) that discriminates, and the next deal may not.

Shape, in two independent pieces:

- a run-config `settlement_date` that becomes the origin for `wal_years` and
  `payback_years`, measured in actual days;
- a declarable payment-day placement — a day rule the pack lowering carries
  into its emitted streams' offsets, the way model-level schedules already
  carry `on day <n>` — so a deal states "distributions on the 25th" once and
  every time-weighted metric lands on the deal's own axis.

With both, the WAL tolerances return to the print floor and the axis stops
being a tolerance line item anywhere.

Found asserting the seven published WALs of FNMA 2019-2, where the 400% PSA
column refused the naive floor and the refusal was the convention speaking.

### 7.27 A pack rule read a curve as a per-period total — RESOLVED, and the first filing was wrong

*Belongs with the CRE pack (section 1).*

**What this entry originally said, and why it was wrong.** It claimed the
language had a gap: that `curve` means "level", that there is no way to state a
per-period flow, and that every schedule is therefore a level pressed into
service. It proposed a `flow` interpolation mode and, failing that, a compile
check.

None of that was needed. A sparse curve carrying an ANNUALISED figure, divided
by the rule's periods-per-year, is correct on every calendar — which is the
convention `rent_year`, `opex_year` and opco's `growth_curve` have followed all
along. Two points, two calendars, one answer:

```cfdl
curve draw_rate_year step { 2026-01: 4000  2026-07: 8000 }
amount = curve_value("draw_rate_year", time.date) / time.ppy
```

| grid | total drawn |
|---|---:|
| quarterly | 6,000.00 |
| monthly | 6,000.00 |

Sparse is not a workaround, it is the intended usage: declare a point where the
value CHANGES and flat-forward holds it in between. That is what a level is for,
and it is why `step` is the default.

**The defect was `cre.construction_loan` reading its curve as a per-period
total.** Under that reading a schedule stated quarterly and run monthly repeats
each quarter's figure three times and funds three times the money, silently:

| model grid | curve points | funded |
|---|---|---:|
| quarterly | quarterly | 4,000.00 |
| monthly | monthly | 4,000.00 |
| monthly | **quarterly** | **12,000.00** |

**Resolved** by dividing every curve read in the contract by
`{{model.periods_per_year}}` and stating the term as an annualised rate. One
sparse point now funds the same amount on a quarterly and a monthly model, and
`benchmarks/cre/one_lincoln_street_contract` still reproduces the
primitive-built case in all 48 cells with zero difference — the curve is stated
x4 and divides straight back.

**The general rule, which the pack interface should carry:** a lowering rule
reading a curve divides by periods-per-year for the same reason it does for
`rent_year`. A curve is a level; a rule that wants a flow annualises it.

Two smaller things this leaves behind, neither worth an entry of its own:

- Nothing rejects a curve read at a grain it was not stated at. With the
  annualised convention there is nothing to reject — any grain is meaningful —
  so the check the first filing proposed has no subject.
- Re-graining now SPREADS: a quarter's funding becomes three equal months. That
  is an assumption, not a fact, and it is the only defensible one when nothing
  finer was stated. Worth knowing before reading a monthly draw report off a
  quarterly schedule.

Found reviewing the contract against an external question about running one deal
at monthly and daily granularity, and corrected after the first diagnosis was
challenged — the language behaved exactly as specified throughout.

### 7.28 The three specifications use RFC 2119 keywords and define none of them

`docs/01_language_spec.md`, `docs/04_compiler_spec.md` and
`docs/07_pack_interface.md` use MUST, SHALL, SHOULD, MAY, REQUIRED, OPTIONAL and
RECOMMENDED **143 times between them** — 67, 45 and 31. Nothing in the
repository cites RFC 2119 or BCP 14.

These documents exist so a second implementation can be written from them. A
second implementer currently cannot tell whether "should" is a requirement or
advice, and the answer differs by sentence.

The fix is one short section per specification, naming BCP 14 and stating that
the keywords carry their RFC 2119 meanings. It is the cheapest correctness
improvement in the documentation estate.

Found by the documentation standards audit — see `docs/21`.

### 7.29 The same words are published in two spellings

Both forms are live in published prose: `amortisation` 21 against
`amortization` 3, `amortising` 19 against `amortizing` 4, `behavior` 25 against
`behaviour` 6, `modeling` 28 against `modelling` 5, `license` 9 against
`licence` 4, `catalog` 1 against `catalogue` 2.

The split does not follow a national convention — `behavior` and `modeling` lean
American while `amortisation` and `amortising` lean British. Nothing recorded a
decision, so the corpus accreted both.

`docs/terminology.toml` now records the decision (US forms). The work is a
find-and-replace across `site/content/docs`, `learn/content/chapters` and
`training/exercises`, plus the generating sources for the pages that are not
authored in place.

### 7.30 One object carries three names, and one instruction uses a verb that is not approved

`run configuration` (57), `run config` (7) and `run settings` (2) name the same
object. `results document` (9) and `output document` (2) name the same artefact.

Separately, `site/content/docs/getting-started.md:31` reads "Hit **Run**." —
`hit` appears 7 times as an instruction verb against `click` once. It is the
most-read procedural line on the site.

Both are settled in `docs/terminology.toml`; this entry is the work of applying
it.

### 7.31 The benchmark example pages publish number formats and passive prose the style guide rules against

The 38 generated example pages draw prose from two places, and the two need
different work:

- **The shared template**, `site/scripts/sync-content.mjs:805`, emits "Every
  number below is checked against an independent reference implementation on
  every commit" to all 38. That sentence is fine — a descriptive passive with an
  irrelevant actor. Worth recording that this template is the one place where a
  single edit reaches 38 pages, for when something there does need changing.
- **The per-case sources**, `benchmarks/*/*/case.toml` and `CASE.md`, carry the
  rest. Three defects live here:
  - "The source cannot be published, so its conventions are recreated
    independently of the model and compared period by period" — doubly passive
    with an unstated agent, on 20 pages.
  - The U+00D7 multiplication sign (`8.0×`) in 4 files. `docs/22` requires
    `8.0x`.
  - `$33.6mm`-style currency in 37 files. `docs/22` requires `$33.6m`; `mm`
    reads as millimetres.

These are per-file edits. Do not estimate them as template work — that was the
first reading, and it was wrong by about 20x.

### 7.32 No site doc page has a description, and there is no glossary

Every one of the 111 pages under `site/content/docs` carries `id` and `title`.
**None carries `description`**, so no page has a meta description for a search
result or a link preview. The `learn` chapters all carry one, so the convention
already exists in the repository and only needs copying.

Separately, there is no glossary anywhere in `site/`, `learn/` or `docs/`. The
product has two overlapping specialist vocabularies — finance and compiler
construction — and the curriculum introduces `grain`, `reversion`, `takeout`,
`promote`, `catch-up` and `lowering` with inline bold definitions a reader
cannot navigate back to.

ISO/IEC/IEEE 26514 and IEC/IEEE 82079-1 both require defined terms for a product
like this. `docs/terminology.toml` holds the definitions and can generate the
page.

### 7.33 Neither site has been assessed against WCAG 2.2 AA

Both are Next.js applications with a custom design system, a theme toggle, an
interactive playground and syntax-highlighted code blocks. Each is a common
source of contrast, focus-order and keyboard-trap failures. **No accessibility
conformance claim should be made until an assessment runs**, and none has.

The reason to schedule it rather than file it and forget it: EN 301 549 is the
European harmonised standard and the European Accessibility Act's obligations
have applied since 28 June 2025. If CFDL is sold into the EU this is a legal
baseline, not a quality goal.

Contrast tokens and focus states are the likely first failures given the theme
toggle. `learn/scripts/check-tokens.sh` already exists as a place a contrast
check could live.

### 7.34 Nothing checks prose against the writing standard

`docs/22_cfdl_controlled_english.md` is adopted and unenforced. The audit that
produced it measured, among other things, that 43% of exercise-prompt sentences
exceed the 20-word limit for an instruction, and that the training chapters write
procedures as questions rather than as imperatives.

Enforcement should extend `tools/check-site-voice.py` rather than add a second
prose linter. That tool already discovers every site-facing source in its
`sources()` function, already has an escape-hatch convention, and is already
wired into `make ci`. A second tool with its own file list would drift from that
one, which is the failure the makefile comments describe.

The reserved annotation form is `ste-allow: <rule id> <reason>`, mirroring the
existing `site-allow:`.

Blocked on nothing, but worth doing after 7.29 and 7.30, so the gate turns green
on its first run rather than red on a known backlog.

### 7.35 Accessibility: the automated findings are fixed; the human pass has not run

The WCAG 2.2 AA assessment (`docs/23`) fixed everything automation found — the
muted-text token failed 4.5:1 in both themes, the playground splitter could not
report its value to a screen reader, and scrollable tables, code blocks and the
results panel were unreachable by keyboard. axe now reports zero violations on
every swept page of both apps, and the deployed sites matched local builds.

What remains is what a rule cannot check: a screen-reader session over the docs
reading flow, the playground round trip and a learn exercise; 2.2's judgment
criteria (focus appearance, dragging alternatives, consistent help); content
order at 200/400% zoom; and a skip link, which landmarks currently stand in
for. Until that pass runs, the public statement is "built to WCAG 2.2 AA;
formal conformance assessment in progress" — not a claim of conformance.

Method note for whoever runs it: test themes through the stored-preference-plus-
reload path. Stamping `data-theme` on a live page produces mixed-token states
that are unreachable in production and read as catastrophic contrast bugs; two
false findings died that way during the assessment.


### 7.36 A repeatable regime cannot use the checked lifecycle vocabulary

The item claimed that a machine with a return edge is not expressible — that a
covenant which breaches and cures, a plant that curtails and restarts, a
facility drawn and repaid each need a state that can be re-entered, and that
the latch forecloses it. Probed with no pack active, the behavior and a
published indicator both work today.

`active when` is level-triggered, re-evaluated every period, so a plant curtails
and restarts with no event at all. A field tracks the regime and flips both
ways, because a `next` may read curves and time, and it publishes as a series:

```
price             100   10  100   10  100   10
active when >=50  100    0  100    0  100    0     curtails and restarts
field curtailed     0    1    0    1    0    1     published, flips both ways
```

**What is actually missing is the vocabulary, not the machine.** `active in
state` and the lifecycle a pack declares in `types.toml` are checked — a state
name is verified against the lifecycle and a typo is `E1332` — but a lifecycle
state is entered by an event, and an event latches, so a regime that returns
cannot use them. It must be a bare field: unchecked, and absent from
`deterministic.transitions`, so the audit trail is silent about when the regime
changed even though the field's series shows it.

That is a type-checking and audit gap. Shape, if it earns its place: a way to
declare states and transitions a model can re-enter, keeping the existing
once-per-period, declaration-order, period-open evaluation, so the states and
edges are reviewable in one place and an undeclared transition is a diagnostic
rather than a silent absence. Truly linear items keep the choice they have
today: calendar-fixed eras are phases, condition-driven regimes are states.

**The three adjacent findings are closed.**

A `set` on a rule-bearing field is no longer discarded. An event's write
overwrites the field at that period and the recurrence resumes from it;
`fixtures/valid/event_reseeds_recurrence` pins `1000, 900, 550, 450, 350, 250`.

An event CAN fire on the boundary of a declared phase.
`when time.phase == "operations"` fires once, at the first period of that phase,
which is what the latch is for — verified firing at 2026-10-01 for a phase
beginning that month. The obstacle was never the event surface: `time.phase` was
null in every model. §6.4 no longer promises event-position helpers that §13
does not define, and says what an expression actually reads.

A date literal does compare, written the way the expression language spells one.
`docs/03` §2 states that expression literals are numbers, booleans and strings,
and §4 provides `date(y, m, d)` and `parse_date(text)`. Both work:
`time.date == date(2022, 1, 1)` fires in that period. A bare `2022-01-01` in an
expression is subtraction because the operator table says it is, which is worth
a lint suggesting the constructor, not a defect in comparison.

Provenance: found modeling the Buenavista del Cobre lifecycle
(`benchmarks/bespoke/buenavista_del_cobre`), August 2026. Narrowed August 2026
by probing each claim: the return edge works, two adjacent findings were stale
or wrong, and the third was a symptom of `time.phase` rather than of events.
---

### 7.37 A recurrence cannot read cash — NOT A GAP, closed August 2026

The item claimed that a note class's balance, `prev` minus what was paid, could
not be written; that `prev.<stream>` and `prev.<waterfall>.<step>` were needed;
and that the engine had to be restructured from layer-major to period-major to
support them. Every claim was probed with no pack active. None survived.

**A balance drawn down by a payment works today.** State the amount ONCE as a
field. The step pays that field and the balance subtracts it — a waterfall step
reads a rule-bearing entity field at the current period:

```
field  pay_amt        0   250   250   250   250
field  bal         1000  1000   750   500   250
step   principal      0   250   250   250   250     <- reads the field
```

No new spelling is needed, and the field/stream boundary `docs/14` draws is
untouched: a field reads no cash, and does not need to.

**There is ONE pot.** Within a run, `remaining` draws it down — that is what it
is for. Across periods, the model states the window the pot draws on, and the
`from` expression is deliberately free (`docs/03` §3.2) so it can. Both shapes
distribute the cash exactly once:

```
end of hold, one waterfall, from series_sum(fcf, 0, time.t):
  cash 600 -> preferred 400, gp split 40, lp split 160        total 600

every period, from available:
  cash 300/period -> distributed 300/period                   total 100%
```

An end-of-hold distribution — a preferred return and then a split — is the
normal shape, and hand-authoring the pot window is a sound fallback where a deal
needs something else.

**Where the wrong answers came from.** Two modelling errors, each of which looks
like an engine defect until the model is read. Declaring two waterfalls on one
entity, each drawing the same pot, double-counts the cash — there is only one
pot, and two claims on it is not something the engine should reconcile. And a
model that asks for cash which cannot exist, then assumes it was received, will
disagree with the cash: that is the model over-specifying an amortization the
collateral cannot support, not the language losing a payment.

**What is true and must stay true.** A distribution is not an operating,
investing or financing flow, so it must never reach a cash flow statement;
subtotals folding before waterfalls run is the correct order. Distribution is
downstream of the cash flow modelling and of the valuation, both struck before
it. A separate WATERFALL statement may be added later and is a different
statement.

Provenance: raised writing `benchmarks/credit/americredit_2017_1`, August 2026;
closed August 2026 after probing each claim. The duplication that prompted it —
that case states its distribution twice — is a consequence of restating an
allocation the model can state once as a field, and is a modeling matter rather
than a missing capability.

---

### 7.38 A misspelled series reads as zero, in silence

`series_sum("no.such.series", 0, time.t)` in a stream returns 0.0 for every
period and emits nothing — no diagnostic at compile time, no warning at run
time. The same read inside a field's `next` does warn: *"series `w.step_a` is
not available in this context; using 0"*. One of the two is wrong, and it is
not the field.

A stream that reads a series which does not exist has almost certainly been
mistyped, or names something the model no longer produces. Reading it as zero
is the worst available answer: a benchmark case can go green while asserting a
line it never computed, and `expected.csv` will agree with it, because zero is
a number.

Cheapest fix: resolve series names at compile time against what the model
lowers to, and refuse the unknown ones. If a late-bound name is genuinely
needed, the run-time path should at minimum warn as the recurrence already
does.

Provenance: found probing 7.37, August 2026, when a waterfall step's series
read as zero and the only reason that was visible at all was a field warning
about a different read.

**Half of this is now closed, and it is worth being exact about which half.**
A name the model does not produce ANYWHERE warns — `W5022_UNKNOWN_SERIES_REFERENCE`
— rather than failing, because a literal naming nothing is a pack idiom as well
as a typo (`cre.exit` names nine NOI components and a given property declares
some of them), and refusing it outright broke four goldens. A name the model
DOES produce but which cannot be seen from where it is read now fails:
`E1342_WATERFALL_SERIES_NOT_VISIBLE`, §7.41 item 3. What remains here is the
first case, and it stays open until the convention is settled — the question is
whether a pack's own expression should be able to name a component the model
lacks, not whether a typo should be caught.

---

### 7.39 A clean-up call cannot end a deal — NOT A GAP, closed August 2026

A clean-up call is a termination right on a threshold: once a deal has paid down
far enough that administering it costs more than the remainder is worth, the
servicer may buy the assets that are left, redeem the notes in full and wind the
deal up. An asset sale, a prepayment in full, a lease termination and an
acceleration are the same shape. The right belongs to the agreement; exercising
it is a decision.

The language already splits it that way, and every piece exists.

The RIGHT and its threshold are an `option` (§14): `exercise when` carries the
condition, `payoff` carries the buy-out price. The DECISION is an event writing
entity state. The CASH stops because a stream's amount reads that state — the
expression environment binds `entity.*` to the stream's owning entity, so no
name is needed and a pack's `amount_expr` can do it as readily as a model can.

Probed with no pack active, an event setting `status = "called"` at t=2:

```
by guard   active when entity.status != "called"        100  100  0  0  0
by amount  100.0 * if(entity.status == "called", 0, 1)  100  100  0  0  0
```

The item's proposed `ends when` clause on a contract would put the decision
inside the record of what was agreed, which is the error §7.40 made. A contract
records the agreement and its `term` states when the obligations run.

**What is true is narrower: no lowering rule uses this.** No rule in any of the
four packs reads entity state in an amount expression, so the streams a contract
lowers to cannot today respond to a decision the model makes. That is rule
authoring, not a missing capability.

**One residue, recorded in §7.40.** A lowering rule can make its stream's amount
zero but cannot make the stream inactive, having no `active_when` key. §9.3
gives every stream an activation predicate and §6.4 of the pack interface
provides no way to set one, so a lowered stream reaches the guard's economics
through its amount and never its statement: a guard says the claim does not
occur, a conditional amount says it occurs and is zero, and the second still
publishes a series.

Provenance: found finishing `benchmarks/credit/americredit_2017_1`, August 2026.
Closed August 2026 after probing each piece. The case asserts nothing after the
call date, which remains honest reporting of a model that does not yet express
the wind-up — expressible now, and worth revisiting when the credit pack reads
state.
---

### 7.40 Capabilities reachable from one layer and not another — NOT A GAP, closed August 2026

The item recorded two instances of a capability the language gives one layer and
withholds from another. Each claim was walked against the specification, then
the implementation, then a probe. None survived.

**Instance 1, a contract cannot be gated though a stream can.** A contract's
`term <start>..<end>` states when its obligations run, and that is complete. The
cases the item lists — a loan repaid early, a lease terminated, a tenant
defaulting, a facility cancelled, a PPA bought out — are decisions and events,
not terms of the agreements, and each has a construct already: an `option` with
its `payoff` (§14), an `event`, or the entity state those write. Putting a guard
on a contract would move a modeling decision into the record of what was
agreed. The item's three proposed fixes all did that.

**Instance 2, the calendar's grain is a pack privilege.** It is not.
`time.ppy` and `time.days_in_period` are documented in `docs/03` §3 and work:
`inputs.rent_year / time.ppy` pays 3000 a quarter on a quarterly book, and
`time.days_in_period` reports 90, 91, 92, 92. The item probed five spellings —
`model.periods_per_year`, `time.periods_per_year`, `model.grain`, `time.grain`,
`time.periods_in_year` — and none of them was the documented one.

**The paths in its table, each for its own reason.**

`run.*` is run configuration. It lives in run.json and is the run's business, not
the model's; `cfg.<name>` exists so a run passes values in deliberately. There is
no `run` root in `docs/03` §3 and there should not be.

`model.npv`, `model.irr` and `domain.*` are settled by §15.2 — "CFDL models do
not declare output metrics" — and an expression reading a fold of the cash it
contributes to would be circular. A hurdle is a post-results item and does not
belong in a model either.

`entity.<symbol>.net_cash_flow` is a RESULTS series key (`docs/06`), not an
expression path; the `entity.` prefix there is an output namespace that shares a
spelling with the expression root. Its null is the open-world `entity` root
behaving as documented, and narrowing that is `docs/18` §4a.

`time.year` and `time.month` have no component helper in the documentation or in
`cfdl-calc`, and need none. A value that varies by date is a curve — seasonality
and a calendar-year rate step both work that way — and periodicity is
`months_between(anchor, time.date) % 12`.

**What the walk did find**, which the item never mentions: `docs/01` §13.4
contradicted §9.3 normatively. §9.3 says a stream MAY carry a guard and is
active for every scheduled occurrence without one; §13.4 said contracts and
streams SHOULD use entity state as "the primary activation mechanism". A SHOULD
against a MAY, an optional mechanism promoted over the effective dates that
actually make a stream active, and contracts named for a guard they do not take.
§13.4 is corrected.

Provenance: raised August 2026; closed August 2026 after walking each claim
against the specification first, the implementation second, and a probe last.
The instance-2 error is worth remembering: five undocumented spellings were
probed and the documented one was not, which is how a working capability was
recorded as missing.
---

### 7.41 Invariants the gates do not check

`make ci` runs fifteen gates and every one of them checks an *output*: goldens
match, benchmark cases reconcile, examples compile, prose conforms. None checks
an *invariant* — a property the engine must hold whatever a model says. Each of
the following is mechanical, and each corresponds to something that went wrong
this month and was found by hand.

**1. Cash purity.** — **shipped**, `tools/invariant-checks.py`, in `make ci`,
and mutation-tested: a waterfall step counted as cash is caught in period 0.
Originally: no field, subtotal, entity rollup or waterfall step may
enter `model_series` or `valued_streams`. The engine holds this by construction
today, and `crates/cfdl-engine/src/lib.rs:1350` explains why — but the
explanation was stale for a year, naming a `state.` prefix that stopped being
the guard when fields began publishing under their owning entity. A gate:
build a model carrying a field, a cumulative subtotal and a waterfall, and
assert `model.net_cash_flow` equals the sum of the stream series to the cent.
Would catch anyone "fixing" the comment's mechanism and losing the real one.

**2. Pack additivity.** — **shipped** as a two-way ratchet in the same gate:
a new stream clause fails until accepted, waived or recorded; a known gap that
closes fails until the record is removed. Originally: For every clause `StreamStmt` accepts, `ContractStmt`
should accept it too or waive it explicitly. §7.40 is what the absence costs: a
contract cannot be gated, so a repaid loan keeps paying. A gate comparing the
two surfaces would have caught it the day the second clause was added to
streams, rather than in a benchmark three packs later.

**3. A series read that cannot resolve from its context must FAIL, not warn.**
— **shipped**. `E1342_WATERFALL_SERIES_NOT_VISIBLE` refuses a `series_sum` /
`series_avg` naming a step of the waterfall it is written in, or of a later
one; an EARLIER waterfall is the documented composition and still compiles.
Checked in the compiler beside `E1341`, its sibling one spelling over, so the
two answer the same reference the same way. The message names the right model:
`paid.<step>` for this period, a balance the distribution moves for a running
total (§7.37).

Re-scoped from a survey gate after review: a waterfall step is a pure function
of its inputs — accept the pot, allocate, move forward — so a step reading its
own waterfall's prior payments is not a missing capability, it is the account
reconstructing its own postings, and the cumulative quantity it wants is a
BALANCE the distribution moves (§7.37).

**It had already shipped a wrong number.**
`fixtures/valid/waterfall_after_contract` capped a note at its balance by
subtracting what it had paid so far. The read saw nothing, the cap never bound,
and a $500,000 note paid out $1,200,000 over six periods with a golden agreeing
— the preferred return paid in full six times, found in the repository rather
than in a report. The fixture now states a per-period cap, which is what it was
computing all along: `ledger_hash` is unchanged, so every published number was
identical and only the expression became honest. `docs/17` §10 was corrected
before this landed and now names the diagnostic.

**4. One path, one value.** — **shipped**, with the semantics decided in
review: an event's write OVERWRITES the field's value at that period, in the
field store itself, and the recurrence resumes from it — a partial liquidation
reduces the balance and the next period amortizes from the reduced balance,
standard finance. Fields and events are now one interleaved walk per period
(`crates/cfdl-engine/src/state.rs`): the recurrence computes the candidate,
guards read the frozen pre-state plus the candidates, writes settle the
column, and `prev` at t+1 reads what settled. A write to a rule-bearing field
never enters the entity-state record, so there is no second copy to go stale.
`fixtures/valid/event_reseeds_recurrence` pins the semantics —
`1000, 900, 550, 450, 350, 250` published, read, and resumed identically.
Originally: the two stores diverged permanently after a write, and a benchmark
asserting the field would have blessed a number no stream ever saw.

**5. A distribution's pot.** `docs/17` §4 says the pot a waterfall allocates is
this period's cash. 29 of 31 waterfalls in the repository build their own out of
assumptions, literals, pack-internal stream ids or hand-maintained fields
(`docs/25`). Until cash-available is bindable there is nothing to assert
against, but the weaker check is available now: flag a `from` expression that
names a stream a pack lowered, since a model reaching into another layer's
internals is the shape that breaks when the pack changes.

The first four are cheap and would each have turned a week of hand-probing into
a failing gate. The fifth waits on the capability.

Provenance: every item is a defect found by hand in August 2026 while writing
`benchmarks/credit/americredit_2017_1` and auditing what it exposed.

---

### 7.42 The discount rate is a run-config item and the specification implies otherwise

Discounting belongs to the run, not the deal: the same cash flows are valued at
different rates by different readers, and a scenario should be able to move the
rate without editing the model. The engine implements that — `config.discount_rate`
drives `npv_at_grain` and `npv_with_offsets`, and is republished as
`run.annual_discount_rate`.

The specification says something else. `docs/01` §12.1 introduces deterministic
assumptions with exactly this example:

```cfdl
assume discount_rate = 0.10
```

which does nothing. Measured, with `assume discount_rate = 0.25` in the model:

| run config | `model.npv` |
|---|---:|
| 0.03 | 2,828.61 |
| 0.10 | 2,486.85 |

The model's figure is ignored at both. A reader following the specification's
own illustration sets the deal's discount rate, sees a plausible NPV, and is
looking at the run's default.

**Items 1 and 2 are done.** §12.1 now illustrates the construct with
`assume base_rent = 4000`, states that a deterministic assumption is a value the
model owns and is read as `inputs.<name>`, and says that discounting belongs to
the run as `annual_discount_rate`, pointing at `docs/09_user_guide.md` where the
run configuration is documented. The specification no longer teaches the trap.

**Item 3, a diagnostic, is open and is not obviously right.** Warning on the
NAME alone would fire on a model that declares `assume discount_rate` and uses
it for something of its own, which is legal. The safer shape is a check that
does not guess at intent: an assumption declared and never read as
`inputs.<name>` is dead weight whatever it is called, and this case is one
instance of it. That is a larger check than this item, and worth deciding on its
own merits rather than as a name heuristic.

Provenance: found while writing a model without a pack, August 2026 — the
assumption was declared, the NPV moved with the config, and nothing said so.

---

### 7.43 Results do not say which entity owns a stream

This is a request rather than a defect, and the part of it that is a defect is
smaller and different from the part first written down.

**The axis exists and is correct.** Without a pack, a model's structured view of
its own cash is `entity.<symbol>.net_cash_flow`, one series per entity,
aggregated through `part of` — twelve pools rolled into a trust to within 2e-06
in the pack-free AmeriCredit model, identical to the packed one.

**A model without a pack publishes no `domain.*` series, and that is by
design.** `docs/01` §15.2 states that CFDL models do not declare output metrics
and points at the pack interface for how a pack defines output categories,
aggregations and metrics. A statement is a pack's job. The request below is for
a DEFAULT presentation, not for a model-declared one, and it should not be read
as a gap in the language.

**The original claim about a parent's own cash was wrong.** A trust with pools
and a fee of its own does not show one number: the fee is published in its own
right, and the arithmetic closes.

```
entity.asset.trust.net_cash_flow   550        rollup
entity.asset.pool1.net_cash_flow   300
entity.asset.pool2.net_cash_flow   200
stream.trust.fee                    50        the parent's own cash
                                              550 - 300 - 200 = 50
```

**What is actually missing is the ownership.** A series entry in results carries
`index`, `offset` and `values`, and nothing else; `docs/06` never names an
owner. So the derivation above needs the parent-child tree, which lives in the
IR rather than in results, and a consumer holding `results.json` alone cannot
attribute a stream to an entity at all. Name inspection is not a substitute: a
pack-lowered `cre.unit.base_rent.anchor` does not contain the symbol of the
entity that owns it.

Publishing stream ownership is the smaller change and the more useful one. It is
a structural fact the engine already holds, it lets any consumer build a
hierarchy view without the engine shipping one, and it makes an entity's own
cash derivable from results alone. Publishing `entity_own` beside the rollup is
the narrower version of the same idea and would serve the default statement
directly.

**The presentation request stands on its own merits.** A default statement
organized by the entity tree — each node's cash with its children beneath it,
no declarations and no pack — would give every model a readable cash flow rather
than a flat list of series keyed by symbol. Nothing in the language prevents it
and nothing forces it; it is a product decision about what a pack-free run
should look like. A declarable statement structure, the pack's fold available to
a model that wants to name its own lines, is the larger job and a separate one.

Provenance: found sectioning `benchmarks/credit/americredit_2017_1` into a
pack-free model, August 2026. Rewritten August 2026 after probing the own-cash
claim, which does not hold — the parent's streams are published individually and
the rollup arithmetic closes — and finding the ownership gap underneath it.
---

### 7.44 The engine is one file, and the stages it runs are invisible in it

`crates/cfdl-engine/src/lib.rs` is 5,341 lines. It holds the timeline, the
fields, the events, both stream phases, the subtotals, the waterfalls, the
entity rollups, the metrics and the results assembly.

Those stages are real. Each completes before the next begins, each sees only
what finished earlier, and the boundaries between them are the language's own
semantics: a field reads no cash, a subtotal folds cash and is never counted as
cash, a waterfall allocates cash and never feeds it back.
`fixtures/valid/evaluation_order` pins them.

None of that is visible in the file layout, and the cost is not hypothetical. A
comment at line 1350 described the boundary that keeps state out of cash, named
the wrong mechanism, and stayed wrong for a year — because nothing about the
file's shape says where one stage ends.

Shape: crates or modules per stage, orchestrated in order. **Modules shipped,
August 2026**: `config`, `timeline`, `ir`, `env`, `fields`, `events`,
`streams`, `distributions`, `results`, `stochastic`, with `run_deterministic`
in `lib.rs` as the orchestrator — 149 goldens byte-identical across the move.
Crates remain open as the second step if the boundaries earn it.

1. timeline
2. fields and events — mutually dependent, so one module
3. streams, in their two phases
4. results — the netted cash, the rollups, the metrics
5. distributions — waterfalls, which consume a result and emit payee amounts

The repository already runs sixteen crates, so the pattern is established. The
split is mechanical: the call order in `run_deterministic` already names the
stages, one call each.

Provenance: raised August 2026, after a session in which every finding was a
boundary between two stages that the code does not separate.

---

### 7.45 A waterfall with no schedule distributes once, at the model start

`lower_schedule` answers a missing schedule with `OnDate(time_start)`
(`crates/cfdl-compile/src/lib.rs`), so a waterfall that says nothing about when
it runs distributes exactly once, in the first period — before the deal has
produced anything to distribute. Probed with no pack active: a constant pot of
500 across five periods paid `500, 0, 0, 0, 0`.

Nothing says so. `docs/17` and `docs/01` state no default, and no diagnostic
fires. The engine believes something different — `run_waterfalls` reads a
missing schedule as *every period* — but that branch is unreachable, because the
compiler never emits `None`. Two components disagree about the default and the
one that loses is the one whose comment explains the intent.

The first period is the least defensible of the three candidate defaults. A
waterfall accumulates over a holding period and then distributes — a preferred
return and then a split — so the useful default is at the END of the hold, not
its start. Distributing at the start answers with whatever the first period
happened to produce.

Shape, in the order they should be considered: require the schedule and reject
the omission, which makes the author say what a distribution date is; or default
to the end of the hold, which is the shape a deal actually has. Defaulting to
every period matches the engine's dead branch but is not the normal case.
Whichever is chosen, the engine's unreachable branch should go, so one component
states the rule.

Provenance: found August 2026 while probing 7.37, when a waterfall drawing
`from available` appeared to pay only the first period's cash. The pot was
correct; the waterfall had run once. Two earlier readings of this repository's
behaviour were wrong because of it.


---

### 7.46 A run with no discount rate still publishes an NPV

A run that states no rate discounts at zero and reports the result as
`model.npv`:

```
cfdl run <ir> --out results.json          (no --config, no --rate)

  model.npv                = 3750.0
  run.annual_discount_rate = 0.0
```

3,750 is the undiscounted total. The rate is published beside it, so the run is
not lying, but a metric named `model.npv` is being reported for a valuation
whose rate nobody stated, and a reader scanning results for a present value sees
a figure that reads as valued and is not.

**No rate, no NPV.** A discounted metric with no discount rate is a missing
term, not a shortcut, and the repository already applies that standard where it
matters most: `cre.permanent_debt` deliberately defaults neither `principal` nor
`rate`, because "a mortgage with an unstated balance or an unstated rate is not
a modeling shortcut, it is a missing term, and E5006 should say so rather than
the pack inventing a zero." A valuation with an unstated discount rate is the
same shape. Omit `model.npv` and the metrics derived from it when no rate is
supplied, and say why.

The zero default is what makes the current behavior defensible-looking: it is a
real arithmetic answer to a question nobody asked. Removing it costs nothing a
model wanted, because a model that means zero can state zero.

**Scope to settle when implementing.** Which metrics travel with the rate —
`model.npv` certainly; whether `model.irr`, `model.payback_years` and
`model.wal_years` do is a separate question, since a time-weighted life needs an
axis rather than a rate. And whether omission or an explicit null is the better
shape in `results.json`, which `docs/06` should state either way.

**Not the same as letting a model set the rate.** Discounting belongs to the
run: one set of cash flows is valued at several rates by different readers, and
neither rate is a fact about the asset (§7.42). A model default with a run
override, or a run that falls back to `inputs`, would put the resolution order
out of sight and let one model value differently depending on which channel won.
The fix here is to require the rate, not to relocate it.

Provenance: found August 2026 while correcting §7.42, when a probe run without a
config produced an NPV equal to its own total.

---

### 7.47 Fourteen reserved words are consumed by no production

*Belongs with language and packs (section 5). Investigate.*

Fourteen of the 95 words the lexer reserves are read by no parse rule. They
appear in `crates/cfdl-parser/src/lib.rs` only inside `keyword_text`, which
renders a keyword back to text for an error message:

```
Mon Tue Wed Thu Fri Sat Sun          weekday names
short_front short_back
long_front long_back                 stub policies
direction owner tags
```

Eleven belong to features `docs/10` records as REJECTED — `schedule ... on
<weekday list>` and `schedule ... stub <policy>` are both struck there. The
other three are vestigial: a stream states its direction with the bare words
`inflow` and `outflow`, `owner` appears in no rule, and `tags` is one of the
blocks `docs/01` §8.1 describes as tolerated by the parser and absent from IR.

They are not free. A reserved word is unavailable everywhere a name is expected
(§7.19), so each one costs a word a model or a pack might want. `owner` and
`direction` are the ones to regret: an asset has an owner, a flow has a
direction, and both are natural field names.

To investigate: whether each is genuinely dead, whether removing it changes any
diagnostic text, and whether the weekday names should go with the rejected
feature or be kept against its return.

Found August 2026, auditing every keyword against the production that consumes
it.

---

### 7.48 The lexer reserves 38 words the specification does not list

*Belongs with language and packs (section 5). Investigate.*

`docs/01` §18 documents 57 reserved words. The lexer reserves 95. Nothing goes
the other way — everything §18 lists is genuinely reserved — so the drift is
one-directional: the implementation grew and the list did not.

```
LogNormal Normal Triangular Uniform clip
active in state parties tags
annual daily monthly quarterly year quarter month months day days due mid net
none following preceding modified_following modified_preceding
short_front short_back long_front long_back
phase_start phase_end phase_enter
curve true false
```

Several are plainly load-bearing and simply went undocumented — `curve` opens a
statement, `active`, `in` and `state` appear in stream and entity blocks,
`true` and `false` are literals, and the calendar and interval words drive
`time` and `schedule`. For those the fix is to publish them.

The rest are the reason this is worth investigating rather than just editing.
`year`, `month`, `net`, `state`, `active`, `in`, `none`, `mid`, `due` and
`clip` are ordinary words a financial model wants for a field, taken silently
for constructs where a bare word in a naming position could not be confused
with them. Eleven of the 38 overlap §7.47 and are reserved for nothing at all.

To investigate: which of the 38 must be reserved, which are local enough to be
contextual, and which should simply be released — then reconcile §18 with
whatever survives, so the published list is the enforced one.

Found August 2026, diffing §18 against the lexer's table.

---

### 7.49 The EBNF does not describe the entity block the parser accepts

*Belongs with language and packs (section 5). Investigate.*

The canonical grammar says an entity block contains fields and nothing else:

```ebnf
entity_block    = "{" { kv_stmt } "}" ;
kv_stmt         = IDENT literal_or_expr ;
```

The parser also accepts `part of <entity_ref>` and `state <name>`, and the
grammar file's own comment beside `entity_field` describes the second —
"`state <name>` inside an entity block is unrelated: it names the lifecycle
state the entity opens in" — so the omission is in the productions rather than
in the intent. `docs/01` §7.1 shows both clauses in its examples.

This is the mirror of §7.19. There the implementation is NARROWER than the
grammar; here it is WIDER. Both are conformance gaps, and both were found by
reading the EBNF rather than the prose.

To investigate: what else the parser accepts that the EBNF does not state. The
grammar is published for tooling — railroad diagrams and parser generators —
so anything it omits is missing from every consumer of it, and a generated
parser would reject models the reference implementation compiles.

Found August 2026, reading the canonical grammar while walking §7.19.

---

### 7.50 A model cannot name the streams its own contracts produce

*Belongs with language and packs (section 5).*

`docs/01` §13.2 gives the modeller `deactivate stream <StreamName>`. §9.1 says a
stream name is a qualified name of at least two segments and illustrates it with
`cre.lease.base_rent`. `docs/07` §6.4 gives that identical string as its example
of a GENERATED name. The specification draws no distinction between a stream a
model declared and a stream a contract produced.

The modeller cannot use it. Two files differing only in that one line:

```cfdl
event refinance when time.t >= 12 {
  set entity asset.tower.status = "refinanced"
  deactivate stream <name>
}
```

```
deactivate stream tower.fee              declared in the model   compiles
deactivate stream cre.lease.base_rent    produced by a contract  E1302_UNRESOLVED_STREAM_REF
```

The second name is exactly what the contract lowers to — verified against the
IR, which carries one stream under that name.

**Both routes the specification offers are closed.** §9.3 grants every stream an
activation predicate, and a model cannot add one to a stream it did not write;
`docs/07` §6.4 publishes the keys a lowering rule may emit — `stream_name`,
`owner`, `direction`, `currency`, `schedule`, `amount_expr` — and the guard is
not among them. So a contract's streams can be zeroed through their amount and
never made inactive.

**The cause is stage order, not syntax.** Traced through the compiler:

```
  1  read source
  2  lex
  3  parse
  4  resolve imports
  5  resolve_symbols          <- E1302 is decided here
  6  resolve_active_pack
  7  validate
  8  validate_expressions
  9  source-level checks
 10  lower_contract_streams   <- the streams are generated here
 11  check_lowered_prev_first_period
 12  construct IR and emit
```

The symbol table is built at 5 and the streams appear at 10, so at the moment
the check runs the streams a contract will produce do not exist. The check
cannot tell "not yet generated" from "misspelled" and reports both as `E1302`.
`docs/08` records that its purpose is the second: a misspelling once "matched
nothing and the action was silently inert."

**The specification's stage list does not describe the operation.** `docs/04`
§1.1 names nine stages with Lowering eighth, and every bullet under it is
transcription — normalize literals, default missing fields, derive canonical
IDs, construct IR objects, preserve provenance. Turning contracts into streams
is not among them. It is described only in `docs/01` §8.1, in the language
specification rather than the compiler's stage list.

That omission is the finding. Entities, waterfalls, assumptions and curves are
TRANSCRIBED — one statement in, one object out, nothing newly named. Only
contract lowering is GENERATIVE. Because the stage list does not separate the
two, it places the operation that creates names after the operation that
resolves them.

**Two repairs, and the smaller one is already proven.** Expansion needs only the
contract declaration and the pack's rules, both available once the pack is
resolved: `stream_name` is a declared property of a rule (§6.4), and the
compiler already works out which rules match which contract without building a
stream. So expansion could become its own stage before name resolution, and the
symbol table would cover every stream that will exist.

Alternatively the check moves to the post-lowering point the compiler already
has. Step 11 above, `check_lowered_prev_first_period`, validates lowered streams
after they are generated, so the position exists and one check already uses it.
Typo detection survives either way, because a misspelling still matches nothing
once every stream is built.

**Where the fix does NOT belong.** Not in the contract: a contract records what
was agreed, and a termination or a switch-off is a modelling decision (§7.39,
§7.40). Not in the lowering rule either — a rule emitting
`active when entity.status != "refinanced"` would bake the model's own
vocabulary into the pack, requiring the rule author to guess which status
strings a modeller will use. The decision is the modeller's, so it is expressed
in the model, and the compiler has to resolve the name.

**A third instance of one shape.** `E1302` here, `E1131` for a field an event
wrote but no entity declared, and `E1342` for a waterfall step reading its own
waterfall. Each check is right about typos, each runs where its subject does not
yet exist, and each removes a capability the specification grants.

Found August 2026, walking §7.39 with a working model supplied by the author:
the same event, in the same form, against two targets.
