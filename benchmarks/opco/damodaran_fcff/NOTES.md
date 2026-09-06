# Damodaran FCFF — the first opco case built from pack contracts

## Why this source, and why the other one was not enough

`benchmarks/opco/banker_dcf_conventions` reconciles an investment bank's DCF to
all nine cells of its published value grid. It is a good case and it validated
**none of the opco pack**: the filing publishes the *result* — per-year unlevered
cash flow — so the model had to hand-write six native streams. Measured across
all six externally-reconciled cases, opco was at **0 of 10 contract types**
(backlog 7.3).

The fix was never the rules; it was the source. Damodaran's FCFF Simple Ginzu
publishes the **drivers** — revenue growth, operating margin, tax rate, a
sales-to-capital ratio — and every line they produce. Drivers are what a pack
rule consumes. This case is built entirely from `opco.revenue_line`,
`opco.opex_line`, `opco.cash_taxes` and `opco.capex_line`, with no native
streams at all.

The license is explicit — *"not copy protected… feel free to modify them to your
own specifications"* — so the workbook is committed under `reference/`. Second
source in the repo a reader can open and mark us against.

## Nothing here is fitted

The drivers converge, which is the defining character of intrinsic valuation:
growth decays toward the riskfree rate and the effective tax rate climbs toward
the marginal one as a firm matures. Both paths are **derived** from the stated
inputs and then checked against the published rows:

```
growth  5.00% (yrs 1-5) -> riskfree 4.58%, linear over yrs 6-10
tax    17.50% (yrs 1-5) -> marginal 25.00%, linear over yrs 6-10
```

Derived against published, years 6–10: growth agrees to 0.0e+00 and tax to
5.6e-17. Applying the derived growth as a running product reproduces published
revenue for all ten years to **3.6e-12**. So the drivers and the convergence
rule are right, and anything that follows is about how the pack *compounds*
them.

Also confirmed, because it is easy to get backwards:
`reinvestment(t) = revenue(t) * g / sales_to_capital` — it funds **next** year's
growth, not the current year's. Verified on every year.

## The result

| line | asserted | agreement |
|---|---|---|
| revenue | **years 1–10** | 1e-6 |
| operating cost | **years 1–10** | 1e-6 |
| cash taxes | **years 1–10** | 1e-6 |
| reinvestment | years 1–4 | 1e-6 |

Which takes opco from 0 to **4 of 10** contract types externally validated.

## The blanks closed, and what closing them proves

The curves carry a **per-period rate**. That is the interface a modeller would
naturally write, and until the language could compound one it was also the
interface that drifted: the rules used `pow(1 + g, t)`, which applies one
period's rate as though it had held from the start. Exact while a rate is flat,
wrong once it moves — and this source's growth decays every year from year 6.

The measured drift was **−2.4% on revenue by year 10**, tabulated here rather
than hidden behind a loosened tolerance:

| year | revenue | EBIT | reinvestment | FCFF |
|---|---|---|---|---|
| 6 | −93.15 | −13.10 | −25.93 | −36.54 |
| 7 | −219.37 | −30.85 | −38.64 | −63.17 |
| 8 | −382.24 | −53.76 | −51.75 | −93.68 |
| 9 | −585.40 | −82.33 | −65.21 | −128.19 |
| 10 | −832.58 | −117.09 | −61.75 | −149.56 |

**Every one of those is now zero.** The pack rules compound through a declared
state — `init 1`, `next prev * (1 + rate)` — so the factor is the running
product of each period's rate, which is what the source computes. Revenue,
operating cost and cash taxes agree with the published forecast to
floating-point noise across all ten years, and `expected.csv` has no blanks in
those three columns.

A cumulative-index curve would have been exact from the start. It was
deliberately not used, because it would have hidden the gap inside every model
that adopted it and left the pack with an interface nobody would choose once the
language could do better. That judgement is what made this measurable.

This is one half of the acceptance test for `docs/14_state_and_recurrence.md`;
the other is `benchmarks/cre/hud_home_multifamily`, an unrelated source in an
unrelated pack whose escalation is a *rounded* recurrence rather than a
multiplicative one. Two independent published sources confirming one mechanism.

## Reinvestment still stops at year 4, for a different reason

Worth being precise, because the old drift table made this look like the same
defect and it is not.

Reinvestment is a **derived** line: `revenue(t) * g(t+1) / sales_to_capital`. It
funds *next* year's growth, so its own growth factor is
`(1 + g_t) * g_{t+1} / g_t`, not `(1 + g_t)`. Read straight off the published
rows, the two paths separate exactly where the growth path starts to decay:

| | year 5 | year 6 | year 7 | year 8 | year 9 | year 10 |
|---|---|---|---|---|---|---|
| reinvestment grows | 3.24% | 3.12% | 3.01% | 2.89% | 2.78% | 4.58% |
| revenue grows | 5.00% | 4.92% | 4.83% | 4.75% | 4.66% | 4.58% |

`opco.capex_line` is a self-growing line — a base times a rate path — so it
cannot express a quantity defined by *another* line's growth. That is a contract
shape gap, not a compounding one, and no recurrence fixes it. Backlogged.

Deriving a reinvestment-ratio curve by hand and asserting all ten years would
have worked and would have been the wrong call, for the same reason the
cumulative index was: it hides a pack gap inside the model that walked into it.

## Value: the ten-year PV is asserted, the enterprise value is not

The cost of capital converges 7.055% → 8.81% over the forecast, and the run
discounts along it: `cost_of_capital` is a step curve in the model and
`run.json` names it as `annual_discount_curve`. With reinvestment now derived
(`opco.reinvestment`: this year's revenue times next year's growth over the
sales-to-capital ratio, the growth read a year ahead with `edate`), all ten
years of FCFF are the workbook's, and `model.npv` reproduces the PV of the
ten years — 16,394.54, cell B20 of "Valuation output" — to 1e-4.

The enterprise value (37,517.53) and the per-share price (7.19) are not
asserted. They need the terminal value — `opco.exit_perpetuity` at the
terminal cost of capital 8.81% and growth 4.58% on the terminal year's FCFF
of 1,855.23 — and the balance-sheet bridge: debt 45,063, minority interests
1,558, cash 19,000, non-operating assets 21,119, over 4,315 shares. The
bridge is a statement over stated amounts rather than a contract, and is
the next thing this case can take on.

## What the pack gained

`growth_curve` on `opco.revenue_line`, `opco.opex_line` and `opco.capex_line`,
and `tax_rate_curve` on `opco.cash_taxes` — each naming a model `curve`, read
with `curve_value` at the period's date. This is the mechanism
a floating-coupon `credit.loan` already uses for a floating index; nothing new was
invented.

All four default to `""`, so every existing model stays on the scalar path.
Verified: `git diff gold/results` minus `model_hash` is empty across all 108
goldens.

`tax_rate` gained a default of `0` so a curve can stand alone, with
`E7010_OPCO_TAXES_MISSING_RATE` requiring one of the two — a defaulted rate
without that check would silently model a business that pays no tax.

Those three growth rules now compound through a **rule-declared state** rather
than `pow`, so every model using `growth_curve` gets the running product without
being edited. The rule declares `state_name` / `state_init` / `state_next`
alongside its stream; the state's `next` keeps the same
`if(curve == "", scalar, curve_value(...))` selector, so a model that states only
a scalar rate is on the identical path it was.

Blast radius, measured rather than assumed: across all 110 goldens the only
movement is `7365967.000481 → 7365967.00048` (1.4e-13 relative) and a `-0.0 → 0.0`
signed-zero, both from reassociating `pow` into iterated multiplication.
