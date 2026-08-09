# Source

**Ally Auto Receivables Trust 2017-3.** Depositor Ally Auto Assets LLC, sponsor
Ally Bank. Registered shelf, Commission file 333-204844-06.

The grid in `published_grid.csv` comes from Exhibit 99.4 to the Form 8-K filed
17 September 2018 (EDGAR CIK 1477336, accession 0001193125-18-274846),
`d623786dex994.htm` — "Weighted Average Life of the Notes". `source_exhibit.txt`
is that document as text.

This is the same exhibit already behind `auto_abs_wal`, `auto_abs_speed_050` and
`auto_abs_speed_150`, which use its 50-sub-pool collateral table. Those cases
take the collateral axis; this one takes the note classes, which they defer.

## What the exhibit states

Six tabulated classes — A-2, A-3, A-4, B, C, D. Class A-1 was paid in full on
16 January 2018 and is not tabulated.

| | coupon | day count |
|---|---|---|
| A-1 | 1.10000% | actual/360 |
| A-2 | 1.53% | 30/360 |
| A-3 | 1.74% | 30/360 |
| A-4 | 2.01% | 30/360 |
| B | 2.24% | 30/360 |
| C | 2.37% | 30/360 |
| D | 2.91% | 30/360 |

Assumptions for the tables, stated in full: receivables prepay at a constant
ABS percentage with **no defaults, losses or repurchases**; each payment is made
on the last day of the month and every month has 30 days; distributions are on
the 15th, commencing 15 October 2018; the servicing fee is 1.00% per annum and
the administration fee $1,500 per month, with all other fees and indemnities
zero; the certificate closing date is 15 September 2018; no event of default
occurs; and the servicer does not exercise its 10% clean-up call except in the
"to call" rows.

Because the tables assume no losses, overcollateralisation never has to build
and no trigger can trip. This case validates sequential pay — interest by class
at its own coupon and day count, then principal by seniority — and the clean-up
call. It does not exercise the OC or trigger machinery; that is what the
AmeriCredit 2017-1 case is for.

## The class principal amounts

The exhibit publishes percent **of initial class balance** without stating those
balances. They come from the trust's own Form 10-D distribution report for the
October 2018 period (EDGAR CIK 1705002, accession 0001705002-18-000057), which
states each class's beginning balance and its note pool factor per $1,000 of
original principal.

| class | original principal |
|---|---:|
| A-1 | 270,000,000.00 |
| A-2 | 371,370,000.00 |
| A-3 | 271,370,000.00 |
| A-4 | 86,010,000.00 |
| B | 22,220,000.00 |
| C | 18,510,000.00 |
| D | 13,750,000.00 |
| **total** | **1,053,230,000.00** |

A-3, A-4, B, C and D still stood at a factor of 1,000.00000000, so their stated
beginning balances ARE their originals. A-2 had begun amortising, at a factor of
239.65123125 against a balance of 88,999,277.75, which puts its original at
371,370,000.003. A-1 had already retired and reports zero, so it is backed out
of the aggregate: the beginning aggregate note pool factor of 475.54596598
against a beginning aggregate balance of 500,859,277.75 makes the total original
issuance 1,053,230,000.00, leaving A-1 at 270,000,000.00.

The derivation checks itself. Six of the seven come out as exact round numbers
and the seventh, A-2, reconciles to three decimal places — which is what a
correct reading of an issuer's factors should look like, and is not what
backing the figures out of the percentage grid would have given.
