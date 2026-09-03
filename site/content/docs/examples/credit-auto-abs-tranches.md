---
id: benchmark-credit-auto-abs-tranches
title: "Credit: auto ABS note classes"
slug: "/docs/examples/credit-auto-abs-tranches"
description: "The note classes of an auto ABS: the trust as a container, collections as accounts, and ordered waterfalls paying seven classes by seniority, reconciled against the issuer's published percent-outstanding grid at every distribution date."
source: benchmarks/credit/auto_abs_tranches
---

# Credit: auto ABS note classes

The note classes of an auto ABS: the trust as a container, collections as accounts, and ordered waterfalls paying seven classes by seniority, reconciled against the issuer's published percent-outstanding grid at every distribution date.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

An auto-receivables trust issued seven classes of notes against a pool of car
loans. Each month the borrowers pay interest and principal; the trust collects
it, pays the servicer and the administrator, pays interest on each class at its
coupon, and repays note principal strictly in order of seniority: nothing
reaches the second class until the first is gone, nothing reaches the third
until the second is gone, and so on down to the most subordinate. The question
a noteholder asks is how fast their own class comes back, and that depends
entirely on where they sit in the queue.

## The reference

Exhibit 99.4 to the Form 8-K of Ally Auto Receivables Trust 2017-3, filed
17 September 2018. It states, for each of six classes and every monthly
distribution date, the percent of that class still outstanding, and the
assumptions the tables rest on: a 1.00% servicing fee, a $1,500 monthly
administration fee, each class's coupon and day count, and no defaults, losses
or repurchases.

`auto_abs_wal` reconciles the collateral in the same exhibit — 43 sub-pools
amortizing to an aggregate the issuer states to the cent — and stops there,
recording that the per-class columns need a priority of payments. This case is
that axis, on the same 43 sub-pools unchanged.

Class principal amounts come from the trust's own Form 10-D. See `SOURCE.md`.

## What it exercises

The trust as a container, the notes as claims on its cash, and the priority of
payments as two ordered allocations from the two amounts the indenture defines.

Interest collected, net of the servicer's fee and the trust's own expense, is
one account; principal collected is another. Each distribution date allocates
the first to the classes' coupons and the second to their principal, by
seniority. Every holder owns an account that receives its principal, and that
account IS the class's position: what a class is still owed is its face less
what its holder has been paid, so a step reads the account the previous
distributions filled and states its claim in one line:

```cfdl
pay a3_principal to party.a3_holders =
      min(remaining, inputs.a3_face - if(time.t == 0.0, 0.0, prev.a3_principal))
```

Declaration order is seniority. A retired class contributes zero because its
claim is zero, without being switched off. Nothing restates the waterfall and
no class carries a balance of its own.

The published grid is therefore asserted directly, not by differencing: each
class's percent outstanding is its face less its account, and the account
balances sit in `expected.csv` beside the payments they explain.

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
208 cells, 205 agree within that floor: the model reproduces the issuer's own
printed number. Every class retires on exactly the grid's date.

Three cells exceed the floor, by 0.0003–0.0005 percentage points — C at
04/15/22, D at 07/15/22 and 08/15/22. Net of rounding, the disagreement those
cells prove is at most $74 on the $537.6m pool.

Interest is paid in full on every class at every distribution: the coupons
never exhaust the interest collected, which is what a deal with no losses
should show. The interest the trust collects beyond the coupons, and the
$13.75m by which the pool exceeds the notes, accumulate in the trust's own
accounts and are reported as their balances.

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
has to build and no trigger can trip, so neither is modeled here. The clean-up
call is not exercised: the tables asserted are the to-maturity columns.

This validates the priority of payments and nothing about the loss-driven
machinery. That belongs to a deal that can lose money.

