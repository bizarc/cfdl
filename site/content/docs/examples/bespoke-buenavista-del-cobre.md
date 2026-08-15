---
id: benchmark-bespoke-buenavista-del-cobre
title: "Bespoke: open-pit copper mine"
slug: "/docs/examples/bespoke-buenavista-del-cobre"
description: "A 41-year open-pit copper mine from Southern Copper's SEC technical report, carrying three payable metals, six cost lines and a four-layer Mexican fiscal stack that resolves in one pass without a solver."
source: benchmarks/bespoke/buenavista_del_cobre
---

# Bespoke: open-pit copper mine

A 41-year open-pit copper mine from Southern Copper's SEC technical report, carrying three payable metals, six cost lines and a four-layer Mexican fiscal stack that resolves in one pass without a solver.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

Buenavista del Cobre is an open-pit copper mine in Sonora, Mexico, in
production since 1899 and today among the largest in the world. The reserve
supports 41 more years: 2.1 billion tonnes of mill ore, a further 2.1 billion
of leach ore, 296 million tonnes of zinc ore and 3.8 billion tonnes of waste,
yielding copper, molybdenum and zinc through to 2065.

What makes it worth modeling is not the mining but the tax. Four charges sit
between EBITDA and net income, and each one's base is defined in terms of the
others: the Derechos de Mineria takes 7.5% of earnings; an employee profit
share takes 10% of what is left after depreciation and that duty; income tax
takes 30% of what remains; and the duty is then credited back against the tax
at 30%. Read as levies on earnings they look mutually circular. They are not,
and the case is here to show that a language which resolves them in one pass
needs no solver to do it.

## The reference

Table 19.1, "Discounted Cash Flow", of the S-K 1300 Technical Report Summary
prepared by WSP USA for Southern Copper Corporation, dated 11 February 2025 and
filed as Exhibit 96.6 to the FY2024 Form 10-K. It prints the whole life of the
mine — material movement, revenue by metal, six cost lines, EBITDA, gross
income, tax, capital, closure, working capital, and both a pre-tax and an
after-tax NPV at a stated 10%. Table 19.2 adds a sensitivity matrix: six
variables at thirteen steps each, 78 published after-tax NPVs. Both are
transcribed here; the PDF is a public filing and is cited rather than vendored.

**The fiscal structure was not taken from this deal.** Buenavista prints EBITDA
and gross income but not the charges between them, so fitting them here would
have meant fitting to the answer. They were read instead off a *different*
mine — La Caridad/Pilares, Exhibit 96.7 of the same Form 10-K, same author and
template — whose Table 19.1 prints all ten intermediate lines explicitly,
including the rows Buenavista omits: Depreciation, Royalty, PTU, Minimum tax,
Income tax and both add-backs. The structure recovered there reproduces all ten
of La Caridad's printed lines to within 0.77 US$ M. Applied unchanged to
Buenavista, it reproduces this mine's printed tax, net income and after-tax
cash flow to within 1.44.

An independent implementation of those conventions produces the expectations
this case asserts. The case is therefore checked twice over: that reference
against the filing, and CFDL against the reference.

## What it exercises

| | |
|---|---|
| Pack | none — written from the bare language |
| Entities | one real asset, carrying its own lifecycle and its one memory |
| Language features | second-tier streams reading the period's result through `series_sum`, open-world lifecycle events with published transitions, declared phases, a carryforward recurrence, annuity-due placement, run-config parameters driving 72 scenarios |
| Conventions | duty on EBITDA, profit share on EBITDA net of depreciation and duty, income tax net of a duty credit, loss carryforward, first year undiscounted |

The second case in the suite written without a pack, after
`ppiaf_toll_highway`. A mine is none of the four: no generation and no
offtaker, no rent roll, no pool of obligors, and revenue is contained metal at
a price rather than a margin on sales.

**The stack is closed form.** Each charge's base is settled before it is
struck, so all four evaluate in a single pass:

    royalty      = 7.5% × ebitda
    ptu          = 10% × max(0, ebitda − depreciation − royalty)
    gross_income = ebitda − depreciation − royalty − ptu
    total_taxes  = max(0, 30% × (gross_income + shelter) − 30% × royalty)

Note that gross income is exactly 0.9 × (ebitda − depreciation − royalty), so
the profit share is one ninth of gross income. That identity is why the stack
unwinds without iteration, and it is what lets depreciation — which this mine
does not publish — be recovered by inverting the two lines it does.

**The carryforward is load-bearing.** The filing prints no tax at all in 2043,
2044 and 2045 although gross income is positive in each, because 2037 through
2042 ran at a loss. Without the shelter those three years are wrong by 46
US$ M while every other column still passes.

**Depreciation must scale with capital.** The filing's capital sensitivity
reprices the depreciation that capital creates. Holding it fixed throws that
row of Table 19.2 out by 125 US$ M; scaling it with `cfg.capex_factor` brings
the same row to 12.

## The result

All 41 periods reproduce across nineteen columns — three revenue lines, six
cost lines, three fiscal charges, the accretion add-back, three capital lines,
the loss carryforward as the mine's own field, and net cash flow — to 1e-5,
the float noise of the price-times-quantity round trip. EBITDA appears in no
column and no curve: it is the result of the base streams, and the fiscal
streams read it from the period's realized series. Three metrics and **72 scenarios** reproduce
on the same tolerance, the scenarios covering all six variables of Table 19.2
at every non-zero step.

Against the filing itself, the reference reproduces all eight derived lines
over the 21 printed annual columns:

| line | max abs. difference | mean |
|---|---:|---:|
| Total revenue | 1.00 | 0.29 |
| Total operating cost | 1.00 | 0.19 |
| EBITDA | 2.00 | 0.52 |
| Pre-tax gross income | 1.85 | 0.45 |
| Total taxes | 0.69 | 0.12 |
| Net income after taxes | 1.32 | 0.39 |
| Pre-tax cash flow | 2.00 | 0.67 |
| After-tax cash flow | 1.44 | 0.53 |

in US$ M, against cells the filing rounds to US$ 1 M. Both published NPVs land
inside 0.10% — pre-tax 5,820.2 against 5,826, after-tax 3,402.8 against 3,405 —
and all 78 sensitivity points inside 1.57% of the base NPV, four of the six
rows inside 0.5%.

## The delta

**The per-column residuals are the filing's own rounding, not error.** Every
cell is printed to the nearest million and the filing says so. A derived line
is a sum or difference of rounded cells, so its bound is the number of cells it
touches times a half million, plus a half for the printed figure compared
against: 2.00 for EBITDA, which touches nine. Nothing exceeds its bound, and
most sit at a third of it. The filing is not internally exact either — summing
its own printed revenue cells gives 76,951 against its own printed total of
76,952.

**Both NPV residuals have a single cause, and it is an assumption rather than
rounding.** 2046 through 2065 are published only as four five-year buckets, and
the model divides each evenly across its five years. The true intra-bucket
profile is not in the document and cannot be recovered from it. Nothing else in
the model is approximate.

