## The case

A stabilized 10,000 square foot commercial building, bought for $1,417,958 —
year-one net operating income capped at 6.0% — held for ten years and sold by
capping the tenth year's net operating income at 6.5%, net of 2% selling costs.

The building is fully leased for the whole hold with no rollover and no
lease-up, so its income is a single stabilized stream rather than a rent roll:
potential gross income of $100,000, full reimbursement of the three recoverable
expenses, and a 10% vacancy and credit deduction taken against the two
together. The leases reimburse 100% of real estate taxes, insurance and common
area maintenance; management is 3% of effective gross revenue; a capital
reserve runs at $0.20 per square foot. Every line escalates at 3% a year.

The interest of it is the shape at the two ends. The purchase settles at the
open of the first year and earns no discount; the sale settles at the close of
the tenth, alongside that year's operating cash, and is struck on the net
operating income of the year that has just finished rather than a forward year.

## The reference

A published teaching model for a basic real estate acquisition, distributed as
a spreadsheet. It states every driver — year-one income, each expense, the
management fee, the reserve, the escalator, both cap rates, the selling cost —
and publishes the resulting cash flow line by line together with its own
internal rate of return, equity multiple and net present value.

**Not redistributable.** No terms are stated by the publisher, so the figures
are asserted against and the workbook is not vendored. `NOTES.md` records what
it is and where it came from.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Contract types | `cre.revenue_line`, `cre.vacancy_loss`, `cre.opex_line`, `cre.exit_cap` |
| Declared | 8 contracts, 3 native streams, one entity, no curves |
| Language features | a flow settled at a period's open against flows settled at its close; a contract term carrying an expression rather than a scalar |
| Conventions | annual escalation compounded from the model's start, a vacancy deduction on income plus recoveries, an exit struck on trailing rather than forward income |

The acquisition price, the capital reserve and the selling costs are declared
as streams; everything else is a pack contract.

The vacancy deduction is taken on potential gross income *plus* reimbursements,
so its base is stated as an expression that escalates with the two lines it
sits on rather than as a fixed figure.

## The result

`model.npv` at 7% = **90,853.72729** against the published 90,853.72728969366,
`model.irr` = **0.078484** against the published 0.07848381094537493, and the
equity multiple **1.851981** against the published 1.8519810968410686. All
three exact.

Every line reproduces per period: potential gross income, reimbursements, the
vacancy deduction, all four operating expenses, the capital reserve, the
purchase, the sale at 1,707,797.546881 and its selling costs.

## The delta

None. Every asserted figure is the source's own and reproduces exactly.

The equity multiple is measured on the capital invested — the acquisition —
with the ten years of operations counted as what the investment returned.
