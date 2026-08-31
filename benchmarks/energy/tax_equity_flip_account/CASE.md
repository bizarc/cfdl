## The case

The twin of `tax_equity_flip`: the same leveraged partnership flip, against the
same external anchors, with the project's cash rebuilt as streams settling into
an account instead of a hand-carried field.

The original says what it is waiting for, in a comment on its own waterfall:
"the project carries it as a field the deal itself tracks. Rehoming it as
streams would move the case's asserted figures, so it stays until the case is
rebuilt." This is that rebuild, carried as a twin so the claim can be checked
rather than asserted.

## The reference

The same one the original uses — the national laboratory's open-source
project-finance model in its leveraged-partnership-flip configuration, run once
outside this repo. See `../tax_equity_flip/NOTES.md` for the version, inputs and
command. The anchors in `expected.csv` are that model's outputs, unchanged.

## What it exercises

The distribution pot as an **account** rather than a field. The plant's revenue,
O&M and debt service are three streams under one name family; the account draws
them with a single glob; the distribution takes the whole balance each period,
so the account returns to zero and carries nothing forward.

The flip test moves with the pot. `return_position` previously read
`prev.asset.project.cash` — the field. It now reads
`series_sum("energy.project.*", time.t - 1, time.t - 1)`: the same quantity,
taken from settled cash strictly backward.

What that buys is visibility. In the original the project's cash exists only
inside an entity, where the distribution can see it and a reviewer cannot. Here
it is in the ledger, published per period, and the figures that decide the flip
are the figures the statement shows.

## The result

Both partners' columns reconcile against the same anchors as the original, at
the same one-cent tolerance, across all 25 operating periods.

Against the ORIGINAL's own output the agreement is tighter still: 50 of 50
cells within tolerance, largest absolute difference 0.0047 dollars on figures
of about four million — one part in 10^9.

## The delta

Where this case is looser than its twin, and why.

The original reconciles to 1.0e-6 dollars against the reference. This one
reconciles to 4.7e-3. That is a reassociation difference, not a modeling one:
the same quantities are summed in a different order — through the ledger and an
account balance, rather than inside one field expression — and floating-point
addition is not associative. Both are far inside the case's one-cent tolerance,
and neither is more correct than the other about the deal.

It is recorded because a twin exists to make a substitution checkable, and
"the numbers moved by 5 milli-dollars" is part of what the check found.

## A variant the reference does not publish

None. The original carries that section; this twin exists to check a
substitution, not to extend the case.
