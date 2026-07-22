# energy pack (v0.1)

Energy & microgrids: solar/wind PPA and merchant revenue, battery storage
arbitrage, capacity payments, O&M, investment tax credits, capex, and level-pay
project debt. All rules are template-driven (`{{contract.*}}` + defaults) —
no hardcoded amounts.

## Conventions

- Annual quantities (`mwh_year`, `om_year`, `payment_year`) spread evenly
  across months.
- Escalation and degradation step **annually**: `factor ^ floor(t / 12)`,
  matching common project-finance Excel practice.
- `energy.debt_service` uses the engine's decimal-exact `pmt()` (Excel sign
  conventions).

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
| `energy.debt_service` | `rate`, `term_months`, `principal` | — |
| `energy.ptc` | `mwh_year`, `credit_per_mwh` | `escalation` (0), `degradation` (0), `availability` (1); term bounds the credit window |
| `energy.macrs_shield` | `basis`, `tax_rate` | `life` (5; also 7/15/20) — IRS Pub 946 GDS half-year tables via `macrs_rate()` |

Tax attributes (ITC, PTC, MACRS shield) report under
`domain.energy.tax_benefits` and are excluded from revenue/EBITDA.

## Not yet modeled (roadmap)

Tax equity / partnership flip structures (HLBV), DSCR-sculpted debt sizing,
full tax computation (the MACRS stream models the shield value, not taxable
income). Planned for later pack increments.
