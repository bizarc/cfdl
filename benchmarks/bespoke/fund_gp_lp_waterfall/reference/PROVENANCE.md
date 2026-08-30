# Reference provenance

**Not redistributable.** The reference is a private fund model held in the
research corpus. The workbook is not committed; this file and the frozen input
set under `inputs/` are what the case carries.

The frozen inputs are the fund's monthly distributable cash flow and the
partnership's stated economics. No party, property, fund or manager name from
the source is carried into this case: the parties here are `lp`, `gp`, `gp_lp`
and `gp_gp`, which are the roles the tiers name, not the entities that signed.

## What the reference supplies

| | |
|---|---|
| Grain | monthly, 30 periods from 2017-08 |
| Contribution | 31,000,000 at period 0, split 90 / 10 |
| Distributions | 39,973,982.80 over 29 periods |
| Net | 8,973,982.80 |

The reference publishes a per-period allocation for each party across each
tier, and an investment summary giving each party's total by tier, its total
return and its annual return. Both are asserted.

## What is NOT taken from the reference

The fund's cash flow is an INPUT here, not a reproduction, and the source does
compute it rather than assume it: `Waterfall` reads `Property Summary`, which
sums twenty-five per-property sheets, each deriving its debt service with `PMT`
from a capital stack and assembling a monthly schedule gated on its closing,
interest-only, rent-commencement and sale dates.

This case takes the resulting monthly total as given and reconciles only the
distribution of it. The property layer is a separate case.

## Scrubbing

`inputs/fund_cash_flow.csv` carries period, date and amount only.
`inputs/terms.json` carries rates and splits only. Neither carries a name, an
address, an asset identifier or a sheet reference from the source.
