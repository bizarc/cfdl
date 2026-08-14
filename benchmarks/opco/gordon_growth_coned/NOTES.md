# Gordon Growth — the first case to assert a value

## Why this source

Every other externally reconciled case in this repo asserts cash flows, a
coverage ratio, or a pool factor. None asserts a **value**.

`benchmarks/opco/damodaran_fcff` says why, in its own words: its cost of capital
converges 7.055% → 8.81%, `RunConfig.discount_rate` is a single scalar, and
"discounting at a flat rate and reporting the agreement would have been easy and
dishonest." So it validates the drivers and asserts nothing discounted.

A regulated utility in steady state has one flat cost of capital. The valuation
*is* the terminal, so there is nothing to discount at a term structure and
nothing to be dishonest about. That is what makes this the case that closes the
gap — not a new engine capability.

Same author and same license as the FCFF model already committed, so the
workbook sits in `reference/` and a reader can open it.

## What is asserted

Nine published values against nine growth rates, all in one model:

| g | published | agreement |
|---|---|---|
| +0.041 | 67.08666666666666 | 1e-6 |
| +0.031 | 51.998260869565215 | 1e-6 |
| +0.021 | 42.29857142857143 | 1e-6 |
| +0.011 | 35.53818181818181 | 1e-6 |
| +0.001 | 30.55684210526315 | 1e-6 |
| −0.009 | 26.733953488372094 | 1e-6 |
| −0.019 | 23.707499999999996 | 1e-6 |
| −0.029 | 21.25207547169811 | 1e-6 |
| −0.039 | 19.220000000000002 | 1e-6 |

`period_tolerance = 1e-6` and it is the ENGINE's precision, not the source's:
the source states sixteen significant figures and results are published to six
decimals. Confirmed binding — at 1e-7 every row fails.

**Nine points spanning a sign change.** The grid runs from the riskfree rate
down through zero into negative growth, so it exercises the denominator on both
sides of `g = 0`. One point could be matched by coincidence; a sign error in the
`(1 + g)` numerator or the `(r − g)` denominator survives at one growth rate and
not across nine. This is the same argument as the nine-cell value grid in
`benchmarks/opco/banker_dcf_conventions`.

## Nothing is fitted

The model states exactly two numbers, and both are derived from the workbook's
own **inputs**, not read off its outputs:

```
dividends per share   EPS 3.17 x payout 0.7318611987381703   = 2.32     diff 0.0e+00
cost of equity        riskfree 0.041 + beta 0.8 x premium 0.045 = 0.077  diff 0.0e+00
```

From those two, every one of the nine published values follows to 0.0e+00 in
exact arithmetic. The workbook publishes both the inputs and the intermediate
results, so the derivation is checkable rather than asserted.

A contract term must be a literal or a single declared input — an expression is
`E0004`, and the diagnostic says so — which is why the two derivations are
`assume` values with their arithmetic recorded here rather than computed in the
model. `damodaran_fcff` takes the same posture for its converging growth and tax
paths.

## The convention worth stating

`opco.exit_perpetuity` applies the `(1 + g)` step itself, so `base_value` is the
**current** dividend and not next year's. The workbook does the same:
`Current Dividends per share = 2.32` and `Gordon Growth Model Value = 2.32 x
1.021 / (0.077 - 0.021)`. Passing next year's dividend instead would overstate
every value by `(1 + g)` — about 2% here, which is small enough to look like a
rounding difference and is not.

`discount_rate` is a contract term rather than the run's rate, and this case is
why that is right rather than a workaround: the cost of equity here is built
from CAPM inputs inside the model. The run's discount rate is set to zero and
nothing depends on it.

## What this case does not cover

**Anything discounted.** The exit is the entire model, so `model.npv` would
re-discount a figure that is already a present value. `expected_metrics.json` is
empty deliberately.

**Any operating rule.** This is a valuation identity, not a cash-flow build — no
revenue, opex or tax contract is exercised. `damodaran_fcff` covers those. The
two are complements: one validates the drivers and asserts no value, the other
asserts value and models no drivers.

**A stream-derived terminal.** `base_value` is stated. A variant reading the
terminal year's flow through `series_sum`, on the model of `opco.exit_ebitda`,
is the follow-on that would let `damodaran_fcff` carry its own terminal.
