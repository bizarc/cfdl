# Provenance

**The Highlands, Rosslyn, Virginia — Arlington County Site Plan SP #445.**
Penzance with The Baupost Group. Land 2011, delivered 2021, exited 2022–2024.

Every figure this case asserts comes from a public record of Arlington County,
Virginia, or is derived from one. Nothing is quoted from a private source and
nothing is fitted.

## The reference implementation

`Highlands_Reference_Model.xlsx` — an independent implementation of the deal,
built from the records below before this model existed.

| | |
|---|---|
| Size | 97731 bytes |
| SHA-256 | `80d94d450b45a8ea…` |

`reference_gen.py` generates `model.cfdl` from `inputs/highlands.toml`, which is
the same frozen input set the workbook reads. The two therefore tie by
construction rather than by transcription, and a disagreement is a real one.

## What is fact, and where it came from

**Program, unit mix by tower, GFA, parking, FAR, heights, the entitlement
chronology and the quantified public obligations** — the SP #445 record. Not
committed here: it runs to 71 MB. Arlington publishes it through the County
Board's meeting archive, and `arlingtonva.us` refuses automated fetch while
Granicus does not:

| document | clip_id | meta_id | bytes | SHA-256 |
|---|---|---|---|---|
| 2017-02-25 approval, 131 pp | 3314 | 157770 | 4798801 | `17e9b7e3a36a9279…` |
| 2017-02-25 drawings, 87 pp | 3314 | 157771 | 28910539 | — |
| 2018-09-22 amendment, 77 pp | 3556 | 179659 | 24355232 | `7f654483d7e029ee…` |
| 2020-11-14 amendment, 15 pp | — | 198705 | 1679557 | `0d6eddc7d250595b…` |

    https://arlington.granicus.com/MetaViewer.php?view_id=2&clip_id=<clip>&meta_id=<meta>

The drawing sets carry a byte-shifted font encoding: `pdftotext` returns
`81,7&28176` where the sheet reads UNIT COUNTS, and real digits map to control
characters and are dropped. Render the page and read it; do not parse it.

**Land basis $67,000,000 (2011-09-30, deed 4491/2257), both tower sale prices,
the 102 condominium closings, and the assessment histories** — Arlington's
assessment roll, free and without login, at `propertysearch.arlingtonva.us`.
`arlington_propertysearch.py` is the query helper: POST to `/Home/Search` with
`SearchFilters.RPCs`, `SearchFilters.TradeName` or the address fields, then
`/Home/{GeneralInformation,Assessments,Sales,Permits}?lrsn=N`.

The West Building sits on seven ground-leased County parcels, so its sale
produces no recorded deed price. It is published instead in the Commercial
Guidebook's apartment sales table.

**Capitalization rates, rents, vacancy, expenses and the sale comparables** —
Arlington County *Commercial Market Analysis & Guidelines*, published annually
by the Department of Real Estate Assessments.

| edition | bytes | SHA-256 |
|---|---|---|
| 2022 | 1167024 | `8e811266c8da1272…` |
| 2023 | 6078487 | `8dda0ebacc1d7d90…` |

Both are reachable through the Wayback Machine; several captures are truncated
at exactly 1 MiB, so compare `Content-Length` against
`x-archive-orig-x-crawler-content-length` and try another timestamp. The 2023
edition is a print-to-PDF with no text layer — render its pages rather than
parsing them. Page 13 carries the apartment sales table, page 8 the guidelines.

## Rents are derived, not assumed

Each tower's 1 January 2022 assessed value multiplied by the Guidebook's
guideline loaded capitalization rate for its class — High-Rise, Metro,
effective age 2010+, 5.15% in both editions — gives the assessor's own net
operating income. Solving that back for rent yields $3,522 per unit per month
for the West Building and $3,273 for the East. No rent is quoted from a listing
service and none is fitted to the answer.

## What is assumed

Construction cost, debt pricing, lease-up pace and the joint-venture tiers. The
Penzance/Baupost terms are private, so the waterfall is a stated placeholder
rather than the real split. `CASE.md` marks the boundary.

## A note on the capitalization rate convention

Arlington derives a *base* rate by dividing net operating income less real
estate taxes by sale price, then *loads* the effective tax rate onto it, and
applies the loaded rate to income **before** tax. Mixing that with a
conventional after-tax capitalization rate is a large silent error. The
recorded sales imply a loaded rate near 4.43% and 4.46%, roughly 70 basis
points inside the county guideline for the class.
