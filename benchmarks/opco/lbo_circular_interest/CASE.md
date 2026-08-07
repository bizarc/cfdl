## The case

A sponsor buys a mid-market business for $720mm — 8.0x an LTM adjusted EBITDA of
$90mm — funded with a $275mm term loan B, $175mm of senior notes, $100mm of
subordinated notes that pay in kind for three years, a 5% management rollover and
$158.9mm of sponsor equity. The model runs the four-year hold: a 35% tax rate, a
$5mm minimum cash balance, 1% mandatory term loan amortisation, and every
remaining dollar of free cash flow sweeping against the term loan.

The case is the debt schedule. Interest accrues on the **average** of each
period's opening and closing balance, which is the standard convention and the
reason an LBO is usually said to need an iterative solver: interest depends on
the closing balance, the closing balance depends on how much cash swept the debt
down, and that cash is net of interest.

## The reference

A seven-step leveraged buyout teaching model published as a downloadable
spreadsheet, free and without registration. It solves the same schedule **by
iteration** — it ships a `CIRC` switch that turns on the spreadsheet's iterative
calculation.

It publishes a complete cash flow table: every balance, every interest line and
every cash figure, as cached values in the workbook, so the comparison is period
by period rather than against a single answer.

**Not redistributable.** The workbook carries an "All Rights Reserved" notice and
no open licence, so it is neither vendored nor wired into the test suite. It was
downloaded once outside the repository and only its output numbers were carried
across.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Declared | five curves, four states, five native streams |
| Language features | declared state with `init`/`next`, curves read by `curve_value`, native streams |
| Conventions | average-balance interest, payment-in-kind accrual, a floating rate off a published path, a 100% cash sweep |

The four states carry the debt balances: the term loan and the subordinated notes,
each with its opening value, so a stream can see both ends of a period.

## The result

**Exact.** Against the reference's own unrounded cached values, the closed form
agrees to **2.8e-14** — machine epsilon — across all sixteen balance and interest
figures.

| year | term loan balance | reference | term loan interest | reference |
|---|---:|---:|---:|---:|
| 2017 | 238.517440443 | 238.517440443 | 8.986555208 | 8.986555208 |
| 2018 | 199.519287769 | 199.519287769 | 8.979752928 | 8.979752928 |
| 2019 | 156.762561123 | 156.762561123 | 8.016341600 | 8.016341600 |
| 2020 | 120.484780576 | 120.484780576 | 7.139119049 | 7.139119049 |

Asserted: the term loan and subordinated note balances, four interest lines and
the repayment, across four years — 33 figures in total.

## The delta

There is no arithmetic delta. The largest figure anywhere in the case is
**4.5e-7**, on the final year's repayment line, and it is the engine's own
publication precision rather than a disagreement: results carry money to six
decimal places, so half of that is the tightest any case here can assert.

What the exactness establishes is narrower than "an LBO needs no solver". The
loop is **linear** in the closing balance — every step is affine in it, with no
products of unknowns and no thresholds — so collecting terms solves it in one
substitution, which is what the model's `next` clause does. That holds because no
constraint binds in this deal: the revolver is never drawn, the term loan never
fully repays, and minimum cash is exactly met. A deal that hit any of those would
be piecewise linear, which is a different problem.
