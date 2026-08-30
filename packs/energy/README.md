# energy pack (v0.1)

Energy & microgrids: solar/wind PPA and merchant revenue, battery storage
arbitrage, capacity payments, O&M, investment tax credits, capex, and level-pay
project debt. All rules are template-driven (`{{contract.*}}` + defaults) —
no hardcoded amounts.

> **Supported calendars: all of them** — `daily`, `monthly`, `quarterly`,
> `annual`. Every rule divides annual quantities by the rule's own
> periods-per-year rather than a literal 12, so the same deal produces the same
> annual figures on any grid, and a gate asserts it.
>
> One caveat, on daily only. An annual quantity is spread as
> `X_year / 365` every day — the Act/365-Fixed convention — so a **leap year
> pays 366/365** of the annual amount, about 0.27% more. That is the convention
> behaving correctly, not drift, and it is why the daily parity fixture uses a
> non-leap window. If you need exact annual totals on a daily grid across a
> leap year, model the quantity per-period rather than per-year.

## Conventions

- Annual quantities (`mwh_year`, `om_year`, `payment_year`) spread evenly
  across the rule's own periods: `X_year / periods_per_year`, which is the
  model's calendar unless a rule declares its own `schedule_every`.
- Escalation and degradation step **annually**: `factor ^ floor(t / 12)`,
  matching common project-finance Excel practice.
- `energy.debt_service` lowers to the whole instrument — proceeds at closing
  (`funded_at_close`, default 1), an interest leg (`ipmt`) and a principal leg
  (`ppmt`), Excel sign conventions throughout. The legs sum to the level
  payment exactly, and both fold into the coverage subtotal.

> **Escalation here is NOMINAL.** `escalation` is one rate and it is applied
> whole: `pow(1 + escalation, elapsed_years)`. Project-finance tools commonly
> state escalation as a *real* rate carried on top of a separate inflation
> assumption, and at least one widely used model combines the two **additively**
> — 2.5% inflation plus 2.0% real escalation is 4.5%/yr, not 4.55%/yr. Moving a
> deal across means entering `escalation = inflation + real`, and the difference
> compounds. Verified in the [utility-scale PV benchmark](/docs/examples/energy-utility-pv-singleowner).
>
> **The ITC reduces the depreciable basis, and this pack will not do it for
> you.** `energy.macrs_shield` takes `basis` as an input rather than deriving
> it, because basis adjustments are jurisdictional and there are several. A 30%
> investment credit conventionally removes half the credit from the basis, so a
> $100m project taking $30m of credit depreciates $85m. Entering the installed
> cost instead overstates the shield by 17.6% for the life of the schedule and
> nothing here will object.
>
> State the adjustment, not the answer. A term holds an expression, so the
> arithmetic belongs in the model rather than in a comment beside a constant:
>
> ```cfdl
> assume installed_cost = 100000000
> assume itc_rate       = 0.30
>
> contract energy.itc on entity asset.pv {
>   terms { credit = inputs.installed_cost * inputs.itc_rate }
> }
>
> contract energy.macrs_shield on entity asset.pv {
>   terms { basis = inputs.installed_cost * (1 - 0.5 * inputs.itc_rate) ... }
> }
> ```
>
> A pasted `85000000` goes stale the moment either input moves. The
> [utility-scale PV benchmark](/docs/examples/energy-utility-pv-singleowner)
> is written this way.
>
> **`energy.ptc` rounds to a tick.** The statutory production credit is
> published rounded after each inflation adjustment, and the rule computes that
> staircase via `round_to`. `round_step` defaults to **1.00**, not 0.10: the
> credit is quoted to the nearest 0.1 c/kWh, which on this rule's per-MWh basis
> is a whole dollar ($0.001/kWh x 1000 kWh). Rounding a per-MWh figure to 0.10
> would round to a hundredth of a cent and change nothing. Set `round_step = 0`
> for a jurisdiction that does not round.
>
> Carrying the rate continuously — as this rule once did — is wrong by up to
> about 1.8% in any one year and roughly -0.3% over a 10-year window. The error
> alternates sign rather than drifting, so it reads as noise in aggregate and
> survives every reconciliation except against a source that rounds.

## Contract types

