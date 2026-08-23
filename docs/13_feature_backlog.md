# CFDL — Feature Backlog

Status: informative. Things worth building that are **not** defects.

Bugs do not belong here; they get fixed or they get a failing test. What
belongs here is capability the language or a pack does not yet have, where the
gap was found by trying to model something real and hitting a wall. Each entry
therefore says what could not be expressed, and what forced the discovery —
a backlog item with no provenance is a guess.

Ordered within each section by how much it unblocks, not by effort.

---

**Closed items are removed, not archived.** A capability that ships is
described in the language documentation; reasoning that turned out to be wrong,
and shapes the language already supports, are recorded in
`docs/26_lessons_learned.md`. This file holds work to do.

## 1. CRE pack

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

This is the pool-factor problem one level up. That defect was a pool's amortisation state not
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

**A third failure mode is now closed, and it was not the one this entry
describes.** A pack expression could read an INSTANCEABLE stream family by its
bare name. `.*` matches the bare name and its children; a bare pattern matches
only the bare name, so every suffixed instance is skipped — and nothing warns,
because the pattern did match something. That is worse than the case below: the
warning that would fire on a name matching nothing never fires at all.

It reached main twice, in the same expression, both times in forward NOI:
`cre.pct_rent` double-counted an unsuffixed contract, and `cre.property.opex`
made an instanced expense line invisible, overstating NOI and the exit price
struck off it. Both were found by hand, months apart. `tools/check-pack-series.py`
is now a gate over three surfaces — lowering expressions, metric selectors, and
statement row selectors — because the same mistake was made independently on
two of them.

**Of what this entry actually describes, half is closed, and it is worth being
exact about which half.**
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
should accept it too or waive it explicitly. The absence costs what §7.50 records: a
contract's streams cannot be gated from the model. A gate comparing the
two surfaces would have caught it the day the second clause was added to
streams, rather than in a benchmark three packs later.

**3. A series read that cannot resolve from its context must FAIL, not warn.**
— **shipped**. `E1342_WATERFALL_SERIES_NOT_VISIBLE` refuses a `series_sum` /
`series_avg` naming a step of the waterfall it is written in, or of a later
one; an EARLIER waterfall is the documented composition and still compiles.
Checked in the compiler beside `E1341`, its sibling one spelling over, so the
two answer the same reference the same way. The message names the right model:
`paid.<step>` for this period, a balance the distribution moves for a running
total.

Re-scoped from a survey gate after review: a waterfall step is a pure function
of its inputs — accept the pot, allocate, move forward — so a step reading its
own waterfall's prior payments is not a missing capability, it is the account
reconstructing its own postings, and the cumulative quantity it wants is a
BALANCE the distribution moves — see `docs/26_lessons_learned.md`.

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
was agreed, and a termination or a switch-off is a modeling decision — see
`docs/26_lessons_learned.md`. Not in the lowering rule either — a rule emitting
`active when entity.status != "refinanced"` would bake the model's own
vocabulary into the pack, requiring the rule author to guess which status
strings a modeller will use. The decision is the modeller's, so it is expressed
in the model, and the compiler has to resolve the name.

**A third instance of one shape.** `E1302` here, `E1131` for a field an event
wrote but no entity declared, and `E1342` for a waterfall step reading its own
waterfall. Each check is right about typos, each runs where its subject does not
yet exist, and each removes a capability the specification grants.

Found August 2026, walking the clean-up call with a working model supplied by
the author:
the same event, in the same form, against two targets.

---

### 7.51 Nothing validates a run configuration against its schema — SHIPPED (first half)

`tools/check-run-schema.py` validates every `run.json` in `benchmarks/`,
`fixtures/`, `training/` and `examples/` against
`docs/schemas/run.schema.json`, and `make ci` runs it. 123 configs pass. It
follows `check-ir-schema.py`, including its rule that a gate which can pass
without running is not a gate: `CFDL_REQUIRE_SCHEMA_GATE=1` turns a missing
`jsonschema` into a failure rather than a skip.

It catches what the schema's own description says the design prevents:

```
deterministic: Additional properties are not allowed ('anual_discount_rate' was unexpected)
deterministic/arithmetic: 'float' is not one of ['decimal', 'excel_compat']
```

The drift that prompted it is closed too. `valuation_grain` had been accepted by
the engine and documented for as long as it existed while the schema never
listed it, and `DeterministicCase` sets `additionalProperties: false` — so a run
stating its own grain would have been rejected by the schema it conforms to. It
is in the schema now, and a config using it passes.