**That same assumption is most of the sensitivity error, and it is
instructive.** The two price rows drift furthest at ±30%, where a large price
move changes which years the loss shelter covers. Averaging a driver is safe
for a linear line and unsafe for anything with a threshold in it; the shelter
is a threshold, and flat bucket income never trips it where lumpy income would.
Any case that smooths an input should be read with that in mind.

**What the case does not claim.** The 0.5% additional royalty on gold, silver
and platinum receipts — confirmed in the parent 10-K — is not modeled: this
mine's published revenue carries only copper, molybdenum and zinc, so the levy
cannot be sized from Table 19.1. And the report applies a 7.5% duty across a
forecast beginning 1 January 2025, although the 10-K records the Ley Federal de
Derechos raising that rate to 8.5% with effect from that very date. The case
reproduces what the filing computed, which is the claim it makes; it is not a
statement that the filing is right.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.1,"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"scenarios":{"opex_m30":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":0.7,"cfg.capex_factor":1}},"opex_m25":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":0.75,"cfg.capex_factor":1}},"opex_m20":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":0.8,"cfg.capex_factor":1}},"opex_m15":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":0.85,"cfg.capex_factor":1}},"opex_m10":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":0.9,"cfg.capex_factor":1}},"opex_m5":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":0.95,"cfg.capex_factor":1}},"opex_p5":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1.05,"cfg.capex_factor":1}},"opex_p10":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1.1,"cfg.capex_factor":1}},"opex_p15":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1.15,"cfg.capex_factor":1}},"opex_p20":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1.2,"cfg.capex_factor":1}},"opex_p25":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1.25,"cfg.capex_factor":1}},"opex_p30":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1.3,"cfg.capex_factor":1}},"capex_m30":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":0.7}},"capex_m25":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":0.75}},"capex_m20":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":0.8}},"capex_m15":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":0.85}},"capex_m10":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":0.9}},"capex_m5":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":0.95}},"capex_p5":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1.05}},"capex_p10":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1.1}},"capex_p15":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1.15}},"capex_p20":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1.2}},"capex_p25":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1.25}},"capex_p30":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1.3}},"commodity_m30":{"parameters":{"cfg.price_cu":2.31,"cfg.price_mo":7,"cfg.price_zn":0.805,"cfg.opex_factor":1,"cfg.capex_factor":1}},"commodity_m25":{"parameters":{"cfg.price_cu":2.475,"cfg.price_mo":7.5,"cfg.price_zn":0.8625,"cfg.opex_factor":1,"cfg.capex_factor":1}},"commodity_m20":{"parameters":{"cfg.price_cu":2.64,"cfg.price_mo":8,"cfg.price_zn":0.92,"cfg.opex_factor":1,"cfg.capex_factor":1}},"commodity_m15":{"parameters":{"cfg.price_cu":2.805,"cfg.price_mo":8.5,"cfg.price_zn":0.9775,"cfg.opex_factor":1,"cfg.capex_factor":1}},"commodity_m10":{"parameters":{"cfg.price_cu":2.97,"cfg.price_mo":9,"cfg.price_zn":1.035,"cfg.opex_factor":1,"cfg.capex_factor":1}},"commodity_m5":{"parameters":{"cfg.price_cu":3.135,"cfg.price_mo":9.5,"cfg.price_zn":1.0925,"cfg.opex_factor":1,"cfg.capex_factor":1}},"commodity_p5":{"parameters":{"cfg.price_cu":3.465,"cfg.price_mo":10.5,"cfg.price_zn":1.2075,"cfg.opex_factor":1,"cfg.capex_factor":1}},"commodity_p10":{"parameters":{"cfg.price_cu":3.63,"cfg.price_mo":11,"cfg.price_zn":1.265,"cfg.opex_factor":1,"cfg.capex_factor":1}},"commodity_p15":{"parameters":{"cfg.price_cu":3.795,"cfg.price_mo":11.5,"cfg.price_zn":1.3225,"cfg.opex_factor":1,"cfg.capex_factor":1}},"commodity_p20":{"parameters":{"cfg.price_cu":3.96,"cfg.price_mo":12,"cfg.price_zn":1.38,"cfg.opex_factor":1,"cfg.capex_factor":1}},"commodity_p25":{"parameters":{"cfg.price_cu":4.125,"cfg.price_mo":12.5,"cfg.price_zn":1.4375,"cfg.opex_factor":1,"cfg.capex_factor":1}},"commodity_p30":{"parameters":{"cfg.price_cu":4.29,"cfg.price_mo":13,"cfg.price_zn":1.495,"cfg.opex_factor":1,"cfg.capex_factor":1}},"copper_m30":{"parameters":{"cfg.price_cu":2.31,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"copper_m25":{"parameters":{"cfg.price_cu":2.475,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"copper_m20":{"parameters":{"cfg.price_cu":2.64,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"copper_m15":{"parameters":{"cfg.price_cu":2.805,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"copper_m10":{"parameters":{"cfg.price_cu":2.97,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"copper_m5":{"parameters":{"cfg.price_cu":3.135,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"copper_p5":{"parameters":{"cfg.price_cu":3.465,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"copper_p10":{"parameters":{"cfg.price_cu":3.63,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"copper_p15":{"parameters":{"cfg.price_cu":3.795,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"copper_p20":{"parameters":{"cfg.price_cu":3.96,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"copper_p25":{"parameters":{"cfg.price_cu":4.125,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"copper_p30":{"parameters":{"cfg.price_cu":4.29,"cfg.price_mo":10,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"molybdenum_m30":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":7,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"molybdenum_m25":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":7.5,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"molybdenum_m20":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":8,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"molybdenum_m15":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":8.5,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"molybdenum_m10":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":9,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"molybdenum_m5":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":9.5,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"molybdenum_p5":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10.5,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"molybdenum_p10":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":11,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"molybdenum_p15":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":11.5,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"molybdenum_p20":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":12,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"molybdenum_p25":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":12.5,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"molybdenum_p30":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":13,"cfg.price_zn":1.15,"cfg.opex_factor":1,"cfg.capex_factor":1}},"zinc_m30":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":0.805,"cfg.opex_factor":1,"cfg.capex_factor":1}},"zinc_m25":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":0.8625,"cfg.opex_factor":1,"cfg.capex_factor":1}},"zinc_m20":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":0.92,"cfg.opex_factor":1,"cfg.capex_factor":1}},"zinc_m15":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":0.9775,"cfg.opex_factor":1,"cfg.capex_factor":1}},"zinc_m10":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.035,"cfg.opex_factor":1,"cfg.capex_factor":1}},"zinc_m5":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.0925,"cfg.opex_factor":1,"cfg.capex_factor":1}},"zinc_p5":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.2075,"cfg.opex_factor":1,"cfg.capex_factor":1}},"zinc_p10":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.265,"cfg.opex_factor":1,"cfg.capex_factor":1}},"zinc_p15":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.3225,"cfg.opex_factor":1,"cfg.capex_factor":1}},"zinc_p20":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.38,"cfg.opex_factor":1,"cfg.capex_factor":1}},"zinc_p25":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.4375,"cfg.opex_factor":1,"cfg.capex_factor":1}},"zinc_p30":{"parameters":{"cfg.price_cu":3.3,"cfg.price_mo":10,"cfg.price_zn":1.495,"cfg.opex_factor":1,"cfg.capex_factor":1}}}}
// Buenavista del Cobre, a 41-year open-pit copper mine in Sonora, Mexico,
// built against the discounted cash flow its operator filed with the SEC.
//
// WHY THIS CASE IS PACK-FREE. Like the toll road, a mine is none of the four
// packs: no generation and no offtaker, no rent roll, no pool of obligors, and
// revenue is contained metal at a price rather than a margin on sales.
//
// THE ARCHITECTURE, IN ONE PARAGRAPH. Base streams state the period's cash:
// three metals sold and six cost lines. EBITDA is nobody's input — it is the
// RESULT of those streams, and everything after it reads that result through
// series_sum over the period. The fiscal charges are second-tier streams, each
// a claim on the period's EBITDA; cross-stream reads are one hop deep by
// design (docs/10: phase-2 streams cannot reference each other), so each
// charge derives from EBITDA in closed form rather than chaining off another
// charge. The mine's one genuine memory is the loss carryforward, a field.
// The mine's regime changes are events writing its status.
//
// THE FISCAL STACK RESOLVES IN ONE PASS. Four charges sit between EBITDA and
// net income, each defined against the others, none circular — every base is
// settled before its charge is struck:
//
//     royalty      = 7.5% * ebitda
//     ptu          = 10% * max(0, ebitda - depreciation - royalty)
//     gross_income = ebitda - depreciation - royalty - ptu
//     total_taxes  = max(0, 30% * (gross_income + shelter) - 30% * royalty)
//
// The 30%-of-royalty term is the "Minimum tax" row of the sibling filing: a
// credit, not a second charge. The structure was read off La Caridad/Pilares
// (Exhibit 96.7 of the same Form 10-K), which prints every intermediate line
// this filing omits, and reproduces both mines inside their own rounding.
// Nothing is fitted to this mine's answer. See CASE.md.
//
// WHERE THE STRUCTURE CAME FROM decides what is data and what is rule.
// Payable metal, the cost lines and capital are printed rows of Table 19.1,
// so they are curves — declared data. The charges are stated rules, so they
// are streams. Depreciation is the deliberate exception; see its curve.
//
// 2025 IS NOT DISCOUNTED. The filing discounts its first year at par, so
// every schedule is written `due`: cash falls at the period's open. Written
// as an ordinary annuity every figure is unchanged and the NPV comes out at
// the published value over 1.10.

