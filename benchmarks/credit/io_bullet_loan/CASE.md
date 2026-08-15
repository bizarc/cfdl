## The case

A $10m interest-only bridge pool at 7.25% for 60 months, bought at par. Nothing
amortizes: the borrower pays interest monthly and the entire principal returns in
a single balloon at maturity. Against that sit a 5% constant prepayment rate, a
1.5% default rate, 40% loss severity and a four-month recovery lag.

## The reference

Interest-only and bullet-maturity conventions as defined by the standard market
formulas for non-amortizing collateral.

**Not redistributable.** The source cannot be published, so its conventions are
recreated independently of the model and compared month by month.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_io_bullet`, `credit.purchase` |
| Language features | a pack contract paired with a purchase price |
| Conventions | interest-only accrual, a bullet maturity, CPR, CDR, severity, recovery lag |

With no scheduled amortization, weighted average life is driven entirely by
prepayment and default.

## The result

Present value **−61,370.42**, multiple on invested capital **1.286054** and
weighted average life **3.864922 years**.

Asserted: net cash flow per period across 60 months, plus the three summary
figures.

## The delta

None: every period agrees inside a one-cent tolerance.