| Contract | Required terms | Optional (default) |
|---|---|---|
| `energy.ppa` | `mwh_year`, `ppa_price` | `escalation` (0), `degradation` (0), `availability` (1) |
| `energy.merchant` | `mwh_year`, `price` | `price_escalation` (0), `degradation` (0), `availability` (1) |
| `energy.storage_arbitrage` | `mwh_cycled_year`, `spread` | `degradation` (0) |
| `energy.capacity` | `payment_year` | — |
| `energy.om` | `om_year` | `escalation` (0) |
| `energy.itc` | `credit` | — (fires on `term_start`) |
| `energy.capex` | `amount` | — (fires on `term_start`) |
| `energy.debt_service` | `rate`, `term_months`, `principal` | `funded_at_close` (1) |
| `energy.ptc` | `mwh_year`, `credit_per_mwh` | `escalation` (0), `degradation` (0), `availability` (1); term bounds the credit window |
| `energy.macrs_shield` | `basis`, `tax_rate` | `life` (5; also 7/15/20) — IRS Pub 946 GDS half-year tables via `macrs_rate()` |

Tax attributes (ITC, PTC, MACRS shield) report under
`domain.energy.tax_benefits` and are excluded from revenue/EBITDA.

## Not yet modeled (roadmap)

Tax equity / partnership flip structures (HLBV), DSCR-sculpted debt sizing,
full tax computation (the MACRS stream models the shield value, not taxable
income). Planned for later pack increments.

## Quick start

A 2 MW solar-plus-storage microgrid with a 25-year escalating PPA:

```cfdl
version 0.1
model "my-microgrid"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 300

entity asset microgrid : Energy.Asset.GenerationFacility

contract energy.capex on entity asset.microgrid {
  term 2026-01..2026-01
  terms { amount = 2400000 }
}

contract energy.ppa on entity asset.microgrid {
  term 2026-01..2050-12
  terms {
    mwh_year = 4200
    ppa_price = 85
    escalation = 0.02
    degradation = 0.005
  }
}

contract energy.om on entity asset.microgrid {
  term 2026-01..2050-12
  terms { om_year = 70000 escalation = 0.025 }
}
```

Vintages are contract-anchored: each contract's escalation and degradation
clocks start at its own term start (its COD), not the model start.

## Run it

```bash
cfdl compile my-microgrid --packs packs --out my-microgrid/ir.json
cfdl run my-microgrid/ir.json --packs packs --pack energy --out my-microgrid/results.json --rate 0.08
```

`--pack energy` computes the pack's domain metrics (revenue, EBITDA,
`domain.energy.tax_benefits`, debt service) alongside NPV/IRR/MOIC.

## Recipes

**Add the investment tax credit** (fires once, at its term start; reports
under tax benefits, not EBITDA):

```cfdl
contract energy.itc on entity asset.microgrid {
  term 2026-12..2026-12
  terms { credit = 720000 }
}
```

**Level-pay project debt** (decimal-exact `pmt()`, Excel sign conventions):

```cfdl
contract energy.debt_service on entity asset.microgrid {
  term 2026-01..2045-12
  terms { rate = 0.06 term_months = 240 principal = 1600000 }
}
```

**Wind with PTC and MACRS**: pair `energy.ptc` (per-MWh credit over the
credit window) with `energy.macrs_shield` (IRS Pub 946 GDS tables via
`macrs_rate()`); see the [wind PTC and MACRS benchmark](/docs/examples/energy-wind-ptc-macrs).

Full worked models: the [solar PPA microgrid](/docs/examples/energy-solar-ppa-microgrid)
benchmark and the [solar microgrid notebook](/docs/notebooks/energy-solar-microgrid).

## Stream categories

Every stream this pack emits declares a `category` — a dotted path rooted in the
cash flow statement's three sections — and aggregation reads that rather than
pattern-matching the stream's name.

`operating.revenue.energy`, `operating.expense.om`, `operating.income_tax.benefit`,
`investing.capital.capex`, `financing.debt.service`.

Deliberately coarse. PPA, merchant, storage margin and capacity are all
`operating.revenue.energy`, because a project's EBITDA does not care which
contract produced a dollar — and a finer split can be added later without
reclassifying anything, since `operating.revenue.*` would still match.

`tax_benefit` is separate from revenue and stays out of EBITDA. An ITC, a PTC
and a MACRS shield are not operating income, and folding them in would overstate
the margin that every DSCR is measured against.

An unlisted category is `E5022`.
