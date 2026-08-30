# CRE Pack v0.1.0

This pack provides deterministic lowering for a minimal Commercial Real Estate
developer lifecycle:

> **Supported calendars: all of them** — `daily`, `monthly`, `quarterly`,
> `annual`. Annual quantities (`rent_year`, `opex_year`, `market_rent_year`,
> `potential_gross_year`, `sales_year`) divide by the rule's own
> periods-per-year, not a literal 12.
>
> `_months` terms — `free_rent_months`, `downtime_months`, `lease_up_months` —
> always mean **calendar months**, on every calendar. They describe the lease,
> not the modeller's grid, so they pro-rate exactly: five months free rent is
> 5 periods monthly, 1.667 quarterly and 0.417 annually, and year one comes out
> at 480,000 x 7/12 = 280,000 on all three.
>
> `base_rent` on `cre.lease` is per-period by definition; `base_rent_year` is
> its annual sibling and is how to state a lease grid-independently. A lease
> must give one of the two (`E6001`).
>
> `cre.exit_forward` derives NOI over the **year** after the sale, which is
> `project 12` on a monthly model, `project 4` quarterly and `project 1`
> annually — it used to be a hardcoded twelve periods, meaning twelve years on
> an annual grid.

- construction (`cre.construction_stub`)
- lease-up (`cre.lease`)
- stabilized operations (`cre.revenue_line`, `cre.opex_line`)
- exit (`cre.exit_cap`)

## Pack identity

- `name = "cre"`
- `version = "0.1.0"`

Models activate this pack with:

```cfdl
use pack "cre" version "0.1.0"
```

## Canonical contract kinds

The current pack host lowers by contract name. The following names are stable
in `lowering/rules.toml`:

- `cre.construction_stub`
- `cre.lease`
- `cre.revenue_line`
- `cre.opex_line`
- `cre.exit_cap`

`cre.opex_line` is the only operating expense contract. It replaced
`cre.property_opex` and `cre.ops_expense`, which differed only in escalation and
in whether they could be instanced — and, confusingly, the one named for a
single property was the repeatable one.

One contract spans the whole range:

- **A single blended figure.** One unsuffixed instance: `amount_year`, maybe
  `escalation`.
- **An itemized schedule.** One instance per expense —
  `cre.opex_line.property_tax`, `cre.opex_line.utilities` — each with its own
  escalation and its own fixed share. `templates.toml` ships the conventional
  set as editor snippets; a modeller can name their own.
- **Any level.** Property, building or suite is the ENTITY the contract hangs
  on, and `part_of` rolls them up. It is not a term, because that would restate
  the entity tree somewhere it could disagree with it.

Occupancy response is `pct_fixed + (1 - pct_fixed) * occupancy`. Property tax
runs whether the building is full or empty; cleaning tracks occupied space. At
the default `pct_fixed = 1` this reduces to a plain escalating series.

A term may also hold an expression — `escalation = curve_value("cpi",
time.date) + 0.005` states an agreed formula directly. A varying rate arrives
through the term itself; there are no curve-selector twin terms.
- `cre.lease_unit.<id>`, `cre.rollover.<id>`, `cre.opex_line`,
  `cre.vacancy_loss`, `cre.percentage_rent`, `cre.exit_forward`
- `cre.permanent_debt`
- `cre.construction_loan`

### `cre.construction_loan`

A construction facility funded behind an equity commitment. Equity draws first;
the loan takes the balance once the commitment is exhausted; interest accrues on
the drawn balance through the build. Emits three streams — the equity draw
(`financing.equity.contribution`), the loan draw (`financing.debt.proceeds`) and interest
(`financing.debt.service`, so coverage during a build counts it).

| term | | |
|---|---|---|
| `draw_curve` | required | the NAME of a declared `curve` giving required funding as an ANNUALIZED rate |
| `equity_commitment` | required | equity funds up to this, then the facility takes over |
| `rate` | required | nominal annual |
| `draw_accrual_fraction` | 0.5 | where in the period a draw lands: 0.5 drawn ratably, 0 at the end, 1 at the start |
| `day_count` | — | drives the accrual divisor, as elsewhere |

**The draw schedule is a curve, not a term.** A development's funding profile is
per-deal data — a published sixteen-quarter schedule, an S-curve, a contractor's
actual requisitions — and all three are the same object. A term carrying a
curve's SHAPE (a steepness parameter, a flat-or-S-curve enum) states an
implementation choice as though the parties had agreed it. The contract names
the curve; the model declares it.

