## The case

A cross-industry operating company valued on free cash flow to the firm: revenue
growing off a declining growth path, operating margins, cash taxes, capital
expenditure and working capital, discounted to an enterprise value.

The rate declines year on year, so revenue is a running product of ten
different growth rates rather than one rate compounded.

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
| Contract types | `opco.revenue_line`, `opco.opex_line`, `opco.capex_line`, `opco.cash_taxes` |
| Declared | two curves |
| Language features | pack contracts driven by curves; declared state inside the pack's growth rules |
| Conventions | a declining growth path, margin-driven operating expense, cash taxes, capital expenditure as a share of revenue |

The reference publishes the **drivers** rather than only the results, which is
what a pack rule consumes, so the pack's lowering is checked and not only the
engine's arithmetic.

## The result

All ten years reproduce exactly.

Revenue is carried as declared state because the growth rate moves:
`pow(1 + g, t)` applies one year's rate as though it had held from the start,
which is exact only while the rate is constant.

## The delta

None.
