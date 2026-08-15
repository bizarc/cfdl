---
id: benchmark-credit-auto-abs-tranches
title: "Credit: auto ABS note classes"
slug: "/docs/examples/credit-auto-abs-tranches"
source: benchmarks/credit/auto_abs_tranches
---

# Credit: auto ABS note classes

The note classes of an auto ABS: one ordered waterfall paying six classes by seniority, reconciled against the issuer's published percent-outstanding grid at every distribution date.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

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

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.03}}
version 0.1
model "auto-abs-tranches"
use pack "credit" version "0.1.0"
time calendar monthly from 2018-10 for 64

// THE NOTE CLASSES OF AN AUTO ABS, against the issuer's published grid.
//
// `auto_abs_wal` reconciles this deal's COLLATERAL — 43 sub-pools amortizing
// to an aggregate the issuer states to the cent. It stops there, and says so:
// the per-class columns need a sequential-pay waterfall, which did not exist
// when it was written. This case is that third axis.
//
// The same exhibit publishes, for each of six note classes and every monthly
// distribution date, the percent of that class still outstanding. Because the
// collateral underneath is already right, anything that breaks here is the
// waterfall and nothing else.
//
// SEQUENTIAL PAY NEEDS NO STATE OF ITS OWN. A class is outstanding until
// everything senior to it has been retired, and a step can read what it has
// already been paid — so each step caps itself at its own remaining balance
// and passes the rest down. Declaration order IS seniority.
//
// The one-month gap between `time.t` and the window bound is the deal's own
// convention: receivables pay on the last day of the month and the notes pay
// on the 15th of the next.
//
// NO LOSSES ARE ASSUMED, by the exhibit's own terms — it states the receivables
// prepay at a constant ABS rate "with no defaults, losses or repurchases". So
// overcollateralization never has to build and no trigger can trip. This case
// checks sequential pay; the loss-driven machinery belongs to a deal that can
// lose money.

entity asset trust : Credit.Asset.LoanPool {
  collateral_type = "auto"
}

entity asset p01 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p02 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p03 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p04 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p06 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p07 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p08 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p09 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p11 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p12 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p13 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p14 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p16 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p17 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p18 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p19 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p21 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p22 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p23 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p24 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p26 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p27 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p28 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p29 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p31 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p32 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p33 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p34 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p36 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p37 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p38 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p39 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p40 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p41 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p42 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p43 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p44 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p45 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p46 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p47 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p48 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p49 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p50 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}

entity party servicer : Credit.Party.Servicer { name = "Servicer" }
entity party a2_holders : Credit.Party.Investor { name = "Class A-2 noteholders" }
entity party a3_holders : Credit.Party.Investor { name = "Class A-3 noteholders" }
entity party a4_holders : Credit.Party.Investor { name = "Class A-4 noteholders" }
entity party b_holders : Credit.Party.Investor { name = "Class B noteholders" }
entity party c_holders : Credit.Party.Investor { name = "Class C noteholders" }
entity party d_holders : Credit.Party.Investor { name = "Class D noteholders" }

