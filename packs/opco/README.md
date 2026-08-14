# OpCo Pack v0.1

Operating-company / LBO pack: recurring operating lines, policy-driven
working capital, capex, scheduled term debt, cash taxes, and entry/exit —
benchmarked against an independent month-by-month
reference. All lowering is template-driven.

> **Supported calendars: all of them** — `daily`, `monthly`, `quarterly`,
> `annual`. Annual quantities divide by the rule's own periods-per-year rather
> than a literal 12.
>
> Two things are per-period by definition and so mean different economics on
> different grids: `amount` and `da_monthly`. Use their annual siblings —
> `amount_year`, `da_year` — to state a deal grid-independently. A line must
> state one of `amount` / `amount_year` (`E7001`); giving both sums them.
>
> **`growth_rate` compounds continuously on the model clock**, `(1+g)^(t/ppy)`,
> which is a deliberate convention and is inherently grid-sensitive: a finer
> grid captures more intra-year compounding. At 5% annual growth, year-one
> revenue on a 120,000/period line is 1,472,709 monthly against 1,440,000
> annually. Both are right for their convention. If you need annual totals to
> match across calendars, hold `growth_rate` at zero and step the amount
> explicitly, or model on the grid you intend to report on.
>
> Term debt and policy-driven working capital are correct on every calendar but
> are not annual-total invariant either: nominal rate accrual differs by
> cadence by design (a 6% loan is 0.5%/month and 1.5%/quarter), and the
> working-capital delta telescopes, so only its sum over the contract's life is
> invariant.

## Activation

```cfdl
use pack "opco" version "0.1.0"
```

## Contract types

All contracts accept instance suffixes (`opco.revenue_line.saas`,
`opco.revenue_line.services`, ...) which suffix the lowered stream names.
Growth is annual-compound stepped continuously on the model clock:
`value(t) = amount * (1 + growth_rate)^(time.t / 12)`.

> **A driver may vary over time.** `growth_rate` and `tax_rate` are scalars,
> which is right for a stable business and wrong for what intrinsic valuation
> actually does: growth decays toward the riskfree rate and the effective tax
> rate climbs toward the marginal one as a firm matures.
>
> `growth_curve` (on `revenue_line`, `opex_line`, `capex_line`) and
> `tax_rate_curve` (on `cash_taxes`) name a model `curve` instead, read at each
> period's date — the same mechanism `credit.pool_float_io_bullet` uses for a
> floating index. Empty by default, so a model stating only the scalar is
> unchanged.
>
> **The curve carries a per-period rate, and compounding is still `pow(1+g, t)`.**
> That applies one period's rate as though it had held from the start: exact
> while the rate is flat, drifting once it moves, because the true factor is the
> running product, and computing it needs a stream to read its own prior
> period. A cumulative-index curve would be exact
> today and was deliberately not chosen — it would hide the gap in every model
> that used it. The drift is measured year by year in
> the [free cash flow to firm benchmark](/docs/examples/opco-damodaran-fcff).
>
> `tax_rate` defaults to 0 so a curve can stand alone; stating neither it nor
> the curve is `E7012_OPCO_TAXES_MISSING_RATE`, not a silent zero-tax model.

### Operating lines

- `opco.revenue_line` — `amount` (monthly), optional `growth_rate` or
  `growth_curve`.
  Stream `opco.revenue.recurring`.
- `opco.opex_line` — same terms. Stream `opco.opex.recurring` (outflow).
- `opco.working_capital` — fixed monthly WC outflow (`amount`).
- `opco.working_capital_policy` — DSO/DPO/DIO-driven:
  `WC(t) = annualized revenue * ar_days/365 + annualized opex * (inv_days - ap_days)/365`
  from the modeled streams (phase-2 series lookups). Books the full initial
  WC in the first period, the period-over-period change afterwards, and
  releases the ending balance in the final period when `release_at_end = 1`.
  Terms: `ar_days`, `ap_days`, `inv_days` (all default 0), `release_at_end`.
