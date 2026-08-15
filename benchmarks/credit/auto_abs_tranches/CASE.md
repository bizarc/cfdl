## The case

An auto-receivables trust issued seven classes of notes against a pool of car
loans. Principal collected on the loans repays the classes strictly in order of
seniority: nothing reaches the second class until the first is gone, nothing
reaches the third until the second is gone, and so on down to the most
subordinate. The question a noteholder asks is how fast their own class comes
back, and that depends entirely on where they sit in the queue.

## The reference

Exhibit 99.4 to the Form 8-K of Ally Auto Receivables Trust 2017-3, filed
17 September 2018. It states, for each of six classes and every monthly
distribution date, the percent of that class still outstanding.

`auto_abs_wal` reconciles the collateral in the same exhibit — 43 sub-pools
amortizing to an aggregate the issuer states to the cent — and stops there,
recording that the per-class columns need a sequential-pay waterfall. This case
is that axis, on the same 43 sub-pools unchanged.

Class principal amounts come from the trust's own Form 10-D. See `SOURCE.md`.

## What it exercises

One ordered waterfall paying six classes by seniority, over 48 distribution
dates.

A class is untouched until everything senior to it is repaid, then takes
principal until it is gone. One expression says that, and says it identically
before, during and after the class pays down:

```cfdl
pay a3_principal to party.a3_holders =
      min(remaining,
          min(max(C - inputs.a3_senior, 0.0), inputs.a3_original)
          - if(time.t == 0.0, 0.0,
               min(max(C_prev - inputs.a3_senior, 0.0), inputs.a3_original)))
```

`C` is cumulative pool principal. A retired class contributes zero without
being switched off, and declaration order is seniority.

The classes carry no balance of their own. A balance would be a second copy of
what the pool already knows, kept in step by nobody — the cascade reads the
cumulative principal the collateral produces and derives every class's position
from it.

Because the collateral was already reconciled, a break here can only be the
waterfall.

## The result

Worst disagreement **0.0054 percentage points**, across all six classes and 208
published cells.

| class | cells | worst |
|---|---:|---:|
| A-2 | 8 | 0.0047 |
| A-3 | 29 | 0.0046 |
| A-4 | 38 | 0.0027 |
| B | 41 | 0.0033 |
| C | 44 | 0.0054 |
| D | 48 | 0.0053 |

The exhibit rounds to 0.01, so 0.005 is the floor a reader can check against.
This sits on it — the same place `auto_abs_wal` lands on the collateral. Of the
204 dates where a class is outstanding, 201 agree within that floor: the model
reproduces the issuer's own printed number. Every class retires on exactly the
grid's date.

Three cells exceed the floor, by 0.0003–0.0005 percentage points — C at
04/15/22, D at 07/15/22 and 08/15/22. Net of rounding, the disagreement those
cells prove is at most $74 on the $537.6m pool.

## The delta

That residue is traceable to the reference's own inputs, not the waterfall.
The exhibit's pool table is exact where it can be — balances to the cent,
integer remaining terms — but prints each pool's APR to three decimals, while
the issuer ran on unrounded receivables data. Half of that last printed digit
(±0.0005%) is enough to move tail cumulative principal by up to $248,
concentrated in the seven large 51–53-month pools still amortizing in 2022;
the $74 the cells prove fits inside it several times over. The signature
agrees: the excess is small, one-signed, and appears only in the deal's final
four months, in the last two classes to pay — accumulated input drift
surfacing at the bottom of a sequential waterfall.

The engine's side of the ledger is clean. An independent month-by-month
recursion from the printed table reproduces the pack's closed-form output
exactly, and no payment-rounding convention (level payment to the cent, or
payment, interest and balance together) moves the aggregate by more than $10.
Nothing was tuned to close the residue: adjusting APRs within their printed
half-digit would fit the benchmark to its own reference, and the model already
sits at the information floor of the published data.

The exhibit's tables assume the receivables prepay at a constant ABS rate "with
no defaults, losses or repurchases". With no losses, overcollateralization never
has to build and no trigger can trip, so neither is modeled here. Interest is
collected and paid but never retires a note, and the published tables are about
principal, so it does not appear either.

This validates sequential pay and nothing about the loss-driven machinery. That
belongs to a deal that can lose money.

`model.total` is a regression anchor from this model, not an external figure.
Every external assertion is a per-period class column in `expected.csv`, derived
from the published grid.
