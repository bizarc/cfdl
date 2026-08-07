## The case

A sponsor buys a mid-market business for $720mm — 8.0x an LTM adjusted EBITDA of
$90mm — holds it five years and sells at the same multiple. Revenue grows
5, 6, 7, 6 and 5 per cent; margin, depreciation and capital expenditure hold at
their trailing ratios; working capital turns on stated days.

The same deal is run at **three capital structures**. Only the financing
changes:

| | Term Loan B | Senior Notes | Sub Notes | Total |
|---|---|---|---|---|
| Base | 3.0x @ L+3.00% | 2.0x @ 7.0% | 1.0x @ 8.5% | 6.0x |
| High leverage | 3.0x @ L+3.50% | 2.5x @ 7.0% | 2.0x @ 10.0% | 7.5x |
| Low leverage | 3.0x @ L+2.75% | 1.5x @ 6.0% | — | 4.4x |

The subordinated notes pay in kind for three years. Every dollar of free cash
flow after a 1% mandatory amortisation sweeps against the term loan, and
interest accrues on the average balance — so the balance depends on the interest
that depends on the balance.

## The reference

A seven-step leveraged buyout teaching model published as a downloadable
spreadsheet, free and without registration. It carries its own financing-case
switch, and publishes a five-year multiple and return for each of the three
structures across a grid of entry and exit multiples.

**Not redistributable.** The workbook carries an "All Rights Reserved" notice
and no open licence, so it is neither vendored nor wired into the test suite. It
was downloaded once outside the repository and only its output numbers were
carried across.

It publishes a period-by-period debt schedule for **Base only**. For the other
two structures it publishes the returns and nothing in between.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Declared | two states, five curves, two native streams, three run scenarios |
| Language features | **run-config scenarios**, `cfg.*` parameters, declared state with `init`/`next`, curves |
| Conventions | average-balance interest, payment-in-kind accrual, a 100% cash sweep, tranche sizing to a debt increment, a sponsor cheque struck as the plug |

The financing case is the **run config**, not the model: the deterministic run is
Base and two scenarios override the tranche sizes, coupons and the sponsor's
cheque. That is what the source's own case switch does.

Sizes are not stated as inputs. Each tranche is its leverage multiple times LTM
EBITDA rounded to a $25mm increment, and the sponsor's cheque is whatever
balances sources against uses. Base is what checks the rule — its published
$275mm, $175mm and $100mm are what 3.0x, 2.0x and 1.0x round to — and the other
two structures are then derived rather than transcribed.

## The result

**All three structures reproduce the published multiple and return.**

| | MoIC | reference | IRR | reference |
|---|---:|---:|---:|---:|
| Base | 2.952823 | 2.952823 | 24.1788% | 24.1788% |
| High leverage | 5.479046 | 5.479046 | 40.5209% | 40.5209% |
| Low leverage | 2.271875 | 2.271875 | 17.8357% | 17.8357% |

Worst disagreement across all six figures: **4.5e-7**.

Base additionally asserts the term loan and subordinated balances period by
period; the term loan agrees at the engine's own publication precision across
all five years.

What makes the two scenarios worth having is that **nothing between the inputs
and the answer is anchored for them**. The operating build, the sizing rule, the
sweep, the PIK accrual, the exit and the returns arithmetic all have to be
right to land on a published multiple. Base anchors every intermediate line;
these two anchor none, and they land anyway.

## The delta

There is no arithmetic delta on the returns.

One column carries a looser tolerance, and the reason is the reference. The
subordinated PIK accrual is self-referential, and the source solves it by
switching on iterative calculation where this model solves it in closed form.
Checked against the reference's own equation, `B = B0 + avg(B0, B) * r`:

| | residual |
|---|---:|
| closed form | −1.4e-14 |
| reference | +3.7e-05 |

The source stopped iterating while its own equation still had a residual of
3.7e-5, so that is what its convergence supports and the column is asserted to
1e-4. The closed form is the more accurate of the two, which is the same finding
a companion case reached at 2.8e-14 on a shorter hold.

One thing the case does **not** cover: the reference publishes a full 5×5 grid
of entry and exit multiples for each structure — 150 figures. This asserts the
8.0x / 8.0x corner of each. The rest needs one scenario per grid point, which is
a matter of generating them rather than of anything being unresolved.
