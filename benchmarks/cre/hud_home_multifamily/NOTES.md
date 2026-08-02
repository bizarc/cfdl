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

Per-period, against the published pro forma, worst disagreement over all 29
years of each line:

| line | worst | what explains it |
|---|---|---|
| Gross potential rent | 0.48 | workbook rounds to whole dollars |
| Rent loss (vacancy) | 0.48 | same |
| Other revenue | 0.47 | same |
| Debt service | **0.00** | exact |
| Replacement reserve | **0.00** | exact — was 4.35, see below |
| Total operating expenses | **0.00** | exact — was 12.26, see below |

And the metric that matters to a lender — debt service coverage, which HUD
publishes at four points to sixteen significant figures:

| | CFDL | published | difference |
|---|---|---|---|
| year 2 | 1.5757802845092563 | 1.5757380799199372 | +4.2e-05 |
| year 5 | 1.5337679171491894 | 1.5337765387089857 | −8.6e-06 |
| year 10 | 1.4335489505325618 | 1.4334834512831511 | +6.6e-05 |
| year 15 | 1.2886742479090734 | 1.2887268568160697 | −5.3e-05 |

Agreement to five decimal places on a ratio built from lines the workbook has
already rounded to whole dollars. The residual is entirely that rounding.

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

Modelling the total as a single rounded line closed most of the gap and left
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

## Two pack gaps this case walked into

Both operating rules that should have fitted did not, so the streams here are
native, named into the CRE taxonomy so `--pack cre` metrics still aggregate
them — the same posture `benchmarks/cre/mit_rentleg_plaza` takes.

- **`cre.property_opex` emits a single un-suffixed stream.** One property, one
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
