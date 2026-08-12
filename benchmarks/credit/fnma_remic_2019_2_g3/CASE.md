## The case

Fannie Mae REMIC Trust 2019-2 divides $307,727,958 across three groups. Group 3
is $148,372,434 of Fannie Mae mortgage-backed securities, and its priority of
payments is a single line:

> The Group 3 Principal Distribution Amount to AB until retired.

Which makes it sound like nothing to model. The interest is where the deal is.

The pool passes through at **5.00%**. Class AB takes **3.25%** of that, and the
remaining **1.75%** is sold separately as Class IO — an interest-only class with
no principal balance at all, entitled instead to 5.00% of a *notional* balance
set at 35.0000000674% of AB's. The two reconstruct the pass-through rate:

```
3.25%  +  0.350000000674 x 5.00%  =  5.00000000337%
```

An investor in IO owns a slice of a coupon and nothing else. Their position
shrinks as AB amortises, they receive not one dollar of principal, and if the
loans prepay quickly they lose most of what they paid.

## The reference

The Prospectus Supplement dated 24 January 2019, page S-14, which publishes for
Classes AB and IO the percentage of the original balance outstanding after each
January's distribution for thirty years, at seven prepayment speeds, with a
weighted average life for each. See `SOURCE.md`.

This case takes the 198% PSA column, the pricing speed.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_level_pay` |
| Language features | two waterfalls over one collateral, one for principal and one for interest; entity fields carrying class balances |
| Conventions | PSA on a pool seasoned past the ramp, a servicing and guaranty strip, a stripped coupon, a notional interest-only class |

**This is the first case in the repo where a coupon is stripped.** Every other
credit case pays interest at the rate the asset earns. Here three rates are in
play at once and none of them is the mortgage rate:

- the loans carry a **5.451%** weighted average coupon;
- **0.451%** is the servicing and guaranty strip, carried as `servicing_fee`, so
  what reaches the trust is 5.00% exactly;
- that 5.00% is then split 3.25% / 1.75% between a principal class and a
  notional one.

The interest waterfall is the test. It declares a residual step, and if the
strip is right that step takes nothing:

```cfdl
pay ab_interest to party.ab_holders = asset.ab.balance * (0.0325 / 12.0)
pay io_interest to party.io_holders = asset.io.balance * (0.05 / 12.0)
pay residual    to party.residual   = remaining
```

**The residual is zero in all 361 periods, to ten decimal places.**

A note on why the class balances are fields at all. AB is a pass-through, so its
balance is the pool's, and IO's is a fixed fraction of AB's — neither carries
state of its own, and neither is copied. The field says what the class *is*
(`next prev.asset.pool.balance`) and derives the number. That the balances land
one period behind the distributions is not a workaround here but the deal's own
convention: the supplement strikes interest on "the outstanding balance of that
Certificate immediately prior to that Distribution Date", which is precisely the
number these fields hold.

## The result

**Worst disagreement 0.3640 percentage points** across 30 published cells,
against a floor of 0.5 set by the table's whole-percent rounding.

| | |
|---|---|
| Decrement, worst / mean | 0.3640 pp / 0.0841 pp |
| Informative cells (published value neither 0 nor 100) | 14 of 30 |
| Weighted average life | **4.696 → 4.7**, published **4.7** |
| Residual to Classes R and RL | **0.0000000000**, every period |
| Principal returned to AB | 148,372,434.00 against an original of 148,372,434 |

Over the 14 informative cells the mean error is 0.1803 pp — the signature of the
issuer's rounding and nothing else.

The interest legs are asserted too, and they are external rather than model
output: a published balance multiplied by a coupon stated on the cover. Their
tolerance is the balance band carried through the coupon.

The weighted average life is asserted at ±0.07: 0.05 is the print floor of a
figure published to one decimal, and ~0.015 is the axis — the engine measures
on its month-end axis while the deal distributes on the 25th measured from
late-January settlement, a uniform bias across all seven published speeds.

The other six published speeds — 0%, 100%, 300%, 400%, 700% and 1000% — each
ship as their own case (`fnma_remic_2019_2_g3_psa000` through `_psa1000`),
asserting their own decrement columns and weighted average lives, including
0% PSA, which the supplement prepares on its own alternative assumption of a
360-month original and remaining term at 7.50%.

## The delta

**Group 3 of three.** Groups 1 and 2 are Structured Collateral: their assets are
seventeen tranches of other Fannie Mae REMICs issued between 2002 and 2006. The
instrument is fully specified for those groups too — one line each, the same as
this one — but the cash arriving at them is another instrument's output, and
reproducing the published tables would need those seventeen deals' own
collateral. That is a compositional boundary, not a gap in this document, and it
is why only Group 3 is here.

**One speed here, seven in all.** This case ships the pricing speed; the other
six columns are sibling cases, so a convention error that hides under the
rounding floor at one speed has to hide at all seven simultaneously.

**No losses.** Fannie Mae guarantees timely payment of principal and interest,
so the collateral cannot default in a way the classes would see.

**Seasoned past the ramp.** Weighted average loan age is 175 months, so 198% PSA
is a flat 11.88% CPR in every period. The ramp is written out in full anyway —
the model should say the pool prepays at 198% PSA, not at 11.88% CPR, because
the second is a consequence of the first and stops being true if the seasoning
changes.

`model.total` is a regression anchor from this model. Every other assertion is a
published figure or derived from one.
