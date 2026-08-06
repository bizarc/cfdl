# The SBC convention fork — what reproducing a disclosed SaaS valuation found

`banker_dcf_conventions` already pins how CFDL places cash inside a period and
discounts it. This case exists for a different reason.

**Stock-based compensation is the most contested convention in software
valuation**, and this filing does something that almost never happens: it
discloses unlevered free cash flow **both ways**, before and after SBC, for the
same company on the same page. The gap is not a rounding difference.

| | 2024E | 2025E | 2026E | 2027E | 2028E |
|---|---:|---:|---:|---:|---:|
| unlevered FCF | 331 | 392 | 448 | 527 | 616 |
| less: SBC | (133) | (166) | (180) | (190) | (193) |
| **unlevered FCF, post-SBC** | **198** | **226** | **268** | **337** | **423** |

Two thirds of first-year cash flow turns on a convention. A model that can only
express one of these is not neutral about it.

## The source

A sell-side banker's discussion materials filed as an exhibit to a
going-private transaction, public on the securities regulator's filing system.
Free to read and citable; the filer retains copyright, so numbers are asserted
against and no file is vendored.

The exhibit uses code names for both parties. This benchmark therefore
describes the *analysis*, not the company — as with `banker_dcf_conventions`,
the conventions are what is being validated and they belong to no one.

What makes it usable is completeness. It discloses the unlevered FCF build-up
line by line **and** the SBC line **and** the post-SBC series, the discount
rate range (12.50% / 13.50% / 14.50%), the terminal method (an NTM unlevered-FCF
multiple of 15.0x / 16.0x / 17.0x / 18.0x on terminal-year uFCF of $696mm), the
discounting convention (stated as mid-period), the valuation date (31 March
2024), and a **3x4 grid of implied enterprise values**.

## The result

All twelve cells of the disclosed grid, implied enterprise value, $mm — CFDL
first, published in bold:

| WACC | 15.0x | 16.0x | 17.0x | 18.0x |
|---|---|---|---|---|
| 12.50% | 6,983.14 / **6,983** | 7,380.92 / **7,381** | 7,778.69 / **7,779** | 8,176.46 / **8,176** |
| 13.50% | 6,714.92 / **6,715** | 7,096.32 / **7,096** | 7,477.72 / **7,478** | 7,859.12 / **7,859** |
| 14.50% | 6,459.66 / **6,460** | 6,825.50 / **6,826** | 7,191.33 / **7,191** | 7,557.16 / **7,557** |

Worst disagreement **0.50 on ~7,000 — 0.007%**. The filing rounds to whole $mm
and its own build-up lines round the same way, so ±0.5 is the floor the source
can support.

## Finding 1 — the filing mixes the two conventions, and that is the point

The explicit-period flows are **post-SBC**. The terminal value multiple is
applied to a **pre-SBC** base.

This was not stated anywhere; it fell out of reproducing the grid. Solving the
grid backwards makes it unambiguous. Enterprise value is linear in the exit
multiple, so the slope of each row recovers the discounted terminal base
directly:

| WACC | $ per turn of multiple | implied discount exponent | implied TV base |
|---|---:|---:|---:|
| 12.50% | 397.67 | 4.752 | 695.8 |
| 13.50% | 381.33 | 4.751 | 695.9 |
| 14.50% | 365.67 | 4.753 | 695.9 |

The base is **$696mm — the pre-SBC terminal-year figure** — not the $484mm it
would be post-SBC. And the exponent is 4.75, confirming the terminal value is
discounted **whole** rather than mid-period. Meanwhile the intercept of the
same lines gives the present value of the explicit flows: ~1,016 / 994 / 973,
against ~1,652 for pre-SBC flows and ~1,029 for post-SBC. The flows are post-SBC.

**This is defensible, not an error.** The exit multiple was calibrated on peers'
*pre-SBC* unlevered FCF — the filing's own comparables page derives NTM uFCF of
$346 from pre-SBC figures — so applying it to a pre-SBC base is the consistent
choice. Treating SBC as a real cost in the explicit period and as a
market-convention adjustment at the exit is a considered position that a lot of
software valuation actually takes.

But it means **one model has to carry both definitions of the same line at
once**, which is exactly the capability under test. It is not something a model
with SBC netted into a single cash flow series can express at all.

## Finding 2 — SBC as its own stream makes the fork structural

The obvious way to model this is to enter the post-SBC series directly. That
would reproduce the grid and prove nothing: two hand-entered series that can
drift from each other, with the relationship between them living in a comment.

Instead the pre-SBC flow and the SBC deduction are **separate streams on the
same date**, and `model.net_cash_flow` asserts that their sum equals the
filing's published post-SBC series — 226, 268, 337, 423. So:

- both conventions come out of **one** model definition;
- the post-SBC series is *derived*, not restated, and cannot drift;
- the pre-SBC valuation is recoverable by deactivating the SBC streams;
- and the arithmetic tying the two is asserted rather than asserted-in-prose.

That is what "the SBC convention fork is expressible rather than hard-coded"
has to mean to be worth claiming.

## Finding 3 — the convention stack, independently confirmed

`banker_dcf_conventions` found a specific stack: mid-period discounting on the
flows, a stub period placed at its own midpoint, full years landing on `.25`
boundaries, and a terminal value discounted whole rather than mid-period.

That was one filing by one bank. **This is a different filing, a different
bank, a different industry, a different fiscal calendar** — December year-end
here against June there — and the same stack reproduces it to within the
source's rounding. Two independent confirmations of a convention are worth
considerably more than one, and the asymmetry between mid-period flows and a
whole-discounted terminal value is the part most often gotten wrong.

The period placements:

```
stub Q2-Q4 2024   mid of month 4    4.5/12  = 0.375
FY2025            end of month 14  15.0/12  = 1.25
FY2026            end of month 26  27.0/12  = 2.25
FY2027            end of month 38  39.0/12  = 3.25
FY2028            end of month 50  51.0/12  = 4.25
terminal value    end of month 56  57.0/12  = 4.75
```

## The one solved input

The filing states FY2024E unlevered FCF as a **full year** and notes the DCF
includes only **Q2-Q4**. It never publishes the quarterly split, so the stub is
not recoverable from the disclosed figures.

It is solved from the published grid instead: **one unknown against twelve
published enterprise values**, which leaves eleven degrees of freedom. At the
solved value of **$135.51mm**, all twelve land within ±0.50. A fitted parameter
that reproduces twelve independent published numbers to four significant
figures is a reconstruction, not a curve fit — but it is stated plainly here,
and in `model.cfdl` at the line itself, because it is the one figure in the
case that is not the filing's.

It is **68.4% of the full year, not 75%**, and the direction is the expected
one: this is an annual-prepaid subscription business, so Q1 carries a
disproportionate share of the year's cash collection. A model that assumed a
flat 75% would overstate the stub by $13mm and the enterprise value by ~$12mm.

The stub is stated post-SBC, because its pre-SBC/SBC split is not recoverable
either. That is why there is no `opco.sbc.stub_2024` line.

## What this case does not model

The **equity bridge** — enterprise value less net debt, divided by diluted
shares, giving the published $40.80-$52.20 per share range. It reconciles on the
filing's disclosed balance sheet and share count, but it is arithmetic on
constants rather than cash flow, so it is not modelled here. The same choice
`banker_dcf_conventions` made, for the same reason.
