# credit pack v0.1

Credit / lending pack: fixed-rate loan pools with CPR prepayments, CDR
defaults, loss severity and a recovery lag. Benchmarked in
the [credit benchmarks](/docs/benchmarks) against independent month-by-month reference
implementations.

> **Supported calendars: all of them.** Two distinct daily shapes both work,
> and the difference matters:
>
> 1. **A daily book that pays monthly** — the ordinary mortgage or ABS pool.
>    Declare `payment_frequency = "month"` and periods-per-year comes from the
>    payment rhythm (12), not the calendar (365). A 30-year mortgage on a daily
>    book is still 360 payments, not 10,950. Verified exactly: the same pool on
>    a 39-period monthly grid and an 1186-period daily book agrees to the cent
>    on every stream.
> 2. **Genuinely daily accrual** — warehouse lines, repo, revolvers,
>    daily-reset floaters. Leave `payment_frequency` unset and ppy is 365.
>
> `term_months` must divide into whole payment periods. For daily accrual that
> means a tenor that is a multiple of 12 months (360 months → 10,950 days);
> otherwise `E5015_TERM_MONTHS_NOT_DIVISIBLE` names the nearest legal values.
>
> Note the two rate conventions, which must not be confused. The note rate,
> servicing strip and float margin are **nominal** and divide by ppy. CPR and
> CDR are **effective annual** and take a root — `cpr_to_periodic(x, ppy)`,
> which is exactly `cpr_to_smm(x)` at ppy = 12.
>
> **Day count is selectable** via a `day_count` term: `30/360` (the default,
> and what every existing model gets), `30e/360`, `act/360` or `act/365`. It
> applies to the note rate, the servicing strip and the floating
> index-plus-margin — every nominal rate in the pack.
>
> Under `act/360`, 6% on 1,200,000 accrues 6,200 in a 31-day January and 5,600
> in February, against a flat 6,000 under 30/360, and 73,000 over a 365-day
> year rather than 72,000. That 365/360 uplift is the convention's whole point,
> and it is the USD credit default. A misspelling is `E5019_UNKNOWN_DAY_COUNT`,
> not a silent fallback.
>
> **Amortization has its own day count.** A level-pay pool strikes its payment
> once and then accrues interest period by period, so `amortization_day_count`
> selects the basis the payment is struck from and defaults to `day_count`.
> Setting `day_count = "act/360"` with `amortization_day_count = "30/360"` — the
> common US commercial case — holds the payment constant while interest varies
> with month length and scheduled principal absorbs the difference. Setting only
> `day_count` leaves every existing model unchanged. Applies to
> `credit.loan`; IO/bullet contracts have no amortization to strike.
>
> Annual totals do **not** match across monthly / quarterly / annual, and
> should not: nominal accrual means a 6% loan is 0.5%/month and 1.5%/quarter,
> which are different instruments. Those cadences are checked against the
> benchmark reference generators rather than against each other.

## Conventions, and where they come from

### Ramped hazards

`cpr` and `cdr` are flat for the pool's life. Real conventions are not: a loan
prepays slowly when new and faster as it seasons. Three terms select a published
ramp instead, and each is a **multiple**, not a percent:

| term | curve |
|---|---|
| `psa_speed` | CPR rises 0.2%/month from month 1 to 6.0% at month 30, flat after |
| `sda_speed` | CDR rises 0.02%/month to 0.60% at month 30, flat to 60, declining to 0.03% at month 120, flat after |
| `abs_speed` | a constant fraction of ORIGINAL balance each month |

All three default to `0`, which selects the flat `cpr`/`cdr` path — so a model
written before they existed is byte-identical.

**All three ramps are indexed from ORIGINATION, not from closing.** A pool
bought at 24 months' seasoning is already two years up the curve on its first
distribution. `age_months` carries that; leaving it at `0` on a seasoned pool
understates prepayment — measured at **20 percentage points** of note balance by
month 4 against a published exhibit at 1.50% ABS.

**`abs_speed` is already a monthly rate.** `cpr`/`cdr` are effective *annual*
rates and take a root through `cpr_to_periodic`; the Absolute Prepayment Model
quotes a monthly figure directly, so it must not be converted. Conflating the
two is a factor-level error that no unit test would notice.

The ramp is what makes the balance a running product rather than `pow(k, p)` —
see the header of `lowering/rules.toml`.

Prepayment and default follow the market-standard MBS conventions for CPR,
SMM and the standard prepayment and default curves; the pack is checked for
parity against the published industry reference schedule.

- **SMM applies to the balance at the BEGINNING of the period, net of
  scheduled amortization only.** Defaults are not removed from the base.
