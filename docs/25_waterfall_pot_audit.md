# Every waterfall in the repository builds its own pot — audit

Status: **findings.** No model has been changed.

## Scope

Every `.cfdl` file in the repository — 203 of them.

| | |
|---|---:|
| models | 203 |
| declaring contracts | 100 |
| declaring streams | 102 |
| declaring a field of their own | 41 |
| declaring a field whose name says "balance" | 21 |
| declaring a waterfall | 30 |
| waterfalls in total | 31 |

Packs create balances too, and they are the ones that work: 25 `field_name`
rules across four packs — `credit_level_pay_survival` and its lagged twin,
`credit_io_bullet_survival`, `credit_float_io_survival`,
`cre_construction_funded`, `opco_revenue_growth` and the rest. Every one of
them is a function of time, rate and schedule, which is why a contract can
maintain it without help.

The 21 hand-carried balances are the other kind: `americredit_2017_1`'s seven
note classes, the Fannie Mae tranche balance repeated across seven speed
variants, `lbo_circular_interest` and `lbo_financing_cases` carrying a debt
balance, `ppiaf_toll_highway` carrying a facility balance. Each exists because
something had to be reduced by what was paid, and nothing could do it.

## The rule

`docs/17` §4:

> A waterfall runs **after** the period's fields and streams are known, and
> before results are published. It reads period-close state, because **the pot
> it allocates is this period's cash** and the balances it measures are this
> period's balances.

`docs/17` §10:

> A waterfall is a **post-free-cash-flow distribution**, and it happens on a
> cadence of its own.

So a distribution allocates the period's netted cash. It does not compute cash,
and it does not reach behind the netting to the lines that produced it.

## The engine's cash-flow path, in order

Every line number is a call site in `crates/cfdl-engine/src/lib.rs`, in the
order the engine runs them:

| line | stage | what it produces |
|---:|---|---|
| 1012 | `compute_states` | every field, every period — the balances a model declares |
| 1013 | `simulate_events` | per-period entity state, reading those fields |
| ~1080 | streams, phase 1 | streams that read no series |
| 1122 | streams, phase 2 | streams that read phase-1 series |
| 1180 | subtotals | folds of stream categories, for statements |
| **1308** | **`run_waterfalls`** | **the distributions** |
| 1403 | `model.net_cash_flow` | the model's netted cash, per period |
| 1472 | `entity_rollup` | each entity's netted cash, per period |

The netting a distribution exists to allocate is assembled at 1403 and 1472.
The distributions ran at 1308.

## The netting itself, and when it happens

`crates/cfdl-engine/src/lib.rs:1470-1500` folds every stream's signed value into
its owning entity, then walks `part of` to add each entity's cash into every
ancestor:

```rust
let mut cursor: Option<&str> = Some(symbol.as_str());
while let Some(current) = cursor {
    let slot = entity_rollup.entry(current.to_string())…;
    for (idx, value) in own.iter().enumerate().take(periods) {
        slot[idx] += value;
    }
    cursor = parent_of.get(current).copied();
}
```

One value per entity per period, aggregated by the relation rather than by name
prefix — what `docs/01` §7.1 promises, published as
`entity.<symbol>.net_cash_flow`.

Two facts about it:

1. **It is never bound into a waterfall's environment.** `docs/03` §3 lists the
   namespaces a waterfall step can read: `model`, `entity` (fields only),
   `asset`/`party`/`contract`/`reference` (the same fields, spelled bare),
   `time`, `inputs`, `cfg`, `obs`, curves, and `series_sum`. There is no entry
   for an entity's cash. `asset.x.net_cash_flow` is `E1131_UNKNOWN_FIELD_READ`;
   `entity.asset.x.net_cash_flow` resolves to `Optional(None)` and the pot
   silently becomes zero.
2. **It is computed at line 1470, and `run_waterfalls` is called at line 1308.**
   The distributions are finished before the engine works out what any entity
   had.

So the quantity §4 says a waterfall allocates is computed exactly, and is
unavailable to the thing that should allocate it.

## What every waterfall does instead

31 waterfalls across 21 models and fixtures. **29 build their own pot; 2 read an
earlier waterfall, which §10 and §12 explicitly allow.**

### A. Reconstructs the flow from the pack's internal stream ids — 18

Sums `credit.pool.sched_principal.*`, `credit.pool.prepay.*`,
`credit.pool.interest.*` and, where the deal needs it, adds the pack's negative
`credit.pool.servicing.*` to net the fee off by hand — re-doing the engine's
netting one stream family at a time.

| model | waterfall |
|---|---|
| `benchmarks/credit/americredit_2017_1` | `notes.distribution` |
| `benchmarks/credit/auto_abs_tranches` | `notes.principal` |
| `benchmarks/credit/fnma_remic_2019_2_g3` | `g3.principal`, `g3.interest` |
| `…_psa000`, `…_psa100`, `…_psa300`, `…_psa400`, `…_psa700`, `…_psa1000` | `g3.principal`, `g3.interest` (each) |
| `fixtures/valid/waterfall_after_contract` | `notes.principal` |
| `fixtures/valid/evaluation_order` | `dist` |

