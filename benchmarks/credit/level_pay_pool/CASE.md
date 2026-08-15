## The case

A $25m auto loan pool at 6.5% over 120 months, bought at a one-point discount.
Every borrower pays the same instalment each month, which splits into shrinking
interest and growing principal. Layered on top: an 8% constant prepayment rate, a
2% default rate, 35% loss severity, a six-month recovery lag, a 50 basis point
servicing strip and a 1% prepayment penalty.

## The reference

Level-payment pool conventions as defined by the standard market formulas for
amortizing collateral — the same definitional source the mortgage cases use.

**Not redistributable.** The source cannot be published, so its conventions are
recreated independently of the model and compared month by month.

The pack lowers this contract to closed-form pool-factor expressions, and the
comparison is against a month-by-month recursion of the same convention.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_level_pay`, `credit.purchase` |
| Language features | a pack contract paired with a purchase price |
| Conventions | level-pay amortization, CPR, CDR, loss severity, recovery lag, a servicing strip, a prepayment penalty, purchase at a discount |

## The result

Present value **−295,975.22**, multiple on invested capital **1.225381** and
weighted average life **3.84394 years**.

Asserted: net cash flow per period across 120 months, plus the three summary
figures.

## The delta

None: every period agrees inside a one-cent tolerance. The weighted average life
and multiple carry a basis-point tolerance, since both are computed from an
iterative root rather than a closed form.
