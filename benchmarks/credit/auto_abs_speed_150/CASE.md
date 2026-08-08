## The case

The same subprime auto receivables as the zero-speed case, prepaying at
**1.50% ABS**. Under the absolute prepayment speed convention, prepayments are a
constant share of the *original* pool each month, so the balance follows a
running product rather than a constant hazard — and the collection profile
shortens as the speed rises.

## The reference

An issuer's own prepayment-speed exhibit, filed publicly with a securities
regulator. It tabulates the Class A-2 percent-outstanding at every monthly
distribution date, for each of seven prepayment speeds.

**Not redistributable.** Public filings are freely readable and citable, but the
filer retains copyright, so figures are asserted against rather than reproduced.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_level_pay`, 43 instances |
| Language features | many instances of one contract type; a per-period pool factor carried as state |
| Conventions | absolute prepayment speed, level-pay amortisation, a promotional 0% rate |

The pool factor is carried as per-period state: under a speed that is a share
of the original pool, the surviving balance is a running product, which a
closed-form `pow(k, p)` gives only while the hazard is constant.

## The result

Principal collections reproduce the exhibit's published percent-outstanding
column at every distribution date, worst **0.0036 percentage points** against a
source that rounds to 0.01.

The aggregate pool balance is asserted directly:
`domain.credit.principal` = **537,640,787.96**.

## The delta

The 0.0036-point residual is inside the exhibit's own rounding: it publishes to
two decimal places, so a difference smaller than 0.01 cannot be distinguished
from the printed figure.
