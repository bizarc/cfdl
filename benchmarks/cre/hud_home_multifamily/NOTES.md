# HUD HOME Multifamily — the first source we could ship

## Why this one is different

Every other external case in this repo reconciles against a document that is
free to read and not free to republish. So they assert against published
*numbers* — which are facts — and record the reconciliation here without
vendoring anything, and shipped documentation says "parity with the market
convention" rather than naming a standards body.

This source is a work of the U.S. federal government, dedicated to the public
domain. The workbook is committed under `reference/`, the source is named
outright, and a reader can open the Operating Pro Forma tab and check every
figure in `expected.csv` themselves. No other case here can make that claim.

The deal is HUD's own populated Sample: a 20-unit HOME-assisted rental
development on a 29-year operating pro forma.

## The result

The model is monthly and the workbook is annual, so the comparison is the
results' annual rollup against the published pro forma. Worst disagreement
over all 29 years of each line:

| line | worst | what explains it |
|---|---|---|
| Gross potential rent | 0.48 | workbook rounds to whole dollars |
| Rent loss (vacancy) | 0.48 | same |
| Other revenue | 0.47 | same |
| Debt service (P+I+MIP) | 0.38 | the workbook rounds the annual payment |
| Mortgage insurance | **0.00** | exact |
| Replacement reserve | **0.00** | exact — was 4.35, see below |
| Total operating expenses | **0.00** | exact — was 12.26, see below |

And the metric that matters to a lender — debt service coverage, which HUD
publishes at four points to sixteen significant figures:

| | CFDL | published | difference |
|---|---|---|---|
| year 2 | 1.563096 | 1.575738079919937 | -1.3e-02 |
| year 5 | 1.51688 | 1.533776538708986 | -1.7e-02 |
| year 10 | 1.408339 | 1.433483451283151 | -2.5e-02 |
| year 14 | 1.288639 | 1.28872685681607 | -8.8e-05 |

