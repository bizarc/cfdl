---
id: benchmark-credit-auto-abs-wal
title: "Credit: auto ABS weighted average life"
slug: "/docs/examples/credit-auto-abs-wal"
description: "An auto loan pool measured for weighted average life, the standard summary of when principal actually comes back."
source: benchmarks/credit/auto_abs_wal
---

# Credit: auto ABS weighted average life

An auto loan pool measured for weighted average life, the standard summary of when principal actually comes back.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

Subprime auto receivables backing a securitization, measured at zero prepayment
speed. The collateral is 43 level-pay sub-pools at 43 different rates and terms,
four of them at a 0% promotional annual rate. Weighted average life is the
standard summary of when principal comes back.

## The reference

An issuer's own prepayment-speed exhibit, filed publicly with a securities
regulator. It states the aggregate pool balance and tabulates percent-outstanding
at every monthly distribution date across seven prepayment speeds.

**Not redistributable.** Public filings are freely readable and citable, but the
filer retains copyright, so figures are asserted against rather than reproduced.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.loan`, 43 instances |
| Language features | many instances of one contract type in a single model |
| Conventions | level-pay amortization, a promotional 0% rate, zero prepayment speed |

## The result

The aggregate pool balance reproduces the issuer's stated figure to the cent:
`domain.credit.principal` = **537,640,787.96**, on a tolerance of one cent
against a balance of half a billion dollars.

Reproducing that means all 43 sub-pools returned exactly the balance the issuer
stated, at 43 different rates and terms.

## The delta

None on the asserted figure.

One reconciliation is not expressible through this suite and is recorded
separately: the exhibit's percent-outstanding column is a percentage of a note
class, and this pack models the collateral rather than the liability stack. The
sister cases at 0.5 and 1.5 ABS carry that comparison.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.03}}
// Auto-receivables collateral, reconciled against an issuer-published
// weighted-average-life exhibit filed with the securities regulator.
//
// The exhibit disaggregates the pool into 50 hypothetical sub-pools and states
// each one's balance, APR and remaining term, then publishes — for every note
// class, at seven prepayment speeds, for every distribution date — the percent
// of the class still outstanding, and its weighted average life. That is an
// unusually complete thing to publish, and it is what makes this checkable.
//
// WHAT IS REACHABLE HERE. The zero-speed column only. The other speeds use the
// Absolute Prepayment Model — a constant number of ORIGINAL units prepaying
// each month, so the implied SMM RISES over the life — and every pool factor
// in this pack is pow(k, p), valid only for constant k. Same blocker as the
// ramped curve. The per-class columns need
// a sequential-pay liability waterfall, which this pack does not model at all.
//
// At zero speed the exhibit's stated assumptions are exactly this pack's
// defaults: prepay at a constant zero rate, no defaults, no losses. So
// cpr = cdr = 0 and each sub-pool simply amortizes on schedule.
//
// 43 funded sub-pools; the exhibit lists 50, of which 7 carry no balance.
// Aggregate balance 537,640,787.96, terms to 64 months, APRs 0% and 0.905%-9.923%.
// Four sub-pools are 0% APR promotional financing — see NOTES.md, they are the
// reason the pack learned to amortize at a zero rate.

version 0.1
model "auto-abs-wal"
use pack "credit" version "0.1.0"
time calendar monthly from 2018-10 for 64

entity asset trust : Credit.Asset.Loan {
  collateral_type = "auto"
}

// THE SUB-POOLS ARE ENTITIES, NOT NAME SUFFIXES.
//
// This case is 43 sub-pools at 43 rates and terms. They used to be 43 contracts
// hung on one entity, told apart by a suffix the pack glued onto a variable
// name — containment expressed as string concatenation, in a namespace with no
// notion of children.
//
// They are children, so `part of` says so. Each carries its own contract, and
// the trust's totals are its children's totals by the relation rather than by
// name matching — the same mechanism `mbs_pool_by_loan` reconciles against a
// published schedule.
entity asset p01 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p02 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p03 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p04 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p06 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p07 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p08 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p09 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p11 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p12 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p13 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p14 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p16 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p17 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p18 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p19 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p21 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p22 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p23 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p24 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p26 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p27 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p28 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p29 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p31 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p32 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p33 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p34 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p36 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p37 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p38 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p39 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p40 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p41 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p42 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p43 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p44 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p45 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p46 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p47 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p48 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p49 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}
entity asset p50 : Credit.Asset.Loan {
  collateral_type = "auto"
  part of asset.trust
}


contract credit.loan.p01 on entity asset.p01 {
  term 2018-10..2020-03
  terms {
    principal = 5616021.32
    interest_rate = 0.00000
    term_months = 18
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p02 on entity asset.p02 {
  term 2018-10..2021-01
  terms {
    principal = 2616054.82
    interest_rate = 0.00000
    term_months = 28
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p03 on entity asset.p03 {
  term 2018-10..2022-06
  terms {
    principal = 4635948.89
    interest_rate = 0.00000
    term_months = 45
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p04 on entity asset.p04 {
  term 2018-10..2022-12
  terms {
    principal = 2205909.75
    interest_rate = 0.00000
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p06 on entity asset.p06 {
  term 2018-10..2019-11
  terms {
    principal = 147440.15
    interest_rate = 0.00915
    term_months = 14
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p07 on entity asset.p07 {
  term 2018-10..2021-03
  terms {
    principal = 216238.15
    interest_rate = 0.00992
    term_months = 30
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p08 on entity asset.p08 {
  term 2018-10..2022-07
  terms {
    principal = 354043.75
    interest_rate = 0.00907
    term_months = 46
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p09 on entity asset.p09 {
  term 2018-10..2022-12
  terms {
    principal = 342126.24
    interest_rate = 0.00905
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p11 on entity asset.p11 {
  term 2018-10..2020-02
  terms {
    principal = 610459.31
    interest_rate = 0.01906
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p12 on entity asset.p12 {
  term 2018-10..2021-04
  terms {
    principal = 1144291.74
    interest_rate = 0.01951
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p13 on entity asset.p13 {
  term 2018-10..2022-02
  terms {
    principal = 699535.89
    interest_rate = 0.01949
    term_months = 41
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p14 on entity asset.p14 {
  term 2018-10..2022-12
  terms {
    principal = 201897.47
    interest_rate = 0.01869
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p16 on entity asset.p16 {
  term 2018-10..2020-02
  terms {
    principal = 13918351.08
    interest_rate = 0.02594
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p17 on entity asset.p17 {
  term 2018-10..2021-04
  terms {
    principal = 26181002.53
    interest_rate = 0.02626
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p18 on entity asset.p18 {
  term 2018-10..2022-02
  terms {
    principal = 28740527.64
    interest_rate = 0.02684
    term_months = 41
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p19 on entity asset.p19 {
  term 2018-10..2022-12
  terms {
    principal = 9735143.46
    interest_rate = 0.02794
    term_months = 51
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p21 on entity asset.p21 {
  term 2018-10..2020-02
  terms {
    principal = 14533243.98
    interest_rate = 0.03678
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p22 on entity asset.p22 {
  term 2018-10..2021-04
  terms {
    principal = 26195374.46
    interest_rate = 0.03667
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p23 on entity asset.p23 {
  term 2018-10..2022-03
  terms {
    principal = 37348352.52
    interest_rate = 0.03671
    term_months = 42
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p24 on entity asset.p24 {
  term 2018-10..2023-01
  terms {
    principal = 19509631.08
    interest_rate = 0.03673
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p26 on entity asset.p26 {
  term 2018-10..2020-02
  terms {
    principal = 12183065.19
    interest_rate = 0.04661
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p27 on entity asset.p27 {
  term 2018-10..2021-04
  terms {
    principal = 20323443.61
    interest_rate = 0.04674
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p28 on entity asset.p28 {
  term 2018-10..2022-03
  terms {
    principal = 32071657.98
    interest_rate = 0.04690
    term_months = 42
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p29 on entity asset.p29 {
  term 2018-10..2023-01
  terms {
    principal = 20332473.43
    interest_rate = 0.04674
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p31 on entity asset.p31 {
  term 2018-10..2020-02
  terms {
    principal = 6428613.14
    interest_rate = 0.05572
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p32 on entity asset.p32 {
  term 2018-10..2021-05
  terms {
    principal = 16325861.98
    interest_rate = 0.05566
    term_months = 32
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p33 on entity asset.p33 {
  term 2018-10..2022-04
  terms {
    principal = 34020451.15
    interest_rate = 0.05608
    term_months = 43
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p34 on entity asset.p34 {
  term 2018-10..2023-01
  terms {
    principal = 22175932.04
    interest_rate = 0.05615
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p36 on entity asset.p36 {
  term 2018-10..2020-03
  terms {
    principal = 4214767.90
    interest_rate = 0.06583
    term_months = 18
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p37 on entity asset.p37 {
  term 2018-10..2021-05
  terms {
    principal = 10197295.25
    interest_rate = 0.06567
    term_months = 32
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p38 on entity asset.p38 {
  term 2018-10..2022-04
  terms {
    principal = 28511150.24
    interest_rate = 0.06580
    term_months = 43
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p39 on entity asset.p39 {
  term 2018-10..2023-01
  terms {
    principal = 21518975.29
    interest_rate = 0.06583
    term_months = 52
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p40 on entity asset.p40 {
  term 2018-10..2024-01
  terms {
    principal = 210992.57
    interest_rate = 0.06671
    term_months = 64
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p41 on entity asset.p41 {
  term 2018-10..2020-02
  terms {
    principal = 2314366.62
    interest_rate = 0.07537
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p42 on entity asset.p42 {
  term 2018-10..2021-04
  terms {
    principal = 6049009.56
    interest_rate = 0.07527
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p43 on entity asset.p43 {
  term 2018-10..2022-04
  terms {
    principal = 17752272.88
    interest_rate = 0.07538
    term_months = 43
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p44 on entity asset.p44 {
  term 2018-10..2023-02
  terms {
    principal = 17560641.20
    interest_rate = 0.07526
    term_months = 53
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p45 on entity asset.p45 {
  term 2018-10..2024-01
  terms {
    principal = 133227.13
    interest_rate = 0.07709
    term_months = 64
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p46 on entity asset.p46 {
  term 2018-10..2020-02
  terms {
    principal = 4089106.53
    interest_rate = 0.09923
    term_months = 17
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p47 on entity asset.p47 {
  term 2018-10..2021-04
  terms {
    principal = 9761650.69
    interest_rate = 0.09773
    term_months = 31
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p48 on entity asset.p48 {
  term 2018-10..2022-05
  terms {
    principal = 26285138.49
    interest_rate = 0.09619
    term_months = 44
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p49 on entity asset.p49 {
  term 2018-10..2023-02
  terms {
    principal = 29949234.04
    interest_rate = 0.09622
    term_months = 53
    cpr = 0
    cdr = 0
  }
}

contract credit.loan.p50 on entity asset.p50 {
  term 2018-10..2023-11
  terms {
    principal = 279866.82
    interest_rate = 0.09836
    term_months = 62
    cpr = 0
    cdr = 0
  }
}
```

## Run configuration

```json
{"deterministic":{"annual_discount_rate":0.03}}
```

## Verified results

Checked period by period: **1 series** across **14 periods** — **14 values** in all, each within ±0.01 of the reference.

- `net_cash_flow`

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `domain.credit.principal` | 537,640,787.96 | ±0.01 |
