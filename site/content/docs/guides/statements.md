---
id: guide-statements
title: "Statements and reporting"
slug: "/docs/guides/statements"
description: "Get a pro forma out of a model, put your own lines on it, and read it at the grain you report in."
generated: none
---

# Statements and reporting

How to get a pro forma out of a model, put your own lines on it, and read it at
the grain you report in.

For what the pieces are, see [Statements](/docs/reference/statements).

## Get a statement

There are three ways to get one: declare a `statement` in the model, use a
pack that ships one, or take the default — a model that declares none renders
its entity hierarchy as a statement marked `default`, so results always read
as the model's shape rather than a flat list of series.

The quickest is a pack. A pack classifies the streams its contracts emit, and
declares the statement they roll into. Nothing else is required.

```cfdl
version 0.1
model "statement-walkthrough"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset tower : CRE.Asset.RealProperty

contract cre.lease {
  term 2026-01..2027-12
  terms {
    rent = 25000
  }
}
```

Run it with the `cre` pack and the results carry a `statements` section. In the
playground it is the **Statement** tab:

```
Base rental revenue          600,000
Effective gross income       600,000
Net operating income         600,000
```

## Put a hand-written stream on it

A stream that carries no category is not on the statement — it appears in an
*Unclassified* row instead, so cash is never quietly lost. Give it a
`category` and it joins the right line and every subtotal above it:

```cfdl
stream cre.parking on entity asset.tower inflow currency USD {
  schedule every month from 2026-01 to 2027-12
  category operating.revenue.other
  amount = 3000
}
```

Now:

```
Base rental revenue          600,000
Other operating income        72,000
Effective gross income       672,000
Net operating income         672,000
```

The stream was classified once, and it is both a visible line and part of the
subtotal. There is no way to have one without the other.

Use the categories the pack declares — see [pack contracts](/docs/reference/packs).
Without a pack, any path rooted in `operating`, `investing` or `financing` is
valid.

## Declare your own

A model declares a statement the same way a pack does. The generated form
names a hierarchy the results already carry — the `part of` tree or the
dotted category path — and a depth to cut it at:

```cfdl
statement portfolio {
  label     "Portfolio by property"
  structure entity
  depth     2
}
```

The rows follow from the tree: a node whose children are shown is a subtotal,
a node whose children are cut off by `depth` is a line carrying all of its
descendants' cash. That single rule keeps the bottom line reconciling at
every depth.

Where the tree is not the presentation — curated labels, expenses shown
positive under "Less:", a coverage ratio at the bottom — a statement states
its own rows instead:

```cfdl
slice noi          { category "operating.*" }
slice debt_service { category "financing.debt.service" }

statement operating {
  label "Operating statement"
  line     "Base rental revenue"   { category "operating.revenue.base_rent" }
  line     "Less: operating costs" { category "operating.expense.*" display positive }
  subtotal "Net operating income"  { category "operating.*" }
  spacer
  ratio    "DSCR"                  { of noi to debt_service display positive }
}
```

A row may instead draw a **published series** — a fold the run already
computed, such as a pack subtotal or the model's own net:

```cfdl
  subtotal "Net cash flow (memo)" { series "model.net_cash_flow" }
```

A series row presents that fold beside the claimed rows and claims nothing
itself: its figure stays out of the bottom line, so a memo of `domain.cre.noi`
never double-counts the cash it summarizes. A claim clause beside a `series`
is refused (`E1370`), and a key the run does not publish renders no values
rather than a column of zeros.

A statement is authored or generated, never both (`E1369`). A generated
statement may also carry a `slice` — an orthogonal filter, so any structure
can be shown for any selection — and `metrics`, naming declared metrics to
publish beside it. See [Statements](/docs/reference/statements) for the
rules.

## Read it annually

The CRE pack publishes a monthly statement and an annual one from the same run.
Nothing is re-modeled: the annual view regroups the same ledger, and its
columns are years rather than months.

Where you need a different grain than a pack offers, declare it: a model
statement carries a `grain` clause and reports at it. Values re-bucket into
the coarser periods; the total stays the lifetime figure; a ratio is
recomputed from the re-bucketed inputs.

```cfdl
statement portfolio {
  structure entity
  grain     annual
}
```

**A coverage ratio is not averaged.** Annual DSCR is annual NOI over annual debt
service, recomputed from the re-bucketed inputs. The mean of twelve monthly
ratios is a different number, and where the monthly denominator varies it is a
badly different one.

## Check the bottom line

Every statement publishes a reconciliation: its own total, the total of the
universe it reports, and the residual between them. For most statements that
universe is the model; a statement filtered by a `slice` reconciles against
the slice's total instead, because reporting the filter itself as a shortfall
would make a warning fire on a correct model.

```
reconciliation.residual = 0.0
```

A non-zero residual means the statement and its universe disagree, and the usual
cause is a stream carrying no category — which is why the *Unclassified* row
exists rather than the cash silently vanishing. A statement that is short by an
unnoticed line looks entirely plausible, so this is published always, whether it
is zero or not.

## Get the numbers out

The statement is part of the results document, so anything that reads results
reads it — the [Python SDK](/docs/python-sdk), the
[API server](/docs/api-server), or the JSON directly.

Per-period subtotals are also published as ordinary series under
`domain.<pack>.<name>`, so a chart or an assertion can read `domain.cre.noi`
without going through the statement at all. That is how the benchmark suite
asserts published NOI and coverage figures period by period.

## Related

- [Statements reference](/docs/reference/statements) — the pieces and the rules
- [Domain packs](/docs/packs) — what each pack classifies and rolls up
- [Reading results](/docs/guides/reading-results) — the rest of the document