**State the curve ANNUALIZED.** Every read divides by the rule's
periods-per-year, the same convention `rent_year` and `opex_year` follow. This
is what keeps the contract cadence-neutral, and it is not cosmetic: a `curve` is
a LEVEL, so a step curve returns its last point on every date whether or not a
point was declared there. A schedule stated as per-period TOTALS and then run on
a finer calendar would repeat each figure and fund several times the money, with
no diagnostic. Annualized, one sparse point — `2026-01: 4000` — funds 4,000 a
year on a quarterly model and on a monthly one alike, and re-graining spreads a
quarter's funding across its months, which is the only defensible reading when
nothing finer was stated.

**The commitment depleting mid-period is not a special case.** Cumulative
funding is the contract's one field, and the equity/debt split is
`min`/`max` over it — so a period where the commitment runs out part-way splits
by arithmetic rather than by a rule. The published One Lincoln Street case is
shipped twice, once built from primitives and once through this contract, and
the two agree in all 48 cells with zero difference.

**Interest is paid, not capitalized.** A capitalizing facility compounds and is
a different recurrence — affine in the closing balance, so it collects rather
than needing a solver — and is not modeled here.

### `cre.permanent_debt`

A commercial mortgage on a stabilized property. Emits three streams — the
whole instrument, per the contract design rules:

- `cre.debt.proceeds{.<id>}` — the draw at closing
  (`financing.debt.proceeds`)
- `cre.debt.interest{.<id>}` — the interest leg
  (`financing.debt.interest_paid`)
- `cre.debt.principal{.<id>}` — scheduled amortization, plus the
  balloon when opted in (`financing.debt.principal`)

Interest plus principal reproduce the level payment exactly (`ipmt + ppmt =
pmt`), and both fold into `domain.cre.debt_service`, so coverage is what it
always was — with the split available for a tax line, interest coverage, or an
amortization schedule.

| term | meaning | default |
|---|---|---|
| `principal` | loan amount | *required* |
| `rate` | nominal annual rate | *required* |
| `amort_months` | amortization term — strikes the payment | *required* |
| `io_months` | interest-only months before amortization begins | `0` |
| `funded_at_close` | share of principal drawn at `term_start`; `0` for a reconciliation whose source starts post-financing | `1` |
| `balloon_at_maturity` | `1` pays the unamortized balance as principal at `term_end` | `0` |
| `payment_frequency` | `day`/`week`/`month`/`quarter`/`year` | the model calendar |
| `day_count`, `amortization_day_count` | interest accrual and payment bases | `30/360` |

**`amort_months` is normally longer than the term.** A 30-year amortization on a
10-year loan is the standard commercial structure, and it is why a balloon
exists at all.

**The balloon defaults off.** Coverage is measured on *periodic* debt service, so
folding an unamortized balance into the final period would make that period's
DSCR meaningless; the standard pro forma repays it out of the sale. Turn it on
when the payoff genuinely belongs in the debt service line.

**Not modeled:** sizing to a target coverage ratio (a solve), refinance (needs
the events layer), and mortgage insurance — MIP is not a payment on the debt.

## Expected terms (authoring contract)

Contract `terms { ... }` payloads are captured as a lightweight key/value map
and validated by CRE lowering-time checks (`E6xxx_*`) during compile.

### Simple whole-property contract reference

Every contract below is term-gated: its streams run from `term_start` to
`term_end`, and time inside an expression is measured from `term_start`. No
amount, rate, or date is supplied by the pack — required terms have no
defaults, so a missing one fails compilation with `E5006` naming the term.

| Contract | Required terms | Optional (default) | Lowers to |
|---|---|---|---|
| `cre.construction_stub` | `amount` (per period) | — | `cre.construction.draws` (outflow) |
| `cre.lease` | `base_rent` (per period) | `lease_up_months` (1 — fully occupied from month one) | `cre.lease.base_rent` (inflow) |
| `cre.revenue_line{.<id>}` | `amount` (per period) **or** `amount_year` (annual) | `escalation` (0; may be an expression, e.g. `curve_value("cpi", time.date)`) | `cre.revenue.line{.<id>}` (inflow) |
| `cre.opex_line` | `amount` (per period) **or** `amount_year` (annual) | `escalation` (0), `pct_fixed` (1), `occupancy` (1) — each may hold an expression, e.g. `curve_value("occupancy", time.date)` | `cre.opex.line{.<id>}` (outflow) |
| `cre.exit_cap` | `noi_value` (annual), `exit_cap` | — | `cre.exit.sale` (inflow, once at `term_start`) |