**The second half is open, and is a decision rather than a gate.** An override
key that resolves to nothing is still accepted in silence. Probed with four
that match nothing — an assumption that does not exist, a `cfg` path no
expression reads, an `obs` path no expression reads, and a stream that does not
exist — all four were accepted, discarded, and unreported, with the cash
unchanged.

Two of the four could be checked and two could not. `inputs.<name>` names an
`assume` statement and `stream.<name>:amount` names a stream: both are declared,
so both are verifiable against the IR. `cfg.<path>` and `obs.<path>` name
nothing declared — they are channels a model opts into by writing the path in an
expression — so the only available signal is that no expression reads it, which
may be legitimate in a run driving several models.

`Parameters` also says the opposite of the document it lives in: *"Four key
shapes are recognized and anything else is ignored"* against the schema's
*"Unknown properties are rejected."* Whichever is intended, they should agree.

### 7.54 The HUD case cannot move onto `cre.permanent_debt`

*Belongs with the CRE pack (section 1). Split from the closed 7.14.*

The reporting half is done: `benchmarks/cre/hud_home_multifamily` states P+I+MIP
the way its source publishes it, and mortgage insurance is no longer counted as
debt service.

The case still hand-writes its mortgage rather than using `cre.permanent_debt`,
because HUD's instrument carries mortgage insurance the contract does not model.
A `cre.mortgage_insurance` contract is the shape that would close it, and it is
not added on one case's evidence — the pack candidate list (§7.5) is where it
belongs if a second source wants it.

This is the coverage question §7.3 and §7.15 measure, in one instance: a case
that reconciles externally while routing around the pack it should validate.

---

### 7.55 A model cannot declare a subtotal or a statement

*Belongs with the language and engine (section 5). Split from the closed 7.17.*

Reporting is a language capability in its design — the category roots are the
language's, a stream states its own `category`, and a pack-less model classifies
its streams correctly. The DECLARATIONS still live only in pack TOML, so a model
with no pack cannot declare a subtotal or a statement of its own.

That needs a surface in the language and the syntax is undecided; `docs/16`
records the question. It is the half this item's original title was about, and
the larger of the two — the pack-side fold, the classification, the grain
folding and the display sign all shipped.

Related: §7.43, where the same absence shows up as results carrying no statement
for a pack-less model, and §7.25, where a model cannot declare a metric either.
The three are one surface question asked from three directions.

---

### 7.56 A term deferred to `inputs.` is never bounds-checked

*Belongs with the language and engine (section 5). Split from the closed 7.24.*

Two questions survived the correction to 7.24, and they are separable.

**A term referencing `inputs.` escapes its pack's bounds.** `docs/01` §8.2.1
accepts this deliberately — the value is not known at compile time — but it
means a scenario may drive a term past a bound the pack states, at compile time
and at run start alike. Either the bound is checked when the value arrives, or
the pack's bound means less than it appears to.

**Should `cfg.*` work in a term as well?** It is the run configuration's other
half. A reader who reaches for it today gets a diagnostic saying the value is
invalid rather than that the channel is wrong: `E9016` naming a bound is
actively misleading when the term is `cfg.psa`. If the answer is no, the
diagnostic should say so.

Both are about a term's relationship to the run rather than about bounds as
such, which is why they belong together.

---

### 7.57 A pack rule cannot accrue on act/act, because a divisor is not a fraction

*Belongs with the packs (section 5). Split from the closed 6.1.*

`year_frac` accepts `act/act` (ISDA), so a hand-written model can accrue on it.
A pack rule cannot: `{{model.accrual_divisor}}` expands to `<ppy>` or
`(360 / time.days_in_period)` — one number per period — and act/act needs a
denominator that changes with the year the period falls in.

The shape is the one the expansion table already implies. A divisor is the
reciprocal of a year fraction:

```
30/360   rate / 12                  ==  rate * year_frac(s, e, "30/360")
act/360  rate * days/360            ==  rate * year_frac(s, e, "act/360")
act/365  rate * days/365            ==  rate * year_frac(s, e, "act/365")
```

So the placeholder could expand to a `year_frac` call over the period's bounds
rather than to a number. act/act then falls out with no special case, and the
pack placeholder becomes sugar over a capability a model already has natively —
which is the property worth having whether or not act/act is the reason.

Note that two of the three expansions are already run-time text, not compile-time
constants: `(360 / time.days_in_period)` reads the environment. So the argument
that the divisor must resolve at compile time holds only for the fixed case.

**What it needs first.** `year_frac` takes two dates, and an expression can read
`time.date` and `time.days_in_period`. Whether those reconstruct the period's
start and end — and which end `time.date` denotes — is the fact to establish
before scoping this.

---

### 7.58 A contract's type is recovered by string prefix, not carried

*Belongs with the packs and the compiler (section 5). Replaces the closed 7.53.*

