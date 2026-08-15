# Source

**Buenavista del Cobre Mine, Sonora, Mexico.** Operator Buenavista del Cobre,
S.A. de C.V.; registrant Southern Copper Corporation (SEC CIK 1001838).

`published_grid.csv` is Table 19.1, "Discounted Cash Flow", of the S-K
1300 Technical Report Summary prepared by WSP USA Environment &
Infrastructure Inc., document 31405632.000-R-Rev0, dated 11 February 2025.
Filed as Exhibit 96.6 to the FY2024 Form 10-K, EDGAR accession
0001558370-25-002017, `scco-20241231xex96d6.pdf`.

## Reading the table out of the filing

Worth recording, because it is not obvious and cost a wrong conclusion once.

Table 19.1 sits on report page 19-3, which is **PDF page 296**, a rotated
landscape page. `pdftotext -layout` returns that page as an empty string, and
`pdfimages -list` reports only a 37×98 logo, so every text-based check says the
page is blank. It is not — the table is vector text and renders cleanly:

```
pdftoppm -f 296 -l 296 -r 300 -png scco-20241231xex96d6.pdf out
```

then rotate −90. The document's producer is "Microsoft: Print To PDF", which is
where the broken text layer comes from.

## What the table states

Twenty-five columns: 2025 through 2045 annually, then 2046–2050, 2051–2055,
2056–2060 and 2061–2065 as five-year totals, plus a Total/Avg column. Every
figure is in US$ millions except the material movement block, which is in
millions of tonnes. The filing notes that costs are rounded to the nearest
million and that this may cause apparent summation differences.

Life-of-mine figures, as printed:

| | US$ M |
|---|---:|
| Total revenue | 76,952 |
| Total operating cost | 57,890 |
| EBITDA | 19,062 |
| Pre-tax gross income | 8,949 |
| Total taxes | 2,415 |
| Total capex | 8,317 |
| Closure | 544 |
| Pre-tax cash flow | 10,223 |
| After-tax cash flow | 6,639 |
| **Pre-Tax NPV @ 10%** | **5,826** |
| **After-Tax NPV @ 10%** | **3,405** |

The transcription was checked by summing every row against its own printed
Total/Avg column — worst deviation 7 on a whole-Mt tonnage row, and exact on
Total Material, EBITDA, Total Taxes, Total Capex and Pre Tax Cash Flow — and by
confirming that revenue, operating cost, EBITDA and pre-tax cash flow satisfy
their own identities in all 21 annual columns.

## Stated assumptions used here

From section 19.1: copper US$3.30/lb, molybdenum US$10.00/lb, zinc US$1.15/lb,
discount rate 10%. From section 19.2, in prose: a 7.5% royalty (Derechos de
Mineria); a 30% tax rate "on pre-tax gross income less 30% of the royalty"; an
employee profit sharing tax (PTU) at 10% of EBITDA less depreciation and
royalty; working capital on 30-day receivables, 45-day payables and 10-day
inventory, netting to zero over the life of the mine.

The tax rule as written in the filing did not reproduce the printed tax line.
The form that does, in all 21 annual columns, is 30% of gross income less the
**whole** duty rather than 30% of it. That reading is what `reference_gen.py`
implements; see `NOTES.md`.

## License

A public filing on EDGAR: freely accessible and citable. The underlying report
carries WSP's and Southern Copper's copyright, so figures are transcribed and
cited and the PDF itself is not vendored — the same posture
`mbs_pool_conventions` takes with SIFMA and `auto_abs_tranches` takes with the
Ally exhibit.

---

# Every assumption the filing states

Extracted from the S-K 1300 Technical Report Summary, Buenavista del Cobre,
WSP USA for Southern Copper Corporation, 11 February 2025. Section references
are to the report. This is the input inventory for `reference_gen.py`: each row
is either implemented, or carries the reason it is not.

## Prices and macro (§19.1)