- Because both attritions are drawn from that same base, survival is
  **additive**: `k = (1 - mdr) - smm`, not `(1 - mdr)(1 - smm)`.
- CPR and CDR are effective annual rates and convert by a root
  (`cpr_to_periodic`); note rates are nominal and convert by division.

The [mortgage pool conventions benchmark](/docs/examples/credit-mbs-pool-conventions) asserts anchor figures across the life
of a 30-year pool and passes, including recoveries — a level-pay pool's
defaulted balance keeps amortizing in foreclosure, so what is liquidated is the
amortized balance rather than face. Age-varying prepayment and default curves
ARE expressible: the balance is an account moved once per payment period,
not `pow(k, p)`, so PSA, SDA and the ABS convention all ramp correctly.

## Contract types

### `credit.loan`

A loan — or a representative loan standing for a pool of them. One type:
the repayment pattern is the `amortization` term and the coupon is fixed or
floating by which rate terms are stated, and the lowering rules select on
those (a floating level-pay loan has no row and is refused, `E1385`). A pool
of loans is not a type: it is a container the modeller names, holding one
`credit.loan` per loan or one representative loan, and its balance is the
fold of its members'.

**The balance is an account.** The loan opens `<subject>.balance` at
`principal`, due to the holder, and every row reads its opening as
`prev.balance`. Scheduled principal, prepayments and the bullet move it as
cash arrives; the default is written off in the period it occurs
(`credit.loan.defaults`, a non-cash row that is the gross loss); recoveries
arrive `recovery_lag_months` later as cash against a claim already gone,
sized off a private lagged twin of the balance. Closing is opening less
those three movements, which is opening times (1 − c) times
(1 − mdr − smm): the recurrence the former survival factor carried, so the
account changes no number. It publishes as `account.<subject>.balance`.

**Level pay** (`amortization` unstated or `"level_pay"`). Interest and
scheduled principal on the opening balance net of the period's defaults,
prepayments on the opening less the scheduled principal (the convention
block at the top of `lowering/rules.toml`).

**Interest only** (`amortization = "interest_only"` or `"bullet"`). A
coupon on the opening balance net of the period's defaults, and the whole
surviving balance as the `credit.loan.bullet` at maturity; the final period
pays no SMM prepayment. `interest_rate` may be anything, including 0.

**Floating coupon** (`index_curve` stated instead of `interest_rate`).
Interest-only or bullet only. The coupon indexes off a model-declared
`curve`:

```cfdl
curve sofr {
  2026-01: 0.048
  2026-07: 0.045
}
```

`coupon = clamp(curve_value(index_curve, date) + margin, rate_floor, rate_cap)`;
the balance dynamics are the interest-only row's, rate-independent, so the
closed form stays exact.

Terms:

| term | meaning | default |
|---|---|---|
| `principal` | balance at the model's start | required |
| `term_months` | remaining term in months | required |
| `amortization` | `level_pay`, `interest_only` or `bullet` | `level_pay` |
| `interest_rate` | annual note rate, fixed coupon | one of `interest_rate` / `index_curve` |
| `index_curve` | name of a `curve` declared in the model, floating coupon | one of `interest_rate` / `index_curve` |
| `margin` | spread over the index | `0` |
| `rate_floor` | coupon floor | `0` |
| `rate_cap` | coupon cap | `1` |
| `cpr` | annual conditional prepayment rate | `0` |
| `cdr` | annual conditional default rate | `0` |
| `psa_speed` | multiple of the standard prepayment curve — `1.5` is 150% PSA | `0` |
| `sda_speed` | multiple of the standard default assumption — `1.0` is 100% SDA | `0` |
| `abs_speed` | Absolute Prepayment Model speed, already monthly | `0` |
| `age_months` | age at the model's start, in months | `0` |
| `severity` | loss severity on defaulted balance | `0` |
| `recovery_lag_months` | months from default to recovery cash | `0` |
| `servicing_fee` | annual servicing strip on performing balance | `0` |
| `prepay_penalty_rate` | flat penalty rate on voluntary prepayments | `0` |

Streams (all suffixed by contract instance): `credit.loan.interest`,
`credit.loan.sched_principal` (level pay), `credit.loan.bullet` (interest
only), `credit.loan.prepay`, `credit.loan.recoveries`,
`credit.loan.servicing` (outflow), `credit.loan.penalty`, and the non-cash
`credit.loan.defaults` (a write-off of the balance).

The penalty is a flat rate on prepaid balance (simplified yield
maintenance); a discounted make-whole needs an engine primitive and stays
on the parity worklist.

