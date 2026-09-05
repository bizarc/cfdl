## The case

Subprime auto receivables backing a securitization, measured at zero prepayment
speed. The collateral is 43 level-pay sub-pools at 43 different rates and terms,
four of them at a 0% promotional annual rate. Weighted average life is the
standard summary of when principal comes back.

## The reference

An issuer's own prepayment-speed exhibit, filed publicly with a securities
regulator. It states the aggregate pool balance and tabulates percent-outstanding
at every monthly distribution date across seven prepayment speeds.

**Not redistributable.** Public filings are freely readable and citable, but the
filer retains copyright, so figures are asserted against rather than reproduced.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.loan`, 43 instances |
| Language features | many instances of one contract type in a single model |
| Conventions | level-pay amortization, a promotional 0% rate, zero prepayment speed |

## The result

The aggregate pool balance reproduces the issuer's stated figure to the cent:
`domain.credit.principal` = **537,640,787.96**, on a tolerance of one cent
against a balance of half a billion dollars.

Reproducing that means all 43 sub-pools returned exactly the balance the issuer
stated, at 43 different rates and terms.

## The delta

None on the asserted figure.

One reconciliation is not expressible through this suite and is recorded
separately: the exhibit's percent-outstanding column is a percentage of a note
class, and this pack models the collateral rather than the liability stack. The
sister cases at 0.5 and 1.5 ABS carry that comparison.