version 0.1
model "buenavista-del-cobre"
time calendar annual from 2025-01 for 41

// The mine's three eras, as the filing states them: full-rate milling to
// 2035, the reduced plant after Concentrator I is taken offline, and the
// reclamation years. Phases carry the calendar; the state machine below
// carries the behavior; phase_enter joins them so each date is stated once.
phase full_rate from 2025-01 to 2035-12
phase reduced_plant from 2036-01 to 2060-12
phase reclamation from 2061-01 to 2065-12

// --- the Mexican fiscal stack, section 19.2 --------------------------------
assume duty_rate = 0.075     // Derechos de Mineria, on EBITDA
assume ptu_rate  = 0.10      // employee profit share, on EBITDA net of
                             // depreciation and the duty
assume tax_rate  = 0.30
assume closure_total = 544.0  // reclamation and closure, 2061-2065 bucket      // income tax, and the rate of the royalty credit

// Prices and the two sensitivity factors are run-config knobs rather than
// assumptions, so that Table 19.2's 78 published points are reachable by
// overriding a parameter instead of editing a curve. Base values are in
// run.json: US$3.30/lb copper, US$10.00/lb molybdenum, US$1.15/lb zinc.

// ---------------------------------------------------------------------------
// Declared drivers. Every one is a printed row of Table 19.1 except
// depreciation, which is inverted from two that are. The annual columns run
// 2025-2045; the four five-year buckets that follow are divided evenly, which
// is the model's only assumption the filing does not state.
// ---------------------------------------------------------------------------

curve cu_payable {
  2025-01: 883.939394
  2026-01: 870.303030
  2027-01: 773.333333
  2028-01: 769.393939
  2029-01: 750.000000
  2030-01: 901.515152
  2031-01: 785.151515
  2032-01: 806.060606
  2033-01: 764.242424
  2034-01: 798.181818
  2035-01: 802.424242
  2036-01: 477.575758
  2037-01: 357.878788
  2038-01: 409.696970
  2039-01: 426.969697
  2040-01: 428.787879
  2041-01: 397.878788
  2042-01: 429.696970
  2043-01: 487.878788
  2044-01: 486.666667
  2045-01: 478.181818
  2046-01: 405.393939
  2047-01: 405.393939
  2048-01: 405.393939
  2049-01: 405.393939
  2050-01: 405.393939
  2051-01: 392.181818
  2052-01: 392.181818
  2053-01: 392.181818
  2054-01: 392.181818
  2055-01: 392.181818
  2056-01: 463.454545
  2057-01: 463.454545
  2058-01: 463.454545
  2059-01: 463.454545
  2060-01: 463.454545
  2061-01: 411.636364
  2062-01: 411.636364
  2063-01: 411.636364
  2064-01: 411.636364
  2065-01: 411.636364
}

curve mo_payable {
  2025-01: 13.400000
  2026-01: 13.600000
  2027-01: 6.200000
  2028-01: 5.700000
  2029-01: 6.800000
  2030-01: 9.100000
  2031-01: 9.800000
  2032-01: 10.400000
  2033-01: 11.800000
  2034-01: 11.900000
  2035-01: 10.400000
  2036-01: 10.500000
  2037-01: 12.300000
  2038-01: 5.200000
  2039-01: 6.000000
  2040-01: 5.100000
  2041-01: 6.400000
  2042-01: 6.400000
  2043-01: 6.300000
  2044-01: 6.600000
  2045-01: 6.600000
  2046-01: 5.480000
  2047-01: 5.480000
  2048-01: 5.480000
  2049-01: 5.480000
  2050-01: 5.480000
  2051-01: 4.980000
  2052-01: 4.980000
  2053-01: 4.980000
  2054-01: 4.980000
  2055-01: 4.980000
  2056-01: 3.280000
  2057-01: 3.280000
  2058-01: 3.280000
  2059-01: 3.280000
  2060-01: 3.280000
  2061-01: 6.080000
  2062-01: 6.080000
  2063-01: 6.080000
  2064-01: 6.080000
  2065-01: 6.080000
}

