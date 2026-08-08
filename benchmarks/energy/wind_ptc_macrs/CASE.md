## The case

A 30 MW wind project selling at merchant prices, claiming the production tax
credit over its first ten years, and depreciating on the five-year MACRS
schedule. Project debt runs underneath. The credit runs for ten years,
depreciation for five and the debt for longer than either, so the cash flow
shape changes twice before the hold ends.

## The reference

Project-finance conventions for a merchant wind asset with federal tax
attributes: the production credit's inflation adjustment and statutory ten-year
window, and the MACRS half-year convention.

**Not redistributable.** The source cannot be published, so its conventions are
recreated independently of the model and compared period by period.

## What it exercises

| | |
|---|---|
| Pack | `energy` |
| Contract types | `energy.merchant`, `energy.ptc`, `energy.om`, `energy.debt_service`, `energy.macrs_shield`, `energy.capex` |
| Language features | pack contracts with staggered terms on one asset |
| Conventions | merchant pricing, a ten-year production credit, MACRS five-year depreciation, level-pay debt |

## The result

Present value **−5,452,881.52**, lifetime revenue **90,382,400.52** and lifetime
EBITDA **58,795,819.78**.

Asserted: net cash flow per period, plus the three summary figures.

## The delta

None: every period agrees inside a one-cent tolerance, including the periods
where the credit expires and where depreciation runs out.