- `opco.capex_line` — fixed `amount` (+ `growth_rate` or `growth_curve`) plus
  `pct_of_revenue` of the modeled revenue streams. Stream `opco.capex`.

### Financing

- `opco.term_debt` — scheduled term loan: `principal`, `rate`,
  `io_months` (default 0), `amort_months`; optional `funded_at_close`
  (default 1) controls the proceeds inflow at `term_start`. After the IO
  period the loan amortizes level-pay over `amort_months`; the remaining
  balance pays as a balloon at the contract's `term_end`. Streams
  `opco.debt.proceeds`, `opco.debt.interest`, `opco.debt.principal`.
  **Cash sweeps and revolvers need per-period persistent state and are not
  in v0.1.**
- `opco.acquisition` — purchase `price` paid at `term_start`
  (the equity check when paired with debt proceeds at the same date).

### Taxes

- `opco.cash_taxes` — `tax_rate` or `tax_rate_curve` on `max(0, EBITDA - D&A - interest)` per
  period. EBITDA and interest come from the modeled streams; D&A is a
  declared deduction (`da_monthly`, optional `da_growth`), not a cash
  stream. **No NOL carryforwards** (losses floor at zero tax per period;
  carryforwards need H3-style state). Stream `opco.taxes`.

### Exit

- `opco.exit_multiple` — `base_value * exit_multiple` at the contract's
  `term_start`.
- `opco.exit_ebitda` — `exit_multiple` × trailing-12-month EBITDA derived
  from the modeled streams, net of `selling_costs`, at `term_start`.

### `opco.exit_perpetuity`

Terminal value as a growing perpetuity — the Gordon form, and the terminal every
intrinsic valuation ends with. The pack could previously express only a
*multiple* of something, so the largest single component of value in a DCF had
no contract.

```
TV = base_value * (1 + growth_rate) / (discount_rate - growth_rate) * (1 - selling_costs)
```

| term | meaning | default |
|---|---|---|
| `base_value` | the terminal-period flow, **before** the `(1 + g)` step | *required* |
| `growth_rate` | perpetual growth; state `0` for a flat perpetuity | *required* |
| `discount_rate` | terminal capitalization rate | *required* |
| `selling_costs` | fraction deducted from proceeds | `0` |

**`discount_rate` is a term, not the run's NPV rate.** That is deliberate. A
terminal cost of capital legitimately differs from the near-term one — it is the
rate for a business that has reached steady state — and the published models
that state these terminals build it explicitly, usually from their own CAPM
inputs. The run's `annual_discount_rate` *discounts* the resulting cash flow;
this rate *capitalizes* it.

**Match the rate to the flow.** A cost of equity belongs against a dividend or
FCFE; a cost of capital belongs against FCFF. The contract is deliberately
neutral about which `base_value` is, and cannot detect a mismatch.

`E7025` guards the one thing that must hold: `discount_rate > growth_rate`. The
exit settles at the end of its period and carries no `mid` — a terminal value is
a price struck at a point in time and discounts whole, unlike the flows around
it (see the [banker DCF conventions benchmark](/docs/examples/opco-banker-dcf-conventions)).

## Metrics

`domain.opco.revenue`, `.ebitda`, `.ebitda_margin`, `.capex`,
`.working_capital` (net investment; releases net out), `.taxes`,
`.debt_service`, `.fcf` (EBITDA − capex − WC − cash taxes; note taxes
deduct interest, so this is FCF after the interest tax shield),
`.fcf_to_debt_service`.

## Diagnostics (E7xxx)

- `E7001_OPCO_LINE_MISSING_AMOUNT`, `E7002_OPCO_LINE_INVALID_SCHEDULE`,
  `E7003_OPCO_LINE_INVALID_GROWTH`
- `E7010_OPCO_WC_MISSING_AMOUNT_OR_RULE`, `E7011_OPCO_WC_INVALID_SCHEDULE`
- `E7020_OPCO_EXIT_MISSING_MULTIPLE`, `E7021_OPCO_EXIT_INVALID_MULTIPLE`,
  `E7022_OPCO_EXIT_MISSING_BASE_VALUE`, `E7023_OPCO_EXIT_INVALID_SCHEDULE`
