## The case

A stable-growth dividend discount valuation of a regulated utility — the
constant-growth perpetuity, where value is next year's dividend divided by the
difference between the discount rate and the growth rate.

The whole model is two drivers and one formula, which is why it is a precise
test: there is nowhere for an error to hide.

## The reference

A widely used academic valuation spreadsheet, published free by its author with
an explicit grant to download and modify. It publishes a nine-point sensitivity
grid over growth rates alongside the base case.

**Redistributable**, and the workbook is committed under `reference/`.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Contract types | `opco.exit_perpetuity` |
| Language features | a single pack contract carrying a closed-form perpetuity |
| Conventions | constant-growth perpetuity, a value derived from stated drivers rather than supplied directly |

The perpetuity value is **derived** by the model from two stated drivers — a
current dividend and a growth rate — rather than entered. Supplying the answer
would test nothing.

## The result

The base case reproduces, and so does the source's own nine-point growth
sensitivity grid.

The tolerance is **1e-6**, the tightest in the suite: a closed-form perpetuity on
exactly stated inputs has no rounding to absorb, so anything looser would accept
an error.

## The delta

None.
