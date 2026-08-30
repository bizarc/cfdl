# Notes — fund_gp_lp_waterfall

Working notes, not published. Reader-facing prose is in `CASE.md`; the
tolerances and their reasons are in `case.toml`.

The source publishes its figures, so those figures are asserted directly and
this file records the reconciliation — the `docs/benchmarks` first case, not
the `reference_gen.py` one. A generator was written here first and removed: it
is for sources that do NOT publish, and this one does.

## What was compared

The source's `Waterfall` sheet publishes, per month, each partner's amount in
each of the five tiers, and an `Investment Summary` giving each partner's total
by tier, its total return and its annual return. All of it is asserted.

`inputs/fund_cash_flow.csv` is the sheet's own monthly cash flow column, and
`inputs/terms.json` its stated economics. Nothing else is taken from it.

## How to repeat it

Read the workbook's `Waterfall` sheet. Columns 64-71 carry the LP's amount by
tier and 73-80 the GP's, one row per month from row 36. The summary is on
`Fund Inputs`, rows 31-41, columns 7-15. Every asserted figure is one of those
cells; `expected_metrics.json` names the cell in each `source` field.

## The two recovered conventions

Neither is stated anywhere in the workbook, and both were settled by measuring
the alternatives against its published figures.

**The preferred accrues on capital outstanding at the OPEN of the period.** The
formula reads `=(M46*$G$7)+S45`, where M is the period's CLOSING equity — which
is what was implemented first, and it is wrong: 2,703,238 against a published
2,902,693, disagreeing in one month of thirty. Accruing on the opening balance
agrees in all thirty and to the cent. The formula's M is evaluated before the
period's repayment lands, which is not visible from the cell reference.

**Each hurdle pays a partner up to that partner's own shortfall.** The tier
splits (50/50, 45/55, 40/60) cap what each partner may draw; they do not
allocate. Because neither cap ever bound at the fund level, the realised splits
came out 90/10 — the ratio of the partners' capital — and a model that read the
stated splits as the allocation would be wrong by about 2.4 million.

Tested and rejected: that an unused share spills to the other partner. It makes
the error fifteen times worse (773,448 against 50,181).

## Building the model: two bugs worth remembering

Both were invisible in the totals and obvious in the state series.

**A contribution is not a distribution.** A party account holds its capital as
a negative inflow, so differencing the account to find "what was paid last
period" reads the capital call as cash paid out. The hurdle balance doubled at
the first period. The fix is the add-back Highlands uses.

**A hurdle balance is seeded at the contribution, not at the contribution
already grown.** Seeding it one accretion ahead compounds quietly for
twenty-six months and shifts about 225,000 between tiers.

## The residual

$1.36, largest single divergence, on 37,073,982.80 allocated. The workbook
rounds the cash-flow vectors it runs its return test against to whole dollars
(`=ROUND(-G46-J46-Q46,)`). Rounding here does not close the gap — total
absolute error 50,195 rounded against 50,181 unrounded — so the model keeps the
unrounded figures and `period_tolerance` carries it.

## The GP re-split, and why it is not in this case

The source runs a SECOND waterfall over the general partner's own proceeds,
dividing them between two classes on hurdles of 10, 15, 20 and 100 per cent.
Its pot is exactly what the fund pays the general partner: 6,658,530.611815,
tying to zero difference, which is the identity that says the two levels are
one flow and not two.

It is left out because it does not yet reconcile. Totals come within 0.018%
(4,300,305 against 4,301,064) but individual tiers drift, and the cause is
known and not yet modelled: the second waterfall's preferred is not paid down
as cash arrives. It accrues to its full 359,750.90 and clears in one month,
2019-11, and nothing reaches its hurdle tiers until it does. The first
implementation paid it progressively, which released cash to the hurdles two
months early.

That is a sequencing condition, not arithmetic. Rounding and arithmetic mode
were both tested and neither is the cause — and the error's shape says so
independently: it is one discrete jump of 22,157 in a single month, not a drift
spread across twenty-nine.

Adding it means a second waterfall drawing on the first's general-partner steps,
and two more party accounts. The fund level stands on its own without it.
