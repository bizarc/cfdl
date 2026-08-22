## The case

A sponsor take-private of a subscription-software business. Stock-based
compensation is the most contested convention in software valuation, and the
source discloses free cash flow **both ways** — before and after it — for the
same company on the same page. The gap is not a rounding difference: $331m
before, $198m after, in the first year alone. Two thirds of first-year cash
flow turns on the convention.

## The reference

Banker's discussion materials filed as an exhibit to a going-private
transaction. It discloses the cash flow build-up line by line, the
stock-compensation line, the post-compensation series, the discount rate range,
the terminal method, the discounting convention, the valuation date, and a 3x4
grid of implied enterprise values.

**Not redistributable.** The filer retains copyright, so figures are asserted
against. The exhibit uses code names for both parties, so the case describes the
analysis rather than the company.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Declared | ten native streams |
| Language features | stock compensation modeled as its own stream, so both conventions come from one model |
| Conventions | mid-period discounting, a nine-month stub, a terminal multiple struck on a pre-compensation base |

Compensation is a separate stream on the same date as the flow it burdens, so
the post-compensation series is *derived* rather than restated and cannot drift
from the pre-compensation one.

## The result

`model.npv` = **7,096** against the filing's published 7,096, at 13.5% and 16.0x.

All twelve cells of the disclosed grid reconcile, worst **±0.50 on ~7,000** —
0.007%, inside the filing's own whole-million rounding.

## The delta

The filing **mixes the two conventions**: the explicit-period flows are
post-compensation while the terminal multiple is applied to a pre-compensation
base. Nothing in the document says so. It is defensible — the multiple was
calibrated on peers' pre-compensation cash flow — but one model has to carry
both definitions at once.

One input is not disclosed. The filing states first-year cash flow as a full year
and notes the valuation includes only the last three quarters, without publishing
the split. That figure is solved from the grid: one unknown against twelve
published values, leaving eleven degrees of freedom. At the solved value all
twelve land within ±0.50. It comes out at 68.4% of the year rather than 75%,
which is the expected direction for an annual-prepaid subscription business where
the first quarter carries the cash.
