# Payment timing (normative)

How a schedule places cash in time, and how that placement discounts. This
section exists because the rule was previously unwritten and was implemented
incorrectly more than once — each time by inferring semantics from code that
already had them wrong.

## 1. Periods are counted from 1

A CFDL model is written in English and read by people.

```cfdl
time calendar monthly from 2026-01 for 60
```

That is **60 months, period 1 through period 60**. Period 1 begins on the start
date. Every discussion, diagnostic, document and error message uses this
form.

The IR and engine index from `0` to `n-1`, as implementations do. That is an
implementation detail and MUST NOT surface in the language, in documentation,
or in anything a user reads. A model that spans a 120-month loan plus a
six-month recovery lag is `for 126` — not 127.

## 2. A payment belongs to the period that earned it

Period 1's payment is in period 1.

An instrument's schedule does not move cash into a later bucket. A 120-month
loan makes its payments in periods 1 through 120. A default in period 120
recovering six months later recovers in period 126.

This holds regardless of annuity convention. The convention does not decide
*which* period holds the cash.

## 3. The convention decides where in the period the cash falls

What separates an ordinary annuity from an annuity due is the position of the
payment inside its own period, and therefore how far it is discounted.

| Written | Position in period | Discounted from |
|---|---|---|
| `on <date>` | start (default) | the period's open |
| `on <date> end`, or a rule with `schedule_placement = "end"` | end | the period's close |
| `every month from … to …` | end (default) | end of the period |
| `every month start from … to …` | start | start of the period |
| `every month on eom from … to …` | end | end of the period |
| `every month on day 15 from … to …` | day 15 | that point in the period |
| `every year mid from … to …` | halfway | the period's midpoint |

This is Excel's convention: `NPV` discounts the first value by one full
period, and an annuity due is the same series with the first payment left
undiscounted (`PMT`'s `type` argument, and `pmt(rate, nper, pv, [fv], [due])`
in the CFDL expression library).

A one-shot flow defaults to its period's open, because that is right for the
case it was written for: a purchase on 2026-01 settles then and has not waited
through a period. It is wrong for a **disposal**. A reversion is taken at the
end of the holding period, so a year-5 sale discounts five periods, not four —
the date names the period, and the position within it is a separate fact. A
pack lowering rule says which it means with `schedule_placement`; the
disposal rules in `cre` and `opco` set it, and acquisitions, funding draws,
dated leasing costs and tax credits do not.

On a monthly model the difference is one month. On an annual model it is a
year, and about 9% of a reversion at 12% — see
`benchmarks/cre/mit_rentleg_plaza`, whose published figure is only reproducible
with the later placement.

### The unified rule

A payment's discount exponent is its period number less one, plus how far
through the period it falls:

```
exponent = (period - 1) + offset
```

where `offset` is:

| Schedule detail | offset |
|---|---|
| `start` | `0.0` |
| `mid` | `0.5` |
| `end`, the recurrence default, or `on eom` | `1.0` |
| one-shot default | `0.0` |
| `on day <n>` | `n / days_in_period` |

There is one mechanism, not three special cases. A schedule specified more
precisely simply produces a more precise offset.

**`mid` is a convention, not a date**, and that is what separates it from a day
rule. `on day 15` asks where in the period the 15th falls, and the answer
depends on how long the period is — half a month, a sixth of a quarter, a
twenty-fourth of a year. `mid` says the cash arrived evenly and is therefore
summarized at the midpoint, which is half a period on every calendar. It is
what project finance and banker DCFs mean by mid-period or mid-year
discounting, and it applies to flows rather than to prices: a terminal value or
a sale is struck at a point in time and is discounted whole.

`mid` states a position, so it cannot be combined with another one. With `start`,
a day rule, or `net` payment terms it is
`E2109_SCHEDULE_CONFLICTING_PLACEMENT`. The `net` case is the interesting one:
payment terms are resolved on the calendar and move cash between period
buckets, while `mid` positions cash inside whichever bucket it lands in.
Combining them would bill at the period end and then discount as though the
cash had arrived halfway through it. Composing them properly means billing from
the midpoint and carrying the lag's sub-period residual into the offset — a
real design question, not a default to pick quietly.

### The same axis carries WAL and payback

Discounting is not the only thing that needs to know when cash moved.
`model.wal_years` and `model.payback_years` use the identical position — a
flow's time in years is `(period + offset) / ppy` — so all four of NPV, IRR,
WAL and payback agree about when a given dollar arrived.

That makes an ordinary annuity's first monthly collection fall at 1/12 of a
year rather than at zero, which is the market definition of weighted average
life: a prospectus states it as *the number of years from the closing date to
the related distribution date*. It also means a bullet's WAL is exactly its
term, an annuity due's WAL is one period shorter than the equivalent ordinary
annuity's, and `mid` sits precisely halfway between — all four asserted in
`tools/analytic-checks.py`.

**Time-weighted metrics net within an offset, not across one.** Two flows in
the same period at different points in it are not the same cash at the same
moment. A purchase settling on its date at period 0 is a full period earlier
than that period's collections, so it cannot cancel them; it simply is not an
inflow and does not enter WAL at all. Where every stream shares a placement
this reduces exactly to the net cash-flow series, which is what it was before.