- `E7024_OPCO_EXIT_EBITDA_INVALID_MULTIPLE`
- `E7030_OPCO_DEBT_INVALID_AMORT`, `E7031_OPCO_DEBT_INVALID_RATE`
- Missing templated terms surface as `E5006_MISSING_CONTRACT_TERM`.

## Not in v0.1 (planned waterfall & capital-stack work)

- Cash-flow sweeps, revolver draws/paydowns, PIK toggles (need per-period
  persistent state).
- NOL carryforwards.
- Waterfall distributions to the capital stack.

## Provenance and determinism

Generated streams carry source contract file/span and
`generated_by.pack/rule_id`; rule ordering, diagnostics ordering, IDs and
results are deterministic under identical inputs.

## Quick start

A services business bought in an LBO — revenue/opex lines, working capital,
capex, term debt:

```cfdl
version 0.1
model "my-buyout"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 60

entity asset target : OpCo.Asset.Enterprise

contract opco.revenue_line on entity asset.target {
  term 2026-01..2030-12
  terms { amount = 1000000 growth_rate = 0.06 }
}

contract opco.opex_line on entity asset.target {
  term 2026-01..2030-12
  terms { amount = 650000 growth_rate = 0.04 }
}

// Net working capital nets to zero over the full term because
// release_at_end returns the investment at exit — that is the point of the
// term, not an inert stream.
// examples-allow: working_capital.adjustment — released in full at exit
contract opco.working_capital_policy on entity asset.target {
  term 2026-01..2030-12
  terms { ar_days = 45 ap_days = 30 inv_days = 10 release_at_end = 1 }
}

contract opco.capex_line on entity asset.target {
  term 2026-01..2030-12
  terms { amount = 40000 pct_of_revenue = 0.01 }
}

contract opco.term_debt on entity asset.target {
  term 2026-01..2030-12
  terms { principal = 20000000 rate = 0.09 amort_months = 84 }
}
```

## Run it

```bash
cfdl compile my-buyout --packs packs --out my-buyout/ir.json
cfdl run my-buyout/ir.json --packs packs --pack opco --out my-buyout/results.json --rate 0.10
```

## Recipes

**Scheduled term debt** (IO period, level-pay amortization via
`ipmt`/`ppmt`, balloon at maturity, proceeds at close):

```cfdl
contract opco.term_debt on entity asset.target {
  term 2026-01..2030-12
  terms {
    principal = 14000000
    rate = 0.085
    io_months = 12
    amort_months = 84
  }
}
```

**Trailing-EBITDA exit** (the LBO convention — trailing twelve months, not
forward):

```cfdl
contract opco.exit_ebitda on entity asset.target {
  term 2030-12..2030-12
  terms { exit_multiple = 8.5 }
}
```

Full worked model: the [leveraged buyout](/docs/examples/opco-lbo-buyout) (validated against an
independent recursive reference) and the LBO notebook in
`examples/notebooks/`.

## Stream categories

Every stream this pack emits declares a `category` — a dotted path rooted in the
cash flow statement's three sections — and aggregation reads that rather than
pattern-matching the stream's name.

`operating.revenue.recurring`, `operating.expense.opex`,
`operating.working_capital`, `operating.tax`, `investing.capital.capex`,
`investing.acquisition`, `investing.exit`, `financing.interest`,
`financing.debt_principal`, `financing.debt_proceeds`.

The split follows the two statements an operating company reports. `interest` is
its own category rather than part of a `debt_service` blob because a P&L
subtracts interest before tax while principal never touches it; for the same
reason `debt_proceeds` is separate from `debt_principal`, since a draw and a
repayment are opposite entries in the financing section rather than one net line.

Note that interest is placed under `financing` here, which is the US GAAP
convention; IFRS permits it under operating. That choice belongs to the pack —
CFDL fixes the vocabulary of sections, not the accounting policy.

An unlisted category is `E5022`.
