# Source

**AmeriCredit Automobile Receivables Trust 2017-1.** Depositor AmeriCredit
Financial Services, Inc.; sponsor and servicer AmeriCredit Financial Services
(GM Financial). $930,000,000 of publicly offered notes against a $1,011,969,929
pool of sub-prime automobile loan contracts.

Everything this case uses comes from the Rule 424(b)(5) prospectus dated
21 February 2017, EDGAR CIK 1694010, accession 0001193125-17-045288,
`d269131d424b5.htm`. Pages cited below are the prospectus's own page numbers.

## What the prospectus publishes

Four tables captioned "Percent of Initial Note Principal Balance at Various ABS
Percentages" (pp. 59-62), covering the six publicly offered classes — A-1, A-2,
A-3, B, C and D — at **four speeds each: 0.50%, 1.00%, 1.50% and 2.00% ABS**.
Each runs the closing date plus 62 monthly distribution dates.

`published_grid.csv` is those four tables: 378 rows, 1,512 cells. Values are
whole percentages, and `*` marks a figure greater than 0.00% but less than
0.50%. **195 of the cells are informative** — the rest are exactly 0 or 100,
which assert only "retired by then" and "not started yet".

`published_wal.csv` is the two weighted average life rows under each table: a
life **to call** and a life **to maturity**, per class per speed, 48 figures.
They are identical for A-1 through C and differ only on Class D, which is the
only class still outstanding when the clean-up call is exercised.

## The assumptions the tables state

All fourteen are printed in full on pp. 57-58, and this case implements them
rather than the deal as priced:

- twelve assumed pools, each with an aggregate principal balance, a gross APR,
  an assumed cutoff date of 1 January or 1 February 2017, a remaining term to
  maturity and a seasoning (the table on p. 58);
- prepayments at a constant ABS percentage, **with no defaults, losses or
  repurchases**;
- each scheduled payment on the last day of the month, and every month 30 days;
- notes purchased 23 February 2017, distributions on the 18th from March 2017;
- a servicing fee of 2.25% per annum on the pool balance, and $625 a month for
  the trustee, owner trustee, collateral agent and asset representations
  reviewer, with all other fees zero;
- **principal paid as necessary to build and maintain the required
  overcollateralization** — the turbo is inside the published grid, not just in
  the prose; and
- the redemption option exercised at the earliest opportunity, so the
  percentage grid is a to-call grid throughout. Only the weighted average life
  row gives the to-maturity leg.

**The class sizes and rates in the tables are not the deal as priced.** The
tables assume Class A-2-A and A-2-B at $152,500,000 each; the deal priced
$230,000,000 and $75,000,000. Every coupon differs too — A-1 0.95% assumed
against 0.92000% priced, A-3 1.91% against 1.87%, B 2.35% against 2.30%,
C 2.88% against 2.71%, D 3.23% against 3.13%. The combined Class A-2 size is
$305,000,000 either way, and principal is paid to A-2-A and A-2-B pro rata, so
the split does not move the grid; the coupons do, through excess cash. The
tables are what this case reproduces.

## The structure

The priority of payments is stated as **22 numbered clauses** (pp. 77-78), plus
a separate 12-clause waterfall that applies after an event of default. The
mechanics the grid exercises:

| | |
|---|---|
| Clauses 3-17 | interest by seniority, interleaved with parity steps that never pay while the pool exceeds the notes |
| Clause 18 | the Noteholders' Principal Distributable Amount — principal collected, **less the Step-Down Amount** |
| Clause 19 | the reserve account, 2.0% of the initial pool, funded at closing |
| Clause 20 | the Accelerated Principal Amount: excess cash turboing the notes toward the target |
| Clause 22 | everything left to the certificateholder |

Overcollateralization starts at approximately 5.75% of the pool and targets
**14.75% of the pool balance less the amount required on deposit in the reserve
account** (glossary, "Required Pro Forma Note Balance"). The Step-Down Amount
is the release valve: principal that would take the notes below that required
balance is retained rather than paid. It is capped — the step-down may not
reduce overcollateralization below **0.50% of the initial pool balance**, which
the prospectus states twice, in the credit enhancement summary and in the
glossary.

The clean-up call is available once the pool falls to 10% or less of its
balance at the cutoff date.

## Licence

A public filing on EDGAR: freely accessible and citable. The document carries
the filer's copyright, so the figures are transcribed and cited and the file
itself is not vendored — the posture `mbs_pool_conventions` takes with SIFMA,
`auto_abs_tranches` with the Ally exhibit and `buenavista_del_cobre` with the
Southern Copper technical report.