`model.total` is a regression anchor from this model, not an external figure.
Every external assertion is a per-period class column in `expected.csv`,
derived from the published grid.

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
// the per-class columns need a sequential-pay waterfall. This case is that
// axis: the same 43 sub-pools, one priority of payments, seven note classes.
//
// THE TRUST IS A CONTAINER, AND THE NOTES ARE CLAIMS ON ITS CASH. Each month
// the receivables pay interest and principal; the trust's fees come out of
// those collections; interest is paid on each
// class at its coupon; and principal repays the classes strictly in order of
// seniority. Interest collected and principal collected are the two amounts the
// indenture defines, each an account the trust holds, and each distribution
// date allocates them by the priority of payments.
//
// A CLASS'S POSITION IS ITS HOLDER'S ACCOUNT. What a class has been repaid is
// the principal allocated to its holder so far, so what it is still owed is
// its face less that account. No class carries a balance of its own, and
// nothing restates the waterfall: each step reads the account the previous
// distributions filled. Declaration order IS seniority, and a retired class
// contributes zero because its claim is zero.
//
// The one-month gap between collection and distribution is the deal's own
// convention: receivables pay on the last day of the month and the notes pay
// on the 15th of the next.
//
// NO LOSSES ARE ASSUMED, by the exhibit's own terms — it states the receivables
// prepay at a constant ABS rate "with no defaults, losses or repurchases". So
// overcollateralization never has to build and no trigger can trip. The $13.75m
// by which the pool exceeds the notes, and interest collected beyond interest
// paid, accumulate in the trust's own accounts.

entity container trust : Container.SPV

entity asset p01 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p02 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p03 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p04 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p06 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p07 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p08 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p09 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p11 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p12 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p13 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p14 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p16 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p17 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p18 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p19 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p21 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p22 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p23 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p24 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p26 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p27 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p28 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p29 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p31 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p32 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p33 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p34 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p36 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p37 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p38 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p39 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p40 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p41 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p42 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p43 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p44 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p45 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p46 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p47 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p48 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p49 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}
entity asset p50 : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of container.trust
}

entity party servicer : Credit.Party.Servicer { name = "Servicer" }
entity party a1_holders : Credit.Party.Investor { name = "Class A-1 noteholders" }
entity party a2_holders : Credit.Party.Investor { name = "Class A-2 noteholders" }
entity party a3_holders : Credit.Party.Investor { name = "Class A-3 noteholders" }
entity party a4_holders : Credit.Party.Investor { name = "Class A-4 noteholders" }
entity party b_holders : Credit.Party.Investor { name = "Class B noteholders" }
entity party c_holders : Credit.Party.Investor { name = "Class C noteholders" }
entity party d_holders : Credit.Party.Investor { name = "Class D noteholders" }

