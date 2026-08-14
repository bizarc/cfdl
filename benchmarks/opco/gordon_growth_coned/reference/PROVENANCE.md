# Provenance

**Gordon Growth Model — a stable-growth dividend discount valuation of a regulated electric utility**

| | |
|---|---|
| Author | Aswath Damodaran, NYU Stern School of Business |
| Retrieved | 2 August 2026 |
| URL | https://www.stern.nyu.edu/~adamodar/pc/eqegs/coned08.xls |
| Index page | https://pages.stern.nyu.edu/~adamodar/New_Home_Page/covals.htm |
| Size |    30720 bytes |
| SHA-256 | `d8fc9f0ed5ebede8115f5e7175f92a6d8ed2f8a08e160e24347e5426a08480e6` |

## License

The index page states:

> These are excel spreadsheets with the valuations that are included in the
> lecture notes. Feel free to download them and have fun changing the inputs or
> updating the information

That is the same permission under which `benchmarks/opco/damodaran_fcff`
already commits `fcffsimpleginzu.xlsx`, so the workbook is committed here too.

## What was read

Sheet **ConEd01.xls**:

- Inputs: current earnings per share 3.17, payout ratio 0.7318611987381703,
  beta 0.8, riskfree rate 0.041, risk premium 0.045, expected growth 0.021.
- Outputs: current dividends per share 2.32, cost of equity 0.077, and
  `Gordon Growth Model Value = 42.29857142857143`.
- The **growth rate sensitivity table**, nine rows from 0.041 down to -0.039,
  which is what `expected.csv` asserts.

Sheet **Sheet1** carries estimated value against price per share by period and
is not used.

## Nothing here is fitted

The model states two numbers, and both are derived exactly from the workbook's
own inputs rather than read off its outputs:

```
dividends per share  3.17 x 0.7318611987381703  = 2.32     diff 0.0e+00
cost of equity       0.041 + 0.8 x 0.045        = 0.077    diff 0.0e+00
```

From those two, all nine published values follow to 0.0e+00 in exact
arithmetic. A contract term must be a literal or a single declared input, so
the two derivations are stated as `assume` values with their arithmetic
recorded here — the same posture `damodaran_fcff` takes for its converging
growth and tax paths.

The workbook is committed as data and is never opened or executed by any test;
the figures were read once and transcribed.
