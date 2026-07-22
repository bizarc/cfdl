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
| `energy.ppa` | `mwh_year`, `ppa_price` | `escalation` (0), `degradation` (0) |
| `energy.merchant` | `mwh_year`, `price` | `price_escalation` (0), `degradation` (0) |
| `energy.storage_arbitrage` | `mwh_cycled_year`, `spread` | `degradation` (0) |
| `energy.capacity` | `payment_year` | — |
| `energy.om` | `om_year` | `escalation` (0) |
| `energy.itc` | `credit` | — (fires on `term_start`) |
| `energy.capex` | `amount` | — (fires on `term_start`) |
| `energy.debt_service` | `rate`, `term_months`, `principal` | — |

## Not yet modeled (roadmap)

MACRS depreciation / tax equity, partnership flip structures (HLBV),
DSCR-sculpted debt sizing, availability/curtailment adjustments, PTC.
These arrive in later Workstream D increments; see LAUNCH_PLAN.md.