curve zn_payable {
  2025-01: 146.956522
  2026-01: 120.869565
  2027-01: 124.347826
  2028-01: 128.695652
  2029-01: 108.695652
  2030-01: 107.826087
  2031-01: 119.130435
  2032-01: 128.695652
  2033-01: 128.695652
  2034-01: 119.130435
  2035-01: 119.130435
  2036-01: 36.521739
  2037-01: 32.173913
  2038-01: 12.173913
  2039-01: 8.695652
  2040-01: 7.826087
  2041-01: 3.478261
  2042-01: 4.347826
  2043-01: 6.086957
  2044-01: 5.217391
  2045-01: 6.086957
  2046-01: 17.043478
  2047-01: 17.043478
  2048-01: 17.043478
  2049-01: 17.043478
  2050-01: 17.043478
  2051-01: 71.130435
  2052-01: 71.130435
  2053-01: 71.130435
  2054-01: 71.130435
  2055-01: 71.130435
  2056-01: 53.217391
  2057-01: 53.217391
  2058-01: 53.217391
  2059-01: 53.217391
  2060-01: 53.217391
  2061-01: 36.173913
  2062-01: 36.173913
  2063-01: 36.173913
  2064-01: 36.173913
  2065-01: 36.173913
}

curve cost_mining {
  2025-01: 585.000000
  2026-01: 593.000000
  2027-01: 605.000000
  2028-01: 744.000000
  2029-01: 726.000000
  2030-01: 708.000000
  2031-01: 679.000000
  2032-01: 660.000000
  2033-01: 629.000000
  2034-01: 621.000000
  2035-01: 618.000000
  2036-01: 624.000000
  2037-01: 556.000000
  2038-01: 571.000000
  2039-01: 557.000000
  2040-01: 598.000000
  2041-01: 547.000000
  2042-01: 569.000000
  2043-01: 605.000000
  2044-01: 526.000000
  2045-01: 520.000000
  2046-01: 490.800000
  2047-01: 490.800000
  2048-01: 490.800000
  2049-01: 490.800000
  2050-01: 490.800000
  2051-01: 501.400000
  2052-01: 501.400000
  2053-01: 501.400000
  2054-01: 501.400000
  2055-01: 501.400000
  2056-01: 466.200000
  2057-01: 466.200000
  2058-01: 466.200000
  2059-01: 466.200000
  2060-01: 466.200000
  2061-01: 441.200000
  2062-01: 441.200000
  2063-01: 441.200000
  2064-01: 441.200000
  2065-01: 441.200000
}

curve cost_concentrator {
  2025-01: 509.000000
  2026-01: 509.000000
  2027-01: 506.000000
  2028-01: 506.000000
  2029-01: 506.000000
  2030-01: 508.000000
  2031-01: 509.000000
  2032-01: 509.000000
  2033-01: 509.000000
  2034-01: 509.000000
  2035-01: 509.000000
  2036-01: 327.000000
  2037-01: 330.000000
  2038-01: 330.000000
  2039-01: 330.000000
  2040-01: 328.000000
  2041-01: 330.000000
  2042-01: 330.000000
  2043-01: 330.000000
  2044-01: 330.000000
  2045-01: 330.000000
  2046-01: 329.000000
  2047-01: 329.000000
  2048-01: 329.000000
  2049-01: 329.000000
  2050-01: 329.000000
  2051-01: 329.000000
  2052-01: 329.000000
  2053-01: 329.000000
  2054-01: 329.000000
  2055-01: 329.000000
  2056-01: 330.000000
  2057-01: 330.000000
  2058-01: 330.000000
  2059-01: 330.000000
  2060-01: 330.000000
  2061-01: 330.000000
  2062-01: 330.000000
  2063-01: 330.000000
  2064-01: 330.000000
  2065-01: 330.000000
}

curve cost_smelting {
  2025-01: 698.000000
  2026-01: 672.000000
  2027-01: 644.000000
  2028-01: 636.000000
  2029-01: 586.000000
  2030-01: 682.000000
  2031-01: 605.000000
  2032-01: 616.000000
  2033-01: 581.000000
  2034-01: 600.000000
  2035-01: 604.000000
  2036-01: 377.000000
  2037-01: 281.000000
  2038-01: 344.000000
  2039-01: 306.000000
  2040-01: 340.000000
  2041-01: 277.000000
  2042-01: 307.000000
  2043-01: 369.000000
  2044-01: 372.000000
  2045-01: 368.000000
  2046-01: 310.800000
  2047-01: 310.800000
  2048-01: 310.800000
  2049-01: 310.800000
  2050-01: 310.800000
  2051-01: 331.400000
  2052-01: 331.400000
  2053-01: 331.400000
  2054-01: 331.400000
  2055-01: 331.400000
  2056-01: 347.600000
  2057-01: 347.600000
  2058-01: 347.600000
  2059-01: 347.600000
  2060-01: 347.600000
  2061-01: 321.800000
  2062-01: 321.800000
  2063-01: 321.800000
  2064-01: 321.800000
  2065-01: 321.800000
}

curve cost_gna {
  2025-01: 62.000000
  2026-01: 62.000000
  2027-01: 61.000000
  2028-01: 61.000000
  2029-01: 61.000000
  2030-01: 62.000000
  2031-01: 62.000000
  2032-01: 62.000000
  2033-01: 62.000000
  2034-01: 62.000000
  2035-01: 62.000000
  2036-01: 38.000000
  2037-01: 38.000000
  2038-01: 38.000000
  2039-01: 38.000000
  2040-01: 38.000000
  2041-01: 38.000000
  2042-01: 38.000000
  2043-01: 38.000000
  2044-01: 38.000000
  2045-01: 38.000000
  2046-01: 38.200000
  2047-01: 38.200000
  2048-01: 38.200000
  2049-01: 38.200000
  2050-01: 38.200000
  2051-01: 38.200000
  2052-01: 38.200000
  2053-01: 38.200000
  2054-01: 38.200000
  2055-01: 38.200000
  2056-01: 38.400000
  2057-01: 38.400000
  2058-01: 38.400000
  2059-01: 38.400000
  2060-01: 38.400000
  2061-01: 38.400000
  2062-01: 38.400000
  2063-01: 38.400000
  2064-01: 38.400000
  2065-01: 38.400000
}