A pack declares its contract types in `ontology/types.toml` with an identity:

```toml
[[contracts]]
type_id = "CRE.Contract.UnitLease"
contract_name = "cre.lease_unit"
subject_family = "asset"
parties = ["landlord", "tenant"]
```

A model must suffix a contract whenever the deal has more than one of
something — two tenants are `cre.lease_unit.tenant_a` and `.tenant_b`. Nothing
carries the fact that both are a `CRE.Contract.UnitLease`. Every consumer
recovers it by string surgery instead: strip the declared name off the front and
check the next character is a dot.

7.53 closed by deleting `ContractMatch` and sharing ONE predicate between
lowering and validations, so the two can no longer disagree. This item is the
next step, and it is a different claim: the predicate should not need to exist.

**What carrying the type would buy.**

*Resolution becomes decidable.* The predicate takes the first declared name that
fits. Across four packs there are 39 contract types and none is a dotted prefix
of another, so today there is exactly one answer — but that is accidental, not
enforced. A pack adding `cre.lease_unit.retail` as a TYPE makes
`cre.lease_unit.retail.suite_3` match two declarations, and the predicate picks
one silently. Resolving once can require a unique type and say so when it cannot
find one.

*A misspelling gets a diagnostic.* `cre.lease_unitt.tenant_a` names no type.
Today it matches no lowering rule and no validation; whether anything reports
that is the first thing to establish here.

*Matching becomes equality.* Lowering and validations both compare `type_id`,
and the prefix logic exists in one place — resolution — instead of at every
call site.

**Scope.** Resolve at contract declaration, carry type and instance name on the
contract, migrate both match sites, and decide whether the type surfaces in the
IR schema. Larger than 7.53 was; nothing is broken while it waits.

---

---

### 7.60 A weekly schedule cannot be anchored to a weekday

*Belongs with the language (section 5). Found building the keyword register.*

`docs/01` §18 documented `Mon` through `Sun` as reserved words for eight
versions. They are reserved, they render in error messages, and no production
reads them. The syntax they imply does not exist:

```
schedule every week on Mon from 2026-01 to 2026-02
  -> Expected 'day <n>' or 'eom' after 'on'
```

`weekly` is not a calendar frequency either — the frequencies are `daily`,
`monthly`, `quarterly` and `annual` — so a weekly TIMELINE is unavailable as
well, though `every week` is a valid schedule interval.

Provenance: found by the gate that now holds §18 to the lexer. The words were
documented as though the feature shipped, which is how it went unnoticed.

**What it needs.** `on <weekday>` in the schedule anchor, and a decision on
whether a weekly calendar frequency is wanted or whether weekly schedules on a
daily timeline are the answer. No case needs it yet; a rent roll on weekly
billing or a daily-book instrument settling on Fridays would.

---

### 7.61 Nothing checks the grammar against the parser

*Belongs with the language (section 5). Replaces the closed 7.49.*

`docs/schemas/CFDL_v0_1_Grammar.ebnf` is NORMATIVE and published. `docs/02`
says implementations MUST support the lexical rules, calls the grammar
"suitable as the basis for a hand-written parser or parser-generator input
after minor adaptation", and the site offers it for download for use with
"railroad diagram generators, parser generators, etc."

Nobody has ever performed that adaptation, so nobody discovered the grammar did
not survive it. Five productions were wrong when checked by hand — `contract`
alone was wrong four ways, and would have rejected 519 of the 520 contract
declarations in this repository. They are fixed. Nothing stops it recurring.

**The parser is hand-written recursive descent, so the grammar is source for
nothing.** That is the right call — the diagnostics are a feature and generated
parsers do not produce them — but it means the two artefacts agree only by
attention.

**The long-term answer is to make the grammar executable in CI, in both
directions**, without generating the product parser:

- Build a RECOGNISER from the EBNF and require it to accept every shipped
  `.cfdl` — around 500 files CI already proves parse. Catches a grammar that is
  too narrow, which is what `contract_stmt` and `entity_block` were.
- Generate sentences from the EBNF and require `cfdl parse` to accept them.
  Catches a grammar that is too broad, which is what `map_entry` was.

That is how a published grammar is normally held to an implementation — spec
tests, in the manner of WebAssembly or test262 — and it keeps the hand-written
diagnostics. Cost is a real project: an EBNF adaptation layer, a generator
dependency, and a gate.

**The cheap interim** is a terminal cross-check: extract the keywords each
production mentions and require the parser to read them, and the reverse. Same
shape as `check-keyword-register.py`, no dependency. It would have caught
`owner` and `direction`; it would NOT have caught `term` moving inside the
contract block, so it is a stopgap and should be labelled one.