`model.moic` deliberately does not use the axis — it is a ratio of cash in to
cash out over the life, and where inside a period the cash sits does not change
how much of it there is.

Two limits, both shared with discounting. The origin is the **model start**,
not a separately stated settlement date; and precision is period fractions
rather than actual days, so a WAL computed here will not tie to a published
Act/360 figure in the fourth decimal.

### Worked example

Five-year annual bond, 5% coupon on 1,000,000, bought at par at the start of
period 1, monthly calendar:

| Flow | Period | Offset | Discounted from |
|---|---|---|---|
| purchase (1,000,000 out) | 1 | 0.0 (`start`) | period 0 — undiscounted |
| coupon 1 (50,000) | 12 | 1.0 | end of period 12 |
| coupon 5 (50,000) | 60 | 1.0 | end of period 60 |
| principal (1,000,000) | 60 | 1.0 | end of period 60 |

Discounted at 5%, present value equals the price paid, so **NPV is exactly
zero**. A par bond discounted at its own coupon rate is worth par; that
identity is the acceptance test for this section and is enforced by
`tools/analytic-checks.py`.

## 4. `time.t` refers to accrual, not settlement

Inside an amount expression, `time.t` is the period the amount is being earned
in. Because a payment settles in the period that earned it (§2), accrual and
settlement periods coincide, and no distinction arises.

If a future convention ever separates them, the amount MUST still be evaluated
against the accrual period. Evaluating against settlement silently skips the
first accrual and shifts every subsequent amount by one period.

## 4a. Payment terms (normative)

A contract states when its cash moves relative to when it was earned:

```cfdl
contract energy.ppa.plant_a on entity project.plant {
  term 2026-01..2050-12
  payment net 45
  terms { ppa_price = 3000 }
}
```

Rules:
- `payment net <n>` applies to every stream the contract lowers. A schedule may
  state its own with `net <n>`, for the case where one contract's streams
  settle on different terms.
- A bare count is **days** — "net 45" means 45 days, as it does commercially.
  `net 6 months` steps by the calendar instead, because a six-month lag is six
  months and diverges from any day count once billing is not at a month end.
- Billing happens when a period **closes**, not when it opens: January's output
  is invoiced on 31 January, so net-30 falls in early March, not late January.
  A day rule (`on day 15`, `on eom`) names the billing date explicitly and
  overrides that.
- The due date is then rolled by the schedule's `convention` and `calendar`.
  It is the due date that moves off a weekend, not the bill.
- The amount is still evaluated against the **accrual** period. `time.t` in an
  amount expression refers to when the flow was earned, never to when its cash
  arrives.
- Several accruals may settle in one period — under net-30 both January and
  February land in March — and their amounts sum. Cash is delayed, never lost.
- Payment terms do not apply to `schedule on <date>`: a one-shot flow has no
  accrual period to settle after, and the attempt is rejected rather than
  ignored.
- A payment settling past the end of the timeline is rejected. Placing it in
  the final period would overstate that period.

### Discounting is at bucket granularity

A payment is discounted from the period it lands in. The fraction of a period
between that period's boundary and the actual due date is **not** modeled.

On a monthly grid at 12% that is worth roughly 0.5% on an affected flow, set
against the first-order effect — moving the cash two periods later — which is
captured. It matters more on a daily or weekly grid, where a 45-day term lands
near a bucket boundary anyway.

This is a stated convention, not an oversight. Removing it means giving each
payment its own discount offset rather than one per stream, which is a
separate change to `npv_with_offsets`.

## 5. Lags compose on top of placement

A lag — recovery lag, collection delay — is counted in periods from the period
that earned the flow, per §2.

A default in period 120 with a six-month recovery lag recovers in period 126.
The model must span period 126 for that cash to land; a flow scheduled past
the end of the timeline is rejected by `E2103_SCHEDULE_OUT_OF_BOUNDS` rather
than silently dropped.

"The end of the timeline" means the cash horizon **plus** any `project <n>`
tail, because the engine evaluates streams over both. A schedule may therefore
reach into the tail deliberately — that is how a forward-NOI exit reads a year
past the sale.

Cash that *settles* in the tail is a different matter. The tail is computed for
series lookups and excluded from cash results and NPV, so a payment pushed
there by its terms — a schedule ending on the horizon under net-60 — is
excluded from the totals. That would be a silent drop, so the engine warns and
names the amount. Extend `for <n>` to cover the lag, or shorten the schedule.

## 6. What packs declare

A lowering rule sets `schedule_due = true` when the stream behaves like an
expense — it falls due in the period it belongs to. Revenue, opex, rent,
recoveries, capex, working capital and tax attributes are all of this kind.

Streams that behave like an annuity — debt service, coupons, loan-pool
collections — take the default and are discounted from period end.

## 7. Verification

`tools/analytic-checks.py` asserts identities that follow from the definition
of present value, so they hold for any correct implementation and cannot be
satisfied by matching whatever the engine currently does:

- a par bond discounted at its coupon rate is worth par;
- a level annuity matches `(1 - (1+i)^-n) / i`;
- an annuity due is worth exactly `(1+i)` times the ordinary annuity;
- a fully-amortizing loan discounted at its own rate is worth its principal.

The third is the direct test of this section. The benchmark suite compares
each model against a reference implementation, which cannot catch a convention
both sides share — that is how the original defect survived eight passing
benchmarks.