curve cost_decommissioning {
  2025-01: 0.000000
  2026-01: 0.000000
  2027-01: 0.000000
  2028-01: 0.000000
  2029-01: 0.000000
  2030-01: 0.000000
  2031-01: 0.000000
  2032-01: 0.000000
  2033-01: 0.000000
  2034-01: 0.000000
  2035-01: 0.000000
  2036-01: 5.000000
  2037-01: 5.000000
  2038-01: 5.000000
  2039-01: 5.000000
  2040-01: 5.000000
  2041-01: 0.000000
  2042-01: 0.000000
  2043-01: 0.000000
  2044-01: 0.000000
  2045-01: 0.000000
  2046-01: 0.000000
  2047-01: 0.000000
  2048-01: 0.000000
  2049-01: 0.000000
  2050-01: 0.000000
  2051-01: 0.000000
  2052-01: 0.000000
  2053-01: 0.000000
  2054-01: 0.000000
  2055-01: 0.000000
  2056-01: 0.000000
  2057-01: 0.000000
  2058-01: 0.000000
  2059-01: 0.000000
  2060-01: 0.000000
  2061-01: 0.000000
  2062-01: 0.000000
  2063-01: 0.000000
  2064-01: 0.000000
  2065-01: 0.000000
}
curve cost_accretion {
  2025-01: 34.000000
  2026-01: 34.000000
  2027-01: 34.000000
  2028-01: 34.000000
  2029-01: 34.000000
  2030-01: 34.000000
  2031-01: 34.000000
  2032-01: 34.000000
  2033-01: 34.000000
  2034-01: 34.000000
  2035-01: 34.000000
  2036-01: 34.000000
  2037-01: 34.000000
  2038-01: 34.000000
  2039-01: 34.000000
  2040-01: 34.000000
  2041-01: 34.000000
  2042-01: 34.000000
  2043-01: 34.000000
  2044-01: 34.000000
  2045-01: 34.000000
  2046-01: 34.200000
  2047-01: 34.200000
  2048-01: 34.200000
  2049-01: 34.200000
  2050-01: 34.200000
  2051-01: 34.200000
  2052-01: 34.200000
  2053-01: 34.200000
  2054-01: 34.200000
  2055-01: 34.200000
  2056-01: 34.200000
  2057-01: 34.200000
  2058-01: 34.200000
  2059-01: 34.200000
  2060-01: 34.200000
  2061-01: 34.200000
  2062-01: 34.200000
  2063-01: 34.200000
  2064-01: 34.200000
  2065-01: 34.200000
}

curve capex {
  2025-01: 168.000000
  2026-01: 138.000000
  2027-01: 245.000000
  2028-01: 357.000000
  2029-01: 187.000000
  2030-01: 138.000000
  2031-01: 147.000000
  2032-01: 271.000000
  2033-01: 294.000000
  2034-01: 325.000000
  2035-01: 401.000000
  2036-01: 215.000000
  2037-01: 218.000000
  2038-01: 197.000000
  2039-01: 498.000000
  2040-01: 386.000000
  2041-01: 151.000000
  2042-01: 154.000000
  2043-01: 160.000000
  2044-01: 138.000000
  2045-01: 136.000000
  2046-01: 192.000000
  2047-01: 192.000000
  2048-01: 192.000000
  2049-01: 192.000000
  2050-01: 192.000000
  2051-01: 182.400000
  2052-01: 182.400000
  2053-01: 182.400000
  2054-01: 182.400000
  2055-01: 182.400000
  2056-01: 170.200000
  2057-01: 170.200000
  2058-01: 170.200000
  2059-01: 170.200000
  2060-01: 170.200000
  2061-01: 134.000000
  2062-01: 134.000000
  2063-01: 134.000000
  2064-01: 134.000000
  2065-01: 134.000000
}


curve working_capital {
  2025-01: 54.000000
  2026-01: -4.000000
  2027-01: -30.000000
  2028-01: -14.000000
  2029-01: 0.000000
  2030-01: 35.000000
  2031-01: -20.000000
  2032-01: 8.000000
  2033-01: -4.000000
  2034-01: 7.000000
  2035-01: 0.000000
  2036-01: -55.000000
  2037-01: -16.000000
  2038-01: -1.000000
  2039-01: 10.000000
  2040-01: -7.000000
  2041-01: 4.000000
  2042-01: 4.000000
  2043-01: 7.000000
  2044-01: 7.000000
  2045-01: -1.000000
  2046-01: -2.000000
  2047-01: -2.000000
  2048-01: -2.000000
  2049-01: -2.000000
  2050-01: -2.000000
  2051-01: 2.400000
  2052-01: 2.400000
  2053-01: 2.400000
  2054-01: 2.400000
  2055-01: 2.400000
  2056-01: -1.000000
  2057-01: -1.000000
  2058-01: -1.000000
  2059-01: -1.000000
  2060-01: -1.000000
  2061-01: -0.400000
  2062-01: -0.400000
  2063-01: -0.400000
  2064-01: -0.400000
  2065-01: -0.400000
}

// DEPRECIATION — READ THIS BEFORE COPYING THE PATTERN. A curve is the WRONG
// home for depreciation in a production model. Depreciation is not data; it
// is a consequence of capital — a rule (straight-line, declining-balance,
// units-of-production) applied to the assets the capex creates, and it
// belongs in a calculated series driven by that rule.
//
// This case cannot do that. The filing states no method, no asset lives and
// no opening basis; it prints only EBITDA and pre-tax gross income, adjacent
// rows whose gap IS depreciation. So the schedule below is RECOVERED DATA —
// the printed gap, inverted through the fiscal identities:
//
//     ebitda - dep - royalty = gross / 0.9   when gross income is positive
//                            = gross          otherwise (PTU floors at zero)
//
// Inventing a depreciation rule the filing does not state would be fitting
// unstated mechanics. Carrying the recovered series as data is the fidelity
// the source supports, and the compromise this case makes.
//
// One rule IS applied to it: the fiscal streams scale this curve by
// cfg.capex_factor, because the filing's own capital sensitivity reprices
// the depreciation that capital creates. Holding it fixed puts the capital
// row of Table 19.2 out by 125 US$ M; scaling it brings that row to 12.
curve depreciation {
  2025-01: 14.322222
  2026-01: 25.855556
  2027-01: 46.013889
  2028-01: 74.850000
  2029-01: 90.783333
  2030-01: 102.966667
  2031-01: 115.613889
  2032-01: 138.119444
  2033-01: 162.247222
  2034-01: 187.719444
  2035-01: 218.091667
  2036-01: 234.150000
  2037-01: 237.725000
  2038-01: 240.800000
  2039-01: 253.475000
  2040-01: 250.100000
  2041-01: 245.375000
  2042-01: 247.400000
  2043-01: 248.977778
  2044-01: 237.241667
  2045-01: 223.925000
  2046-01: 216.125000
  2047-01: 216.125000
  2048-01: 216.125000
  2049-01: 216.125000
  2050-01: 216.125000
  2051-01: 208.030000
  2052-01: 208.030000
  2053-01: 208.030000
  2054-01: 208.030000
  2055-01: 208.030000
  2056-01: 207.104444
  2057-01: 207.104444
  2058-01: 207.104444
  2059-01: 207.104444
  2060-01: 207.104444
  2061-01: 167.504444
  2062-01: 167.504444
  2063-01: 167.504444
  2064-01: 167.504444
  2065-01: 167.504444
}

// ---------------------------------------------------------------------------
// The mine's lifecycle. Its type declares no lifecycle, so the states are
// open-world: each event writes `status`, the write is published in
// deterministic.transitions, and streams gate on it with `active when`.
// This is a linear, two-transition state machine — the degenerate case.
// The transitions are plan facts (Concentrator I offline at end-2035 per
// section 19.2, cutting ore processed by 40%; reclamation from 2061), so
// they fire on phase boundaries rather than on modeled conditions. A mine
// whose regime moved on price or grade would put a condition in the `when`
// and the same machinery would carry it.
// ---------------------------------------------------------------------------