contract credit.pool_level_pay.p01 on entity asset.p01 {
  term 2018-10..2020-03
  terms {
    principal = 5616021.32
    interest_rate = 0.00000
    term_months = 18
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p02 on entity asset.p02 {
  term 2018-10..2021-01
  terms {
    principal = 2616054.82
    interest_rate = 0.00000
    term_months = 28
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p03 on entity asset.p03 {
  term 2018-10..2022-06
  terms {
    principal = 4635948.89
    interest_rate = 0.00000
    term_months = 45
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p04 on entity asset.p04 {
  term 2018-10..2022-12
  terms {
    principal = 2205909.75
    interest_rate = 0.00000
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p06 on entity asset.p06 {
  term 2018-10..2019-11
  terms {
    principal = 147440.15
    interest_rate = 0.00915
    term_months = 14
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p07 on entity asset.p07 {
  term 2018-10..2021-03
  terms {
    principal = 216238.15
    interest_rate = 0.00992
    term_months = 30
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p08 on entity asset.p08 {
  term 2018-10..2022-07
  terms {
    principal = 354043.75
    interest_rate = 0.00907
    term_months = 46
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p09 on entity asset.p09 {
  term 2018-10..2022-12
  terms {
    principal = 342126.24
    interest_rate = 0.00905
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p11 on entity asset.p11 {
  term 2018-10..2020-02
  terms {
    principal = 610459.31
    interest_rate = 0.01906
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p12 on entity asset.p12 {
  term 2018-10..2021-04
  terms {
    principal = 1144291.74
    interest_rate = 0.01951
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p13 on entity asset.p13 {
  term 2018-10..2022-02
  terms {
    principal = 699535.89
    interest_rate = 0.01949
    term_months = 41
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p14 on entity asset.p14 {
  term 2018-10..2022-12
  terms {
    principal = 201897.47
    interest_rate = 0.01869
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p16 on entity asset.p16 {
  term 2018-10..2020-02
  terms {
    principal = 13918351.08
    interest_rate = 0.02594
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p17 on entity asset.p17 {
  term 2018-10..2021-04
  terms {
    principal = 26181002.53
    interest_rate = 0.02626
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p18 on entity asset.p18 {
  term 2018-10..2022-02
  terms {
    principal = 28740527.64
    interest_rate = 0.02684
    term_months = 41
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p19 on entity asset.p19 {
  term 2018-10..2022-12
  terms {
    principal = 9735143.46
    interest_rate = 0.02794
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p21 on entity asset.p21 {
  term 2018-10..2020-02
  terms {
    principal = 14533243.98
    interest_rate = 0.03678
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p22 on entity asset.p22 {
  term 2018-10..2021-04
  terms {
    principal = 26195374.46
    interest_rate = 0.03667
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p23 on entity asset.p23 {
  term 2018-10..2022-03
  terms {
    principal = 37348352.52
    interest_rate = 0.03671
    term_months = 42
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p24 on entity asset.p24 {
  term 2018-10..2023-01
  terms {
    principal = 19509631.08
    interest_rate = 0.03673
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p26 on entity asset.p26 {
  term 2018-10..2020-02
  terms {
    principal = 12183065.19
    interest_rate = 0.04661
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p27 on entity asset.p27 {
  term 2018-10..2021-04
  terms {
    principal = 20323443.61
    interest_rate = 0.04674
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p28 on entity asset.p28 {
  term 2018-10..2022-03
  terms {
    principal = 32071657.98
    interest_rate = 0.04690
    term_months = 42
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p29 on entity asset.p29 {
  term 2018-10..2023-01
  terms {
    principal = 20332473.43
    interest_rate = 0.04674
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p31 on entity asset.p31 {
  term 2018-10..2020-02
  terms {
    principal = 6428613.14
    interest_rate = 0.05572
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p32 on entity asset.p32 {
  term 2018-10..2021-05
  terms {
    principal = 16325861.98
    interest_rate = 0.05566
    term_months = 32
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p33 on entity asset.p33 {
  term 2018-10..2022-04
  terms {
    principal = 34020451.15
    interest_rate = 0.05608
    term_months = 43
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p34 on entity asset.p34 {
  term 2018-10..2023-01
  terms {
    principal = 22175932.04
    interest_rate = 0.05615
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p36 on entity asset.p36 {
  term 2018-10..2020-03
  terms {
    principal = 4214767.90
    interest_rate = 0.06583
    term_months = 18
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p37 on entity asset.p37 {
  term 2018-10..2021-05
  terms {
    principal = 10197295.25
    interest_rate = 0.06567
    term_months = 32
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p38 on entity asset.p38 {
  term 2018-10..2022-04
  terms {
    principal = 28511150.24
    interest_rate = 0.06580
    term_months = 43
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p39 on entity asset.p39 {
  term 2018-10..2023-01
  terms {
    principal = 21518975.29
    interest_rate = 0.06583
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p40 on entity asset.p40 {
  term 2018-10..2024-01
  terms {
    principal = 210992.57
    interest_rate = 0.06671
    term_months = 64
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p41 on entity asset.p41 {
  term 2018-10..2020-02
  terms {
    principal = 2314366.62
    interest_rate = 0.07537
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p42 on entity asset.p42 {
  term 2018-10..2021-04
  terms {
    principal = 6049009.56
    interest_rate = 0.07527
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p43 on entity asset.p43 {
  term 2018-10..2022-04
  terms {
    principal = 17752272.88
    interest_rate = 0.07538
    term_months = 43
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p44 on entity asset.p44 {
  term 2018-10..2023-02
  terms {
    principal = 17560641.20
    interest_rate = 0.07526
    term_months = 53
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p45 on entity asset.p45 {
  term 2018-10..2024-01
  terms {
    principal = 133227.13
    interest_rate = 0.07709
    term_months = 64
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p46 on entity asset.p46 {
  term 2018-10..2020-02
  terms {
    principal = 4089106.53
    interest_rate = 0.09923
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p47 on entity asset.p47 {
  term 2018-10..2021-04
  terms {
    principal = 9761650.69
    interest_rate = 0.09773
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p48 on entity asset.p48 {
  term 2018-10..2022-05
  terms {
    principal = 26285138.49
    interest_rate = 0.09619
    term_months = 44
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p49 on entity asset.p49 {
  term 2018-10..2023-02
  terms {
    principal = 29949234.04
    interest_rate = 0.09622
    term_months = 53
    cpr = 0
    cdr = 0
  }
}

contract credit.pool_level_pay.p50 on entity asset.p50 {
  term 2018-10..2023-11
  terms {
    principal = 279866.82
    interest_rate = 0.09836
    term_months = 62
    cpr = 0
    cdr = 0
  }
}

// ---------------------------------------------------------------------------
// The notes. Faces are each class's balance at the exhibit's cut-off, which is
// the base its percent-outstanding grid is stated on: A-1 was paid in full in
// January 2018 and is carried at zero; A-2 had amortized to 112,026,644 (the
// trust's Form 10-D); the rest stood at their original principal. Coupons and
// the 30/360 day count are the exhibit's.
// ---------------------------------------------------------------------------
assume a1_face   = 0.00
assume a2_face   = 112026644.00
assume a3_face   = 271370000.00
assume a4_face   = 86010000.00
assume b_face   = 22220000.00
assume c_face   = 18510000.00
assume d_face   = 13750000.00

assume a1_coupon = 0.0110
assume a2_coupon = 0.0153
assume a3_coupon = 0.0174
assume a4_coupon = 0.0201
assume b_coupon = 0.0224
assume c_coupon = 0.0237
assume d_coupon = 0.0291

// ---------------------------------------------------------------------------
// The trust's expenses, one item each. The servicing fee is 1.00% per annum
// on the pool balance the trust carried into the month — the initial pool
// less the principal collected so far — and the administration fee is $1,500
// a month. Neither the servicer nor the administrator is modeled as a payee:
// the fees leave the trust's cash before anything reaches the notes, which is
// all the published grid depends on.
// ---------------------------------------------------------------------------
assume initial_pool = 537640787.96

stream credit.trust.servicing_fee on entity container.trust outflow currency USD {
  schedule every month from 2018-10 to 2024-01
  category operating.expense.servicing
  amount = 0.01 / 12.0 * (inputs.initial_pool
             - if(time.t == 0.0, 0.0,
                  series_sum("credit.pool.sched_principal.*", 0, time.t - 1)
                  + series_sum("credit.pool.prepay.*", 0, time.t - 1)))
}

stream credit.trust.admin_fee on entity container.trust outflow currency USD {
  schedule every month from 2018-10 to 2024-01
  category operating.expense.servicing
  amount = 1500.0
}

// ---------------------------------------------------------------------------
// The accounts. The indenture defines two amounts on each distribution date,
// and each is a location cash sits in: AVAILABLE INTEREST — the interest
// collected, net of the trust's fees — and the
// PRINCIPAL DISTRIBUTABLE AMOUNT — the principal collected. Each holder owns an
// account that receives its principal, which IS the class's position, and a
// separate interest account beside it, so what has been repaid and what has
// been earned are never mixed.
// ---------------------------------------------------------------------------
account interest_collections {
  from series_sum("credit.pool.interest.*", time.t, time.t)
     + series_sum("credit.trust.servicing_fee", time.t, time.t)
     + series_sum("credit.trust.admin_fee", time.t, time.t)
}

account principal_collections {
  from series_sum("credit.pool.sched_principal.*", time.t, time.t)
     + series_sum("credit.pool.prepay.*", time.t, time.t)
}

account a1_principal { owner party.a1_holders }
account a2_principal { owner party.a2_holders }
account a3_principal { owner party.a3_holders }
account a4_principal { owner party.a4_holders }
account b_principal { owner party.b_holders }
account c_principal { owner party.c_holders }
account d_principal { owner party.d_holders }

account a1_interest { from 0.0 }
account a2_interest { from 0.0 }
account a3_interest { from 0.0 }
account a4_interest { from 0.0 }
account b_interest { from 0.0 }
account c_interest { from 0.0 }
account d_interest { from 0.0 }

// ---------------------------------------------------------------------------
// Interest, on each distribution date: every class at its coupon on the
// balance it carried in — its face less the principal its holder's account
// already holds. Interest collected beyond the coupons stays in the trust's
// interest account.
// ---------------------------------------------------------------------------
waterfall notes.interest on entity container.trust {
  schedule every month from 2018-10 to 2024-01
  from interest_collections

  pay a1_interest to account a1_interest =
        min(remaining, (inputs.a1_face - if(time.t == 0.0, 0.0, prev.a1_principal)) * inputs.a1_coupon / 12.0)
  pay a2_interest to account a2_interest =
        min(remaining, (inputs.a2_face - if(time.t == 0.0, 0.0, prev.a2_principal)) * inputs.a2_coupon / 12.0)
  pay a3_interest to account a3_interest =
        min(remaining, (inputs.a3_face - if(time.t == 0.0, 0.0, prev.a3_principal)) * inputs.a3_coupon / 12.0)
  pay a4_interest to account a4_interest =
        min(remaining, (inputs.a4_face - if(time.t == 0.0, 0.0, prev.a4_principal)) * inputs.a4_coupon / 12.0)
  pay b_interest to account b_interest =
        min(remaining, (inputs.b_face - if(time.t == 0.0, 0.0, prev.b_principal)) * inputs.b_coupon / 12.0)
  pay c_interest to account c_interest =
        min(remaining, (inputs.c_face - if(time.t == 0.0, 0.0, prev.c_principal)) * inputs.c_coupon / 12.0)
  pay d_interest to account d_interest =
        min(remaining, (inputs.d_face - if(time.t == 0.0, 0.0, prev.d_principal)) * inputs.d_coupon / 12.0)
}

// ---------------------------------------------------------------------------
// Principal, strictly by seniority. A step states what its class is still
// owed and the engine pays it out of what remains, so nothing reaches a class
// until every class above it is gone. At the first distribution no account
// has a prior balance, so the claim is the face. The $13.75m by which the pool
// exceeds the notes stays in the principal account as the trust's own cash.
// ---------------------------------------------------------------------------
waterfall notes.principal on entity container.trust {
  schedule every month from 2018-10 to 2024-01
  from principal_collections

  pay a1_principal to party.a1_holders =
        min(remaining, inputs.a1_face - if(time.t == 0.0, 0.0, prev.a1_principal))
  pay a2_principal to party.a2_holders =
        min(remaining, inputs.a2_face - if(time.t == 0.0, 0.0, prev.a2_principal))
  pay a3_principal to party.a3_holders =
        min(remaining, inputs.a3_face - if(time.t == 0.0, 0.0, prev.a3_principal))
  pay a4_principal to party.a4_holders =
        min(remaining, inputs.a4_face - if(time.t == 0.0, 0.0, prev.a4_principal))
  pay b_principal to party.b_holders =
        min(remaining, inputs.b_face - if(time.t == 0.0, 0.0, prev.b_principal))
  pay c_principal to party.c_holders =
        min(remaining, inputs.c_face - if(time.t == 0.0, 0.0, prev.c_principal))
  pay d_principal to party.d_holders =
        min(remaining, inputs.d_face - if(time.t == 0.0, 0.0, prev.d_principal))
}
```

## Run configuration

```json
{"deterministic":{"annual_discount_rate":0.03}}
```

## Verified results

Checked period by period: **12 series** across **48 periods** — **496 values** in all, each within the tolerance shown.

- `notes.principal.a2_principal` — within ±11202.66
- `notes.principal.a3_principal` — within ±27137.0
- `notes.principal.a4_principal` — within ±8601.0
- `notes.principal.b_principal` — within ±2222.0
- `notes.principal.c_principal` — within ±1851.0
- `notes.principal.d_principal` — within ±1375.0
- `account.a2_principal` — within ±11202.66
- `account.a3_principal` — within ±27137.0
- `account.a4_principal` — within ±8601.0
- `account.b_principal` — within ±2222.0
- `account.c_principal` — within ±1851.0
- `account.d_principal` — within ±1375.0

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.total` | 580,114,574.55 | ±1 |