| Assumption | Value | Status |
|---|---|---|
| Copper price | US$3.30/lb | implemented |
| Molybdenum price | US$10.00/lb | implemented |
| Zinc price | US$1.15/lb | implemented |
| Discount rate | 10% | implemented |
| Currency | all US$; no FX | n/a |
| Basis | **constant Q4 2024 US$ — a real model, no forward escalation.** CPI applied only to restate 2021–2023 historical costs to a Q4 2024 basis (§18.2.1, Table 18.5) | implemented implicitly; must be stated |
| Discounting placement | first year at par (recovered from the printed NPV) | implemented |

## Metallurgy and recovery (§12.2.4, §14)

| Assumption | Value | Status |
|---|---|---|
| Copper recovery, concentrator | 86.66% actual 3-yr avg; 86.49% design basis (Table 14.1) | available |
| Molybdenum recovery | 70.10% actual 3-yr avg; 66.00% design basis (Table 14.1) | available |
| Leach recovery, planning basis | 95.0% of acid-soluble Cu, 65.0% of cyanide-soluble Cu (§14.1.2.2) | **base data missing** — the soluble split per year is not published, only total Cu grade (Table 13.3) |
| Copper payability, concentrate | 96.7% (Tables 12.4, 12.6) | available |
| Molybdenum payability | 100.0% (Table 12.4) | available |
| Zinc payability | 85.0% (Table 12.6) | available |
| Copper payability, leach (SX-EW cathode) | 100.0% (Table 12.3) | available |
| Copper concentrator head grade target | 0.50% (2025), 0.48%, 0.43%, 0.40% from 2028 (Table 12.5) | available |
| Zinc concentrator head grade target | 1.2% Zn for first 11 years | available |
| Concentrator capacity | 74 Mt/yr to 2035, 43 Mt/yr after; zinc mill 7 Mt/yr (Table 13.3) | implemented as declared driver |

## Operating cost (§18.2, §18.3)

| Assumption | Value | Status |
|---|---|---|
| Mining unit cost, ex-haulage | US$0.91/t mined (Table 18.6) | available |
| Mining unit cost incl. haulage, LOM avg | US$2.71/t; by material: Cu ore 2.41, crushed leach 2.21, ROM leach 2.34, Zn ore 2.13, waste fresh 3.22, waste fill 2.68 (Table 18.7) | available |
| Copper processing | US$5.50/t milled; moly stream US$0.33/t milled (Table 18.8) | available |
| Leach crushing / leach processing / SX-EW | US$0.84/t crushed, US$0.40/t leached, US$0.46/lb Cu (Table 18.8) | available |
| Zinc plant | US$10.63/t milled; TC US$250/dmt; penalty US$3.88/dmt; freight US$154.62/wmt (Table 18.9) | available |
| G&A overhead | US$0.76/t milled (§18.3.1) | available |
| Concentrate transport | ~35% of Cu concentrate to La Caridad smelter, Nacozari (§18.3.2) | available |
| ARO accretion | ~US$34 M/yr (§19.2) | implemented as declared driver |

## Capital and closure (§18.1, §11)

| Assumption | Value | Status |
|---|---|---|
| Total LOM capital | US$8,317 M (Table 18.1) | implemented as declared driver |
| Maintenance & projects capital | US$5,211 M — mine 370, concentrator 2,003, hydromet 659, SX-EW 104, TSF 1,804, other 271 (Table 18.3) | available, LOM totals only |
| Equipment replacement basis | per-equipment productivity, operated hours and max life hours; staggered so ≤ ~15 haul trucks in one year (§18.1.1) | methodology only, no annual profile |
| Special projects | crusher projects, belt relocations, new zinc mill (Table 18.4) | available, LOM total only |
| Reclamation and closure | ~US$544 M (§19.2) | implemented as declared driver |
| Asset retirement obligation, 2023 basis | US$384 M BVC + US$127 M BVC Minera Mexico (§11) | available |

