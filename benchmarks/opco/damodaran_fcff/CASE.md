## The case

A cross-industry operating company valued on free cash flow to the firm: revenue
growing off a declining growth path, operating margins, cash taxes, capital
expenditure and working capital, discounted to an enterprise value.

The rate declines year on year, so revenue is a running product of ten
different growth rates rather than one rate compounded. The cost of capital
converges the same way, and the run discounts each year at that year's rate.

## The reference

A widely used academic valuation spreadsheet, published free by its author with
an explicit grant to download and modify. It publishes the full ten-year build-up
and the resulting value.

**Redistributable**, and the workbook is committed under `reference/` so a reader
can mark every figure against the original.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Contract types | `opco.revenue_line`, `opco.opex_line`, `opco.reinvestment`, `opco.cash_taxes` |
| Declared | three curves, one of them the cost of capital the run discounts along |
| Language features | pack contracts driven by curves; declared state inside the pack's growth rules |
| Conventions | a declining growth path, margin-driven operating expense, cash taxes, reinvestment funding next year's growth, a cost of capital that converges over the forecast |

The reference publishes the **drivers** rather than only the results, which is
what a pack rule consumes, so the pack's lowering is checked and not only the
engine's arithmetic.

## The result

All ten years reproduce exactly, reinvestment included, and the present value
of the ten years of free cash flow along the converging cost of capital is the
workbook's own figure.

Revenue is carried as declared state because the growth rate moves:
`pow(1 + g, t)` applies one year's rate as though it had held from the start,
which is exact only while the rate is constant. Reinvestment is derived from
revenue and the following year's growth, which the model reads a year ahead.

## The delta

None.
