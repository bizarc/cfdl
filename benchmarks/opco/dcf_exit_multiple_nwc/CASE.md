## The case

A practitioner discounted cash flow of a mid-market operating company, valued
at 31 December 2017 over a five-year forecast. Revenue grows from 121 to 154,
EBITDA margin holds near 36–37%, and the terminal year holds revenue, EBITDA,
capital expenditure and working capital flat while depreciation steps to 95% of
terminal capex.

What distinguishes it is that the template computes its terminal value **both
ways** — perpetuity growth and a multiple of trailing EBITDA — and publishes a
full three-by-three output grid for each, so the two methods can be read against
one another. This case asserts the exit-multiple arm.

The two methods do not discount alike, and the difference is in the formulas
rather than the prose: the exit-multiple terminal is discounted over 5.0 years
and the perpetuity terminal over 4.5. A terminal value is a price struck at a
point in time, and `opco.exit_multiple` places a disposal at the end of the
holding period, so the exit-multiple arm is the one the contract can spell.

## The reference

A published DCF template from a practitioner site, downloadable without
registration, carrying its inputs as typed constants and its outputs as live
formulas. It states the working-capital line and the exit multiple explicitly
rather than deriving either, which is why this source and not another.

**Not redistributable.** The publisher reserves copyright, so the figures are
asserted against and the workbook is not vendored — the same posture as
`banker_dcf_conventions`. `NOTES.md` records the URL, size and SHA-256 so a
reader can fetch the identical file.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Contract types | `opco.revenue_line`, `opco.opex_line`, `opco.cash_taxes`, `opco.capex_line`, `opco.working_capital`, `opco.exit_multiple` |
| Declared | 21 contracts, one entity, no curves, no native streams |
| Language features | a contract term restricted to a single period, to place an annual flow on a monthly grid |
| Conventions | mid-year discounting of flows, a terminal value discounted whole, depreciation carried as a tax term rather than a cash line |

It closes the last two unexercised types on the `opco` roster:
`opco.working_capital` — the fixed variant, as against the DSO/DPO
`working_capital_policy` that `lbo_buyout` covers — and `opco.exit_multiple`,
the stated-base form, as against the stream-derived `opco.exit_ebitda`.

Depreciation never becomes a flow. The published build subtracts it to reach
EBIT, taxes that, then adds it straight back, so it touches cash only through
the tax — which is what `opco.cash_taxes` carries `da_monthly` for. Nothing
here books a depreciation stream, and the model needs no native stream at all.

The whole model is built from pack contracts, which makes the pack's lowering
the thing under test rather than only the engine's arithmetic.

## The result

`model.npv` = **338.366157** against the template's published 338.3661574792812,
at 10.0% WACC and a 7.0x LTM EBITDA multiple — the low-rate, low-multiple cell
of the published grid.

Every line of the cash flow build reproduces exactly: revenue, cash operating
expense, cash taxes, capital expenditure and the increase in net working
capital, each asserted per period against the template's own figure, and the
terminal value at 401.396044.

All nine cells of the exit-multiple grid reconcile, the worst by 4.8e-7. The
other eight need a different discount rate or multiple per run, so the asserted
cell is one of them and the rest are in `NOTES.md`.

## The delta

None. The residual on every cell is within ±5e-7, which is the results
document's six-decimal rounding rather than a disagreement — unlike
`banker_dcf_conventions`, whose source rounded to whole millions and whose
tolerance had to be set by that.

The tolerance is 1e-5 on the metric and 1e-3 per period, both set by the
rounding and neither by the arithmetic.
