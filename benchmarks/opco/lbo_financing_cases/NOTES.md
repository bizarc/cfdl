# Three capital structures — what building it found

## Why this is not `lbo_circular_interest` again

That case asserts the Base debt schedule line by line. It is a claim about the
circularity: interest on an average balance collects in closed form.

This one asserts the **endpoint**, at three leverage levels, and the source
publishes intermediate figures for only one of them. For High Leverage and Low
Leverage nothing between the inputs and the answer is anchored — the operating
build, the tranche sizing, the sweep, the PIK accrual, the exit and the returns
arithmetic all have to be right at once. Base anchors every line; those two
anchor none.

It is also the first case in the repository to use **run-config scenarios**.
Before this, no benchmark exercised them at all.

## What the reference publishes, and what it does not

| | published |
|---|---|
| Financing case definitions | all three, complete |
| Debt schedule, period by period | Base only |
| MoIC and IRR, 5×5 entry/exit grid | all three — 150 figures |
| Equity waterfall detail | at 9 exit multiples |

This case asserts the 8.0x / 8.0x corner of each grid, plus the Base schedule.
The remaining grid points are one scenario each and are a matter of generating
them.

## Sizes are derived, not transcribed

The reference sizes each tranche as leverage × LTM EBITDA rounded to a $25m
increment, and the sponsor's cheque is the plug that balances sources against
uses. Reproducing both rules is what lets the other two structures be *derived*
rather than read off:

    round_to(3.0 * 90, 25) = 275     round_to(2.0 * 90, 25) = 175
    round_to(1.0 * 90, 25) = 100     total 550 = 6.1x

Those three are what the source publishes for Base, so the rule is checked
before it is relied on. High Leverage then gives 275 / 225 / 175 and Low
Leverage 275 / 125 / 0.

## Two traps in reading the source

Both came from reading formulas rather than displayed values, and both would
have produced a wrong model that looked plausible.

**Depreciation is not 5% of sales.** The sheet displays 5%; the formula holds
the trailing ratio, which is 17 / 342 = **4.9708%**. Using the displayed figure
misstates D&A every year and flows through tax into free cash flow and the
sweep.

**LIBOR in year four is 0.0215, displayed as 0.021.** The companion case's notes
record this exact rounding causing a false mismatch once already.

## The finding: a scenario computed everything and published one number

The engine runs each scenario as a full deterministic evaluation —
`run_deterministic` returns the same metric map the base run produces. The
scenario summary then discarded all of it and published NPV alone.

So a model whose entire subject is how returns move with leverage had nothing to
report for the two scenarios that varied it. MoIC, IRR, payback, WAL and every
per-stream total were computed and thrown away.

Fixed by publishing the run's own metric map, which also means a scenario and
the deterministic block cannot report different metric sets.

## The delta is the reference's, not ours

The subordinated balance carries a 1e-4 tolerance where the term loan is exact
at 1e-6. The PIK accrual is self-referential:

    B = B0 + avg(B0, B) * r    ->    B = B0 (1 + r/2) / (1 - r/2)

The source solves it by switching on iterative calculation. Checked against its
own equation at B0 = 100, r = 8.5%:

| | value | residual |
|---|---:|---:|
| closed form | 108.87728459530 | −1.4e-14 |
| reference | 108.87732342007 | +3.7e-05 |

The reference stopped iterating while its own equation still had a residual, so
1e-4 is what its convergence supports rather than a concession by this model.
The closed form is the more accurate of the two.

The term loan agrees at 1e-6 across all five years despite depending on the
subordinated coupon, because the coupon enters as a level term rather than
compounding into the sweep.