The contract `term` must span `term_months + recovery_lag_months` schedule
periods so the recovery tail has periods to land in; the expressions gate
themselves, so a longer term is harmless.

### `credit.purchase`

Acquisition price paid at `term_start` (`price` term), stream
`credit.purchase.price`. Discount/premium purchases are just a `price`
different from face (e.g. 99.0 = `0.99 * balance`); the level_pay_pool
benchmark purchases at a 1-point discount.

### `credit.participation`

A pro rata interest in the collateral's cash, passed through to the holder each
period — a Fannie Mae or Ginnie Mae pass-through, a Freddie Mac
participation certificate, a loan participation. The first refinement of
`Contract.Security` (`docs/40` §4.13): nothing is ranked and nothing is
chosen, so both lines are lowered from the collateral rather than allocated
by a priority of payments. Written from the issuer's seat: the two streams
are outflows from the pool entity, and the model's net is what the issuer
retains.

| term | meaning | default |
|---|---|---|
| `face` | the certificate's original principal | required |
| `share` | the holder's undivided share of the pool's cash, 0 to 1 | required |

Streams `credit.participation.interest` (the pool's interest net of
servicing, times the share; category `financing.debt.interest_paid`) and
`credit.participation.principal` (scheduled principal, the bullet,
prepayments and recoveries, times the share; `financing.debt.principal`) —
the same categories a borrower's debt service carries in every pack, because
IAS 7 classifies the activity, and the instrument is on the series' contract.
Prepayment penalties stay with the issuer. A participation carries the
SUFFIX of the pool it participates in: `credit.participation.smoke` reads
`credit.loan.*.smoke`.

### `credit.note`

A structured note: a class of a securitization — an ABS class, a REMIC
tranche — paid what the trust's priority of payments allocates it. The
second refinement of `Contract.Security`, and the opposite shape from the
participation: both lines are ALLOCATED, so the rules lower no cash. They
lower the two claims the waterfall's steps read, as fields on the trust.

| term | meaning | default |
|---|---|---|
| `face` | the class's initial principal | required |
| `coupon` | annual coupon (or `index_curve` with `margin`) | required, one way |
| `principal_account` | the declared account the holder's principal is paid into | required |
| `payment_frequency` | the class's payment rhythm | the calendar's |

