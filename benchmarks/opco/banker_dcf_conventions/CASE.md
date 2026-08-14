## The case

A sell-side banker's discounted cash flow valuation of an enterprise-software
business. The difficulty is entirely in the timing. The valuation date is 30
September and the fiscal year ends 30 June, so the first forecast period is a
nine-month stub and the full years that follow sit at 1.25, 2.25, 3.25 and 4.25
years out rather than at whole numbers. Cash flows are discounted mid-period; the
terminal value is not.

## The reference

Banker's discussion materials filed as an exhibit to a merger document. It
gives the unlevered free cash flow build-up line by line, the discount rate
range, the terminal method and multiple, the discounting convention, the
dilution assumption, and a 3×3 grid of implied enterprise values — more than
most fairness opinions, which disclose a value range and little else.

**Not redistributable.** The filer retains copyright, so figures are asserted
against and the document is not vendored. The exhibit uses code names for the
parties, so the case describes the *analysis* rather than the company.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Declared | six native streams |
| Language features | native streams placed on specific dates to carry a convention |
| Conventions | mid-period discounting, a stub period at its own midpoint, full years on quarter-year boundaries, a terminal value discounted whole, cumulative dilution |

## The result

`model.npv` = **15,764** against the filing's published 15,764, at the center of
the disclosed grid — 10.375% discount rate, 25.0× terminal multiple.

All nine cells of the grid reconcile; the worst is +1.17 on $19bn. The other
eight need a different rate or multiple per run, so the asserted cell is the
center one.

## The delta

±1 is the floor the source can support: the filing rounds to whole millions and
its build-up lines round the same way. The engine's own agreement on the asserted
cell is +0.16.

The convention is asymmetric: flows are discounted mid-period and the terminal
value whole, a terminal value being a price struck at a point in time. The
filing's own figures confirm it.
