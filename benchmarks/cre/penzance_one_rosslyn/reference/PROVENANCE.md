# Provenance — One Rosslyn

Every source below was retrieved on **2026-08-25** unless stated. Figures are
labeled in `inputs/one_rosslyn.toml` as FACT, DERIVED or PROJECTION; this file
records where the FACT and DERIVED ones came from and how to verify them.

## The reference implementation

`One_Rosslyn_Reference_Model.xlsx` — 48,349 bytes,
SHA-256 `398ae9a63cd59ec350a30b774d87134ea39b9f5df16d91cbdff15545bdfb3d99`.

Sheets: README, Assumptions, CashFlow, ScenarioA, ScenarioB, Returns. Built
**before** the CFDL model, from the same frozen input set, so the two tie by
construction rather than by transcription. Both scenarios agree to under a
dollar.

## Entitlement — FACT

Arlington County Board Report, **REZN25-00001 / SPLA24-00041 / SPLA24-00040**,
"One Rosslyn — 1901 & 1911 Ft. Myer Dr.", approved 2025-07-19. 128 pages, 19 MB.

Retrieved from the County's public meeting archive:
`meetings.arlingtonva.us/CountyBoard/Documents/DownloadFileBytes/` with
`meetingId=2643`, `itemId=55766`, `publishId=71668`.

Read: the statistical summary at pp. 11–12 (site area, FAR, GFA, dwelling
units, per-tower breakdown, parking) and the community benefits conditions at
pp. 20–21 (the AHIF contributions and their certificate-of-occupancy tranches).

**Not redistributed** — 19 MB of public record. The archive ids above are
sufficient to re-fetch it.

**Note on the trade press.** Widely reported as an 82-unit condominium tower and
319 apartments. The County record says **73** and **311**. The record governs.

## Land and comparable sales — FACT

Arlington County Department of Real Estate Assessments, parcel and sales
records, `propertysearch.arlingtonva.us`. Queried per RPC; the sales history is
at `/Home/Sales?lrsn=<n>` and the parcel record at `/Home/GeneralInformation`.
`reference/arlington_propertysearch.py` in the companion case is the helper.

| Item | Value |
|---|---|
| Land, all three parcels | $52,000,000, recorded 2023-11-14, deed 20230100013266, sales code "4-Multiple RPCs" |
| RPCs | 16-020-001, 16-020-002, 16-020-006 — grantee ROSSLYN GATEWAY LLC |
| 2026 assessed, three parcels | $66,786,000 ($26,818,400 + $35,074,000 + $4,893,600) |
| Prior sales of 1911 Fort Myer | $33,700,000 on 2003-03-14 (deed 3475/2804); $19,000,000 on 1997-04-14 (deed 2825/1990). Recorded, unused. |

Comparable transactions, used as a cross-check on derived value rather than as
an input:

| Asset | Date | Price | Units | $/unit |
|---|---|---:|---:|---:|
| Aubrey (Cortland Rosslyn West) | 2022-05-17 | 266,455,000 | 331 | 805,000 |
| Evo (Cortland Rosslyn East) | 2022-05-17 | 334,600,000 | 455 | 735,385 |
| 3275 Washington Blvd | 2024-12-18 | 158,300,000 | 267 | 592,884 |
| Central Place | 2026-07-21 | 206,250,000 | 377 | 547,082 |

Central Place is RPCs 16-038-014 (74 units) and 16-038-019 (303 units), deed
20260100010398. Both are class 313 Apartment High-Rise in one economic unit, so
the price is a pure multifamily trade with no office or retail absorbing the
per-unit figure. Its 2026 assessed value is $233,287,300, so it traded at
**88.4% of assessed**, and assessed implies $618,799 per unit.

The Aubrey and Evo figures are independently confirmed in the 2026 Guidebook's
own apartment sales table.

**Excluded, with reason.** N Herndon St, 2025-08-28, $150,580,000: the 21 units
on that parcel are part of a multi-parcel trade and price to $7,170,476 per
unit. 1200 N Courthouse Rd, 2026-06-16, $194,444 per unit: inconsistent with
every other observation, likely affordable or a partial interest. Neither is
evidence; both are named so the exclusion is visible.

## Operating guidelines — FACT

Arlington County, **2026 Guidebook: Commercial Market Analysis & Guidelines**,
Department of Real Estate Assessments, April 2026 revision. 3.6 MB.
`arlingtonva.us/files/sharedassets/public/v/1/realestate/2026-commercial-guidebook-updated-april-2026.final.pdf`

Read: **MARKET** Apartment Guidelines, High-Rise 9+ stories (PCC 313),
effective age 2010+, Metro — 5.45% loaded capitalization rate, $9,466 per unit
of expenses, 5% vacancy and collection, garage parking $50–150 per space per
month, and market apartment land at $78,000 per rental unit within a half-mile
walk of Metro.

**Deliberately NOT used:** the Guidebook carries a second, similar set of tables
for **Committed Affordable** units (6.2% cap, $8,100 expenses, 3% vacancy).
Those govern affordable units. A market-rate tower takes the market tables.

## Published indices — FACT

Retrieved from FRED, 2026-08-25.

| Series | Use | Value |
|---|---|---|
| `WPUIP2312001` | construction escalation | PPI, new multifamily and other nonresidential building construction **output** prices. 53.4% from 2019-11 to 2026-07 |
| `WPUIP2311101` | cross-check | warehouse construction output, 49.6% over the same window |
| `CUSR0000SEHA` | rent growth | CPI-U, rent of primary residence, US city average. Trailing 10-year 4.23%, 3-year 3.98%, 1-year 3.11% |
| `SOFR` | construction debt | 3.65% at 2026-08-24 |
| `DGS10` | permanent debt | 4.70% at 2026-08-24 |

The **inputs** index `WPUSI012011` was used first and then rejected: it measures
materials and assumes contractor margin and productivity move with them, which
overstates. It gives 60.7% against the output index's 53.4%.

The DC-area rent series `CUURA311SEHA` and `CUUSA311SEHA` were checked and are
**discontinued** — both end in 2017 — so the national series is used instead.
Arlington's own multifamily assessed value rose 6.2% in 2026 over 2025 per the
Guidebook; the 3.0% growth rate the model applies is below every one of these.

## Derived, not assumed

Rent of **$3,789 per unit per month** in 2026 dollars, by the County's own
method: Central Place's 2026 assessed value of $233,287,300 times the 5.45%
guideline loaded cap gives the assessor's NOI of $12,714,158, or $33,725 per
unit; adding back $9,466 of guideline expenses and grossing up for 5% vacancy
gives the figure. Current Rosslyn asking rents bracket it — roughly $3,163 for
a one-bedroom and $4,667 for a two-bedroom — against a rental product averaging
1,025 sq ft.

## What this is not

Nothing here states what Penzance will earn. The project is entitled and
unbuilt; construction cost, debt pricing, lease-up pace, growth, condominium
pricing, holding period and the JV terms are all forecast, and the JV terms in
particular are private and are stated placeholders.
