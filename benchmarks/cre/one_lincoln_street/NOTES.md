# One Lincoln Street — construction period funding and interest

## The source

MIT OpenCourseWare 11.431J, Case Assignment 3: the proposed 36-storey, roughly
one million rentable SF office development in Boston's Financial District, with
15,000 SF of retail and a 725-space underground garage. A real, named
transaction, taught with its exhibits published.

CC BY-NC-SA 4.0, so the PDF is committed under `reference/`. That makes three
sources in this repo a reader can open directly — the others are the HUD
template and Damodaran's workbook.

The same course's Problem Set 1 is `benchmarks/cre/mit_rentleg_plaza`.

## Which exhibit, and why

Exhibit 5 is an eleven-year stabilised pro forma and is the obvious target. It
is not used, deliberately.

Its derived lines do reconcile internally — for 2004, effective gross income
56,059, net operating income 37,257 and property before-tax cash flow 37,100
each recompute exactly from their components. But the lease-level assumptions
that drive base rent, absorption and turnover vacancy, and the two
reimbursement lines are not published. Asserting Exhibit 5 would mean stating
most of it as input and then checking that the engine added it up, which is the
failure this programme keeps naming: a source that publishes results rather
than drivers validates arithmetic, not modelling.

**Exhibit 7 publishes both.** Three drivers — a sixteen-quarter draw schedule,
an 8.00% rate compounded quarterly, and a $110,738,000 net equity commitment —
and every line they produce, quarter by quarter across the whole build.

## The result

| line | worst | note |
|---|---|---|
| equity contribution | **0.00** | exact to the dollar, all 16 quarters |
| construction loan draw | **0.00** | exact to the dollar, all 16 quarters |
| construction interest | 480 | the exhibit rounds to whole thousands |

`period_tolerance = 500` is the rounding floor and is confirmed binding — at 400
the case fails.

The opening debt balances agree exactly too, though the harness cannot assert
them: they are a balance, not a flow, and `expected.csv` columns resolve to
streams. Same limitation as `docs/13_feature_backlog.md` 7.12.

## The mechanic: a depleting commitment

Equity funds the project first; the construction loan takes over only once the
commitment is exhausted. That happens **mid-quarter**, in 2001.4, splitting a
$29,430,000 requirement into $10,522,000 of equity and $18,908,000 of debt.

That split is not stated in the model. It falls out of a running total:

```
cum(t)        running total of required funding through t   <- a declared state
debt draw     min(required, max(0, cum - equity))
equity draw   required - debt draw
opening debt  max(0, cum - required - equity)
interest      (opening debt + debt draw / 2) * rate / 4
```

One state, and the rest is closed form. The halved draw is the exhibit's own
convention — funding "assumed to occur ratably throughout the quarter", so a
quarter's own draw earns half a quarter of interest.

**Interest is paid, not capitalised.** The exhibit's closing balance is opening
plus draw with nothing added, and the interest is funded from the equity budget
as a separate stated line ($16,312,000 of the gross $175,000,000). Capitalising
it would compound it and drift immediately.

## Why the metric tolerance is wider than the series tolerance

`domain.cre.debt_service` is asserted at 16,310,570 with a tolerance of 1,500,
against the exhibit's stated total of 16,312,000.

The exhibit's total is the sum of its **rounded** quarterly figures. The engine
sums the exact ones. The 1,430 difference is nine quarters of rounding
accumulating in one direction, not a modelling difference — each quarter is
individually within 480. Asserting the engine's exact sum against the exhibit's
rounded sum at a tolerance tight enough to look impressive would have meant
picking a number that hid the reason.

## What this case does not cover

The operating pro forma, for the reason above. Also the development budget
(Exhibit 4, $330,495,000) — it is a static cost breakdown rather than a cash
flow, and the funding schedule already carries its timing.

No CRE pack contract is exercised, and that is now the point rather than a
gap. This case builds the funding waterfall from primitives — a curve, a field
and three streams — so it proves the LANGUAGE expresses a depleting equity
commitment with no domain vocabulary at all. That is the stronger claim, and it
is why this case was not converted when the contract arrived.

`cre.construction_loan` has since shipped, and
`benchmarks/cre/one_lincoln_street_contract` is its twin of this case: the same
exhibit, the same figures, declared as one contract. It reproduces this model in
all 48 cells with zero difference. The pair is the assertion — the primitives
prove the deal is expressible, the contract proves the pack changed no answer.
If they ever disagree, the contract is wrong.
