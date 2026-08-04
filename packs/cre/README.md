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
- stabilized operations (`cre.ops_revenue`, `cre.ops_expense`)
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
- `cre.ops_revenue`
- `cre.ops_expense`
- `cre.exit_cap`
- `cre.lease_unit.<id>`, `cre.rollover.<id>`, `cre.property_opex`,
  `cre.vacancy_loss`, `cre.percentage_rent`, `cre.exit_forward`
- `cre.permanent_debt`

### `cre.permanent_debt`

A commercial mortgage on a stabilised property. Emits one stream,
`loan.permanent_debt_service`, which is the exact name `domain.cre.debt_service`
selects — and therefore what `domain.cre.dscr` divides by.

| term | meaning | default |
|---|---|---|
| `principal` | loan amount | *required* |
| `rate` | nominal annual rate | *required* |
| `amort_months` | amortisation term — strikes the payment | *required* |
| `io_months` | interest-only months before amortisation begins | `0` |
| `balloon_at_maturity` | `1` pays the unamortised balance as debt service at `term_end` | `0` |
| `payment_frequency` | `day`/`week`/`month`/`quarter`/`year` | the model calendar |
| `day_count`, `amortization_day_count` | interest accrual and payment bases | `30/360` |

**`amort_months` is normally longer than the term.** A 30-year amortisation on a
10-year loan is the standard commercial structure, and it is why a balloon
exists at all.

**The balloon defaults off.** Coverage is measured on *periodic* debt service, so
folding an unamortised balance into the final period would make that period's
DSCR meaningless; the standard pro forma repays it out of the sale. Turn it on
when the payoff genuinely belongs in the debt service line.

**Not modelled:** sizing to a target coverage ratio (a solve), refinance (needs
the events layer), and mortgage insurance — MIP is not a payment on the debt.
See `docs/13_feature_backlog.md` 7.14.

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
| `cre.ops_revenue` | `amount` (per period) | — | `cre.ops.revenue` (inflow) |
| `cre.ops_expense` | `amount` (per period) | — | `cre.ops.expense` (outflow) |
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

## Scenario testing (run config overrides)

The engine supports deterministic scenario overrides through run config files.
CRE fixtures and examples include:

- `run.base.json`
- `run.stress.json`
- `run.json` (single run containing multiple named scenarios)

Scenario knobs currently demonstrated:

- `stream.cre.lease.base_rent:amount`
- `stream.cre.ops.expense:amount`
- `stream.cre.exit.sale:amount`

Example:

`./target/debug/cfdl run /tmp/cre.ir.json --out /tmp/cre.results.json --config fixtures/valid/cre_developer_scenarios/run.json --packs packs`

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
- deterministic results under identical IR + run config inputs

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
- `E6004_CRE_LEASE_UP_INVALID_OCCUPANCY`
- `E6010_CRE_EXIT_MISSING_EXIT_CAP`
- `E6011_CRE_EXIT_INVALID_EXIT_CAP`
- `E6012_CRE_EXIT_MISSING_NOI_REF_OR_VALUE`
- `E6020_CRE_OPS_MISSING_AMOUNT`
- `E6021_CRE_OPS_INVALID_SCHEDULE`


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
| `cre.property_opex` | `opex_year` | `escalation` (0) |
| `cre.exit` | `noi_forward_year`, `exit_cap` | `selling_costs` (0); fires at `term_start` |
| `cre.exit_forward` | `exit_cap` | `selling_costs` (0); NOI derived via `series_sum` over the 12 months after sale |
| `cre.percentage_rent.<id>` | `sales_year`, `breakpoint_year`, `overage_pct` | `sales_growth` (0) — retail overage rent above the breakpoint |

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
`benchmarks/cre/mit_rentleg_plaza`, which does exactly that and documents why.

### Simple whole-property contracts

`cre.lease`, `cre.ops_revenue`, `cre.ops_expense`, `cre.exit_cap`, and
`cre.construction_stub` model a property at the whole-asset level, for when
lease-by-lease detail isn't warranted. They follow the same conventions as
the lease-by-lease set: schedules run over the contract's own term, time is
measured from `term_start`, and every material value is a required term —
the pack supplies no amounts, rates, or dates of its own.

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
contract cre.property_opex on entity asset.tower {
  term 2026-01..2036-12
  terms {
    opex_year = 300000
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
contract cre.property_opex on entity asset.tower {
  term 2026-01..2035-12
  terms { opex_year = 300000 escalation = 0.025 }
}
```

**Stochastic rollover** — draw the renew/re-lease outcome per trial instead
of expected-value blending; see `fixtures/valid/cre_stochastic_rollover/`
and the stochastic-modeling docs.

Full worked models: `benchmarks/cre/office_two_tenant/` (full institutional-parity
case), `benchmarks/cre/retail_strip/` (base-year gross-up + percentage
rent), and the CRE office notebook in `examples/notebooks/`.

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
| `investing.reversion` | sale proceeds at the end of the hold |
| `financing.debt_service` | principal and interest |

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
what `benchmarks/cre/mit_rentleg_plaza` does with its abatement line:

    stream cre.abatement.suite_200 on entity asset.rentleg outflow currency USD {
      schedule every year from 2001-01 to 2006-01
      category operating.deduction.abatement
      amount = ...
    }

An unlisted category is `E5022` rather than a new bucket, because the failure it
prevents is silent: the stream would still report as a line, so the statement
would look complete while the subtotal it belonged in came up short.
