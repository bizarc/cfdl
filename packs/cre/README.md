# CRE Pack v0.1.0

This pack provides deterministic lowering for a minimal Commercial Real Estate
developer lifecycle:

- construction (`cre_construction_stub`)
- lease-up (`cre_lease`)
- stabilized operations (`cre_ops_revenue`, `cre_ops_expense`)
- exit (`cre_exit_cap`)

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

- `cre_construction_stub`
- `cre_lease`
- `cre_ops_revenue`
- `cre_ops_expense`
- `cre_exit_cap`

## Expected terms (authoring contract)

Contract `terms { ... }` payloads are captured as a lightweight key/value map
and validated by CRE lowering-time checks (`E6xxx_*`) during compile.

### `cre_lease`

Required:
- `start` / `end` period dates
- `base_rent` (Money-equivalent numeric amount in model currency)
- `frequency` (`monthly`)

Optional:
- `growth` (Decimal)
- `free_rent_months` (Int)
- `lease_up.start_period` (Int; default `0`)
- `lease_up.months` (Int; required when lease-up terms are supplied)
- `lease_up.start_occupancy` (Decimal; default `0.0`)
- `lease_up.end_occupancy` (Decimal; default `1.0`)

Lowering output:
- stream `cre.lease.base_rent` inflow to the contract subject entity (`on entity ...`)

Current deterministic implementation uses a built-in linear occupancy ramp for
`cre_lease`:

- occupancy(t) = `clamp((t - 6 + 1) / 18, 0, 1)`
- rent(t) = `base_rent * occupancy(t)` (with `base_rent = 25000` in v0.1 rules)

Important implementation note:

- `lease_up.*` names are validated when present.
- Current rent-ramp math still uses deterministic v0.1 rule defaults.
- The active v0.1 behavior is the deterministic default ramp above.
- Scenario testing can still vary effective lease-up economics using run-config
  overrides (for example, `stream.cre.lease.base_rent.amount`).

### `cre_exit_cap`

Required:
- `exit_period` (Int) or `exit_date`
- `exit_cap` (Decimal)
- `noi_ref` (identifier/expression)

Lowering output:
- one terminal sale inflow stream `cre.exit.sale` at the configured exit date
- simple cap-rate shape (`NOI / exit_cap`) in rule form

## Scenario testing (run config overrides)

The engine supports deterministic scenario overrides through run config files.
CRE fixtures and examples include:

- `run.base.json`
- `run.stress.json`
- `run.json` (single run containing multiple named scenarios)

Scenario knobs currently demonstrated:

- `stream.cre.lease.base_rent.amount`
- `stream.cre.ops.expense.amount`
- `stream.cre.exit.sale.amount`

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
