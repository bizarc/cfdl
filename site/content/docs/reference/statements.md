---
id: reference-statements
title: "Statements"
slug: "/docs/reference/statements"
description: "How a statement is produced — an operating pro forma, a remittance report, a cash flow build-up — and how to put your own lines on one."
generated: regions
---

# Statements

A **statement** is the artifact a practitioner recognizes: an operating pro
forma, a remittance report, a free cash flow build-up. CFDL produces one per
period from the model you already wrote — you do not build it, and you do not
maintain it alongside the model.

Run a model with a pack and the results carry a `statements` section. In the
playground it is the **Statement** tab.

## What makes it work

Three things, and only the first is something you write.

**A category** says what a stream *is*, economically. It is a dotted path whose
first segment is `operating`, `investing` or `financing` — the sections of a
statement of cash flows. A pack's contracts classify the streams they emit, so
in most models you never write one. A hand-written stream can declare its own:

```
stream tower.parking on entity asset.tower inflow currency USD {
  schedule every month from 2026-01 to 2035-12
  category operating.revenue.other
  amount = 4500
}
```

**A subtotal** aggregates categories, every period — net operating income is
everything classified `operating.*`. Packs declare these; you read them as
`domain.<pack>.<name>` in the results.

**A statement** puts them in order with labels and indentation.

## Reading one

**Every line is signed as it is stored.** An expense is negative. A row that a
reader expects to see positive — "Less: vacancy", debt service — carries a
display sign that flips it *for rendering only*. The published value stays
signed, so anything consuming the JSON still adds up correctly while a rendered
statement reads the way a pro forma should.

**A ratio is recomputed at the grain it is shown at.** An annual coverage ratio
is annual NOI over annual debt service — never the average of twelve monthly
ratios, which is a different and wrong number. Where the denominator is zero
the value is `null` rather than zero: a period with no debt service has no
coverage ratio, and averaging a zero in would understate the deal.

**The bottom line is checked, not assumed.** Every statement publishes a
`reconciliation` — its own total, the model's total, and the residual between
them. If a stream carries no category it appears in a visible *Unclassified*
row rather than quietly vanishing, because a statement that is short by an
unnoticed line looks entirely plausible.

## Grain

Grain belongs to the output, not the run. One model can publish a monthly pro
forma and an annual summary of the same cash, and both are in the same results
document. Each statement carries its own column labels, because an annual view
of a monthly model has ten columns where the model has 120.

## Several views of one domain

A pack can ship more than one layout over the same categories, because one
asset class has more than one reporting convention. The same loan pool reads as
a remittance report — principal split scheduled and unscheduled, because that is
what a prepayment speed acts on — or as a statement of operations reporting
total and net investment income.

Each view is checked for completeness when the pack loads: every category the
pack declares appears in exactly one line row. A view cannot quietly omit or
double-count while looking plausible.

## What each pack ships

<!-- cfdl:generated pack-statements -->
| Pack | Statement | Reported at |
|---|---|---|
| `energy` | Project operating statement *(default)* | model grid |
| `energy` | Project operating statement (annual) | annual |
| `cre` | Operating statement *(default)* | model grid |
| `cre` | Operating statement (annual) | annual |
| `credit` | Collections statement *(default)* | model grid |
| `credit` | Collections statement (annual) | annual |
| `credit` | Remittance report | model grid |
| `credit` | Statement of operations | model grid |
| `opco` | Free cash flow *(default)* | model grid |
| `opco` | Sponsor cash flow | model grid |
| `opco` | Statement of cash flows | model grid |
<!-- /cfdl:generated pack-statements -->

## Related

- [Domain packs](/docs/packs) — the categories and contracts each pack provides.
- [Reading results and IR](/docs/guides/reading-results) — the shape of the
  results document.
- [Pack interface](/docs/specification/pack-interface) — declaring subtotals and
  statements, for pack authors.