Fields `credit_note_claim_<id>` (face less the account's prior balance) and
`credit_note_interest_due_<id>` (the claim times the coupon over the period)
on the trust. A step pays each line and says so:

```cfdl
pay a2_interest to account a2_interest for contract credit.note.a2 line interest
      = min(remaining, container.trust.credit_note_interest_due_a2)
pay a2_principal to party.a2_holders for contract credit.note.a2 line principal
      = min(remaining, container.trust.credit_note_claim_a2)
```

The step's series then carries the contract and the line, the results graph
lists the step under the note, and `type Contract.Security line principal`
reaches it. The Ally auto ABS benchmark case declares its seven classes this
way.

### Paid off, or repurchased

The `credit.loan` machine carries `on enter repurchased { set balance = 0 }`
and the same on `prepaid`. `balance` is the account the loan opens, so the
write is a WRITE-OFF of the whole opening balance, journaled as a `move`
from the machine; every row reading `prev.balance` that period reads zero,
and the lagged twin recoveries read is ended with it, so a repurchased loan
recovers nothing for its seller. A clean-up call repurchases the remaining
collateral with one status write per loan, or one event at the trust:

```cfdl
event clean_up_call when time.t == 6 {
  set entity asset.loan.status = "repurchased"
}
```

## Conventions

- Defaults leave the pool at the start of the period and earn no interest
  that period.
- `cpr`/`cdr` are annualized; converted with `cpr_to_smm(x) = 1 - (1-x)^(1/12)`.
- Prepayment applies SMM to the performing balance net of scheduled
  principal.
- Recoveries return `(1 - severity)` of defaulted face,
  `recovery_lag_months` later. Defaulted-balance write-offs are not cash
  and emit no stream.

## Metrics

`domain.credit.interest`, `.principal` (scheduled + prepay + bullet),
`.recoveries`, `.penalties`, `.servicing` (omitted when zero),
`.collections` (interest + principal + recoveries + penalties),
`.purchase`, `.collections_multiple` (collections / purchase), and
`.wal_years` — principal-weighted average life over principal returned
(scheduled + prepay + bullet + recoveries; interest/penalties excluded).

> **The WAL time axis.** A collection at the close of period `t` sits at
> `(t + 1)/ppy` years, not `t/ppy` — so a monthly pool's first collection is at
> one twelfth of a year. That matches the market definition, the one a
> prospectus states as "the number of years from the closing date to the
> related distribution date", and it is the same axis the pack's cash is
> discounted on. Reconstructed from an issuer-published schedule, the
> difference is not academic: on a short auto class with a published WAL of
> 0.37 years, measuring from period zero gives 0.286.
>
> Two limits worth knowing. The origin is the **model start**, not a separate
> settlement date — identical whenever the deal closes at t = 0, which every
> credit model here does. And precision is period fractions rather than actual
> days, the same convention as discounting, so this will not tie to a published
> Act/360 figure in the fourth decimal.

> **A 0% note rate works.** Promotional financing is ordinary in auto and
> retail credit — it is about 3% of the collateral in the published auto-ABS
> pool this pack is checked against. `credit.loan` amortizes it
> straight line with no interest. This was previously accepted-and-NaN rather
> than supported: the `interest_rate` validation asked only for non-negative, and the
> closed form is 0/0 at zero.

## Not in v0.1

- **Floating-rate loans** — needs the `curve` input concept and
  mean-reverting rate paths (stochastic roadmap item 4).
- Zero note rate on a level-pay `credit.loan` (closed form divides by `r`).
- Delinquency states and servicer advances.
- An Actual `amortization_day_count`, refused by `E5027`: a level payment is
  struck once and held, and a period-local divisor makes it move with month
  length. Accrue on `act/360` and strike the payment on `30/360`.

## Quick start

A $25m level-pay pool with prepayments, defaults, a servicing strip, and a prepayment penalty:

```cfdl
version 0.1
model "my-pool"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 126

entity asset buyer : Credit.Asset.LoanPool

contract credit.loan.auto_a on entity asset.buyer {
  term 2026-01..2036-06
  terms {
    principal = 25000000
    interest_rate = 0.065
    term_months = 120
    cpr = 0.08
    cdr = 0.02
    severity = 0.35
    recovery_lag_months = 6
    servicing_fee = 0.005
    prepay_penalty_rate = 0.01
  }
}

contract credit.purchase.auto_a on entity asset.buyer {
  term 2026-01..2026-01
  terms { price = 24750000 }
}
```

`credit.purchase` takes the dollar purchase amount — a 1-point discount
(99.0) on the $25m balance here.

Let the contract term span `term_months + recovery_lag_months` so lagged
recoveries have periods to land in.

## Run it

```bash
cfdl compile my-pool --packs packs --out my-pool/ir.json
cfdl run my-pool/ir.json --packs packs --pack credit --out my-pool/results.json --rate 0.08
```

Domain metrics include interest/principal/recoveries/collections, the
collections multiple, servicing, penalties, and principal WAL
(`domain.credit.wal_years`).

## Recipes

**Floating-rate IO bullet off a rate curve** (margin/floor/cap against a
model-declared curve):

```cfdl
curve sofr linear {
  2026-01: 0.050
  2027-01: 0.038
}

contract credit.loan.bridge on entity asset.buyer {
  term 2026-01..2028-12
  terms {
    principal = 15000000
    index_curve = "sofr"
    margin = 0.0275
    rate_floor = 0.065
    rate_cap = 0.095
    term_months = 36
    cpr = 0.05
    cdr = 0.01
    severity = 0.30
    recovery_lag_months = 3
  }
}
```

**IO pool with bullet maturity**: `credit.loan` — same loss
vocabulary, interest-only until the balloon.

Full worked models: the [level-pay auto pool](/docs/examples/credit-level-pay-pool),
`io_bullet_loan/`, `float_bridge_pool/`, and the loan-pool notebook in
`examples/notebooks/`.

## Stream categories

Every stream this pack emits declares a `category` — a dotted path rooted in the
cash flow statement's three sections — and aggregation reads that rather than
pattern-matching the stream's name.

`operating.collection.interest`, `operating.collection.principal`,
`operating.collection.prepayment`, `operating.collection.recovery`,
`operating.collection.penalty`, `operating.expense.servicing`,
`investing.acquisition.purchase`.

Collections sit under `operating`, not `financing`: for a lender, interest
received is operating revenue rather than a financing flow. That is the same
judgement an IFRS filer makes for a financial institution, and it is why CFDL
enforces only the root vocabulary and leaves the assignment to the pack.

`principal` covers both scheduled amortization and a bullet, since both retire
principal. `prepayment` stands apart because it is what a speed assumption
moves, and every published factor table reports it separately. `recovery` is not
principal: it arrives after a default, on its own lag, and a weighted average
life that treated it as scheduled would be wrong.

An unlisted category is `E5022`.
