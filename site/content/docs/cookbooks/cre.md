---
id: cookbook-cre
title: "CRE pack guide"
slug: "/docs/cookbooks/cre"
source: packs/cre/README.md
---

This pack provides deterministic lowering for a minimal Commercial Real Estate
developer lifecycle:

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
starts at expiry, the first `downtime_months` pay only the renewal-scenario
rent (p × renewal), and the full probability-weighted blend applies after.
`cre.exit_forward` derives the sale-year NOI from the modeled streams over
the 12 months after the sale date (requires `time ... project 12`);
`cre.exit` remains for analyst-supplied forward NOI. Remaining simplification
(documented): blended rollover TI/LC pays entirely at expiry rather than
splitting the new-lease portion to after downtime.

### Simple whole-property contracts

`cre.lease`, `cre.ops_revenue`, `cre.ops_expense`, `cre.exit_cap`, and
`cre.construction_stub` model a property at the whole-asset level, for when
lease-by-lease detail isn't warranted. They follow the same conventions as
the lease-by-lease set: schedules run over the contract's own term, time is
measured from `term_start`, and every material value is a required term —
the pack supplies no amounts, rates, or dates of its own.

## Quick start

A two-tenant office tower, lease-by-lease with recoveries and rollover:

```cfdl
version 0.1
model "my-office"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 120 project 12

entity asset tower

contract cre.lease_unit.tenant_a on entity asset.tower {
  term 2026-01..2030-12
  terms {
    rent_year = 480000
    free_rent_months = 3
    escalation = 0.03
    opex_year = 300000
    expense_stop_year = 300000
    pro_rata_share = 0.40
    ti_total = 120000
    lc_total = 80000
  }
}

contract cre.rollover.tenant_a on entity asset.tower {
  term 2031-01..2036-12
  terms {
    renewal_probability = 0.7
    renewal_rent_year = 520000
    market_rent_year = 560000
    downtime_months = 3
  }
}
```

The `project 12` tail extends evaluation past the hold so exit valuation
sees a full forward year. Rollover windows start AT EXPIRY; escalations
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

## Worked example models

Benchmark cases are validated period-by-period against an independent
reference implementation.

- [CRE: two-tenant office](/docs/examples/cre-office-two-tenant)
- [CRE: retail strip with expense stops](/docs/examples/cre-retail-strip)
- [CRE examples overview](/docs/examples/cre-examples)
- [Lease-up](/docs/examples/cre_lease_up)
- [Developer lifecycle](/docs/examples/cre_developer)
- [Phased development](/docs/examples/cre_phased)
- [Multi-file model](/docs/examples/cre_multi_file)
- [Development with financing](/docs/examples/cre_development_with_financing)