`cre.lease` applies an optional straight-line lease-up ramp:

```
occupancy(m) = clamp((m + 1) / lease_up_months, 0, 1)
rent(m)      = base_rent * occupancy(m)
```

where `m` is months since `term_start`. With the default of 1 the ramp is
inert and rent is full from the first month.

`cre.exit_cap` values the sale as `noi_value / exit_cap` — state the
stabilized annual NOI you are capitalizing. To value off NOI the engine
derives from the modeled streams instead, use `cre.exit_forward`.

CRE contracts are additionally checked at compile time by pack validations
(`E6xxx_*`) covering missing required terms, term ranges outside the model
timeline, and out-of-range cap rates.

## Scenario testing (run configuration overrides)

The engine supports deterministic scenario overrides through run configuration files.
CRE fixtures and examples include:

- `run.base.json`
- `run.stress.json`
- `run.json` (single run containing multiple named scenarios)

Scenario knobs currently demonstrated:

- `stream.cre.lease.base_rent:amount`
- `stream.cre.opex.line:amount`
- `stream.cre.exit.sale:amount`

Example:

`cfdl run cre.ir.json --out cre.results.json --config run.json --packs packs`

## Provenance

All streams lowered by this pack include:
- source contract file/span
- `generated_by.pack.name = "cre"`
- `generated_by.pack.version = "0.1.0"`
- `generated_by.rule_id = <rule id>`

Determinism guarantees for this pack:

- deterministic file-based pack loading
- deterministic lowering rule application order
- deterministic IDs from compiler seed + stable keys
- deterministic results under identical IR + run configuration inputs

Owner binding notes:

- Lowering rules use `owner_entity = "${subject}"`.
- `${subject}` resolves to the contract subject entity declared in source.
- If a contract omits `on entity`, compiler compatibility fallback binds to the model's first declared entity.

## Validations status

CRE domain checks are enforced during lowering-time validation (compile path)
and emitted as standard diagnostics (`E6xxx_*`), without a separate pack
validation stage.

Current codes:

- `E6001_CRE_LEASE_MISSING_BASE_RENT`
- `E6002_CRE_LEASE_INVALID_TERM_RANGE`
- `E6003_CRE_LEASE_UP_MISSING_MONTHS`
- `E6010_CRE_EXIT_MISSING_EXIT_CAP`
- `E6011_CRE_EXIT_INVALID_EXIT_CAP`
- `E6012_CRE_EXIT_MISSING_NOI_VALUE`
- `E6020_CRE_OPS_MISSING_AMOUNT`


## Lease-by-lease contracts (institutional DCF parity)

Per-tenant contracts use suffixed names (`cre.lease_unit.tenant_a`); one rule
lowers every instance, emitting per-instance streams
(`cre.unit.base_rent.tenant_a`). Metrics aggregate them with `.*` wildcards.
Escalations anchor to **lease anniversaries** (`months_between(term_start,
time.date)`), not model years.

| Contract | Required terms | Optional (default) |
|---|---|---|
| `cre.lease_unit.<id>` | `rent_year` | `free_rent_months` (0), `escalation` (0), `expense_stop_year`/`opex_year`/`opex_escalation`/`pro_rata_share` (0 — recoveries off), `ti_total`/`lc_total` (0) |
| `cre.rollover.<id>` | `renewal_probability`, `renewal_rent_year`, `market_rent_year` | `market_escalation` (0), `downtime_months` (0), `renewal_ti_lc`/`new_ti_lc` (0). Term starts AT EXPIRY. |
| `cre.vacancy_loss` | `rate`, `potential_gross_year` | — |
| `cre.opex_line{.<id>}` | `amount` or `amount_year` | `escalation` (0), `pct_fixed` (1), `occupancy` (1) — expressions welcome. Instance it per expense for an itemized schedule; the entity it hangs on sets the level. |
| `cre.exit` | `noi_forward_year`, `exit_cap` | `selling_costs` (0); fires at `term_start` |
| `cre.exit_forward` | `exit_cap` | `selling_costs` (0); NOI derived via `series_sum` over the 12 months after sale |
| `cre.percentage_rent.<id>` | `sales_year`, `breakpoint_year`, `overage_pct` | `sales_growth` (0) — retail overage rent above the breakpoint |
| `cre.percentage_rent_expected.<id>` | `sales_quantile`, `breakpoint_year`, `overage_pct` | `sales_growth` (0) — the same overage as an EXPECTATION over a sales distribution |
| `cre.construction_loan` | `draw_curve` (a curve NAME, stated ANNUALIZED), `equity_commitment`, `rate` | `draw_accrual_fraction` (0.5 — drawn ratably through the period) |

