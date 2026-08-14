---
id: guide-statements
title: "Statements and reporting"
slug: "/docs/guides/statements"
generated: none
---

# Statements and reporting

How to get a pro forma out of a model, put your own lines on it, and read it at
the grain you report in.

For what the pieces are, see [Statements](/docs/reference/statements).

## Get a statement

Use a pack. A pack classifies the streams its contracts emit, and declares the
statement they roll into. Nothing else is required.

```cfdl
version 0.1
model "statement-walkthrough"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset tower : CRE.Asset.RealProperty

contract cre.lease {
  term 2026-01..2027-12
  terms {
    base_rent = 25000
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

## Read it annually

The CRE pack publishes a monthly statement and an annual one from the same run.
Nothing is re-modeled: the annual view regroups the same ledger, and its
columns are years rather than months.

Where you need a different grain than a pack offers, that is a pack change —
a statement declares the grain it reports at.

**A coverage ratio is not averaged.** Annual DSCR is annual NOI over annual debt
service, recomputed from the re-bucketed inputs. The mean of twelve monthly
ratios is a different number, and where the monthly denominator varies it is a
badly different one.

## Check the bottom line

Every statement publishes a reconciliation: its own total, the model's total,
and the residual between them.

```
reconciliation.residual = 0.0
```

A non-zero residual means the statement and the model disagree, and the usual
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