## Working capital (§19.2)

| Assumption | Value | Status |
|---|---|---|
| Accounts receivable | 30 days | **not implemented** — taken as a declared driver instead |
| Accounts payable | 45 days | **not implemented** |
| Inventory | 10 days | **not implemented** |
| Terminal release | remaining working capital recovered in the final year, so the series sums to zero | consistent with the printed row (−22, i.e. zero within rounding) |

## Fiscal (§19.2)

Every rule below is implemented, and each was confirmed line-by-line against
the sibling report for La Caridad/Pilares, whose Table 19.1 prints the
intermediate rows this one omits.

| Assumption | Value | Status |
|---|---|---|
| Mining duty (Derechos de Mineria) | 7.5% of EBITDA | implemented |
| Employee profit share (PTU) | 10% of EBITDA less depreciation and duty, floored at zero | implemented |
| Pre-tax gross income | EBITDA less depreciation, duty and PTU | implemented |
| Income tax | 30% of gross income | implemented |
| "Minimum tax" | 30% of the duty — a credit against income tax, not a charge | implemented |
| Total taxes | income tax less that credit, floored at zero | implemented |
| Loss carryforward | not stated; required to reproduce the nil tax of 2043-2045 | implemented |
| Depreciation | not printed for this mine; inverted from EBITDA and gross income, and scales with capital | implemented |
| After-tax cash flow | "total revenue less all operating, taxes, capital costs and ARO outlays" | implemented — depreciation and accretion are added back as non-cash |

The one-line summary in §19.2 — "30% on pre-tax gross income less 30% of the
royalty" — is exact as written once the "Minimum tax" row is read as the duty
credit. It parses as (30% of gross income) less (30% of the duty).

## What the parent 10-K settles

Southern Copper's FY2024 Form 10-K — the filing this report is Exhibit 96.6 to,
same accession — states the Mexican regime directly, in "Mexican Tax Matters":

> Since 2014, Mexican mining entities have been required to pay a mining
> royalty of 7.5% on taxable earnings before taxes, depreciation, and interest;
> and an additional royalty of 0.5% over gross receipts from sales of gold,
> silver and platinum. In 2024, the mining royalty was $119.5 million and the
> additional royalty was $1.4 million.

Three things follow:

1. **The duty base is EBITDA**, confirmed by the issuer in its own words —
   "taxable earnings before taxes, depreciation, and interest". That is the
   base the sibling report's printed rows give, so statute and filing agree.
2. **There is a second levy** of 0.5% on gold, silver and platinum receipts.
   Buenavista produces silver, but the report's revenue lines carry only
   copper, molybdenum and zinc, so it cannot be sized from Table 19.1.
3. **The report applies a superseded rate.** The 10-K records that the Ley
   Federal de Derechos was amended on 19 December 2024, effective 1 January
   2025, raising the mining royalty from 7.5% to 8.5% and the additional
   royalty from 0.5% to 1%. The technical report is dated 11 February 2025 and
   its forecast begins 1 January 2025, yet §19.2 applies 7.5% throughout.

Point 1 is the one that mattered. Before the sibling report was read, an 8.5%
duty looked attractive because it cut the then-unexplained gap between EBITDA
and net income by more than three times. It was a red herring: the gap was the
employee profit share, and at the stated 7.5% the whole stack closes. The rate
increase is real and is recorded here, but it is not in this model.

All 78 sensitivity points are reproduced, to within 1.57% of the base NPV.
Two items in the parent 10-K remain deliberately unmodeled:

1. The 0.5% additional royalty on gold, silver and platinum receipts. This
   mine's published revenue carries only copper, molybdenum and zinc, so the
   levy cannot be sized from Table 19.1.
2. The rate increase to 8.5% effective 1 January 2025. The report applies 7.5%
   across a forecast that begins on that date. The case reproduces what the
   filing computed; it does not assert that the filing is right.
