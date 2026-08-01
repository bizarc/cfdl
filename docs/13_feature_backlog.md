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

**This is not item 3.2 (per-period state) and should not be bundled with it.** A
PSA ramp is deterministic in loan age — nothing accumulates — so the natural
fix is a calc builtin holding the schedule, exactly the `macrs_rate` pattern:
a published table behind a function. Substituting one call for `pow(k, p)`
would make every existing pool rule work under a ramp.

Found building `benchmarks/credit/mbs_pool_conventions`. The constant-hazard
case (1% SMM / 1% MDR) reproduces to the reference figure; the ramped variant on
the same pool (150% PSA, 100% SDA) needs this.

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

## 3. Language and engine

### 3.1 A stream may not read another period's value

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

### 3.2 Per-period persistent state

No accumulator, no carryforward, no balance that a period can add to and a
later period draw down. Cash sweeps, revolver draws, FF&E reserves, escrow
accounts, NOL carryforwards and construction-interest capitalisation all need
it, and `packs/opco/lowering/rules.toml` says so in its header.

Not discovered by this work — it is a known absence — but recorded here because
3.1 is a strictly smaller version of it and the two should share a design.

---

## 4. Cross-pack

### 4.1 Day count beyond the four supported bases

`{{model.accrual_divisor}}` handles `30/360`, `30e/360`, `act/360` and
`act/365`. `act/act` is not supported: it needs the days in the *year* the
period falls in, which the expression environment does not expose
(`time.days_in_period` is the period, not the year).

Low urgency — the four cover most instruments — but `act/act` is the government
bond convention and will be wanted if a sovereign or municipal pack appears.

### 4.2 excel_compat cannot be selected for a model run

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

Found while validating the credit pack against an external reference and asking
whether Excel mode would move the numbers. It cannot be turned on to find out.

### 4.3 An acquisition or disposal in a period other than the term's

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

Section 1 and item 3.1 were found building `benchmarks/cre/mit_rentleg_plaza`
against MIT OpenCourseWare 11.431J Problem Set 1 — the first CFDL benchmark
checked against a published third-party figure rather than an in-house
reference. Section 2 came the same way, from `benchmarks/credit/mbs_pool_conventions`
against the published industry reference for MBS cash flows — which also found
three outright defects, in the prepayment base, the recovery basis and the
payment-striking divisor, all fixed rather than listed here.

That is the argument for building more of them: an external number finds gaps
that two of your own implementations agreeing never will. See
`research/CFDL_pack_roadmap_and_model_sourcing.md` for the catalogue.
