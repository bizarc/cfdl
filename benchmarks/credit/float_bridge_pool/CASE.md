## The case

A $15m floating-rate bridge pool. The coupon is a benchmark rate plus 275 basis
points, resetting each period off a stepped forward curve, with a 7.00% floor
that binds in the early periods. It runs 36 months to a bullet, bought at par,
against a 10% prepayment rate, a 2.5% default rate, 45% severity and a five-month
recovery lag.

## The reference

Floating-rate pool conventions as defined by the standard market formulas —
coupon reset off an index, with a rate floor applied before the spread.

**Not redistributable.** The source cannot be published, so its conventions are
recreated independently of the model and compared month by month.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_float_io_bullet`, `credit.purchase` |
| Language features | a declared `curve` read per period by `curve_value` with step interpolation |
| Conventions | a coupon that resets rather than fixing at origination, a binding rate floor, a bullet maturity |

Curves are exercised end to end here: the curve statement, its representation
in the compiled model, and the per-period lookup.

## The result

Present value **−433,719.03**, multiple on invested capital **1.151953** and
weighted average life **2.367044 years**.

Asserted: net cash flow per period across 36 months, plus the three summary
figures.

## The delta

None: every period agrees inside a one-cent tolerance, including the periods
where the floor binds and the coupon stops tracking the curve.