// The grammar declares `phase_enter` for schedule position only, so an
// event states its boundary by period index. One period is one year: t=11
// is 2036, t=36 is 2061. The phase declarations above name the same eras;
// the dates appear in both places by necessity, not by choice.
event concentrator_one_offline when time.t >= 11 {
  set entity asset.mine.status = "reduced"
}

event closure_era_opens when time.t >= 36 {
  set entity asset.mine.status = "closing"
}

entity asset mine : Asset.Real {
  shelter init min(0.0, ((((cfg.price_cu * curve_value("cu_payable", time.date)
                 + cfg.price_mo * curve_value("mo_payable", time.date)
                 + cfg.price_zn * curve_value("zn_payable", time.date))
                 - (cfg.opex_factor
                 * (curve_value("cost_mining", time.date)
                    + curve_value("cost_concentrator", time.date)
                    + curve_value("cost_smelting", time.date)
                    + curve_value("cost_gna", time.date)
                    + curve_value("cost_decommissioning", time.date)
                    + curve_value("cost_accretion", time.date)))) - (cfg.capex_factor * curve_value("depreciation", time.date)) - (inputs.duty_rate * ((cfg.price_cu * curve_value("cu_payable", time.date)
                 + cfg.price_mo * curve_value("mo_payable", time.date)
                 + cfg.price_zn * curve_value("zn_payable", time.date))
                 - (cfg.opex_factor
                 * (curve_value("cost_mining", time.date)
                    + curve_value("cost_concentrator", time.date)
                    + curve_value("cost_smelting", time.date)
                    + curve_value("cost_gna", time.date)
                    + curve_value("cost_decommissioning", time.date)
                    + curve_value("cost_accretion", time.date)))))) - (inputs.ptu_rate * max(0.0, (((cfg.price_cu * curve_value("cu_payable", time.date)
                 + cfg.price_mo * curve_value("mo_payable", time.date)
                 + cfg.price_zn * curve_value("zn_payable", time.date))
                 - (cfg.opex_factor
                 * (curve_value("cost_mining", time.date)
                    + curve_value("cost_concentrator", time.date)
                    + curve_value("cost_smelting", time.date)
                    + curve_value("cost_gna", time.date)
                    + curve_value("cost_decommissioning", time.date)
                    + curve_value("cost_accretion", time.date)))) - (cfg.capex_factor * curve_value("depreciation", time.date)) - (inputs.duty_rate * ((cfg.price_cu * curve_value("cu_payable", time.date)
                 + cfg.price_mo * curve_value("mo_payable", time.date)
                 + cfg.price_zn * curve_value("zn_payable", time.date))
                 - (cfg.opex_factor
                 * (curve_value("cost_mining", time.date)
                    + curve_value("cost_concentrator", time.date)
                    + curve_value("cost_smelting", time.date)
                    + curve_value("cost_gna", time.date)
                    + curve_value("cost_decommissioning", time.date)
                    + curve_value("cost_accretion", time.date))))))))))
    next min(0.0, prev + ((((cfg.price_cu * curve_value("cu_payable", time.date)
                 + cfg.price_mo * curve_value("mo_payable", time.date)
                 + cfg.price_zn * curve_value("zn_payable", time.date))
                 - (cfg.opex_factor
                 * (curve_value("cost_mining", time.date)
                    + curve_value("cost_concentrator", time.date)
                    + curve_value("cost_smelting", time.date)
                    + curve_value("cost_gna", time.date)
                    + curve_value("cost_decommissioning", time.date)
                    + curve_value("cost_accretion", time.date)))) - (cfg.capex_factor * curve_value("depreciation", time.date)) - (inputs.duty_rate * ((cfg.price_cu * curve_value("cu_payable", time.date)
                 + cfg.price_mo * curve_value("mo_payable", time.date)
                 + cfg.price_zn * curve_value("zn_payable", time.date))
                 - (cfg.opex_factor
                 * (curve_value("cost_mining", time.date)
                    + curve_value("cost_concentrator", time.date)
                    + curve_value("cost_smelting", time.date)
                    + curve_value("cost_gna", time.date)
                    + curve_value("cost_decommissioning", time.date)
                    + curve_value("cost_accretion", time.date)))))) - (inputs.ptu_rate * max(0.0, (((cfg.price_cu * curve_value("cu_payable", time.date)
                 + cfg.price_mo * curve_value("mo_payable", time.date)
                 + cfg.price_zn * curve_value("zn_payable", time.date))
                 - (cfg.opex_factor
                 * (curve_value("cost_mining", time.date)
                    + curve_value("cost_concentrator", time.date)
                    + curve_value("cost_smelting", time.date)
                    + curve_value("cost_gna", time.date)
                    + curve_value("cost_decommissioning", time.date)
                    + curve_value("cost_accretion", time.date)))) - (cfg.capex_factor * curve_value("depreciation", time.date)) - (inputs.duty_rate * ((cfg.price_cu * curve_value("cu_payable", time.date)
                 + cfg.price_mo * curve_value("mo_payable", time.date)
                 + cfg.price_zn * curve_value("zn_payable", time.date))
                 - (cfg.opex_factor
                 * (curve_value("cost_mining", time.date)
                    + curve_value("cost_concentrator", time.date)
                    + curve_value("cost_smelting", time.date)
                    + curve_value("cost_gna", time.date)
                    + curve_value("cost_decommissioning", time.date)
                    + curve_value("cost_accretion", time.date))))))))))

  shelter_in init 0.0
    next prev.asset.mine.shelter
}

// ---------------------------------------------------------------------------
// The fiscal state. `shelter` is the loss carried forward, held as a negative
// number or zero; it is the only value in the model that depends on a previous
// period. It is not decoration: the filing prints no tax in 2043, 2044 or 2045
// although gross income is positive in each, because 2037-2042 ran at a loss.
//
// The remaining fields are published so the derivation can be inspected and
// asserted period by period. They are entity fields rather than streams
// because they are intermediate quantities, not cash -- the cash they imply is
// carried by the royalty, PTU and tax streams below.
// ---------------------------------------------------------------------------


// ---------------------------------------------------------------------------
// Revenue: three metals, each a payable quantity at its own price.
// ---------------------------------------------------------------------------

stream mine.revenue.copper on entity asset.mine inflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.revenue.recurring
  amount = cfg.price_cu * curve_value("cu_payable", time.date)
}

stream mine.revenue.molybdenum on entity asset.mine inflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.revenue.recurring
  amount = cfg.price_mo * curve_value("mo_payable", time.date)
}

stream mine.revenue.zinc on entity asset.mine inflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.revenue.recurring
  amount = cfg.price_zn * curve_value("zn_payable", time.date)
}

// ---------------------------------------------------------------------------
// Operating cost: the six lines the filing prints, kept separate so a break
// localises to one of them rather than to their sum.
// ---------------------------------------------------------------------------

stream mine.opex.mining on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.expense.opex
  amount = cfg.opex_factor * curve_value("cost_mining", time.date)
}