Recoveries support expense stops with a `gross_up_factor` (opex grossed to
stabilized occupancy before the stop test); a base-year structure is the
stop set to year-0 grossed-up opex.

Rollover downtime follows industry-standard expected-value semantics: the window
starts at expiry, the renewal scenario (p × renewal) pays throughout, and the
re-let scenario's rent phases in once the downtime has elapsed. Turnover costs
split the same way — the renewal portion at expiry, the re-let portion when the
new tenant takes occupancy.

`cre.exit_forward` derives the sale-year NOI from the modeled streams over the
**year** after the sale date, which needs a projection tail of one year:
`project 12` on a monthly model, `project 4` quarterly, `project 1` annually.
`cre.exit` remains for analyst-supplied forward NOI.

Both exit rules settle **on their stated date**, which discounts from the start
of the period containing it rather than the end. On a monthly model that is one
month of discounting; on an annual model it is a full year, and a reversion is
usually taken at period end. If that matters to your model, express the sale as
a one-period stream on an ordinary schedule instead — see
the [rent-regulated plaza benchmark](/docs/examples/cre-mit-rentleg-plaza), which
does exactly that and says why.

### Simple whole-property contracts

`cre.lease`, `cre.revenue_line`, `cre.opex_line`, `cre.exit_cap`, and
`cre.construction_stub` model a property at the whole-asset level, for when
lease-by-lease detail is not warranted. They follow the same conventions as
the lease-by-lease set: schedules run over the contract's own term, time is
measured from `term_start`, and every material value is a required term —
the pack supplies no amounts, rates, or dates of its own.

### Overage rent at a point estimate, and over a distribution

`cre.percentage_rent` pays `max(0, sales - breakpoint) * pct` on a single sales
figure. That is a call option evaluated at the expected underlying, which is not
the expected value of the option — it is strictly below it, and it is exactly
zero whenever the point estimate sits under the breakpoint, however much
probability mass lies above.

A 1,200,000 breakpoint against sales expected at 1,000,000 pays **0.00** at the
point estimate and **4,937.50 a year** over a distribution with that same mean.
The gap is the whole payment, and a natural breakpoint above expected sales is
the ordinary case rather than a corner.

`cre.percentage_rent_expected` states the sales distribution as a `quantile` and
takes the partial expectation over it. Use it when the lease is worth
underwriting on a range. The original stays correct for a lease underwritten on
one figure, and is the form the reconciled retail benchmark uses.

## Quick start

A two-tenant office tower: lease-by-lease rent, recoveries above an expense
stop, property operating expenses, and probability-weighted rollover.

```cfdl
version 0.1
model "my-office"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 120 project 12

entity asset tower

// Full expense stop at the year-1 level: the tenant reimburses its share of
// opex growth above that stop, not the base.
contract cre.lease_unit.tenant_a on entity asset.tower {
  term 2026-01..2030-12
  terms {
    rent_year = 480000
    free_rent_months = 3
    escalation = 0.03
    opex_year = 300000
    opex_escalation = 0.025
    expense_stop_year = 300000
    pro_rata_share = 0.40
    ti_total = 120000
    lc_total = 80000
  }
}

// A lower stop: this tenant reimburses its share above $180k from day one.
// examples-allow: cre.unit.abatement.tenant_b — no free rent on this lease, so
// its abatement line is correctly zero. Tenant A's is not, and shows the split.
contract cre.lease_unit.tenant_b on entity asset.tower {
  term 2026-07..2033-06
  terms {
    rent_year = 360000
    escalation = 0.025
    opex_year = 300000
    opex_escalation = 0.025
    expense_stop_year = 180000
    pro_rata_share = 0.30
    ti_total = 100000
    lc_total = 50000
  }
}

// The expense the recoveries above are measured against.
contract cre.opex_line on entity asset.tower {
  term 2026-01..2036-12
  terms {
    amount_year = 300000
    escalation = 0.025
  }
}

// Rollover starts AT EXPIRY. During downtime only the renewal scenario pays.
contract cre.rollover.tenant_a on entity asset.tower {
  term 2031-01..2036-12
  terms {
    renewal_probability = 0.7
    renewal_rent_year = 520000
    market_rent_year = 560000
    market_escalation = 0.03
    downtime_months = 3
    renewal_ti_lc = 100000
    new_ti_lc = 350000
  }
}
```