This is the largest group and the most fragile. The pot names pack internals, so
it breaks when a pack adds a line, double-counts when a selector overlaps a bare
name, and sums to zero without a diagnostic when a name changes. `americredit`
carries the extreme case: the pot is four terms with a first-period window
correction, and the same components are then restated a second time as entity
fields, because the balances need them too.

### B. A hand-entered assumption, disconnected from the model's cash — 5

| model | waterfall | pot |
|---|---|---|
| `fixtures/valid/waterfall_fund_carry` | `fund.distribution` | `inputs.proceeds` |
| `fixtures/valid/waterfall_irr_hurdles` | `fund.distribution` | `inputs.proceeds` |
| `fixtures/valid/waterfall_nested_split` | `fund.distribution` | `inputs.proceeds` |
| `fixtures/valid/waterfall_partial_catchup` | `fund.distribution` | `cfg.proceeds` |
| `fixtures/valid/waterfall_cre_jv_promote` | `jv.distribution` | `inputs.net_sale_proceeds` |

The distribution allocates a number the author typed. Nothing connects it to
what the model's own assets produced, so the two can disagree without any check
noticing.

### C. A hand-maintained field — 4

| model | waterfall | pot |
|---|---|---|
| `fixtures/valid/waterfall_abs_22_step` | `abs.distribution` | `asset.trust.available_funds` |
| `fixtures/valid/waterfall_smoke` | `abs.distribution` | `asset.trust.available_funds` |
| `benchmarks/energy/tax_equity_flip` | `partnership.distribution` | `asset.project.cash` |
| `benchmarks/opco/lbo_option_pool_exit` | `opco.exit` | `asset.target.exit_equity + 44.500` |

These are the closest to right in intent and the furthest in fact. The 22-step
fixture writes `available_funds init 12500000.0 next prev * 0.97` — a cash
balance decaying by an invented factor, and its own comment says the figures are
"illustrative, not the deal's," because nothing can post real collections into
it. The LBO exit adds a bare `44.500` to a field.

### D. A literal — 1

`fixtures/valid/entity_property_bare_path`, `abs.distribution`, `from 10000.0`.

### E. Recomputes revenue inside the pot — 1

`fixtures/valid/flip_monthly_grain`, `interest.distribution`:
`inputs.energy_year_one * inputs.ppa_price * pow(1 + inputs.ppa_escalation, …)`.
The energy revenue is computed a second time, in the distribution, from the same
assumptions the streams use.

### F. Composition — 2, and these are correct

`fixtures/valid/waterfall_nested_split`: `firm.carry_allocation` reads
`fund.distribution.gp_catchup`, and `firm.owner_split` reads
`firm.carry_allocation.firm_share`. §10 and §12 provide for exactly this — one
waterfall's payment becoming another's pot.

## What this costs

- **The netting is done twice**, once by the engine and once by hand, and only
  the hand copy is asserted. `americredit_2017_1` shipped with the servicing fee
  charged for two months in the waterfall (correctly, via the pack's series) and
  one month in its balance recurrence, and eleven published cells were wrong
  with every assertion in the case passing.
- **A pot can silently be zero.** An unmatched literal series name aggregates to
  nothing without a diagnostic, and the `W5022` warning added this week does not
  fire for a name that exists but is not visible in that context.
- **The fixtures teach it.** `waterfall_abs_22_step` is the reference example for
  the construct, and its pot is a decaying invented balance. Every model written
  from it will build its own pot too.

## What would close it

A waterfall is a module with one input and one kind of output. The input is the
available cash. The outputs are amounts to payees. Nothing else crosses the
boundary.

The input has a name already, and it is not a field. `remaining` is a
waterfall-scoped binding the engine supplies. **`available` is its sibling**:
the netted cash of the entity the waterfall is attached to, for this period,
supplied by the result layer.

```cfdl
waterfall notes.distribution on entity asset.trust {
  schedule every month from 2017-02 to 2022-11
  from available

  pay servicing to party.servicer = …
  pay residual  to party.certificate = remaining
}
```

Cash stays where it belongs. It does not become a property of the asset, and no
model declares a field to hold it. Categories A through E above collapse into
one line.

The engine change is contained. `entity_own` folds signed stream values by
owner and reads only stream values and the `part of` relations, both complete at
line 1122. It runs at 1431 because the results assemble last. The stream-derived
half moves above `run_waterfalls`; the payee attribution in the same fold stays
below it, because a waterfall's payments are what it attributes.

## The specifications teach the workaround

Three documents illustrate the pot with a field the modeller declares and
fills:

| document | example |
|---|---|
| `docs/01` §10.1, the waterfall's syntax | `from asset.trust.available_funds` |
| `docs/17` §3, the proposed surface | `from state.available_funds` |
| `docs/18` §3, arguing for the field spelling | "`from asset.trust.available_funds` says where the pot comes from" |

`docs/17` §3 is doubly stale: a model-level `state` no longer exists, having
been removed for overloading the lifecycle concept, so that example names a
construct the language dropped.

All three should read `from available` once the binding exists. Until then they
teach every reader to invent a field, which is what
`fixtures/valid/waterfall_abs_22_step` does — `available_funds init 12500000.0
next prev * 0.97`, a balance that decays by an invented factor.

Provenance: found while writing `benchmarks/credit/americredit_2017_1`, whose
pot is the worst case in category A, August 2026.
