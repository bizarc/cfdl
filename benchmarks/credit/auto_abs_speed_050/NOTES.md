# Auto ABS collateral at 0.50% ABS

## The source

The same issuer exhibit as `benchmarks/credit/auto_abs_wal`: a weighted-average-life
exhibit filed with the securities regulator, publishing for six note classes, at
seven prepayment speeds, for every monthly distribution date, the percent of that
class still outstanding. Public record, freely readable, not redistributable, so
it is not vendored.

`auto_abs_wal` takes the zero-speed column. This case takes the 0.50% column.

## The result

Pool principal collections against the published Class A-2 column:

| distribution date | CFDL | published | diff |
|---|---|---|---|
| 10/15/18 | 84.645 | 84.65 | -0.0048 |
| 11/15/18 | 69.389 | 69.39 | -0.0013 |
| 12/15/18 | 54.231 | 54.23 | +0.0012 |
| 01/15/19 | 39.173 | 39.17 | +0.0035 |
| 02/15/19 | 24.216 | 24.22 | -0.0040 |
| 03/15/19 | 9.360 | 9.36 | -0.0004 |
| 04/15/19 | 0.000 | 0.00 | +0.0000 |
| 05/15/19 | 0.000 | 0.00 | +0.0000 |

**Worst disagreement 0.0048 percentage points.** The exhibit rounds to 0.01, so
0.005 is the floor a reader can check against.

Published Class A-2 weighted average life at this speed: 0.32 years.

## Why this column was unreachable before

The exhibit uses the Absolute Prepayment Model: a constant fraction of the
ORIGINAL number of receivables prepays each month. The pool shrinks while the
denominator does not, so the implied single-month mortality rises over the life:

    SMM(t) = ABS / (1 - ABS * (t - 1))

`k` is therefore not constant, and every pool factor in the pack was
`pow(k, p)`, which is the closed form of the product only when it is. The
balance under a ramp is a running product with no elementary closed form. That
is `docs/13_feature_backlog.md` 2.1, closed by declared state variables.

## The finding: ABS is indexed from ORIGINATION

`t` counts from the loan's origination, not from the deal's closing. This pool is
seasoned — the exhibit states a weighted average age per sub-pool, running 11 to
42 months — so at the first distribution it is already part-way up the curve.

Measured both ways against this exhibit:

| speed | age-indexed | closing-indexed |
|---|---|---|
| 0.50% ABS | 0.005 pp | 1.955 pp |
| 1.50% ABS | 0.004 pp | **19.960 pp** |

The pack's ramp terms had been written against months-since-closing. `age_months`
now carries the seasoning, and PSA and SDA take it too — all three conventions
are indexed by loan age. A pool at age 0 is unaffected, which is why no committed
golden moved.

This is the fourth defect in this programme found only because the reference was
external. An in-house reference would have carried the same reading of `t`.

## What the exhibit states, and what is assumed

Stated: no defaults, losses or repurchases; payments on the last day of each
month with 30-day months; the clean-up call not exercised; Class A-1 paid in full
on 16 January 2018, before the first distribution date shown, so Class A-2
receives 100% of pool principal for the whole of its life at every speed.

Fitted: one constant, the Class A-2 initial balance of 112,026,000, which the
exhibit does not state. It is the same constant `auto_abs_wal` uses and it is
speed-independent. One constant has to fit every point of every speed column at
once; a scale factor moves a curve up and down but cannot change its shape.
