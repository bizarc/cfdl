# LBO circular interest — the loop is linear, and linear loops have answers

"You need an iterative solver to model an LBO" is the standard argument for why
a declarative, loop-free engine cannot do leveraged finance. **For the
average-balance interest convention, that argument is wrong**, and this case is
the demonstration.

## The circularity, stated plainly

Interest is charged on the average of the opening and closing debt balance. The
closing balance depends on how much cash swept the debt down. The cash
available to sweep is free cash flow, which is net of interest. So:

```
interest(t)  ->  net income(t)  ->  free cash flow(t)  ->  closing balance(t)  ->  interest(t)
```

The reference model closes this by enabling Excel's iterative calculation. It
ships a literal `CIRC` switch in its assumptions block; with it off, interest is
charged on the opening balance and the loop disappears.

## Why it does not need a solver

The loop is **linear in the closing balance**. Every step above is an affine
function of it — no products of unknowns, no thresholds, no absolute values. So
writing the closing balance as the unknown and collecting terms solves it in
one step:

```
B(t)        = B(t-1) - LFCF(t)
LFCF(t)     = (1 - tax) * (EBIT(t) - interest(t)) + C(t)
interest(t) = rate(t) * (B(t-1) + B(t)) / 2 + K(t)
```

with `k = (1 - tax) * rate(t) / 2`:

```
B(t) = [ B(t-1) * (1 + k) - (1 - tax) * (EBIT(t) - K(t)) - C(t) ] / (1 - k)
```

`K(t)` is the interest that does **not** depend on the swept balance — the
commitment fee on the undrawn revolver, the fixed-rate senior notes, the PIK
subordinated coupon, amortised financing fees, less interest earned on the
minimum cash balance. All of it is known before `B(t)` is, which is precisely
what makes collecting `B(t)` legitimate. `C(t)` is the non-EBIT cash flow: D&A
back, non-cash interest back, working capital, less capital expenditure.

That expression is the `next` clause on `tlb_balance` in `model.cfdl`. It is
ordinary arithmetic on the previous period's state.

## The result

**Exact.** Against the reference workbook's own unrounded cached values, the
closed form agrees to **2.8e-14** — machine epsilon — across all sixteen
balance and interest figures.

| year | term loan balance | reference | term loan interest | reference |
|---|---:|---:|---:|---:|
| 2017 | 238.517440443 | 238.517440443 | 8.986555208 | 8.986555208 |
| 2018 | 199.519287769 | 199.519287769 | 8.979752928 | 8.979752928 |
| 2019 | 156.762561123 | 156.762561123 | 8.016341600 | 8.016341600 |
| 2020 | 120.484780576 | 120.484780576 | 7.139119049 | 7.139119049 |

Through the engine the worst disagreement across all 33 asserted figures is
4.5e-7, on the final year's repayment line — the engine's six-decimal
publication rounding, not arithmetic. The same floor `crest_solar_cost_based`
documents.

The model compiled and reproduced the schedule on the **first** run. Nothing
was tuned to fit.

## The PIK tranche is a second fixed point, also closed

Subordinated notes accrue payment-in-kind interest on their **own** average
balance, so the balance is on both sides again:

```
B = B0 + r * (B0 + B) / 2   ->   B = B0 * (1 + r/2) / (1 - r/2)
```

Nothing external enters, so it collapses to a constant growth factor — the
bilinear transform, the same algebra that maps continuous to discrete time in
signal processing. Three years of accretion (100 -> 108.877 -> 118.543 ->
129.066), then the coupon turns cash and the balance holds flat.

Worth separating from the term loan case: this one is circular in a *single*
quantity and would still be closed even if the rest of the model were
nonlinear. The term loan's is circular *through the whole cash flow statement*,
which is the harder and more interesting one.

A detail that falls out for free: while the coupon is PIK, the interest **is**
the balance increase. `model.cfdl` writes it as that difference rather than
recomputing `r * average`, so the two cannot disagree — the coupon and the
accretion are one number by construction.

## Where the closed form stops working — the honest limit

The derivation depends on **no constraint binding**. It holds here because:

- the revolver is never drawn, so there is no `max(0, ...)` on a draw;
- the term loan never fully repays, so the sweep is never capped by the
  remaining balance and never cascades to the next tranche;
- the minimum cash balance is exactly met every year, so it never forces a draw.

Break any of those and the recursion becomes **piecewise** linear. A `min` or a
`max` in the loop is a genuinely different problem: the closed form still holds
*within* each branch, but which branch applies depends on the answer. That is
solvable — solve each branch, test which is consistent — but it is not the
one-line substitution above, and CFDL cannot express the branch selection
today.

So the claim this case supports is precise: **the average-balance interest
convention does not itself require iteration.** It is not the broader claim
that every LBO is closed-form. A deal with a live revolver needs more.

## What is out of scope

- **Returns.** MoIC and IRR over the hold, and the entry/exit sensitivity grid.
  They need the equity waterfall and a management rollover, which is a separate
  build; the debt schedule is what the circularity claim rests on.
- **The revolver as a mechanism.** It is present as a commitment fee only,
  because it is never drawn. Modelling a revolver that actually draws is the
  first thing that would break the closed form, per above.
- **The other two financing cases.** The reference publishes Base (6.0x), High
  Leverage (7.5x) and Low Leverage (4.5x); only Base is reproduced here. The
  other two swap leverage and tranche pricing together, which the harness would
  need a per-case run to express.

## The source

A publicly downloadable seven-step LBO teaching model, free and without
registration, published by a financial-modelling site. It carries an explicit
"All Rights Reserved" notice and no open licence, so it is **neither vendored
nor wired into CI**: it was downloaded once outside this repo, its cached
values read, and only those numbers carried across. The same handling
`crest_solar_cost_based` and `utility_pv_singleowner` give their references.

The deal: $90mm LTM adjusted EBITDA acquired at 8.0x for a $720mm transaction
value, funded with $275mm term loan B, $175mm senior notes, $100mm
subordinated notes, a 5% management rollover and $158.9mm of sponsor equity,
against $15mm of transaction expenses and $9.9mm of financing fees. Four-year
hold, 35% tax rate, $5mm minimum cash, 1% mandatory term loan amortisation with
a 100% sweep of the remainder.
