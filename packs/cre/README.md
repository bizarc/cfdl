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

The parser/pack host in this SDK revision does not enforce pack-level contract
term schemas yet. Model authors should still provide the following terms in
their contract body for forward compatibility.

### `cre_lease`

Required:
- `start` / `end` period dates
- `base_rent` (Money-equivalent numeric amount in model currency)
- `frequency` (`monthly`)

Optional:
- `growth` (Decimal)
- `free_rent_months` (Int)

Lowering output:
- stream `cre.lease.base_rent` inflow to `real_estate.property`

### `cre_exit_cap`

Required:
- `exit_period` (Int) or `exit_date`
- `exit_cap` (Decimal)
- `noi_ref` (identifier/expression)

Lowering output:
- one terminal sale inflow stream `cre.exit.sale` at the configured exit date
- simple cap-rate shape (`NOI / exit_cap`) in rule form

## Provenance

All streams lowered by this pack include:
- source contract file/span
- `generated_by.pack.name = "cre"`
- `generated_by.pack.version = "0.1.0"`
- `generated_by.rule_id = <rule id>`

## Validations status

Pack-level validations are not wired into the host pipeline yet. Planned CRE
diagnostics are reserved in the `E6xxx_*` range and can be implemented once
pack validation hooks are available.