stream mine.opex.concentrator on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.expense.opex
  amount = cfg.opex_factor * curve_value("cost_concentrator", time.date)
}

stream mine.opex.smelting on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.expense.opex
  amount = cfg.opex_factor * curve_value("cost_smelting", time.date)
}

stream mine.opex.gna on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.expense.opex
  amount = cfg.opex_factor * curve_value("cost_gna", time.date)
}

stream mine.opex.decommissioning on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.expense.opex
  amount = cfg.opex_factor * curve_value("cost_decommissioning", time.date)
}

stream mine.opex.accretion on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.expense.opex
  amount = cfg.opex_factor * curve_value("cost_accretion", time.date)
}

// ---------------------------------------------------------------------------
// The fiscal charges, as cash. Each repeats its definition rather than reading
// the field of the same name: a stream may reach a previous period through
// `prev`, not the current one, and these all settle in the period they arise.
// ---------------------------------------------------------------------------

stream mine.fiscal.royalty on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.tax
  amount = inputs.duty_rate * (series_sum("mine.revenue.*", time.t, time.t)
             + series_sum("mine.opex.*", time.t, time.t))
}

stream mine.fiscal.ptu on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.tax
  amount = inputs.ptu_rate
             * max(0.0, (1.0 - inputs.duty_rate) * (series_sum("mine.revenue.*", time.t, time.t)
             + series_sum("mine.opex.*", time.t, time.t))
                        - cfg.capex_factor * curve_value("depreciation", time.date))
}

stream mine.fiscal.income_tax on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.tax
  amount = max(0.0,
               inputs.tax_rate
                 * ((1.0 - inputs.duty_rate) * (series_sum("mine.revenue.*", time.t, time.t)
             + series_sum("mine.opex.*", time.t, time.t))
                    - cfg.capex_factor * curve_value("depreciation", time.date)
                    - inputs.ptu_rate
                        * max(0.0, (1.0 - inputs.duty_rate) * (series_sum("mine.revenue.*", time.t, time.t)
             + series_sum("mine.opex.*", time.t, time.t))
                                   - cfg.capex_factor * curve_value("depreciation", time.date))
                    + asset.mine.shelter_in)
               - inputs.tax_rate * inputs.duty_rate * (series_sum("mine.revenue.*", time.t, time.t)
             + series_sum("mine.opex.*", time.t, time.t)))
}

// The ARO accretion is charged in operating cost and never leaves the bank.
// The filing's sibling report prints this as its own "Add back Accretion" row.
stream mine.noncash.accretion_addback on entity asset.mine inflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.expense.opex
  amount = cfg.opex_factor * curve_value("cost_accretion", time.date)
}

// ---------------------------------------------------------------------------
// Capital, closure and working capital. Working capital carries the filing's
// sign -- a positive number is a use of cash -- and nets to nothing over the
// life of the mine.
// ---------------------------------------------------------------------------

stream mine.capital.capex on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category investing.capital.capex
  amount = cfg.capex_factor * curve_value("capex", time.date)
}

stream mine.capital.closure on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category investing.capital.capex
  active when entity.status == "closing"
  amount = inputs.closure_total / 5.0
}