The projection tail extends evaluation past the hold so exit valuation sees
a full forward year: `project 12` on a monthly model, `project 4` quarterly,
`project 1` annually. Rollover windows start AT EXPIRY; escalations
step on lease anniversaries.

## Run it

```bash
cfdl compile my-office --packs packs --out my-office/ir.json
cfdl run my-office/ir.json --packs packs --pack cre --out my-office/results.json --rate 0.07
```

## Recipes

**Exit on engine-derived forward NOI**:

```cfdl
contract cre.exit_forward on entity asset.tower {
  term 2035-12..2035-12
  terms { exit_cap = 0.0625 }
}
```

**Property-level opex** (escalating):

```cfdl
contract cre.opex_line on entity asset.tower {
  term 2026-01..2035-12
  terms { amount_year = 300000 escalation = 0.025 }
}
```

**Stochastic rollover** — draw the renew/re-lease outcome per trial instead
of expected-value blending; see the
[stochastic modeling guide](/docs/stochastic-modeling).

Full worked models: the [two-tenant office](/docs/examples/cre-office-two-tenant)
(full institutional-parity case), the
[retail strip](/docs/examples/cre-retail-strip) (base-year gross-up and
percentage rent), and the
[CRE office notebook](/docs/notebooks/cre-office-acquisition).

## Stream categories

Every stream this pack emits declares a `category`, and aggregation reads that
rather than pattern-matching the stream's name. A name is an address; a category
is a meaning. Deciding that `cre.vacancy.loss` is a deduction by looking at its
spelling means every metric, fold and statement re-derives the same judgement
independently — and they drift, which is exactly how two selector dialects came
to disagree about what `.*` matched.

Categories are dotted **paths** rooted in the cash flow statement's three
sections, so a subtotal is a prefix query over the same selector streams use:

| category | what it holds |
|---|---|
| `operating.revenue.base_rent` | contract and market rent, including rollover |
| `operating.revenue.other` | other operating income |
| `operating.revenue.percentage_rent` | overage rent |
| `operating.revenue.recovery` | expense reimbursements billed to tenants |
| `operating.deduction.vacancy` | vacancy and credit loss |
| `operating.deduction.abatement` | free rent |
| `operating.expense.opex` | property operating expenses |
| `investing.capital.leasing` | TI and leasing commissions |
| `investing.capital.construction` | construction draws |
| `investing.capital.capex` | general capital improvements |
| `investing.disposal.reversion` | sale proceeds at the end of the hold |
| `financing.debt.service` | principal and interest |

So net operating income is everything under `operating.*`, effective gross
income is `operating.revenue.*` plus `operating.deduction.*`, and the leasing
and capital costs that sit below the NOI line are `investing.capital.*`. No
subtotal has to list stream names.

`recovery` sits under `revenue` rather than beside it because a pro forma
reports it as its own line while still counting it above NOI — the tree
expresses both facts at once. `deduction` is deliberately not an `expense`:
vacancy is not a cost of operating the building, and netting the two would make
the expense ratio meaningless.

A hand-written stream may declare a category too, which is how a model expresses
something the pack has no contract for and still has it counted. This is exactly
what the [rent-regulated plaza benchmark](/docs/examples/cre-mit-rentleg-plaza)
does with its abatement line:

    stream cre.abatement.suite_200 on entity asset.rentleg outflow currency USD {
      schedule every year from 2001-01 to 2006-01
      category operating.deduction.abatement
      amount = ...
    }

An unlisted category is `E5022` rather than a new bucket, because the failure it
prevents is silent: the stream would still report as a line, so the statement
would look complete while the subtotal it belonged in came up short.