Agreement to five decimal places on a ratio built from lines the workbook has
already rounded to whole dollars. The residual is entirely that rounding: the
workbook's DSCR cell is `-E24/E27`, and both of those are `=ROUND(...,0)`, so it
divides a rounded NOI by a rounded debt service (13,989) while CFDL divides the
unrounded ones (13,989.38 from the loan's terms).

**These are now assertions rather than a table.** This section used to end by
saying the four values were reproduced by hand and that nothing checked them —
the harness could reach per-stream cash and lifetime scalars, and a coverage
ratio is neither. `expected.csv` now carries `domain.cre.egi`, `domain.cre.noi`
and `domain.cre.dscr` straight from the Operating Pro Forma's rows 15, 24 and
102, at every anchor year rather than four.

Two things made that possible. The subtotals exist per period at all, as folds
over categories rather than named streams. And `case.toml` can set a tolerance
per column: the money lines need a twelfth of a dollar a month because the
workbook publishes whole dollars a year, the DSCR needs 1e-4 because it agrees
to five decimals, and a single number cannot express both — a shared 1.0 would assert nothing about the ratio
and a shared 1e-4 would fail every line above it.

DSCR is asserted at six of the eleven anchors and left blank at the rest. The
mortgage matures in year 14, after which the workbook's formula returns a
literal 100 — `IF(debt=0, 100, ...)`, a sentinel and not a ratio. CFDL publishes
null there, which is the honest answer, and the harness rejects a stated value
against a null rather than coercing it to zero. Asserting the sentinel would be
asserting Excel's error handling.

## The affordability cliff, which is the point of the case

HOME-assisted units are rent-restricted for the affordability period and revert
to market rents afterwards. The workbook carries both tracks side by side —
restricted at 153,881.28 and market at 220,320.00, each trending 2% — and
selects between them.

Gross rent therefore steps **199,062 → 290,708 between years 14 and 15**, a 46%
jump against a 2% trend on either side. A model with the right trend and the
wrong switch year reproduces thirteen years correctly and then diverges by 46%,
which is why years 14–17 are anchored and why the case would be nearly
worthless without them.

**The switch fires a year earlier than the label reads.** The assumptions tab
states a 15-year affordability period and "switch HOME unit rents to market
after: 15 years", but the workbook's own selection puts year 15 on the market
track — restricted years are 1 through 14. Verified against both published
tracks at every year. We follow the data, not the label, and `restricted_years`
is 14 in the model. This is the source's own convention and not a discrepancy
to chase.

## Finding — the workbook escalates by a RECURRENCE, and now we can

The two expense lines were the only ones that missed by more than rounding, and
they missed for a structural reason.

The workbook does not compute year *n* as `base × trend^n`. It computes it as
**last year's already-rounded figure times the trend**, rounding again. Verified
directly: of its four expense sub-lines, `Operations and Maintenance` and
`Taxes/Insurance/Reserves` reproduce exactly under that recurrence and under no
closed form.

`pow(1 + trend, t)` compounds exact decimals from the base instead, and rounding
does not commute with exponentiation, so the two paths separated a little more
every year — 12.26 on 204,655 at year 29, monotone in years compounded. That
residual, on one line, was the sole reason this case carried
`period_tolerance = 13`.

**Declared states express the recurrence directly** and both lines now reproduce
the published figures exactly over 29 years:

```cfdl
state opex_management {
  init inputs.opex_management
  next round_to(prev * (1 + inputs.opex_trend), 1)
}
```

The tolerance drops 13 → **0.5**, which is the theoretical floor: the workbook
publishes whole dollars, so half a dollar is the most an exact figure can differ
from its rounded print. Confirmed binding — at 0.4 the case fails.

### One state per sub-line, not one for the total

Modeling the total as a single rounded line closed most of the gap and left
**11.00**. The workbook rounds each of its four sub-lines *before* summing them,
and rounding the sum is different arithmetic — 12,607.5 rounds up on its own and
disappears inside a total.

So the model carries four states, seeded from the four published sub-lines
(37,413 / 37,925 / 12,300 / 14,863), and their sum is the expense stream. The
102,501 total is now only ever an output. That is what took the line from 11.00
to 0.00, and it is the sort of thing that reads as noise unless the mechanism is
exactly right.

This is one half of the acceptance test for `docs/14_state_and_recurrence.md`.
The other is `benchmarks/opco/damodaran_fcff` — an unrelated source, an
unrelated pack, a multiplicative growth path rather than a rounded escalation.
Two independent published sources confirming one mechanism.

## The calendar is monthly, and the mortgage is the pack's

This case was written on an annual calendar, because the workbook is an
annual pro forma and the harness asserts at the model's period. That was the
presentation grain, not the instruments': the mortgage pays monthly, the
rent roll is monthly and the vacancy reads the rent. A monthly-paying loan on
an annual calendar is refused (`E2108_SCHEDULE_FINER_THAN_CALENDAR`), which
this file once recorded as the blocker to using `cre.permanent_debt`. The
refusal was right and the calendar was wrong.

The model now runs monthly. Every published line is level within a year, so
each month carries a twelfth of the year's figure and the monthly coverage
ratio is the annual one; `expected.csv` anchors the January of each anchor
year at the published annual line over twelve, and the results' annual
rollup reproduces the pro forma rows themselves (the table above is read
from it). Nothing in the language, the engine or the harness changed to make
that possible: the annual view of a monthly model was already a section of
the results.

**The loan is stated from its terms.** The First Mortgage Sizing tab states
$150,000 at 4.00% over a 15-year term, self-amortizing, paid monthly, with
mortgage insurance at 0.450% of the original principal. An earlier version of
this model carried the tab's *answer* — the "Calculated Monthly P+I+MIP
Payment" of 1,165.7819, transcribed to four places — and derived the two
published legs from it. Two contracts now carry the terms and produce the
answer:

```
cre.permanent_debt        150,000 at 4.00%, 180 months    1,109.5319 a month
cre.mortgage_insurance    0.450% of 150,000, flat            56.2500 a month
                                                          ----------
                                                           1,165.7819   the sizing tab, to the fourth decimal
```

Mortgage insurance is not a payment on the debt, and the debt contract does
not carry it; it is an agreement of its own, `cre.mortgage_insurance`, the
CRE refinement of `Contract.Insurance`. Coverage is measured on the whole
published line because the pack's `domain.cre.debt_service` subtotal includes
the insurance category — this source, the one that carries MIP, defines
coverage as NOI over P+I+MIP.

**The workbook pays fourteen of the fifteen years.** Its pro forma carries
the payment through year 14 and shows zero from year 15 — the same one-year
slip as the affordability period. The contracts' term therefore ends in 2037.
After the last payment the loan's account (`account.asset.home_project.balance`)
still holds 13,030.34, the year the workbook did not pay. A hand-written
stream simply stopped; a contract shows what stopping leaves behind.

**The lifetime figure moves by the rounding.** The workbook's 195,846 is
fourteen copies of a payment it rounds to 13,989; the contracts pay fourteen
copies of 13,989.38, 195,851.36. `expected_metrics.json` keeps the published
figure with a tolerance of six dollars, which is that rounding and nothing
else.

## Two pack gaps this case walked into

Both operating rules that should have fitted did not, so the streams here are
native, named into the CRE taxonomy so `--pack cre` metrics still aggregate
them — the same posture `benchmarks/cre/mit_rentleg_plaza` takes.

- **`cre.opex_line` emits a single un-suffixed stream.** One property, one
  expense line. Every real pro forma splits management, maintenance, utilities
  and taxes, and this one publishes all four. Adding `{{contract.dot_suffix}}`
  is a one-line change to the rule; the `domain.cre.noi` metric would need its
  exact-name selector widened to a prefix match to follow.
- **`cre.vacancy_loss` takes a constant `potential_gross_year`.** Vacancy is a
  rate against potential gross rent, and potential gross rent grows — but the
  rule cannot see the rent roll, so a growing property's vacancy loss is
  inexpressible. Here it also has to step at the affordability cliff.

## What this case does not cover

The workbook is an *underwriting* template and most of it sizes the deal rather
than projecting it. Out of scope, deliberately:

- **Gap funding solved as a residual.** The HOME subsidy is defined as the
  amount that closes the funding gap; the template solves for it. CFDL has no
  solver, so the subsidy is taken as the Sample's stated figure. This is the
  headline capability the source asks for and does not get.
- **Deferred payment loans.** Two soft loans sit at a constant 1,400,000 for
  the whole hold and repay from surplus cash at sale. Repayment contingent on
  available cash needs carry-forward state (§5.2). They produce no operating
  cash flow, so excluding them costs nothing until the sale.
- **The disposition.** Final-year cash flow is −1,285,610 against sale proceeds
  of 1,634,143, because the deferred loans are repaid out of them. This case
  models the hold, not the sale, and the last anchor asserts operating lines
  only.
- **AMI-indexed rent limits and sources & uses.** Regulatory rent caps are
  inputs to the Sample, not derivations within it.

## What this case asserts now that it did not

Two published decompositions became machine-checked assertions when streams
stopped having to be aggregates.

**The four expense sub-lines.** The workbook publishes Management, Operations
and Maintenance, Utilities, and Taxes/Insurance/Reserves separately (Operating
Pro Forma rows 18–21) and this case previously asserted only their total,
because `cre.opex_line` emitted one un-suffixed stream and a property could
declare exactly one expense line. Four streams now carry them, and
`expected.csv` asserts all four at every anchor year. The four states already
existed — they were split for the rounding reason — so this moved nothing:
their sum reproduces the total the file asserted before, at every anchor.

**P&I and MIP.** The pro forma's debt line is one number and the workbook
defines it as P+I+MIP. The two are separate contracts now, and three separate
streams — interest, principal and the premium — each grounded in the First
Mortgage Sizing tab's terms rather than in its published payment. The premium
is asserted at every anchor (56.25 a month, exact); interest and principal are
not published separately by the workbook and are not asserted, but their
sum with the premium is the sizing tab's 1,165.7819 to the fourth decimal.
The pro forma's debt cell is `=ROUND(...,0)`, so its 13,989 a year sits 0.38
under the contracts' 13,989.38; the coverage ratio it publishes is that
rounded line divided into a rounded NOI, which is the residual in the DSCR
table above.

The model applies the workbook's round through the same `round_to` it already
uses for the expense recurrence, rather than restating 13,989 as a constant — so
the derivation is visible and tracks the sizing inputs. P&I is then 13,314 and
MIP 675, summing to the published line exactly. Nothing about the previous
expectations changed.
