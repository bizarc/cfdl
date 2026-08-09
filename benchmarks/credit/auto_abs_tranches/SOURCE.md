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

## Still needed

The exhibit publishes percent **of initial class balance**, not the class
balances themselves, and the 8-K's four exhibits do not state them. They are in
this deal's own 424B5 prospectus, and in any of its 10-D distribution reports,
both public on EDGAR under the same registrant.

Deriving them from the published percentages instead would be calibrating the
inputs from the answers, which is not a reconciliation.