stream mine.capital.working_capital on entity asset.mine outflow currency USD {
  schedule every year due from 2025-01 to 2065-01
  category operating.working_capital
  amount = curve_value("working_capital", time.date)
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.1,
    "parameters": {
      "cfg.price_cu": 3.3,
      "cfg.price_mo": 10.0,
      "cfg.price_zn": 1.15,
      "cfg.opex_factor": 1.0,
      "cfg.capex_factor": 1.0
    }
  },
  "scenarios": {
    "opex_m30": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 0.7,
        "cfg.capex_factor": 1.0
      }
    },
    "opex_m25": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 0.75,
        "cfg.capex_factor": 1.0
      }
    },
    "opex_m20": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 0.8,
        "cfg.capex_factor": 1.0
      }
    },
    "opex_m15": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 0.85,
        "cfg.capex_factor": 1.0
      }
    },
    "opex_m10": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 0.9,
        "cfg.capex_factor": 1.0
      }
    },
    "opex_m5": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 0.95,
        "cfg.capex_factor": 1.0
      }
    },
    "opex_p5": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.05,
        "cfg.capex_factor": 1.0
      }
    },
    "opex_p10": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.1,
        "cfg.capex_factor": 1.0
      }
    },
    "opex_p15": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.15,
        "cfg.capex_factor": 1.0
      }
    },
    "opex_p20": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.2,
        "cfg.capex_factor": 1.0
      }
    },
    "opex_p25": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.25,
        "cfg.capex_factor": 1.0
      }
    },
    "opex_p30": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.3,
        "cfg.capex_factor": 1.0
      }
    },
    "capex_m30": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 0.7
      }
    },
    "capex_m25": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 0.75
      }
    },
    "capex_m20": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 0.8
      }
    },
    "capex_m15": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 0.85
      }
    },
    "capex_m10": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 0.9
      }
    },
    "capex_m5": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 0.95
      }
    },
    "capex_p5": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.05
      }
    },
    "capex_p10": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.1
      }
    },
    "capex_p15": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.15
      }
    },
    "capex_p20": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.2
      }
    },
    "capex_p25": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.25
      }
    },
    "capex_p30": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.3
      }
    },
    "commodity_m30": {
      "parameters": {
        "cfg.price_cu": 2.31,
        "cfg.price_mo": 7.0,
        "cfg.price_zn": 0.805,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "commodity_m25": {
      "parameters": {
        "cfg.price_cu": 2.475,
        "cfg.price_mo": 7.5,
        "cfg.price_zn": 0.8625,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "commodity_m20": {
      "parameters": {
        "cfg.price_cu": 2.64,
        "cfg.price_mo": 8.0,
        "cfg.price_zn": 0.92,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "commodity_m15": {
      "parameters": {
        "cfg.price_cu": 2.805,
        "cfg.price_mo": 8.5,
        "cfg.price_zn": 0.9775,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "commodity_m10": {
      "parameters": {
        "cfg.price_cu": 2.97,
        "cfg.price_mo": 9.0,
        "cfg.price_zn": 1.035,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "commodity_m5": {
      "parameters": {
        "cfg.price_cu": 3.135,
        "cfg.price_mo": 9.5,
        "cfg.price_zn": 1.0925,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "commodity_p5": {
      "parameters": {
        "cfg.price_cu": 3.465,
        "cfg.price_mo": 10.5,
        "cfg.price_zn": 1.2075,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "commodity_p10": {
      "parameters": {
        "cfg.price_cu": 3.63,
        "cfg.price_mo": 11.0,
        "cfg.price_zn": 1.265,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "commodity_p15": {
      "parameters": {
        "cfg.price_cu": 3.795,
        "cfg.price_mo": 11.5,
        "cfg.price_zn": 1.3225,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "commodity_p20": {
      "parameters": {
        "cfg.price_cu": 3.96,
        "cfg.price_mo": 12.0,
        "cfg.price_zn": 1.38,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "commodity_p25": {
      "parameters": {
        "cfg.price_cu": 4.125,
        "cfg.price_mo": 12.5,
        "cfg.price_zn": 1.4375,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "commodity_p30": {
      "parameters": {
        "cfg.price_cu": 4.29,
        "cfg.price_mo": 13.0,
        "cfg.price_zn": 1.495,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "copper_m30": {
      "parameters": {
        "cfg.price_cu": 2.31,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "copper_m25": {
      "parameters": {
        "cfg.price_cu": 2.475,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "copper_m20": {
      "parameters": {
        "cfg.price_cu": 2.64,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "copper_m15": {
      "parameters": {
        "cfg.price_cu": 2.805,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "copper_m10": {
      "parameters": {
        "cfg.price_cu": 2.97,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "copper_m5": {
      "parameters": {
        "cfg.price_cu": 3.135,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "copper_p5": {
      "parameters": {
        "cfg.price_cu": 3.465,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "copper_p10": {
      "parameters": {
        "cfg.price_cu": 3.63,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "copper_p15": {
      "parameters": {
        "cfg.price_cu": 3.795,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "copper_p20": {
      "parameters": {
        "cfg.price_cu": 3.96,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "copper_p25": {
      "parameters": {
        "cfg.price_cu": 4.125,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "copper_p30": {
      "parameters": {
        "cfg.price_cu": 4.29,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "molybdenum_m30": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 7.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "molybdenum_m25": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 7.5,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "molybdenum_m20": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 8.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "molybdenum_m15": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 8.5,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "molybdenum_m10": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 9.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "molybdenum_m5": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 9.5,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "molybdenum_p5": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.5,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "molybdenum_p10": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 11.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "molybdenum_p15": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 11.5,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "molybdenum_p20": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 12.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "molybdenum_p25": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 12.5,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "molybdenum_p30": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 13.0,
        "cfg.price_zn": 1.15,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "zinc_m30": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 0.805,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "zinc_m25": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 0.8625,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "zinc_m20": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 0.92,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "zinc_m15": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 0.9775,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "zinc_m10": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.035,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "zinc_m5": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.0925,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "zinc_p5": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.2075,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "zinc_p10": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.265,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "zinc_p15": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.3225,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "zinc_p20": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.38,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "zinc_p25": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.4375,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    },
    "zinc_p30": {
      "parameters": {
        "cfg.price_cu": 3.3,
        "cfg.price_mo": 10.0,
        "cfg.price_zn": 1.495,
        "cfg.opex_factor": 1.0,
        "cfg.capex_factor": 1.0
      }
    }
  }
}
```

## Verified results

Checked period by period: **18 series** across **41 periods** — **738 values** in all, each within ±0.00001 of the reference.

- `mine.revenue.copper`
- `mine.revenue.molybdenum`
- `mine.revenue.zinc`
- `mine.opex.mining`
- `mine.opex.concentrator`
- `mine.opex.smelting`
- `mine.opex.gna`
- `mine.opex.decommissioning`
- `mine.opex.accretion`
- `mine.capital.capex`
- `mine.capital.closure`
- `mine.capital.working_capital`
- `mine.fiscal.royalty`
- `mine.fiscal.ptu`
- `mine.fiscal.income_tax`
- `mine.noncash.accretion_addback`
- `asset.mine.shelter`
- `net_cash_flow`

Checked per scenario, each a full run under its own parameters:

| Scenario | `model.npv` |
|---|---:|
| `opex_m30` | 6,652.44 |
| `opex_m25` | 6,121.9 |
| `opex_m20` | 5,591.35 |
| `opex_m15` | 5,060.81 |
| `opex_m10` | 4,520.02 |
| `opex_m5` | 3,974.07 |
| `opex_p5` | 2,811.35 |
| `opex_p10` | 2,210.3 |
| `opex_p15` | 1,606.95 |
| `opex_p20` | 1,002.39 |
| `opex_p25` | 397.821038 |
| `opex_p30` | -206.750667 |
| `capex_m30` | 4,013.07 |
| `capex_m25` | 3,914.95 |
| `capex_m20` | 3,816.35 |
| `capex_m15` | 3,712.87 |
| `capex_m10` | 3,610.31 |
| `capex_m5` | 3,506.72 |
| `capex_p5` | 3,298.18 |
| `capex_p10` | 3,192.92 |
| `capex_p15` | 3,086 |
| `capex_p20` | 2,978.01 |
| `capex_p25` | 2,870.01 |
| `capex_p30` | 2,761.65 |
| `commodity_m30` | -1,973.01 |
| `commodity_m25` | -1,012.75 |
| `commodity_m20` | -123.178745 |
| `commodity_m15` | 762.785831 |
| `commodity_m10` | 1,648.33 |
| `commodity_m5` | 2,531.09 |
| `commodity_p5` | 4,244.74 |
| `commodity_p10` | 5,061.1 |
| `commodity_p15` | 5,865.89 |
| `commodity_p20` | 6,664.79 |
| `commodity_p25` | 7,463.7 |
| `commodity_p30` | 8,262.6 |
| `copper_m30` | -1,501.83 |
| `copper_m25` | -668.787862 |
| `copper_m20` | 149.105622 |
| `copper_m15` | 966.999107 |
| `copper_m10` | 1,784.29 |
| `copper_m5` | 2,598.89 |
| `copper_p5` | 4,180.23 |
| `copper_p10` | 4,932.59 |
| `copper_p15` | 5,677.4 |
| `copper_p20` | 6,414.27 |
| `copper_p25` | 7,150.54 |
| `copper_p30` | 7,886.82 |
| `molybdenum_m30` | 3,214.83 |
| `molybdenum_m25` | 3,246.26 |
| `molybdenum_m20` | 3,277.64 |
| `molybdenum_m15` | 3,308.88 |
| `molybdenum_m10` | 3,340.23 |
| `molybdenum_m5` | 3,371.59 |
| `molybdenum_p5` | 3,433.86 |
| `molybdenum_p10` | 3,465.27 |
| `molybdenum_p15` | 3,496.3 |
| `molybdenum_p20` | 3,527.33 |
| `molybdenum_p25` | 3,558.7 |
| `molybdenum_p30` | 3,589.67 |
| `zinc_m30` | 3,194.13 |
| `zinc_m25` | 3,228.97 |
| `zinc_m20` | 3,263.71 |
| `zinc_m15` | 3,298.53 |
| `zinc_m10` | 3,333.2 |
| `zinc_m5` | 3,368.15 |
| `zinc_p5` | 3,437.4 |
| `zinc_p10` | 3,472.07 |
| `zinc_p15` | 3,506.98 |
| `zinc_p20` | 3,541.56 |
| `zinc_p25` | 3,576.15 |
| `zinc_p30` | 3,610.74 |

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.npv` | 3,402.77 | ±0.0001 |
| `stream.mine.revenue.copper.total` | 71,442 | ±0.0001 |
| `stream.mine.capital.capex.total` | -8,317 | ±0.0001 |
