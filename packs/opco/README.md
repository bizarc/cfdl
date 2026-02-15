# OpCo Pack v0.1.0

`opco` provides deterministic lowering for a basic operating business valuation:

- recurring revenue (`opco_revenue_line`)
- recurring opex (`opco_opex_line`)
- optional working capital adjustment (`opco_working_capital`)
- terminal exit value (`opco_exit_multiple`)

## Activation

```cfdl
use pack "opco" version "0.1.0"
```

## Canonical aliases

- `opco.RevenueLine` -> `opco_revenue_line`
- `opco.OpexLine` -> `opco_opex_line`
- `opco.WorkingCapital` -> `opco_working_capital`
- `opco.ExitMultiple` -> `opco_exit_multiple`

## Lowering behavior

- Revenue/Opex/WC contracts lower to recurring streams using contract `term` range.
- Exit multiple lowers to one terminal inflow at `exit_period`.
- Exit amount uses simple deterministic math: `base_value * exit_multiple`.

## Required terms

### `opco_revenue_line` and `opco_opex_line`
- `amount` (numeric)
- `term` range must be valid and within timeline
- optional `growth_rate` (numeric)

### `opco_working_capital`
- `amount` (numeric)
- valid `term` range

### `opco_exit_multiple`
- `exit_period` (Int, in timeline bounds)
- `exit_multiple` (Decimal > 0)
- `base_value` (numeric)

## Lowering-time diagnostics (E7xxx)

- `E7001_OPCO_LINE_MISSING_AMOUNT`
- `E7002_OPCO_LINE_INVALID_SCHEDULE`
- `E7003_OPCO_LINE_INVALID_GROWTH`
- `E7010_OPCO_WC_MISSING_AMOUNT_OR_RULE`
- `E7011_OPCO_WC_INVALID_SCHEDULE`
- `E7020_OPCO_EXIT_MISSING_MULTIPLE`
- `E7021_OPCO_EXIT_INVALID_MULTIPLE`
- `E7022_OPCO_EXIT_MISSING_BASE_VALUE`
- `E7023_OPCO_EXIT_MISSING_EXIT_PERIOD`

## Provenance and determinism

Generated streams include:
- source contract file/span
- `generated_by.pack.name = "opco"`
- `generated_by.pack.version = "0.1.0"`
- `generated_by.rule_id`

Determinism guarantees:
- deterministic rule ordering
- deterministic diagnostics ordering
- deterministic IDs and results under identical inputs
