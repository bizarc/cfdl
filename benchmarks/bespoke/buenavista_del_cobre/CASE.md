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
| Entities | one real asset, one financial state entity |
| Language features | declared curves, a two-field carryforward recurrence, annuity-due placement, run-config parameters driving 72 scenarios |
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

All 41 periods reproduce across twenty columns — three revenue lines, six
cost lines, three fiscal charges, the accretion add-back, three capital lines,
three published fields and net cash flow — to 1e-5, the float noise of the
price-times-quantity round trip. Three metrics and **72 scenarios** reproduce
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
