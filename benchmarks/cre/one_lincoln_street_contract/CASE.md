## The case

The same ground-up office development as the One Lincoln Street case beside
it: 36 storeys in Boston's Financial District, funded quarter by quarter across
the 2000–2003 build, with equity drawing first against a $110,738,000 commitment
and a construction facility taking the balance once it is exhausted.

What differs is how it is said. The other case builds the funding waterfall from
primitives. This one declares a single `cre.construction_loan` contract.

## The reference

MIT OpenCourseWare 11.431J Case Assignment 3, Exhibit 7 — the same source, the
same three published drivers: a sixteen-quarter draw schedule, an 8.00% rate
compounded quarterly, and the net equity commitment.

**Redistributable.** CC BY-NC-SA 4.0. The PDF is committed once, under the
native case's `reference/`, and both cases read the same figures from it.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Declared | one curve, one contract |
| Contracts | `cre.construction_loan` |
| Language features | a pack rule declaring a field, reading a model curve by name, and re-deriving an opening balance rather than carrying one |
| Conventions | equity-first funding, a facility drawing only once the commitment depletes, interest on the drawn balance with ratable draw timing |

The draw schedule stays a `curve` in the model rather than becoming a term. A
development's funding profile is per-deal data — sixteen published quarters
here, an S-curve or a contractor's schedule on the next deal — and all three are
the same object. What the contract adds is the funding convention.

The curve is stated ANNUALISED — each point is the exhibit's quarterly figure
times four — and the contract divides by periods-per-year, the same convention
every annual quantity in the pack follows. On this quarterly model that divides
straight back to the published number. It matters because a curve is a level: a
step curve returns its last point on every date, so a schedule stated as
per-period totals would be correct here and would fund three times the money if
the same deal were run monthly.

## The result

**Zero difference from the native case, in all 48 cells**, and `model.total`,
`model.npv` and `model.wal_years` agree to the last decimal. Against the exhibit
itself the residuals are therefore the native case's: equity contribution and
loan draw exact to the dollar in all sixteen quarters, interest within the
exhibit's own rounding.

The quarter that proves the contract is 2001.4, where the commitment runs out
mid-quarter and $29,430,000 of required funding splits into $10,522,000 of
equity and $18,908,000 of debt. That split is stated nowhere; it falls out of
the cumulative field crossing the commitment inside a period.

## The delta

**Nothing between the two cases.** Where they ever differ, the contract is
wrong: the language case is the reference, because it was validated against the
exhibit first and depends on no domain vocabulary.

**Against the exhibit**, the deltas are inherited and unchanged — the exhibit
rounds interest to whole thousands, and its stated debt-service total of
16,312,000 is the sum of those rounded quarterlies against the engine's
16,310,570 of exact ones.

**Interest is paid, not capitalised**, as the exhibit funds it from the equity
budget as a separate line. A capitalising facility compounds and is a different
recurrence; the contract does not model it.
