---
id: reference-slices
title: "Slices"
slug: "/docs/reference/slices"
description: "Named, deliberately partial selections of a model's streams — what a slice selects, what it publishes, and why it carries no reconciliation."
generated: none
---

# Slices

A **slice** is a named, deliberately partial selection of a model's own
streams, with figures computed over the selection: one artist's royalties, a
label minus its merchandise line, everything that is debt, one asset over two
years.

```cfdl
slice artist_a_royalties {
  entity asset.artist_a
  category "operating.revenue.royalty"
}

slice label_ex_merch {
  entity container.label
  except category "operating.revenue.merchandise"
}

slice debt {
  type Contract.Debt
}

slice west_2027 {
  entity asset.west_tower
  window from 2027-01 to 2028-12
}
```

## Clauses

Clause **kinds intersect**; values within a kind **union**; `except`
subtracts last. A kind that is absent does not constrain, so a slice of
nothing but excepts reads "everything minus these".

- `entity` (and `except entity`) takes a **reference** — an undeclared entity
  is refused (`E1362`) — and selects the entity together with its `part of`
  descendants, so a container's slice is its members'.
- `type` names an ontology type, matched transitively through `refines`:
  `type Contract.Debt` selects every stream lowered from a contract whose
  type is_a `Contract.Debt`, and streams owned by entities of a conforming
  type. An unknown type is refused with the known types named (`E1363`).
- `category` and `stream` take **quoted selectors** — the same dialect
  `series_sum` reads, exact or one trailing `.*` — and a category selector
  must be rooted in a statement section (`E1364`).
- `window from <date> to <date>` bounds the **periods**, where every other
  clause bounds the streams. A period outside it contributes nothing, so the
  slice's figures are folds over the window. At most one window per slice.
  Dates, not period indices: a window that survives a change of calendar has
  to be stated in dates. A window is not a phase — a phase is a lifecycle
  anchor that drives schedules; a window is a reporting bound applied to a
  finished projection.

A slice name must be unique within the model (`E1361`).

## What a slice publishes

Results carry, per slice: its **selection** (the lineage — the clauses it was
declared with, window included), the **streams** it matched, its **net**
per-period series, and `total` / `npv` / `irr` over the matched streams on
the model's own axis.

## No reconciliation, by design

A slice carries **no reconciliation block**. It is partial on purpose, and
must be seen to be: a slice never publishes a residual and never claims the
model's total, so a partial number cannot dress as a complete one. A
[statement](/docs/reference/statements) is the construct that reconciles —
and a statement filtered by a slice reconciles against the slice's total.

## A view, not a model change

A slice filters a completed result; it produces no cash. The compiler files
it under the document's `views`, which `model_hash` is taken over without —
so two users who look at identical results differently share a model hash,
and adding a slice moves neither `model_hash` nor `ledger_hash`. A declared
[metric](/docs/reference/metrics) is not a view: it is a figure the model
claims, so it does move `model_hash`.

## Related

- [Statements](/docs/reference/statements) — organizing results; a statement
  may filter by a slice.
- [Reading results and IR](/docs/guides/reading-results) — where `slices`
  sits in the document.
- [Results schema](/docs/specification/results-schema) — the exact shape.
