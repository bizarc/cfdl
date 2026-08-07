## The case

A 2 MW solar and storage microgrid selling under a 25-year power purchase
agreement, with production degrading each year and a fixed escalator on the
contracted price. Level-pay project debt sits underneath.

## The reference

Project-finance conventions for a contracted renewable asset: contracted offtake
with degradation and escalation, operations and maintenance, and sculpted debt.

**Not redistributable.** The source cannot be published, so its conventions are
recreated independently of the model and compared period by period.

## What it exercises

| | |
|---|---|
| Pack | `energy` |
| Contract types | `energy.ppa`, `energy.storage_arbitrage`, `energy.om`, `energy.debt_service`, `energy.capex` |
| Language features | pack contracts composing revenue, cost and debt on one asset |
| Conventions | production degradation, contracted price escalation, storage arbitrage margin, level-pay debt |

## The result

Present value **1,220,668.85**, undiscounted total **5,771,865.78** and lifetime
revenue **12,594,004.52**.

Asserted: net cash flow per period, plus the three summary figures.

## The delta

None: every period agrees inside a one-cent tolerance.
