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
| Replacement reserve | 4.35 | escalation recurrence, below |
| Total operating expenses | 12.26 | escalation recurrence, below |

And the metric that matters to a lender — debt service coverage, which HUD
publishes at four points to sixteen significant figures:

| | CFDL | published | difference |
|---|---|---|---|
| year 2 | 1.5757802845092563 | 1.5757380799199372 | +4.2e-05 |
| year 5 | 1.5337630092930168 | 1.5337765387089857 | −1.4e-05 |
| year 10 | 1.4335621406819647 | 1.4334834512831511 | +7.9e-05 |
| year 15 | 1.2887876803917375 | 1.2887268568160697 | +6.1e-05 |

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

## Finding — the workbook escalates by a recurrence, and we cannot

The two expense lines are the only ones that miss by more than rounding, and
they miss for a structural reason worth recording.

The workbook does not compute year *n* as `base × trend^n`. It computes it as
**last year's already-rounded figure times the trend**, rounding again. Verified
directly: of its four expense sub-lines, `Operations and Maintenance` and
`Taxes/Insurance/Reserves` reproduce **exactly** — worst 0.00 over 29 years —
under that recurrence, and not under any closed form. The replacement reserve
likewise: 21,013 → 21,538 → … → 41,948 matches the recurrence exactly and the
closed form drifts to 41,952.

We carry exact decimals from a base, so the two paths separate slowly. The
residual is monotone in the number of years compounded and reaches 12.26 on
204,655 at year 29 — 0.006%.

Expressing the recurrence needs two things CFDL does not have: a **backward
period reference**, so a stream can read its own prior period
(`docs/13_feature_backlog.md` §5.1), and a **rounding builtin** (§4.1). Neither
is worth building for this, but it is the second independent source to demand
the rounding builtin — the production tax credit needed it too — and the first
to demand it *combined* with a period reference. That combination is now
recorded.

Note the direction of the error: rounding at each step, then compounding, makes
the workbook's own figures drift from the exact arithmetic. Ours is the precise
answer. Agreeing to 0.006% with a spreadsheet that rounds every intermediate is
the right outcome.

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
