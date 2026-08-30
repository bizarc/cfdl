# Notes

Not published. Working record for this case.

## The source

| | |
|---|---|
| Model | DCF template ("DCF Like a Banker") |
| Publisher | Multiple Expansion |
| Page | https://multipleexpansion.com/2017/11/07/dcf/ |
| File | https://multipleexpansion.com/excel/dcf.xlsx |
| Retrieved | 30 August 2026 |
| Size | 22,599 bytes |
| SHA-256 | `ccc2af27bfd93fad09443dd3e9bb9fa9fdd24ee3367a641d08e9ae3bd29c78f2` |
| Sheet | `DCF - Calc`, the only sheet |

Licence: "Copyright © 2025 MultipleExpansion.com. All Rights Reserved." No
registration, no paywall, no grant of redistribution. So the workbook is not
committed; the hash above identifies the file this case was built against.

## Why this source and not Damodaran

The two candidates for closing `opco.working_capital` and `opco.exit_multiple`
were this and the Damodaran company-valuation library. Damodaran cannot supply
either figure:

- Working capital is folded into *reinvestment* through a sales-to-capital
  ratio. There is no ΔWC row to assert. `damodaran_fcff` already models that
  correctly as `opco.capex_line.reinvestment`.
- The terminal value is Gordon growth capped at the riskfree rate, by doctrine
  — no exit multiple appears. That shape is `opco.exit_perpetuity`, already the
  most-declared contract in the suite.
- The cost of capital converges over the forecast, and a discount rate cannot
  vary over time (`docs/13` §7.4). `damodaran_fcff` had to decline to assert
  value, NPV or per-share price for exactly this reason; every model in that
  library inherits it.

This template uses a flat WACC, which is why an enterprise value is assertable
here at all.

## What was read

Rows 29–44 for the build: revenue, Adj. EBITDA, D&A, tax rate, capex and the
increase in NWC are typed constants; the growth and margin rows are derived
diagnostics off them. Row 47 for the discount periods. Rows 76–88 and the
output grid at rows 114–129 for the exit-multiple method.

Cash operating expense is the one line the template does not print: it is
revenue less Adj. EBITDA, both published. Nothing else in `expected.csv` is
derived.

## The discount periods

`=MROUND((FYE - $F$18)/365, 0.5) - 0.5*($F$47=1)` gives 0.5, 1.5, 2.5, 3.5, 4.5
with mid-year discounting on. The terminal differs by method:

- exit multiple, row 84: `(1/(1+r))^$L$47` — 5.0
- perpetuity, row 67: `(1/(1+r))^$K$47` — 4.5

`opco_exit_multiple` lowers to `on_date` with `schedule_placement = "end"`,
i.e. the full holding period. That is 5.0 and matches the exit-multiple arm
exactly; the perpetuity arm's 4.5 is not reachable through the contract. This
is the same position `packs/opco/lowering/rules.toml` takes in its "NO `mid`"
note, arrived at independently from a second source.

On a monthly grid a single-period `every` is an ordinary annuity and falls at
the period's close, so end of period index *i* is (*i*+1)/12 years:

| flow | period index | years |
|---|---|---|
| FY2018 | 5 | 0.5 |
| FY2019 | 17 | 1.5 |
| FY2020 | 29 | 2.5 |
| FY2021 | 41 | 3.5 |
| FY2022 | 53 | 4.5 |
| terminal | 59 | 5.0 |

No flow needs `mid` — every exponent is a month boundary, which is luck. In
`banker_dcf_conventions` the stub was not, and needed it.

## The full grid

Published enterprise values (exit-multiple method, rows 118–120) against ours.
Each cell needs its own run, so only the first is asserted.

| WACC | mult | ours | published | diff |
|---|---|---|---|---|
| 10% | 7.0x | 338.366157 | 338.3661574793 | −4.79e−07 |
| 10% | 8.0x | 373.971209 | 373.9712092960 | −2.96e−07 |
| 10% | 9.0x | 409.576261 | 409.5762611126 | −1.13e−07 |
| 11% | 7.0x | 325.375610 | 325.3756097150 | +2.85e−07 |
| 11% | 8.0x | 359.405469 | 359.4054690571 | −5.71e−08 |
| 11% | 9.0x | 393.435328 | 393.4353283992 | −3.99e−07 |
| 12% | 7.0x | 313.039207 | 313.0392072248 | −2.25e−07 |
| 12% | 8.0x | 345.576764 | 345.5767636748 | +3.25e−07 |
| 12% | 9.0x | 378.114320 | 378.1143201248 | −1.25e−07 |

Every residual is under 5e-7 and none has a sign pattern — it is the results
document's 6dp rounding.

## Mutation testing

Per `docs/20` §3.3. Each mutation applied to a copy, run at 10% / 7.0x,
compared against the baseline 338.366157 and its 1e-5 tolerance. All eight
caught.

| mutation | npv | caught |
|---|---|---|
| ΔNWC FY2018 3.0 → 3.5 | 337.889426 | yes |
| exit multiple 7.0 → 7.5 | 356.168683 | yes |
| LTM EBITDA base −1.0 | 334.019708 | yes |
| tax rate FY2018 0.38 → 0.40 | 337.660595 | yes |
| D&A FY2018 6.0 → 0.0 | 336.192263 | yes |
| FY2018 flows moved to 2018-07 | 338.223295 | yes |
| terminal moved to 2022-11 | 340.353595 | yes |
| capex FY2022 −0.5 | 338.691771 | yes |

The last three matter most. The two timing mutations confirm the mid-year
placement is load-bearing and asserted rather than incidentally right, and the
D&A mutation confirms depreciation reaches the answer through the tax — if
`da_monthly` were ignored the figure would not move.

There is no residual step and no waterfall here, so the one-sided-assertion
shape of `docs/20` §3.2 does not arise.

## Not asserted

Equity value. The template states net debt of 55 and publishes equity value
283.3661574792812 at the asserted cell, but net debt is a balance-sheet scalar
rather than a flow, and booking it as one to make the figure appear would
assert an accounting claim the model does not make.

The perpetuity arm, for the discounting reason above. Its grid, its implied
exit multiples (row 65) and the exit-multiple arm's implied perpetuity growth
rates (row 82) are all published and all reconcile by hand; the round-trip
between them is a real invariant and would want a language that can assert a
derived diagnostic against a published one.