contract credit.pool_level_pay.p01 on entity asset.p01 {
  term 2018-10..2020-03
  terms {
    balance = 5616021.32
    rate = 0.00000
    term_months = 18
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p02 on entity asset.p02 {
  term 2018-10..2021-01
  terms {
    balance = 2616054.82
    rate = 0.00000
    term_months = 28
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p03 on entity asset.p03 {
  term 2018-10..2022-06
  terms {
    balance = 4635948.89
    rate = 0.00000
    term_months = 45
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p04 on entity asset.p04 {
  term 2018-10..2022-12
  terms {
    balance = 2205909.75
    rate = 0.00000
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p06 on entity asset.p06 {
  term 2018-10..2019-11
  terms {
    balance = 147440.15
    rate = 0.00915
    term_months = 14
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p07 on entity asset.p07 {
  term 2018-10..2021-03
  terms {
    balance = 216238.15
    rate = 0.00992
    term_months = 30
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p08 on entity asset.p08 {
  term 2018-10..2022-07
  terms {
    balance = 354043.75
    rate = 0.00907
    term_months = 46
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p09 on entity asset.p09 {
  term 2018-10..2022-12
  terms {
    balance = 342126.24
    rate = 0.00905
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p11 on entity asset.p11 {
  term 2018-10..2020-02
  terms {
    balance = 610459.31
    rate = 0.01906
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p12 on entity asset.p12 {
  term 2018-10..2021-04
  terms {
    balance = 1144291.74
    rate = 0.01951
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p13 on entity asset.p13 {
  term 2018-10..2022-02
  terms {
    balance = 699535.89
    rate = 0.01949
    term_months = 41
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p14 on entity asset.p14 {
  term 2018-10..2022-12
  terms {
    balance = 201897.47
    rate = 0.01869
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p16 on entity asset.p16 {
  term 2018-10..2020-02
  terms {
    balance = 13918351.08
    rate = 0.02594
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p17 on entity asset.p17 {
  term 2018-10..2021-04
  terms {
    balance = 26181002.53
    rate = 0.02626
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p18 on entity asset.p18 {
  term 2018-10..2022-02
  terms {
    balance = 28740527.64
    rate = 0.02684
    term_months = 41
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p19 on entity asset.p19 {
  term 2018-10..2022-12
  terms {
    balance = 9735143.46
    rate = 0.02794
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p21 on entity asset.p21 {
  term 2018-10..2020-02
  terms {
    balance = 14533243.98
    rate = 0.03678
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p22 on entity asset.p22 {
  term 2018-10..2021-04
  terms {
    balance = 26195374.46
    rate = 0.03667
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p23 on entity asset.p23 {
  term 2018-10..2022-03
  terms {
    balance = 37348352.52
    rate = 0.03671
    term_months = 42
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p24 on entity asset.p24 {
  term 2018-10..2023-01
  terms {
    balance = 19509631.08
    rate = 0.03673
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p26 on entity asset.p26 {
  term 2018-10..2020-02
  terms {
    balance = 12183065.19
    rate = 0.04661
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p27 on entity asset.p27 {
  term 2018-10..2021-04
  terms {
    balance = 20323443.61
    rate = 0.04674
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p28 on entity asset.p28 {
  term 2018-10..2022-03
  terms {
    balance = 32071657.98
    rate = 0.04690
    term_months = 42
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p29 on entity asset.p29 {
  term 2018-10..2023-01
  terms {
    balance = 20332473.43
    rate = 0.04674
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p31 on entity asset.p31 {
  term 2018-10..2020-02
  terms {
    balance = 6428613.14
    rate = 0.05572
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p32 on entity asset.p32 {
  term 2018-10..2021-05
  terms {
    balance = 16325861.98
    rate = 0.05566
    term_months = 32
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p33 on entity asset.p33 {
  term 2018-10..2022-04
  terms {
    balance = 34020451.15
    rate = 0.05608
    term_months = 43
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p34 on entity asset.p34 {
  term 2018-10..2023-01
  terms {
    balance = 22175932.04
    rate = 0.05615
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p36 on entity asset.p36 {
  term 2018-10..2020-03
  terms {
    balance = 4214767.90
    rate = 0.06583
    term_months = 18
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p37 on entity asset.p37 {
  term 2018-10..2021-05
  terms {
    balance = 10197295.25
    rate = 0.06567
    term_months = 32
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p38 on entity asset.p38 {
  term 2018-10..2022-04
  terms {
    balance = 28511150.24
    rate = 0.06580
    term_months = 43
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p39 on entity asset.p39 {
  term 2018-10..2023-01
  terms {
    balance = 21518975.29
    rate = 0.06583
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p40 on entity asset.p40 {
  term 2018-10..2024-01
  terms {
    balance = 210992.57
    rate = 0.06671
    term_months = 64
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p41 on entity asset.p41 {
  term 2018-10..2020-02
  terms {
    balance = 2314366.62
    rate = 0.07537
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p42 on entity asset.p42 {
  term 2018-10..2021-04
  terms {
    balance = 6049009.56
    rate = 0.07527
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p43 on entity asset.p43 {
  term 2018-10..2022-04
  terms {
    balance = 17752272.88
    rate = 0.07538
    term_months = 43
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p44 on entity asset.p44 {
  term 2018-10..2023-02
  terms {
    balance = 17560641.20
    rate = 0.07526
    term_months = 53
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p45 on entity asset.p45 {
  term 2018-10..2024-01
  terms {
    balance = 133227.13
    rate = 0.07709
    term_months = 64
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p46 on entity asset.p46 {
  term 2018-10..2020-02
  terms {
    balance = 4089106.53
    rate = 0.09923
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p47 on entity asset.p47 {
  term 2018-10..2021-04
  terms {
    balance = 9761650.69
    rate = 0.09773
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p48 on entity asset.p48 {
  term 2018-10..2022-05
  terms {
    balance = 26285138.49
    rate = 0.09619
    term_months = 44
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p49 on entity asset.p49 {
  term 2018-10..2023-02
  terms {
    balance = 29949234.04
    rate = 0.09622
    term_months = 53
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p50 on entity asset.p50 {
  term 2018-10..2023-11
  terms {
    balance = 279866.82
    rate = 0.09836
    term_months = 62
    cpr = 0
    cdr = 0
  }
}

// Class principal amounts, from the trust's October 2018 Form 10-D.
assume a2_original = 112026644.00
assume a3_original = 271370000.00
assume a4_original = 86010000.00
assume b_original = 22220000.00
assume c_original = 18510000.00
assume d_original = 13750000.00

assume a2_senior   = 0.00
assume a3_senior   = 112026644.00
assume a4_senior   = 383396644.00
assume b_senior   = 469406644.00
assume c_senior   = 491626644.00
assume d_senior   = 510136644.00

assume a2_coupon   = 0.0153
assume a3_coupon   = 0.0174
assume a4_coupon   = 0.0201
assume b_coupon   = 0.0224
assume c_coupon   = 0.0237
assume d_coupon   = 0.0291

// ---------------------------------------------------------------------------
// The priority of payments
//
// Principal collected on the receivables, shared out by seniority. A class
// takes what it is still owed and the rest falls through; when a class is
// retired its step pays zero for the remainder of the deal, which is what
// "sequential" means.
// ---------------------------------------------------------------------------

waterfall notes.principal on entity asset.trust {
  schedule every month from 2018-10 to 2024-01

  // THE POT IS THE POOL'S PRINCIPAL. The exhibit tabulates principal
  // outstanding, so interest is beside the point: it is collected and paid,
  // but it never retires a note.
  from series_sum("credit.pool.sched_principal.*", time.t, time.t)
        + series_sum("credit.pool.prepay.*", time.t, time.t)

  // Class A-2. It is untouched until nothing is repaid, then takes
  // principal until it is gone. Retired classes contribute zero, so the same
  // expression describes the class before, during and after its pay-down.
  pay a2_principal to party.a2_holders =
        min(remaining,
            min(max((series_sum("credit.pool.sched_principal.*", 0, time.t)
                       + series_sum("credit.pool.prepay.*", 0, time.t))
                    - inputs.a2_senior, 0.0), inputs.a2_original)
            - if(time.t == 0.0, 0.0,
                 min(max((series_sum("credit.pool.sched_principal.*", 0, time.t - 1)
                       + series_sum("credit.pool.prepay.*", 0, time.t - 1))
                         - inputs.a2_senior, 0.0), inputs.a2_original)))

  // Class A-3. It is untouched until the 112,026,644 senior to it is repaid, then takes
  // principal until it is gone. Retired classes contribute zero, so the same
  // expression describes the class before, during and after its pay-down.
  pay a3_principal to party.a3_holders =
        min(remaining,
            min(max((series_sum("credit.pool.sched_principal.*", 0, time.t)
                       + series_sum("credit.pool.prepay.*", 0, time.t))
                    - inputs.a3_senior, 0.0), inputs.a3_original)
            - if(time.t == 0.0, 0.0,
                 min(max((series_sum("credit.pool.sched_principal.*", 0, time.t - 1)
                       + series_sum("credit.pool.prepay.*", 0, time.t - 1))
                         - inputs.a3_senior, 0.0), inputs.a3_original)))

  // Class A-4. It is untouched until the 383,396,644 senior to it is repaid, then takes
  // principal until it is gone. Retired classes contribute zero, so the same
  // expression describes the class before, during and after its pay-down.
  pay a4_principal to party.a4_holders =
        min(remaining,
            min(max((series_sum("credit.pool.sched_principal.*", 0, time.t)
                       + series_sum("credit.pool.prepay.*", 0, time.t))
                    - inputs.a4_senior, 0.0), inputs.a4_original)
            - if(time.t == 0.0, 0.0,
                 min(max((series_sum("credit.pool.sched_principal.*", 0, time.t - 1)
                       + series_sum("credit.pool.prepay.*", 0, time.t - 1))
                         - inputs.a4_senior, 0.0), inputs.a4_original)))

  // Class B. It is untouched until the 469,406,644 senior to it is repaid, then takes
  // principal until it is gone. Retired classes contribute zero, so the same
  // expression describes the class before, during and after its pay-down.
  pay b_principal to party.b_holders =
        min(remaining,
            min(max((series_sum("credit.pool.sched_principal.*", 0, time.t)
                       + series_sum("credit.pool.prepay.*", 0, time.t))
                    - inputs.b_senior, 0.0), inputs.b_original)
            - if(time.t == 0.0, 0.0,
                 min(max((series_sum("credit.pool.sched_principal.*", 0, time.t - 1)
                       + series_sum("credit.pool.prepay.*", 0, time.t - 1))
                         - inputs.b_senior, 0.0), inputs.b_original)))

  // Class C. It is untouched until the 491,626,644 senior to it is repaid, then takes
  // principal until it is gone. Retired classes contribute zero, so the same
  // expression describes the class before, during and after its pay-down.
  pay c_principal to party.c_holders =
        min(remaining,
            min(max((series_sum("credit.pool.sched_principal.*", 0, time.t)
                       + series_sum("credit.pool.prepay.*", 0, time.t))
                    - inputs.c_senior, 0.0), inputs.c_original)
            - if(time.t == 0.0, 0.0,
                 min(max((series_sum("credit.pool.sched_principal.*", 0, time.t - 1)
                       + series_sum("credit.pool.prepay.*", 0, time.t - 1))
                         - inputs.c_senior, 0.0), inputs.c_original)))

  // Class D. It is untouched until the 510,136,644 senior to it is repaid, then takes
  // principal until it is gone. Retired classes contribute zero, so the same
  // expression describes the class before, during and after its pay-down.
  pay d_principal to party.d_holders =
        min(remaining,
            min(max((series_sum("credit.pool.sched_principal.*", 0, time.t)
                       + series_sum("credit.pool.prepay.*", 0, time.t))
                    - inputs.d_senior, 0.0), inputs.d_original)
            - if(time.t == 0.0, 0.0,
                 min(max((series_sum("credit.pool.sched_principal.*", 0, time.t - 1)
                       + series_sum("credit.pool.prepay.*", 0, time.t - 1))
                         - inputs.d_senior, 0.0), inputs.d_original)))
}
```

## Run configuration

```json
{"deterministic":{"annual_discount_rate":0.03}}
```

## Verified results

Checked period by period: **6 series** across **48 periods** — **208 values** in all, each within the tolerance shown.

- `notes.principal.a2_principal` — within ±11202.66
- `notes.principal.a3_principal` — within ±27137.0
- `notes.principal.a4_principal` — within ±8601.0
- `notes.principal.b_principal` — within ±2222.0
- `notes.principal.c_principal` — within ±1851.0
- `notes.principal.d_principal` — within ±1375.0

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.total` | 589,606,387.86 | ±1 |
