## The case

Buenavista del Cobre is an open-pit copper mine in Sonora, Mexico. It has
operated since 1899 and is among the largest copper mines in the world. The
reserve supports 41 more years of production, through 2065: 2.1 billion tonnes
of mill ore, 2.1 billion tonnes of leach ore, 296 million tonnes of zinc ore,
and 3.8 billion tonnes of waste. The products are copper, molybdenum and zinc.

The modeling problem is the Mexican fiscal stack. Four charges sit between
EBITDA and net income, and each base is defined in terms of the others. The
Derechos de Mineria takes 7.5% of earnings. An employee profit share takes 10%
of what remains after depreciation and the duty. Income tax takes 30% of what
remains after that. The duty is then credited against the tax at 30%. Read as
levies on earnings, the four look mutually circular. They are not, and the
case shows that they resolve in one pass with no solver.

## The reference

The reference is Table 19.1, "Discounted Cash Flow", of the S-K 1300 Technical
Report Summary for the mine. WSP USA prepared the report for Southern Copper
Corporation, dated 11 February 2025 and filed as Exhibit 96.6 to the FY2024
Form 10-K. The table prints the whole life of the mine: material movement,
revenue by metal, six cost lines, EBITDA, gross income, tax, capital, closure,
working capital, and a pre-tax and an after-tax NPV at a stated 10%. Table
19.2 adds a sensitivity matrix of 78 published after-tax NPVs: six variables
at thirteen steps each. Both tables are transcribed here. The PDF is a public
filing and is cited rather than vendored.

The fiscal structure was not taken from this deal. Buenavista prints EBITDA
and gross income but not the charges between them, so a structure fitted here
would be fitted to the answer. The structure was read from a different mine:
La Caridad/Pilares, Exhibit 96.7 of the same Form 10-K, by the same author on
the same template. That table prints all ten intermediate lines, including the
rows Buenavista omits: Depreciation, Royalty, PTU, Minimum tax, Income tax,
and both add-backs. The recovered structure reproduces all ten of La Caridad's
printed lines to within 0.77 US$ M. Applied unchanged to Buenavista, it
reproduces the printed tax, net income and after-tax cash flow to within 1.44.

An independent implementation of these conventions produces the expectations
this case asserts. The case is therefore checked twice: the reference against
the filing, and CFDL against the reference.

## What it exercises

| | |
|---|---|
| Pack | none — written from the bare language |
| Entities | one real asset, carrying its own lifecycle and its one memory |
| Language features | second-tier streams reading the period's result through `series_sum`, open-world lifecycle events with published transitions, declared phases, a carryforward recurrence, annuity-due placement, run-config parameters driving 72 scenarios |
| Conventions | duty on EBITDA, profit share on EBITDA net of depreciation and duty, income tax net of a duty credit, loss carryforward, first year undiscounted |

This is the second case in the suite written without a pack, after
`ppiaf_toll_highway`. A mine fits none of the four packs. It has no generation
and no offtaker, no rent roll, and no pool of obligors. Its revenue is
contained metal at a price, not a margin on sales.

**The stack resolves in one pass.** Each charge's base is settled before the
charge applies:

    royalty      = 7.5% × ebitda
    ptu          = 10% × max(0, ebitda − depreciation − royalty)
    gross_income = ebitda − depreciation − royalty − ptu
    total_taxes  = max(0, 30% × (gross_income + shelter) − 30% × royalty)

Gross income is exactly 0.9 × (ebitda − depreciation − royalty), so the profit
share is one ninth of gross income. This identity is why the stack evaluates
without iteration. It is also what recovers depreciation, which this mine does
not publish, from the two lines it does.

**The carryforward is necessary, not decorative.** The filing prints no tax in
2043, 2044 or 2045, although gross income is positive in each. The cause is
the losses of 2037 through 2042. Without the shelter, those three years are
wrong by 46 US$ M while every other column still passes.

**Depreciation must scale with capital.** The filing's capital sensitivity
reprices the depreciation that capital creates. Held fixed, the capital row of
Table 19.2 is out by 125 US$ M. Scaled with `cfg.capex_factor`, the same row
is within 12.

## The result

All 41 periods reproduce across nineteen columns to 1e-5, which is the float
noise of the price-times-quantity round trip. The columns are three revenue
lines, six cost lines, three fiscal charges, the accretion add-back, three
capital lines, the loss carryforward as the mine's own field, and net cash
flow. EBITDA appears in no column and no curve. It is the result of the base
streams, and the fiscal streams read it from the period's realized series.
Three metrics and 72 scenarios reproduce on the same tolerance. The scenarios
cover all six variables of Table 19.2 at every non-zero step.

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

The units are US$ M, against cells the filing rounds to US$ 1 M. Both
published NPVs are within 0.10%: pre-tax 5,820.2 against 5,826, and after-tax
3,402.8 against 3,405. All 78 sensitivity points are within 1.57% of the base
NPV, and four of the six rows are within 0.5%.

## The delta

**The per-column residuals are rounding, not error.** The filing rounds every
cell to the nearest million and states the rounding in a table note. A derived
line is a sum or difference of rounded cells. Its bound is therefore the
number of cells it touches times a half million, plus a half for the printed
figure it is compared against: 2.00 for EBITDA, which touches nine. No line
exceeds its bound, and most sit at a third of it. The filing is not internally
exact either: its printed revenue cells sum to 76,951 against its own printed
total of 76,952.

**Both NPV residuals have one cause, and it is an assumption rather than
rounding.** The filing publishes 2046 through 2065 only as four five-year
buckets. The model divides each bucket evenly across its five years. The true
profile inside a bucket is not in the document and cannot be recovered from
it. Nothing else in the model is approximate.

**The same assumption causes most of the sensitivity error.** The two price
rows drift furthest at ±30%, where a large price move changes which years the
loss shelter covers. An averaged driver is safe for a linear line and unsafe
for a line with a threshold. The shelter is a threshold: flat bucket income
never trips it where lumpy income would. Read any case that smooths an input
with this in mind.

**What the case does not claim.** The parent 10-K confirms a 0.5% additional
royalty on gold, silver and platinum receipts. It is not modeled, because this
mine's published revenue carries only copper, molybdenum and zinc, so the levy
cannot be sized from Table 19.1. The report also applies a 7.5% duty across a
forecast that begins on 1 January 2025, although the 10-K records the Ley
Federal de Derechos raising the rate to 8.5% from that date. The case
reproduces what the filing computed. It does not assert that the filing is
right.
